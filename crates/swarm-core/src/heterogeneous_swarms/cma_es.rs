//! # 协方差矩阵自适应进化策略 (CMA-ES Optimizer)
//!
//! 论文: *Emergence of Specialised Collective Behaviors in Evolving Heterogeneous Swarms* (PPSN 2024 / Springer)
//!
//! 算法细节 (见论文第 3 节与表 1):
//! 1. 连续黑盒优化器: CMA-ES (Hansen 经典无导数进化算法)；
//! 2. 优化参数空间:
//!    - 异构群体: 维度 D = 36 (子群 1 占 18 维，子群 2 占 18 维)；
//!    - 同质基准: 维度 D = 18；
//!    - 参数边界: U[-5.0, 5.0]；
//! 3. 种群规模 \lambda = 30, 精英父代数 \mu = 15；
//! 4. 步长控制: 初始步长 \sigma_0 = 1.0, 配合路径累积 (Cumulative Step-size Adaptation, CSA)；
//! 5. 协方差矩阵自适应更新: 结合 Rank-one 与 Rank-\mu 更新；
//! 6. 重复性评估: 每个个体进行 N_repeats = 3 次独立模拟评测，取中位数作为真实适应度，消除“偶然运气”偏差。

use rand::distributions::{Distribution, Uniform};
use rand::Rng;
use rand_distr::StandardNormal;
use serde::{Deserialize, Serialize};

/// CMA-ES 进化优化器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmaEsConfig {
    /// 决策变量维度 D (异构为 36, 同质为 18)
    pub dim: usize,
    /// 种群大小 \lambda (论文为 30)
    pub lambda: usize,
    /// 优秀父代数 \mu (通常为 \lambda / 2 = 15)
    pub mu: usize,
    /// 初始全局步长 \sigma_0 (论文为 1.0)
    pub sigma0: f64,
    /// 参数下界与上界 [-5.0, 5.0]
    pub bounds: (f64, f64),
    /// 最大代数 (论文为 100 代)
    pub max_generations: usize,
    /// 单个个体独立评测次数 (论文为 3 次，取中位数)
    pub repeats_per_eval: usize,
}

impl Default for CmaEsConfig {
    fn default() -> Self {
        Self {
            dim: 36,
            lambda: 30,
            mu: 15,
            sigma0: 1.0,
            bounds: (-5.0, 5.0),
            max_generations: 100,
            repeats_per_eval: 3,
        }
    }
}

/// CMA-ES 优化器状态机
#[derive(Debug, Clone)]
pub struct CmaEsOptimizer {
    pub config: CmaEsConfig,
    /// 当前分布均值 m \in R^D
    pub mean: Vec<f64>,
    /// 当前全局步长 \sigma
    pub sigma: f64,
    /// 协方差对角特征值 / 缩放系数 diag(C)
    pub diag_c: Vec<f64>,
    /// 步长进化路径 p_\sigma
    pub p_sigma: Vec<f64>,
    /// 协方差进化路径 p_c
    pub p_c: Vec<f64>,
    /// 重组权重 w_1 >= w_2 >= ... >= w_\mu
    pub weights: Vec<f64>,
    /// 有效选拔量 \mu_eff
    pub mu_eff: f64,
    /// 步长适应衰减率 c_\sigma
    pub c_sigma: f64,
    /// 步长适应阻尼系数 d_\sigma
    pub d_sigma: f64,
    /// 协方差累积衰减率 c_c
    pub c_c: f64,
    /// Rank-1 学习率 c_1
    pub c_1: f64,
    /// Rank-\mu 学习率 c_\mu
    pub c_mu: f64,
    /// 标准正态分布向量期望范数 E ||N(0, I)||
    pub chi_n: f64,
    /// 当前代数
    pub generation: usize,
    /// 历史上评测过的最优个体与适应度
    pub best_candidate: Vec<f64>,
    pub best_fitness: f64,
}

