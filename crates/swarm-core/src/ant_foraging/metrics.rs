//! # Ant Foraging Metrics
//!
//! 统计蚂蚁趋化觅食过程中的宏观与微观演化指标：
//! - 剩余食物总量 $C(t)$ 与食物运送效率（消耗百分比）；
//! - 蚂蚁总质量 $U(t) + W(t)$（用于检验质量守恒）；
//! - 觅食蚁与搬运蚁的比例；
//! - 信息素峰值浓度与蚁道显著度（Trail Prominence）。

use super::grid::Grid2D;

/// 某一时刻的宏观统计量快照
#[derive(Debug, Clone, PartialEq)]
pub struct AntForagingMetrics {
    /// 当前无量纲物理时间 $t$
    pub time: f64,
    /// 剩余食物质量 $\iint c(t, x, y) \, dx dy$
    pub total_food: f64,
    /// 累计食物消耗率 $(C_0 - C(t)) / C_0 \in [0, 1]$
    pub food_depletion_ratio: f64,
    /// 觅食蚁总质量 $\iint u \, dx dy$
    pub foraging_ants_mass: f64,
    /// 搬运蚁总质量 $\iint w \, dx dy$
    pub returning_ants_mass: f64,
    /// 全场总蚂蚁质量（守恒量） $U + W$
    pub total_ants_mass: f64,
    /// 全场信息素峰值 $\max v(x, y)$
    pub max_pheromone: f64,
    /// 全场信息素积分 $\iint v \, dx dy$
    pub total_pheromone: f64,
}

impl AntForagingMetrics {
    /// 从当前网格状态计算指标快照
    pub fn compute(
        time: f64,
        u: &Grid2D,
        w: &Grid2D,
        v: &Grid2D,
        c: &Grid2D,
        initial_food: f64,
    ) -> Self {
        let total_food = c.integrate();
        let food_depletion_ratio = if initial_food > 1e-12 {
            ((initial_food - total_food) / initial_food).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let foraging_ants_mass = u.integrate();
        let returning_ants_mass = w.integrate();
        let total_ants_mass = foraging_ants_mass + returning_ants_mass;

        let mut max_pheromone: f64 = 0.0;
        for &val in &v.data {
            if val > max_pheromone {
                max_pheromone = val;
            }
        }
        let total_pheromone = v.integrate();

        Self {
            time,
            total_food,
            food_depletion_ratio,
            foraging_ants_mass,
            returning_ants_mass,
            total_ants_mass,
            max_pheromone,
            total_pheromone,
        }
    }
}
