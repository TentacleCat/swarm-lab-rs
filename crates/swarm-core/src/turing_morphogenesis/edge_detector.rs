//! 局域自适应边缘检测器 (Local Edge Detector)
//!
//! 依据 Science Robotics 2018 原理：
//! 机器人无需任何全局坐标系或外部俯视摄像头，
//! 仅通过自身邻居数与邻居的距离加权邻居均值之比，
//! 即可判断自己是位于集群内部还是暴露在外边缘。

use serde::{Deserialize, Serialize};

/// 局域边缘检测器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDetector {
    /// 自身邻居数量的指数移动平均 (Running average of Ns)
    pub running_avg_ns: f64,
    /// 邻居的邻居数量的加权指数移动平均 (Running average of NNs)
    pub running_avg_nns: f64,
    /// 指数平滑滤波因子 alpha (原物理机器人物理周期较长取 0.0001，离散仿真可设 0.05~0.1)
    pub alpha: f64,
    /// 边缘判定比值阈值 (原代码 EDGE_TH = 0.80)
    pub edge_th: f64,
}

impl Default for EdgeDetector {
    fn default() -> Self {
        Self {
            running_avg_ns: 0.0,
            running_avg_nns: 0.0,
            alpha: 0.05,
            edge_th: 0.80,
        }
    }
}

impl EdgeDetector {
    /// 新建边缘检测器
    pub fn new(alpha: f64, edge_th: f64) -> Self {
        Self {
            running_avg_ns: 0.0,
            running_avg_nns: 0.0,
            alpha,
            edge_th,
        }
    }

    /// 使用邻居观测数据重置/初始化平滑均值
    pub fn initialize(&mut self, my_neighbors: usize, neighbors_info: &[(f64, usize)]) {
        self.running_avg_ns = my_neighbors as f64;
        self.running_avg_nns = Self::compute_weighted_nns(neighbors_info);
    }

    /// 计算邻居的距离反比加权邻居均值
    ///
    /// w_i = 1 / dist_i
    /// avg_NNs = sum(w_i * N_i) / sum(w_i)
    pub fn compute_weighted_nns(neighbors_info: &[(f64, usize)]) -> f64 {
        if neighbors_info.is_empty() {
            return 0.0;
        }
        let mut sum = 0.0;
        let mut w_sum = 0.0;
        for &(dist, n_neighbors) in neighbors_info {
            let w = 1.0 / dist.max(1.0);
            sum += w * (n_neighbors as f64);
            w_sum += w;
        }
        if w_sum > 1e-6 {
            sum / w_sum
        } else {
            0.0
        }
    }

    /// 单步观测更新滑动均值
    pub fn update(&mut self, my_neighbors: usize, neighbors_info: &[(f64, usize)]) {
        let current_n = my_neighbors as f64;
        let current_nns = Self::compute_weighted_nns(neighbors_info);

        self.running_avg_ns = self.alpha * current_n + (1.0 - self.alpha) * self.running_avg_ns;
        self.running_avg_nns = self.alpha * current_nns + (1.0 - self.alpha) * self.running_avg_nns;
    }

    /// 判定是否处于集群边缘 (Edge)
    ///
    /// 当 running_avg_Ns / running_avg_NNs < edge_th 时判定为边缘
    pub fn is_edge(&self) -> bool {
        if self.running_avg_nns < 1e-3 {
            return true;
        }
        (self.running_avg_ns / self.running_avg_nns) < self.edge_th
    }

    /// 获取当前边缘比率
    pub fn edge_ratio(&self) -> f64 {
        if self.running_avg_nns < 1e-3 {
            0.0
        } else {
            self.running_avg_ns / self.running_avg_nns
        }
    }
}
