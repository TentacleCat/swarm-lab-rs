//! # Morphological Swarm Model (arXiv:2601.07610)
//!
//! 基于 2026 年最新论文：
//! *Aggregating swarms through morphology handling design contingencies: from the sweet spot to a rich expressivity*
//! (Jeremy Fersula, Nicolas Bredeche, Olivier Dauchot)
//!
//! 核心动力学特性：
//! - $N=64$ 个具有自主推进与形态力学自对齐 (Self-Alignment) 的 Kilobot 机器人在周期性边界二维盒子中运动；
//! - 外部碰撞受力采用软核 Weeks-Chandler-Andersen (WCA) 截断 Lennard-Jones 排斥势；
//! - 中心圆形光照区域（面积占比 $\sigma \approx 6\%$），机器人在光照区执行减速策略（$v_\circ / v_\bullet = 1/3$）；
//! - 形态学力学自对齐力矩：
//!   $$\tau_n \frac{d\vec{n}_i}{dt} = \epsilon (\vec{n}_i \times \vec{v}_i) \times \vec{n}_i + \sqrt{2D} \xi_i \vec{n}_{i, \perp}$$
//!   - $\epsilon / \tau_n < 0$ (Fronter / 前向突刺形态)：受力后机身朝向接触点转动，碰撞中实际运动速度大幅衰减，在光照区触发 MIPS 成核聚集；
//!   - $\epsilon / \tau_n > 0$ (Aligner / 顺应形态)：受力后朝向外力同向转动，碰撞后倾向于平行滑行，产生宏观极化游弋 (Flocking)，无法在光照区聚集；
//!   - $\epsilon / \tau_n = 0$ (ABP / 经典主动布朗粒子)：无形态力矩，纯随机探索。

use rand::distributions::Distribution;
use rand::Rng;
use rand_distr::StandardNormal;
use serde::{Deserialize, Serialize};

/// WCA 软排斥势截断半径 $2^{1/6} \sigma \approx 1.122462048$
pub const WCA_CUTOFF: f64 = 1.122_462_048_309_373;

/// 机器人形态分类
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MorphologyType {
    /// Fronter (前向支撑脚在后、重心偏后，外力使其反向逆对齐并锁定碰撞)
    Fronter,
    /// Aligner (前向单刚性脚在前、重心偏前，外力使其顺对齐并平行滑开)
    Aligner,
    /// ABP (标准主动布朗粒子，无外力扭矩对齐响应)
    ActiveBrownian,
    /// 自定义无量纲参数 $\kappa = \epsilon / \tau_n$
    Custom(f64),
}

impl MorphologyType {
    /// 获取无量纲对齐强度与符号 $\kappa = \epsilon / \tau_n$
    pub fn epsilon_over_tau_n(&self) -> f64 {
        match self {
            Self::Fronter => -1.0,
            Self::Aligner => 1.0,
            Self::ActiveBrownian => 0.0,
            Self::Custom(val) => *val,
        }
    }
}

/// 仿真物理参数配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmParams {
    /// 机器人数量 (论文标准值: 64)
    pub n_particles: usize,
    /// 盒子半长 (论文标准值: L = 13.85, 边长 27.7)
    pub half_box_l: f64,
    /// 中心光照区域半径 (论文标准值: 3.83, 面积占比 6%)
    pub light_radius: f64,
    /// 时间步长 dt (论文标准值: 0.001)
    pub dt: f64,
    /// 速度弛豫时间 tau_v (论文标准值: 0.001)
    pub tau_v: f64,
    /// 转向力矩特征参数 $\kappa = \epsilon / \tau_n \in [-5.0, 5.0]$
    pub epsilon_over_tau_n: f64,
    /// 角度扩散系数 D (论文标准值: 0.01)
    pub d_rot: f64,
    /// 黑暗区域自主推进速度 v_bullet (标准值: 1.0)
    pub v_dark: f64,
    /// 光照区域自主推进速度 v_circ (标准值: 1/3 ~ 0.33)
    pub v_light: f64,
    /// WCA 软排斥特征能量 epsilon_lj (标准值: 1.0)
    pub epsilon_lj: f64,
    /// WCA 粒子等效直径 sigma (标准值: 1.0)
    pub sigma: f64,
}

impl Default for SwarmParams {
    fn default() -> Self {
        Self {
            n_particles: 64,
            half_box_l: 13.85,
            light_radius: 3.83,
            dt: 0.001,
            tau_v: 0.001,
            epsilon_over_tau_n: -1.0, // 默认 Fronter 处于甜点区
            d_rot: 0.01,
            v_dark: 1.0,
            v_light: 1.0 / 3.0,
            epsilon_lj: 1.0,
            sigma: 1.0,
        }
    }
}

/// 单个机器人质点状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Particle {
    /// 空间坐标 (x, y)，范围 [-L, L]
    pub pos: [f64; 2],
    /// 瞬时速度 (vx, vy)
    pub vel: [f64; 2],
    /// 机身主动朝向单位向量 (mu_x, mu_y)
    pub mu: [f64; 2],
    /// 上一步受到的外部接触力 (fx, fy)
    pub force: [f64; 2],
    /// 是否处于光照区域
    pub in_light: bool,
}

