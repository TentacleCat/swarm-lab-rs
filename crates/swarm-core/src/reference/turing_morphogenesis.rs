//! # 💡 [参考答案区] 群体机器人图灵形态发生标准参考实现 (Reference Implementation)
//!
//! 论文: *Morphogenesis in robot swarms* (Science Robotics 2018, 3(25), eaau9178)
//!
//! 本文件包含经过完备单测验证的标准理论计算与基准实现，供闯关学习者对比查阅。

use rand::Rng;
use serde::{Deserialize, Serialize};

pub use crate::turing_morphogenesis::morphogen::{LedColor, MorphogenParams};
pub use crate::turing_morphogenesis::robot::BotState;

pub type MorphogenParamsReference = MorphogenParams;
pub type LedColorReference = LedColor;
pub type BotStateReference = BotState;

/// 机器人体内虚拟形态素浓度标准参考实现
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MorphogenConcentrationReference {
    pub u: f64,
    pub v: f64,
}

impl MorphogenConcentrationReference {
    pub fn new(u: f64, v: f64) -> Self {
        Self { u, v }
    }

    pub fn is_polarized(&self, threshold: f64) -> bool {
        self.u > threshold
    }

    /// 计算分段线性饱和反应动力学生成速率标准参考实现
    pub fn reaction_rates(&self, p: &MorphogenParams) -> (f64, f64) {
        let mut rate_u = p.a * self.u + p.b * self.v + p.c;
        if rate_u < 0.0 {
            rate_u = 0.0;
        } else if rate_u > p.synth_u_max {
            rate_u = p.synth_u_max;
        }
        rate_u -= p.d * self.u;

        let mut rate_v = p.e * self.u - p.f;
        if rate_v < 0.0 {
            rate_v = 0.0;
        } else if rate_v > p.synth_v_max {
            rate_v = p.synth_v_max;
        }
        rate_v -= p.g * self.v;

        (rate_u, rate_v)
    }

    /// 单步欧拉显式数值积分更新形态素浓度标准参考实现
    pub fn step(&mut self, lap_u: f64, lap_v: f64, dt: f64, p: &MorphogenParams) {
        let (rate_u, rate_v) = self.reaction_rates(p);
        let du = p.r_scale * rate_u + p.d_u * lap_u;
        let dv = p.r_scale * rate_v + p.d_v * lap_v;

        self.u += dt * du;
        self.v += dt * dv;

        if self.u < 0.0 {
            self.u = 0.0;
        }
        if self.v < 0.0 {
            self.v = 0.0;
        }
    }

    pub fn gradient_color(&self, polar_th: f64) -> LedColor {
        if self.u > polar_th {
            LedColor::Green
        } else if self.u > polar_th - 1.0 {
            LedColor::Cyan
        } else if self.u > polar_th - 2.0 {
            LedColor::Blue
        } else if self.u > polar_th - 3.0 {
            LedColor::Pink
        } else {
            LedColor::Black
        }
    }
}

/// 局域边缘检测器标准参考实现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDetectorReference {
    pub running_avg_ns: f64,
    pub running_avg_nns: f64,
    pub alpha: f64,
    pub edge_th: f64,
}

impl Default for EdgeDetectorReference {
    fn default() -> Self {
        Self {
            running_avg_ns: 0.0,
            running_avg_nns: 0.0,
            alpha: 0.05,
            edge_th: 0.80,
        }
    }
}

impl EdgeDetectorReference {
    pub fn new(alpha: f64, edge_th: f64) -> Self {
        Self {
            running_avg_ns: 0.0,
            running_avg_nns: 0.0,
            alpha,
            edge_th,
        }
    }

    pub fn initialize(&mut self, my_neighbors: usize, neighbors_info: &[(f64, usize)]) {
        self.running_avg_ns = my_neighbors as f64;
        self.running_avg_nns = Self::compute_weighted_nns(neighbors_info);
    }

    /// 计算邻居的距离反比加权邻居均值标准参考实现
    pub fn compute_weighted_nns(neighbors_info: &[(f64, usize)]) -> f64 {
        if neighbors_info.is_empty() {
            return 0.0;
        }
        let mut sum = 0.0;
        let mut w_sum = 0.0;
        for &(dist, n_neighbors) in neighbors_info {
            let w = 1.0 / dist.max(1.0);
            sum += w * (n_neighbors as f64);
            w_sum += w;
        }
        if w_sum > 1e-6 {
            sum / w_sum
        } else {
            0.0
        }
    }

