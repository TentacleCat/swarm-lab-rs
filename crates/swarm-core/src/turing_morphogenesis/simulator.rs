#![allow(unused_variables, dead_code, unused_imports)]

//! 群体形态发生仿真器与物理运动学 (Morphogenesis Swarm Simulator & Kinematics)
//!
//! 整合图灵反应-扩散动力学、局域边界识别、环绕/跟随运动学与软碰撞斥力模型。

use super::morphogen::MorphogenParams;
use super::robot::{BotState, Kilobot, NeighborObservation};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// 形态发生全局物理与仿真统计快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmMetrics {
    pub step: usize,
    pub time: f64,
    pub total_robots: usize,
    pub polarized_count: usize,
    pub orbiting_count: usize,
    pub following_count: usize,
    pub waiting_count: usize,
    pub turing_spots_count: usize,
    pub gyration_radius: f64,
    pub max_extent: f64,
}

/// 群体机器人形态发生仿真世界
#[derive(Debug, Clone)]
pub struct MorphogenesisSwarm {
    /// 机器人个体列表
    pub robots: Vec<Kilobot>,
    /// 图灵动力学与通信参数
    pub params: MorphogenParams,
    /// 机器人物理半径 (Kilobot 半径约 16.5 mm, 直径 33.0 mm)
    pub bot_radius: f64,
    /// 环绕临界距离 (原代码 DIST_CRIT = 45.0 mm)
    pub dist_crit: f64,
    /// 机器人直线平移物理速度 (Kilobot 约 10 mm/s)
    pub move_speed: f64,
    /// 当前仿真时钟步数
    pub step_count: usize,
    /// 累计物理时间 (秒)
    pub elapsed_time: f64,
    /// 是否开启形态发生组织移动 (若为 false 则仅运行静态图灵反应-扩散)
    pub movement_enabled: bool,
}

impl MorphogenesisSwarm {
    /// 初始化密集团簇构型 (Pile formation, 类似 Kilombo morphogenesis.json 初始设定)
    pub fn new_pile<R: Rng>(
        n: usize,
        center: [f64; 2],
        cluster_radius: f64,
        params: MorphogenParams,
        rng: &mut R,
    ) -> Self {
        let mut robots = Vec::with_capacity(n);
        let bot_radius = 16.5;
        let min_sep = bot_radius * 2.1;

        // 在圆形区域内密堆积放置机器人
        for i in 0..n {
            let mut pos = center;
            let mut attempts = 0;
            while attempts < 2000 {
                let r = cluster_radius * rng.gen::<f64>().sqrt();
                let theta = rng.gen::<f64>() * 2.0 * std::f64::consts::PI;
                let candidate = [center[0] + r * theta.cos(), center[1] + r * theta.sin()];

                // 碰撞间距约束
                let overlap = robots.iter().any(|b: &Kilobot| {
                    let dx = b.pos[0] - candidate[0];
                    let dy = b.pos[1] - candidate[1];
                    (dx * dx + dy * dy).sqrt() < min_sep
                });

                if !overlap || attempts > 1800 {
                    pos = candidate;
                    break;
                }
                attempts += 1;
            }

            // 初始化学形态素赋予微小随机扰动以触发图灵失稳
            // 原代码: a0_b = rand_byte() * 100 / 255; u = a0_b * 0.01 * 6;
            let u0 = rng.gen_range(0.5..2.5);
            let v0 = rng.gen_range(0.5..2.5);

            robots.push(Kilobot::new(i, pos, u0, v0));
        }

        Self {
            robots,
            params,
            bot_radius,
            dist_crit: 45.0,
            move_speed: 10.0,
            step_count: 0,
            elapsed_time: 0.0,
            movement_enabled: true,
        }
    }

    /// 初始化矩形团簇构型 (用于测试不同初始几何的形态发生适应性，对应原论文 Fig. 4)
    pub fn new_rectangle<R: Rng>(
        n_x: usize,
        n_y: usize,
        spacing: f64,
        params: MorphogenParams,
        rng: &mut R,
    ) -> Self {
        let n = n_x * n_y;
        let mut robots = Vec::with_capacity(n);
        let start_x = -((n_x as f64 - 1.0) * spacing) / 2.0;
        let start_y = -((n_y as f64 - 1.0) * spacing) / 2.0;

        let mut id = 0;
        for i in 0..n_x {
            for j in 0..n_y {
                let jitter_x = rng.gen_range(-2.0..2.0);
                let jitter_y = rng.gen_range(-2.0..2.0);
                let pos = [
                    start_x + (i as f64) * spacing + jitter_x,
                    start_y + (j as f64) * spacing + jitter_y,
                ];
                let u0 = rng.gen_range(0.5..2.5);
                let v0 = rng.gen_range(0.5..2.5);
                robots.push(Kilobot::new(id, pos, u0, v0));
                id += 1;
            }
        }

        Self {
            robots,
            params,
            bot_radius: 16.5,
            dist_crit: 45.0,
            move_speed: 10.0,
            step_count: 0,
            elapsed_time: 0.0,
            movement_enabled: true,
        }
    }

