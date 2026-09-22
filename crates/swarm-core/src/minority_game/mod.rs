//! # 少数派博弈模块 (Minority Game & Herding Behavior)
//!
//! 论文: "Minority game with local interactions due to the presence of herding behavior" (arXiv:physics/0512087)
//! 本地文档: `papers/minority-game/01-herding-behavior-physics0512087/README.md`

pub mod metrics;
pub mod model;
pub mod strategy;

pub use metrics::{alpha_parameter, normalized_volatility};
pub use model::MinorityGame;
pub use strategy::{MgAgent, Strategy};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::naming_game::network::{AdjacencyGraph, CompleteGraph};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_strategy_and_score() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut strat = Strategy::random(3, &mut rng);
        assert_eq!(strat.actions.len(), 8);
        assert_eq!(strat.virtual_score, 0);

        let pred = strat.predict(2);
        strat.update_score(pred, 2);
        assert_eq!(strat.virtual_score, 1);

        strat.update_score(-pred, 2);
        assert_eq!(strat.virtual_score, 0);
    }

    #[test]
    fn test_minority_game_standard_step() {
        let mut rng = StdRng::seed_from_u64(123);
        let n = 11;
        let mut mg = MinorityGame::<CompleteGraph>::new(n, 3, 2, None, false, &mut rng);

        for _ in 0..100 {
            let att = mg.step(&mut rng);
            assert!(att.abs() <= n as i32);
        }
        assert_eq!(mg.time_step, 100);
    }

    #[test]
    fn test_herding_imitation_behavior() {
        let mut rng = StdRng::seed_from_u64(999);
        // 2 个人全连通
        let net = AdjacencyGraph::regular_ring(4, 2);
        let mut mg = MinorityGame::new(4, 2, 2, Some(net), true, &mut rng);

        // 强行把智能体 0 的第一条策略积分拔高到 100
        mg.agents[0].strategies[0].virtual_score = 100;

        let att = mg.step(&mut rng);
        assert!(att.abs() <= 4);
    }

    #[test]
    fn test_volatility_metric() {
        let sample = vec![1, -1, 1, -1, 1, -1, 1, -1];
        let vol = normalized_volatility(&sample, 1);
        assert!((vol - 1.0).abs() < 1e-6);
    }
}
