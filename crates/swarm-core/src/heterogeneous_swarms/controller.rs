#![allow(unused_variables, dead_code, unused_imports)]

//! # 储备池神经网络与基因型系统 (Reservoir Controller & Genotype)
//!
//! 论文: *Emergence of Specialised Collective Behaviors in Evolving Heterogeneous Swarms* (PPSN 2024 / Springer)
//!
//! 根据论文第 3 节 (Methodology - Controller Design):
//! 1. 采用前馈储备池神经网络 (Reservoir Neural Network, RNN):
//!    - 输入层: 9 神经元 (4 象限的距离与航向 + 1 局部标量光敏)；
//!    - 隐藏层 1: 9 神经元，ReLU 激活；
//!    - 隐藏层 2: 9 神经元，ReLU 激活；
//!    - 输出层: 2 神经元，tanh 激活，分别对应目标线速度 v 与目标角速度 w；
//! 2. 储备池权重 W_h1 \in R^{9x9}, W_h2 \in R^{9x9} 均在初始化时按均匀分布 U[-1, 1] 随机采样并严格冻结；
//! 3. 神经元偏置全置 0；
//! 4. 进化算法仅优化输出层连接权重 W_out \in R^{2x9} (每个控制器仅 18 个参数)；
//! 5. 异构群体划分两个子群 (Green / Red):
//!    - 分别拥有独立的冻结储备池 (Reservoir 1 与 Reservoir 2)；
//!    - 单一全基因型 x \in R^{36} = [W_out_1, W_out_2] 前 18 维编码子群 1，后 18 维编码子群 2。

use rand::distributions::{Distribution, Uniform};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// 储备池隐藏层结构 (包含 2 层 9x9 权重矩阵)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reservoir {
    /// 第一隐层权重 W_h1: 9 x 9
    pub w_h1: [[f64; 9]; 9],
    /// 第二隐层权重 W_h2: 9 x 9
    pub w_h2: [[f64; 9]; 9],
}

impl Reservoir {
    /// 从随机数生成器按 U[-1.0, 1.0] 初始化冻结储备池
    pub fn new_random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let dist = Uniform::new_inclusive(-1.0, 1.0);
        let mut w_h1 = [[0.0; 9]; 9];
        let mut w_h2 = [[0.0; 9]; 9];

        for i in 0..9 {
            for j in 0..9 {
                w_h1[i][j] = dist.sample(rng);
                w_h2[i][j] = dist.sample(rng);
            }
        }

        Self { w_h1, w_h2 }
    }

    /// 【关卡 12 - 任务 3】前向传播通过储备池隐藏层，计算隐藏激活特征 h2 \in R^9
    ///
    /// 公式:
    /// - h1 = ReLU(W_h1 * input)
    /// - h2 = ReLU(W_h2 * h1)
    ///
    /// # 提示
    /// - 隐藏层维度均为 9；
    /// - ReLU(x) = x.max(0.0)；
    /// - 若卡壳可参考 [`crates/swarm-core/src/reference/heterogeneous_swarms.rs`](../reference/heterogeneous_swarms.rs)。
    #[inline]
    pub fn forward_hidden(&self, input: &[f64; 9]) -> [f64; 9] {
        todo!("【关卡 12 - 任务 3】在 controller.rs 中实现储备池隐藏层前向推理 forward_hidden");
    }
}

/// 储备池神经网络控制器 (RNN Controller)
#[derive(Debug, Clone)]
pub struct ReservoirNN {
    /// 冻结的隐层储备池
    pub reservoir: Reservoir,
    /// 可学习进化的输出层权重矩阵 W_out: 2 x 9 (共 18 个参数)
    pub w_out: [[f64; 9]; 2],
}

impl ReservoirNN {
    /// 使用指定的储备池和输出权重构建控制器
    pub fn new(reservoir: Reservoir, w_out_flat: &[f64; 18]) -> Self {
        let mut w_out = [[0.0; 9]; 2];
        for i in 0..2 {
            for j in 0..9 {
                w_out[i][j] = w_out_flat[i * 9 + j];
            }
        }
        Self { reservoir, w_out }
    }

    /// 更新可学习的输出层权重
    pub fn set_weights(&mut self, w_out_flat: &[f64; 18]) {
        for i in 0..2 {
            for j in 0..9 {
                self.w_out[i][j] = w_out_flat[i * 9 + j];
            }
        }
    }

