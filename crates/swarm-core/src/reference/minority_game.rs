//! # 💡 [参考答案区] 少数派博弈与从众效应标准参考实现 (Reference Implementation)
//!
//! 论文: "Minority game with local interactions due to the presence of herding behavior" (physics/0512087)
//!
//! 本文件包含经过完备单测验证的标准理论计算与基准实现，供闯关学习者对比查阅。

use crate::naming_game::network::Network;
use rand::Rng;

/// 智能体策略表：将 2^M 个历史状态映射为离散动作 a in {-1, +1}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Strategy {
    pub actions: Vec<i8>,
    pub virtual_score: i32,
}

impl Strategy {
    pub fn random<R: Rng>(memory: usize, rng: &mut R) -> Self {
        let num_states = 1 << memory;
        let mut actions = Vec::with_capacity(num_states);
        for _ in 0..num_states {
            let a = if rng.gen_bool(0.5) { 1 } else { -1 };
            actions.push(a);
        }
        Self {
            actions,
            virtual_score: 0,
        }
    }

    #[inline]
    pub fn predict(&self, history: usize) -> i8 {
        self.actions[history]
    }

    #[inline]
    pub fn update_score(&mut self, winning_action: i8, history: usize) {
        if self.actions[history] == winning_action {
            self.virtual_score += 1;
        } else {
            self.virtual_score -= 1;
        }
    }
}

/// 少数派博弈智能体
#[derive(Clone, Debug)]
pub struct MgAgent {
    pub strategies: Vec<Strategy>,
}

impl MgAgent {
    pub fn new<R: Rng>(memory: usize, num_strategies: usize, rng: &mut R) -> Self {
        let mut strategies = Vec::with_capacity(num_strategies);
        for _ in 0..num_strategies {
            strategies.push(Strategy::random(memory, rng));
        }
        Self { strategies }
    }

    pub fn best_strategy_index(&self) -> usize {
        let mut best_idx = 0;
        let mut max_score = self.strategies[0].virtual_score;
        for (i, strat) in self.strategies.iter().enumerate().skip(1) {
            if strat.virtual_score > max_score {
                max_score = strat.virtual_score;
                best_idx = i;
            }
        }
        best_idx
    }

    #[inline]
    pub fn highest_score(&self) -> i32 {
        self.strategies[self.best_strategy_index()].virtual_score
    }

    #[inline]
    pub fn self_action(&self, history: usize) -> i8 {
        self.strategies[self.best_strategy_index()].predict(history)
    }
}

/// 归一化市场波动率计算
pub fn normalized_volatility(attendances: &[i32], num_agents: usize) -> f64 {
    if attendances.is_empty() || num_agents == 0 {
        return 0.0;
    }
    let t = attendances.len() as f64;
    let sum: f64 = attendances.iter().map(|&a| a as f64).sum();
    let mean = sum / t;

    let var: f64 = attendances
        .iter()
        .map(|&a| {
            let diff = a as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / t;

    var / num_agents as f64
}

/// 信息比例参数 alpha = 2^M / N
#[inline]
pub fn alpha_parameter(memory: usize, num_agents: usize) -> f64 {
    (1 << memory) as f64 / num_agents as f64
}

/// 少数派博弈仿真系统标准参考实现
#[derive(Clone, Debug)]
pub struct MinorityGameReference<G: Network> {
    pub network: Option<G>,
    pub agents: Vec<MgAgent>,
    pub memory: usize,
    pub history: usize,
    pub time_step: usize,
    pub herding_enabled: bool,
}

impl<G: Network> MinorityGameReference<G> {
    pub fn new<R: Rng>(
        num_agents: usize,
        memory: usize,
        num_strategies: usize,
        network: Option<G>,
        herding_enabled: bool,
        rng: &mut R,
    ) -> Self {
        let mut agents = Vec::with_capacity(num_agents);
        for _ in 0..num_agents {
            agents.push(MgAgent::new(memory, num_strategies, rng));
        }
        let max_history = 1 << memory;
        let initial_history = rng.gen_range(0..max_history);

        Self {
            network,
            agents,
            memory,
            history: initial_history,
            time_step: 0,
            herding_enabled,
        }
    }

    #[inline]
    pub fn num_agents(&self) -> usize {
        self.agents.len()
    }

    pub fn step<R: Rng>(&mut self, rng: &mut R) -> i32 {
        let n = self.num_agents();
        let mut actions = Vec::with_capacity(n);

        // 1. 个体决策（含从众判定）
        if self.herding_enabled && self.network.is_some() {
            let net = self.network.as_ref().unwrap();

            for i in 0..n {
                let my_score = self.agents[i].highest_score();
                let neighbors = net.neighbors(i);

                if neighbors.is_empty() {
                    actions.push(self.agents[i].self_action(self.history));
                    continue;
                }

                let mut best_neighbor = neighbors[0];
                let mut max_neighbor_score = self.agents[best_neighbor].highest_score();

                for &nb in neighbors.iter().skip(1) {
                    let s = self.agents[nb].highest_score();
                    if s > max_neighbor_score {
                        max_neighbor_score = s;
                        best_neighbor = nb;
                    }
                }

                if max_neighbor_score > my_score {
                    actions.push(self.agents[best_neighbor].self_action(self.history));
                } else {
                    actions.push(self.agents[i].self_action(self.history));
                }
            }
        } else {
            for agent in &self.agents {
                actions.push(agent.self_action(self.history));
            }
        }

        // 2. 计算净动作 A(t) 与获胜动作 W(t)
        let attendance: i32 = actions.iter().map(|&a| a as i32).sum();

        let winning_action = if attendance < 0 {
            1
        } else if attendance > 0 {
            -1
        } else {
            if rng.gen_bool(0.5) { 1 } else { -1 }
        };

        // 3. 更新所有智能体的全部策略积分
        for agent in &mut self.agents {
            for strat in &mut agent.strategies {
                strat.update_score(winning_action, self.history);
            }
        }

        // 4. 更新历史位串
        let bit = if winning_action == 1 { 1 } else { 0 };
        let mask = (1 << self.memory) - 1;
        self.history = ((self.history << 1) | bit) & mask;
        self.time_step += 1;

        attendance
    }

    pub fn run_simulation<R: Rng>(&mut self, steps: usize, rng: &mut R) -> Vec<i32> {
        let mut attendances = Vec::with_capacity(steps);
        for _ in 0..steps {
            attendances.push(self.step(rng));
        }
        attendances
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::naming_game::network::AdjacencyGraph;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_ref_strategy_update() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut strat = Strategy::random(3, &mut rng);
        let pred = strat.predict(2);
        strat.update_score(pred, 2);
        assert_eq!(strat.virtual_score, 1);
    }

    #[test]
    fn test_ref_minority_game_step() {
        let mut rng = StdRng::seed_from_u64(123);
        let net = AdjacencyGraph::regular_ring(5, 2);
        let mut mg = MinorityGameReference::new(5, 3, 2, Some(net), true, &mut rng);
        let att = mg.step(&mut rng);
        assert!(att.abs() <= 5);
    }
}
