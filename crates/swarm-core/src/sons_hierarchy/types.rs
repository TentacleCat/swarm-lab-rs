use serde::{Deserialize, Serialize};

/// Type of robot in the heterogeneous aerial-ground swarm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RobotType {
    /// Aerial quadrotor (acts as supervisor/coordinator with wider field-of-view)
    Drone,
    /// Differential-drive ground robot (e-puck / pi-puck)
    Ground,
}

/// 2D Vector representation
pub type Vec2 = [f64; 2];

pub fn vec2_sub(a: Vec2, b: Vec2) -> Vec2 {
    [a[0] - b[0], a[1] - b[1]]
}

pub fn vec2_add(a: Vec2, b: Vec2) -> Vec2 {
    [a[0] + b[0], a[1] + b[1]]
}

pub fn vec2_scale(a: Vec2, s: f64) -> Vec2 {
    [a[0] * s, a[1] * s]
}

pub fn vec2_length(a: Vec2) -> f64 {
    (a[0] * a[0] + a[1] * a[1]).sqrt()
}

pub fn vec2_normalize(a: Vec2) -> Vec2 {
    let len = vec2_length(a);
    if len > 1e-9 {
        [a[0] / len, a[1] / len]
    } else {
        [0.0, 0.0]
    }
}

pub fn vec2_rotate(v: Vec2, angle: f64) -> Vec2 {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    [v[0] * cos_a - v[1] * sin_a, v[0] * sin_a + v[1] * cos_a]
}

/// A target morphology slot to be populated in the hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphologySlot {
    /// Slot identifier
    pub slot_id: usize,
    /// Target relative position displacement (d in the paper) in the parent's frame
    pub relative_offset: Vec2,
    /// Target relative orientation
    pub relative_yaw: f64,
    /// Expected robot type
    pub expected_type: RobotType,
    /// Recursive child slots downstream from this node
    pub downstream_slots: Vec<MorphologySlot>,
}

impl MorphologySlot {
    /// Total number of nodes in this subtree (including self)
    pub fn subtree_size(&self) -> usize {
        1 + self.downstream_slots.iter().map(|s| s.subtree_size()).sum::<usize>()
    }
}

/// State of an individual robot in the SoNS swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotNode {
    pub id: usize,
    pub robot_type: RobotType,
    pub position: Vec2,
    pub velocity: Vec2,
    pub yaw: f64,
    /// Current Brain ID of the SoNS to which this robot belongs
    pub brain_id: usize,
    /// Parent robot ID in the hierarchy (None if this robot is currently the Brain)
    pub parent_id: Option<usize>,
    /// Active children IDs currently connected
    pub children_ids: Vec<usize>,
    /// Quality or Rank metric for merge resolution (e.g. subtree scale or random priority in [0, 1])
    pub rank: f64,
    /// Aggregated subtree scale (number of robots in its branch including self)
    pub downstream_scale: usize,
    /// Hierarchy depth from this node down to farthest leaf
    pub hierarchy_depth: usize,
    /// Assigned target displacement from parent (if child)
    pub target_relative_offset: Option<Vec2>,
    /// Cooldown / lock timer to prevent rapid cyclic switching
    pub lock_timer: usize,
}

impl RobotNode {
    pub fn new(id: usize, robot_type: RobotType, position: Vec2, rank: f64) -> Self {
        Self {
            id,
            robot_type,
            position,
            velocity: [0.0, 0.0],
            yaw: 0.0,
            brain_id: id, // starts as independent single-robot SoNS (its own brain)
            parent_id: None,
            children_ids: Vec::new(),
            rank,
            downstream_scale: 1,
            hierarchy_depth: 1,
            target_relative_offset: None,
            lock_timer: 0,
        }
    }

    pub fn is_brain(&self) -> bool {
        self.parent_id.is_none() && self.brain_id == self.id
    }
}
