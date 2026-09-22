//! # Policy Generation, Pattern Encoding, and Verification
//!
//! 包含：
//! - 经典构型定义 `Pattern`（三角形、正方形、六边形等）；
//! - 自动由构型提取期望局部状态集合 $S_{\text{des}}$；
//! - 安全动作策略 $\Pi_{\text{safe}} = \Pi \setminus (\Pi_{\text{collision}} \cup \Pi_{\text{separation}})$ 的预计算；
//! - 构型自组织策略 $\Pi_f = \Pi_{\text{safe}} \setminus (S_{\text{des}} \times A)$ 的合成；
//! - 匹配方向矩阵 $D(S_{\text{des}})$ 与匹配度矩阵 $M(S_{\text{des}})$。

use super::state::{Direction, LocalState};
use std::collections::{BTreeSet, HashSet};

/// 几何构型定义
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    pub name: String,
    pub coords: Vec<(i32, i32)>,
}

impl Pattern {
    pub fn new(name: &str, coords: Vec<(i32, i32)>) -> Self {
        Self {
            name: name.to_string(),
            coords,
        }
    }

    /// 机器人总数
    #[inline(always)]
    pub fn size(&self) -> usize {
        self.coords.len()
    }

    /// 提取该构型对应的期望局部状态集合 $S_{\text{des}}$
    pub fn extract_desired_states(&self) -> Vec<LocalState> {
        let coord_set: HashSet<(i32, i32)> = self.coords.iter().cloned().collect();
        let mut state_set = BTreeSet::new();

        for &(x, y) in &self.coords {
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

    /// 4 机器人三角形构型 (Tri-4)
    pub fn triangle_4() -> Self {
        Self::new("Triangle-4", vec![(0, 0), (1, 0), (2, 0), (1, 1)])
    }

    /// 4 机器人正方形构型 (Square-4)
    pub fn square_4() -> Self {
        Self::new("Square-4", vec![(0, 0), (1, 0), (0, 1), (1, 1)])
    }

    /// 6 机器人正六边形构型 (Hexagon-6)
    pub fn hexagon_6() -> Self {
        Self::new(
            "Hexagon-6",
            vec![(0, 1), (1, 1), (2, 0), (1, -1), (0, -1), (-1, 0)],
        )
    }

    /// 9 机器人大型三角形构型 (Tri-9)
    pub fn triangle_9() -> Self {
        let mut coords = Vec::new();
        // 底层 5 个
        for x in 0..5 {
            coords.push((x, 0));
        }
        // 中层 3 个
        for x in 1..4 {
            coords.push((x, 1));
        }
        // 顶层 1 个
        coords.push((2, 2));

        Self::new("Triangle-9", coords)
    }

    /// 5 机器人十字星构型 (Cross-5)
    pub fn cross_5() -> Self {
        Self::new("Cross-5", vec![(0, 0), (0, 1), (0, -1), (1, 0), (-1, 0)])
    }

    /// N 机器人直线构型 (Line-N)
    pub fn line_n(n: usize) -> Self {
        let coords = (0..n).map(|x| (x as i32, 0)).collect();
        Self::new(&format!("Line-{}", n), coords)
    }
}

/// 状态-动作映射策略表
#[derive(Debug, Clone)]
pub struct Policy {
    /// 目标构型名称
    pub pattern_name: String,
    /// 期望局部状态集 $S_{\text{des}}$
    pub s_des: Vec<LocalState>,
    /// 映射表：索引为 `LocalState.0 as usize` (0..255)，值为允许执行的动作列表
    pub actions: [Vec<Direction>; 256],
}

impl Policy {
    /// 判定动作是否会导致局部邻域断连（Separation Check）
    ///
    /// 给定机器人处于 $(0, 0)$，其当前邻居集合为 $N$。
    /// 机器人若移动至新位置 $p_{\text{new}} = (\Delta x_k, \Delta y_k)$，
    /// 则在由节点集 $V = \{ p_{\text{new}} \} \cup N$ 构成的局部诱导图中：
    /// 任意两个节点如果 Chebyshev 距离 $\le 1$，则存在一条无向边。
    /// 若该图的连通分量数 $> 1$，说明这次移动会使原有邻居断连或机器人脱离原有邻居！
    pub fn is_separation_action(state: LocalState, dir: Direction) -> bool {
        let neighbors = state.neighbor_coords();
        if neighbors.is_empty() {
            // 没有邻居，单独移动脱离（在此体系中无孤立机器人允许移动）
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

        // 如果访问到的连通节点数不等于总节点数，说明图不连通
        count != n
    }

    /// 预计算全局基础安全策略 $\Pi_{\text{safe}}$
    ///
    /// 满足：
    /// 1. Be Careful: 不撞上已有邻居（$l_k = 0$）
    /// 2. Be Social: 移动后局部拓扑保持连通（`!is_separation_action`）
    pub fn compute_safe_policy() -> [Vec<Direction>; 256] {
        let mut safe = [const { Vec::new() }; 256];
        for s_val in 0..256 {
            let state = LocalState(s_val as u8);
            let mut allowed = Vec::with_capacity(8);

            for dir in Direction::ALL {
                // 1. 碰撞检查：该方向已有邻居
                if state.has_neighbor(dir) {
                    continue;
                }
                // 2. 断连检查：移动后邻域保持连通
                if Self::is_separation_action(state, dir) {
                    continue;
                }
                allowed.push(dir);
            }
            safe[s_val] = allowed;
        }
        safe
    }

    /// 从指定目标构型构建自组织策略 $\Pi_f = \Pi_{\text{safe}} \setminus (S_{\text{des}} \times A)$
    pub fn from_pattern(pattern: &Pattern) -> Self {
        let s_des = pattern.extract_desired_states();
        let s_des_set: HashSet<LocalState> = s_des.iter().cloned().collect();
        let safe = Self::compute_safe_policy();

        let mut actions = [const { Vec::new() }; 256];
        for s_val in 0..256 {
            let state = LocalState(s_val as u8);
            if s_des_set.contains(&state) {
                // Be Happy: 处于期望状态，绝不移动
                actions[s_val] = Vec::new();
            } else {
                actions[s_val] = safe[s_val].clone();
            }
        }

        Self {
            pattern_name: pattern.name.clone(),
            s_des,
            actions,
        }
    }

    /// 获取某一状态的可用动作列表
    #[inline(always)]
    pub fn get_actions(&self, state: LocalState) -> &[Direction] {
        &self.actions[state.0 as usize]
    }

    /// 判定某一状态是否为阻塞态（S_blocked: 非期望状态且无可行动作）
    #[inline(always)]
    pub fn is_blocked(&self, state: LocalState) -> bool {
        !self.is_desired(state) && self.actions[state.0 as usize].is_empty()
    }

    /// 判定某一状态是否为期望态（S_des）
    #[inline(always)]
    pub fn is_desired(&self, state: LocalState) -> bool {
        self.s_des.contains(&state)
    }

    /// 判定某一状态是否为活跃态（S_active: 拥有可选安全动作）
    #[inline(always)]
    pub fn is_active(&self, state: LocalState) -> bool {
        !self.actions[state.0 as usize].is_empty()
    }

    /// 计算匹配矩阵：匹配方向矩阵 $D(S_{\text{des}})$ 与匹配度矩阵 $M(S_{\text{des}})$
    ///
    /// - $D(i, j)$: 状态 $s_i$ 与状态 $s_j$ 可作为相邻邻居的相对方向列表；
    /// - $M(i, j)$: 状态 $s_i$ 与状态 $s_j$ 能够配对的方向数量。
    pub fn compute_match_matrices(&self) -> (Vec<Vec<Vec<Direction>>>, Vec<Vec<usize>>) {
        let d = self.s_des.len();
        let mut dir_matrix = vec![vec![Vec::new(); d]; d];
        let mut count_matrix = vec![vec![0usize; d]; d];

        for i in 0..d {
            let s_i = self.s_des[i];
            for j in 0..d {
                let s_j = self.s_des[j];
                for dir in Direction::ALL {
                    // 若 s_i 在 dir 处有邻居，且 s_j 在其反方向处有邻居
                    if s_i.has_neighbor(dir) && s_j.has_neighbor(dir.opposite()) {
                        dir_matrix[i][j].push(dir);
                    }
                }
                count_matrix[i][j] = dir_matrix[i][j].len();
            }
        }

        (dir_matrix, count_matrix)
    }
}
