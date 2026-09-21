//! # 💡 [参考答案区] Naming Game 标准参考实现 (Reference Implementation)
//!
//! 本文件包含已经过完备单测验证的标准实现。卡壳时可对照查阅。

use crate::naming_game::network::Network;
use rand::Rng;
use std::collections::{BTreeMap, HashSet};

pub type WordId = u32;
pub type AgentId = usize;

/// 单次谈判博弈的微观交互结果快照
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteractionResult {
    pub speaker: AgentId,
    pub hearer: AgentId,
    pub transmitted_word: WordId,
    pub success: bool,
}

/// Naming Game 离散多智能体系统标准实现
#[derive(Clone, Debug)]
pub struct NamingGameReference<G: Network> {
    pub network: G,
    pub inventories: Vec<HashSet<WordId>>,
    pub next_word_id: WordId,
    pub time_step: usize,
    pub total_successes: usize,
}

impl<G: Network> NamingGameReference<G> {
    pub fn new(network: G) -> Self {
        let n = network.num_nodes();
        Self {
            network,
            inventories: vec![HashSet::new(); n],
            next_word_id: 1,
            time_step: 0,
            total_successes: 0,
        }
    }

    #[inline]
    pub fn num_agents(&self) -> usize {
        self.network.num_nodes()
    }

    /// 执行一步 Direct Naming Game 博弈标准实现
    pub fn step<R: Rng>(&mut self, rng: &mut R) -> InteractionResult {
        let n = self.num_agents();

        // 1. 均匀抽取 Speaker
        let speaker = rng.gen_range(0..n);

        // 2. 从 Speaker 的邻居中均匀抽取 Hearer
        let hearer = self.network.random_neighbor(speaker, rng);

        // 3. 说话者传词
        if self.inventories[speaker].is_empty() {
            let new_word = self.next_word_id;
            self.next_word_id += 1;
            self.inventories[speaker].insert(new_word);
        }

        // 从 Speaker 词库中随机选择一个词汇
        let speaker_inv = &self.inventories[speaker];
        let chosen_idx = rng.gen_range(0..speaker_inv.len());
        let word = *speaker_inv.iter().nth(chosen_idx).unwrap();

        // 4. 听者匹配与更新
        let hearer_has_word = self.inventories[hearer].contains(&word);

        if hearer_has_word {
            // 谈判成功：双方词库清空重置为 {word}
            self.inventories[speaker].clear();
            self.inventories[speaker].insert(word);

            self.inventories[hearer].clear();
            self.inventories[hearer].insert(word);

            self.total_successes += 1;
        } else {
            // 谈判失败：听者收录新词汇，说话者不变
            self.inventories[hearer].insert(word);
        }

        self.time_step += 1;

        InteractionResult {
            speaker,
            hearer,
            transmitted_word: word,
            success: hearer_has_word,
        }
    }
}

/// 全网词汇总量 $N_w(t) = \sum |V_i|$
pub fn total_words_ref(inventories: &[HashSet<WordId>]) -> usize {
    inventories.iter().map(|inv| inv.len()).sum()
}

/// 全网不同词汇种类数 $N_d(t) = |\bigcup V_i|$
pub fn distinct_words_ref(inventories: &[HashSet<WordId>]) -> usize {
    let all: HashSet<WordId> = inventories.iter().flatten().copied().collect();
    all.len()
}

/// 微观词汇量分布 $\mathcal{P}_n(k | t)$
pub fn inventory_size_distribution_ref(
    inventories: &[HashSet<WordId>],
    filter_nodes: Option<&[usize]>,
) -> BTreeMap<usize, f64> {
    let mut counts: BTreeMap<usize, usize> = BTreeMap::new();
    let total_counted;

    match filter_nodes {
        Some(nodes) => {
            total_counted = nodes.len();
            for &idx in nodes {
                let sz = inventories[idx].len();
                *counts.entry(sz).or_insert(0) += 1;
            }
        }
        None => {
            total_counted = inventories.len();
            for inv in inventories {
                let sz = inv.len();
                *counts.entry(sz).or_insert(0) += 1;
            }
        }
    }

    if total_counted == 0 {
        return BTreeMap::new();
    }

    counts
        .into_iter()
        .map(|(sz, cnt)| (sz, cnt as f64 / total_counted as f64))
        .collect()
}

/// 检查是否达成全网单一共识
pub fn is_consensus_ref(inventories: &[HashSet<WordId>]) -> bool {
    if inventories.is_empty() {
        return false;
    }
    let first = &inventories[0];
    if first.len() != 1 {
        return false;
    }
    let consensus_word = *first.iter().next().unwrap();
    inventories
        .iter()
        .all(|inv| inv.len() == 1 && inv.contains(&consensus_word))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::naming_game::network::CompleteGraph;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_reference_step_and_metrics() {
        let mut rng = StdRng::seed_from_u64(42);
        let graph = CompleteGraph::new(5);
        let mut game = NamingGameReference::new(graph);

        // 初始状态
        assert_eq!(total_words_ref(&game.inventories), 0);
        assert_eq!(distinct_words_ref(&game.inventories), 0);
        assert!(!is_consensus_ref(&game.inventories));

        // 进行 100 步博弈
        for _ in 0..100 {
            game.step(&mut rng);
        }

        let nw = total_words_ref(&game.inventories);
        let nd = distinct_words_ref(&game.inventories);
        assert!(nw > 0);
        assert!(nd > 0);

        let dist = inventory_size_distribution_ref(&game.inventories, None);
        let sum_p: f64 = dist.values().sum();
        assert!((sum_p - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_reference_consensus() {
        let mut rng = StdRng::seed_from_u64(123);
        let graph = CompleteGraph::new(6);
        let mut game = NamingGameReference::new(graph);

        // 小网络必定能较快收敛
        let mut converged = false;
        for _ in 0..10_000 {
            game.step(&mut rng);
            if is_consensus_ref(&game.inventories) {
                converged = true;
                break;
            }
        }
        assert!(converged, "小规模完全图应在 10000 步内收敛达到共识");
        assert_eq!(distinct_words_ref(&game.inventories), 1);
        assert_eq!(total_words_ref(&game.inventories), 6);
    }
}
