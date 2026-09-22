//! # 少数派博弈策略与智能体定义 (Strategy & Agent for Minority Game)

use rand::Rng;

/// 智能体策略表：将 2^M 个历史状态映射为离散动作 a in {-1, +1}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Strategy {
    /// 历史状态查找表，长度为 2^M
    pub actions: Vec<i8>,
    /// 策略累计的虚拟积分 (Virtual Score)
    pub virtual_score: i32,
}

impl Strategy {
    /// 随机初始化一条策略
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

    /// 根据当前历史状态预测动作
    #[inline]
    pub fn predict(&self, history: usize) -> i8 {
        self.actions[history]
    }

    /// 根据实际少数派获胜动作更新虚拟得分
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
    /// 智能体拥有的 S 条候选策略表 (通常 S = 2)
    pub strategies: Vec<Strategy>,
}

impl MgAgent {
    /// 为智能体随机分配 S 条策略
    pub fn new<R: Rng>(memory: usize, num_strategies: usize, rng: &mut R) -> Self {
        let mut strategies = Vec::with_capacity(num_strategies);
        for _ in 0..num_strategies {
            strategies.push(Strategy::random(memory, rng));
        }
        Self { strategies }
    }

    /// 获取当前得分最高的策略下标
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

    /// 获取当前手头策略的最高得分
    #[inline]
    pub fn highest_score(&self) -> i32 {
        self.strategies[self.best_strategy_index()].virtual_score
    }

    /// 由自身最高得分策略给出的动作建议
    #[inline]
    pub fn self_action(&self, history: usize) -> i8 {
        self.strategies[self.best_strategy_index()].predict(history)
    }
}
