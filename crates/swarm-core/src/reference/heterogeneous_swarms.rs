//! # 💡 [参考答案区] 异构群体演化与表型可塑性标准参考实现 (Reference Implementation)
//!
//! 论文: *Emergence of Specialised Collective Behaviors in Evolving Heterogeneous Swarms* (PPSN 2024 / Springer)
//!
//! 本文件包含经过完备单测验证的标准理论计算与基准实现，供闯关学习者对比查阅。

use std::f64::consts::PI;
use rand::distributions::{Distribution, Uniform};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

pub use crate::heterogeneous_swarms::cma_es::{CmaEsConfig, CmaEsOptimizer};
pub use crate::heterogeneous_swarms::environment::{ArenaType, Environment};
pub use crate::heterogeneous_swarms::metrics::SwarmMetricsSnapshot;
pub use crate::heterogeneous_swarms::regulatory::RegulatoryConfig;
pub use crate::heterogeneous_swarms::robot::{Robot, SubGroup};
pub use crate::heterogeneous_swarms::sensors::{Quadrant, SensorConfig};
pub use crate::heterogeneous_swarms::simulator::{SimulationResult, SwarmSimConfig};

pub type ArenaTypeReference = ArenaType;
pub type CmaEsConfigReference = CmaEsConfig;
pub type CmaEsOptimizerReference = CmaEsOptimizer;
pub type SwarmSimConfigReference = SwarmSimConfig;
pub type SimulationResultReference = SimulationResult;
pub type RobotReference = Robot;
pub type SubGroupReference = SubGroup;

/// 将任意角度严格折叠至 [-PI, PI)
#[inline]
pub fn ref_wrap_to_pi(mut angle: f64) -> f64 {
    while angle >= PI {
        angle -= 2.0 * PI;
    }
    while angle < -PI {
        angle += 2.0 * PI;
    }
    angle
}

/// 4 象限分类标准参考实现
#[inline]
pub fn ref_bearing_to_quadrant(bearing: f64) -> Quadrant {
    let b = ref_wrap_to_pi(bearing);
    let pi_4 = PI / 4.0;
    let pi_3_4 = 3.0 * PI / 4.0;

    if b >= -pi_4 && b < pi_4 {
        Quadrant::Front
    } else if b >= pi_4 && b < pi_3_4 {
        Quadrant::Right
    } else if b >= -pi_3_4 && b < -pi_4 {
        Quadrant::Left
    } else {
        Quadrant::Back
    }
}

/// 在线表型可塑性切换概率函数标准参考实现 (公式 P_green)
#[inline]
pub fn ref_prob_green(light: f64, config: &RegulatoryConfig) -> f64 {
    if light > config.high_threshold {
        1.00
    } else if light > config.low_threshold {
        0.75
    } else {
        0.50
    }
}

/// 储备池神经网络前向推理标准参考实现
///
/// RNN = tanh( W_out * ReLU( W_h2 * ReLU( W_h1 * s_in ) ) )
pub fn ref_rnn_forward(
    s_in: &[f64; 9],
    w_h1: &[[f64; 9]; 9],
    w_h2: &[[f64; 9]; 9],
    w_out: &[[f64; 9]; 2],
) -> [f64; 2] {
    let mut h1 = [0.0; 9];
    for i in 0..9 {
        let mut sum = 0.0;
        for j in 0..9 {
            sum += w_h1[i][j] * s_in[j];
        }
        h1[i] = sum.max(0.0);
    }

    let mut h2 = [0.0; 9];
    for i in 0..9 {
        let mut sum = 0.0;
        for j in 0..9 {
            sum += w_h2[i][j] * h1[j];
        }
        h2[i] = sum.max(0.0);
    }

    let mut out = [0.0; 2];
    for i in 0..2 {
        let mut sum = 0.0;
        for j in 0..9 {
            sum += w_out[i][j] * h2[j];
        }
        out[i] = sum.tanh();
    }

    out
}