    /// 计算所有机器人两两之间的距离矩阵
    fn compute_distances(&self) -> Vec<Vec<f64>> {
        let n = self.robots.len();
        let mut dists = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = self.robots[j].pos[0] - self.robots[i].pos[0];
                let dy = self.robots[j].pos[1] - self.robots[i].pos[1];
                let d = (dx * dx + dy * dy).sqrt();
                dists[i][j] = d;
                dists[j][i] = d;
            }
        }
        dists
    }

    /// 【关卡 11 - 任务 4】执行单步完整仿真更新
    ///
    /// 步骤包括：
    /// 1. 邻居与边缘检测：依据通信半径 `self.params.diff_r` 收集每个机器人的近邻观测 `NeighborObservation`，
    ///    回填各邻居的邻居数 `n_neighbors`，并调用 `edge_detector.update` 更新边缘平滑比率；
    /// 2. 图拉普拉斯反应-扩散更新：
    ///    - 仅静止态 `BotState::Wait` 参与扩散；
    ///    - $lap_u = \sum_{j \in \mathcal{N}_i, state=Wait} (u_j - u_i)$；
    ///    - $lap_v = \sum_{j \in \mathcal{N}_i, state=Wait} (v_j - v_i)$；
    ///    - 调用 `robot.morphogen.step(lap_u, lap_v, dt, &self.params)`；
    /// 3. 状态机评估：若 `self.movement_enabled`，调用 `evaluate_state_transitions`；
    /// 4. 物理运动指令生成：
    ///    - `BotState::Orbit`: 围绕最近的 `Wait` 邻居计算切向线速度向量与径向纠偏速度；
    ///    - `BotState::Follow`: 沿指向最近邻居单位向量以 `move_speed` 前进；
    /// 5. 实体软核防重叠排斥力：对距离 $d < 2 \cdot r_{bot}$ 的任意实体对施加对称弹性斥力；
    /// 6. 施加位移并自增 `step_count` 和 `elapsed_time`。
    ///
    /// # 提示
    /// - 若卡壳可参考 [`crates/swarm-core/src/reference/turing_morphogenesis.rs`](../reference/turing_morphogenesis.rs)。
    pub fn step(&mut self, dt: f64) {
        todo!("【关卡 11 - 任务 4】在 simulator.rs 中实现连续空间多智能体图灵形态发生单步仿真 step");
    }

    /// 仅运行图灵扩散步骤 (用于初期斑图孕育阶段)
    pub fn step_diffusion_only(&mut self, steps: usize, dt: f64) {
        let prev_movement = self.movement_enabled;
        self.movement_enabled = false;
        for _ in 0..steps {
            self.step(dt);
        }
        self.movement_enabled = prev_movement;
    }

    /// 切除断肢扰动操作 (Amputation Experiment, 验证自愈与再生机制)
    pub fn amputate<F>(&mut self, mut predicate: F) -> usize
    where
        F: FnMut(&Kilobot) -> bool,
    {
        let initial_len = self.robots.len();
        self.robots.retain(|bot| !predicate(bot));
        // 重新连续编号
        for (idx, bot) in self.robots.iter_mut().enumerate() {
            bot.id = idx;
        }
        initial_len - self.robots.len()
    }

    /// 计算斑点连通块数量 (Connected Components of Polarized Robots)
    pub fn count_turing_spots(&self) -> usize {
        let n = self.robots.len();
        let polarized_indices: Vec<usize> = (0..n)
            .filter(|&i| self.robots[i].is_polarized(self.params.polar_th))
            .collect();

        if polarized_indices.is_empty() {
            return 0;
        }

        let mut visited = vec![false; n];
        let mut spot_count = 0;

        for &start in &polarized_indices {
            if visited[start] {
                continue;
            }
            spot_count += 1;
            let mut queue = vec![start];
            visited[start] = true;

            while let Some(curr) = queue.pop() {
                for &other in &polarized_indices {
                    if !visited[other] {
                        let dx = self.robots[curr].pos[0] - self.robots[other].pos[0];
                        let dy = self.robots[curr].pos[1] - self.robots[other].pos[1];
                        if (dx * dx + dy * dy).sqrt() <= self.params.comm_r {
                            visited[other] = true;
                            queue.push(other);
                        }
                    }
                }
            }
        }

        spot_count
    }

    /// 收集当前仿真统计指标
    pub fn metrics(&self) -> SwarmMetrics {
        let n = self.robots.len();
        if n == 0 {
            return SwarmMetrics {
                step: self.step_count,
                time: self.elapsed_time,
                total_robots: 0,
                polarized_count: 0,
                orbiting_count: 0,
                following_count: 0,
                waiting_count: 0,
                turing_spots_count: 0,
                gyration_radius: 0.0,
                max_extent: 0.0,
            };
        }

        let mut polarized_count = 0;
        let mut orbiting_count = 0;
        let mut following_count = 0;
        let mut waiting_count = 0;

        let mut cx = 0.0;
        let mut cy = 0.0;

        for bot in &self.robots {
            cx += bot.pos[0];
            cy += bot.pos[1];
            if bot.is_polarized(self.params.polar_th) {
                polarized_count += 1;
            }
            match bot.state {
                BotState::Wait => waiting_count += 1,
                BotState::Orbit => orbiting_count += 1,
                BotState::Follow => following_count += 1,
            }
        }

        cx /= n as f64;
        cy /= n as f64;

        let mut gyr_sq = 0.0;
        let mut max_ext: f64 = 0.0;
        for bot in &self.robots {
            let dx = bot.pos[0] - cx;
            let dy = bot.pos[1] - cy;
            let d_sq = dx * dx + dy * dy;
            gyr_sq += d_sq;
            let d = d_sq.sqrt();
            if d > max_ext {
                max_ext = d;
            }
        }
        let gyration_radius = (gyr_sq / n as f64).sqrt();

        SwarmMetrics {
            step: self.step_count,
            time: self.elapsed_time,
            total_robots: n,
            polarized_count,
            orbiting_count,
            following_count,
            waiting_count,
            turing_spots_count: self.count_turing_spots(),
            gyration_radius,
            max_extent: max_ext,
        }
    }
}
