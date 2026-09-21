//! Paper: Swarmalators on a ring with distributed couplings (Phys. Rev. E 2022)
//! PDF 本地路径: papers/1d-ring/04-distributed-couplings-pre2022/paper.pdf
//! arXiv 链接: https://arxiv.org/abs/2203.11183
//!
//! 一键运行命令:
//!   cargo run --release --example 05_pre2022_distributed_couplings

use std::fs::File;
use std::process::Command;
use rand::rngs::StdRng;
use rand::SeedableRng;
use swarm_core::integrator::Rk4Integrator;
use swarm_core::models::SwarmalatorRing1D;
use swarm_core::types::DynamicalSystem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📖 [05] Phys. Rev. E 2022: Swarmalators on a ring with distributed couplings");
    println!("📄 PDF 本地路径: papers/1d-ring/04-distributed-couplings-pre2022/paper.pdf\n");

    // =========================================================================
    // ✏️ 【参数与分布设置区】
    // 本文核心：耦合强度 J_i, K_i 不再是常数，而是服从概率分布 P(J), P(K)
    // 产生失挫 (Frustration) 与多稳态竞争
    // =========================================================================
    let n = 150;
    let j_mean = 0.5;
    let k_mean = -0.3;
    let dt = 0.05;
    let steps = 2500;

    let model = SwarmalatorRing1D::new(n, j_mean, k_mean);
    let mut rng = StdRng::seed_from_u64(42);
    let mut state = model.random_initial_state(&mut rng);
    let mut integrator = Rk4Integrator::new(model.dimension());

    std::fs::create_dir_all("data")?;
    std::fs::create_dir_all("output")?;

    let traj_path = "data/05_pre2022_distributed_traj.csv";
    let metric_path = "data/05_pre2022_distributed_metrics.csv";

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
