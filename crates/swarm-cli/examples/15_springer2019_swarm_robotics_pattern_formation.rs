//! # Example 15: Provable Swarm Robotics Pattern Formation
//!
//! **论文**: *Provable self-organizing pattern formation by a swarm of robots with limited knowledge*  
//! **作者**: Mario Coppola, Jian Guo, Eberhard Gill, Guido C. H. E. de Croon (*Swarm Intelligence*, 2019 / TU Delft)  
//!
//! ## 实验内容
//! 1. **极简约束多智能体自组织构型仿真**:
//!    - 无通信、无全局坐标、无记忆、无领航者；
//!    - 仅依靠 8-邻域 Moore 感知与共同北向；
//!    - 严格满足 $\Pi_{\text{safe}}$ 防碰撞与防局部撕裂断连。
//! 2. **多形态自组织演化**:
//!    - Triangle-4 (4 机器人三角形)
//!    - Square-4 (4 机器人正方形)
//!    - Cross-5 (5 机器人十字星)
//!    - Hexagon-6 (6 机器人正六边形)
//! 3. **启发式优化对比 (Baseline vs ALT1 vs ALT2)**:
//!    - Monte Carlo 统计收敛步数分布，复现论文图 11 与图 12 直方图。

use rand::rngs::StdRng;
use rand::SeedableRng;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::process::Command;
use std::time::Instant;
// =========================================================================
// 💡 模式切换：
// 1. 默认使用 reference 参考答案，确保一键直接出效果和复现论文结果；
// 2. 当你完成了关卡 10 的练习后，可取消注释下方、注释上方，测试自己手写的代码！
// =========================================================================
use swarm_core::reference::swarm_robotics::SwarmWorldReference as SwarmWorld;
use swarm_core::swarm_robotics::{ExecutionMode, Pattern, Policy};

