//! # 🎯 [实战关卡 6 - 任务 1] 命名博弈宏观与微观序参量计算 (Metrics)
//!
//! 论文: "Microscopic activity patterns in the Naming Game" (cond-mat/0606125)
//!
//! 在这里你将学习并练习：
//! 1. `std::collections::HashSet` 的聚合与并集运算 (`union`)
//! 2. 函数式迭代器统计：`iter().map(...).sum()`
//! 3. 概率分布统计直方图与归一化 (`BTreeMap`)
//! 4. 节点度过滤切片借用

#![allow(unused_variables, dead_code)]

use std::collections::{BTreeMap, HashSet};

/// 词汇编号标识符
pub type WordId = u32;

/// ## 任务 1: 全网词汇总量 $N_w(t)$
///
/// 公式: $N_w(t) = \sum_{i=1}^N |V_i(t)|$
///
/// 计算系统中所有智能体当前拥有的词汇条目总数。
pub fn total_words(inventories: &[HashSet<WordId>]) -> usize {
    // TODO: 请使用迭代器对每个智能体的 inventory.len() 进行求和
    //
    // 提示:
    // inventories.iter().map(|inv| inv.len()).sum()
    todo!("【关卡 6 - 任务 1】请在此处实现 total_words 函数");
}

/// ## 任务 2: 全网不同词汇种类数 $N_d(t)$
///
/// 公式: $N_d(t) = |\bigcup_{i=1}^N V_i(t)|$
///
/// 统计当前整个群体中存在多少种不同的词汇（去重后的总数）。
pub fn distinct_words(inventories: &[HashSet<WordId>]) -> usize {
    // TODO: 统计所有智能体词汇库的并集大小
    //
    // 提示:
    // 可以创建一个 HashSet<WordId>，然后遍历每一个智能体的词汇并 insert，最后返回 len()。
    // 或者使用迭代器：
    // let all_words: HashSet<WordId> = inventories.iter().flatten().copied().collect();
    // all_words.len()
    todo!("【关卡 6 - 任务 2】请在此处实现 distinct_words 函数");
}

/// ## 任务 3: 微观词汇量分布 $\mathcal{P}_n(k | t)$
///
/// 论文核心统计量：统计给定节点集合（例如某一度数 $k$ 的节点，或全部节点）中，
/// 词汇库大小为 $n$ 的智能体占比。
///
/// - `inventories`: 全部智能体的词汇库切片
/// - `filter_nodes`: 可选的节点下标切片（例如仅传入度数为 $k$ 的节点列表；若为 None 则统计全网节点）
///
/// 返回: 有序字典 `BTreeMap<usize, f64>`，键为词汇量 $n$（如 1, 2, 3...），值为概率 $\mathcal{P}_n \in [0, 1]$。
pub fn inventory_size_distribution(
    inventories: &[HashSet<WordId>],
    filter_nodes: Option<&[usize]>,
) -> BTreeMap<usize, f64> {
    // TODO:
    // 1. 确定要统计的节点索引列表（filter_nodes 传入的切片，或 0..inventories.len() 全体）；
    // 2. 统计每个词汇量 n (即 inv.len()) 出现的频数 count；
    // 3. 将频数除以目标节点总数进行归一化，使得 sum(P_n) = 1.0；
    // 4. 返回 BTreeMap<usize, f64>。
    todo!("【关卡 6 - 任务 3】请在此处实现 inventory_size_distribution 函数");
}

/// ## 任务 4: 检查是否达成全网单一共识 (Consensus)
///
/// 条件: 每个智能体词汇表均只有 1 个词汇，且全网不同词汇总数 $N_d = 1$。
pub fn is_consensus(inventories: &[HashSet<WordId>]) -> bool {
    // TODO:
    // 检查是否非空，且第一个智能体的词汇非空，且所有智能体的词汇都与其完全相同且长度为 1
    todo!("【关卡 6 - 任务 4】请在此处实现 is_consensus 函数");
}
