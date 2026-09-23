//! # 少数派博弈统计量与波动率计算 (Metrics for Minority Game)

#![allow(unused_variables, dead_code)]

/// ## 任务 3: 计算市场归一化波动率 $\sigma^2 / N$
///
/// 论文核心统计指标:
/// $$A_{\text{mean}} = \frac{1}{T} \sum_{t=1}^T A(t)$$
/// $$\sigma^2 = \frac{1}{T} \sum_{t=1}^T (A(t) - A_{\text{mean}})^2$$
/// $$\text{volatility} = \frac{\sigma^2}{N}$$
///
/// 当智能体纯粹随机抛硬币时，理论波动率为 1.0；
/// 低于 1.0 说明群体自发形成了高效协调；高于 1.0 说明群体盲目扎堆（羊群效应）。
pub fn normalized_volatility(attendances: &[i32], num_agents: usize) -> f64 {
    // TODO: 实现归一化波动率计算
    todo!("【关卡 8 - 任务 3】请实现市场归一化波动率 sigma^2 / N 计算");
}

/// 计算信息比例参数 alpha = 2^M / N
#[inline]
pub fn alpha_parameter(memory: usize, num_agents: usize) -> f64 {
    (1 << memory) as f64 / num_agents as f64
}
