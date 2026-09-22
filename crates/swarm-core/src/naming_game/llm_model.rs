//! # 大模型命名博弈微观动力学模型 (LLM Naming Game)
//!
//! 论文: "Microscopic dynamics of consensus formation in multi-agent LLM Naming Games" (arXiv:2608.02178, 2026)
//!
//! 本模块实现将经典 Naming Game 的确定性词库检验，替换为温度 $T$ 下基于 LLM 单 Token 判断的
//! 微观双速率随机通道 $(\pi, \phi)$ 模型。

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
    /// 根据论文表 1 及实验测量计算在给定解码温度 T 下的有效速率 (pi, phi)
    pub fn get_rates(&self, temperature: f64) -> (f64, f64) {
        let t = temperature.clamp(0.0, 2.5);
        match self {
            LlmArchitecture::Llama3_1_8B => {
                let pi = (1.0 - 0.225 * t).clamp(0.55, 1.0);
                let phi = (0.05 + 0.225 * t).clamp(0.05, 0.50);
                (pi, phi)
            }
            LlmArchitecture::Mistral7B => {
                // 近确定型：pi 接近 1.0，phi 极低
                let pi = 0.99;
                let phi = (0.02 * (-0.5 * t).exp()).clamp(0.005, 0.05);
                (pi, phi)
            }
            LlmArchitecture::Phi3_14B => {
                // 保守型：phi 几乎为 0，但 pi 随温度严重衰退
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
        let (pi, phi) = architecture.get_rates(temperature);

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

    /// 执行一步 LLM Naming Game 博弈
    ///
    /// ## 论文 Section II & III 规则：
    /// 1. 抽取 Speaker $S$ 及从邻居中抽取 Hearer $H$；
    /// 2. 若 $S$ 为空，发明新词；$S$ 从词库中随机挑词 $w$ 发送；
    /// 3. **微观通道判定与 LLM 判断模拟**:
    ///    - 若 $w \in P_H$ (In-inventory):
    ///      以概率 $\pi(T)$ 判定为 YES（合法坍缩，True Positive）；
    ///      以概率 $1-\pi(T)$ 判定为 NO（漏坍缩，False Negative）；
    ///    - 若 $w \notin P_H$ (Out-inventory):
    ///      以概率 $\phi(T)$ 判定为 YES（非法重涂，False Positive）；
    ///      以概率 $1-\phi(T)$ 判定为 NO（正常未命中，True Negative）；
    /// 4. **状态更新**:
    ///    - 若判定为 YES（包含 TP 与 FP 重涂）：
    ///      双方坍缩为 $\{w\}$：$P_S \leftarrow \{w\}, P_H \leftarrow \{w\}$；
    ///    - 若判定为 NO（包含 FN 与 TN）：
    ///      听者追加词汇：$P_H \leftarrow P_H \cup \{w\}$（若已包含则保持原样）。
    pub fn step<R: Rng>(&mut self, rng: &mut R) -> LlmInteractionResult {
        let n = self.num_agents();

        // 1. 随机对局对
        let speaker = rng.gen_range(0..n);
        let hearer = self.network.random_neighbor(speaker, rng);

        // 2. 说话者传词
        if self.inventories[speaker].is_empty() {
            let new_word = self.next_word_id;
            self.next_word_id += 1;
            self.inventories[speaker].insert(new_word);
        }

        let speaker_inv = &self.inventories[speaker];
        let chosen_idx = rng.gen_range(0..speaker_inv.len());
        let word = *speaker_inv.iter().nth(chosen_idx).unwrap();

        // 3. 微观通道与随机决策
        let is_in_inventory = self.inventories[hearer].contains(&word);

        let (llm_accepted, channel) = if is_in_inventory {
            // 库内通道
            if rng.gen_bool(self.pi) {
                self.tp_count += 1;
                (true, ChannelOutcome::TruePositive)
            } else {
                self.fn_count += 1;
                (false, ChannelOutcome::FalseNegative)
            }
        } else {
            // 库外通道
            if rng.gen_bool(self.phi) {
                self.fp_count += 1;
                (true, ChannelOutcome::FalsePositive)
            } else {
                self.tn_count += 1;
                (false, ChannelOutcome::TrueNegative)
            }
        };

        // 维护滑动窗口
        if self.recent_history.len() >= self.window_size {
            self.recent_history.pop_front();
        }
        self.recent_history.push_back(channel);

        // 4. 状态更新
        let collapse_triggered = llm_accepted;
        if collapse_triggered {
            // 坍缩触发（合法共识 或 噪声重涂）
            self.inventories[speaker].clear();
            self.inventories[speaker].insert(word);

            self.inventories[hearer].clear();
            self.inventories[hearer].insert(word);
        } else {
            // 未触发坍缩，听者收录该词
            self.inventories[hearer].insert(word);
        }

        self.time_step += 1;

        LlmInteractionResult {
            speaker,
            hearer,
            transmitted_word: word,
            is_in_inventory,
            llm_accepted,
            channel,
            collapse_triggered,
        }
    }

    /// 在库交互比例 m(t)（滑动窗口估计）
    pub fn in_inventory_fraction(&self) -> f64 {
        if self.recent_history.is_empty() {
            return 0.0;
        }
        let in_inv = self
            .recent_history
            .iter()
            .filter(|&&c| c == ChannelOutcome::TruePositive || c == ChannelOutcome::FalseNegative)
            .count();
        in_inv as f64 / self.recent_history.len() as f64
    }

    /// 微观净有序漂移算子 Delta(t) = m(t)*pi - (1 - m(t))*phi
    pub fn drift_proxy(&self) -> f64 {
        let m = self.in_inventory_fraction();
        m * self.pi - (1.0 - m) * self.phi
    }

    /// 理论两词平均场有序条件 R = 3*pi - 2*phi - 1
    /// 当 R > 0 时，系统位于有序相（能够自发破缺收敛到单一词汇）
    #[inline]
    pub fn ordering_parameter_r(&self) -> f64 {
        3.0 * self.pi - 2.0 * self.phi - 1.0
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
