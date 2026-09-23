//! # 示例 16: Science Robotics 2018 群体机器人图灵形态发生复现
//!
//! 论文: *Morphogenesis in robot swarms* (Science Robotics 2018, 3(25), eaau9178)
//!
//! 本程序模拟:
//! 1. 150 个 Kilobot 在局域红外通信网络上进行虚拟化学图灵反应-扩散 (Activator-Inhibitor)；
//! 2. 局部边缘检测识别外轮廓，未极化边缘机器人沿外周环绕流动 (ORBIT)；
//! 3. 极化斑点捕获流动机器人，自发增生出多指状生物形态突起 (Protrusions / Lobes)；
//! 4. 断肢切除 (Amputation) 后的自组织斑点重构与自愈再生 (Self-healing)。

use rand::rngs::StdRng;
use rand::SeedableRng;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::time::Instant;
// =========================================================================
// 💡 [闯关模式开关]
// 默认使用参考实现 (Reference)，保证开箱即用并可作为对照。
// 当你在 crates/swarm-core/src/turing_morphogenesis/ 中亲手编写完成练习任务后，
// 注释掉下面这行，取消注释下一行，即可切换为你自己的实战代码检验成果！
// =========================================================================
use swarm_core::reference::turing_morphogenesis::{
    BotStateReference as BotState,
    MorphogenesisSwarmReference as MorphogenesisSwarm,
    MorphogenParamsReference as MorphogenParams,
};
// use swarm_core::turing_morphogenesis::{BotState, MorphogenesisSwarm, MorphogenParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("======================================================================");
    println!("🧪 示例 16: Science Robotics 2018 群体机器人图灵形态发生实验");
    println!("   Morphogenesis in robot swarms (DOI: 10.1126/scirobotics.aau9178)");
    println!("======================================================================\n");

    create_dir_all("data")?;
    let snap_path = "data/turing_morphogenesis_snapshots.csv";
    let metric_path = "data/turing_morphogenesis_metrics.csv";

    let mut snap_file = File::create(snap_path)?;
    writeln!(
        snap_file,
        "phase,step,time,id,x,y,u,v,state,color,is_polarized,is_edge"
    )?;

    let mut metric_file = File::create(metric_path)?;
    writeln!(
        metric_file,
        "step,time,total_robots,polarized_count,orbiting_count,following_count,waiting_count,turing_spots_count,gyration_radius,max_extent"
    )?;

    let start_timer = Instant::now();
    let mut rng = StdRng::seed_from_u64(2018);

    // 1. 初始化 150 台 Kilobot 密集团簇 (Pile formation)
    let n_bots = 150;
    let cluster_radius = 100.0; // mm
    let params = MorphogenParams::default();

    let mut swarm = MorphogenesisSwarm::new_pile(
        n_bots,
        [0.0, 0.0],
        cluster_radius,
        params,
        &mut rng,
    );

    println!("📍 初始化完成: N = {} Kilobots, 初始聚集半径 = {:.1} mm", n_bots, cluster_radius);
    println!("   扩散系数: D_u = {:.2}, D_v = {:.2} (D_v/D_u = 20, 经典图灵失稳)", params.d_u, params.d_v);
    println!("   极化阈值 POLAR_TH = {:.1}, 边缘检测比阈值 EDGE_TH = {:.2}\n", params.polar_th, 0.80);

    // 保存初始快照 (Phase 0: Initial)
    save_snapshot(&mut snap_file, "0_Initial", &swarm, params.polar_th)?;
    save_metrics(&mut metric_file, &swarm)?;

    // 阶段 1: 纯图灵反应-扩散阶段 (Symmetry Breaking)
    // 机器人保持静止，仅虚拟形态素在通信网上传播，生成稳定的极化斑点 (Turing Spots)
    println!("🔬 阶段 1: 图灵反应-扩散孕育期 (纯化学反应-扩散，静止态)...");
    let dt_rd = 0.05;
    let pre_diffusion_steps = 350;

    for _ in 0..pre_diffusion_steps {
        swarm.step_diffusion_only(1, dt_rd);
        if swarm.step_count % 50 == 0 {
            save_metrics(&mut metric_file, &swarm)?;
        }
    }

    let m1 = swarm.metrics();
    println!("   ✅ 图灵失稳斑图已建立: 极化机器人数 = {}, 识别出图灵斑点数 = {}\n",
             m1.polarized_count, m1.turing_spots_count);
    save_snapshot(&mut snap_file, "1_TuringSpots", &swarm, params.polar_th)?;

    // 阶段 2: 耦合物理形态发生生长 (Turing-Guided Swarm Morphogenesis)
    // 开启边缘环绕与极化斑点捕获
    println!("🌱 阶段 2: 开启差异性边缘迁移与组织生长 (ORBIT -> 极化中心捕获)...");
    swarm.movement_enabled = true;
    let growth_steps = 600;

    for s in 0..growth_steps {
        swarm.step(0.08);

        if (s + 1) % 15 == 0 {
            save_metrics(&mut metric_file, &swarm)?;
        }

        // 保存生长中期快照
        if s == 200 {
            save_snapshot(&mut snap_file, "2_EarlyGrowth", &swarm, params.polar_th)?;
            println!("   [生长 200 步] 正在流动环绕的机器人数: {}", swarm.metrics().orbiting_count);
        }
    }

    let m2 = swarm.metrics();
    println!("   ✅ 成熟形态发生完成: 回转半径从 {:.1} mm 扩展至 {:.1} mm, 最大展宽 {:.1} mm\n",
             m1.gyration_radius, m2.gyration_radius, m2.max_extent);
    save_snapshot(&mut snap_file, "3_MatureShape", &swarm, params.polar_th)?;

    // 阶段 3: 损伤切除与自愈测试 (Amputation & Self-Healing Experiment)
    println!("✂️ 阶段 3: 施加外源切断损伤 (Amputation: 切除 x > 65.0 mm 突起指状分支)...");
    let removed = swarm.amputate(|bot| bot.pos[0] > 65.0);
    println!("   切除了 {} 台机器人分支, 剩余 {} 台机器人进入自愈阶段", removed, swarm.robots.len());

    save_snapshot(&mut snap_file, "4_Amputated", &swarm, params.polar_th)?;
    save_metrics(&mut metric_file, &swarm)?;

    // 继续演化让系统自发自愈与图灵斑点重新自组织
    let heal_steps = 400;
    for _ in 0..heal_steps {
        swarm.step(0.08);
        if swarm.step_count % 15 == 0 {
            save_metrics(&mut metric_file, &swarm)?;
        }
    }

    let m3 = swarm.metrics();
    println!("   ✅ 自愈与再生完成: 剩余机器人数 = {}, 稳定斑点数 = {}\n",
             m3.total_robots, m3.turing_spots_count);
    save_snapshot(&mut snap_file, "5_Regrown", &swarm, params.polar_th)?;

    println!("======================================================================");
    println!("🎉 仿真圆满完成！耗时: {:.2?} 秒", start_timer.elapsed());
    println!("📁 轨迹与快照数据已写入: {}", snap_path);
    println!("📁 演化时序指标已写入: {}", metric_path);
    println!("👉 接下来可运行: uv run python python/plot_turing_morphogenesis.py 出图");
    println!("======================================================================");

    Ok(())
}

