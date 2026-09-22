//! # 💡 [参考答案区] 异构群体演化与表型可塑性标准参考实现 (Reference Implementation)
//!
//! 论文: *Emergence of Specialised Collective Behaviors in Evolving Heterogeneous Swarms* (PPSN 2024 / Springer)
//!
//! 本文件包含经过完备单测验证的标准理论计算与基准实现，供闯关学习者对比查阅。

use std::f64::consts::PI;

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
pub fn ref_bearing_to_quadrant(bearing: f64) -> usize {
    let b = ref_wrap_to_pi(bearing);
    let pi_4 = PI / 4.0;
    let pi_3_4 = 3.0 * PI / 4.0;

    if b >= -pi_4 && b < pi_4 {
        0 // Front
    } else if b >= pi_4 && b < pi_3_4 {
        1 // Right
    } else if b >= -pi_3_4 && b < -pi_4 {
        3 // Left
    } else {
        2 // Back
    }
}

/// 在线表型可塑性切换概率函数标准参考实现 (公式 P_green)
#[inline]
pub fn ref_prob_green(light: f64) -> f64 {
    if light > 229.0 {
        1.00
    } else if light > 76.0 {
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
    // 隐藏层 1
    let mut h1 = [0.0; 9];
    for i in 0..9 {
        let mut sum = 0.0;
        for j in 0..9 {
            sum += w_h1[i][j] * s_in[j];
        }
        h1[i] = sum.max(0.0);
    }

    // 隐藏层 2
    let mut h2 = [0.0; 9];
    for i in 0..9 {
        let mut sum = 0.0;
        for j in 0..9 {
            sum += w_h2[i][j] * h1[j];
        }
        h2[i] = sum.max(0.0);
    }

    // 输出层
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ref_prob_green() {
        assert_eq!(ref_prob_green(240.0), 1.0);
        assert_eq!(ref_prob_green(100.0), 0.75);
        assert_eq!(ref_prob_green(50.0), 0.50);
    }

    #[test]
    fn test_ref_order_parameter() {
        let pos = vec![[0.0, 0.0], [0.5, 0.0], [1.0, 0.0]];
        let heads = vec![0.0, 0.0, 0.0];
        let order = ref_compute_swarm_order(&pos, &heads, 2.0);
        assert!((order - 1.0).abs() < 1e-5);
    }
}