    /// 【关卡 12 - 任务 3】执行前向推理，输出 [v, w] \in [-1.0, 1.0]^2
    ///
    /// 公式: RNN = tanh( W_out * ReLU( W_h2 * ReLU( W_h1 * s_in ) ) )
    ///
    /// # 提示
    /// - 先调用 `self.reservoir.forward_hidden(input)` 计算隐藏层激活特征 h2；
    /// - 再计算输出层线性组合并取双曲正切激活：`out[i] = (W_out[i] · h2).tanh()`。
    #[inline]
    pub fn forward(&self, input: &[f64; 9]) -> [f64; 2] {
        todo!("【关卡 12 - 任务 3】在 controller.rs 中实现输出层前向推理 forward");
    }
}

/// 异构群体全基因型 (Heterogeneous Swarm Genotype)
///
/// 论文中将表型可塑性表达为单一基因型中的两段组成:
/// - 前 18 维: 子群 1 (Green) 的输出层权重；
/// - 后 18 维: 子群 2 (Red) 的输出层权重；
/// - 全长 36 维，由单个适应度函数统一协同演化。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeterogeneousGenotype {
    pub sub1: [f64; 18],
    pub sub2: [f64; 18],
}

impl HeterogeneousGenotype {
    pub fn new(sub1: [f64; 18], sub2: [f64; 18]) -> Self {
        Self { sub1, sub2 }
    }

    /// 从 36 维扁平切片创建基因型
    pub fn from_flat_slice(slice: &[f64]) -> Self {
        assert!(slice.len() >= 36);
        let mut sub1 = [0.0; 18];
        let mut sub2 = [0.0; 18];
        sub1.copy_from_slice(&slice[0..18]);
        sub2.copy_from_slice(&slice[18..36]);
        Self { sub1, sub2 }
    }

    /// 转换为 36 维扁平 Vec<f64>
    pub fn to_flat_vec(&self) -> Vec<f64> {
        let mut v = Vec::with_capacity(36);
        v.extend_from_slice(&self.sub1);
        v.extend_from_slice(&self.sub2);
        v
    }

    /// 获取子群 1 (Green) 的 18 维输出权重切片
    #[inline]
    pub fn sub1_weights(&self) -> &[f64; 18] {
        &self.sub1
    }

    /// 获取子群 2 (Red) 的 18 维输出权重切片
    #[inline]
    pub fn sub2_weights(&self) -> &[f64; 18] {
        &self.sub2
    }

    /// 均匀随机采样初始基因型 U[-5.0, 5.0] (见论文表 1)
    pub fn random_uniform<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let dist = Uniform::new_inclusive(-5.0, 5.0);
        let mut sub1 = [0.0; 18];
        let mut sub2 = [0.0; 18];
        for w in &mut sub1 {
            *w = dist.sample(rng);
        }
        for w in &mut sub2 {
            *w = dist.sample(rng);
        }
        Self { sub1, sub2 }
    }
}

/// 同质基准群体基因型 (Homogeneous Baseline Genotype, 18 维)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineGenotype {
    pub weights: [f64; 18],
}

impl BaselineGenotype {
    pub fn new(weights: [f64; 18]) -> Self {
        Self { weights }
    }

    pub fn random_uniform<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let dist = Uniform::new_inclusive(-5.0, 5.0);
        let mut weights = [0.0; 18];
        for w in &mut weights {
            *w = dist.sample(rng);
        }
        Self { weights }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_forward_pass_bounded() {
        let mut rng = StdRng::seed_from_u64(42);
        let res = Reservoir::new_random(&mut rng);
        let weights = [0.5; 18];
        let nn = ReservoirNN::new(res, &weights);

        let input = [0.5, -0.2, 1.0, 0.0, -1.0, 0.8, 0.1, -0.4, 0.9];
        let out = nn.forward(&input);

        // tanh 输出严格在 [-1.0, 1.0] 范围内
        assert!(out[0] >= -1.0 && out[0] <= 1.0);
        assert!(out[1] >= -1.0 && out[1] <= 1.0);
    }

    #[test]
    fn test_genotype_split() {
        let mut w = [0.0; 36];
        for i in 0..18 {
            w[i] = 1.0;
        }
        for i in 18..36 {
            w[i] = 2.0;
        }
        let geno = HeterogeneousGenotype::from_flat_slice(&w);
        assert_eq!(geno.sub1_weights()[0], 1.0);
        assert_eq!(geno.sub2_weights()[0], 2.0);
    }
}
