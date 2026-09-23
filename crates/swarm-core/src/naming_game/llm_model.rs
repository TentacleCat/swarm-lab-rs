//! # 🎯 [实战关卡 7] 大模型多智能体命名博弈微观动力学模型 (LLM Naming Game)
//!
//! 论文: "Microscopic dynamics of consensus formation in multi-agent LLM Naming Games" (arXiv:2608.02178, 2026)
//! 本地文档: `papers/naming-game/02-llm-naming-game-arxiv2026/README.md`
//!
//! 在这里你将学习并亲手实现：
//! 1. 浮点数区段截断与模式匹配：在 `LlmArchitecture::get_rates` 中计算经验微观参数 $(\pi(T), \phi(T))$；
//! 2. 随机伯努利试验与四通道微观分解：
//!    - 库内有序通道 $\pi$: True Positive (合法坍缩) 与 False Negative (漏坍缩)
//!    - 库外无序通道 $\phi$: False Positive (噪声重涂) 与 True Negative (正常扩库)
//! 3. 滑动窗口 `VecDeque` 状态追踪与微观净漂移算子 $\Delta(t) = m(t)\pi - (1-m(t))\phi$；
//! 4. 论文两词平均场临界有序相判定条件 $R = 3\pi - 2\phi - 1 > 0$。
//!
//! 遇到卡壳可查阅参考答案: `crates/swarm-core/src/reference/llm_naming_game.rs`

#![allow(unused_variables, dead_code)]

use crate::naming_game::network::Network;
use rand::Rng;
use std::collections::{HashSet, VecDeque};

pub type WordId = u32;
pub type AgentId = usize;

/// 微观交互事件的四通道分解 (Microscopic Channel Outcomes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelOutcome {
    /// 库内且回答 YES (True Positive): 合法坍缩，有序通道
    TruePositive,
    /// 库内且回答 NO (False Negative): 漏坍缩，保守误差
    FalseNegative,
    /// 库外且回答 YES (False Positive): 非法重涂 (Repaint)，无序通道
    FalsePositive,
    /// 库外且回答 NO (True Negative): 正常收纳新词
    TrueNegative,
}

/// LLM 监听者架构类型及其在解码温度 $T$ 下的经验微观响应
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LlmArchitecture {
    /// LLaMA-3.1:8B (宽容型 / 噪声重涂主导型)
    /// - 固化率随着 T 下降: pi ≈ 1.0 (T=0) -> 0.55 (T=2.0)
    /// - 重涂率随着 T 上升: phi ≈ 0.05 (T=0) -> 0.50 (T=2.0)
    Llama3_1_8B,

    /// Mistral:7B (近确定型 / 温度盲性)
    /// - pi ≈ 1.0, phi ≈ 0 在所有测试温度下保持稳定
    Mistral7B,

    /// Phi-3:14B (保守型 / 漏坍缩主导型)
    /// - phi ≈ 0.01~0.03
    /// - 但 pi 从 0.75 (T=0.05) 骤降至 0.30 (T=2.0)
    Phi3_14B,

    /// 自定义或给定的两速率常数通道 (pi, phi)
    CustomTwoRate { pi: f64, phi: f64 },
}

impl LlmArchitecture {
    /// ## 任务 1: 计算大模型在给定解码温度 T 下的有效通道速率 (pi, phi)
    ///
    /// 根据论文表 1 及实测经验拟合公式：
    /// 1. 温度范围裁剪：$t = \text{temperature.clamp}(0.0, 2.5)$；
    /// 2. 分架构计算：
    ///    - `Llama3_1_8B`:
    ///      $\pi = (1.0 - 0.225 \cdot t).\text{clamp}(0.55, 1.0)$
    ///      $\phi = (0.05 + 0.225 \cdot t).\text{clamp}(0.05, 0.50)$
    ///    - `Mistral7B`:
    ///      $\pi = 0.99$
    ///      $\phi = (0.02 \cdot \exp(-0.5 \cdot t)).\text{clamp}(0.005, 0.05)$
    ///    - `Phi3_14B`:
    ///      $\pi = (0.76 - 0.23 \cdot t).\text{clamp}(0.28, 0.76)$
    ///      $\phi = (0.01 + 0.015 \cdot t).\text{clamp}(0.01, 0.04)$
    ///    - `CustomTwoRate { pi, phi }`: 直接解构返回 `(*pi, *phi)`
    pub fn get_rates(&self, temperature: f64) -> (f64, f64) {
        // TODO: 请按照上述经验公式实现各架构速率计算

        let t = temperature.clamp(0.0, 2.5);
        match self {
            LlmArchitecture::Llama3_1_8B => {
                let pi = (1.0 - 0.225 * t).clamp(0.55, 1.0);
                let phi = (0.05 + 0.225 * t).clamp(0.05, 0.50);
                (pi, phi)
            }
            LlmArchitecture::Mistral7B => {
                let pi = 0.99;
                let phi = (0.02 * (-0.5 * t).exp()).clamp(0.005, 0.05);
                (pi, phi)
            }
            LlmArchitecture::Phi3_14B => {
                let pi = (0.76 - 0.23 * t).clamp(0.28, 0.76);
                let phi = (0.01 + 0.015 * t).clamp(0.01, 0.04);
                (pi, phi)
            }
            LlmArchitecture::CustomTwoRate { pi, phi } => (*pi, *phi),
        }
    }
}

