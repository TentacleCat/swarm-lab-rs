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

pub mod llm_model;
pub mod metrics;
pub mod model;
pub mod network;

pub use llm_model::{ChannelOutcome, LlmArchitecture, LlmInteractionResult, LlmNamingGame};
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

    // =========================================================================
    // 🎯 [关卡 7 单元测试] 大模型命名博弈四通道动力学与平均场相变
    // =========================================================================

    #[test]
    fn test_llm_task1_rates() {
        // LLaMA-3.1:8B 在 T=1.0: pi = 1.0 - 0.225 = 0.775, phi = 0.05 + 0.225 = 0.275
        let (pi_llama, phi_llama) = LlmArchitecture::Llama3_1_8B.get_rates(1.0);
        assert!((pi_llama - 0.775).abs() < 1e-4, "LLaMA T=1.0 pi 计算错误");
        assert!((phi_llama - 0.275).abs() < 1e-4, "LLaMA T=1.0 phi 计算错误");

        // Mistral:7B 温度盲性: pi = 0.99
        let (pi_mistral, phi_mistral) = LlmArchitecture::Mistral7B.get_rates(0.0);
        assert_eq!(pi_mistral, 0.99, "Mistral pi 应为 0.99");
        assert!((phi_mistral - 0.02).abs() < 1e-4, "Mistral phi(0) 应为 0.02");

        // Phi-3:14B 在 T=0.0: pi = 0.76, phi = 0.01
        let (pi_phi, phi_phi) = LlmArchitecture::Phi3_14B.get_rates(0.0);
        assert!((pi_phi - 0.76).abs() < 1e-4, "Phi-3 T=0.0 pi 应为 0.76");
        assert!((phi_phi - 0.01).abs() < 1e-4, "Phi-3 T=0.0 phi 应为 0.01");

        // 截断测试: T > 2.5 应被截断在 2.5
        let (pi_clamp, phi_clamp) = LlmArchitecture::Llama3_1_8B.get_rates(10.0);
        assert!((pi_clamp - 0.55).abs() < 1e-4, "LLaMA 低温下界截断应为 0.55");
        assert!((phi_clamp - 0.50).abs() < 1e-4, "LLaMA 高温上界截断应为 0.50");
    }

    #[test]
    fn test_llm_task2_step_mechanism() {
        let mut rng = StdRng::seed_from_u64(1234);
        let graph = CompleteGraph::new(2);

        // 设定极端情况：pi=1.0 (库内必回答 YES), phi=0.0 (库外必回答 NO)
        let mut game = LlmNamingGame::new(
            graph,
            LlmArchitecture::CustomTwoRate { pi: 1.0, phi: 0.0 },
            0.0,
        );
        game.pi = 1.0;
        game.phi = 0.0;

        // 步骤 1: 两人初态为空，Speaker 发明词汇并传达给 Hearer
        // 词汇不在 Hearer 库内，由于 phi=0，必触发 TrueNegative (Hearer 收纳该词汇)
        let res1 = game.step(&mut rng);
        assert_eq!(res1.channel, ChannelOutcome::TrueNegative);
        assert!(!res1.collapse_triggered);
        assert_eq!(game.inventories[res1.hearer].len(), 1);

        // 步骤 2: 此时双方均已掌握该词汇，下次交互必为库内且 pi=1.0，必触发 TruePositive (坍缩)
        let res2 = game.step(&mut rng);
        assert_eq!(res2.channel, ChannelOutcome::TruePositive);
        assert!(res2.collapse_triggered);
        assert!(game.is_consensus());
    }

    #[test]
    fn test_llm_task3_drift_and_fraction() {
        let mut rng = StdRng::seed_from_u64(42);
        let graph = CompleteGraph::new(5);
        let mut game = LlmNamingGame::new(
            graph,
            LlmArchitecture::CustomTwoRate { pi: 0.8, phi: 0.2 },
            0.5,
        );
        game.pi = 0.8;
        game.phi = 0.2;

        for _ in 0..100 {
            game.step(&mut rng);
        }

        let m = game.in_inventory_fraction();
        assert!(m >= 0.0 && m <= 1.0, "在库比例 m(t) 必须位于 [0, 1] 区间");

        let drift = game.drift_proxy();
        let expected_drift = m * 0.8 - (1.0 - m) * 0.2;
        assert!((drift - expected_drift).abs() < 1e-6, "漂移算子 Delta(t) 计算错误");
    }

    #[test]
    fn test_llm_task4_ordering_parameter() {
        let graph = CompleteGraph::new(4);

        // 1. 验证临界有序指标 R = 3*pi - 2*phi - 1
        let mut game_ordered = LlmNamingGame::new(
            graph.clone(),
            LlmArchitecture::CustomTwoRate { pi: 0.9, phi: 0.1 },
            0.5,
        );
        game_ordered.pi = 0.9;
        game_ordered.phi = 0.1;
        // R = 3*0.9 - 2*0.1 - 1 = 2.7 - 0.2 - 1 = 1.5 > 0
        assert!((game_ordered.ordering_parameter_r() - 1.5).abs() < 1e-6);

        let mut game_disordered = LlmNamingGame::new(
            graph,
            LlmArchitecture::CustomTwoRate { pi: 0.3, phi: 0.4 },
            1.5,
        );
        game_disordered.pi = 0.3;
        game_disordered.phi = 0.4;
        // R = 3*0.3 - 2*0.4 - 1 = 0.9 - 0.8 - 1 = -0.9 < 0
        assert!((game_disordered.ordering_parameter_r() - (-0.9)).abs() < 1e-6);
    }
}

