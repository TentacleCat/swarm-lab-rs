//! # 群体仿真系统与实验驱动器 (Swarm Simulator & Experiment Runner)
//!
//! 论文: *Emergence of Specialised Collective Behaviors in Evolving Heterogeneous Swarms* (PPSN 2024 / Springer)
//!
//! 仿真系统核心设计:
//! 1. 物理环境与初值生成:
//!    - 在距离目标中心 r 处的圆周上随机选取一点，以 3m x 3m 矩形包围盒初始化 N 台机器人；
//!    - 机器人朝向在 [-PI, PI) 均匀随机；
//! 2. 仿真主循环 (10Hz 采样与控制周期，dt = 0.1s):
//!    - (可选) 在线自适应调控机制: 每隔 5.0s 根据局域光强以概率 P_green 动态重抽样表型；
//!    - 4 象限方位与邻居距离、相对航向及标量光强感知；
//!    - 神经网络前向推理目标差速动作 [v, w] \in [-1, 1]^2；
//!    - 差速小车运动学步进与弹性硬核排斥、边界约束；
//!    - 统计采样光强均值 l_t、群体对齐序参量 \Phi(t) 与子群宏观指标；
//! 3. 评测协议 (Evaluation Protocol):
//!    - 适应度计算严格对应公式 1: f = (\sum l_t) / (255 * T)；
//!    - 支持 N_repeats 次独立评测并取中位数。

use crate::heterogeneous_swarms::controller::{HeterogeneousGenotype, Reservoir, ReservoirNN};
use crate::heterogeneous_swarms::environment::{ArenaType, Environment};
use crate::heterogeneous_swarms::metrics::{
    compute_mean_intensity, compute_spatial_stats, compute_subgroup_order, compute_swarm_order,
    SwarmMetricsSnapshot,
};
use crate::heterogeneous_swarms::regulatory::{update_swarm_phenotypes, RegulatoryConfig};
use crate::heterogeneous_swarms::robot::{resolve_swarm_collisions, Robot, SubGroup};
use crate::heterogeneous_swarms::sensors::{compute_robot_sensor_inputs, SensorConfig};
use rand::distributions::{Distribution, Uniform};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// 群体仿真参数配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmSimConfig {
    /// 竞技场类型 (Center, Bimodal, Linear, Banana)
    pub arena_type: ArenaType,
    /// 机器人总数 (论文默认为 20，可拓展至 10, 50)
    pub swarm_size: usize,
    /// 固定子群比例 (Green 数量, Red 数量)
    pub ratio: (usize, usize),
    /// 距离目标中心的初始生成距离 (米, 论文默认为 12.0m)
    pub spawn_distance: f64,
    /// 生成包围盒尺寸 (米, 论文为 3.0m)
    pub spawn_box_size: f64,
    /// 仿真时长 (秒, 论文正式实验为 600s, 评测验证可灵活配置)
    pub simulation_time: f64,
    /// 物理仿真步长 (秒, 论文控制频率为 10Hz, dt = 0.1s)
    pub dt: f64,
    /// 感知半径 (米, 2.0m)
    pub perception_range: f64,
    /// 是否开启在线表型可塑性调控机制
    pub regulatory_enabled: bool,
    /// 调控周期 (秒, 5.0s)
    pub regulatory_interval: f64,
}

impl Default for SwarmSimConfig {
    fn default() -> Self {
        Self {
            arena_type: ArenaType::Center,
            swarm_size: 20,
            ratio: (10, 10),
            spawn_distance: 12.0,
            spawn_box_size: 3.0,
            simulation_time: 120.0, // 默认快速测试 120s，正式运行可设置为 600s
            dt: 0.1,
            perception_range: 2.0,
            regulatory_enabled: false,
            regulatory_interval: 5.0,
        }
    }
}

