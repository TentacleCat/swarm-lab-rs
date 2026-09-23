//! # Example 19: SoNS Self-organizing Nervous Systems for Robot Swarms
//!
//! Reproduction and study of **Section 4.1 (SoNS Control)** and **Section 4.2 (Analysis Metrics)**
//! from *Self-organizing nervous systems for robot swarms* (arXiv:2401.13103 / Science Robotics 2024).
//!
//! Features demonstrated:
//! 1. Heterogeneous aerial-ground swarm (Drones + Ground robots).
//! 2. Distributed hierarchy establishment, recruitment, and quality-based merging.
//! 3. Section 4.1 Node Allocation with Dynamic Replacement mechanism.
//! 4. Mass-spring-damper kinematic tracking and decentralized obstacle avoidance.
//! 5. Tracking error $E(t)$ vs theoretical lower bound $B(t)$ (Section 4.2).

use csv::Writer;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;
use std::fs::{create_dir_all, File};
use std::path::Path;
// ============================================================================
// 关卡 14: SoNS 自组织神经系统分层网络 (arXiv:2401.13103 / Science Robotics 2024)
//
// 教学模式切换指南：
// 1. 【默认】标准参考答案模式：直接运行体验顶级自组织神经树形成与动态置换模拟
// 2. 【动手实战】学员练习模式：注释掉下方参考实现，取消注释学员实现
// ============================================================================

// --- 模式 1: 使用标准参考实现 (默认启用) ---
use swarm_core::reference::sons_hierarchy::{
    vec2_add, vec2_scale, MorphologySlotReference as MorphologySlot,
    RobotNodeReference as RobotNode, RobotTypeReference as RobotType,
    SoNSSimulatorReference as SoNSSimulator,
};

// --- 模式 2: 使用学员自己的实战代码 (完成 crates/swarm-core/src/sons_hierarchy/ 中的任务后启用) ---
// use swarm_core::sons_hierarchy::{
//     vec2_add, vec2_scale, MorphologySlot, RobotNode, RobotType, SoNSSimulator,
// };

#[derive(Debug, Serialize)]
struct RecordRow {
    step: usize,
    time: f64,
    tracking_error: f64,
    theoretical_lower_bound: f64,
    max_depth: usize,
    num_swarms: usize,
    max_swarm_size: usize,
    robot_id: usize,
    robot_type: String,
    pos_x: f64,
    pos_y: f64,
    vel_x: f64,
    vel_y: f64,
    parent_id: i32,
    brain_id: usize,
}