    pub fn update(&mut self, my_neighbors: usize, neighbors_info: &[(f64, usize)]) {
        let current_n = my_neighbors as f64;
        let current_nns = Self::compute_weighted_nns(neighbors_info);

        self.running_avg_ns = self.alpha * current_n + (1.0 - self.alpha) * self.running_avg_ns;
        self.running_avg_nns = self.alpha * current_nns + (1.0 - self.alpha) * self.running_avg_nns;
    }

    pub fn is_edge(&self) -> bool {
        if self.running_avg_nns < 1e-3 {
            return true;
        }
        (self.running_avg_ns / self.running_avg_nns) < self.edge_th
    }

    pub fn edge_ratio(&self) -> f64 {
        if self.running_avg_nns < 1e-3 {
            0.0
        } else {
            self.running_avg_ns / self.running_avg_nns
        }
    }
}

/// 局域邻居观测数据结构标准参考实现
#[derive(Debug, Clone, Copy)]
pub struct NeighborObservationReference {
    pub id: usize,
    pub dist: f64,
    pub state: BotState,
    pub morphogen: MorphogenConcentrationReference,
    pub n_neighbors: usize,
}

/// Kilobot 机器人个体标准参考实现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KilobotReference {
    pub id: usize,
    pub pos: [f64; 2],
    pub heading: f64,
    pub state: BotState,
    pub morphogen: MorphogenConcentrationReference,
    pub edge_detector: EdgeDetectorReference,
    pub wait_counter: usize,
    pub orbit_dir: f64,
    pub is_stuck: bool,
}

impl KilobotReference {
    pub fn new(id: usize, pos: [f64; 2], u: f64, v: f64) -> Self {
        Self {
            id,
            pos,
            heading: 0.0,
            state: BotState::Wait,
            morphogen: MorphogenConcentrationReference::new(u, v),
            edge_detector: EdgeDetectorReference::default(),
            wait_counter: 0,
            orbit_dir: 1.0,
            is_stuck: false,
        }
    }

    pub fn led_color(&self, polar_th: f64) -> LedColor {
        match self.state {
            BotState::Orbit => LedColor::White,
            BotState::Follow => LedColor::Red,
            BotState::Wait => self.morphogen.gradient_color(polar_th),
        }
    }

    pub fn is_polarized(&self, polar_th: f64) -> bool {
        self.morphogen.is_polarized(polar_th)
    }

    /// 评估并执行状态机转移标准参考实现
    pub fn evaluate_state_transitions(
        &mut self,
        neighbors: &[NeighborObservationReference],
        dist_crit: f64,
        polar_th: f64,
    ) {
        let is_edge = self.edge_detector.is_edge();
        let my_polarized = self.is_polarized(polar_th);

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
                    self.orbit_dir = 1.0;
                    return;
                }

                let can_follow = is_edge
                    && nearest_neighbor_state == BotState::Wait
                    && min_all_dist > (dist_crit + 15.0)
                    && !neighbors.is_empty();

                if can_follow {
                    self.state = BotState::Follow;
                    return;
                }

