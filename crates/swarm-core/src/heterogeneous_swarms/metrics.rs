#![allow(unused_variables, dead_code, unused_imports)]

//! # 统计与序参量计算模块 (Metrics & Order Parameters)
//!
//! 论文: *Emergence of Specialised Collective Behaviors in Evolving Heterogeneous Swarms* (PPSN 2024 / Springer)
//!
//! 核心指标:
//! 1. 瞬时光强均值 l_t 与累积适应度 f (公式 1):
//!    $$f = \frac{\sum_{t=0}^T l_t}{G_{\max} \cdot T}, \quad l_t = \frac{1}{N} \sum_{n=1}^N G_n$$
//! 2. 群体运动对齐序参量 \Phi (公式 2):
//!    $$\Phi = \frac{1}{N} \sum_{n=1}^N \varphi_n, \quad \varphi_n = \frac{\| \sum_{p=1}^P e^{j \theta_p} + e^{j \theta_n} \|}{P + 1}$$
//!    其中 P 为机器人 n 在感知半径 (2.0m) 内所感知到的邻居集合；\Phi \in [0, 1]，趋于 1 说明高度一致协同运动；
//! 3. 子群专门化行为解耦指标:
//!    - 绿色子群光强 l_{t, green} 与红色子群光强 l_{t, red}；
//!    - 绿色子群航向序参量 \Phi_{green} 与红色子群航向序参量 \Phi_{red}；
//!    - 群体质心距目标中心距离与回转半径 R_g。

use crate::heterogeneous_swarms::robot::{Robot, SubGroup};
use serde::{Deserialize, Serialize};

/// 宏观统计快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmMetricsSnapshot {
    /// 当前步数与时间
    pub step: usize,
    pub time: f64,
    /// 全群体即时光强均值 l_t
    pub mean_intensity: f64,
    /// 绿色子群即时光强均值
    pub green_intensity: f64,
    /// 红色子群即时光强均值
    pub red_intensity: f64,
    /// 全群体对齐序参量 \Phi
    pub swarm_order: f64,
    /// 绿色子群对齐序参量
    pub green_order: f64,
    /// 红色子群对齐序参量
    pub red_order: f64,
    /// 绿色机器人数量占比
    pub green_ratio: f64,
    /// 群体质心到竞技场中心的欧氏距离
    pub center_distance: f64,
    /// 群体回转半径 (分散度)
    pub gyration_radius: f64,
}

/// 计算即时全群体光强均值 l_t \in [0.0, 255.0]
#[inline]
pub fn compute_mean_intensity(intensities: &[f64]) -> f64 {
    if intensities.is_empty() {
        return 0.0;
    }
    intensities.iter().sum::<f64>() / (intensities.len() as f64)
}

/// 【关卡 12 - 任务 4】计算群体运动对齐序参量 \Phi (公式 2)
///
/// \varphi_n = \| \sum_{p \in \mathcal{N}_n} e^{j \theta_p} + e^{j \theta_n} \| / (P_n + 1)
/// \Phi = (1 / N) \sum_{n=1}^N \varphi_n
///
/// # 提示
/// - 若机器人列表为空，直接返回 0.0；
/// - 对每个机器人 i，初值包含自身的航向向量：`cos_sum = cos(theta_i), sin_sum = sin(theta_i), count = 1`；
/// - 遍历所有其他机器人 j，若距离平方 $\le r_{range}^2$，则累加其航向向量分量并自增 count；
/// - 计算局部向量模长除以 count：`phi_i = sqrt(cos_sum^2 + sin_sum^2) / count`；
/// - 返回全群体平均值 `total_phi / N`；
/// - 若卡壳可参考 [`crates/swarm-core/src/reference/heterogeneous_swarms.rs`](../reference/heterogeneous_swarms.rs)。
pub fn compute_swarm_order(robots: &[Robot], perception_range: f64) -> f64 {
    todo!("【关卡 12 - 任务 4】在 metrics.rs 中实现群体运动对齐序参量 compute_swarm_order");
}

/// 计算特定子群内部的对齐序参量
pub fn compute_subgroup_order(robots: &[Robot], target_subgroup: SubGroup, perception_range: f64) -> f64 {
    let filtered: Vec<&Robot> = robots.iter().filter(|r| r.subgroup == target_subgroup).collect();
    let n = filtered.len();
    if n == 0 {
        return 0.0;
    }

    let mut total_phi = 0.0;
    let r_sq = perception_range * perception_range;

    for i in 0..n {
        let mut cos_sum = filtered[i].heading.cos();
        let mut sin_sum = filtered[i].heading.sin();
        let mut count = 1;

        let pos_i = filtered[i].position;

        for j in 0..n {
            if i == j {
                continue;
            }
            let dx = filtered[j].position[0] - pos_i[0];
            let dy = filtered[j].position[1] - pos_i[1];
            if dx * dx + dy * dy <= r_sq {
                cos_sum += filtered[j].heading.cos();
                sin_sum += filtered[j].heading.sin();
                count += 1;
            }
        }

        let mag = (cos_sum * cos_sum + sin_sum * sin_sum).sqrt();
        total_phi += mag / (count as f64);
    }

    total_phi / (n as f64)
}

/// 计算群体质心与回转半径
pub fn compute_spatial_stats(robots: &[Robot], center: [f64; 2]) -> (f64, f64) {
    let n = robots.len();
    if n == 0 {
        return (0.0, 0.0);
    }

    let mut mean_x = 0.0;
    let mut mean_y = 0.0;

    for r in robots {
        mean_x += r.position[0];
        mean_y += r.position[1];
    }
    mean_x /= n as f64;
    mean_y /= n as f64;

    let center_dist = ((mean_x - center[0]).powi(2) + (mean_y - center[1]).powi(2)).sqrt();

    let mut var_dist = 0.0;
    for r in robots {
        let dx = r.position[0] - mean_x;
        let dy = r.position[1] - mean_y;
        var_dist += dx * dx + dy * dy;
    }
    let rg = (var_dist / (n as f64)).sqrt();

    (center_dist, rg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_swarm_order_identical() {
        // 所有机器人同向朝东 (heading = 0.0)，序参量应严格为 1.0
        let bots = vec![
            Robot::new(0, [15.0, 15.0], 0.0, SubGroup::Green),
            Robot::new(1, [15.5, 15.0], 0.0, SubGroup::Green),
            Robot::new(2, [16.0, 15.0], 0.0, SubGroup::Red),
        ];

        let order = compute_swarm_order(&bots, 2.0);
        assert!((order - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_compute_swarm_order_orthogonal() {
        // 机器人互相垂直，序参量应小于 1.0
        let bots = vec![
            Robot::new(0, [15.0, 15.0], 0.0, SubGroup::Green),
            Robot::new(1, [15.5, 15.0], std::f64::consts::PI / 2.0, SubGroup::Red),
        ];

        let order = compute_swarm_order(&bots, 2.0);
        // (1 + i) / 2 的模长为 sqrt(2)/2 ≈ 0.7071
        assert!((order - (2.0f64.sqrt() / 2.0)).abs() < 1e-3);
    }
}
