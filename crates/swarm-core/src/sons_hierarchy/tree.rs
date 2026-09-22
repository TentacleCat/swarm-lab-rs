use super::types::{vec2_length, vec2_sub, RobotNode};

/// Hierarchy management for SoNS network lifecycle.
#[derive(Debug, Clone)]
pub struct SoNSTreeManager {
    /// Maximum communication/sensing range for neighbor recruitment
    pub recruitment_range: f64,
}

impl Default for SoNSTreeManager {
    fn default() -> Self {
        Self {
            recruitment_range: 0.90, // 90 cm sensing/communication range
        }
    }
}

impl SoNSTreeManager {
    pub fn new(recruitment_range: f64) -> Self {
        Self { recruitment_range }
    }

    /// Establish child link between parent and child.
    pub fn add_child_link(robots: &mut [RobotNode], parent_id: usize, child_id: usize) {
        if parent_id == child_id {
            return;
        }

        // Get brain ID of parent
        let brain_id = robots[parent_id].brain_id;

        // Update child
        robots[child_id].parent_id = Some(parent_id);
        robots[child_id].brain_id = brain_id;

        // Update parent's children list
        if !robots[parent_id].children_ids.contains(&child_id) {
            robots[parent_id].children_ids.push(child_id);
        }

        // Propagate brain ID downstream through child's subtree
        Self::propagate_brain_id(robots, child_id, brain_id);
        // Refresh scale and depth
        Self::update_subtree_scales_and_depths(robots);
    }

    /// Split a child branch from parent (Section 4.1 "Splitting a SoNS").
    /// The expelled child automatically becomes the Brain of its own multi-level SoNS,
    /// retaining all its downstream children intact without re-initializing the subtree.
    pub fn split_branch(robots: &mut [RobotNode], parent_id: usize, child_id: usize) {
        // Remove child from parent
        robots[parent_id].children_ids.retain(|&id| id != child_id);

        // Child becomes independent Brain
        robots[child_id].parent_id = None;
        robots[child_id].brain_id = child_id;
        robots[child_id].target_relative_offset = None;

        // Propagate new brain ID through child's downstream subtree
        Self::propagate_brain_id(robots, child_id, child_id);
        Self::update_subtree_scales_and_depths(robots);
    }

