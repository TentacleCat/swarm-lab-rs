//! # 💡 [参考答案区] 自组织神经系统 (SoNS) 动态多层级控制标准参考实现 (Reference Implementation)
//!
//! 论文: *Self-organizing nervous systems for robot swarms* (arXiv:2401.13103 / Science Robotics 2024)
//!
//! 本文件包含经过完备单测验证的标准理论计算与基准实现，供闯关学习者对比查阅。

pub use crate::sons_hierarchy::allocator::AllocationResult;
pub use crate::sons_hierarchy::controller::MotionControlParams;
pub use crate::sons_hierarchy::metrics::SoNSMetrics;
pub use crate::sons_hierarchy::types::{
    vec2_add, vec2_length, vec2_normalize, vec2_rotate, vec2_scale, vec2_sub, MorphologySlot,
    RobotNode, RobotType, Vec2,
};

pub type AllocationResultReference = AllocationResult;
pub type MotionControlParamsReference = MotionControlParams;
pub type SoNSMetricsReference = SoNSMetrics;
pub type MorphologySlotReference = MorphologySlot;
pub type RobotNodeReference = RobotNode;
pub type RobotTypeReference = RobotType;
pub type Vec2Reference = Vec2;

/// 节点分配与自组织就近置换标准参考实现
#[derive(Debug, Clone)]
pub struct SoNSAllocatorReference {
    pub replacement_margin: f64,
}

impl Default for SoNSAllocatorReference {
    fn default() -> Self {
        Self {
            replacement_margin: 0.15,
        }
    }
}

impl SoNSAllocatorReference {
    pub fn new(replacement_margin: f64) -> Self {
        Self { replacement_margin }
    }

    pub fn allocate_local_slots(
        &self,
        parent: &RobotNode,
        target_slots: &[MorphologySlot],
        candidates: &[RobotNode],
    ) -> AllocationResult {
        let mut result = AllocationResult::default();
        if target_slots.is_empty() || candidates.is_empty() {
            result.unallocated_candidates = candidates.iter().map(|c| c.id).collect();
            return result;
        }

        let slot_targets: Vec<(usize, Vec2)> = target_slots
            .iter()
            .map(|s| {
                let world_offset = vec2_rotate(s.relative_offset, parent.yaw);
                let world_pos = vec2_add(parent.position, world_offset);
                (s.slot_id, world_pos)
            })
            .collect();

        let mut assigned_slots: Vec<Option<usize>> = vec![None; slot_targets.len()];
        let mut assigned_candidates = std::collections::HashSet::new();

        // 1. 保留已有子节点
        for (slot_idx, (_slot_id, _slot_pos)) in slot_targets.iter().enumerate() {
            let expected_type = target_slots[slot_idx].expected_type;
            if let Some(child_id) = parent.children_ids.get(slot_idx) {
                if let Some(child) = candidates.iter().find(|c| c.id == *child_id) {
                    if child.robot_type == expected_type {
                        assigned_slots[slot_idx] = Some(child.id);
                        assigned_candidates.insert(child.id);
                    }
                }
            }
        }

        // 2. 贪心为未分配槽位匹配最近候选者
        for (slot_idx, (_slot_id, slot_pos)) in slot_targets.iter().enumerate() {
            if assigned_slots[slot_idx].is_some() {
                continue;
            }
            let expected_type = target_slots[slot_idx].expected_type;

            let mut best_candidate_id = None;
            let mut min_dist = f64::MAX;

            for candidate in candidates {
                if assigned_candidates.contains(&candidate.id) {
                    continue;
                }
                if candidate.robot_type != expected_type {
                    continue;
                }
                let dist = vec2_length(vec2_sub(candidate.position, *slot_pos));
                if dist < min_dist {
                    min_dist = dist;
                    best_candidate_id = Some(candidate.id);
                }
            }

            if let Some(cand_id) = best_candidate_id {
                assigned_slots[slot_idx] = Some(cand_id);
                assigned_candidates.insert(cand_id);
            }
        }

        // 3. Dynamic Replacement 自组织就近置换
        for candidate in candidates {
            if assigned_candidates.contains(&candidate.id) {
                continue;
            }

            let mut best_replacement_slot = None;
            let mut max_improvement = 0.0;

            for (slot_idx, (_slot_id, slot_pos)) in slot_targets.iter().enumerate() {
                if let Some(current_occupant_id) = assigned_slots[slot_idx] {
                    let expected_type = target_slots[slot_idx].expected_type;
                    if candidate.robot_type != expected_type {
                        continue;
                    }

                    if let Some(curr) = candidates.iter().find(|c| c.id == current_occupant_id) {
                        let curr_dist = vec2_length(vec2_sub(curr.position, *slot_pos));
                        let cand_dist = vec2_length(vec2_sub(candidate.position, *slot_pos));

                        if curr_dist - cand_dist > self.replacement_margin {
                            let improvement = curr_dist - cand_dist;
                            if improvement > max_improvement {
                                max_improvement = improvement;
                                best_replacement_slot = Some((slot_idx, current_occupant_id));
                            }
                        }
                    }
                }
            }

            if let Some((slot_idx, old_child_id)) = best_replacement_slot {
                result.replaced_children.push(old_child_id);
                assigned_candidates.remove(&old_child_id);

                assigned_slots[slot_idx] = Some(candidate.id);
                assigned_candidates.insert(candidate.id);
            }
        }

        for (slot_idx, (slot_id, slot_pos)) in slot_targets.into_iter().enumerate() {
            if let Some(robot_id) = assigned_slots[slot_idx] {
                result.assignments.push((slot_id, robot_id, slot_pos));
            }
        }

        for candidate in candidates {
            if !assigned_candidates.contains(&candidate.id) {
                result.unallocated_candidates.push(candidate.id);
            }
        }

        result
    }
}