/// 群体运动对齐序参量标准参考实现 (公式 2)
pub fn ref_compute_swarm_order(
    positions: &[[f64; 2]],
    headings: &[f64],
    perception_range: f64,
) -> f64 {
    let n = positions.len();
    if n == 0 {
        return 0.0;
    }

    let r_sq = perception_range * perception_range;
    let mut total_phi = 0.0;

    for i in 0..n {
        let mut cos_sum = headings[i].cos();
        let mut sin_sum = headings[i].sin();
        let mut count = 1;

        let p_i = positions[i];
        for j in 0..n {
            if i == j {
                continue;
            }
            let dx = positions[j][0] - p_i[0];
            let dy = positions[j][1] - p_i[1];
            if dx * dx + dy * dy <= r_sq {
                cos_sum += headings[j].cos();
                sin_sum += headings[j].sin();
                count += 1;
            }
        }

        let mag = (cos_sum * cos_sum + sin_sum * sin_sum).sqrt();
        total_phi += mag / (count as f64);
    }

    total_phi / (n as f64)
}

/// 群体时程光强累积适应度标准参考实现 (公式 1)
pub fn ref_compute_fitness(mean_intensities: &[f64], g_max: f64) -> f64 {
    if mean_intensities.is_empty() {
        return 0.0;
    }
    let sum_l: f64 = mean_intensities.iter().sum();
    sum_l / (g_max * (mean_intensities.len() as f64))
}

/// 计算单个机器人的 9 维受限感知向量标准参考实现
pub fn ref_compute_robot_sensor_inputs(
    robot_idx: usize,
    positions: &[[f64; 2]],
    headings: &[f64],
    env: &Environment,
    config: &SensorConfig,
) -> [f64; 9] {
    let my_pos = positions[robot_idx];
    let my_heading = headings[robot_idx];

    let mut nearest = [(config.default_range, 0.0); 4];

    for (j, &other_pos) in positions.iter().enumerate() {
        if j == robot_idx {
            continue;
        }

        let dx = other_pos[0] - my_pos[0];
        let dy = other_pos[1] - my_pos[1];
        let dist = (dx * dx + dy * dy).sqrt();

        if dist <= config.max_range {
            let global_bearing = dy.atan2(dx);
            let rel_bearing = ref_wrap_to_pi(global_bearing - my_heading);
            let quad = ref_bearing_to_quadrant(rel_bearing) as usize;

            if dist < nearest[quad].0 {
                let other_heading = headings[j];
                let rel_heading = ref_wrap_to_pi(other_heading - my_heading);
                nearest[quad] = (dist, rel_heading);
            }
        }
    }

    let mut inputs = [0.0; 9];
    for q in 0..4 {
        let (d, th) = nearest[q];
        let d_norm = (d / config.max_range) * 2.0 - 1.0;
        let th_norm = th / PI;

        inputs[q * 2] = d_norm;
        inputs[q * 2 + 1] = th_norm;
    }

    inputs[8] = env.normalized_intensity(my_pos);
    inputs
}

