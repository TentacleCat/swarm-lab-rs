//! # 🎯 [实战关卡 2] 数值积分器 (Numerical ODE Integrators)
//!
//! 在这里你将学习并练习 Rust 的关键工程与语法技能：
//! - 可变借用 (`&mut [f64]`) 与切片更新
//! - 高性能无内存分配设计（在结构体中复用 `Vec<f64>` 缓冲区，避免每步 `malloc`）
//! - 面向特征（Trait）的多态调用：`system.derivative(...)`
//!
//! 💡 完成后运行以下命令验证：
//!    cargo test -p swarm-core -- integrator
//!
//! 遇到卡壳可查阅参考答案: `crates/swarm-core/src/reference/integrator.rs`

#![allow(unused_variables, dead_code)]

use crate::types::DynamicalSystem;

/// ## 任务 1: 前向欧拉积分器 (Forward Euler)
///
/// ### 数学更新式:
/// $$s(t + \Delta t) = s(t) + \Delta t \cdot f(s(t))$$
pub struct EulerIntegrator {
    deriv: Vec<f64>,
}

impl EulerIntegrator {
    pub fn new(dim: usize) -> Self {
        Self {
            deriv: vec![0.0; dim],
        }
    }

    pub fn step<S: DynamicalSystem>(&mut self, system: &S, state: &mut [f64], dt: f64) {
        // TODO: 请实现一阶欧拉法更新逻辑
        // 步骤:
        // 1. 调用 system.derivative(state, &mut self.deriv) 计算导数
        // 2. 遍历 state，对每个分量执行: state[i] += dt * self.deriv[i]
        // 3. 调用 system.post_step(state) 执行周期性边界归一化
        todo!("【任务 1】请实现 EulerIntegrator::step");
    }
}

/// ## 任务 2: 经典四阶 Runge-Kutta 积分器 (RK4)
///
/// ### 数学更新式:
/// $$k_1 = f(s)$$
/// $$k_2 = f\left(s + \frac{\Delta t}{2} k_1\right)$$
/// $$k_3 = f\left(s + \frac{\Delta t}{2} k_2\right)$$
/// $$k_4 = f(s + \Delta t \cdot k_3)$$
/// $$s(t + \Delta t) = s(t) + \frac{\Delta t}{6} (k_1 + 2 k_2 + 2 k_3 + k_4)$$
pub struct Rk4Integrator {
    k1: Vec<f64>,
    k2: Vec<f64>,
    k3: Vec<f64>,
    k4: Vec<f64>,
    tmp: Vec<f64>,
}

impl Rk4Integrator {
    pub fn new(dim: usize) -> Self {
        Self {
            k1: vec![0.0; dim],
            k2: vec![0.0; dim],
            k3: vec![0.0; dim],
            k4: vec![0.0; dim],
            tmp: vec![0.0; dim],
        }
    }

    pub fn step<S: DynamicalSystem>(&mut self, system: &S, state: &mut [f64], dt: f64) {
        // TODO: 请实现 RK4 积分更新逻辑
        // 步骤提示:
        // 1. 计算 k1: system.derivative(state, &mut self.k1);
        // 2. 计算 k2:
        //    for i in 0..dim { self.tmp[i] = state[i] + 0.5 * dt * self.k1[i]; }
        //    system.derivative(&self.tmp, &mut self.k2);
        // 3. 计算 k3:
        //    for i in 0..dim { self.tmp[i] = state[i] + 0.5 * dt * self.k2[i]; }
        //    system.derivative(&self.tmp, &mut self.k3);
        // 4. 计算 k4:
        //    for i in 0..dim { self.tmp[i] = state[i] + dt * self.k3[i]; }
        //    system.derivative(&self.tmp, &mut self.k4);
        // 5. 合成更新:
        //    state[i] += (dt / 6.0) * (self.k1[i] + 2.0*self.k2[i] + 2.0*self.k3[i] + self.k4[i]);
        // 6. 调用 system.post_step(state);
        todo!("【任务 2】请实现 Rk4Integrator::step");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MetricsSnapshot;

    /// 简单的指数衰减模型: dy/dt = -y, 解析解为 y(t) = y_0 * exp(-t)
    struct DecaySystem;
    impl DynamicalSystem for DecaySystem {
        fn dimension(&self) -> usize { 1 }
        fn num_agents(&self) -> usize { 1 }
        fn derivative(&self, s: &[f64], ds: &mut [f64]) {
            ds[0] = -s[0];
        }
        fn metrics(&self, _s: &[f64]) -> MetricsSnapshot {
            MetricsSnapshot::default()
        }
    }

    #[test]
    fn test_euler_step() {
        let sys = DecaySystem;
        let mut integrator = EulerIntegrator::new(1);
        let mut state = vec![1.0];
        let dt = 0.01;

        for _ in 0..100 {
            integrator.step(&sys, &mut state, dt);
        }

        // y(1.0) = exp(-1) ≈ 0.367879
        let analytical = (-1.0f64).exp();
        assert!((state[0] - analytical).abs() < 0.01, "欧拉法步进误差过大: 得到 {}, 期望 {}", state[0], analytical);
    }

    #[test]
    fn test_rk4_high_accuracy() {
        let sys = DecaySystem;
        let mut integrator = Rk4Integrator::new(1);
        let mut state = vec![1.0];
        let dt = 0.1;

        // 步进 10 步到达 t = 1.0
        for _ in 0..10 {
            integrator.step(&sys, &mut state, dt);
        }

        // 四阶 RK4 在 dt=0.1 下应具有极高精度 (误差 < 1e-5)
        let analytical = (-1.0f64).exp();
        assert!((state[0] - analytical).abs() < 1e-5, "RK4 精度未达预期: 得到 {}, 期望 {}", state[0], analytical);
    }
}