/// 质量-弹簧-阻尼运动学控制器标准参考实现
#[derive(Debug, Clone)]
pub struct SoNSMotionControllerReference {
    pub params: MotionControlParams,
}

impl Default for SoNSMotionControllerReference {
    fn default() -> Self {
        Self {
            params: MotionControlParams::default(),
        }
    }
}

impl SoNSMotionControllerReference {
    pub fn new(params: MotionControlParams) -> Self {
        Self { params }
    }

    pub fn compute_follower_velocity(
        &self,
        robot: &RobotNode,
        parent: &RobotNode,
        obstacles: &[Vec2],
        nearby_robots: &[Vec2],
    ) -> Vec2 {
        let target_offset = match robot.target_relative_offset {
            Some(offset) => offset,
            None => [0.0, 0.0],
        };

        let target_world_offset = vec2_rotate(target_offset, parent.yaw);
        let target_pos = vec2_add(parent.position, target_world_offset);

        let err = vec2_sub(target_pos, robot.position);
        let dist = vec2_length(err);

        // 1. 质量-弹簧-阻尼运动学
        let v_track = if dist < self.params.stop_zone {
            [0.0, 0.0]
        } else if dist < self.params.slowdown_zone {
            let scale = self.params.max_speed * (dist / self.params.slowdown_zone);
            vec2_scale(vec2_normalize(err), scale)
        } else {
            vec2_scale(vec2_normalize(err), self.params.max_speed)
        };

        // 2. 障碍物排斥
        let mut v_obs = [0.0, 0.0];
        for &obs_pos in obstacles {
            let diff = vec2_sub(robot.position, obs_pos);
            let d = vec2_length(diff);
            if d > 1e-4 && d < self.params.obstacle_radius {
                let mag = self.params.obstacle_gain * (1.0 / d - 1.0 / self.params.obstacle_radius);
                let force = vec2_scale(vec2_normalize(diff), mag);
                v_obs = vec2_add(v_obs, force);
            }
        }

        // 3. 机器人间避碰
        let mut v_repel = [0.0, 0.0];
        for &other_pos in nearby_robots {
            let diff = vec2_sub(robot.position, other_pos);
            let d = vec2_length(diff);
            if d > 1e-4 && d < self.params.inter_robot_avoid_radius {
                let mag = self.params.inter_robot_avoid_gain
                    * (self.params.inter_robot_avoid_radius - d);
                let force = vec2_scale(vec2_normalize(diff), mag);
                v_repel = vec2_add(v_repel, force);
            }
        }

        let mut v_cmd = vec2_add(vec2_add(v_track, v_obs), v_repel);

        let speed = vec2_length(v_cmd);
        if speed > self.params.max_speed {
            v_cmd = vec2_scale(vec2_normalize(v_cmd), self.params.max_speed);
        }

        // 4. 安全区视距约束
        let next_pos = vec2_add(robot.position, vec2_scale(v_cmd, 0.1));
        let dist_to_parent_next = vec2_length(vec2_sub(next_pos, parent.position));
        if dist_to_parent_next > self.params.safezone_radius {
            let parent_dir = vec2_normalize(vec2_sub(parent.position, robot.position));
            let v_radial = v_cmd[0] * parent_dir[0] + v_cmd[1] * parent_dir[1];
            if v_radial < 0.0 {
                v_cmd = [
                    v_cmd[0] - v_radial * parent_dir[0],
                    v_cmd[1] - v_radial * parent_dir[1],
                ];
            }
        }

        v_cmd
    }
}

