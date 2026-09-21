//! # 🎯 [实战关卡 6] Naming Game 离散多智能体系统与微观动力学
//!
//! 论文: "Microscopic activity patterns in the Naming Game" (Luca Dall'Asta, Andrea Baronchelli, cond-mat/0606125)
//! 本地笔记: `papers/naming-game/01-microscopic-activity-condmat2006/README.md`
//!
//! 在这里你将学习并亲手实现：
//! 1. 复杂网络（完全图、ER 随机图、BA 无标度网络）的拓扑抽象；
//! 2. 基于哈希集合（`HashSet`）的多智能体离散词库管理与谈判更新机制；
//! 3. 宏观序参量（全网词汇量 $N_w$、不同词汇量 $N_d$、共识判定）；
//! 4. 微观统计分布 $\mathcal{P}_n(k | t)$ 的直方图构建与归一化。
//!
//! 遇到卡壳可查阅参考答案: `crates/swarm-core/src/reference/naming_game.rs`

pub mod metrics;
pub mod model;
pub mod network;

pub use metrics::{distinct_words, inventory_size_distribution, is_consensus, total_words};
pub use model::{AgentId, InteractionResult, NamingGame, WordId};
pub use network::{AdjacencyGraph, CompleteGraph, Network};

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::collections::HashSet;

    #[test]
    fn test_metrics_total_and_distinct() {
        let invs = vec![
            HashSet::from([1, 2]),
            HashSet::from([2, 3]),
            HashSet::from([1, 3, 4]),
            HashSet::new(),
        ];
        // 词汇总数: 2 + 2 + 3 + 0 = 7
        assert_eq!(total_words(&invs), 7);
        // 不同词汇总数: {1, 2, 3, 4} = 4
        assert_eq!(distinct_words(&invs), 4);
    }

    #[test]
    fn test_metrics_inventory_size_distribution() {
        let invs = vec![
            HashSet::from([1]),       // size 1
            HashSet::from([1, 2]),    // size 2
            HashSet::from([1, 2, 3]), // size 3
            HashSet::from([4]),       // size 1
        ];
        let dist = inventory_size_distribution(&invs, None);
        // size 1 出现 2 次 (概率 0.5)
        // size 2 出现 1 次 (概率 0.25)
        // size 3 出现 1 次 (概率 0.25)
        assert_eq!(dist.get(&1).copied(), Some(0.5));
        assert_eq!(dist.get(&2).copied(), Some(0.25));
        assert_eq!(dist.get(&3).copied(), Some(0.25));
        assert_eq!(dist.get(&4).copied(), None);

        // 测试带有节点过滤
        let filter = vec![0, 1]; // 只统计节点 0 和 1
        let dist_filtered = inventory_size_distribution(&invs, Some(&filter));
        assert_eq!(dist_filtered.get(&1).copied(), Some(0.5));
        assert_eq!(dist_filtered.get(&2).copied(), Some(0.5));
    }

    #[test]
    fn test_metrics_is_consensus() {
        let not_consensus = vec![
            HashSet::from([1]),
            HashSet::from([2]),
        ];
        assert!(!is_consensus(&not_consensus));

        let not_consensus_size = vec![
            HashSet::from([1, 2]),
            HashSet::from([1, 2]),
        ];
        assert!(!is_consensus(&not_consensus_size));

        let consensus = vec![
            HashSet::from([42]),
            HashSet::from([42]),
            HashSet::from([42]),
        ];
        assert!(is_consensus(&consensus));
    }

    #[test]
    fn test_naming_game_step_rules() {
        let mut rng = StdRng::seed_from_u64(999);
        let graph = CompleteGraph::new(2);
        let mut game = NamingGame::new(graph);

        // 初始状态：两人词库均为空
        assert_eq!(game.inventories[0].len(), 0);
        assert_eq!(game.inventories[1].len(), 0);

        // 第一步博弈：Speaker 必定发明一个新词汇（例如 1），并传给 Hearer
        // Hearer 初始为空，必定 failure，将该词汇收录
        let res1 = game.step(&mut rng);
        assert!(!res1.success, "第一步博弈双方初态为空，必定谈判失败");
        assert_eq!(game.inventories[res1.speaker].len(), 1);
        assert_eq!(game.inventories[res1.hearer].len(), 1);
        assert_eq!(game.inventories[res1.speaker], game.inventories[res1.hearer]);

        // 第二步博弈：两人词库已经包含相同唯一词汇，必定 success
        let res2 = game.step(&mut rng);
        assert!(res2.success, "第二步博弈双方已掌握该词汇，必定谈判成功");
        assert!(is_consensus(&game.inventories));
    }

    #[test]
    fn test_naming_game_convergence_small() {
        let mut rng = StdRng::seed_from_u64(42);
        let graph = CompleteGraph::new(8);
        let mut game = NamingGame::new(graph);

        let mut converged = false;
        for _ in 0..20_000 {
            game.step(&mut rng);
            if is_consensus(&game.inventories) {
                converged = true;
                break;
            }
        }
        assert!(converged, "8 个智能体的完全图在 20,000 步内必能达成全网共识");
    }
}
