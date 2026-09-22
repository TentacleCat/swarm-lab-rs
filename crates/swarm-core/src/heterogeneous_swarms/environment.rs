//! # 标量场环境模块 (Scalar Field Environments)
//!
//! 论文: *Emergence of Specialised Collective Behaviors in Evolving Heterogeneous Swarms* (PPSN 2024 / Springer)
//!
//! 本模块实现 30m x 30m 竞技场中的 4 种标量光强场:
//! 1. `Center`: 经典中心径向衰减场 (训练主场，最高光强 255 在中心 (15, 15)，半径 14.5m 处降为 0)；
//! 2. `Bimodal`: 双峰竞争场 (峰值位于 (9, 15) 与 (21, 15)，考察群体的分布式决策与分流/聚合能力)；
//! 3. `Linear`: 全局单调线性倾斜场 (沿 X 轴从 0 渐变至 255，低梯度弱信号挑战)；
//! 4. `Banana`: 经典非线性 Rosenbrock 香蕉弯曲浅谷场 (存在局部极值点与狭窄弯曲鞍部，最具欺骗性的鲁棒性测试场)。

use serde::{Deserialize, Serialize};

/// 竞技场类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArenaType {
    /// 中心渐变场 (默认训练环境)
    Center,
    /// 双峰环境 (考察群体决策)
    Bimodal,
    /// 线性单调场 (弱梯度环境)
    Linear,
    /// 香蕉非线性曲面 (局部极值与浅谷陷阱)
    Banana,
}

impl std::str::FromStr for ArenaType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "center" => Ok(ArenaType::Center),
            "bimodal" | "bi-modal" => Ok(ArenaType::Bimodal),
            "linear" => Ok(ArenaType::Linear),
            "banana" => Ok(ArenaType::Banana),
            _ => Err(format!("Unknown arena type: {}", s)),
        }
    }
}

/// 标量场环境
#[derive(Debug, Clone)]
pub struct Environment {
    /// 竞技场类型
    pub arena_type: ArenaType,
    /// 竞技场宽 (m)
    pub width: f64,
    /// 竞技场高 (m)
    pub height: f64,
    /// 场中心坐标 (m)
    pub center: [f64; 2],
    /// 最大光强 (固定为 255.0)
    pub g_max: f64,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new(ArenaType::Center)
    }
}

impl Environment {
    /// 创建标准 30m x 30m 竞技场
    pub fn new(arena_type: ArenaType) -> Self {
        Self {
            arena_type,
            width: 30.0,
            height: 30.0,
            center: [15.0, 15.0],
            g_max: 255.0,
        }
    }

    /// 查询某物理坐标 `(x, y)` 处的光强标量值 `G(x, y) \in [0.0, 255.0]`
    pub fn sample_intensity(&self, pos: [f64; 2]) -> f64 {
        let x = pos[0].clamp(0.0, self.width);
        let y = pos[1].clamp(0.0, self.height);

        match self.arena_type {
            ArenaType::Center => {
                let dx = x - self.center[0];
                let dy = y - self.center[1];
                let dist = (dx * dx + dy * dy).sqrt();
                let r_max = 14.5;
                if dist >= r_max {
                    0.0
                } else {
                    self.g_max * (1.0 - dist / r_max).clamp(0.0, 1.0)
                }
            }
            ArenaType::Bimodal => {
                // 两个高斯/圆锥峰值，中心在 (9.0, 15.0) 与 (21.0, 15.0)
                let c1 = [9.0, 15.0];
                let c2 = [21.0, 15.0];
                let r_max = 8.5;

                let d1 = ((x - c1[0]).powi(2) + (y - c1[1]).powi(2)).sqrt();
                let d2 = ((x - c2[0]).powi(2) + (y - c2[1]).powi(2)).sqrt();

                let g1 = if d1 < r_max {
                    self.g_max * (1.0 - d1 / r_max)
                } else {
                    0.0
                };
                let g2 = if d2 < r_max {
                    self.g_max * (1.0 - d2 / r_max)
                } else {
                    0.0
                };
                g1.max(g2).clamp(0.0, self.g_max)
            }
            ArenaType::Linear => {
                // 沿 X 轴线性增长: x=0 时为 0，x=30 时为 255
                (self.g_max * (x / self.width)).clamp(0.0, self.g_max)
            }
            ArenaType::Banana => {
                // 经典 Rosenbrock-like 香蕉弯曲曲面
                // 坐标变换到 [-2, 2] x [-1, 3]
                let u = (x - 15.0) / 7.5; // [-2.0, 2.0]
                let v = (y - 12.0) / 6.0; // [-2.0, 3.0]

                // Rosenbrock: (1 - u)^2 + 10 * (v - u^2)^2
                let val = (1.0 - u).powi(2) + 8.0 * (v - u.powi(2)).powi(2);
                // 转化为高斯峰和浅谷
                let intensity = self.g_max * (-val / 6.0).exp();
                intensity.clamp(0.0, self.g_max)
            }
        }
    }

    /// 归一化光强到神经网络输入范围 `[-1.0, 1.0]`
    pub fn normalized_intensity(&self, pos: [f64; 2]) -> f64 {
        let raw = self.sample_intensity(pos);
        2.0 * (raw / self.g_max) - 1.0
    }

    /// 将机器人限制在合法竞技场边界内
    pub fn clamp_position(&self, pos: [f64; 2], radius: f64) -> [f64; 2] {
        [
            pos[0].clamp(radius, self.width - radius),
            pos[1].clamp(radius, self.height - radius),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_center_intensity() {
        let env = Environment::new(ArenaType::Center);
        // 中心点应达到最大光强 255
        let center_val = env.sample_intensity([15.0, 15.0]);
        assert!((center_val - 255.0).abs() < 1e-3);

        // 归一化后中心应为 1.0
        let norm_center = env.normalized_intensity([15.0, 15.0]);
        assert!((norm_center - 1.0).abs() < 1e-3);

        // 边界外部应衰减至 0.0
        let far_val = env.sample_intensity([0.0, 0.0]);
        assert_eq!(far_val, 0.0);
        let norm_far = env.normalized_intensity([0.0, 0.0]);
        assert!((norm_far - (-1.0)).abs() < 1e-3);
    }

    #[test]
    fn test_bimodal_intensity() {
        let env = Environment::new(ArenaType::Bimodal);
        let peak1 = env.sample_intensity([9.0, 15.0]);
        let peak2 = env.sample_intensity([21.0, 15.0]);
        assert!((peak1 - 255.0).abs() < 1e-3);
        assert!((peak2 - 255.0).abs() < 1e-3);
    }
}