/// 拓扑树状网络管理器标准参考实现
#[derive(Debug, Clone)]
pub struct SoNSTreeManagerReference {
    pub recruitment_range: f64,
}

impl Default for SoNSTreeManagerReference {
    fn default() -> Self {
        Self {
            recruitment_range: 0.90,
        }
    }
}

impl SoNSTreeManagerReference {
    pub fn new(recruitment_range: f64) -> Self {
        Self { recruitment_range }
    }

    pub fn add_child_link(robots: &mut [RobotNode], parent_id: usize, child_id: usize) {
        if parent_id == child_id {
            return;
        }

        let brain_id = robots[parent_id].brain_id;
        robots[child_id].parent_id = Some(parent_id);
        robots[child_id].brain_id = brain_id;

        if !robots[parent_id].children_ids.contains(&child_id) {
            robots[parent_id].children_ids.push(child_id);
        }

        Self::propagate_brain_id(robots, child_id, brain_id);
        Self::update_subtree_scales_and_depths(robots);
    }

    pub fn split_branch(robots: &mut [RobotNode], parent_id: usize, child_id: usize) {
        robots[parent_id].children_ids.retain(|&id| id != child_id);

        robots[child_id].parent_id = None;
        robots[child_id].brain_id = child_id;
        robots[child_id].target_relative_offset = None;

        Self::propagate_brain_id(robots, child_id, child_id);
        Self::update_subtree_scales_and_depths(robots);
    }

    pub fn attempt_merge(
        &self,
        robots: &mut [RobotNode],
        robot_a_id: usize,
        robot_b_id: usize,
    ) -> bool {
        let dist = vec2_length(vec2_sub(robots[robot_a_id].position, robots[robot_b_id].position));
        if dist > self.recruitment_range {
            return false;
        }

        let brain_a = robots[robot_a_id].brain_id;
        let brain_b = robots[robot_b_id].brain_id;
        if brain_a == brain_b {
            return false;
        }

        let scale_a = robots[brain_a].downstream_scale;
        let scale_b = robots[brain_b].downstream_scale;
        let rank_a = robots[brain_a].rank;
        let rank_b = robots[brain_b].rank;

        let a_wins = if scale_a != scale_b {
            scale_a > scale_b
        } else {
            rank_a >= rank_b
        };

        if a_wins {
            Self::detach_from_parent(robots, brain_b);
            Self::add_child_link(robots, robot_a_id, brain_b);
        } else {
            Self::detach_from_parent(robots, brain_a);
            Self::add_child_link(robots, robot_b_id, brain_a);
        }

        true
    }

    pub fn detach_from_parent(robots: &mut [RobotNode], node_id: usize) {
        if let Some(old_parent) = robots[node_id].parent_id {
            robots[old_parent].children_ids.retain(|&id| id != node_id);
            robots[node_id].parent_id = None;
        }
    }

    fn propagate_brain_id(robots: &mut [RobotNode], root_id: usize, new_brain_id: usize) {
        let children = robots[root_id].children_ids.clone();
        for child_id in children {
            robots[child_id].brain_id = new_brain_id;
            Self::propagate_brain_id(robots, child_id, new_brain_id);
        }
    }

    pub fn update_subtree_scales_and_depths(robots: &mut [RobotNode]) {
        let n = robots.len();
        let brains: Vec<usize> = (0..n).filter(|&i| robots[i].is_brain()).collect();

        for brain_id in brains {
            Self::calc_scale_depth_recursive(robots, brain_id);
        }
    }

    fn calc_scale_depth_recursive(robots: &mut [RobotNode], node_id: usize) -> (usize, usize) {
        let children = robots[node_id].children_ids.clone();
        let mut total_scale = 1;
        let mut max_depth = 0;

        for child_id in children {
            let (child_scale, child_depth) = Self::calc_scale_depth_recursive(robots, child_id);
            total_scale += child_scale;
            if child_depth > max_depth {
                max_depth = child_depth;
            }
        }

        robots[node_id].downstream_scale = total_scale;
        robots[node_id].hierarchy_depth = max_depth + 1;

        (total_scale, max_depth + 1)
    }
}

