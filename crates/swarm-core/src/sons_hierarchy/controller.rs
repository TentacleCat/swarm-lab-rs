#![allow(unused_variables, dead_code, unused_imports)]

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

    /// 【关卡 14 - 任务 2】计算从属节点的质量-弹簧-阻尼运动学速度指令 (Section 4.1 Collective Actuation)
    ///
    /// 控制算法分步：
    /// 1. 目标位姿计算：根据 parent 姿态将 `target_relative_offset` 旋转加平移，计算误差向量 $\vec{e} = \mathbf{p}_{\text{target}} - \mathbf{p}$ 与距离 $d = \|\vec{e}\|$；
    /// 2. 弹簧-阻尼运动学跟踪速度 $v_{\text{track}}$：
    ///    - $d < r_{\text{stop}}$ (死区，如 2cm): 停止速度 0.0；
    ///    - $r_{\text{stop}} \le d < r_{\text{slow}}$ (线性减速区，如 20cm): $v_{\max} \cdot (d / r_{\text{slow}}) \cdot \hat{e}$；
    ///    - $d \ge r_{\text{slow}}$: 全速 $v_{\max} \cdot \hat{e}$；
    /// 3. 去中心化障碍物势场排斥速度 $v_{\text{obs}}$；
    /// 4. 机器人个体间避碰排斥速度 $v_{\text{repel}}$；
    /// 5. 速度限幅与安全区视距约束 ($r_{\text{safe}}$ 视距保底)：若下个时间步超出 safezone 则消除径向远离速度分量。
    ///
    /// # 提示
    /// - 若卡壳可参考 [`crates/swarm-core/src/reference/sons_hierarchy.rs`](../reference/sons_hierarchy.rs)。
    pub fn compute_follower_velocity(
        &self,
        robot: &RobotNode,
        parent: &RobotNode,
        obstacles: &[Vec2],
        nearby_robots: &[Vec2],
    ) -> Vec2 {
        todo!("【关卡 14 - 任务 2】在 controller.rs 中实现质量-弹簧-阻尼运动学速度计算 compute_follower_velocity");
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
