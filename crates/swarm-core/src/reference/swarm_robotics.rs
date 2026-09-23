//! # 💡 [参考答案区] 极简群体机器人自组织构型标准参考实现 (Reference Implementation)
//!
//! 论文: "Provable self-organizing pattern formation by a swarm of robots with limited knowledge" (Springer 2019)
//!
//! 本文件包含经过完备单测验证的标准理论计算与基准实现，供闯关学习者对比查阅。

use crate::swarm_robotics::policy::{Pattern, Policy};
use crate::swarm_robotics::simulator::{ExecutionMode, StepResult};
use crate::swarm_robotics::state::{Direction, LocalState};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::{BTreeSet, HashSet};

/// 单纯形状态判定标准参考实现
#[inline]
pub fn ref_is_simplicial(state: LocalState) -> bool {
    let c = state.neighbor_count();
    c > 0 && state.count_cliques() == 1
}

/// 判定动作是否会导致局部断连标准参考实现
pub fn ref_is_separation_action(state: LocalState, dir: Direction) -> bool {
    let neighbors = state.neighbor_coords();
    if neighbors.is_empty() {
        return true;
    }

    let p_new = dir.offset();
    let mut nodes = neighbors;
    nodes.push(p_new);
    let n = nodes.len();

    let mut visited = vec![false; n];
    let mut queue = vec![0];
    visited[0] = true;
    let mut count = 0;

    while let Some(curr) = queue.pop() {
        count += 1;
        let (cx, cy) = nodes[curr];
        for j in 0..n {
            if !visited[j] {
                let (nx, ny) = nodes[j];
                if (cx - nx).abs() <= 1 && (cy - ny).abs() <= 1 {
                    visited[j] = true;
                    queue.push(j);
                }
            }
        }
    }

    count != n
}

/// 从目标构型提取期望局部状态集合标准参考实现
pub fn ref_extract_desired_states(pattern: &Pattern) -> Vec<LocalState> {
    let coord_set: HashSet<(i32, i32)> = pattern.coords.iter().cloned().collect();
    let mut state_set = BTreeSet::new();

    for &(x, y) in &pattern.coords {
        let mut state = LocalState::EMPTY;
        for dir in Direction::ALL {
            let (dx, dy) = dir.offset();
            if coord_set.contains(&(x + dx, y + dy)) {
                state.set_neighbor(dir, true);
            }
        }
        state_set.insert(state);
    }

    state_set.into_iter().collect()
}

/// 离散多智能体网格仿真器标准参考实现
#[derive(Debug, Clone)]
pub struct SwarmWorldReference {
    pub positions: Vec<(i32, i32)>,
    pub policy: Policy,
    pub mode: ExecutionMode,
    pub last_moved_agent: Option<usize>,
    pub step_count: usize,
}

impl SwarmWorldReference {
    pub fn new(positions: Vec<(i32, i32)>, policy: Policy, mode: ExecutionMode) -> Self {
        assert!(!positions.is_empty());
        Self {
            positions,
            policy,
            mode,
            last_moved_agent: None,
            step_count: 0,
        }
    }

    pub fn random_connected<R: Rng>(n: usize, policy: Policy, mode: ExecutionMode, rng: &mut R) -> Self {
        assert!(n >= 2);
        let mut occupied: HashSet<(i32, i32)> = HashSet::with_capacity(n);
        let mut positions: Vec<(i32, i32)> = Vec::with_capacity(n);

        occupied.insert((0, 0));
        positions.push((0, 0));

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

    #[inline(always)]
    pub fn num_agents(&self) -> usize {
        self.positions.len()
    }

    #[inline(always)]
    pub fn size(&self) -> usize {
        self.positions.len()
    }

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

    pub fn is_converged(&self) -> bool {
        for i in 0..self.positions.len() {
            let s = self.get_local_state(i);
            if !self.policy.is_desired(s) {
                return false;
            }
        }
        true
    }

    pub fn step<R: Rng>(&mut self, rng: &mut R) -> StepResult {
        let n = self.positions.len();
        let mut active_candidates: Vec<(usize, Vec<Direction>)> = Vec::new();

        for i in 0..n {
            let state = self.get_local_state(i);
            if self.mode == ExecutionMode::Alt2 && state.neighbor_count() > 5 {
                continue;
            }
            let acts = self.policy.get_actions(state);
            if !acts.is_empty() {
                active_candidates.push((i, acts.to_vec()));
            }
        }

        if active_candidates.is_empty() {
            return StepResult::NoActiveAgents;
        }

        let candidates_to_pick = if (self.mode == ExecutionMode::Alt1 || self.mode == ExecutionMode::Alt2)
            && active_candidates.len() > 1
        {
            if let Some(last) = self.last_moved_agent {
                let filtered: Vec<_> = active_candidates
                    .iter()
                    .filter(|(idx, _)| *idx != last)
                    .cloned()
                    .collect();
                if !filtered.is_empty() {
                    filtered
                } else {
                    active_candidates
                }
            } else {
                active_candidates
            }
        } else {
            active_candidates
        };

        let (agent_idx, actions) = candidates_to_pick.choose(rng).unwrap();
        let dir = *actions.choose(rng).unwrap();

        let from = self.positions[*agent_idx];
        let (dx, dy) = dir.offset();
        let to = (from.0 + dx, from.1 + dy);

        self.positions[*agent_idx] = to;
        self.last_moved_agent = Some(*agent_idx);
        self.step_count += 1;

        StepResult::Moved {
            agent_idx: *agent_idx,
            from,
            to,
            dir,
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ref_simplicial() {
        let mut s = LocalState::EMPTY;
        s.set_neighbor(Direction::North, true);
        assert!(ref_is_simplicial(s));
    }
}
