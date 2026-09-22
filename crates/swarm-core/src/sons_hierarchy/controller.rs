use super::types::{
    vec2_add, vec2_length, vec2_normalize, vec2_rotate, vec2_scale, vec2_sub, RobotNode, Vec2,
};

/// Parameters for SoNS motion control (derived from Driver.lua, Avoider.lua, Stabilizer.lua).
#[derive(Debug, Clone)]
pub struct MotionControlParams {
    /// Maximum linear velocity (m/s)
    pub max_speed: f64,
    /// Distance threshold below which tracking velocity is zero (deadband)
    pub stop_zone: f64,
    /// Distance threshold below which velocity scales down linearly
    pub slowdown_zone: f64,
    /// Safezone radius: maximum allowable distance to maintain parent link
    pub safezone_radius: f64,
    /// Obstacle repulsion gain
    pub obstacle_gain: f64,
    /// Obstacle influence radius
    pub obstacle_radius: f64,
    /// Inter-robot collision avoidance radius
    pub inter_robot_avoid_radius: f64,
    /// Inter-robot avoidance force gain
    pub inter_robot_avoid_gain: f64,
}

impl Default for MotionControlParams {
    fn default() -> Self {
        Self {
            max_speed: 0.15,                 // 15 cm/s (e-puck standard)
            stop_zone: 0.02,                 // 2 cm
            slowdown_zone: 0.20,             // 20 cm
            safezone_radius: 1.20,           // 1.2 m
            obstacle_gain: 0.08,             // obstacle repulsion coefficient
            obstacle_radius: 0.40,           // 40 cm
            inter_robot_avoid_radius: 0.25,   // 25 cm
            inter_robot_avoid_gain: 0.05,
        }
    }
}

/// Motion controller for collective actuation in SoNS.
#[derive(Debug, Clone)]
pub struct SoNSMotionController {
    pub params: MotionControlParams,
}

impl Default for SoNSMotionController {
    fn default() -> Self {
        Self {
            params: MotionControlParams::default(),
        }
    }
}

impl SoNSMotionController {
    pub fn new(params: MotionControlParams) -> Self {
        Self { params }
    }

    /// Calculate target velocity for a follower robot based on parent guidance and environment.
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

        // 1. Compute target position in world coordinates based on parent pose
        let target_world_offset = vec2_rotate(target_offset, parent.yaw);
        let target_pos = vec2_add(parent.position, target_world_offset);

        // Tracking error
        let err = vec2_sub(target_pos, robot.position);
        let dist = vec2_length(err);

        // 2. Mass-spring-damper kinematic tracking velocity (Section 4.1 & Driver.lua)
        let v_track = if dist < self.params.stop_zone {
            [0.0, 0.0]
        } else if dist < self.params.slowdown_zone {
            let scale = self.params.max_speed * (dist / self.params.slowdown_zone);
            vec2_scale(vec2_normalize(err), scale)
        } else {
            vec2_scale(vec2_normalize(err), self.params.max_speed)
        };

        // 3. Decentralized Obstacle Repulsion (Avoider.lua)
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

        // 4. Inter-robot collision avoidance (Spreader.lua)
        let mut v_repel = [0.0, 0.0];
        for &other_pos in nearby_robots {
            let diff = vec2_sub(robot.position, other_pos);
            let d = vec2_length(diff);
            if d > 1e-4 && d < self.params.inter_robot_avoid_radius {
                let mag = self.params.inter_robot_avoid_gain * (self.params.inter_robot_avoid_radius - d);
                let force = vec2_scale(vec2_normalize(diff), mag);
                v_repel = vec2_add(v_repel, force);
            }
        }

        // Combined velocity
        let mut v_cmd = vec2_add(vec2_add(v_track, v_obs), v_repel);

        // Clamp speed
        let speed = vec2_length(v_cmd);
        if speed > self.params.max_speed {
            v_cmd = vec2_scale(vec2_normalize(v_cmd), self.params.max_speed);
        }

        // 5. Safezone containment check (Driver.lua lines 130-150)
        // If commanded velocity would move robot beyond safezone from parent, dampen or stop it
        let next_pos = vec2_add(robot.position, vec2_scale(v_cmd, 0.1));
        let dist_to_parent_next = vec2_length(vec2_sub(next_pos, parent.position));
        if dist_to_parent_next > self.params.safezone_radius {
            // Project velocity so it does not increase distance to parent
            let parent_dir = vec2_normalize(vec2_sub(parent.position, robot.position));
            let v_radial = v_cmd[0] * parent_dir[0] + v_cmd[1] * parent_dir[1];
            if v_radial < 0.0 {
                // Moving away from parent, cancel outward component
                v_cmd = [v_cmd[0] - v_radial * parent_dir[0], v_cmd[1] - v_radial * parent_dir[1]];
            }
        }

        v_cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sons_hierarchy::types::RobotType;

    #[test]
    fn test_mass_spring_damper_deadzone_and_slowdown() {
        let parent = RobotNode::new(0, RobotType::Drone, [0.0, 0.0], 1.0);
        let mut follower = RobotNode::new(1, RobotType::Ground, [0.99, 0.0], 0.5);
        follower.target_relative_offset = Some([1.0, 0.0]);

        let ctrl = SoNSMotionController::default();

        // 1. Within stop_zone (dist = 0.01 < 0.02)
        let v1 = ctrl.compute_follower_velocity(&follower, &parent, &[], &[]);
        assert_eq!(v1, [0.0, 0.0]);

        // 2. In slowdown zone (dist = 0.10 in (0.02, 0.20))
        follower.position = [0.90, 0.0];
        let v2 = ctrl.compute_follower_velocity(&follower, &parent, &[], &[]);
        let speed2 = vec2_length(v2);
        assert!(speed2 > 0.0 && speed2 < ctrl.params.max_speed);

        // 3. Beyond slowdown zone (dist = 0.5 > 0.20)
        follower.position = [0.50, 0.0];
        let v3 = ctrl.compute_follower_velocity(&follower, &parent, &[], &[]);
        let speed3 = vec2_length(v3);
        assert!((speed3 - ctrl.params.max_speed).abs() < 1e-6);
        assert!(v3[0] > 0.0); // moving toward +x target
    }

    #[test]
    fn test_obstacle_avoidance_superposition() {
        let parent = RobotNode::new(0, RobotType::Drone, [0.0, 0.0], 1.0);
        let mut follower = RobotNode::new(1, RobotType::Ground, [0.5, 0.0], 0.5);
        follower.target_relative_offset = Some([1.0, 0.0]);

        let ctrl = SoNSMotionController::default();

        // Place obstacle directly ahead at [0.7, 0.0]
        let obstacles = vec![[0.7, 0.0]];
        let v = ctrl.compute_follower_velocity(&follower, &parent, &obstacles, &[]);

        // Obstacle repulsion pushes back along -x
        let v_no_obs = ctrl.compute_follower_velocity(&follower, &parent, &[], &[]);
        assert!(v[0] < v_no_obs[0]);
    }
}
