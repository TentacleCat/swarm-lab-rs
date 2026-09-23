//! # 少数派博弈核心模型（支持局域从众羊群效应）
//!
//! 论文: "Minority game with local interactions due to the presence of herding behavior" (physics/0512087)

#![allow(unused_variables, dead_code)]

use crate::minority_game::strategy::MgAgent;
use crate::naming_game::network::Network;
use rand::Rng;

/// 少数派博弈仿真系统
#[derive(Clone, Debug)]
pub struct MinorityGame<G: Network> {
    /// 局域拓扑网络（若为 None 则为全知无拓扑的标准 MG）
    pub network: Option<G>,
    /// 参与博弈的所有智能体
    pub agents: Vec<MgAgent>,
    /// 记忆深度 M
    pub memory: usize,
    /// 公开历史状态编码（整数值 in [0, 2^M)）
    pub history: usize,
    /// 当前时间步
    pub time_step: usize,
    /// 是否开启局域从众羊群效应 (Herding Behavior)
    pub herding_enabled: bool,
}

impl<G: Network> MinorityGame<G> {
    /// 初始化少数派博弈
    /// - `num_agents`: 智能体总数 N (通常为奇数，如 101)
    /// - `memory`: 历史记忆步数 M (2..10)
    /// - `num_strategies`: 每位智能体分配的策略数 S (通常为 2)
    /// - `network`: 可选的交互网络
    /// - `herding_enabled`: 是否开启邻居跟风模仿
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

    /// ## 任务 4: 执行一步少数派博弈 (单步状态机)
    ///
    /// ## 规则 (physics/0512087 Section II):
    /// 1. **个体决策**:
    ///    - 若未开启从众 (`!self.herding_enabled || self.network.is_none()`):
    ///      每位个体执行自身当前虚拟得分最高策略所给出的动作：`agent.self_action(self.history)`；
    ///    - 若开启从众 (`self.herding_enabled && self.network.is_some()`):
    ///      个体 $i$ 查看自身所有邻居，找到邻域中最高策略得分最高者 $j^*$；
    ///      - 若 $\text{score}(j^*) > \text{score}(i)$: 个体 $i$ 放弃自我，模仿采纳 $j^*$ 的最佳动作；
    ///      - 若 $\text{score}(j^*) \le \text{score}(i)$ 或邻居为空: 个体 $i$ 坚持自己的策略动作。
    /// 2. **胜负判定与市场出清**:
    ///    - 净出席数 $A(t) = \sum a_i(t)$；
    ///    - 若 $A(t) < 0$，少数派动作为 $+1$（选 $+1$ 者胜）；
    ///    - 若 $A(t) > 0$，少数派动作为 $-1$（选 $-1$ 者胜）；
    ///    - 若 $A(t) = 0$，以 0.5 概率随机抽取 $\pm 1$。
    /// 3. **虚拟策略积分更新与历史推进**:
    ///    - 所有智能体手头的所有策略均调用 `strat.update_score(winning_action, self.history)`；
    ///    - 历史状态左移 1 位并压入获胜结果（1 为 1，-1 为 0）：
    ///      `let bit = if winning_action == 1 { 1 } else { 0 };`
    ///      `self.history = ((self.history << 1) | bit) & ((1 << self.memory) - 1);`
    ///    - `self.time_step += 1`，返回 $A(t)$。
    pub fn step<R: Rng>(&mut self, rng: &mut R) -> i32 {
        // TODO: 请按照上述 3 个步骤实现少数派博弈的单步状态机
        todo!("【关卡 8 - 任务 4】请实现少数派博弈单步状态机 step（含邻域从众判定、胜负判定与历史位运算更新）");
    }

    /// 连续推进多个时间步，并返回各步的净动作序列 A(t)
    pub fn run_simulation<R: Rng>(&mut self, steps: usize, rng: &mut R) -> Vec<i32> {
        let mut attendances = Vec::with_capacity(steps);
        for _ in 0..steps {
            attendances.push(self.step(rng));
        }
        attendances
    }
}