/// 跟踪误差与理论下界计算标准参考实现
pub fn ref_compute_tracking_error(
    robots: &[RobotNode],
    brain_id: usize,
    target_world_positions: &[Option<Vec2>],
) -> f64 {
    let n = robots.len();
    if n == 0 {
        return 0.0;
    }

    let p1 = robots[brain_id].position;
    let f1 = target_world_positions[brain_id].unwrap_or(p1);

    let mut sum_error = 0.0;
    let mut count = 0;

    for (i, robot) in robots.iter().enumerate() {
        if i == brain_id {
            count += 1;
            continue;
        }

        if let Some(fi) = target_world_positions[i] {
            let actual_dist = vec2_length(vec2_sub(robot.position, p1));
            let target_dist = vec2_length(vec2_sub(fi, f1));
            let ei = (actual_dist - target_dist).abs();
            sum_error += ei;
            count += 1;
        }
    }

    if count > 0 {
        sum_error / (count as f64)
    } else {
        0.0
    }
}

pub fn ref_compute_theoretical_lower_bound(
    start_positions: &[Vec2],
    target_positions: &[Option<Vec2>],
    max_speeds: &[f64],
    elapsed_time: f64,
) -> f64 {
    let n = start_positions.len();
    if n == 0 {
        return 0.0;
    }

    let mut sum_bound = 0.0;
    let mut count = 0;

    for i in 0..n {
        if let Some(target) = target_positions[i] {
            let initial_dist = vec2_length(vec2_sub(start_positions[i], target));
            let travel_dist = max_speeds[i] * elapsed_time;
            let bi = (initial_dist - travel_dist).max(0.0);
            sum_bound += bi;
            count += 1;
        }
    }

    if count > 0 {
        sum_bound / (count as f64)
    } else {
        0.0
    }
}

/// 高层离散时间多智能体 SoNS 仿真器标准参考实现
#[derive(Debug, Clone)]
pub struct SoNSSimulatorReference {
    pub robots: Vec<RobotNode>,
    pub obstacles: Vec<Vec2>,
    pub target_morphology: Vec<MorphologySlot>,
    pub allocator: SoNSAllocatorReference,
    pub controller: SoNSMotionControllerReference,
    pub tree_mgr: SoNSTreeManagerReference,
    pub time: f64,
    pub dt: f64,
    pub initial_positions: Vec<Vec2>,
    pub max_speeds: Vec<f64>,
}

impl SoNSSimulatorReference {
    pub fn new(
        robots: Vec<RobotNode>,
        obstacles: Vec<Vec2>,
        target_morphology: Vec<MorphologySlot>,
        dt: f64,
    ) -> Self {
        let initial_positions = robots.iter().map(|r| r.position).collect();
        let max_speeds = robots
            .iter()
            .map(|r| match r.robot_type {
                RobotType::Drone => 0.25,
                RobotType::Ground => 0.15,
            })
            .collect();

        Self {
            robots,
            obstacles,
            target_morphology,
            allocator: SoNSAllocatorReference::default(),
            controller: SoNSMotionControllerReference::default(),
            tree_mgr: SoNSTreeManagerReference::default(),
            time: 0.0,
            dt,
            initial_positions,
            max_speeds,
        }
    }

    pub fn step(&mut self) -> SoNSMetrics {
        let n = self.robots.len();

        for i in 0..n {
            for j in (i + 1)..n {
                self.tree_mgr.attempt_merge(&mut self.robots, i, j);
            }
        }

        SoNSTreeManagerReference::update_subtree_scales_and_depths(&mut self.robots);

        let brain_id = self
            .robots
            .iter()
            .enumerate()
            .filter(|(_, r)| r.is_brain())
            .max_by_key(|(_, r)| r.downstream_scale)
            .map(|(i, _)| i)
            .unwrap_or(0);

        let mut target_world_positions: Vec<Option<Vec2>> = vec![None; n];
        target_world_positions[brain_id] = Some(self.robots[brain_id].position);

        let parent = self.robots[brain_id].clone();
        let candidates = self.robots.clone();
        let alloc_res = self
            .allocator
            .allocate_local_slots(&parent, &self.target_morphology, &candidates);

        for (slot_id, robot_id, world_target) in &alloc_res.assignments {
            if let Some(slot) = self.target_morphology.iter().find(|s| s.slot_id == *slot_id) {
                self.robots[*robot_id].target_relative_offset = Some(slot.relative_offset);
                target_world_positions[*robot_id] = Some(*world_target);
            }
        }

        let mut new_velocities = vec![[0.0, 0.0]; n];
        for i in 0..n {
            if i == brain_id {
                new_velocities[i] = [0.0, 0.0];
                continue;
            }

            if let Some(parent_idx) = self.robots[i].parent_id {
                let parent_node = self.robots[parent_idx].clone();
                let robot_node = self.robots[i].clone();

                let other_robot_positions: Vec<Vec2> = self
                    .robots
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| *idx != i)
                    .map(|(_, r)| r.position)
                    .collect();

                let vel = self.controller.compute_follower_velocity(
                    &robot_node,
                    &parent_node,
                    &self.obstacles,
                    &other_robot_positions,
                );
                new_velocities[i] = vel;
            }
        }

