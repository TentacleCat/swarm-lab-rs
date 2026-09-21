//! # 🎯 [实战关卡 1] 序参量计算 (Metrics & Order Parameters)
//!
//! 在这里你将学习并练习 Rust 的基础语法：
//! - 切片只读借用 (`&[f64]`)
//! - 基础数学运算 (`.cos()`, `.sin()`, `.powi(2)`, `.sqrt()`)
//! - 循环方式（`for &th in phases` 或函数式 `.iter().map().sum()`）
//!
//! 💡 完成后运行以下命令验证：
//!    cargo test -p swarm-core -- metrics
//!
//! 遇到卡壳可查阅参考答案: `crates/swarm-core/src/reference/metrics.rs`

#![allow(unused_variables, dead_code)]

/// ## 任务 1: 计算 Kuramoto 相位相干度 R ∈ [0, 1]
///
/// ### 数学公式:
/// $$Z = \frac{1}{N} \sum_{j=1}^N e^{i \theta_j} = R e^{i \psi}$$
/// $$R = \sqrt{\left(\frac{1}{N} \sum_{j=1}^N \cos\theta_j\right)^2 + \left(\frac{1}{N} \sum_{j=1}^N \sin\theta_j\right)^2}$$
///
/// ### 物理意义:
/// - 当所有粒子同相 ($\theta_1 = \theta_2 = \dots$) 时，R = 1；
/// - 当相位在 [0, 2π) 均匀杂乱分布时，R ≈ 0。
pub fn kuramoto_order_parameter(phases: &[f64]) -> f64 {
    // TODO: 请在此处实现 Kuramoto 序参量计算
    // 提示:
    // 1. 处理空数组边界情况: if phases.is_empty() { return 0.0; }
    // 2. 统计 sum_cos 与 sum_sin
    // 3. 计算 ((sum_cos / n).powi(2) + (sum_sin / n).powi(2)).sqrt()
    todo!("【任务 1】请实现 kuramoto_order_parameter 函数");
}

/// ## 任务 2: 计算圆环空间聚集度 S ∈ [0, 1]
///
/// ### 物理意义:
/// 对于一维圆环，粒子空间位置同样用角度 $\phi_j \in [-\pi, \pi)$ 表示，
/// 因此空间聚集度 S 的计算在数学形式上与 Kuramoto R 完全一致。
pub fn ring_spatial_order_parameter(positions: &[f64]) -> f64 {
    // TODO: 直接复用 kuramoto_order_parameter 计算 positions
    todo!("【任务 2】请实现 ring_spatial_order_parameter 函数");
}

/// ## 任务 3: 计算一维圆环时空关联序参量 S_+ 与 S_- ∈ [0, 1]
///
/// ### 数学公式:
/// $$W_{\pm} = \frac{1}{N} \sum_{j=1}^N e^{i (\phi_j \pm \theta_j)}$$
/// $$S_{\pm} = |W_{\pm}| = \sqrt{\left(\frac{1}{N} \sum \cos(\phi_j \pm \theta_j)\right)^2 + \left(\frac{1}{N} \sum \sin(\phi_j \pm \theta_j)\right)^2}$$
///
/// ### 物理意义:
/// - $S_+ \approx 1$ 或 $S_- \approx 1$ 意味着相位与空间位置严格绑定（$\theta_j = \mp \phi_j + C$），
///   系统处于完美的“静态相位波态（Static Phase Wave）”。
pub fn ring_spatiotemporal_order_parameters(positions: &[f64], phases: &[f64]) -> (f64, f64) {
    // TODO: 计算 S_+ 与 S_-
    // 提示: 可以使用 positions.iter().zip(phases.iter()) 同时遍历位置与相位
    todo!("【任务 3】请实现 ring_spatiotemporal_order_parameters 函数");
}

/// ## 任务 4: 计算二维粒子的回转半径 (Radius of Gyration)
///
/// ### 数学公式:
/// $$R_{\text{gyr}} = \sqrt{\frac{1}{N} \sum_{j=1}^N \left( (x_j - \bar{x})^2 + (y_j - \bar{y})^2 \right)}$$
/// 其中 $\bar{x}, \bar{y}$ 分别为所有粒子在 x, y 方向的质心均值。
pub fn radius_of_gyration_2d(x: &[f64], y: &[f64]) -> f64 {
    // TODO: 计算质心和回转半径
    todo!("【任务 4】请实现 radius_of_gyration_2d 函数");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_kuramoto_identical_phases() {
        // 完全锁相: 所有粒子同相 -> R = 1.0
        let phases = vec![0.5, 0.5, 0.5, 0.5];
        let r = kuramoto_order_parameter(&phases);
        assert!((r - 1.0).abs() < 1e-6, "同相粒子 Kuramoto R 应当为 1.0，实际得到: {}", r);
    }

    #[test]
    fn test_kuramoto_uniform_phases() {
        // 完全非相干: 均匀对称分布在圆周上 -> R = 0.0
        let phases = vec![0.0, PI / 2.0, PI, 3.0 * PI / 2.0];
        let r = kuramoto_order_parameter(&phases);
        assert!(r < 1e-6, "均匀分散粒子 Kuramoto R 应当接近 0.0，实际得到: {}", r);
    }

    #[test]
    fn test_phase_wave_order_parameter() {
        // 纯相位波: theta_i = phi_i -> phi - theta = 0 -> S_- = 1.0, S_+ = 0.0
        let n = 20;
        let positions: Vec<f64> = (0..n).map(|i| (i as f64) * 2.0 * PI / (n as f64) - PI).collect();
        let phases = positions.clone();
        let (s_plus, s_minus) = ring_spatiotemporal_order_parameters(&positions, &phases);
        assert!(s_plus < 1e-6, "纯反向波下 S_+ 应当接近 0.0");
        assert!((s_minus - 1.0).abs() < 1e-6, "theta_i = phi_i 下 S_- 应当接近 1.0");
    }
}
