//! Paper: Oscillators that sync and swarm (Nature Communications 2017)
//! PDF 本地路径: papers/2d-plane/01-sync-and-swarm-natcomm2017/paper.pdf
//! 论文链接: https://www.nature.com/articles/s41467-017-01190-3
//!
//! 一键运行命令:
//!   cargo run --release --example 01_natcomm2017_oscillators_sync_and_swarm

use std::fs::File;
use std::process::Command;
use rand::rngs::StdRng;
use rand::SeedableRng;
use swarm_core::integrator::Rk4Integrator;
use swarm_core::models::Swarmalator2D;
use swarm_core::types::DynamicalSystem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📖 [01] Nature Communications 2017: Oscillators that sync and swarm");
    println!("📄 PDF 本地路径: papers/2d-plane/01-sync-and-swarm-natcomm2017/paper.pdf\n");

    // =========================================================================
    // ✏️ 【参数设置区域】对照论文 Figure 2 调参：
    // - 静态同步态 (Static Sync):        J = 0.1,  K = 1.0
    // - 静态异步态 (Static Async):       J = -0.1, K = -1.0
    // - 静态相位波 (Static Phase Wave):  J = 0.5,  K = -0.2 (当前默认)
    // - 主动相位波 (Active Phase Wave):  J = -0.1, K = 0.1
    // - 碎裂相位波 (Splintered Wave):    J = 1.0,  K = 0.0
    // =========================================================================
    let n = 150;        // 粒子总数
    let j = 0.5;        // 空间同相吸引强度 J
    let k = -0.2;       // 相位耦合强度 K
    let dt = 0.05;      // 时间步长
    let steps = 2500;   // 仿真总步数

    // 初始化模型
    let model = Swarmalator2D::new(n, j, k);
    let mut rng = StdRng::seed_from_u64(42);
    let mut state = model.random_initial_state(&mut rng);
    let mut integrator = Rk4Integrator::new(model.dimension());

    std::fs::create_dir_all("data")?;
    std::fs::create_dir_all("output")?;

    let traj_path = "data/01_natcomm2017_traj.csv";
    let metric_path = "data/01_natcomm2017_metrics.csv";

    let mut traj_writer = csv::Writer::from_writer(File::create(traj_path)?);
    let mut metric_writer = csv::Writer::from_writer(File::create(metric_path)?);

    traj_writer.write_record(&["step", "time", "agent_id", "c1", "c2", "phase"])?;
    metric_writer.write_record(&["step", "time", "order_r", "order_s", "order_s_plus", "order_s_minus"])?;

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
                "".to_string(),
                "".to_string(),
            ])?;

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
    println!("✅ 仿真完成！轨迹已保存至 {}", traj_path);

    // 自动作图
    println!("🎨 正在调用可视化作图...");
    let _ = Command::new("uv")
        .args([
            "run", "python/plot_phases.py",
            "--traj", traj_path,
            "--metrics", metric_path,
            "--model", "2d",
            "--outdir", "output",
        ])
        .status();

    println!("🎉 作图完成！可在 output/ 查看：");
    println!("   - output/final_state.png       (2D 空间彩虹相态)");
    println!("   - output/metrics_evolution.png (序参量 R(t) 演化)");
    Ok(())
}