/// LLM 命名博弈单步微观交互结果
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LlmInteractionResult {
    pub speaker: AgentId,
    pub hearer: AgentId,
    pub transmitted_word: WordId,
    pub is_in_inventory: bool,
    pub llm_accepted: bool,
    pub channel: ChannelOutcome,
    pub collapse_triggered: bool,
}

/// LLM Naming Game 仿真系统
#[derive(Clone, Debug)]
pub struct LlmNamingGame<G: Network> {
    pub network: G,
    pub inventories: Vec<HashSet<WordId>>,
    pub architecture: LlmArchitecture,
    pub temperature: f64,
    pub pi: f64,
    pub phi: f64,
    pub next_word_id: WordId,
    pub time_step: usize,

    // 微观通道统计
    pub tp_count: usize,
    pub fn_count: usize,
    pub fp_count: usize,
    pub tn_count: usize,

    // 最近窗口通道历史，用于平滑估计在库比例 m(t) 与漂移 Delta(t)
    recent_history: VecDeque<ChannelOutcome>,
    window_size: usize,
}

impl<G: Network> LlmNamingGame<G> {
    pub fn new(network: G, architecture: LlmArchitecture, temperature: f64) -> Self {
        let n = network.num_nodes();
        // 允许练习区在 get_rates 尚未实现时优雅处理，若已实现则取对应值
        let (pi, phi) =
            std::panic::catch_unwind(|| architecture.get_rates(temperature)).unwrap_or((1.0, 0.0));

        Self {
            network,
            inventories: vec![HashSet::new(); n],
            architecture,
            temperature,
            pi,
            phi,
            next_word_id: 1,
            time_step: 0,
            tp_count: 0,
            fn_count: 0,
            fp_count: 0,
            tn_count: 0,
            recent_history: VecDeque::with_capacity(1000),
            window_size: 500,
        }
    }

    #[inline]
    pub fn num_agents(&self) -> usize {
        self.network.num_nodes()
    }

    /// ## 任务 2: 执行一步 LLM Naming Game 博弈
    ///
    /// ### 论文 Section II & III 规则：
    /// 1. **对局抽取**:
    ///    - 均匀随机选择一个说话者 $S \in [0, N)$；
    ///    - 从 $S$ 的网络邻居中均匀随机选择听者 $H$（调用 `self.network.random_neighbor(speaker, rng)`）。
    ///
    /// 2. **说话者传词**:
    ///    - 若 $S$ 词库为空，发明新词 `self.next_word_id`，存入 $V_S$，并使 `next_word_id += 1`；
    ///    - 从 $V_S$ 中均匀随机抽取一个词汇 $w$ 传达给听者 $H$。
    ///
    /// 3. **微观通道判定与 LLM 随机回答**:
    ///    - 检查词汇是否在听者库内：`let is_in_inventory = self.inventories[hearer].contains(&word);`
    ///    - 若在库内 (`is_in_inventory == true`):
    ///      - 以概率 $\pi$ 回答 YES（合法坍缩，`ChannelOutcome::TruePositive`，`self.tp_count += 1`）；
    ///      - 以概率 $1-\pi$ 回答 NO（漏坍缩，`ChannelOutcome::FalseNegative`，`self.fn_count += 1`）；
    ///    - 若在库外 (`is_in_inventory == false`):
    ///      - 以概率 $\phi$ 回答 YES（非法重涂，`ChannelOutcome::FalsePositive`，`self.fp_count += 1`）；
    ///      - 以概率 $1-\phi$ 回答 NO（正常收纳，`ChannelOutcome::TrueNegative`，`self.tn_count += 1`）；
    ///    - 维护滑动窗口：若 `recent_history.len() >= self.window_size` 则 `pop_front()`，并将本次 `channel` `push_back()`。
    ///
    /// 4. **状态更新**:
    ///    - 若判定为 YES（包含 TP 与 FP 噪声重涂）：
    ///      双方词库均清空并坍缩为仅包含 $\{w\}$：$V_S \leftarrow \{w\}, V_H \leftarrow \{w\}$；
    ///    - 若判定为 NO（包含 FN 与 TN）：
    ///      听者将该词加入自身词库：$V_H \leftarrow V_H \cup \{w\}$；
    ///
    /// 5. **自增步数并返回结果快照**:
    ///    - `self.time_step += 1`；
    ///    - 返回 `LlmInteractionResult`。
    pub fn step<R: Rng>(&mut self, rng: &mut R) -> LlmInteractionResult {
        // TODO: 请按照上述规则实现四通道博弈单步 step
        let n = self.num_agents();
        let speaker = rng.gen_range(0..n);
        let hearer = self.network.random_neighbor(speaker, rng);
        if self.inventories[speaker].is_empty() {
            let new_word = self.next_word_id;
            self.next_word_id += 1;
            self.inventories[speaker].insert(new_word);
        }

        let inv = &self.inventories[speaker];
        let idx = rng.gen_range(0..inv.len());
        let word = *inv.iter().nth(idx).unwrap();

        let is_in_inventory = self.inventories[hearer].contains(&word);
        let (llm_accepted, channel) = if is_in_inventory {
            if rng.gen_bool(self.pi) {
                self.tp_count += 1;
                (true, ChannelOutcome::TruePositive)
            } else {
                self.fn_count += 1;
                (false, ChannelOutcome::FalseNegative)
            }
        } else {
            if rng.gen_bool(self.phi) {
                self.fp_count += 1;
                (true, ChannelOutcome::FalsePositive)
            } else {
                self.tn_count += 1;
                (false, ChannelOutcome::TrueNegative)
            }
        };
        if self.recent_history.len() >= self.window_size {
            self.recent_history.pop_front();
        }
        self.recent_history.push_back(channel);
    }

