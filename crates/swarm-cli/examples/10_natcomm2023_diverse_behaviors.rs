//! Paper: Diverse behaviors in swarmalator systems (Nature Communications 2023)
//! PDF 本地路径: papers/2d-plane/06-diverse-behaviors-natcomm2023/paper.pdf
//! 论文链接: https://www.nature.com/articles/s41467-023-36563-4
//!
//! 一键运行命令:
//!   cargo run --release --example 10_natcomm2023_diverse_behaviors

use std::fs::File;
use std::process::Command;
use rand::rngs::StdRng;
use rand::SeedableRng;
use swarm_core::integrator::Rk4Integrator;
use swarm_core::models::Swarmalator2D;
use swarm_core::types::DynamicalSystem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📖 [10] Nature Communications 2023: Diverse behaviors in swarmalator systems");
    println!("📄 PDF 本地路径: papers/2d-plane/06-diverse-behaviors-natcomm2023/paper.pdf\n");

    // =========================================================================
    // ✏️ 【丰富相态探索区】
    // 论文突破：系统性拓宽耦合函数形式（引入高阶谐波、各向异性相互作用等）
    // 观察更多样态的聚集与非平衡定态
    // =========================================================================
    let n = 150;
    let j = 0.8;
    let k = -0.3;
    let dt = 0.05;
    let steps = 2500;

    let model = Swarmalator2D::new(n, j, k);
    let mut rng = StdRng::seed_from_u64(42);
    let mut state = model.random_initial_state(&mut rng);
    let mut integrator = Rk4Integrator::new(model.dimension());

    std::fs::create_dir_all("data")?;
    std::fs::create_dir_all("output")?;

    let traj_path = "data/10_natcomm2023_diverse_traj.csv";
    let metric_path = "data/10_natcomm2023_diverse_metrics.csv";

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
