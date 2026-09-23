//! # 🎯 [实战关卡 6 - 任务 5] Direct Naming Game 核心博弈动力学模型
//!
//! 论文: "Microscopic activity patterns in the Naming Game" (cond-mat/0606125)
//!
//! 在这里你将学习并练习：
//! 1. 泛型 Trait 约束设计：`struct NamingGame<G: Network>`；
//! 2. 离散多智能体状态管理：`Vec<HashSet<WordId>>`；
//! 3. 随机抽取与离散谈判分支逻辑（说话者发明词汇、均匀抽词、听者匹配检测、成功与失败时的词库更新）；
//! 4. 借用与所有权转移技巧。
//!
//! 遇到卡壳可查阅参考答案: `crates/swarm-core/src/reference/naming_game.rs`

#![allow(unused_variables, dead_code)]

use crate::naming_game::network::Network;
use rand::Rng;
use std::collections::HashSet;

pub type WordId = u32;
pub type AgentId = usize;

/// 单次谈判博弈的微观交互结果快照
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteractionResult {
    /// 说话者（Speaker）智能体编号
    pub speaker: AgentId,
    /// 听者（Hearer）智能体编号
    pub hearer: AgentId,
    /// 说话者本次传达的词汇
    pub transmitted_word: WordId,
    /// 本次谈判是否达成共识（听者词库中是否存在该词汇）
    pub success: bool,
}

/// Naming Game 离散多智能体系统
#[derive(Clone, Debug)]
pub struct NamingGame<G: Network> {
    /// 底层拓扑网络
    pub network: G,
    /// 所有智能体的词汇库列表（长度为 N）
    pub inventories: Vec<HashSet<WordId>>,
    /// 用于自增生成全局唯一新词汇的计数器
    pub next_word_id: WordId,
    /// 当前已推进的博弈总步数 t
    pub time_step: usize,
    /// 历史上累计成功的博弈次数
    pub total_successes: usize,
}

impl<G: Network> NamingGame<G> {
    /// 创建一个新的 Naming Game 系统，初始所有智能体词汇库为空
    pub fn new(network: G) -> Self {
        let n = network.num_nodes();
        Self {
            network,
            inventories: vec![HashSet::new(); n],
            next_word_id: 1,
            time_step: 0,
            total_successes: 0,
        }
    }

    /// 智能体总数 N
    #[inline]
    pub fn num_agents(&self) -> usize {
        self.network.num_nodes()
    }

    /// 执行一步 Direct Naming Game 博弈
    ///
    /// ## 论文控制规则 (Section II):
    /// 1. **抽取对局对 (Speaker & Hearer)**:
    ///    - 均匀随机选择一个节点作为说话者 $S \in [0, N)$；
    ///    - 从 $S$ 的所有网络邻居中均匀随机选择一个节点作为听者 $H$（调用 `self.network.random_neighbor(speaker, rng)`）。
    ///
    /// 2. **说话者传词 (Speaker)**:
    ///    - 若说话者词汇库为空 ($V_S = \emptyset$)，则 $S$ 发明一个新词汇 $w = \text{self.next\_word\_id}$，
    ///      自增 `next_word_id += 1`，并将 $w$ 存入 $V_S$；
    ///    - 说话者从自身词库 $V_S$ 中均匀随机抽取一个词汇 $w$ 传达给听者 $H$。
    ///      *(提示: 可以从 `self.inventories[speaker].iter()` 中随机抽取一个)*
    ///
    /// 3. **听者匹配与状态更新 (Hearer & Success / Failure)**:
    ///    - 若听者词库中包含该词汇 ($w \in V_H$): **谈判成功 (Success)**！
    ///      双方达成共识，消除所有冗余记忆，**双方词库均清空重置为仅包含 $\{w\}$**；
    ///    - 若听者词库中不包含该词汇 ($w \notin V_H$): **谈判失败 (Failure)**！
    ///      听者将该词汇加入自身词库 ($V_H \leftarrow V_H \cup \{w\}$)，说话者词库保持不变。
    ///
    /// 4. **统计更新**:
    ///    - `self.time_step += 1`；
    ///    - 若成功，`self.total_successes += 1`；
    ///    - 返回 `InteractionResult`。
    pub fn step<R: Rng>(&mut self, rng: &mut R) -> InteractionResult {
        // TODO: 请按照上述 4 个步骤实现单步博弈规则
        let n = self.num_agents();
        let speaker = rng.gen_range(0..n);
        let hearer = self.network.random_neighbor(speaker, rng);
        if self.inventories[speaker].is_empty() {
            let new_word = self.next_word_id;
            self.next_word_id += 1;
            self.inventories[speaker].insert(new_word);
        }

        let speaker_inv = &self.inventories[speaker];
        let chosen_idx = rng.gen_range(0..speaker_inv.len());
        let word = *speaker_inv.iter().nth(chosen_idx).unwrap();

        let success = self.inventories[hearer].contains(&word);
        if success {
            self.inventories[speaker] = HashSet::from([word]);
            self.inventories[hearer] = HashSet::from([word]);
            self.total_successes += 1;
        } else {
            self.inventories[hearer].insert(word);
        }
        self.time_step += 1;
        InteractionResult {
            speaker,
            hearer,
            transmitted_word: word,
            success,
        }
    }
}
