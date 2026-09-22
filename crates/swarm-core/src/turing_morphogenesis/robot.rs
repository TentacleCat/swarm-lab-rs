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

    /// 评估并执行状态机转移 (完全依据原版 C 代码 edge_flow 逻辑)
    pub fn evaluate_state_transitions(
        &mut self,
        neighbors: &[NeighborObservation],
        dist_crit: f64,
        polar_th: f64,
    ) {
        let is_edge = self.edge_detector.is_edge();
        let my_polarized = self.is_polarized(polar_th);

        // 统计极化邻居信息
        let mut count_polarized = 0;
        let mut min_polar_dist = f64::MAX;
        let mut min_all_dist = f64::MAX;
        let mut nearest_neighbor_state = BotState::Wait;

        for nb in neighbors {
            if nb.dist < min_all_dist {
                min_all_dist = nb.dist;
                nearest_neighbor_state = nb.state;
            }
            if nb.morphogen.is_polarized(polar_th) {
                count_polarized += 1;
                if nb.dist < min_polar_dist {
                    min_polar_dist = nb.dist;
                }
            }
        }

        let all_neighbors_wait = neighbors.iter().all(|nb| nb.state == BotState::Wait);

        match self.state {
            BotState::Wait => {
                // 1. 判断是否进入 ORBIT
                // 条件：在边缘、所有邻居 WAIT、非极化(或虽极化但远离其他斑点)、计数归零、有邻居
                let can_orbit = is_edge
                    && all_neighbors_wait
                    && (!my_polarized
                        || count_polarized == 0
                        || (count_polarized >= 1 && min_polar_dist > dist_crit))
                    && (min_polar_dist > dist_crit || count_polarized < 2)
                    && self.wait_counter == 0
                    && !neighbors.is_empty();

                if can_orbit {
                    self.state = BotState::Orbit;
                    self.orbit_dir = 1.0; // 默认顺时针
                    return;
                }

                // 2. 判断是否进入 FOLLOW
                // 条件：在边缘、最近邻居处于 WAIT 且间距超过 dist_crit + 15
                let can_follow = is_edge
                    && nearest_neighbor_state == BotState::Wait
                    && min_all_dist > (dist_crit + 15.0)
                    && !neighbors.is_empty();

                if can_follow {
                    self.state = BotState::Follow;
                    return;
                }

                // 维护等待计数器
                if !all_neighbors_wait {
                    self.wait_counter = 10; // 暂停防止多机器人同时移动拥堵
                } else if self.wait_counter > 0 {
                    self.wait_counter -= 1;
                }
            }

            BotState::Orbit => {
                // 判断是否转回 WAIT (捕获于极化中心或脱离边缘)
                // 条件：最近邻也在移动，或者脱离边缘，或者到达极化斑点 (间距近且至少2个极化邻居)
                let reached_polar_spot = min_polar_dist <= dist_crit && count_polarized >= 2;
                let stop_orbit = !is_edge
                    || reached_polar_spot
                    || nearest_neighbor_state != BotState::Wait
                    || min_all_dist > (dist_crit + 25.0)
                    || neighbors.is_empty();

                if stop_orbit {
                    self.state = BotState::Wait;
                }
            }

            BotState::Follow => {
                // 判断是否结束 FOLLOW
                // 条件：已靠近邻居，或失去邻居，或最近邻不再 WAIT
                let stop_follow = min_all_dist <= dist_crit
                    || neighbors.is_empty()
                    || nearest_neighbor_state != BotState::Wait;

                if stop_follow {
                    self.state = BotState::Wait;
                }
            }
        }
    }
}
