//! # Provable Swarm Robotics Pattern Formation
//!
//! 基于 Coppola et al. (*Swarm Intelligence*, 2019 / [10.1007/s11721-019-00163-0](https://doi.org/10.1007/s11721-019-00163-0))
//! 的极简认知（无通信、无身份、无记忆、无全局坐标）群体机器人自组织构型系统：
//!
//! - `state`: 8-邻域 Moore 网格方向与 256 种二值局部状态 `LocalState`、单纯形状态判断
//! - `policy`: 自动生成安全策略 $\Pi_{\text{safe}}$ 与构型策略 $\Pi_f$、几何构型定义与期望状态提取
//! - `simulator`: 离散多智能体网格仿真器 `SwarmWorld`（支持 Baseline / ALT1 / ALT2）

pub mod policy;
pub mod simulator;
pub mod state;

pub use policy::{Pattern, Policy};
pub use simulator::{ExecutionMode, StepResult, SwarmWorld};
pub use state::{Direction, LocalState};

#[cfg(test)]
pub mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_direction_and_local_state() {
        let dir = Direction::North;
        assert_eq!(dir.offset(), (0, 1));
        assert_eq!(dir.opposite(), Direction::South);
        assert_eq!(Direction::East.opposite(), Direction::West);

        let mut state = LocalState::EMPTY;
        assert_eq!(state.neighbor_count(), 0);
        state.set_neighbor(Direction::North, true);
        state.set_neighbor(Direction::East, true);
        assert_eq!(state.neighbor_count(), 2);
        assert!(state.has_neighbor(Direction::North));
        assert!(state.has_neighbor(Direction::East));
        assert!(!state.has_neighbor(Direction::South));
    }

    #[test]
    fn test_simplicial_state_classification() {
        // 单个邻居天然构成 1 个团簇 -> Simplicial
        let mut s1 = LocalState::EMPTY;
        s1.set_neighbor(Direction::North, true);
        assert!(s1.is_simplicial());

        // 两个紧邻邻居 (North + NE) 互相连通 -> Simplicial
        let mut s2 = LocalState::EMPTY;
        s2.set_neighbor(Direction::North, true);
        s2.set_neighbor(Direction::NorthEast, true);
        assert_eq!(s2.count_cliques(), 1);
        assert!(s2.is_simplicial());

        // 两个相对不连通邻居 (North + South) 互不相连 -> 2 cliques -> Non-simplicial
        let mut s3 = LocalState::EMPTY;
        s3.set_neighbor(Direction::North, true);
        s3.set_neighbor(Direction::South, true);
        assert_eq!(s3.count_cliques(), 2);
        assert!(!s3.is_simplicial());
    }

    #[test]
    fn test_safe_policy_collision_and_separation() {
        let safe = Policy::compute_safe_policy();

        // 当只有一个邻居在 North 时:
        let mut s = LocalState::EMPTY;
        s.set_neighbor(Direction::North, true);

        let allowed = &safe[s.0 as usize];
        // 绝不可撞向 North
        assert!(!allowed.contains(&Direction::North));
        // 移动到离 North 距离 <= 1 的格子 (NE, NW) 是安全的
        assert!(allowed.contains(&Direction::NorthEast));
        assert!(allowed.contains(&Direction::NorthWest));
        // 移动到 South (0, -2) 会断开与 North (0, 1) 的连通性，必须被安全过滤！
        assert!(!allowed.contains(&Direction::South));
    }

    #[test]
    fn test_desired_state_extraction() {
        let tri4 = Pattern::triangle_4();
        let s_des_tri = tri4.extract_desired_states();
        assert!(!s_des_tri.is_empty());
        // Triangle-4 在固定北向参考系下，4 个机器人各自具有独特的局部拓扑感知
        assert_eq!(s_des_tri.len(), 4);

        let sq4 = Pattern::square_4();
        let s_des_sq = sq4.extract_desired_states();
        // 2x2 正方形 4 个角落旋转等价，但在固定北向坐标系下具有各自独特的方向特征
        assert_eq!(s_des_sq.len(), 4);
    }

    #[test]
    fn test_triangle_4_self_organization() {
        let tri4 = Pattern::triangle_4();
        let policy = Policy::from_pattern(&tri4);

        let mut rng = StdRng::seed_from_u64(42);

        // 测试 5 次随机初始形态下的收敛情况
        for _ in 0..5 {
            let mut world = SwarmWorld::random_connected(4, policy.clone(), ExecutionMode::Alt1, &mut rng);
            let converged = world.run_until_converged(2000, &mut rng);
            assert!(converged, "Triangle-4 集群应当收敛到期望构型！");
            assert!(world.is_converged());
        }
    }
}
