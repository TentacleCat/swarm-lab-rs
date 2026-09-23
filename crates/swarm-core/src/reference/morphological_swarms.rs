//! # 💡 [参考答案区] 形态计算与自对齐聚集标准参考实现 (Reference Implementation)
//!
//! 论文: *Aggregating swarms through morphology handling design contingencies: from the sweet spot to a rich expressivity*
//! (Jeremy Fersula, Nicolas Bredeche, Olivier Dauchot, arXiv:2601.07610, Jan 2026)
//!
//! 本文件包含经过完备单测验证的标准理论计算与基准实现，供闯关学习者对比查阅。

use rand::distributions::Distribution;
use rand::Rng;
use rand_distr::StandardNormal;
pub use crate::morphological_swarms::metrics::SwarmMetrics;
pub use crate::morphological_swarms::model::{MorphologyType, Particle, SwarmParams, WCA_CUTOFF};

pub type MorphologyTypeReference = MorphologyType;
pub type SwarmParamsReference = SwarmParams;
pub type ParticleReference = Particle;
pub type SwarmMetricsReference = SwarmMetrics;

/// 周期性边界下的最小镜像位移向量标准参考实现
#[inline]
pub fn ref_minimum_image_displacement(
    r_i: [f64; 2],
    r_j: [f64; 2],
    half_l: f64,
) -> (f64, f64) {
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

/// 周期性坐标缠绕映射标准参考实现
#[inline]
pub fn ref_wrap_coordinate(mut coord: f64, half_l: f64) -> f64 {
    let box_len = 2.0 * half_l;
    while coord > half_l {
        coord -= box_len;
    }
    while coord < -half_l {
        coord += box_len;
    }
    coord
}

/// 宏观极化对齐度标准参考实现
pub fn ref_polar_alignment(particles: &[Particle]) -> f64 {
    let n = particles.len();
    if n == 0 {
        return 0.0;
    }

    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    for p in particles {
        sum_x += p.mu[0];
        sum_y += p.mu[1];
    }

    (sum_x * sum_x + sum_y * sum_y).sqrt() / (n as f64)
}

/// 光照区机器人占比标准参考实现
pub fn ref_light_occupancy_ratio(particles: &[Particle]) -> f64 {
    if particles.is_empty() {
        return 0.0;
    }
    let in_count = particles.iter().filter(|p| p.in_light).count();
    in_count as f64 / particles.len() as f64
}

/// 形态学自对齐群体模拟系统标准参考实现
#[derive(Debug, Clone)]
pub struct MorphologicalSwarmReference {
    pub params: SwarmParams,
    pub particles: Vec<Particle>,
    pub current_time: f64,
    pub step_count: usize,
}

impl MorphologicalSwarmReference {
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
                    let (dx, dy) = ref_minimum_image_displacement(pos, other.pos, l);
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

        swarm.relax_overlaps(500);
        swarm
    }

    pub fn relax_overlaps(&mut self, steps: usize) {
        let dt = 0.005;
        let l = self.params.half_box_l;
        let cut_sq = (self.params.sigma * WCA_CUTOFF).powi(2);

        for _ in 0..steps {
            let n = self.particles.len();
            let mut forces = vec![[0.0, 0.0]; n];

            for i in 0..n {
                for j in (i + 1)..n {
                    let (dx, dy) = ref_minimum_image_displacement(
                        self.particles[i].pos,
                        self.particles[j].pos,
                        l,
                    );
                    let r_sq = dx * dx + dy * dy;
                    if r_sq < cut_sq && r_sq > 1e-12 {
                        let r = r_sq.sqrt();
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
                p.pos[0] = ref_wrap_coordinate(p.pos[0] + f[0] * dt, l);
                p.pos[1] = ref_wrap_coordinate(p.pos[1] + f[1] * dt, l);
            }
        }
    }

    /// 计算所有粒子受到的 WCA 软排斥力总和标准参考实现
    pub fn compute_wca_forces(&self) -> Vec<[f64; 2]> {
        let n = self.particles.len();
        let mut forces = vec![[0.0, 0.0]; n];
        let l = self.params.half_box_l;
        let sig = self.params.sigma;
        let eps = self.params.epsilon_lj;
        let cut_sq = (sig * WCA_CUTOFF).powi(2);

        for i in 0..n {
            for j in (i + 1)..n {
                let (dx, dy) = ref_minimum_image_displacement(
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

    /// 单步数值积分演化标准参考实现
    pub fn step<R: Rng>(&mut self, rng: &mut R) {
        let dt = self.params.dt;
        let l = self.params.half_box_l;
        let r_light_sq = self.params.light_radius * self.params.light_radius;
        let kappa = self.params.epsilon_over_tau_n;
        let noise_std = (2.0 * self.params.d_rot * dt).sqrt();
        let tau_v = self.params.tau_v;

        let forces = self.compute_wca_forces();

        for (i, p) in self.particles.iter_mut().enumerate() {
            let f = forces[i];
            p.force = f;

            let r_sq = p.pos[0] * p.pos[0] + p.pos[1] * p.pos[1];
            p.in_light = r_sq < r_light_sq;
            let v_a = if p.in_light {
                self.params.v_light
            } else {
                self.params.v_dark
            };

            // 速度弛豫更新
            p.vel[0] += dt * (v_a * p.mu[0] - p.vel[0] + f[0]) / tau_v;
            p.vel[1] += dt * (v_a * p.mu[1] - p.vel[1] + f[1]) / tau_v;

            // 形态学力矩更新
            let mux = kappa * (p.mu[1] * p.mu[1] * p.vel[0] - p.mu[0] * p.mu[1] * p.vel[1]);
            let muy = kappa * (p.mu[0] * p.mu[0] * p.vel[1] - p.mu[0] * p.mu[1] * p.vel[0]);

            let next_mu_x = p.mu[0] + dt * mux;
            let next_mu_y = p.mu[1] + dt * muy;

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

            p.pos[0] = ref_wrap_coordinate(p.pos[0] + p.vel[0] * dt, l);
            p.pos[1] = ref_wrap_coordinate(p.pos[1] + p.vel[1] * dt, l);
        }

        self.current_time += dt;
        self.step_count += 1;
    }

    pub fn step_n<R: Rng>(&mut self, n_steps: usize, rng: &mut R) {
        for _ in 0..n_steps {
            self.step(rng);
        }
    }

    pub fn contact_cluster_statistics(&self) -> (usize, usize) {
        let n = self.particles.len();
        if n == 0 {
            return (0, 0);
        }

        let l = self.params.half_box_l;
        let contact_dist_sq = (self.params.sigma * WCA_CUTOFF).powi(2);

        let mut adj = vec![Vec::new(); n];
        let mut contact_pairs = 0;

        for i in 0..n {
            for j in (i + 1)..n {
                let (dx, dy) = ref_minimum_image_displacement(
                    self.particles[i].pos,
                    self.particles[j].pos,
                    l,
                );
                if dx * dx + dy * dy < contact_dist_sq {
                    adj[i].push(j);
                    adj[j].push(i);
                    contact_pairs += 1;
                }
            }
        }

        let mut visited = vec![false; n];
        let mut max_cluster_size = 1;

        for i in 0..n {
            if !visited[i] {
                let mut size = 0;
                let mut queue = std::collections::VecDeque::new();
                queue.push_back(i);
                visited[i] = true;

                while let Some(node) = queue.pop_front() {
                    size += 1;
                    for &neighbor in &adj[node] {
                        if !visited[neighbor] {
                            visited[neighbor] = true;
                            queue.push_back(neighbor);
                        }
                    }
                }

                if size > max_cluster_size {
                    max_cluster_size = size;
                }
            }
        }

        (contact_pairs, max_cluster_size)
    }

    pub fn evaluate_metrics(&self) -> SwarmMetrics {
        let (contact_pairs, max_cluster_size) = self.contact_cluster_statistics();
        let n = self.particles.len();
        let mean_speed = if n == 0 {
            0.0
        } else {
            let total: f64 = self.particles.iter().map(|p| p.speed()).sum();
            total / (n as f64)
        };

        SwarmMetrics {
            light_ratio: ref_light_occupancy_ratio(&self.particles),
            polar_alignment: ref_polar_alignment(&self.particles),
            contact_pairs,
            max_cluster_size,
            mean_speed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_ref_periodic_boundary() {
        let half_l = 10.0;
        let (dx, dy) = ref_minimum_image_displacement([9.0, 0.0], [-9.0, 0.0], half_l);
        assert!((dx - 2.0).abs() < 1e-10);
        assert!(dy.abs() < 1e-10);

        assert!((ref_wrap_coordinate(11.5, half_l) - (-8.5)).abs() < 1e-10);
        assert!((ref_wrap_coordinate(-12.0, half_l) - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_ref_wca_forces() {
        let mut params = SwarmParams::default();
        params.n_particles = 2;
        params.half_box_l = 10.0;

        let mut p1 = Particle::new([0.0, 0.0], 0.0);
        let mut p2 = Particle::new([0.8, 0.0], 0.0);
        p1.pos = [0.0, 0.0];
        p2.pos = [0.8, 0.0];

        let swarm = MorphologicalSwarmReference {
            params,
            particles: vec![p1, p2],
            current_time: 0.0,
            step_count: 0,
        };

        let forces = swarm.compute_wca_forces();
        assert!(forces[0][0] < 0.0);
        assert!(forces[1][0] > 0.0);
        assert!((forces[0][0] + forces[1][0]).abs() < 1e-10);
    }

    #[test]
    fn test_ref_self_alignment_torque() {
        let mut p = Particle::new([0.0, 0.0], 0.0);
        p.vel = [p.mu[0] - 5.0, p.mu[1] + 1.0];

        let kappa_align = 1.0;
        let muy_align = kappa_align * (p.mu[0] * p.mu[0] * p.vel[1] - p.mu[0] * p.mu[1] * p.vel[0]);

        let kappa_front = -1.0;
        let muy_front = kappa_front * (p.mu[0] * p.mu[0] * p.vel[1] - p.mu[0] * p.mu[1] * p.vel[0]);

        assert!(muy_align > 0.0);
        assert!(muy_front < 0.0);
    }

    #[test]
    fn test_ref_simulation_step() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut params = SwarmParams::default();
        params.n_particles = 10;
        params.half_box_l = 8.0;

        let mut swarm = MorphologicalSwarmReference::new_random(params, &mut rng);
        for _ in 0..50 {
            swarm.step(&mut rng);
        }

        let m = swarm.evaluate_metrics();
        assert!(m.light_ratio >= 0.0 && m.light_ratio <= 1.0);
        assert!(m.polar_alignment >= 0.0 && m.polar_alignment <= 1.0);
    }
}
