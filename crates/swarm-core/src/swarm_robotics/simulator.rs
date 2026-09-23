//! # Swarm Grid Simulator
//!
//! 实现离散网格上的异步多智能体群聚构型仿真器 `SwarmWorld`：
//! - 遵循 C1~C10 认知约束与 A1~A4 假设；
//! - 支持 Baseline（标准随机异步决策）、ALT1（冷却防颠簸启发式）与 ALT2（限制拥挤节点启发式）；
//! - 自动生成初始无碰撞单连通拓扑 $P_0$；
//! - 记录构型演化时间步数与碰撞/断连安全检查。

#![allow(unused_variables, dead_code)]

use super::policy::Policy;
use super::state::{Direction, LocalState};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashSet;

/// 决策执行模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// 标准基准模式（完全随机在所有活跃个体中抽选）
    Baseline,
    /// 启发式 ALT1：刚移动过的机器人在下一步不优先移动（防止局部来回颠簸震荡）
    Alt1,
    /// 启发式 ALT2：在 ALT1 基础上，邻居数 > 5 的内部拥挤节点不移动
    Alt2,
}

/// 单步执行结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepResult {
    /// 某机器人成功执行了一次移动
    Moved {
        agent_idx: usize,
        from: (i32, i32),
        to: (i32, i32),
        dir: Direction,
    },
    /// 当前全场无任何活跃机器人（可能已成功收敛，或陷入静态死锁）
    NoActiveAgents,
}

/// 离散多智能体网格仿真器
#[derive(Debug, Clone)]
pub struct SwarmWorld {
    /// 机器人当前坐标列表
    pub positions: Vec<(i32, i32)>,
    /// 构型策略映射表 $\Pi_f$
    pub policy: Policy,
    /// 执行模式
    pub mode: ExecutionMode,
    /// 上一步刚移动的机器人索引（用于 ALT1）
    pub last_moved_agent: Option<usize>,
    /// 已执行总时间步数
    pub step_count: usize,
}

impl SwarmWorld {
    /// 用给定的初始坐标集合创建仿真世界
    pub fn new(positions: Vec<(i32, i32)>, policy: Policy, mode: ExecutionMode) -> Self {
        assert!(!positions.is_empty(), "机器人数量必须大于 0");
        Self {
            positions,
            policy,
            mode,
            last_moved_agent: None,
            step_count: 0,
        }
    }

    /// 随机生成符合假设 A4 的初始单连通且无碰撞的随机集群 $P_0$
    pub fn random_connected<R: Rng>(n: usize, policy: Policy, mode: ExecutionMode, rng: &mut R) -> Self {
        assert!(n >= 2, "集群规模至少为 2");
        let mut occupied: HashSet<(i32, i32)> = HashSet::with_capacity(n);
        let mut positions: Vec<(i32, i32)> = Vec::with_capacity(n);

        // 第一个机器人置于原点
        occupied.insert((0, 0));
        positions.push((0, 0));

        // 每次从现有机器人的未占据邻居中随机选取一个扩展，天然保证单连通性
        while positions.len() < n {
            let mut candidates: Vec<(i32, i32)> = Vec::new();
            for &(x, y) in &positions {
                for dir in Direction::ALL {
                    let (dx, dy) = dir.offset();
                    let neighbor = (x + dx, y + dy);
                    if !occupied.contains(&neighbor) && !candidates.contains(&neighbor) {
                        candidates.push(neighbor);
                    }
                }
            }

            let chosen = *candidates.choose(rng).expect("必须存在可用邻居格子");
            occupied.insert(chosen);
            positions.push(chosen);
        }

        Self::new(positions, policy, mode)
    }

    /// 机器人总数
    #[inline(always)]
    pub fn size(&self) -> usize {
        self.positions.len()
    }

    /// 获取指定机器人的局部感知状态
    pub fn get_local_state(&self, agent_idx: usize) -> LocalState {
        let (x, y) = self.positions[agent_idx];
        let mut state = LocalState::EMPTY;

        for (other_idx, &(ox, oy)) in self.positions.iter().enumerate() {
            if other_idx == agent_idx {
                continue;
            }
            let dx = ox - x;
            let dy = oy - y;
            if dx.abs() <= 1 && dy.abs() <= 1 {
                if let Some(dir) = Direction::from_offset(dx, dy) {
                    state.set_neighbor(dir, true);
                }
            }
        }
        state
    }

    /// ## 任务 4: 执行一次离散异步时间步
    ///
    /// ## 规则 (Springer 2019):
    /// 1. **候选活跃节点搜集**:
    ///    - 遍历所有机器人 $i \in [0, n)$，获取其当前局部感知状态 `let state = self.get_local_state(i);`；
    ///    - 若处于 `ExecutionMode::Alt2` 且 `state.neighbor_count() > 5`，跳过该内部拥挤节点；
    ///    - 获取可用安全动作 `let acts = self.policy.get_actions(state);`，非空则加入活跃候选池；
    /// 2. **无活跃节点提前返回**: 若活跃池为空，返回 `StepResult::NoActiveAgents`；
    /// 3. **启发式过滤 (ALT1)**:
    ///    - 若开启 ALT1/ALT2 且存在非上一轮刚移动的个体，优先在这些个体中抽取；
    /// 4. **随机执行动作与位移更新**:
    ///    - 随机挑取一个机器人与其动作 `dir`，更新 `self.positions[*agent_idx]` 与 `self.last_moved_agent`；
    ///    - `self.step_count += 1`，返回 `StepResult::Moved`。
    pub fn step<R: Rng>(&mut self, rng: &mut R) -> StepResult {
        // TODO: 请实现群体机器人离散异步单步仿真
        todo!("【关卡 10 - 任务 4】请实现多智能体异步离散仿真单步 step（含启发式过滤）");
    }

    /// 检查集群是否已成功收敛到期望构型：
    /// 全体机器人均处于期望状态 $S_{\text{des}}$（即活跃机器人数量为 0 且无阻塞节点）
    pub fn is_converged(&self) -> bool {
        for i in 0..self.positions.len() {
            let state = self.get_local_state(i);
            if !self.policy.is_desired(state) {
                return false;
            }
        }
        true
    }

    /// 统计处于期望态、活跃态与阻塞态的机器人数量
    pub fn state_breakdown(&self) -> (usize, usize, usize) {
        let mut n_des = 0;
        let mut n_act = 0;
        let mut n_blk = 0;

        for i in 0..self.positions.len() {
            let state = self.get_local_state(i);
            if self.policy.is_desired(state) {
                n_des += 1;
            } else if self.policy.is_active(state) {
                n_act += 1;
            } else {
                n_blk += 1;
            }
        }
        (n_des, n_act, n_blk)
    }

    /// 运行仿真直至收敛或达到最大步数限制
    pub fn run_until_converged<R: Rng>(&mut self, max_steps: usize, rng: &mut R) -> bool {
        for _ in 0..max_steps {
            if self.is_converged() {
                return true;
            }
            if let StepResult::NoActiveAgents = self.step(rng) {
                return self.is_converged();
            }
        }
        self.is_converged()
    }
}