impl Particle {
    /// 创建处于原点、随机朝向的机器人
    pub fn new(pos: [f64; 2], theta: f64) -> Self {
        let mu = [theta.cos(), theta.sin()];
        Self {
            pos,
            vel: mu,
            mu,
            force: [0.0, 0.0],
            in_light: false,
        }
    }

    /// 获取当前朝向方位角 $\theta \in [-\pi, \pi]$
    pub fn heading_angle(&self) -> f64 {
        self.mu[1].atan2(self.mu[0])
    }

    /// 瞬时速度大小
    pub fn speed(&self) -> f64 {
        (self.vel[0].powi(2) + self.vel[1].powi(2)).sqrt()
    }
}

/// 形态学自对齐群体模拟系统
#[derive(Debug, Clone)]
pub struct MorphologicalSwarm {
    /// 仿真参数
    pub params: SwarmParams,
    /// 机器人粒子列表
    pub particles: Vec<Particle>,
    /// 当前已演化总时间
    pub current_time: f64,
    /// 当前已完成步数
    pub step_count: usize,
}

impl MorphologicalSwarm {
    /// 初始化系统，并在无初始重叠的情况下随机布设粒子
    pub fn new_random<R: Rng>(params: SwarmParams, rng: &mut R) -> Self {
        let mut particles: Vec<Particle> = Vec::with_capacity(params.n_particles);
        let l = params.half_box_l;
        let min_sep_sq = (params.sigma * 0.9).powi(2);

        for _ in 0..params.n_particles {
            let mut attempts = 0;
            let mut pos = [0.0, 0.0];
            while attempts < 1000 {
                pos = [
                    rng.gen_range(-l * 0.95..l * 0.95),
                    rng.gen_range(-l * 0.95..l * 0.95),
                ];
                let mut overlap = false;
                for other in &particles {
                    let (dx, dy) = Self::minimum_image_displacement(pos, other.pos, l);
                    if dx * dx + dy * dy < min_sep_sq {
                        overlap = true;
                        break;
                    }
                }
                if !overlap {
                    break;
                }
                attempts += 1;
            }

            let theta = rng.gen_range(0.0..std::f64::consts::TAU);
            let mut p = Particle::new(pos, theta);
            let r_sq = pos[0] * pos[0] + pos[1] * pos[1];
            p.in_light = r_sq < params.light_radius * params.light_radius;
            particles.push(p);
        }

        let mut swarm = Self {
            params,
            particles,
            current_time: 0.0,
            step_count: 0,
        };

        // 像论文一样，先进行短暂的预松弛以完全消除重叠
        swarm.relax_overlaps(500);
        swarm
    }

    /// 周期性边界下的最小镜像位移向量 (r_j - r_i)
    #[inline]
    pub fn minimum_image_displacement(r_i: [f64; 2], r_j: [f64; 2], half_l: f64) -> (f64, f64) {
        let box_len = 2.0 * half_l;
        let mut dx = r_j[0] - r_i[0];
        let mut dy = r_j[1] - r_i[1];

        if dx > half_l {
            dx -= box_len;
        } else if dx < -half_l {
            dx += box_len;
        }

        if dy > half_l {
            dy -= box_len;
        } else if dy < -half_l {
            dy += box_len;
        }

        (dx, dy)
    }

    /// 将坐标周期性缠绕映射回 [-L, L]
    #[inline]
    pub fn wrap_coordinate(mut coord: f64, half_l: f64) -> f64 {
        let box_len = 2.0 * half_l;
        while coord > half_l {
            coord -= box_len;
        }
        while coord < -half_l {
            coord += box_len;
        }
        coord
    }

    /// 预松弛：利用软排斥消除初始可能存在的重叠
    pub fn relax_overlaps(&mut self, steps: usize) {
        let dt = 0.005;
        let l = self.params.half_box_l;
        let cut_sq = (self.params.sigma * WCA_CUTOFF).powi(2);

        for _ in 0..steps {
            let n = self.particles.len();
            let mut forces = vec![[0.0, 0.0]; n];

            for i in 0..n {
                for j in (i + 1)..n {
                    let (dx, dy) = Self::minimum_image_displacement(
                        self.particles[i].pos,
                        self.particles[j].pos,
                        l,
                    );
                    let r_sq = dx * dx + dy * dy;
                    if r_sq < cut_sq && r_sq > 1e-12 {
                        let r = r_sq.sqrt();
                        // 软排斥力随侵入线性排斥
                        let f_mag = 10.0 * (1.0 - r / (self.params.sigma * WCA_CUTOFF));
                        let fx = f_mag * (dx / r);
                        let fy = f_mag * (dy / r);

                        forces[i][0] -= fx;
                        forces[i][1] -= fy;
                        forces[j][0] += fx;
                        forces[j][1] += fy;
                    }
                }
            }

            for (p, f) in self.particles.iter_mut().zip(forces.iter()) {
                p.pos[0] = Self::wrap_coordinate(p.pos[0] + f[0] * dt, l);
                p.pos[1] = Self::wrap_coordinate(p.pos[1] + f[1] * dt, l);
            }
        }
    }

