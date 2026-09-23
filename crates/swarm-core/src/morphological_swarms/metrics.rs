#![allow(unused_variables, dead_code, unused_imports)]

//! # Morphological Swarm Metrics (arXiv:2601.07610)
//!
//! 论文核心宏观与微观序参量计算：
//! 1. `light_occupancy_ratio`: 光照聚集比例 $N_\circ / N$ (与无相互作用随机游走基线 0.18 对比)
//! 2. `polar_alignment`: 宏观极化度 / 总对齐序参量 $\langle \Psi \rangle = \frac{1}{N} |\sum_i \vec{n}_i|$
//! 3. `contact_pairs_count`: 处于 WCA 碰撞斥力范围内的接触对总数
//! 4. `cluster_size_statistics`: 连通接触团簇的最大规模与平均规模

use super::model::{MorphologicalSwarm, WCA_CUTOFF};

/// 宏观序参量快照
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwarmMetrics {
    /// 光照区机器人占比 $N_\circ / N \in [0, 1]$
    pub light_ratio: f64,
    /// 宏观极化对齐序参量 $\langle \Psi \rangle \in [0, 1]$
    pub polar_alignment: f64,
    /// 处于物理碰撞接触的机器人对数
    pub contact_pairs: usize,
    /// 最大聚集团簇大小
    pub max_cluster_size: usize,
    /// 平均粒子标量速率 $\langle |v| \rangle$
    pub mean_speed: f64,
}

impl MorphologicalSwarm {
    /// 【关卡 13 - 任务 4】计算当前光照区机器人数量占比 $N_\circ / N$
    ///
    /// # 提示
    /// - 若粒子列表为空，直接返回 0.0；
    /// - 统计 `p.in_light == true` 的粒子数除以总粒子数；
    /// - 若卡壳可参考 [`crates/swarm-core/src/reference/morphological_swarms.rs`](../reference/morphological_swarms.rs)。
    pub fn light_occupancy_ratio(&self) -> f64 {
        todo!("【关卡 13 - 任务 4】在 metrics.rs 中实现光照聚集比例计算 light_occupancy_ratio");
    }

    /// 【关卡 13 - 任务 4】计算宏观极化对齐序参量 $\langle \Psi \rangle = \frac{1}{N} |\sum_{i=1}^N \vec{n}_i|$
    ///
    /// # 提示
    /// - 累加所有粒子的朝向向量分量 `sum_x += p.mu[0], sum_y += p.mu[1]`；
    /// - 取向量模长除以粒子总数 N：`sqrt(sum_x^2 + sum_y^2) / N`。
    pub fn polar_alignment(&self) -> f64 {
        todo!("【关卡 13 - 任务 4】在 metrics.rs 中实现宏观极化对齐序参量计算 polar_alignment");
    }

    /// 计算所有粒子的平均瞬时运动速率
    pub fn mean_speed(&self) -> f64 {
        let n = self.particles.len();
        if n == 0 {
            return 0.0;
        }
        let total: f64 = self.particles.iter().map(|p| p.speed()).sum();
        total / (n as f64)
    }

    /// 计算当前接触网络中的碰撞对数与最大团簇规模
    pub fn contact_cluster_statistics(&self) -> (usize, usize) {
        let n = self.particles.len();
        if n == 0 {
            return (0, 0);
        }

        let l = self.params.half_box_l;
        let contact_dist_sq = (self.params.sigma * WCA_CUTOFF).powi(2);

        // 构建邻接表
        let mut adj = vec![Vec::new(); n];
        let mut contact_pairs = 0;

        for i in 0..n {
            for j in (i + 1)..n {
                let (dx, dy) = Self::minimum_image_displacement(
                    self.particles[i].pos,
                    self.particles[j].pos,
                    l,
                );
                if dx * dx + dy * dy < contact_dist_sq {
                    adj[i].push(j);
                    adj[j].push(i);
                    contact_pairs += 1;
                }
            }
        }

        // BFS 连通分量求最大团簇
        let mut visited = vec![false; n];
        let mut max_cluster_size = 1;

        for i in 0..n {
            if !visited[i] {
                let mut size = 0;
                let mut queue = std::collections::VecDeque::new();
                queue.push_back(i);
                visited[i] = true;

                while let Some(node) = queue.pop_front() {
                    size += 1;
                    for &neighbor in &adj[node] {
                        if !visited[neighbor] {
                            visited[neighbor] = true;
                            queue.push_back(neighbor);
                        }
                    }
                }

                if size > max_cluster_size {
                    max_cluster_size = size;
                }
            }
        }

        (contact_pairs, max_cluster_size)
    }

    /// 一次性提取全部关键指标
    pub fn evaluate_metrics(&self) -> SwarmMetrics {
        let (contact_pairs, max_cluster_size) = self.contact_cluster_statistics();
        SwarmMetrics {
            light_ratio: self.light_occupancy_ratio(),
            polar_alignment: self.polar_alignment(),
            contact_pairs,
            max_cluster_size,
            mean_speed: self.mean_speed(),
        }
    }
}