/// 机器人硬核碰撞消解与竞技场边界约束标准参考实现
pub fn ref_resolve_swarm_collisions(robots: &mut [Robot], env: &Environment) {
    let n = robots.len();

    for i in 0..n {
        for j in (i + 1)..n {
            let dx = robots[j].position[0] - robots[i].position[0];
            let dy = robots[j].position[1] - robots[i].position[1];
            let dist_sq = dx * dx + dy * dy;
            let min_dist = robots[i].radius + robots[j].radius;

            if dist_sq < min_dist * min_dist && dist_sq > 1e-8 {
                let dist = dist_sq.sqrt();
                let overlap = min_dist - dist;
                let nx = dx / dist;
                let ny = dy / dist;

                robots[i].position[0] -= 0.5 * overlap * nx;
                robots[i].position[1] -= 0.5 * overlap * ny;
                robots[j].position[0] += 0.5 * overlap * nx;
                robots[j].position[1] += 0.5 * overlap * ny;
            }
        }
    }

    for robot in robots.iter_mut() {
        let r = robot.radius;
        if robot.position[0] < r {
            robot.position[0] = r;
            if robot.heading.cos() < 0.0 {
                robot.heading = ref_wrap_to_pi(PI - robot.heading);
            }
        } else if robot.position[0] > env.width - r {
            robot.position[0] = env.width - r;
            if robot.heading.cos() > 0.0 {
                robot.heading = ref_wrap_to_pi(PI - robot.heading);
            }
        }

        if robot.position[1] < r {
            robot.position[1] = r;
            if robot.heading.sin() < 0.0 {
                robot.heading = ref_wrap_to_pi(-robot.heading);
            }
        } else if robot.position[1] > env.height - r {
            robot.position[1] = env.height - r;
            if robot.heading.sin() > 0.0 {
                robot.heading = ref_wrap_to_pi(-robot.heading);
            }
        }
    }
}

/// 冻结隐层储备池结构体标准参考实现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservoirReference {
    pub w_h1: [[f64; 9]; 9],
    pub w_h2: [[f64; 9]; 9],
}

impl ReservoirReference {
    pub fn new_random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let dist = Uniform::new_inclusive(-1.0, 1.0);
        let mut w_h1 = [[0.0; 9]; 9];
        let mut w_h2 = [[0.0; 9]; 9];

        for i in 0..9 {
            for j in 0..9 {
                w_h1[i][j] = dist.sample(rng);
                w_h2[i][j] = dist.sample(rng);
            }
        }

        Self { w_h1, w_h2 }
    }

    pub fn forward_hidden(&self, input: &[f64; 9]) -> [f64; 9] {
        let mut h1 = [0.0; 9];
        for i in 0..9 {
            let mut sum = 0.0;
            for j in 0..9 {
                sum += self.w_h1[i][j] * input[j];
            }
            h1[i] = sum.max(0.0);
        }

        let mut h2 = [0.0; 9];
        for i in 0..9 {
            let mut sum = 0.0;
            for j in 0..9 {
                sum += self.w_h2[i][j] * h1[j];
            }
            h2[i] = sum.max(0.0);
        }

        h2
    }
}

/// 异构群体全基因型标准参考实现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeterogeneousGenotypeReference {
    pub sub1: [f64; 18],
    pub sub2: [f64; 18],
}

impl HeterogeneousGenotypeReference {
    pub fn new(sub1: [f64; 18], sub2: [f64; 18]) -> Self {
        Self { sub1, sub2 }
    }

    pub fn from_flat_slice(slice: &[f64]) -> Self {
        assert_eq!(slice.len(), 36);
        let mut sub1 = [0.0; 18];
        let mut sub2 = [0.0; 18];
        sub1.copy_from_slice(&slice[0..18]);
        sub2.copy_from_slice(&slice[18..36]);
        Self { sub1, sub2 }
    }

    pub fn sub1_weights(&self) -> &[f64; 18] {
        &self.sub1
    }

    pub fn sub2_weights(&self) -> &[f64; 18] {
        &self.sub2
    }

    pub fn random_uniform<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let dist = Uniform::new_inclusive(-5.0, 5.0);
        let mut sub1 = [0.0; 18];
        let mut sub2 = [0.0; 18];
        for v in sub1.iter_mut() {
            *v = dist.sample(rng);
        }
        for v in sub2.iter_mut() {
            *v = dist.sample(rng);
        }
        Self { sub1, sub2 }
    }
}

/// 异构群体多智能体仿真引擎标准参考实现
pub struct SwarmSimulatorReference {
    pub config: SwarmSimConfig,
    pub env: Environment,
    pub sensor_config: SensorConfig,
    pub regulatory_config: RegulatoryConfig,
    pub reservoir_green: ReservoirReference,
    pub reservoir_red: ReservoirReference,
}

