#![allow(unused_variables, dead_code, unused_imports)]

//! # 在线自适应调控机制 (Online Regulatory Mechanism / Phenotypic Plasticity)
//!
//! 论文: *Emergence of Specialised Collective Behaviors in Evolving Heterogeneous Swarms* (PPSN 2024 / Springer)
//!
//! 根据论文第 3 节与第 4 节 (Online Regulatory Mechanism):
//! 1. 模拟自然界中信息素诱导的“表型可塑性” (Phenotypic Plasticity)；
//! 2. 启发式概率有限状态机: 机器人基于纯局域感知到的即时标量光强，动态调整表达为 Subgroup 1 (Green) 的概率；
//! 3. 概率切换方程:
//!    $$P_{\text{green}}(\text{light}) = \begin{cases}
//!        1.00 & \text{if } \text{light} > 229 \\
//!        0.75 & \text{if } 76 < \text{light} \le 229 \\
//!        0.50 & \text{if } \text{light} \le 76
//!    \end{cases}$$
//! 4. 调控更新周期: 每隔 \tau = 5.0s (参数网格搜索所得最优稳定频率) 重新采样一次表达状态；
//! 5. 整个过程 100% 去中心化、无记忆、无机器人间通信，而在宏观群体尺度自发涌现最优子群比例！

use crate::heterogeneous_swarms::robot::{Robot, SubGroup};
use rand::Rng;

/// 在线表型可塑性调控器配置
#[derive(Debug, Clone)]
pub struct RegulatoryConfig {
    /// 调控状态更新时间步周期 (秒, 论文最优为 5.0s)
    pub update_interval: f64,
    /// 高光强强聚集阈值 (论文为 229.0)
    pub high_threshold: f64,
    /// 低光强弱信号阈值 (论文为 76.0)
    pub low_threshold: f64,
}

impl Default for RegulatoryConfig {
    fn default() -> Self {
        Self {
            update_interval: 5.0,
            high_threshold: 229.0,
            low_threshold: 76.0,
        }
    }
}

/// 【关卡 12 - 任务 2】计算在当前局域光强值下切换/表达为 Green (子群 1) 的概率
///
/// 论文方程:
/// - if light > config.high_threshold (229.0) -> 1.00
/// - else if light > config.low_threshold (76.0) -> 0.75
/// - else -> 0.50
///
/// # 提示
/// - 若卡壳可参考 [`crates/swarm-core/src/reference/heterogeneous_swarms.rs`](../reference/heterogeneous_swarms.rs)。
#[inline]
pub fn prob_green(light: f64, config: &RegulatoryConfig) -> f64 {
    todo!("【关卡 12 - 任务 2】在 regulatory.rs 中实现表型可塑性概率切换 prob_green");
}

/// 执行一次全群体的在线自适应表型调控抽样
pub fn update_swarm_phenotypes<R: Rng + ?Sized>(
    robots: &mut [Robot],
    intensities: &[f64],
    config: &RegulatoryConfig,
    rng: &mut R,
) {
    for (robot, &light) in robots.iter_mut().zip(intensities.iter()) {
        let p = prob_green(light, config);
        let roll: f64 = rng.gen();
        robot.subgroup = if roll < p {
            SubGroup::Green
        } else {
            SubGroup::Red
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prob_green() {
        let config = RegulatoryConfig::default();

        assert_eq!(prob_green(250.0, &config), 1.00);
        assert_eq!(prob_green(150.0, &config), 0.75);
        assert_eq!(prob_green(50.0, &config), 0.50);
        assert_eq!(prob_green(76.0, &config), 0.50);
        assert_eq!(prob_green(229.0, &config), 0.75);
    }
}
