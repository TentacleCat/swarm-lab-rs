//! # Example 18: Morphological Swarms - Aggregating Swarms via Design Contingencies
//!
//! **论文**: *Aggregating swarms through morphology handling design contingencies: from the sweet spot to a rich expressivity*  
//! **作者**: Jeremy Fersula, Nicolas Bredeche, Olivier Dauchot (*arXiv:2601.07610*, Jan 2026 / Sorbonne & ESPCI Paris)  
//! **代码来源**: `https://github.com/jeremyaqp/AggregatingSwarms2026`  
//!
//! ## 实验内容与物理机制
//! 1. **形态计算（Morphological Computation）**:
//!    - 机器人（Kilobot）内部控制器极其朴素：仅在进入光照区时通过 1:15 间歇启闭降低平均速度（$v_\circ / v_\bullet = 1/3$），**不允许停步**；
//!    - 聚集并非源于复杂的智能寻路或传感器融合，而是由 **3D 打印外骨骼的非对称接触力学**（反向对齐力矩）在物理接触时被动计算出转动；
//! 2. **自对齐动力学与四种典型相态**:
//!    - $\kappa = \epsilon / \tau_n < -2.5$ (**Fronter 冻结堵塞态 / Jamming**): 极强逆对齐导致在黑暗区遭遇即锁死成两两微团簇，产生反向趋光（$N_\circ/N < 0.18$）；
//!    - $\kappa \in [-2.0, -0.6]$ (**Fronter 最佳甜点区 / Sweet Spot**): 碰撞减速与光照区慢行协同，触发动力学诱导相分离（MIPS），在光区形成大团簇（$N_\circ/N \approx 0.35 \sim 0.40$）；
//!    - $\kappa \approx 0.0$ (**主动布朗粒子 / ABP Baseline**): 无外力形态力矩，全空间随机均匀扩散（$N_\circ/N \approx 0.18$）；
//!    - $\kappa > 0.0$ (**Aligner 顺应极化游弋态 / Flocking**): 碰撞同向对齐，形成宏观整齐游荡流（$\langle \Psi \rangle > 0.8$），但完全无法聚集在光照区。
//! 3. **输出与出图**:
//!    - 导出相图扫描数据 `data/morphological_swarms_phase_scan.csv`
//!    - 导出 4 类典型状态轨迹与快照 `data/morphological_swarms_trajectories.csv` 与 `data/morphological_swarms_snapshots.csv`
//!    - 自动触发 Python 绘制高分辨率对比科研图表。