/// 仿真运行产生的结果记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// 全局综合适应度 f \in [0.0, 1.0]
    pub fitness: f64,
    /// 最终时刻群体平均光强
    pub final_mean_intensity: f64,
    /// 最终时刻群体对齐序参量
    pub final_order: f64,
    /// 最终时刻群体质心距目标中心距离
    pub final_center_dist: f64,
    /// 全时程宏观指标时序记录 (可选采样)
    pub metrics_history: Vec<SwarmMetricsSnapshot>,
    /// 最终机器人位置与航向快照
    pub final_robots: Vec<Robot>,
}

/// 群体仿真引擎
pub struct SwarmSimulator {
    pub config: SwarmSimConfig,
    pub env: Environment,
    pub sensor_config: SensorConfig,
    pub regulatory_config: RegulatoryConfig,
    pub reservoir_green: Reservoir,
    pub reservoir_red: Reservoir,
}

impl SwarmSimulator {
    /// 根据配置与固定的两个隐层储备池构建仿真器
    pub fn new(config: SwarmSimConfig, res_green: Reservoir, res_red: Reservoir) -> Self {
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

    /// 执行单次完整仿真测试
    ///
    /// `genotype`: 异构全基因型 (前 18 维赋给 Green 控制器，后 18 维赋给 Red 控制器)
    /// `seed`: 仿真随机种子 (用于初始位置、朝向与调控状态采样)
    /// `record_history`: 是否详细记录时序指标
    pub fn run_trial(
        &self,
        genotype: &HeterogeneousGenotype,
        seed: u64,
        record_history: bool,
    ) -> SimulationResult {
        let mut rng = StdRng::seed_from_u64(seed);

        // 1. 初始化两套神经网络控制器
        let nn_green = ReservoirNN::new(self.reservoir_green.clone(), genotype.sub1_weights());
        let nn_red = ReservoirNN::new(self.reservoir_red.clone(), genotype.sub2_weights());

        // 2. 初始化机器人位置与子群分配
        let mut robots = self.spawn_robots(&mut rng);
        let n_bots = robots.len();

        let total_steps = (self.config.simulation_time / self.config.dt).round() as usize;
        let reg_step_interval = (self.regulatory_config.update_interval / self.config.dt).round() as usize;

        let mut cumulative_intensity_sum = 0.0;
        let mut metrics_history = Vec::new();

        // 3. 仿真时间推进循环
        for step in 0..total_steps {
            let sim_time = (step as f64) * self.config.dt;

            // 当前所有机器人的位置与朝向切片
            let positions: Vec<[f64; 2]> = robots.iter().map(|r| r.position).collect();
            let headings: Vec<f64> = robots.iter().map(|r| r.heading).collect();

            // 采样各车当前局域标量光强
            let current_intensities: Vec<f64> =
                positions.iter().map(|&p| self.env.sample_intensity(p)).collect();

            // 步骤 A: 在线表型可塑性调控更新 (如果开启且处于调控触发步)
            if self.config.regulatory_enabled && step > 0 && step % reg_step_interval == 0 {
                update_swarm_phenotypes(
                    &mut robots,
                    &current_intensities,
                    &self.regulatory_config,
                    &mut rng,
                );
            }

            // 步骤 B: 传感器感知与神经网络推理
            let mut actions = Vec::with_capacity(n_bots);
            for i in 0..n_bots {
                let inputs = compute_robot_sensor_inputs(
                    i,
                    &positions,
                    &headings,
                    &self.env,
                    &self.sensor_config,
                );

                let act = match robots[i].subgroup {
                    SubGroup::Green => nn_green.forward(&inputs),
                    SubGroup::Red => nn_red.forward(&inputs),
                };
                actions.push(act);
            }

            // 步骤 C: 差速运动学物理推进
            for i in 0..n_bots {
                robots[i].step_motion(actions[i], self.config.dt);
            }

            // 步骤 D: 碰撞与边界约束消解
            resolve_swarm_collisions(&mut robots, &self.env);

            // 步骤 E: 统计瞬时指标
            let mean_l = compute_mean_intensity(&current_intensities);
            cumulative_intensity_sum += mean_l;

            // 记录时序指标 (每 10 步记录一次或按需)
            if record_history && (step % 10 == 0 || step == total_steps - 1) {
                let swarm_order = compute_swarm_order(&robots, self.config.perception_range);
                let green_order = compute_subgroup_order(&robots, SubGroup::Green, self.config.perception_range);
                let red_order = compute_subgroup_order(&robots, SubGroup::Red, self.config.perception_range);

                let mut green_int_sum = 0.0;
                let mut green_cnt = 0;
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
                let (center_dist, rg) = compute_spatial_stats(&robots, self.env.center);

                metrics_history.push(SwarmMetricsSnapshot {
                    step,
                    time: sim_time,
                    mean_intensity: mean_l,
                    green_intensity: if green_cnt > 0 { green_int_sum / (green_cnt as f64) } else { 0.0 },
                    red_intensity: if red_cnt > 0 { red_int_sum / (red_cnt as f64) } else { 0.0 },
                    swarm_order,
                    green_order,
                    red_order,
                    green_ratio,
                    center_distance: center_dist,
                    gyration_radius: rg,
                });
            }
        }

        // 最终适应度计算 (公式 1): f = (\sum l_t) / (G_max * T_steps)
        let trial_fitness = cumulative_intensity_sum / (self.env.g_max * (total_steps as f64));

        let final_positions: Vec<[f64; 2]> = robots.iter().map(|r| r.position).collect();
        let final_intensities: Vec<f64> = final_positions.iter().map(|&p| self.env.sample_intensity(p)).collect();
        let final_mean_intensity = compute_mean_intensity(&final_intensities);
        let final_order = compute_swarm_order(&robots, self.config.perception_range);
        let (final_center_dist, _) = compute_spatial_stats(&robots, self.env.center);

        SimulationResult {
            fitness: trial_fitness,
            final_mean_intensity,
            final_order,
            final_center_dist,
            metrics_history,
            final_robots: robots,
        }
    }

    /// 在指定距离的 3m x 3m 包围盒内生成初始机器人集群
    fn spawn_robots<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec<Robot> {
        let n = self.config.swarm_size;
        let mut robots = Vec::with_capacity(n);

        // 随机选择一个初始生成方位角 theta_spawn \in [0, 2*PI)
        let spawn_angle: f64 = rng.gen_range(0.0..(2.0 * PI));
        let box_center_x = self.env.center[0] + self.config.spawn_distance * spawn_angle.cos();
        let box_center_y = self.env.center[1] + self.config.spawn_distance * spawn_angle.sin();

        let half_box = self.config.spawn_box_size / 2.0;
        let dist_x = Uniform::new_inclusive(box_center_x - half_box, box_center_x + half_box);
        let dist_y = Uniform::new_inclusive(box_center_y - half_box, box_center_y + half_box);
        let dist_heading = Uniform::new_inclusive(-PI, PI);

        // 子群比例分配: 前 green_target 个为 Green，其余为 Red
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

            robots.push(Robot::new(i, [x, y], th, subgroup));
        }

        // 消除生成时的初始重叠
        resolve_swarm_collisions(&mut robots, &self.env);

        robots
    }

    /// 多次评测并取中位数适应度 (评估鲁棒性与消除随机性运气)
    pub fn evaluate_median(
        &self,
        genotype: &HeterogeneousGenotype,
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
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_short_simulation_run() {
        let mut rng = StdRng::seed_from_u64(42);
        let res_g = Reservoir::new_random(&mut rng);
        let res_r = Reservoir::new_random(&mut rng);

        let mut config = SwarmSimConfig::default();
        config.simulation_time = 2.0; // 快速运行 2 秒
        config.swarm_size = 6;
        config.ratio = (3, 3);

        let sim = SwarmSimulator::new(config, res_g, res_r);
        let geno = HeterogeneousGenotype::random_uniform(&mut rng);

        let result = sim.run_trial(&geno, 1001, true);

        assert!(result.fitness >= 0.0 && result.fitness <= 1.0);
        assert_eq!(result.final_robots.len(), 6);
        assert!(!result.metrics_history.is_empty());
    }
}
