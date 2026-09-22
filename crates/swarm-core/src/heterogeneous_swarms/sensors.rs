//! # 传感器系统 (Limited Sensing System)
//!
//! 论文: *Emergence of Specialised Collective Behaviors in Evolving Heterogeneous Swarms* (PPSN 2024 / Springer)
//!
//! 根据论文第 3 节 (Methodology - Robot Design):
//! 1. 机器人基于 Thymio II 极简差速移动硬件，无任何显式通信能力 (无蓝牙/WiFi/射频广播)；
//! 2. 无记忆 (Memoryless): 控制器仅根据当前即时感知的局部传感器输入进行决策；
//! 3. 感知范围严格受限 (Limited Sensing):
//!    - 4 个 90° 象限方向传感器 (前 Front、右 Right、后 Back、左 Left)；
//!    - 每个象限测量感知范围内 (r_max = 2.0m) 最近邻居的距离 d_i 与相对航向角 θ_i；
//!    - 超过 2.0m 时，默认距离 d_i = 2.01m，相对航向 θ_i = 0.0；
//!    - 1 个局部标量光敏传感器，测量当前位置的光强 G \in [0, 255]；
//!    - 所有 9 个输入统一归一化至 [-1.0, 1.0]。

use crate::heterogeneous_swarms::environment::Environment;
use std::f64::consts::PI;

/// 象限编号
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quadrant {
    Front = 0,
    Right = 1,
    Back = 2,
    Left = 3,
}

/// 传感器配置参数
#[derive(Debug, Clone)]
pub struct SensorConfig {
    /// 最近邻最大感知距离 (m, 论文为 2.0m)
    pub max_range: f64,
    /// 无邻居时的默认距离 (m, 论文为 2.01m)
    pub default_range: f64,
}

impl Default for SensorConfig {
    fn default() -> Self {
        Self {
            max_range: 2.0,
            default_range: 2.01,
        }
    }
}

/// 将角度严格折叠至 [-PI, PI)
#[inline]
pub fn wrap_to_pi(mut angle: f64) -> f64 {
    while angle >= PI {
        angle -= 2.0 * PI;
    }
    while angle < -PI {
        angle += 2.0 * PI;
    }
    angle
}

/// 将方位相对角分类至 4 象限之一:
/// - Front (0): [-PI/4, +PI/4]
/// - Right (1): [+PI/4, +3*PI/4]
/// - Back  (2): [+3*PI/4, +PI] ∪ [-PI, -3*PI/4]
/// - Left  (3): [-3*PI/4, -PI/4]
#[inline]
pub fn bearing_to_quadrant(bearing: f64) -> Quadrant {
    let b = wrap_to_pi(bearing);
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

/// 计算单台机器人针对整个群体的 9 维归一化传感器感知向量
///
/// 向量布局:
/// `[d_front, theta_front, d_right, theta_right, d_back, theta_back, d_left, theta_left, light_norm]`
pub fn compute_robot_sensor_inputs(
    robot_idx: usize,
    positions: &[[f64; 2]],
    headings: &[f64],
    env: &Environment,
    config: &SensorConfig,
) -> [f64; 9] {
    let my_pos = positions[robot_idx];
    let my_heading = headings[robot_idx];

    // 初始化 4 象限的最近邻搜索: (最近距离, 相对航向)
    // 默认值: 距离为 default_range (2.01m), 航向为 0.0
    let mut nearest = [(config.default_range, 0.0); 4];

    for (j, &other_pos) in positions.iter().enumerate() {
        if j == robot_idx {
            continue;
        }

        let dx = other_pos[0] - my_pos[0];
        let dy = other_pos[1] - my_pos[1];
        let dist = (dx * dx + dy * dy).sqrt();

        // 仅考虑感知范围内的邻居
        if dist <= config.max_range {
            // 计算全局视线角并转换至本体坐标系下的方位角 (Bearing)
            let global_bearing = dy.atan2(dx);
            let rel_bearing = wrap_to_pi(global_bearing - my_heading);
            let quad = bearing_to_quadrant(rel_bearing) as usize;

            // 如果比当前该象限已有的邻居更近，则更新
            if dist < nearest[quad].0 {
                let other_heading = headings[j];
                let rel_heading = wrap_to_pi(other_heading - my_heading);
                nearest[quad] = (dist, rel_heading);
            }
        }
    }

    // 归一化各象限输入至 [-1.0, 1.0]
    // 距离归一化: 将 [0, max_range] 线性映射至 [-1.0, 1.0]
    // d_norm = (d / max_range) * 2.0 - 1.0
    // 当 d = default_range = 2.01 时，d_norm = 1.01 (无邻居标识)
    // 相对航向归一化: rel_heading \in [-PI, PI) -> rel_heading / PI \in [-1.0, 1.0]
    let mut inputs = [0.0; 9];
    for q in 0..4 {
        let (d, th) = nearest[q];
        let d_norm = (d / config.max_range) * 2.0 - 1.0;
        let th_norm = th / PI;

        inputs[q * 2] = d_norm;
        inputs[q * 2 + 1] = th_norm;
    }

    // 第 9 维: 局部标量光敏强度，归一化至 [-1.0, 1.0]
    inputs[8] = env.normalized_intensity(my_pos);

    inputs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heterogeneous_swarms::environment::ArenaType;

    #[test]
    fn test_bearing_to_quadrant() {
        assert_eq!(bearing_to_quadrant(0.0), Quadrant::Front);
        assert_eq!(bearing_to_quadrant(PI / 2.0), Quadrant::Right);
        assert_eq!(bearing_to_quadrant(PI), Quadrant::Back);
        assert_eq!(bearing_to_quadrant(-PI), Quadrant::Back);
        assert_eq!(bearing_to_quadrant(-PI / 2.0), Quadrant::Left);
    }

    #[test]
    fn test_compute_sensor_inputs_isolated() {
        let env = Environment::new(ArenaType::Center);
        let config = SensorConfig::default();

        let positions = vec![[15.0, 15.0]];
        let headings = vec![0.0];

        let inputs = compute_robot_sensor_inputs(0, &positions, &headings, &env, &config);

        // 无邻居时，4 象限的距离应大于 1.0，航向应为 0.0
        for q in 0..4 {
            assert!(inputs[q * 2] > 1.0);
            assert_eq!(inputs[q * 2 + 1], 0.0);
        }
        // 中心点光强最大，归一化应为 1.0
        assert!((inputs[8] - 1.0).abs() < 1e-3);
    }

    #[test]
    fn test_compute_sensor_inputs_with_neighbor() {
        let env = Environment::new(ArenaType::Center);
        let config = SensorConfig::default();

        // 机器人 0 在 (15, 15)，朝向东 (0)
        // 机器人 1 在 (16, 15)，在正前方 1m 处，朝向北 (PI/2)
        let positions = vec![[15.0, 15.0], [16.0, 15.0]];
        let headings = vec![0.0, PI / 2.0];

        let inputs = compute_robot_sensor_inputs(0, &positions, &headings, &env, &config);

        // Front 象限 (q=0): 距离 1.0m -> (1.0 / 2.0) * 2 - 1 = 0.0
        assert!((inputs[0] - 0.0).abs() < 1e-3);
        // 相对航向: PI/2 - 0 = PI/2 -> (PI/2) / PI = 0.5
        assert!((inputs[1] - 0.5).abs() < 1e-3);

        // 其余象限无邻居
        assert!(inputs[2] > 1.0); // Right
        assert!(inputs[4] > 1.0); // Back
        assert!(inputs[6] > 1.0); // Left
    }
}
