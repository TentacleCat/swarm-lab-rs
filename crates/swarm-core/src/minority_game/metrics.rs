//! # 少数派博弈统计量与波动率计算 (Metrics for Minority Game)

/// 计算市场归一化波动率 sigma^2 / N
///
/// 公式:
/// $$A_{\text{mean}} = \frac{1}{T} \sum_{t=1}^T A(t)$$
/// $$\sigma^2 = \frac{1}{T} \sum_{t=1}^T (A(t) - A_{\text{mean}})^2$$
/// $$\text{volatility} = \frac{\sigma^2}{N}$$
///
/// 当智能体纯粹随机抛硬币时，理论波动率为 1.0；
/// 低于 1.0 说明群体自发形成了高效协调；高于 1.0 说明群体盲目扎堆。
pub fn normalized_volatility(attendances: &[i32], num_agents: usize) -> f64 {
    if attendances.is_empty() || num_agents == 0 {
        return 0.0;
    }
    let t = attendances.len() as f64;
    let sum: f64 = attendances.iter().map(|&a| a as f64).sum();
    let mean = sum / t;

    let var: f64 = attendances
        .iter()
        .map(|&a| {
            let diff = a as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / t;

    var / num_agents as f64
}

/// 计算信息比例参数 alpha = 2^M / N
#[inline]
pub fn alpha_parameter(memory: usize, num_agents: usize) -> f64 {
    (1 << memory) as f64 / num_agents as f64
}