impl CmaEsOptimizer {
    /// 初始化 CMA-ES 优化器
    pub fn new<R: Rng + ?Sized>(config: CmaEsConfig, rng: &mut R) -> Self {
        let d = config.dim;
        let _lambda = config.lambda;
        let mu = config.mu;

        // 1. 初始化均值 m: 在 bounds 范围内随机均匀采样
        let init_dist = Uniform::new_inclusive(config.bounds.0, config.bounds.1);
        let mean: Vec<f64> = (0..d).map(|_| init_dist.sample(rng)).collect();

        // 2. 计算对数递减重组权重 w_i = ln(mu + 0.5) - ln(i)
        let mut raw_weights = Vec::with_capacity(mu);
        for i in 1..=mu {
            let w = ((mu as f64) + 0.5).ln() - (i as f64).ln();
            raw_weights.push(w);
        }
        let sum_w: f64 = raw_weights.iter().sum();
        let weights: Vec<f64> = raw_weights.iter().map(|w| w / sum_w).collect();

        // 3. 方差有效选择量 mu_eff = 1 / sum(w_i^2)
        let sum_w_sq: f64 = weights.iter().map(|w| w * w).sum();
        let mu_eff = 1.0 / sum_w_sq;

        // 4. 超参数推导
        let c_sigma = (mu_eff + 2.0) / ((d as f64) + mu_eff + 5.0);
        let d_sigma = 1.0
            + 2.0 * (((mu_eff - 1.0) / ((d as f64) + 1.0)).sqrt() - 1.0).max(0.0)
            + c_sigma;

        let c_c = (4.0 + mu_eff / (d as f64)) / ((d as f64) + 4.0 + 2.0 * mu_eff / (d as f64));
        let c_1 = 2.0 / (((d as f64) + 1.3).powi(2) + mu_eff);
        let c_mu = (1.0 - c_1).min(
            2.0 * (mu_eff - 2.0 + 1.0 / mu_eff) / (((d as f64) + 2.0).powi(2) + mu_eff),
        );

        // 标准正态期望范数 chi_n ≈ sqrt(D) * (1 - 1/(4D) + 1/(21D^2))
        let df = d as f64;
        let chi_n = df.sqrt() * (1.0 - 1.0 / (4.0 * df) + 1.0 / (21.0 * df * df));

        Self {
            best_candidate: mean.clone(),
            best_fitness: f64::NEG_INFINITY,
            config,
            mean,
            sigma: 1.0,
            diag_c: vec![1.0; d],
            p_sigma: vec![0.0; d],
            p_c: vec![0.0; d],
            weights,
            mu_eff,
            c_sigma,
            d_sigma,
            c_c,
            c_1,
            c_mu,
            chi_n,
            generation: 0,
        }
    }

    /// 采样生成本代的 \lambda 个候选解样本 (Sample Population)
    pub fn sample_population<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec<Vec<f64>> {
        let d = self.config.dim;
        let lambda = self.config.lambda;
        let mut population = Vec::with_capacity(lambda);

        for _ in 0..lambda {
            let mut candidate = Vec::with_capacity(d);
            for j in 0..d {
                let z: f64 = rng.sample(StandardNormal);
                // x_j = mean_j + sigma * sqrt(C_jj) * z_j
                let std_j = self.diag_c[j].sqrt();
                let val = self.mean[j] + self.sigma * std_j * z;
                // 边界硬截断到合法范围
                let clamped = val.clamp(self.config.bounds.0, self.config.bounds.1);
                candidate.push(clamped);
            }
            population.push(candidate);
        }

        population
    }

