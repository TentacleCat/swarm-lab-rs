//! Paper: Collective behavior of swarmalators on a ring (Phys. Rev. E 2022)
//! PDF 本地路径: papers/1d-ring/02-collective-ring-pre2022/paper.pdf
//! arXiv 链接: https://arxiv.org/abs/2110.13404
//!
//! 一键运行命令:
//!   cargo run --release --example 03_pre2022_collective_ring

use std::fs::File;
use std::process::Command;
use rand::rngs::StdRng;
use rand::SeedableRng;
use swarm_core::integrator::Rk4Integrator;
use swarm_core::models::SwarmalatorRing1D;
use swarm_core::types::DynamicalSystem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📖 [03] Phys. Rev. E 2022: Collective behavior of swarmalators on a ring");
    println!("📄 PDF 本地路径: papers/1d-ring/02-collective-ring-pre2022/paper.pdf\n");

    // =========================================================================
    // ✏️ 【参数设置与公式实验区】对照论文深入探索圆环连续极限分岔：
    // - Ott-Antonsen 理论预测的分岔边界
    // - 尝试调节参数以复现奇美拉相 (Chimera states) 或边界态
    // =========================================================================
    let n = 200;        // 粒子数 (可增加以贴近连续极限 N -> inf)
    let j = 0.6;        // 空间耦合 J
    let k = -0.4;       // 相位耦合 K
    let dt = 0.05;      // 时间步长
    let steps = 3000;   // 步数

    let model = SwarmalatorRing1D::new(n, j, k);
    let mut rng = StdRng::seed_from_u64(42);
    let mut state = model.random_initial_state(&mut rng);
    let mut integrator = Rk4Integrator::new(model.dimension());

    std::fs::create_dir_all("data")?;
    std::fs::create_dir_all("output")?;

    let traj_path = "data/03_pre2022_ring_traj.csv";
    let metric_path = "data/03_pre2022_ring_metrics.csv";

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