    /// Merge two SoNS trees when their members encounter each other (Section 4.1 "Merging SoNSs").
    ///
    /// Compares the Brain quality/rank:
    /// - Primary criterion: total scale (number of robots in SoNS)
    /// - Secondary criterion: internal rank `id_N`
    /// The lower-quality Brain agrees to become a child (or joins under the higher-quality tree).
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
            return false; // already in the same SoNS
        }

        // Compare brain qualities
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
            // Brain B joins under Robot A
            Self::detach_from_parent(robots, brain_b);
            Self::add_child_link(robots, robot_a_id, brain_b);
        } else {
            // Brain A joins under Robot B
            Self::detach_from_parent(robots, brain_a);
            Self::add_child_link(robots, robot_b_id, brain_a);
        }

        true
    }

    /// Detach a node from its current parent if it has one.
    pub fn detach_from_parent(robots: &mut [RobotNode], node_id: usize) {
        if let Some(old_parent) = robots[node_id].parent_id {
            robots[old_parent].children_ids.retain(|&id| id != node_id);
            robots[node_id].parent_id = None;
        }
    }

    /// Recursively propagate brain ID downstream.
    fn propagate_brain_id(robots: &mut [RobotNode], root_id: usize, new_brain_id: usize) {
        let children = robots[root_id].children_ids.clone();
        for child_id in children {
            robots[child_id].brain_id = new_brain_id;
            Self::propagate_brain_id(robots, child_id, new_brain_id);
        }
    }

    /// Bottom-up aggregation of subtree scale and hierarchy depth (ScaleManager.lua).
    pub fn update_subtree_scales_and_depths(robots: &mut [RobotNode]) {
        let n = robots.len();
        // Identify all brains (roots)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sons_hierarchy::types::RobotType;

    #[test]
    fn test_tree_establishment_and_scale_aggregation() {
        let mut robots = vec![
            RobotNode::new(0, RobotType::Drone, [0.0, 0.0], 1.0),
            RobotNode::new(1, RobotType::Ground, [0.3, 0.0], 0.8),
            RobotNode::new(2, RobotType::Ground, [0.6, 0.0], 0.6),
            RobotNode::new(3, RobotType::Ground, [0.9, 0.0], 0.4),
        ];

        // 0 -> 1 -> 2 -> 3
        SoNSTreeManager::add_child_link(&mut robots, 0, 1);
        SoNSTreeManager::add_child_link(&mut robots, 1, 2);
        SoNSTreeManager::add_child_link(&mut robots, 2, 3);

        assert_eq!(robots[0].downstream_scale, 4);
        assert_eq!(robots[0].hierarchy_depth, 4);
        assert_eq!(robots[1].downstream_scale, 3);
        assert_eq!(robots[2].downstream_scale, 2);
        assert_eq!(robots[3].downstream_scale, 1);

        for r in &robots {
            assert_eq!(r.brain_id, 0);
        }
    }

    #[test]
    fn test_splitting_preserves_downstream_subtree() {
        let mut robots = vec![
            RobotNode::new(0, RobotType::Drone, [0.0, 0.0], 1.0),
            RobotNode::new(1, RobotType::Drone, [0.5, 0.0], 0.9),
            RobotNode::new(2, RobotType::Ground, [0.8, 0.0], 0.5),
            RobotNode::new(3, RobotType::Ground, [1.0, 0.0], 0.5),
        ];

        // 0 is Brain, with child 1. Child 1 has children 2 and 3.
        SoNSTreeManager::add_child_link(&mut robots, 0, 1);
        SoNSTreeManager::add_child_link(&mut robots, 1, 2);
        SoNSTreeManager::add_child_link(&mut robots, 1, 3);

        assert_eq!(robots[0].downstream_scale, 4);

        // Split child 1 from 0
        SoNSTreeManager::split_branch(&mut robots, 0, 1);

        // Robot 0 now has scale 1
        assert_eq!(robots[0].downstream_scale, 1);
        assert!(robots[0].children_ids.is_empty());

        // Robot 1 is now a new Brain with its children 2 and 3 intact!
        assert!(robots[1].is_brain());
        assert_eq!(robots[1].downstream_scale, 3);
        assert_eq!(robots[1].children_ids, vec![2, 3]);
        assert_eq!(robots[2].brain_id, 1);
        assert_eq!(robots[3].brain_id, 1);
    }

    #[test]
    fn test_merging_swarms_quality_comparison() {
        let mut robots = vec![
            // Swarm A: 0 -> 1 (scale 2)
            RobotNode::new(0, RobotType::Drone, [0.0, 0.0], 0.8),
            RobotNode::new(1, RobotType::Ground, [0.2, 0.0], 0.8),
            // Swarm B: 2 -> 3 -> 4 (scale 3)
            RobotNode::new(2, RobotType::Drone, [0.5, 0.0], 0.5),
            RobotNode::new(3, RobotType::Ground, [0.6, 0.0], 0.5),
            RobotNode::new(4, RobotType::Ground, [0.7, 0.0], 0.5),
        ];

        SoNSTreeManager::add_child_link(&mut robots, 0, 1);
        SoNSTreeManager::add_child_link(&mut robots, 2, 3);
        SoNSTreeManager::add_child_link(&mut robots, 3, 4);

        let mgr = SoNSTreeManager::new(1.0);
        // Swarm A member 1 meets Swarm B member 2
        let merged = mgr.attempt_merge(&mut robots, 1, 2);
        assert!(merged);

        // Swarm B (scale 3) dominates Swarm A (scale 2), so 2 recruits 1
        let common_brain = robots[2].brain_id;
        assert_eq!(common_brain, 2);
        for r in &robots {
            assert_eq!(r.brain_id, 2);
        }
        assert_eq!(robots[2].downstream_scale, 5);
    }
}