    /// 根据候选解的评测适应度 (升序排序选最大) 更新优化器分布状态
    ///
    /// `candidates`: 长度为 \lambda 的样本数组
    /// `fitnesses`: 对应的适应度 (越大越优)
    pub fn update(&mut self, candidates: &[Vec<f64>], fitnesses: &[f64]) {
        assert_eq!(candidates.len(), self.config.lambda);
        assert_eq!(fitnesses.len(), self.config.lambda);

        let d = self.config.dim;
        let mu = self.config.mu;

        // 1. 根据适应度从高到低对个体索引进行排序
        let mut indices: Vec<usize> = (0..self.config.lambda).collect();
        indices.sort_by(|&a, &b| {
            fitnesses[b]
                .partial_cmp(&fitnesses[a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 记录历史最优解
        if fitnesses[indices[0]] > self.best_fitness {
            self.best_fitness = fitnesses[indices[0]];
            self.best_candidate = candidates[indices[0]].clone();
        }

        // 2. 计算新一代均值 m^(g+1) = \sum_{i=1}^\mu w_i * x_{i:\lambda}
        let old_mean = self.mean.clone();
        let mut new_mean = vec![0.0; d];
        for i in 0..mu {
            let idx = indices[i];
            let w = self.weights[i];
            for j in 0..d {
                new_mean[j] += w * candidates[idx][j];
            }
        }
        self.mean = new_mean;

        // 3. 计算均值位移步向量 y = (m^(g+1) - m^g) / \sigma
        let mut y = vec![0.0; d];
        let mut inv_sqrt_c_y = vec![0.0; d];
        for j in 0..d {
            y[j] = (self.mean[j] - old_mean[j]) / self.sigma;
            inv_sqrt_c_y[j] = y[j] / self.diag_c[j].sqrt().max(1e-8);
        }

        // 4. 更新步长路径 p_\sigma
        // p_\sigma = (1 - c_\sigma) * p_\sigma + sqrt(c_\sigma * (2 - c_\sigma) * \mu_eff) * C^{-1/2} * y
        let cs_factor = (self.c_sigma * (2.0 - self.c_sigma) * self.mu_eff).sqrt();
        let mut p_sigma_norm_sq = 0.0;
        for j in 0..d {
            self.p_sigma[j] = (1.0 - self.c_sigma) * self.p_sigma[j] + cs_factor * inv_sqrt_c_y[j];
            p_sigma_norm_sq += self.p_sigma[j] * self.p_sigma[j];
        }
        let p_sigma_norm = p_sigma_norm_sq.sqrt();

        // 5. 更新协方差路径 p_c
        // 带有启发式步长过大阻断机制
        let h_sigma = if p_sigma_norm
            / (1.0 - (1.0 - self.c_sigma).powi(2 * ((self.generation + 1) as i32))).sqrt()
            < (1.4 + 2.0 / ((d as f64) + 1.0)) * self.chi_n
        {
            1.0
        } else {
            0.0
        };

        let cc_factor = (self.c_c * (2.0 - self.c_c) * self.mu_eff).sqrt();
        for j in 0..d {
            self.p_c[j] = (1.0 - self.c_c) * self.p_c[j] + h_sigma * cc_factor * y[j];
        }

        // 6. 更新对角协方差 diag_c
        // C_diag = (1 - c_1 - c_\mu) * C_diag + c_1 * (p_c)^2 + c_\mu * \sum w_i * ((x_i - old_m)/sigma)^2
        let c_decay = 1.0 - self.c_1 - self.c_mu;
        for j in 0..d {
            let mut rank_mu_sum = 0.0;
            for i in 0..mu {
                let idx = indices[i];
                let diff_j = (candidates[idx][j] - old_mean[j]) / self.sigma;
                rank_mu_sum += self.weights[i] * diff_j * diff_j;
            }

            self.diag_c[j] = c_decay * self.diag_c[j]
                + self.c_1 * (self.p_c[j] * self.p_c[j])
                + self.c_mu * rank_mu_sum;

            // 保持正定下界
            self.diag_c[j] = self.diag_c[j].clamp(1e-6, 1e4);
        }

        // 7. 更新全局步长 \sigma
        // \sigma = \sigma * exp( (c_\sigma / d_\sigma) * (||p_\sigma|| / chi_n - 1) )
        let step_ratio = (p_sigma_norm / self.chi_n - 1.0) * (self.c_sigma / self.d_sigma);
        self.sigma *= step_ratio.clamp(-0.5, 0.5).exp();
        self.sigma = self.sigma.clamp(1e-4, 5.0);

        self.generation += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_cma_es_sphere_optimization() {
        // 使用简单的多维球函数测试 CMA-ES 收敛性: f(x) = - \sum x_i^2 (最大化向 0 收敛)
        let config = CmaEsConfig {
            dim: 5,
            lambda: 14,
            mu: 7,
            sigma0: 1.0,
            bounds: (-5.0, 5.0),
            max_generations: 50,
            repeats_per_eval: 1,
        };

        let mut rng = StdRng::seed_from_u64(12345);
        let mut opt = CmaEsOptimizer::new(config, &mut rng);

        let init_best = opt.best_fitness;

        for _ in 0..30 {
            let pop = opt.sample_population(&mut rng);
            let fits: Vec<f64> = pop
                .iter()
                .map(|cand| {
                    let sum_sq: f64 = cand.iter().map(|&x| x * x).sum();
                    -sum_sq
                })
                .collect();
            opt.update(&pop, &fits);
        }

        // 适应度应显著上升 (接近 0.0)
        assert!(opt.best_fitness > init_best);
        assert!(opt.best_fitness > -1.0);
    }
}