use indicatif::{ProgressBar, ProgressStyle};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::process::Command;
use std::time::Instant;
use swarm_core::morphological_swarms::{MorphologicalSwarm, SwarmParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 =================================================================");
    println!("   arXiv:2601.07610 Aggregating Swarms Through Morphology");
    println!("   形态计算、自对齐力矩与聚集相态表达力仿真实验 (Sorbonne / ESPCI 2026)");
    println!("=================================================================\n");

    let out_dir = "data";
    fs::create_dir_all(out_dir)?;
    fs::create_dir_all("output")?;

    let base_seed = 20260112;

    // -------------------------------------------------------------------------
    // 实验 1: 扫描无量纲形态力矩参数 kappa = epsilon / tau_n in [-5.0, 5.0]
    // -------------------------------------------------------------------------
    println!("📊 [实验 1/2] 扫描形态力矩参数 kappa 空间，构建宏观相变与光照聚集相图...");
    let scan_path = format!("{}/morphological_swarms_phase_scan.csv", out_dir);
    let scan_file = File::create(&scan_path)?;
    let mut scan_writer = BufWriter::new(scan_file);
    writeln!(
        scan_writer,
        "kappa,trial,light_ratio,polar_alignment,contact_pairs,max_cluster,mean_speed,regime"
    )?;

    // 选取 21 个具有物理代表性的采样点
    let kappas: Vec<f64> = (-10..=10).map(|i| (i as f64) * 0.5).collect();
    let n_trials_per_kappa = 3;
    let scan_steps = 150_000; // 对应 150 tau (dt=0.001)
    let sample_window_steps = 30_000; // 采集最后 30 tau 的时间平均

    let pb = ProgressBar::new((kappas.len() * n_trials_per_kappa) as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
            .progress_chars("#>-"),
    );

    let start_scan = Instant::now();

    for &kappa in &kappas {
        let regime_name = classify_regime(kappa);
        for trial in 0..n_trials_per_kappa {
            pb.set_message(format!("kappa={:>4.1} [{}] run={}", kappa, regime_name, trial));

            let mut params = SwarmParams::default();
            params.epsilon_over_tau_n = kappa;
            let mut rng = StdRng::seed_from_u64(base_seed + (trial as u64) * 997 + (kappa * 10.0) as u64);

            let mut swarm = MorphologicalSwarm::new_random(params, &mut rng);

            // 预热进入准稳态
            let warmup_steps = scan_steps - sample_window_steps;
            swarm.step_n(warmup_steps, &mut rng);

            // 在后段采样时间平均
            let sample_interval = 500;
            let n_samples = sample_window_steps / sample_interval;
            let mut sum_light = 0.0;
            let mut sum_align = 0.0;
            let mut sum_contacts = 0.0;
            let mut sum_max_cluster = 0.0;
            let mut sum_speed = 0.0;

            for _ in 0..n_samples {
                swarm.step_n(sample_interval, &mut rng);
                let m = swarm.evaluate_metrics();
                sum_light += m.light_ratio;
                sum_align += m.polar_alignment;
                sum_contacts += m.contact_pairs as f64;
                sum_max_cluster += m.max_cluster_size as f64;
                sum_speed += m.mean_speed;
            }

            let avg_light = sum_light / (n_samples as f64);
            let avg_align = sum_align / (n_samples as f64);
            let avg_contacts = sum_contacts / (n_samples as f64);
            let avg_max_cluster = sum_max_cluster / (n_samples as f64);
            let avg_speed = sum_speed / (n_samples as f64);

            writeln!(
                scan_writer,
                "{:.3},{},{:.5},{:.5},{:.2},{:.2},{:.5},{}",
                kappa,
                trial,
                avg_light,
                avg_align,
                avg_contacts,
                avg_max_cluster,
                avg_speed,
                regime_name
            )?;

            pb.inc(1);
        }
    }
    pb.finish_with_message("相图扫描完成！");
    scan_writer.flush()?;
    drop(scan_writer);
    println!("   - 相图扫描耗时: {:.2}s，数据已保存至: {}", start_scan.elapsed().as_secs_f64(), scan_path);

    // -------------------------------------------------------------------------
    // 实验 2: 提取 4 种典型相态的细致轨迹与空间快照
    // -------------------------------------------------------------------------
    println!("\n📍 [实验 2/2] 跟踪 4 种典型形态的粒子完整时空轨迹与空间瞬态快照...");
    let traj_path = format!("{}/morphological_swarms_trajectories.csv", out_dir);
    let snap_path = format!("{}/morphological_swarms_snapshots.csv", out_dir);

    let traj_file = File::create(&traj_path)?;
    let mut traj_writer = BufWriter::new(traj_file);
    writeln!(traj_writer, "case_name,kappa,step,time,robot_id,x,y,in_light,speed")?;

    let snap_file = File::create(&snap_path)?;
    let mut snap_writer = BufWriter::new(snap_file);
    writeln!(snap_writer, "case_name,kappa,robot_id,x,y,mu_x,mu_y,in_light,speed")?;

    let cases = [
        ("Fronter_Jammed", -3.5, "逆对齐死锁 / 黑暗区团簇冷冻 (反向趋光)"),
        ("Fronter_SweetSpot", -1.2, "最佳形态甜点区 / MIPS 光照成核聚集"),
        ("Active_Brownian", 0.0, "无力矩 ABP / 空间随机均匀扩散基线"),
        ("Aligner_Flocking", 2.5, "顺应同向对齐 / 宏观极化群体游荡"),
    ];

    let traj_total_steps = 100_000; // 100 tau
    let traj_record_interval = 500; // 每 0.5 tau 记录一次轨迹

    for (name, kappa, desc) in &cases {
        println!("   - 模拟典型状态: {:<18} (kappa = {:>4.1}) | 特征: {}", name, kappa, desc);

        let mut params = SwarmParams::default();
        params.epsilon_over_tau_n = *kappa;
        let mut rng = StdRng::seed_from_u64(base_seed + 42);

        let mut swarm = MorphologicalSwarm::new_random(params, &mut rng);

        for step in 0..=traj_total_steps {
            if step % traj_record_interval == 0 {
                let t = swarm.current_time;
                for (id, p) in swarm.particles.iter().enumerate() {
                    writeln!(
                        traj_writer,
                        "{},{:.1},{},{:.3},{},{:.4},{:.4},{},{:.4}",
                        name,
                        kappa,
                        step,
                        t,
                        id,
                        p.pos[0],
                        p.pos[1],
                        if p.in_light { 1 } else { 0 },
                        p.speed()
                    )?;
                }
            }

            swarm.step(&mut rng);
        }

        // 保存最终时刻瞬态快照
        for (id, p) in swarm.particles.iter().enumerate() {
            writeln!(
                snap_writer,
                "{},{:.1},{},{:.4},{:.4},{:.4},{:.4},{},{:.4}",
                name,
                kappa,
                id,
                p.pos[0],
                p.pos[1],
                p.mu[0],
                p.mu[1],
                if p.in_light { 1 } else { 0 },
                p.speed()
            )?;
        }

        let final_m = swarm.evaluate_metrics();
        println!(
            "     -> 终态光照区占比 N_circ/N: {:.1}% (基准 18.0%), 极化度 <Psi>: {:.3}, 最大团簇: {}",
            final_m.light_ratio * 100.0,
            final_m.polar_alignment,
            final_m.max_cluster_size
        );
    }

    traj_writer.flush()?;
    drop(traj_writer);
    snap_writer.flush()?;
    drop(snap_writer);

    println!("\n✅ 仿真数据生成完毕！");
    println!("   - 相图数据: {}", scan_path);
    println!("   - 轨迹数据: {}", traj_path);
    println!("   - 空间快照: {}", snap_path);

    // -------------------------------------------------------------------------
    // 触发 Python 科学出图
    // -------------------------------------------------------------------------
    println!("\n📈 正在调用 Python 生成高精度论文复现图 (Figure 3 & 相态空间构型)...");
    let py_status = Command::new("uv")
        .args(["run", "python", "python/plot_morphological_aggregating_swarms.py"])
        .status();

    match py_status {
        Ok(status) if status.success() => {
            println!("🎉 出图成功！图像已保存在 output/ 目录：");
            println!("   - output/morphological_swarms_phase_diagram.png (复现论文 Figure 3 核心相图与甜点区)");
            println!("   - output/morphological_swarms_spatial_snapshots.png (4 类典型形态空间瞬态与运动流)");
        }
        Ok(status) => {
            eprintln!("⚠️ Python 绘图脚本返回非零退出码: {}", status);
            eprintln!("   可稍后手动执行: uv run python python/plot_morphological_aggregating_swarms.py");
        }
        Err(e) => {
            eprintln!("⚠️ 未能直接唤起 uv 命令 ({})，可手动运行 python 脚本绘图。", e);
        }
    }

    Ok(())
}

fn classify_regime(kappa: f64) -> &'static str {
    if kappa < -2.2 {
        "Jammed"
    } else if kappa <= -0.5 {
        "SweetSpot"
    } else if kappa < 0.5 {
        "ActiveBrownian"
    } else {
        "Flocking"
    }
}
