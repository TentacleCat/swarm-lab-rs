//! Paper: Mixed sign interactions in the 1D swarmalator model (arXiv 2023)
//! PDF 本地路径: papers/1d-ring/05-mixed-interactions-arxiv2023/paper.pdf
//! arXiv 链接: https://arxiv.org/abs/2309.02342
//!
//! 一键运行命令:
//!   cargo run --release --example 06_arxiv2023_mixed_sign_interactions

use std::fs::File;
use std::process::Command;
use rand::rngs::StdRng;
use rand::SeedableRng;
use swarm_core::integrator::Rk4Integrator;
use swarm_core::models::SwarmalatorRing1D;
use swarm_core::types::DynamicalSystem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📖 [06] arXiv 2023: Mixed sign interactions in the 1D swarmalator model");
    println!("📄 PDF 本地路径: papers/1d-ring/05-mixed-interactions-arxiv2023/paper.pdf\n");

    // =========================================================================
    // ✏️ 【正负混合符号相互作用实验区】
    // 论文核心：系统中同时存在正负耦合（部分个体吸引，部分个体排斥）
    // 导致玻璃化状态 (Glassy states) 与复杂极限环
    // =========================================================================
    let n = 150;
    let j = -0.5;       // 负空间耦合 (排斥占优)
    let k = 0.5;        // 正相位耦合 (同步占优)
    let dt = 0.05;
    let steps = 3000;

    let model = SwarmalatorRing1D::new(n, j, k);
    let mut rng = StdRng::seed_from_u64(42);
    let mut state = model.random_initial_state(&mut rng);
    let mut integrator = Rk4Integrator::new(model.dimension());

    std::fs::create_dir_all("data")?;
    std::fs::create_dir_all("output")?;

    let traj_path = "data/06_arxiv2023_mixed_traj.csv";
    let metric_path = "data/06_arxiv2023_mixed_metrics.csv";

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

    println!("🎨 正在调用可视化作图...");
    let _ = Command::new("uv")
        .args([
            "run", "python/plot_phases.py",
            "--traj", traj_path,
            "--metrics", metric_path,
            "--model", "ring",
            "--outdir", "output",
        ])
        .status();

    println!("🎉 作图完成！可在 output/ 查看生成图表。");
    Ok(())
}
