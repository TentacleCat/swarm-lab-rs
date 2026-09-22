//! # SoNS: Self-organizing Nervous Systems for Robot Swarms (arXiv:2401.13103 / Science Robotics 2024)
//!
//! Reproduction and algorithmic study focused on **Section 4.1: SoNS Control** and **Section 4.2: Analysis Metrics**.
//!
//! ## Core Components
//! - [`allocator`]: Section 4.1 Node Allocation with Dynamic Replacement mechanism.
//! - [`controller`]: Section 4.1 Collective Actuation via mass-spring-damper motion and safezones.
//! - [`tree`]: Section 4.1 Hierarchy establishment, recruitment, splitting, and merging.
//! - [`metrics`]: Section 4.2 Equation (1) tracking error $E(t)$ and Equation (2) lower bound $B(t)$.

pub mod allocator;
pub mod controller;
pub mod metrics;
pub mod tree;
pub mod types;

pub use allocator::{AllocationResult, SoNSAllocator};
pub use controller::{MotionControlParams, SoNSMotionController};
pub use metrics::{compute_theoretical_lower_bound, compute_tracking_error, SoNSMetrics};
pub use tree::SoNSTreeManager;
pub use types::{
    vec2_add, vec2_length, vec2_normalize, vec2_rotate, vec2_scale, vec2_sub, MorphologySlot,
    RobotNode, RobotType, Vec2,
};

/// High-level discrete-time simulator for SoNS swarm self-organization and formation tracking.
#[derive(Debug, Clone)]
pub struct SoNSSimulator {
    pub robots: Vec<RobotNode>,
    pub obstacles: Vec<Vec2>,
    pub target_morphology: Vec<MorphologySlot>,
    pub allocator: SoNSAllocator,
    pub controller: SoNSMotionController,
    pub tree_mgr: SoNSTreeManager,
    pub time: f64,
    pub dt: f64,
    pub initial_positions: Vec<Vec2>,
    pub max_speeds: Vec<f64>,
}

impl SoNSSimulator {
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
            allocator: SoNSAllocator::default(),
            controller: SoNSMotionController::default(),
            tree_mgr: SoNSTreeManager::default(),
            time: 0.0,
            dt,
            initial_positions,
            max_speeds,
        }
    }

    /// Advance the SoNS system by one simulation timestep dt.
    pub fn step(&mut self) -> SoNSMetrics {
        let n = self.robots.len();

        // 1. Neighbor discovery & Swarm Merging (Section 4.1 "Merging SoNSs")
        for i in 0..n {
            for j in (i + 1)..n {
                self.tree_mgr.attempt_merge(&mut self.robots, i, j);
            }
        }

        // 2. Scale and Depth bottom-up aggregation (Section 4.1 & ScaleManager.lua)
        SoNSTreeManager::update_subtree_scales_and_depths(&mut self.robots);

        // 3. Recursive Slot Allocation & Dynamic Replacement (Section 4.1 "Node allocation")
        // Find dominant brain
        let brain_id = self
            .robots
            .iter()
            .enumerate()
            .filter(|(_, r)| r.is_brain())
            .max_by_key(|(_, r)| r.downstream_scale)
            .map(|(i, _)| i)
            .unwrap_or(0);

        // Target positions cache for error computation
        let mut target_world_positions: Vec<Option<Vec2>> = vec![None; n];
        target_world_positions[brain_id] = Some(self.robots[brain_id].position);

        // Allocate slots for parents with active morphology
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

        // 4. Motion Control: Compute velocities (Section 4.1 "Collective actuation via motion")
        let mut new_velocities = vec![[0.0, 0.0]; n];
        for i in 0..n {
            if i == brain_id {
                // Brain velocity (e.g. constant drift or guided)
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

        // 5. Integrate positions
        for i in 0..n {
            self.robots[i].velocity = new_velocities[i];
            self.robots[i].position =
                vec2_add(self.robots[i].position, vec2_scale(new_velocities[i], self.dt));
        }

        self.time += self.dt;

        // 6. Compute metrics (Section 4.2 Equations 1 & 2)
        let tracking_error =
            compute_tracking_error(&self.robots, brain_id, &target_world_positions);
        let lower_bound = compute_theoretical_lower_bound(
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
    fn test_sons_simulator_convergence() {
        // Create 1 Brain Drone at center and 4 ground robots scattered nearby
        let robots = vec![
            RobotNode::new(0, RobotType::Drone, [0.0, 0.0], 1.0),
            RobotNode::new(1, RobotType::Ground, [0.6, 0.2], 0.5),
            RobotNode::new(2, RobotType::Ground, [-0.5, -0.3], 0.5),
            RobotNode::new(3, RobotType::Ground, [0.1, 0.7], 0.5),
            RobotNode::new(4, RobotType::Ground, [-0.2, -0.6], 0.5),
        ];

        // Target cross morphology around Brain
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
            MorphologySlot {
                slot_id: 3,
                relative_offset: [0.0, 0.5],
                relative_yaw: 0.0,
                expected_type: RobotType::Ground,
                downstream_slots: Vec::new(),
            },
            MorphologySlot {
                slot_id: 4,
                relative_offset: [0.0, -0.5],
                relative_yaw: 0.0,
                expected_type: RobotType::Ground,
                downstream_slots: Vec::new(),
            },
        ];

        let mut sim = SoNSSimulator::new(robots, vec![], target_morphology, 0.1);

        let mut initial_metric = None;
        let mut final_metric = None;

        for step in 0..120 {
            let m = sim.step();
            if step == 0 {
                initial_metric = Some(m.clone());
            }
            if step == 119 {
                final_metric = Some(m);
            }
        }

        let m_init = initial_metric.unwrap();
        let m_final = final_metric.unwrap();

        // 1. Swarm should merge into a single unified SoNS
        assert_eq!(m_final.num_swarms, 1);
        assert_eq!(m_final.max_swarm_size, 5);

        // 2. Tracking error E must decrease significantly and converge close to 0
        assert!(m_final.tracking_error < m_init.tracking_error);
        assert!(
            m_final.tracking_error < 0.05,
            "Final tracking error was {}, expected < 0.05",
            m_final.tracking_error
        );
    }
}
