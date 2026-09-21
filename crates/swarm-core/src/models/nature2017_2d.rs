//! # 🎯 [实战关卡 3] 2017 Nature Communications 经典 2D 模型
//!
//! 论文: "Oscillators that sync and swarm"
//! 本地 PDF: papers/2d-plane/01-sync-and-swarm-natcomm2017/paper.pdf
//!
//! 在这里你将学习并练习：
//! - 粒子坐标切片拆分 (`let (x, rest) = s.split_at(...)`)
//! - $O(N^2)$ 两两粒子相互作用循环的 Rust 实现
//! - 周期性角度归一化 (`wrap_to_pi`)
//!
//! 遇到卡壳可查阅参考答案: `crates/swarm-core/src/reference/nature2017_2d.rs`

#![allow(unused_variables, dead_code)]

use crate::metrics::{kuramoto_order_parameter, radius_of_gyration_2d};
use crate::types::{wrap_to_pi, DynamicalSystem, MetricsSnapshot};
use rand::Rng;
use rand_distr::{Distribution, Uniform};

/// 2D Swarmalator 动力学模型
#[derive(Clone, Debug)]
pub struct Swarmalator2D {
    pub n: usize,
    /// 空间同相吸引强度 J
    pub j: f64,
    /// 相位同步耦合强度 K
    pub k: f64,
    /// 固有自然频率 omega_i
    pub omega: Vec<f64>,
    /// 防止距离为 0 的平滑小量 epsilon
    pub eps: f64,
}

impl Swarmalator2D {
    pub fn new(n: usize, j: f64, k: f64) -> Self {
        Self {
            n,
            j,
            k,
            omega: vec![0.0; n],
            eps: 1e-5,
        }
    }

    pub fn with_frequencies(mut self, omega: Vec<f64>) -> Self {
        assert_eq!(omega.len(), self.n);
        self.omega = omega;
        self
    }

    /// 随机初始化初态: x, y in [-1, 1], theta in [-pi, pi)
    pub fn random_initial_state<R: Rng>(&self, rng: &mut R) -> Vec<f64> {
        let pos_dist = Uniform::new(-1.0, 1.0);
        let phase_dist = Uniform::new(-std::f64::consts::PI, std::f64::consts::PI);
        let mut state = vec![0.0; 3 * self.n];

        for i in 0..self.n {
            state[i] = pos_dist.sample(rng);
            state[self.n + i] = pos_dist.sample(rng);
            state[2 * self.n + i] = phase_dist.sample(rng);
        }
        state
    }
}

impl DynamicalSystem for Swarmalator2D {
    fn dimension(&self) -> usize {
        3 * self.n
    }

    fn num_agents(&self) -> usize {
        self.n
    }

    /// ## 物理控制微分方程 (RHS):
    /// $$\dot{\mathbf{x}}_i = \frac{1}{N} \sum_{j \neq i} \left[ \frac{\mathbf{x}_j - \mathbf{x}_i}{|\mathbf{x}_j - \mathbf{x}_i|} (1 + J \cos(\theta_j - \theta_i)) - \frac{\mathbf{x}_j - \mathbf{x}_i}{|\mathbf{x}_j - \mathbf{x}_i|^2} \right]$$
    /// $$\dot{\theta}_i = \omega_i + \frac{K}{N} \sum_{j \neq i} \frac{\sin(\theta_j - \theta_i)}{|\mathbf{x}_j - \mathbf{x}_i|}$$
    fn derivative(&self, s: &[f64], ds: &mut [f64]) {
        // TODO: 请实现 2D Swarmalator 的相互作用微分方程导数计算
        //
        // 步骤提示:
        // 1. 从状态数组 s 中提取 x, y, theta:
        //    let n = self.n;
        //    let x = &s[0..n];
        //    let y = &s[n..2*n];
        //    let theta = &s[2*n..3*n];
        //
        // 2. 双重循环计算粒子 i 受到所有粒子 j 的合力:
        //    for i in 0..n {
        //        let mut sum_fx = 0.0;
        //        let mut sum_fy = 0.0;
        //        let mut sum_fth = 0.0;
        //        for j in 0..n {
        //            if i == j { continue; }
        //            // 计算 dx_ij, dy_ij, dth_ij, dist...
        //        }
        //        // 写回 ds:
        //        ds[i] = (1.0 / n as f64) * sum_fx;
        //        ds[n + i] = (1.0 / n as f64) * sum_fy;
        //        ds[2*n + i] = self.omega[i] + (1.0 / n as f64) * sum_fth;
        //    }
        todo!("【实战关卡 3】请在此处实现 2D Swarmalator 的运动微分方程");
    }

    fn metrics(&self, s: &[f64]) -> MetricsSnapshot {
        let n = self.n;
        let x = &s[0..n];
        let y = &s[n..2 * n];
        let theta = &s[2 * n..3 * n];

        let r = kuramoto_order_parameter(theta);
        let r_gyr = radius_of_gyration_2d(x, y);

        MetricsSnapshot {
            t: 0.0,
            order_r: r,
            order_s: Some(r_gyr),
            order_s_plus: None,
            order_s_minus: None,
        }
    }

    fn post_step(&self, s: &mut [f64]) {
        let n = self.n;
        for i in (2 * n)..(3 * n) {
            s[i] = wrap_to_pi(s[i]);
        }
    }
}
