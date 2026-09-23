#![allow(unused_variables, dead_code, unused_imports)]

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
    /// 【关卡 14 - 任务 1】执行局域槽位分配与自组织就近置换 (Section 4.1 Node Allocation & Dynamic Replacement)
    ///
    /// 步骤包括：
    /// 1. 目标槽位世界坐标计算：根据 parent 姿态将 `relative_offset` 旋转加平移映射到全局世界坐标；
    /// 2. 保留已分配且类型匹配的合法现有子节点；
    /// 3. 贪心为未占用槽位指派欧氏距离最近且类型匹配的候选机器人；
    /// 4. 自组织就近置换 (Dynamic Replacement)：
    ///    - 遍历未分配候选者，检查其到已有槽位的距离是否比现有分配者更近至少 `self.replacement_margin`；
    ///    - 若优势明显，则将旧 Child 降级踢回候选者池（放入 `replaced_children`），将新候选者替换入槽；
    /// 5. 汇总生成 `AllocationResult`。
    ///
    /// # 提示
    /// - 若卡壳可参考 [`crates/swarm-core/src/reference/sons_hierarchy.rs`](../reference/sons_hierarchy.rs)。
    pub fn allocate_local_slots(
        &self,
        parent: &RobotNode,
        target_slots: &[MorphologySlot],
        candidates: &[RobotNode],
    ) -> AllocationResult {
        todo!("【关卡 14 - 任务 1】在 allocator.rs 中实现自组织就近置换槽位分配 allocate_local_slots");
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