impl SwarmSimulatorReference {
    pub fn new(
        config: SwarmSimConfig,
        res_green: ReservoirReference,
        res_red: ReservoirReference,
    ) -> Self {
        let env = Environment::new(config.arena_type);
        let sensor_config = SensorConfig {
            max_range: config.perception_range,
            default_range: 2.01,
        };
        let regulatory_config = RegulatoryConfig {
            update_interval: config.regulatory_interval,
            high_threshold: 229.0,
            low_threshold: 76.0,
        };

        Self {
            config,
            env,
            sensor_config,
            regulatory_config,
            reservoir_green: res_green,
            reservoir_red: res_red,
        }
    }

    pub fn run_trial(
        &self,
        genotype: &HeterogeneousGenotypeReference,
        seed: u64,
        record_history: bool,
    ) -> SimulationResult {
        let mut rng = StdRng::seed_from_u64(seed);

        let mut w_out_green = [[0.0; 9]; 2];
        for i in 0..2 {
            for j in 0..9 {
                w_out_green[i][j] = genotype.sub1[i * 9 + j];
            }
        }
        let mut w_out_red = [[0.0; 9]; 2];
        for i in 0..2 {
            for j in 0..9 {
                w_out_red[i][j] = genotype.sub2[i * 9 + j];
            }
        }

        let mut robots = self.spawn_robots(&mut rng);
        let n_bots = robots.len();

        let total_steps = (self.config.simulation_time / self.config.dt).round() as usize;
        let reg_step_interval =
            (self.regulatory_config.update_interval / self.config.dt).round() as usize;

        let mut cumulative_intensity_sum = 0.0;
        let mut metrics_history = Vec::new();

        for step in 0..total_steps {
            let sim_time = (step as f64) * self.config.dt;

            let positions: Vec<[f64; 2]> = robots.iter().map(|r| r.position).collect();
            let headings: Vec<f64> = robots.iter().map(|r| r.heading).collect();

            let current_intensities: Vec<f64> =
                positions.iter().map(|&p| self.env.sample_intensity(p)).collect();

            // 在线调控
            if self.config.regulatory_enabled && step > 0 && step % reg_step_interval == 0 {
                for (robot, &light) in robots.iter_mut().zip(current_intensities.iter()) {
                    let p = ref_prob_green(light, &self.regulatory_config);
                    let roll: f64 = rng.gen();
                    robot.subgroup = if roll < p {
                        SubGroup::Green
                    } else {
                        SubGroup::Red
                    };
                }
            }

            // 传感器与前向推理
            let mut actions = Vec::with_capacity(n_bots);
            for i in 0..n_bots {
                let inputs = ref_compute_robot_sensor_inputs(
                    i,
                    &positions,
                    &headings,
                    &self.env,
                    &self.sensor_config,
                );

                let act = match robots[i].subgroup {
                    SubGroup::Green => ref_rnn_forward(
                        &inputs,
                        &self.reservoir_green.w_h1,
                        &self.reservoir_green.w_h2,
                        &w_out_green,
                    ),
                    SubGroup::Red => ref_rnn_forward(
                        &inputs,
                        &self.reservoir_red.w_h1,
                        &self.reservoir_red.w_h2,
                        &w_out_red,
                    ),
                };
                actions.push(act);
            }

            // 运动推进
            for i in 0..n_bots {
                let v_norm = actions[i][0].clamp(-1.0, 1.0);
                let w_norm = actions[i][1].clamp(-1.0, 1.0);
                let v = v_norm * robots[i].max_speed;
                let w = w_norm * robots[i].max_angular_speed;

                robots[i].heading = ref_wrap_to_pi(robots[i].heading + w * self.config.dt);
                robots[i].position[0] += v * robots[i].heading.cos() * self.config.dt;
                robots[i].position[1] += v * robots[i].heading.sin() * self.config.dt;
            }

            ref_resolve_swarm_collisions(&mut robots, &self.env);

            let mean_l = if current_intensities.is_empty() {
                0.0
            } else {
                current_intensities.iter().sum::<f64>() / (current_intensities.len() as f64)
            };
            cumulative_intensity_sum += mean_l;

            if record_history && (step % 10 == 0 || step == total_steps - 1) {
                let swarm_order =
                    ref_compute_swarm_order(&positions, &headings, self.config.perception_range);

                let mut green_cnt = 0;
                let mut green_int_sum = 0.0;
                let mut red_int_sum = 0.0;
                let mut red_cnt = 0;

                for (r, &val) in robots.iter().zip(current_intensities.iter()) {
                    if r.subgroup == SubGroup::Green {
                        green_int_sum += val;
                        green_cnt += 1;
                    } else {
                        red_int_sum += val;
                        red_cnt += 1;
                    }
                }

                let green_ratio = (green_cnt as f64) / (n_bots as f64);

                let mut mean_x = 0.0;
                let mut mean_y = 0.0;
                for p in &positions {
                    mean_x += p[0];
                    mean_y += p[1];
                }
                mean_x /= n_bots as f64;
                mean_y /= n_bots as f64;
                let center_dist =
                    ((mean_x - self.env.center[0]).powi(2) + (mean_y - self.env.center[1]).powi(2))
                        .sqrt();

                let mut var_dist = 0.0;
                for p in &positions {
                    let dx = p[0] - mean_x;
                    let dy = p[1] - mean_y;
                    var_dist += dx * dx + dy * dy;
                }
                let rg = (var_dist / (n_bots as f64)).sqrt();

                metrics_history.push(SwarmMetricsSnapshot {
                    step,
                    time: sim_time,
                    mean_intensity: mean_l,
                    green_intensity: if green_cnt > 0 {
                        green_int_sum / (green_cnt as f64)
                    } else {
                        0.0
                    },
                    red_intensity: if red_cnt > 0 {
                        red_int_sum / (red_cnt as f64)
                    } else {
                        0.0
                    },
                    swarm_order,
                    green_order: 0.0,
                    red_order: 0.0,
                    green_ratio,
                    center_distance: center_dist,
                    gyration_radius: rg,
                });
            }
        }

        let trial_fitness = cumulative_intensity_sum / (self.env.g_max * (total_steps as f64));

        let final_positions: Vec<[f64; 2]> = robots.iter().map(|r| r.position).collect();
        let final_headings: Vec<f64> = robots.iter().map(|r| r.heading).collect();
        let final_intensities: Vec<f64> =
            final_positions.iter().map(|&p| self.env.sample_intensity(p)).collect();
        let final_mean_intensity = if final_intensities.is_empty() {
            0.0
        } else {
            final_intensities.iter().sum::<f64>() / (final_intensities.len() as f64)
        };
        let final_order =
            ref_compute_swarm_order(&final_positions, &final_headings, self.config.perception_range);

        let mut mean_x = 0.0;
        let mut mean_y = 0.0;
        for p in &final_positions {
            mean_x += p[0];
            mean_y += p[1];
        }
        mean_x /= n_bots as f64;
        mean_y /= n_bots as f64;
        let final_center_dist =
            ((mean_x - self.env.center[0]).powi(2) + (mean_y - self.env.center[1]).powi(2)).sqrt();

        SimulationResult {
            fitness: trial_fitness,
            final_mean_intensity,
            final_order,
            final_center_dist,
            metrics_history,
            final_robots: robots,
        }
    }