// 练习区模式（完成实战后取消注释）：
// use swarm_core::swarm_robotics::{ExecutionMode, Pattern, Policy, SwarmWorld};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 =================================================================");
    println!("   Swarm Intelligence (2019) Provable Self-Organizing Pattern Formation");
    println!("   极简认知群体机器人可证明自组织构型仿真 (TU Delft)");
    println!("=================================================================\n");

    let out_dir = "data";
    fs::create_dir_all(out_dir)?;
    fs::create_dir_all("output")?;

    let mut rng = StdRng::seed_from_u64(1337);

    // -------------------------------------------------------------------------
    // 实验 1: 跟踪单个完整轨迹演化并导出空间坐标切片
    // -------------------------------------------------------------------------
    println!("📍 [实验 1/2] 跟踪 Triangle-4 与 Square-4 构型自组织完整演化轨迹...");
    let traj_path = format!("{}/swarm_pattern_trajectories.csv", out_dir);
    let traj_file = File::create(&traj_path)?;
    let mut traj_writer = BufWriter::new(traj_file);
    writeln!(traj_writer, "pattern,trial,step,robot_id,x,y,state_type")?;

    let demo_patterns = [Pattern::triangle_4(), Pattern::square_4(), Pattern::hexagon_6()];

    for pat in &demo_patterns {
        let policy = Policy::from_pattern(pat);
        println!("   - 构型: {:<12} | 机器人数量: {} | 期望局部状态种数: {}", pat.name, pat.size(), policy.s_des.len());

        let mut world = SwarmWorld::random_connected(pat.size(), policy.clone(), ExecutionMode::Alt1, &mut rng);
        let max_steps = 5000;

        let mut history: Vec<(usize, Vec<(i32, i32)>, Vec<&'static str>)> = Vec::new();

        // 记录第 0 步
        let initial_types: Vec<&'static str> = (0..world.size())
            .map(|i| {
                let s = world.get_local_state(i);
                if world.policy.is_desired(s) {
                    "desired"
                } else if world.policy.is_active(s) {
                    "active"
                } else {
                    "blocked"
                }
            })
            .collect();
        history.push((0, world.positions.clone(), initial_types));

        while world.step_count < max_steps && !world.is_converged() {
            world.step(&mut rng);
            if world.step_count % 5 == 0 || world.is_converged() {
                let types: Vec<&'static str> = (0..world.size())
                    .map(|i| {
                        let s = world.get_local_state(i);
                        if world.policy.is_desired(s) {
                            "desired"
                        } else if world.policy.is_active(s) {
                            "active"
                        } else {
                            "blocked"
                        }
                    })
                    .collect();
                history.push((world.step_count, world.positions.clone(), types));
            }
        }

        let total_steps = world.step_count;
        let is_ok = world.is_converged();
        println!("     -> 收敛结果: {} (总步数: {})", if is_ok { "成功 ✅" } else { "未达最大步数 ⚠️" }, total_steps);

        // 抽取 4 个代表性阶段切片: Initial, 33%, 66%, Final
        let n_snap = history.len();
        if n_snap > 0 {
            let slice_indices = [
                0,
                n_snap / 3,
                (2 * n_snap) / 3,
                n_snap - 1,
            ];
            for &idx in &slice_indices {
                let (st, ref pos, ref tps) = history[idx];
                for (r_id, (&(rx, ry), &tp)) in pos.iter().zip(tps.iter()).enumerate() {
                    writeln!(traj_writer, "{},1,{},{},{},{},{}", pat.name, st, r_id, rx, ry, tp)?;
                }
            }
        }
    }
    traj_writer.flush()?;
    println!("   📸 演化轨迹切片已写入: {}", traj_path);

    // -------------------------------------------------------------------------
    // 实验 2: Monte Carlo 统计收敛步数直方图 (复现 Fig 11 & Fig 12)
    // -------------------------------------------------------------------------
    println!("\n🎲 [实验 2/2] 运行 Monte Carlo 统计收敛步数分布 (复现 Fig 11 & Fig 12)...");
    let mc_path = format!("{}/swarm_pattern_formation_steps.csv", out_dir);
    let mc_file = File::create(&mc_path)?;
    let mut mc_writer = BufWriter::new(mc_file);
    writeln!(mc_writer, "pattern,mode,trial,steps,converged")?;

    let patterns = [
        Pattern::triangle_4(),
        Pattern::square_4(),
        Pattern::cross_5(),
        Pattern::hexagon_6(),
    ];

    let modes = [
        (ExecutionMode::Baseline, "Baseline"),
        (ExecutionMode::Alt1, "ALT1"),
        (ExecutionMode::Alt2, "ALT2"),
    ];

    let n_trials = 60;
    let max_mc_steps = 8000;
    let start_mc = Instant::now();

    for pat in &patterns {
        let policy = Policy::from_pattern(pat);
        println!("   📊 构型: {:<12} (机器人数: {})", pat.name, pat.size());

        for &(mode, mode_name) in &modes {
            let mut steps_list = Vec::with_capacity(n_trials);
            let mut succ_count = 0;

            for trial in 1..=n_trials {
                let mut world = SwarmWorld::random_connected(pat.size(), policy.clone(), mode, &mut rng);
                let converged = world.run_until_converged(max_mc_steps, &mut rng);
                let st = world.step_count;
                if converged {
                    succ_count += 1;
                    steps_list.push(st);
                }
                writeln!(mc_writer, "{},{},{},{},{}", pat.name, mode_name, trial, st, converged)?;
            }

            if !steps_list.is_empty() {
                steps_list.sort_unstable();
                let mean = steps_list.iter().sum::<usize>() as f64 / steps_list.len() as f64;
                let median = steps_list[steps_list.len() / 2];
                let p25 = steps_list[steps_list.len() / 4];
                let p75 = steps_list[(steps_list.len() * 3) / 4];

                println!(
                    "      [{:<8}] 成功率: {:>3}/{} | 中位数步数: {:>5} | 均值: {:>6.1} (IQR: {} ~ {})",
                    mode_name, succ_count, n_trials, median, mean, p25, p75
                );
            }
        }
    }
    mc_writer.flush()?;

    println!("\n✅ 全部测试完成！总耗时: {:.2} 秒", start_mc.elapsed().as_secs_f64());

    // -------------------------------------------------------------------------
    // 自动调用 Python 绘图
    // -------------------------------------------------------------------------
    println!("\n📊 正在启动 Python 绘制可证明构型自组织科学图表...");
    let py_status = Command::new("uv")
        .args(["run", "python", "python/plot_swarm_pattern_formation.py"])
        .status();

    match py_status {
        Ok(s) if s.success() => {
            println!("🎉 图像生成成功！图表位于 output/ 目录。");
        }
        _ => {
            println!("⚠️ uv 运行失败，尝试系统 python3...");
            let _ = Command::new("python3")
                .arg("python/plot_swarm_pattern_formation.py")
                .status();
        }
    }

    Ok(())
}
