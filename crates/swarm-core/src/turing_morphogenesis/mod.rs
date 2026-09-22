//! Science Robotics 2018 群体机器人形态发生系统 (Turing Morphogenesis)
//!
//! 论文: *Morphogenesis in robot swarms* (Science Robotics 2018, eaau9178)
//!
//! 模块包含：
//! - `morphogen`: 激活子与抑制子反应-扩散动力学、图拉普拉斯与 LED 颜色映射
//! - `edge_detector`: 基于局域加权邻居之比的自适应边缘探测
//! - `robot`: Kilobot 个体、三态行为状态机 (Wait / Orbit / Follow)
//! - `simulator`: 多智能体离散事件世界、轮廓环绕运动学、软碰撞排斥与截肢自愈

pub mod edge_detector;
pub mod morphogen;
pub mod robot;
pub mod simulator;

pub use edge_detector::EdgeDetector;
pub use morphogen::{LedColor, MorphogenConcentration, MorphogenParams};
pub use robot::{BotState, Kilobot, NeighborObservation};
pub use simulator::{MorphogenesisSwarm, SwarmMetrics};

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_morphogen_reaction_kinetics() {
        let params = MorphogenParams::default();
        let conc = MorphogenConcentration::new(1.0, 1.0);
        let (rate_u, rate_v) = conc.reaction_rates(&params);

        // a=0.08, b=-0.08, c=0.03, d=0.03 -> rate_u = (0.08 - 0.08 + 0.03) - 0.03 = 0.0
        // e=0.10, f=0.12, g=0.06 -> rate_v = clamp(0.10 - 0.12, 0) - 0.06 = -0.06
        assert!((rate_u - 0.0).abs() < 1e-4);
        assert!((rate_v - (-0.06)).abs() < 1e-4);
    }

    #[test]
    fn test_edge_detector_ratio() {
        let mut detector = EdgeDetector::new(0.5, 0.8);
        // 外部边缘节点：自身邻居 3 个，邻居的邻居平均有 6 个
        let n_neighbors = 3;
        let neighbors_info = vec![(40.0, 6), (45.0, 6), (50.0, 6)];

        for _ in 0..10 {
            detector.update(n_neighbors, &neighbors_info);
        }

        assert!(detector.is_edge(), "Ratio 3/6 = 0.5 < 0.8 应判定为边缘节点");
    }

    #[test]
    fn test_robot_state_machine_capture() {
        let mut bot = Kilobot::new(0, [0.0, 0.0], 1.0, 1.0);
        bot.state = BotState::Orbit;

        // 遭遇极化斑点：距离 < dist_crit 且至少有两个极化邻居
        let polar_conc = MorphogenConcentration::new(5.0, 1.0); // u=5.0 > 4.0 极化
        let neighbors = vec![
            NeighborObservation {
                id: 1,
                dist: 35.0,
                state: BotState::Wait,
                morphogen: polar_conc,
                n_neighbors: 5,
            },
            NeighborObservation {
                id: 2,
                dist: 38.0,
                state: BotState::Wait,
                morphogen: polar_conc,
                n_neighbors: 5,
            },
        ];

        bot.evaluate_state_transitions(&neighbors, 45.0, 4.0);
        assert_eq!(
            bot.state,
            BotState::Wait,
            "到达图灵极化斑点应终止环绕被捕获转为 WAIT 态"
        );
    }

    #[test]
    fn test_morphogenesis_pile_simulation() {
        let mut rng = StdRng::seed_from_u64(42);
        let params = MorphogenParams::default();
        let mut swarm = MorphogenesisSwarm::new_pile(50, [0.0, 0.0], 80.0, params, &mut rng);

        assert_eq!(swarm.robots.len(), 50);

        // 先执行若干步纯扩散
        swarm.step_diffusion_only(5, 0.05);
        assert_eq!(swarm.step_count, 5);

        // 运行形态发生组合步
        for _ in 0..10 {
            swarm.step(0.05);
        }

        let m = swarm.metrics();
        assert_eq!(m.total_robots, 50);
        assert!(m.gyration_radius > 0.0);
    }

    #[test]
    fn test_amputation_recovery() {
        let mut rng = StdRng::seed_from_u64(123);
        let params = MorphogenParams::default();
        let mut swarm = MorphogenesisSwarm::new_pile(60, [0.0, 0.0], 80.0, params, &mut rng);

        // 切除右半侧机器人 (x > 20.0)
        let removed = swarm.amputate(|bot| bot.pos[0] > 20.0);
        assert!(removed > 0, "应切除部分处于正半轴的机器人");
        assert_eq!(swarm.robots.len(), 60 - removed);

        // 截肢后继续仿真若干步
        for _ in 0..10 {
            swarm.step(0.05);
        }
        let m = swarm.metrics();
        assert_eq!(m.total_robots, 60 - removed);
    }
}