    /// ## 任务 3: 滑动窗口在库比例 $m(t)$ 与微观净漂移算子 $\Delta(t)$
    ///
    /// ### 1. 在库交互比例 $m(t)$ (`in_inventory_fraction`)
    /// - 统计 `self.recent_history` 中处于库内通道（即 `TruePositive` 或 `FalseNegative`）的事件占比；
    /// - 若历史为空则返回 `0.0`。
    pub fn in_inventory_fraction(&self) -> f64 {
        // TODO: 计算在库事件的滑动窗口经验比例 m(t)
        todo!("【关卡 7 - 任务 3】请实现滑动窗口在库比例 in_inventory_fraction");
    }

    /// ### 2. 微观净有序漂移算子 $\Delta(t)$ (`drift_proxy`)
    ///
    /// 公式: $\Delta(t) = m(t) \cdot \pi - (1 - m(t)) \cdot \phi$
    /// - 当 $\Delta(t) > 0$ 时，有序坍缩速率超越无序重涂速率，系统具有向单一共识收敛的微观驱动力。
    pub fn drift_proxy(&self) -> f64 {
        // TODO: 结合 in_inventory_fraction() 计算漂移算子
        todo!("【关卡 7 - 任务 3】请实现微观净漂移算子 drift_proxy");
    }

    /// ## 任务 4: 理论两词平均场临界有序相判定指标 $R$
    ///
    /// 论文核心相变公式 (Section III):
    /// $$R \equiv 3\pi - 2\phi - 1$$
    /// - 当 $R > 0$ 时，系统位于有序相（能够自发破缺收敛到单一共识）；
    /// - 当 $R < 0$ 时，系统位于无序相（被漏坍缩与噪声重涂主导，无法达成共识）。
    #[inline]
    pub fn ordering_parameter_r(&self) -> f64 {
        // TODO: 实现两词平均场有序度指标 R
        todo!("【关卡 7 - 任务 4】请实现两词平均场临界有序相判定指标 ordering_parameter_r");
    }

    /// 智能体平均词汇库大小 bar{k}(t) = 1/N sum |P_i|
    pub fn average_inventory_size(&self) -> f64 {
        let total: usize = self.inventories.iter().map(|inv| inv.len()).sum();
        total as f64 / self.num_agents() as f64
    }

    /// 全网不同词汇种类数 N_d(t)
    pub fn distinct_words(&self) -> usize {
        let all: HashSet<WordId> = self.inventories.iter().flatten().copied().collect();
        all.len()
    }

    /// 全网词汇总量 N_w(t)
    pub fn total_words(&self) -> usize {
        self.inventories.iter().map(|inv| inv.len()).sum()
    }

    /// 是否达成全局严格单一共识
    pub fn is_consensus(&self) -> bool {
        if self.inventories.is_empty() {
            return false;
        }
        let first = &self.inventories[0];
        if first.len() != 1 {
            return false;
        }
        let word = *first.iter().next().unwrap();
        self.inventories
            .iter()
            .all(|inv| inv.len() == 1 && inv.contains(&word))
    }
}