    /// 计算所有粒子受到的 WCA 软排斥力总和
    pub fn compute_wca_forces(&self) -> Vec<[f64; 2]> {
        let n = self.particles.len();
        let mut forces = vec![[0.0, 0.0]; n];
        let l = self.params.half_box_l;
        let sig = self.params.sigma;
        let eps = self.params.epsilon_lj;
        let cut_sq = (sig * WCA_CUTOFF).powi(2);

        for i in 0..n {
            for j in (i + 1)..n {
                let (dx, dy) = Self::minimum_image_displacement(
                    self.particles[i].pos,
                    self.particles[j].pos,
                    l,
                );
                let r_sq = dx * dx + dy * dy;
                if r_sq < cut_sq && r_sq > 1e-12 {
                    let r2_inv = 1.0 / r_sq;
                    let sig2_r2 = (sig * sig) * r2_inv;
                    let sig6_r6 = sig2_r2 * sig2_r2 * sig2_r2;
                    let sig12_r12 = sig6_r6 * sig6_r6;

                    // WCA 力的解析标量系数: 48 * eps / r^2 * ( (sig/r)^12 - 0.5 * (sig/r)^6 )
                    // 排斥力指向远离 j 的方向: F_ij = F_mag * (r_i - r_j) = -F_mag * dr
                    let f_coeff = 48.0 * eps * r2_inv * (sig12_r12 - 0.5 * sig6_r6);
                    let fx = f_coeff * (-dx);
                    let fy = f_coeff * (-dy);

                    forces[i][0] += fx;
                    forces[i][1] += fy;
                    forces[j][0] -= fx;
                    forces[j][1] -= fy;
                }
            }
        }

        forces
    }

    /// 单步数值积分演化 (严格对应 LAMMPS 插件 fix_spp.cpp / fix_evolution.cpp)
    pub fn step<R: Rng>(&mut self, rng: &mut R) {
        let dt = self.params.dt;
        let l = self.params.half_box_l;
        let r_light_sq = self.params.light_radius * self.params.light_radius;
        let kappa = self.params.epsilon_over_tau_n;
        let noise_std = (2.0 * self.params.d_rot * dt).sqrt();
        let tau_v = self.params.tau_v;

        // 1. 计算两两粒子间 WCA 碰撞斥力
        let forces = self.compute_wca_forces();

        // 2. 遍历每个粒子更新状态
        for (i, p) in self.particles.iter_mut().enumerate() {
            let f = forces[i];
            p.force = f;

            // 判断是否在光照区并选取推进速度 v_a
            let r_sq = p.pos[0] * p.pos[0] + p.pos[1] * p.pos[1];
            p.in_light = r_sq < r_light_sq;
            let v_a = if p.in_light {
                self.params.v_light
            } else {
                self.params.v_dark
            };

            // 速度更新 (动量弛豫方程):
            // dv/dt = (v_a * mu - v + f) / tau_v
            p.vel[0] += dt * (v_a * p.mu[0] - p.vel[0] + f[0]) / tau_v;
            p.vel[1] += dt * (v_a * p.mu[1] - p.vel[1] + f[1]) / tau_v;

            // 形态学力矩与朝向更新 (论文核心方程与 fix_spp.cpp 源码实现):
            // mux = kappa * (mu_y^2 * vx - mu_x * mu_y * vy)
            // muy = kappa * (mu_x^2 * vy - mu_x * mu_y * vx)
            let mux = kappa * (p.mu[1] * p.mu[1] * p.vel[0] - p.mu[0] * p.mu[1] * p.vel[1]);
            let muy = kappa * (p.mu[0] * p.mu[0] * p.vel[1] - p.mu[0] * p.mu[1] * p.vel[0]);

            let next_mu_x = p.mu[0] + dt * mux;
            let next_mu_y = p.mu[1] + dt * muy;

            // 高斯角扩散噪声
            let norm_sample: f64 = StandardNormal.sample(rng);
            let ang_noise = norm_sample * noise_std;
            let cos_noise = ang_noise.cos();
            let sin_noise = ang_noise.sin();

            let rot_x = cos_noise * next_mu_x - sin_noise * next_mu_y;
            let rot_y = sin_noise * next_mu_x + cos_noise * next_mu_y;

            let mag = (rot_x * rot_x + rot_y * rot_y).sqrt();
            if mag > 1e-12 {
                p.mu[0] = rot_x / mag;
                p.mu[1] = rot_y / mag;
            }

            // 空间位置更新与周期性边界包裹
            p.pos[0] = Self::wrap_coordinate(p.pos[0] + p.vel[0] * dt, l);
            p.pos[1] = Self::wrap_coordinate(p.pos[1] + p.vel[1] * dt, l);
        }

        self.current_time += dt;
        self.step_count += 1;
    }

    /// 演化指定时间步长
    pub fn step_n<R: Rng>(&mut self, n_steps: usize, rng: &mut R) {
        for _ in 0..n_steps {
            self.step(rng);
        }
    }
}