                if !all_neighbors_wait {
                    self.wait_counter = 10;
                } else if self.wait_counter > 0 {
                    self.wait_counter -= 1;
                }
            }

            BotState::Orbit => {
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

pub use crate::turing_morphogenesis::simulator::SwarmMetrics;
pub type SwarmMetricsReference = SwarmMetrics;

/// 群体机器人形态发生仿真世界标准参考实现
#[derive(Debug, Clone)]
pub struct MorphogenesisSwarmReference {
    pub robots: Vec<KilobotReference>,
    pub params: MorphogenParams,
    pub bot_radius: f64,
    pub dist_crit: f64,
    pub move_speed: f64,
    pub step_count: usize,
    pub elapsed_time: f64,
    pub movement_enabled: bool,
}

impl MorphogenesisSwarmReference {
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

        for i in 0..n {
            let mut pos = center;
            let mut attempts = 0;
            while attempts < 2000 {
                let r = cluster_radius * rng.gen::<f64>().sqrt();
                let theta = rng.gen::<f64>() * 2.0 * std::f64::consts::PI;
                let candidate = [center[0] + r * theta.cos(), center[1] + r * theta.sin()];

                let overlap = robots.iter().any(|b: &KilobotReference| {
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

            let u0 = rng.gen_range(0.5..2.5);
            let v0 = rng.gen_range(0.5..2.5);

            robots.push(KilobotReference::new(i, pos, u0, v0));
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

    /// 执行单步完整仿真更新标准参考实现
    pub fn step(&mut self, dt: f64) {
        let n = self.robots.len();
        if n == 0 {
            return;
        }

        let dists = self.compute_distances();

        // 1. 提取邻居快照
        let mut neighbor_obs: Vec<Vec<NeighborObservationReference>> = Vec::with_capacity(n);
        for i in 0..n {
            let mut my_obs = Vec::new();
            for j in 0..n {
                if i != j && dists[i][j] <= self.params.diff_r {
                    my_obs.push(NeighborObservationReference {
                        id: j,
                        dist: dists[i][j],
                        state: self.robots[j].state,
                        morphogen: self.robots[j].morphogen,
                        n_neighbors: 0,
                    });
                }
            }
            neighbor_obs.push(my_obs);
        }

        // 回填邻居的邻居数 N_Neighbors 并更新边缘检测器
        let neighbor_counts: Vec<usize> = neighbor_obs.iter().map(|obs| obs.len()).collect();
        for i in 0..n {
            let count_i = neighbor_counts[i];
            for obs in &mut neighbor_obs[i] {
                obs.n_neighbors = neighbor_counts[obs.id];
            }
            let edge_input: Vec<(f64, usize)> =
                neighbor_obs[i].iter().map(|o| (o.dist, o.n_neighbors)).collect();
            self.robots[i].edge_detector.update(count_i, &edge_input);
        }

        // 2. 图拉普拉斯计算与反应-扩散更新 (仅 WAIT 参与扩散)
        for i in 0..n {
            if self.robots[i].state != BotState::Orbit && self.robots[i].state != BotState::Follow {
                let mut lap_u = 0.0;
                let mut lap_v = 0.0;
                let my_u = self.robots[i].morphogen.u;
                let my_v = self.robots[i].morphogen.v;

                for obs in &neighbor_obs[i] {
                    if obs.state == BotState::Wait {
                        lap_u += obs.morphogen.u - my_u;
                        lap_v += obs.morphogen.v - my_v;
                    }
                }

                self.robots[i].morphogen.step(lap_u, lap_v, dt, &self.params);
            }
        }

        // 3. 状态机评估
        if self.movement_enabled {
            for i in 0..n {
                self.robots[i].evaluate_state_transitions(
                    &neighbor_obs[i],
                    self.dist_crit,
                    self.params.polar_th,
                );
            }
        }

        // 4. 物理运动更新
        let mut displacements = vec![[0.0, 0.0]; n];

        for i in 0..n {
            match self.robots[i].state {
                BotState::Wait => {}
                BotState::Orbit => {
                    if let Some(nearest) = neighbor_obs[i]
                        .iter()
                        .filter(|o| o.state == BotState::Wait)
                        .min_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap())
                    {
                        let nb_pos = self.robots[nearest.id].pos;
                        let rx = self.robots[i].pos[0] - nb_pos[0];
                        let ry = self.robots[i].pos[1] - nb_pos[1];
                        let r_len = (rx * rx + ry * ry).sqrt().max(1e-4);

                        let tangent_x = (ry / r_len) * self.robots[i].orbit_dir;
                        let tangent_y = (-rx / r_len) * self.robots[i].orbit_dir;

                        let radial_err = r_len - self.dist_crit;
                        let radial_x = -(rx / r_len) * (radial_err * 0.3);
                        let radial_y = -(ry / r_len) * (radial_err * 0.3);

                        displacements[i][0] += (tangent_x * self.move_speed + radial_x) * dt;
                        displacements[i][1] += (tangent_y * self.move_speed + radial_y) * dt;
                    }
                }
                BotState::Follow => {
                    if let Some(nearest) = neighbor_obs[i]
                        .iter()
                        .min_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap())
                    {
                        let nb_pos = self.robots[nearest.id].pos;
                        let dx = nb_pos[0] - self.robots[i].pos[0];
                        let dy = nb_pos[1] - self.robots[i].pos[1];
                        let dist = (dx * dx + dy * dy).sqrt().max(1e-4);

                        displacements[i][0] += (dx / dist) * self.move_speed * dt;
                        displacements[i][1] += (dy / dist) * self.move_speed * dt;
                    }
                }
            }
        }

        // 5. 实体软核非重叠排斥力
        let min_dist = self.bot_radius * 2.0;
        for i in 0..n {
            for j in (i + 1)..n {
                let d = dists[i][j];
                if d < min_dist && d > 1e-4 {
                    let overlap = min_dist - d;
                    let nx = (self.robots[i].pos[0] - self.robots[j].pos[0]) / d;
                    let ny = (self.robots[i].pos[1] - self.robots[j].pos[1]) / d;
                    let push = overlap * 0.45;

                    displacements[i][0] += nx * push;
                    displacements[i][1] += ny * push;
                    displacements[j][0] -= nx * push;
                    displacements[j][1] -= ny * push;
                }
            }
        }

        for i in 0..n {
            self.robots[i].pos[0] += displacements[i][0];
            self.robots[i].pos[1] += displacements[i][1];
        }

        self.step_count += 1;
        self.elapsed_time += dt;
    }

    pub fn step_diffusion_only(&mut self, steps: usize, dt: f64) {
        let prev_movement = self.movement_enabled;
        self.movement_enabled = false;
        for _ in 0..steps {
            self.step(dt);
        }
        self.movement_enabled = prev_movement;
    }

    pub fn amputate<F>(&mut self, mut predicate: F) -> usize
    where
        F: FnMut(&KilobotReference) -> bool,
    {
        let initial_len = self.robots.len();
        self.robots.retain(|bot| !predicate(bot));
        for (idx, bot) in self.robots.iter_mut().enumerate() {
            bot.id = idx;
        }
        initial_len - self.robots.len()
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_ref_morphogen_kinetics() {
        let params = MorphogenParams::default();
        let conc = MorphogenConcentrationReference::new(1.0, 1.0);
        let (rate_u, rate_v) = conc.reaction_rates(&params);

        assert!((rate_u - 0.0).abs() < 1e-4);
        assert!((rate_v - (-0.06)).abs() < 1e-4);
    }

    #[test]
    fn test_ref_edge_detector() {
        let mut detector = EdgeDetectorReference::new(0.5, 0.8);
        let n_neighbors = 3;
        let neighbors_info = vec![(40.0, 6), (45.0, 6), (50.0, 6)];

        for _ in 0..10 {
            detector.update(n_neighbors, &neighbors_info);
        }

        assert!(detector.is_edge(), "Ratio 3/6 = 0.5 < 0.8 应判定为边缘节点");
    }

    #[test]
    fn test_ref_robot_state_machine() {
        let mut bot = KilobotReference::new(0, [0.0, 0.0], 1.0, 1.0);
        bot.state = BotState::Orbit;

        let polar_conc = MorphogenConcentrationReference::new(5.0, 1.0);
        let neighbors = vec![
            NeighborObservationReference {
                id: 1,
                dist: 35.0,
                state: BotState::Wait,
                morphogen: polar_conc,
                n_neighbors: 5,
            },
            NeighborObservationReference {
                id: 2,
                dist: 38.0,
                state: BotState::Wait,
                morphogen: polar_conc,
                n_neighbors: 5,
            },
        ];

        bot.evaluate_state_transitions(&neighbors, 45.0, 4.0);
        assert_eq!(bot.state, BotState::Wait);
    }

    #[test]
    fn test_ref_simulation_and_amputation() {
        let mut rng = StdRng::seed_from_u64(42);
        let params = MorphogenParams::default();
        let mut swarm = MorphogenesisSwarmReference::new_pile(50, [0.0, 0.0], 80.0, params, &mut rng);

        assert_eq!(swarm.robots.len(), 50);

        swarm.step_diffusion_only(5, 0.05);
        assert_eq!(swarm.step_count, 5);

        for _ in 0..10 {
            swarm.step(0.05);
        }

        let m = swarm.metrics();
        assert_eq!(m.total_robots, 50);

        let removed = swarm.amputate(|bot| bot.pos[0] > 20.0);
        assert!(removed > 0);
        assert_eq!(swarm.robots.len(), 50 - removed);
    }
}
