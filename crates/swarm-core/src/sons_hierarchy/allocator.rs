use super::types::{vec2_add, vec2_length, vec2_rotate, vec2_sub, MorphologySlot, RobotNode, Vec2};

/// Result of an allocation round executed by a parent robot.
#[derive(Debug, Clone, Default)]
pub struct AllocationResult {
    /// Slot assignments: (slot_id, assigned_robot_id, target_world_pos)
    pub assignments: Vec<(usize, usize, Vec2)>,
    /// Robots that were replaced (demoted back to candidate pool) during this step
    pub replaced_children: Vec<usize>,
    /// Candidates that remained unallocated in this parent's scope
    pub unallocated_candidates: Vec<usize>,
}

/// Node Allocator implementing Section 4.1 "Node allocation" of the SoNS paper.
///
/// Features:
/// 1. Cost matrix computation based on target slot positions vs candidate positions.
/// 2. Optimal greedy/bipartite assignment of available candidates.
/// 3. Dynamic replacement: If an unallocated candidate is significantly closer to a slot
///    than the current assigned child, the parent swaps them, demoting the old child
///    so the swarm internally shifts instead of undergoing deep link cascades.
#[derive(Debug, Clone)]
pub struct SoNSAllocator {
    /// Replacement threshold margin: candidate must be closer by at least this margin (in meters)
    /// to trigger a dynamic replacement and prevent thrashing.
    pub replacement_margin: f64,
}

impl Default for SoNSAllocator {
    fn default() -> Self {
        Self {
            replacement_margin: 0.15, // 15 cm hysteresis margin
        }
    }
}

impl SoNSAllocator {
    pub fn new(replacement_margin: f64) -> Self {
        Self { replacement_margin }
    }

    /// Allocate children to local slots for a given parent.
    ///
    /// - `parent`: The parent robot executing the allocation.
    /// - `target_slots`: Child slots specified in the target morphology for this parent.
    /// - `candidates`: Pool of candidate robots (including newly seen free robots and existing children).
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

        // 1. Calculate world positions for each target slot based on parent's pose
        let slot_targets: Vec<(usize, Vec2)> = target_slots
            .iter()
            .map(|s| {
                let world_offset = vec2_rotate(s.relative_offset, parent.yaw);
                let world_pos = vec2_add(parent.position, world_offset);
                (s.slot_id, world_pos)
            })
            .collect();

        // Track assignment status
        let mut assigned_slots: Vec<Option<usize>> = vec![None; slot_targets.len()];
        let mut assigned_candidates = std::collections::HashSet::new();

        // 2. First, evaluate existing children already in slots
        for (slot_idx, (_slot_id, _slot_pos)) in slot_targets.iter().enumerate() {
            let expected_type = target_slots[slot_idx].expected_type;
            // Check if parent already has a child that matches this slot and robot type
            if let Some(child_id) = parent.children_ids.get(slot_idx) {
                if let Some(child) = candidates.iter().find(|c| c.id == *child_id) {
                    if child.robot_type == expected_type {
                        assigned_slots[slot_idx] = Some(child.id);
                        assigned_candidates.insert(child.id);
                    }
                }
            }
        }

        // 3. For any unassigned slot, greedily find the best unassigned candidate of matching type
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

        // 4. Dynamic Replacement (重点4.1: 自组织就近置换机制)
        // Check if any remaining candidate is substantially closer to an already-assigned slot
        // than its current occupant.
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

                        // If candidate is closer by at least replacement_margin
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

            // Execute replacement if beneficial
            if let Some((slot_idx, old_child_id)) = best_replacement_slot {
                // Demote old child to candidate pool
                result.replaced_children.push(old_child_id);
                assigned_candidates.remove(&old_child_id);

                // Assign new candidate to slot
                assigned_slots[slot_idx] = Some(candidate.id);
                assigned_candidates.insert(candidate.id);
            }
        }

        // 5. Build final output
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sons_hierarchy::types::RobotType;

    #[test]
    fn test_initial_allocation_by_distance() {
        let parent = RobotNode::new(0, RobotType::Drone, [0.0, 0.0], 1.0);
        let slots = vec![
            MorphologySlot {
                slot_id: 1,
                relative_offset: [1.0, 0.0],
                relative_yaw: 0.0,
                expected_type: RobotType::Ground,
                downstream_slots: Vec::new(),
            },
            MorphologySlot {
                slot_id: 2,
                relative_offset: [-1.0, 0.0],
                relative_yaw: 0.0,
                expected_type: RobotType::Ground,
                downstream_slots: Vec::new(),
            },
        ];

        let candidates = vec![
            RobotNode::new(10, RobotType::Ground, [0.9, 0.1], 0.5), // near slot 1
            RobotNode::new(20, RobotType::Ground, [-0.8, -0.1], 0.5), // near slot 2
            RobotNode::new(30, RobotType::Ground, [5.0, 5.0], 0.5),  // far
        ];

        let allocator = SoNSAllocator::default();
        let res = allocator.allocate_local_slots(&parent, &slots, &candidates);

        assert_eq!(res.assignments.len(), 2);
        assert_eq!(res.assignments[0].1, 10);
        assert_eq!(res.assignments[1].1, 20);
        assert_eq!(res.unallocated_candidates, vec![30]);
        assert!(res.replaced_children.is_empty());
    }

    #[test]
    fn test_dynamic_replacement_mechanism() {
        // Parent has existing child 10 assigned to slot 1 ([1.0, 0.0]), but child 10 is currently at [1.8, 0.0].
        // A newly recruited candidate 30 appears at [1.05, 0.0] (much closer!).
        let mut parent = RobotNode::new(0, RobotType::Drone, [0.0, 0.0], 1.0);
        parent.children_ids = vec![10];

        let slots = vec![MorphologySlot {
            slot_id: 1,
            relative_offset: [1.0, 0.0],
            relative_yaw: 0.0,
            expected_type: RobotType::Ground,
            downstream_slots: Vec::new(),
        }];

        let candidates = vec![
            RobotNode::new(10, RobotType::Ground, [1.8, 0.0], 0.5), // existing child (dist = 0.8)
            RobotNode::new(30, RobotType::Ground, [1.05, 0.0], 0.5), // candidate (dist = 0.05)
        ];

        let allocator = SoNSAllocator::new(0.2); // margin = 0.2m
        let res = allocator.allocate_local_slots(&parent, &slots, &candidates);

        // Replacement should trigger: 30 gets slot 1, 10 is demoted/replaced
        assert_eq!(res.assignments.len(), 1);
        assert_eq!(res.assignments[0].1, 30);
        assert_eq!(res.replaced_children, vec![10]);
        assert_eq!(res.unallocated_candidates, vec![10]);
    }
}