fn save_snapshot(
    file: &mut File,
    phase: &str,
    swarm: &MorphogenesisSwarm,
    polar_th: f64,
) -> std::io::Result<()> {
    for bot in &swarm.robots {
        let state_str = match bot.state {
            BotState::Wait => "WAIT",
            BotState::Orbit => "ORBIT",
            BotState::Follow => "FOLLOW",
        };
        let color_str = match bot.led_color(polar_th) {
            swarm_core::turing_morphogenesis::LedColor::Black => "Black",
            swarm_core::turing_morphogenesis::LedColor::Pink => "Pink",
            swarm_core::turing_morphogenesis::LedColor::Blue => "Blue",
            swarm_core::turing_morphogenesis::LedColor::Cyan => "Cyan",
            swarm_core::turing_morphogenesis::LedColor::Green => "Green",
            swarm_core::turing_morphogenesis::LedColor::White => "White",
            swarm_core::turing_morphogenesis::LedColor::Red => "Red",
        };

        writeln!(
            file,
            "{},{},{:.2},{},{:.2},{:.2},{:.4},{:.4},{},{},{},{}",
            phase,
            swarm.step_count,
            swarm.elapsed_time,
            bot.id,
            bot.pos[0],
            bot.pos[1],
            bot.morphogen.u,
            bot.morphogen.v,
            state_str,
            color_str,
            if bot.is_polarized(polar_th) { 1 } else { 0 },
            if bot.edge_detector.is_edge() { 1 } else { 0 },
        )?;
    }
    Ok(())
}

fn save_metrics(file: &mut File, swarm: &MorphogenesisSwarm) -> std::io::Result<()> {
    let m = swarm.metrics();
    writeln!(
        file,
        "{},{:.2},{},{},{},{},{},{},{:.2},{:.2}",
        m.step,
        m.time,
        m.total_robots,
        m.polarized_count,
        m.orbiting_count,
        m.following_count,
        m.waiting_count,
        m.turing_spots_count,
        m.gyration_radius,
        m.max_extent
    )
}
