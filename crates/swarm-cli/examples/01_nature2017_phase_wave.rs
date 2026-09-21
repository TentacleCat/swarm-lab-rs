//! Example: Reproducing Nature Communications 2017 (2D Swarmalators)
//!
//! Run with:
//!   cargo run --release --example 01_nature2017_phase_wave
//!
//! This will simulate 100 swarmalators in 2D forming a Static Phase Wave,
//! and automatically plot the resulting phase space and order parameter curves.

use std::fs::File;
use std::process::Command;
use rand::rngs::StdRng;
use rand::SeedableRng;
use swarm_core::integrator::Rk4Integrator;
use swarm_core::models::Swarmalator2D;
use swarm_core::types::DynamicalSystem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Reproducing Nature Communications 2017: Static Phase Wave");

    // 1. 设置物理参数 (对应论文 Figure 2)
    let n = 150;        // 粒子数
    let j = 0.5;        // 空间同相吸引强度
    let k = -0.2;       // 异相排斥 / 相位反相耦合
    let dt = 0.05;      // 积分步长
    let steps = 2500;   // 总步数 (t = 125)

    // 2. 初始化动力系统
    let model = Swarmalator2D::new(n, j, k);
    let mut rng = StdRng::seed_from_u64(42);
    let mut state = model.random_initial_state(&mut rng);
    let mut integrator = Rk4Integrator::new(model.dimension());

    std::fs::create_dir_all("data")?;
    std::fs::create_dir_all("output")?;

    let traj_path = "data/nature2017_traj.csv";
    let metric_path = "data/nature2017_metrics.csv";

    let mut traj_writer = csv::Writer::from_writer(File::create(traj_path)?);
    let mut metric_writer = csv::Writer::from_writer(File::create(metric_path)?);

    traj_writer.write_record(&["step", "time", "agent_id", "c1", "c2", "phase"])?;
    metric_writer.write_record(&["step", "time", "order_r", "order_s", "order_s_plus", "order_s_minus"])?;

    // 3. 动力学演化
    println!("⏳ 正在运行数值积分 (RK4)...");
    for step in 0..=steps {
        let t = (step as f64) * dt;

        // 每隔 20 步记录一次数据
        if step % 20 == 0 {
            let snap = model.metrics(&state);
            metric_writer.write_record(&[
                step.to_string(),
                format!("{:.3}", t),
                format!("{:.4}", snap.order_r),
                snap.order_s.map(|v| format!("{:.4}", v)).unwrap_or_default(),
                "".to_string(),
                "".to_string(),
            ])?;

            // 记录粒子坐标 (x, y, theta)
            let x = &state[0..n];
            let y = &state[n..2*n];
            let theta = &state[2*n..3*n];
            for i in 0..n {
                traj_writer.write_record(&[
                    step.to_string(),
                    format!("{:.3}", t),
                    i.to_string(),
                    format!("{:.4}", x[i]),
                    format!("{:.4}", y[i]),
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
            "--model", "2d",
            "--outdir", "output",
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("🎉 绘图成功！已输出高清图片到 output/ 目录：");
            println!("   - output/final_state.png       (2D 空间彩虹相位波)");
            println!("   - output/metrics_evolution.png (序参量收敛曲线)");
        }
        _ => {
            println!("💡 可手动运行以下命令作图：");
            println!("   uv run python/plot_phases.py --traj {} --metrics {} --model 2d", traj_path, metric_path);
        }
    }

    Ok(())
}
