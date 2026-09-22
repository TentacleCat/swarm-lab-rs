//! # 机器人动力学与差速运动学 (Thymio Robot Kinematics)
//!
//! 论文: *Emergence of Specialised Collective Behaviors in Evolving Heterogeneous Swarms* (PPSN 2024 / Springer)
//!
//! 物理建模:
//! 1. 采用 Thymio II 差速驱动小车底盘；
//! 2. 轮距 L = 0.085m (8.5cm), 轮半径 R = 0.021m (2.1cm)；
//! 3. 最大线速度 v_max = 0.14 m/s (14 cm/s)；
//! 4. 控制器输出目标线速度比例 v \in [-1, 1] 与角速度比例 w \in [-1, 1]；
//! 5. 差速航向与位移积分、刚体碰撞排斥与竞技场边框反弹。

use crate::heterogeneous_swarms::environment::Environment;
use crate::heterogeneous_swarms::sensors::wrap_to_pi;
use serde::{Deserialize, Serialize};

/// 子群分类标签
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubGroup {
    /// 子群 1: 绿色 (剥削利用型 Exploitative, 偏向在强光区密集聚拢)
    Green = 1,
    /// 子群 2: 红色 (探索协调型 Exploratory, 偏向在弱光区高对齐度集体寻优)
    Red = 2,
}

impl SubGroup {
    pub fn name(&self) -> &'static str {
        match self {
            SubGroup::Green => "Green",
            SubGroup::Red => "Red",
        }
    }
}

/// 机器人物理与运动学状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Robot {
    /// 机器人唯一 ID
    pub id: usize,
    /// 当前位置坐标 [x, y] (m)
    pub position: [f64; 2],
    /// 当前航向角 theta \in [-PI, PI) (rad)
    pub heading: f64,
    /// 当前所分配/切换到的子群身份 (Green 或 Red)
    pub subgroup: SubGroup,
    /// 机器人碰撞包围半径 (m, Thymio 约为 0.08m)
    pub radius: f64,
    /// 最大线速度 (m/s, 论文规定为 0.14 m/s)
    pub max_speed: f64,
    /// 最大角速度 (rad/s, 由 2 * v_max / L 导出约为 3.294 rad/s)
    pub max_angular_speed: f64,
}

impl Robot {
    /// 初始化一台机器人
    pub fn new(id: usize, position: [f64; 2], heading: f64, subgroup: SubGroup) -> Self {
        let max_speed = 0.14;
        let wheel_base = 0.085;
        let max_angular_speed = (2.0 * max_speed) / wheel_base;

        Self {
            id,
            position,
            heading: wrap_to_pi(heading),
            subgroup,
            radius: 0.08,
            max_speed,
            max_angular_speed,
        }
    }

    /// 应用神经网络输出的动作控制量 [v, w] \in [-1.0, 1.0]^2 推进一个微元时间步 dt
    pub fn step_motion(&mut self, action: [f64; 2], dt: f64) {
        let v_norm = action[0].clamp(-1.0, 1.0);
        let w_norm = action[1].clamp(-1.0, 1.0);

        let v = v_norm * self.max_speed;
        let w = w_norm * self.max_angular_speed;

        // 差速运动学更新
        self.heading = wrap_to_pi(self.heading + w * dt);
        self.position[0] += v * self.heading.cos() * dt;
        self.position[1] += v * self.heading.sin() * dt;
    }
}

/// 处理整个群体的物理边界约束与机器人之间的弹性碰撞排斥
pub fn resolve_swarm_collisions(robots: &mut [Robot], env: &Environment) {
    let n = robots.len();

    // 1. 机器人之间的硬核/软核弹性排斥
    for i in 0..n {
        for j in (i + 1)..n {
            let dx = robots[j].position[0] - robots[i].position[0];
            let dy = robots[j].position[1] - robots[i].position[1];
            let dist_sq = dx * dx + dy * dy;
            let min_dist = robots[i].radius + robots[j].radius;

            if dist_sq < min_dist * min_dist && dist_sq > 1e-8 {
                let dist = dist_sq.sqrt();
                let overlap = min_dist - dist;
                let nx = dx / dist;
                let ny = dy / dist;

                // 沿接触法线各自分离一半重叠量
                robots[i].position[0] -= 0.5 * overlap * nx;
                robots[i].position[1] -= 0.5 * overlap * ny;
                robots[j].position[0] += 0.5 * overlap * nx;
                robots[j].position[1] += 0.5 * overlap * ny;
            }
        }
    }

    // 2. 竞技场边界约束与反弹
    for robot in robots.iter_mut() {
        let r = robot.radius;
        if robot.position[0] < r {
            robot.position[0] = r;
            // 碰西墙反弹朝向向东
            if robot.heading.cos() < 0.0 {
                robot.heading = wrap_to_pi(std::f64::consts::PI - robot.heading);
            }
        } else if robot.position[0] > env.width - r {
            robot.position[0] = env.width - r;
            // 碰东墙反弹朝向向西
            if robot.heading.cos() > 0.0 {
                robot.heading = wrap_to_pi(std::f64::consts::PI - robot.heading);
            }
        }

        if robot.position[1] < r {
            robot.position[1] = r;
            // 碰南墙反弹朝向向北
            if robot.heading.sin() < 0.0 {
                robot.heading = wrap_to_pi(-robot.heading);
            }
        } else if robot.position[1] > env.height - r {
            robot.position[1] = env.height - r;
            // 碰北墙反弹朝向向南
            if robot.heading.sin() > 0.0 {
                robot.heading = wrap_to_pi(-robot.heading);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heterogeneous_swarms::environment::ArenaType;

    #[test]
    fn test_robot_motion() {
        let mut bot = Robot::new(0, [15.0, 15.0], 0.0, SubGroup::Green);
        // 全速向前直行 1 秒
        bot.step_motion([1.0, 0.0], 1.0);

        assert!((bot.position[0] - (15.0 + 0.14)).abs() < 1e-4);
        assert!((bot.position[1] - 15.0).abs() < 1e-4);
        assert_eq!(bot.heading, 0.0);
    }

    #[test]
    fn test_collision_resolution() {
        let env = Environment::new(ArenaType::Center);
        let mut bots = vec![
            Robot::new(0, [15.0, 15.0], 0.0, SubGroup::Green),
            Robot::new(1, [15.05, 15.0], 0.0, SubGroup::Red),
        ];

        // 初始两车中心间距仅 0.05m，小于 2 * 0.08 = 0.16m
        resolve_swarm_collisions(&mut bots, &env);

        let dx = bots[1].position[0] - bots[0].position[0];
        let dy = bots[1].position[1] - bots[0].position[1];
        let dist = (dx * dx + dy * dy).sqrt();

        // 排斥后间距应被推开至至少 0.16m
        assert!(dist >= 0.159);
    }
}