    fn spawn_robots<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec<Robot> {
        let n = self.config.swarm_size;
        let mut robots = Vec::with_capacity(n);

        let spawn_angle: f64 = rng.gen_range(0.0..(2.0 * PI));
        let box_center_x = self.env.center[0] + self.config.spawn_distance * spawn_angle.cos();
        let box_center_y = self.env.center[1] + self.config.spawn_distance * spawn_angle.sin();

        let half_box = self.config.spawn_box_size / 2.0;
        let dist_x = Uniform::new_inclusive(box_center_x - half_box, box_center_x + half_box);
        let dist_y = Uniform::new_inclusive(box_center_y - half_box, box_center_y + half_box);
        let dist_heading = Uniform::new_inclusive(-PI, PI);

        let green_target = self.config.ratio.0;

        for i in 0..n {
            let x = dist_x.sample(rng).clamp(0.2, self.env.width - 0.2);
            let y = dist_y.sample(rng).clamp(0.2, self.env.height - 0.2);
            let th = dist_heading.sample(rng);

            let subgroup = if i < green_target {
                SubGroup::Green
            } else {
                SubGroup::Red
            };

            let max_speed = 0.14;
            let wheel_base = 0.085;
            let max_angular_speed = (2.0 * max_speed) / wheel_base;

            robots.push(Robot {
                id: i,
                position: [x, y],
                heading: ref_wrap_to_pi(th),
                subgroup,
                radius: 0.08,
                max_speed,
                max_angular_speed,
            });
        }

        ref_resolve_swarm_collisions(&mut robots, &self.env);
        robots
    }