fn build_hierarchical_morphology() -> Vec<MorphologySlot> {
    // Top-level slots under Brain Drone 0:
    // Slot 1: Sub-leader Drone (Left Wing) at [-1.2, 0.5]
    // Slot 2: Sub-leader Drone (Right Wing) at [1.2, 0.5]
    // Slot 3..6: Ground robots protecting the Brain center
    vec![
        MorphologySlot {
            slot_id: 1,
            relative_offset: [-1.0, 0.6],
            relative_yaw: 0.0,
            expected_type: RobotType::Drone,
            downstream_slots: vec![],
        },
        MorphologySlot {
            slot_id: 2,
            relative_offset: [1.0, 0.6],
            relative_yaw: 0.0,
            expected_type: RobotType::Drone,
            downstream_slots: vec![],
        },
        MorphologySlot {
            slot_id: 3,
            relative_offset: [0.0, 0.7],
            relative_yaw: 0.0,
            expected_type: RobotType::Ground,
            downstream_slots: vec![],
        },
        MorphologySlot {
            slot_id: 4,
            relative_offset: [0.0, -0.7],
            relative_yaw: 0.0,
            expected_type: RobotType::Ground,
            downstream_slots: vec![],
        },
        MorphologySlot {
            slot_id: 5,
            relative_offset: [-0.6, 0.0],
            relative_yaw: 0.0,
            expected_type: RobotType::Ground,
            downstream_slots: vec![],
        },
        MorphologySlot {
            slot_id: 6,
            relative_offset: [0.6, 0.0],
            relative_yaw: 0.0,
            expected_type: RobotType::Ground,
            downstream_slots: vec![],
        },
        MorphologySlot {
            slot_id: 7,
            relative_offset: [-1.4, 0.2],
            relative_yaw: 0.0,
            expected_type: RobotType::Ground,
            downstream_slots: vec![],
        },
        MorphologySlot {
            slot_id: 8,
            relative_offset: [-1.4, 1.0],
            relative_yaw: 0.0,
            expected_type: RobotType::Ground,
            downstream_slots: vec![],
        },
        MorphologySlot {
            slot_id: 9,
            relative_offset: [1.4, 0.2],
            relative_yaw: 0.0,
            expected_type: RobotType::Ground,
            downstream_slots: vec![],
        },
        MorphologySlot {
            slot_id: 10,
            relative_offset: [1.4, 1.0],
            relative_yaw: 0.0,
            expected_type: RobotType::Ground,
            downstream_slots: vec![],
        },
    ]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("==========================================================================");
    println!("  SoNS: Self-organizing Nervous Systems for Robot Swarms (arXiv:2401.13103) ");
    println!("  Section 4.1 SoNS Control & Section 4.2 Analysis Metrics Reproduction Lab  ");
    println!("==========================================================================");

    let mut rng = StdRng::seed_from_u64(42);

    // 1. Initialize heterogeneous swarm: 3 Drones + 8 Ground robots = 11 robots
    let mut robots = Vec::new();

    // Robot 0: Designated Brain Drone (starts near origin)
    robots.push(RobotNode::new(0, RobotType::Drone, [0.0, 0.0], 1.0));

    // Robots 1..2: Sub-leader Drones scattered
    robots.push(RobotNode::new(
        1,
        RobotType::Drone,
        [-1.3 + rng.gen_range(-0.2..0.2), 0.4 + rng.gen_range(-0.2..0.2)],
        0.8,
    ));
    robots.push(RobotNode::new(
        2,
        RobotType::Drone,
        [1.3 + rng.gen_range(-0.2..0.2), 0.4 + rng.gen_range(-0.2..0.2)],
        0.8,
    ));

    // Robots 3..10: Ground robots (Pi-pucks) scattered around arena
    for id in 3..=10 {
        let angle = (id as f64 - 3.0) * (std::f64::consts::TAU / 8.0) + rng.gen_range(-0.1..0.1);
        let dist = 1.2 + rng.gen_range(-0.3..0.4);
        let pos = [dist * angle.cos(), dist * angle.sin()];
        robots.push(RobotNode::new(id, RobotType::Ground, pos, 0.5));
    }

    // 2. Define obstacles in the arena
    let obstacles = vec![[0.4, 0.9], [-0.5, -0.8]];

    // 3. Define target morphology
    let target_morphology = build_hierarchical_morphology();

    let dt = 0.05; // 50 ms timestep
    let total_steps = 300;

    let mut sim = SoNSSimulator::new(robots, obstacles, target_morphology, dt);

    // Setup CSV logger
    create_dir_all("data")?;
    let csv_path = Path::new("data/sons_self_organizing_hierarchy.csv");
    let file = File::create(csv_path)?;
    let mut writer = Writer::from_writer(file);

    println!(
        "Swarm initialized: {} robots ({} Drones, {} Ground robots), {} obstacles.",
        sim.robots.len(),
        sim.robots.iter().filter(|r| r.robot_type == RobotType::Drone).count(),
        sim.robots.iter().filter(|r| r.robot_type == RobotType::Ground).count(),
        sim.obstacles.len()
    );
    println!("Target morphology slots: {}", sim.target_morphology.len());
    println!("Beginning simulation (total {} steps, dt = {}s)...", total_steps, dt);

    for step in 0..total_steps {
        // Slow constant drift of the Brain to demonstrate dynamic formation following
        if step > 80 {
            sim.robots[0].position = vec2_add(sim.robots[0].position, vec2_scale([0.05, 0.0], dt));
        }

        let metrics = sim.step();

        // Log each robot state
        for r in &sim.robots {
            let row = RecordRow {
                step,
                time: metrics.time,
                tracking_error: metrics.tracking_error,
                theoretical_lower_bound: metrics.theoretical_lower_bound,
                max_depth: metrics.max_depth,
                num_swarms: metrics.num_swarms,
                max_swarm_size: metrics.max_swarm_size,
                robot_id: r.id,
                robot_type: match r.robot_type {
                    RobotType::Drone => "Drone".to_string(),
                    RobotType::Ground => "Ground".to_string(),
                },
                pos_x: r.position[0],
                pos_y: r.position[1],
                vel_x: r.velocity[0],
                vel_y: r.velocity[1],
                parent_id: r.parent_id.map(|p| p as i32).unwrap_or(-1),
                brain_id: r.brain_id,
            };
            writer.serialize(row)?;
        }

        if step % 50 == 0 || step == total_steps - 1 {
            println!(
                "Step {:3} (t={:5.2}s) | Error E: {:6.4}m | Bound B: {:6.4}m | Swarms: {:2} | Max Tree Scale: {:2} | Depth: {}",
                step,
                metrics.time,
                metrics.tracking_error,
                metrics.theoretical_lower_bound,
                metrics.num_swarms,
                metrics.max_swarm_size,
                metrics.max_depth,
            );
        }
    }

    writer.flush()?;
    println!("Simulation successfully completed!");
    println!("Trajectory data saved to: {}", csv_path.display());
    println!("Run python visualization: uv run python python/plot_sons_self_organizing_hierarchy.py");

    Ok(())
}
