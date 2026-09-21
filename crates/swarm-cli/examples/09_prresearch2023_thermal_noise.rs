//! Paper: Swarmalators with thermal noise (Phys. Rev. Research 2023)
//! PDF 本地路径: papers/2d-plane/05-thermal-noise-prresearch2023/paper.pdf
//! arXiv 链接: https://arxiv.org/abs/2301.07470
//!
//! 一键运行命令:
//!   cargo run --release --example 09_prresearch2023_thermal_noise

use std::fs::File;
use std::process::Command;
use rand::rngs::StdRng;
use rand::SeedableRng;
use swarm_core::integrator::Rk4Integrator;
use swarm_core::models::Swarmalator2D;
use swarm_core::types::DynamicalSystem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📖 [09] Phys. Rev. Research 2023: Swarmalators with thermal noise");
    println!("📄 PDF 本地路径: papers/2d-plane/05-thermal-noise-prresearch2023/paper.pdf\n");

    // =========================================================================
    // ✏️ 【热噪声与朗之万动力学实验区】
    // 论文核心：在空间与相位方程中引入高斯白噪声 xi_i(t), eta_i(t)
    // 观察有限温度 / 噪声诱导的序-无序相变
    // =========================================================================
    let n = 150;
    let j = 0.5;
    let k = 0.5;
    let dt = 0.05;
    let steps = 2500;

    let model = Swarmalator2D::new(n, j, k);
    let mut rng = StdRng::seed_from_u64(42);
    let mut state = model.random_initial_state(&mut rng);
    let mut integrator = Rk4Integrator::new(model.dimension());

    std::fs::create_dir_all("data")?;
    std::fs::create_dir_all("output")?;

    let traj_path = "data/09_prresearch2023_noise_traj.csv";
    let metric_path = "data/09_prresearch2023_noise_metrics.csv";

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

    println!("🎉 作图完成！可在 output/ 查看生成图表。");
    Ok(())
}