        for i in 0..n {
            self.robots[i].velocity = new_velocities[i];
            self.robots[i].position =
                vec2_add(self.robots[i].position, vec2_scale(new_velocities[i], self.dt));
        }

        self.time += self.dt;

        let tracking_error =
            ref_compute_tracking_error(&self.robots, brain_id, &target_world_positions);
        let lower_bound = ref_compute_theoretical_lower_bound(
            &self.initial_positions,
            &target_world_positions,
            &self.max_speeds,
            self.time,
        );

        let num_swarms = self.robots.iter().filter(|r| r.is_brain()).count();
        let max_depth = self.robots[brain_id].hierarchy_depth;
        let max_swarm_size = self.robots[brain_id].downstream_scale;

        SoNSMetrics {
            time: self.time,
            tracking_error,
            theoretical_lower_bound: lower_bound,
            max_depth,
            num_swarms,
            max_swarm_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ref_allocation_and_replacement() {
        let parent = RobotNode::new(0, RobotType::Drone, [0.0, 0.0], 1.0);
        let slots = vec![MorphologySlot {
            slot_id: 1,
            relative_offset: [1.0, 0.0],
            relative_yaw: 0.0,
            expected_type: RobotType::Ground,
            downstream_slots: Vec::new(),
        }];

        let candidates = vec![
            RobotNode::new(10, RobotType::Ground, [1.8, 0.0], 0.5),
            RobotNode::new(30, RobotType::Ground, [1.05, 0.0], 0.5),
        ];

        let allocator = SoNSAllocatorReference::new(0.2);
        let res = allocator.allocate_local_slots(&parent, &slots, &candidates);
        assert_eq!(res.assignments[0].1, 30);
    }

    #[test]
    fn test_ref_motion_controller() {
        let parent = RobotNode::new(0, RobotType::Drone, [0.0, 0.0], 1.0);
        let mut follower = RobotNode::new(1, RobotType::Ground, [0.5, 0.0], 0.5);
        follower.target_relative_offset = Some([1.0, 0.0]);

        let ctrl = SoNSMotionControllerReference::default();
        let v = ctrl.compute_follower_velocity(&follower, &parent, &[], &[]);
        assert!(v[0] > 0.0);
    }

    #[test]
    fn test_ref_tree_split_merge() {
        let mut robots = vec![
            RobotNode::new(0, RobotType::Drone, [0.0, 0.0], 1.0),
            RobotNode::new(1, RobotType::Ground, [0.3, 0.0], 0.8),
        ];

        SoNSTreeManagerReference::add_child_link(&mut robots, 0, 1);
        assert_eq!(robots[0].downstream_scale, 2);

        SoNSTreeManagerReference::split_branch(&mut robots, 0, 1);
        assert_eq!(robots[0].downstream_scale, 1);
        assert!(robots[1].is_brain());
    }

    #[test]
    fn test_ref_simulator_convergence() {
        let robots = vec![
            RobotNode::new(0, RobotType::Drone, [0.0, 0.0], 1.0),
            RobotNode::new(1, RobotType::Ground, [0.6, 0.2], 0.5),
            RobotNode::new(2, RobotType::Ground, [-0.5, -0.3], 0.5),
        ];

        let target_morphology = vec![
            MorphologySlot {
                slot_id: 1,
                relative_offset: [0.5, 0.0],
                relative_yaw: 0.0,
                expected_type: RobotType::Ground,
                downstream_slots: Vec::new(),
            },
            MorphologySlot {
                slot_id: 2,
                relative_offset: [-0.5, 0.0],
                relative_yaw: 0.0,
                expected_type: RobotType::Ground,
                downstream_slots: Vec::new(),
            },
        ];

        let mut sim = SoNSSimulatorReference::new(robots, vec![], target_morphology, 0.1);
        for _ in 0..50 {
            sim.step();
        }

        let m = sim.step();
        assert_eq!(m.num_swarms, 1);
        assert_eq!(m.max_swarm_size, 3);
    }
}
