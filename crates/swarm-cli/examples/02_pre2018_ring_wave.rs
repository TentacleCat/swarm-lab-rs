//! Example: Reproducing PRE 2018 (1D Swarmalators on a Ring)
//!
//! Run with:
//!   cargo run --release --example 02_pre2018_ring_wave
//!
//! This will simulate 100 swarmalators on a ring forming a Phase Wave state (S_- -> 1),
//! and automatically plot the resulting phase-position scatter and order parameter curves.

use std::fs::File;
use std::process::Command;
use rand::rngs::StdRng;
use rand::SeedableRng;
use swarm_core::integrator::Rk4Integrator;
use swarm_core::models::SwarmalatorRing1D;
use swarm_core::types::DynamicalSystem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Reproducing PRE 2018: 1D Ring Phase Wave");

    // 1. 设置物理参数
    let n = 100;
    let j = 0.5;
    let k = -0.5;
    let dt = 0.05;
    let steps = 2000;

    // 2. 初始化动力系统
    let model = SwarmalatorRing1D::new(n, j, k);
    let mut rng = StdRng::seed_from_u64(42);
    let mut state = model.random_initial_state(&mut rng);
    let mut integrator = Rk4Integrator::new(model.dimension());

    std::fs::create_dir_all("data")?;
    std::fs::create_dir_all("output")?;

    let traj_path = "data/pre2018_ring_traj.csv";
    let metric_path = "data/pre2018_ring_metrics.csv";

    let mut traj_writer = csv::Writer::from_writer(File::create(traj_path)?);
    let mut metric_writer = csv::Writer::from_writer(File::create(metric_path)?);

    traj_writer.write_record(&["step", "time", "agent_id", "c1", "c2", "phase"])?;
    metric_writer.write_record(&["step", "time", "order_r", "order_s", "order_s_plus", "order_s_minus"])?;

    // 3. 动力学演化
    println!("⏳ 正在运行数值积分 (RK4)...");
    for step in 0..=steps {
        let t = (step as f64) * dt;

        if step % 20 == 0 {
            let snap = model.metrics(&state);
            metric_writer.write_record(&[
                step.to_string(),
                format!("{:.3}", t),
                format!("{:.4}", snap.order_r),
                snap.order_s.map(|v| format!("{:.4}", v)).unwrap_or_default(),
                snap.order_s_plus.map(|v| format!("{:.4}", v)).unwrap_or_default(),
                snap.order_s_minus.map(|v| format!("{:.4}", v)).unwrap_or_default(),
            ])?;

            let phi = &state[0..n];
            let theta = &state[n..2*n];
            for i in 0..n {
                traj_writer.write_record(&[
                    step.to_string(),
                    format!("{:.3}", t),
                    i.to_string(),
                    format!("{:.4}", phi[i]),
                    "".to_string(),
                    format!("{:.4}", theta[i]),
                ])?;
            }
        }

        if step < steps {
            integrator.step(&model, &mut state, dt);
        }
    }

    traj_writer.flush()?;
    metric_writer.flush()?;
    println!("✅ 仿真完成！轨迹保存在 {}", traj_path);

    // 4. 自动调用 Python 绘图
    println!("🎨 正在调用可视化脚本自动作图...");
    let status = Command::new("uv")
        .args([
            "run",
            "python/plot_phases.py",
            "--traj", traj_path,
            "--metrics", metric_path,
            "--model", "ring",
            "--outdir", "output",
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("🎉 绘图成功！已输出高清图片到 output/ 目录：");
            println!("   - output/final_state.png       (1D 圆环位置与相位相关图)");
            println!("   - output/metrics_evolution.png (序参量收敛曲线: S_-, R, S)");
        }
        _ => {
            println!("💡 可手动运行以下命令作图：");
            println!("   uv run python/plot_phases.py --traj {} --metrics {} --model ring", traj_path, metric_path);
        }
    }

    Ok(())
}
