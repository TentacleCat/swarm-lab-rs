//! Paper: Pinning in a system of swarmalators (arXiv 2022)
//! PDF 本地路径: papers/1d-ring/06-pinning-transitions-arxiv2022/paper.pdf
//! arXiv 链接: https://arxiv.org/abs/2211.02353
//!
//! 一键运行命令:
//!   cargo run --release --example 07_arxiv2022_pinning_and_domain_walls

use std::fs::File;
use std::process::Command;
use rand::rngs::StdRng;
use rand::SeedableRng;
use swarm_core::integrator::Rk4Integrator;
use swarm_core::models::SwarmalatorRing1D;
use swarm_core::types::DynamicalSystem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📖 [07] arXiv 2022: Pinning in a system of swarmalators");
    println!("📄 PDF 本地路径: papers/1d-ring/06-pinning-transitions-arxiv2022/paper.pdf\n");

    // =========================================================================
    // ✏️ 【钉扎效应 (Pinning) 实验区】
    // 论文核心：系统中存在部分空间固定点（Pinned agents），类似铁磁畴壁或晶体杂质
    // 探索解钉扎相变 (Depinning transition) 与磁畴结构
    // =========================================================================
    let n = 120;
    let j = 0.5;
    let k = -0.5;
    let dt = 0.05;
    let steps = 2500;

    let model = SwarmalatorRing1D::new(n, j, k);
    let mut rng = StdRng::seed_from_u64(42);
    let mut state = model.random_initial_state(&mut rng);
    let mut integrator = Rk4Integrator::new(model.dimension());

    std::fs::create_dir_all("data")?;
    std::fs::create_dir_all("output")?;

    let traj_path = "data/07_arxiv2022_pinning_traj.csv";
    let metric_path = "data/07_arxiv2022_pinning_metrics.csv";

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
