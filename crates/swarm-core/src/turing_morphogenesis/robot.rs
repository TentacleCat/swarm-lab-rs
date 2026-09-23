#![allow(unused_variables, dead_code, unused_imports)]

//! 实体 Kilobot 机器人个体与形态发生行为状态机 (Kilobot Agent & State Machine)
//!
//! 实现 Science Robotics 2018 原文中的三种行为状态：
//! - WAIT: 组织静止态，参与图灵化学扩散，显示对应形态素 LED 颜色；
//! - ORBIT: 边缘巡航环绕态，沿未极化边界迁移；
//! - FOLLOW: 靠拢重连态，防止在复杂边界掉队。

use super::edge_detector::EdgeDetector;
use super::morphogen::{LedColor, MorphogenConcentration};
use serde::{Deserialize, Serialize};

/// 机器人行为状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BotState {
    /// 静止待命态 (组织基质/斑点核心)
    Wait,
    /// 边缘沿外轮廓环绕态 (轮廓生长迁移)
    Orbit,
    /// 向最近邻靠拢重连态 (掉队规避)
    Follow,
}

/// 局域邻居观测数据结构
#[derive(Debug, Clone, Copy)]
pub struct NeighborObservation {
    pub id: usize,
    pub dist: f64,
    pub state: BotState,
    pub morphogen: MorphogenConcentration,
    pub n_neighbors: usize,
}

/// Kilobot 机器人个体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kilobot {
    /// 机器人唯一编号
    pub id: usize,
    /// 二维物理空间坐标 (单位: mm)
    pub pos: [f64; 2],
    /// 机器人朝向角 (单位: 弧度 [-pi, pi])
    pub heading: f64,
    /// 当前行为状态
    pub state: BotState,
    /// 体内虚拟化学形态素浓度 (u, v)
    pub morphogen: MorphogenConcentration,
    /// 局域边缘检测器
    pub edge_detector: EdgeDetector,
    /// 状态转移等待计时器 (原代码 COUNTER_WAIT)
    pub wait_counter: usize,
    /// 环绕运动转向倾向 (+1.0 顺时针, -1.0 逆时针)
    pub orbit_dir: f64,
    /// 是否处于软碰撞/受困状态
    pub is_stuck: bool,
}

impl Kilobot {
    /// 创建新机器人实例
    pub fn new(id: usize, pos: [f64; 2], u: f64, v: f64) -> Self {
        Self {
            id,
            pos,
            heading: 0.0,
            state: BotState::Wait,
            morphogen: MorphogenConcentration::new(u, v),
            edge_detector: EdgeDetector::default(),
            wait_counter: 0,
            orbit_dir: 1.0,
            is_stuck: false,
        }
    }

    /// 获取机器人当前显示的 LED 颜色
    pub fn led_color(&self, polar_th: f64) -> LedColor {
        match self.state {
            BotState::Orbit => LedColor::White,
            BotState::Follow => LedColor::Red,
            BotState::Wait => self.morphogen.gradient_color(polar_th),
        }
    }

    /// 是否处于极化状态 (即图灵斑点核心)
    pub fn is_polarized(&self, polar_th: f64) -> bool {
        self.morphogen.is_polarized(polar_th)
    }

    /// 【关卡 11 - 任务 3】评估并执行状态机转移 (完全依据原版 C 代码 edge_flow 逻辑)
    ///
    /// 依据 Science Robotics 2018 原文的三态转移图：
    /// 1. 状态 `Wait`:
    ///    - 若处于边缘 (`is_edge`)，所有邻居均处于 `Wait` 态，非极化（或极化但远离其他极化中心），
    ///      且等待计数为 0、邻居不为空：转移至 `BotState::Orbit`，设置 `self.orbit_dir = 1.0`；
    ///    - 否则若处于边缘，最近邻居为 `Wait` 态且距离过大（`min_all_dist > dist_crit + 15.0`）：
    ///      转移至 `BotState::Follow` 避免掉队；
    ///    - 维护 `wait_counter`：若有邻居正在移动则重置为 10，否则逐步递减至 0。
    /// 2. 状态 `Orbit`:
    ///    - 当脱离边缘、到达极化斑点（`min_polar_dist <= dist_crit && count_polarized >= 2`）、
    ///      最近邻居正在移动、或间距过大脱团（`min_all_dist > dist_crit + 25.0`）：转移回 `BotState::Wait`。
    /// 3. 状态 `Follow`:
    ///    - 当已靠近邻居（`min_all_dist <= dist_crit`）或最近邻居不再是 `Wait`：转移回 `BotState::Wait`。
    ///
    /// # 提示
    /// - 若卡壳可查阅参考实现 [`crates/swarm-core/src/reference/turing_morphogenesis.rs`](../reference/turing_morphogenesis.rs)。
    pub fn evaluate_state_transitions(
        &mut self,
        neighbors: &[NeighborObservation],
        dist_crit: f64,
        polar_th: f64,
    ) {
        todo!("【关卡 11 - 任务 3】在 robot.rs 中实现 Kilobot 三态行为状态机转移 evaluate_state_transitions");
    }
}