    pub fn evaluate_median(
        &self,
        genotype: &HeterogeneousGenotypeReference,
        base_seed: u64,
        repeats: usize,
    ) -> f64 {
        let mut fits = Vec::with_capacity(repeats);
        for rep in 0..repeats {
            let seed = base_seed.wrapping_add((rep as u64) * 10007);
            let res = self.run_trial(genotype, seed, false);
            fits.push(res.fitness);
        }
        fits.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        fits[repeats / 2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ref_prob_green() {
        let cfg = RegulatoryConfig::default();
        assert_eq!(ref_prob_green(240.0, &cfg), 1.0);
        assert_eq!(ref_prob_green(100.0, &cfg), 0.75);
        assert_eq!(ref_prob_green(50.0, &cfg), 0.50);
    }

    #[test]
    fn test_ref_order_parameter() {
        let pos = vec![[0.0, 0.0], [0.5, 0.0], [1.0, 0.0]];
        let heads = vec![0.0, 0.0, 0.0];
        let order = ref_compute_swarm_order(&pos, &heads, 2.0);
        assert!((order - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_ref_bearing_quadrant() {
        assert_eq!(ref_bearing_to_quadrant(0.0), Quadrant::Front);
        assert_eq!(ref_bearing_to_quadrant(PI / 2.0), Quadrant::Right);
        assert_eq!(ref_bearing_to_quadrant(PI), Quadrant::Back);
        assert_eq!(ref_bearing_to_quadrant(-PI / 2.0), Quadrant::Left);
    }

    #[test]
    fn test_ref_rnn_forward() {
        let s_in = [1.0; 9];
        let w_h1 = [[0.1; 9]; 9];
        let w_h2 = [[0.1; 9]; 9];
        let w_out = [[0.1; 9]; 2];
        let out = ref_rnn_forward(&s_in, &w_h1, &w_h2, &w_out);
        assert!(out[0].abs() <= 1.0);
        assert!(out[1].abs() <= 1.0);
    }

    #[test]
    fn test_ref_simulation_short() {
        let mut rng = StdRng::seed_from_u64(42);
        let res_g = ReservoirReference::new_random(&mut rng);
        let res_r = ReservoirReference::new_random(&mut rng);

        let mut config = SwarmSimConfig::default();
        config.simulation_time = 1.0;
        config.swarm_size = 4;
        config.ratio = (2, 2);

        let sim = SwarmSimulatorReference::new(config, res_g, res_r);
        let geno = HeterogeneousGenotypeReference::random_uniform(&mut rng);

        let res = sim.run_trial(&geno, 1001, true);
        assert!(res.fitness >= 0.0 && res.fitness <= 1.0);
        assert_eq!(res.final_robots.len(), 4);
    }
}
