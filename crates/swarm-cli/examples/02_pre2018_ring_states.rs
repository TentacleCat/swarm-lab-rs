//! Paper: Ring states in swarmalator systems (Phys. Rev. E 2018)
//! PDF 本地路径: papers/1d-ring/01-ring-states-pre2018/paper.pdf
//! arXiv 链接: https://arxiv.org/abs/1712.03058
//!
//! 一键运行命令:
//!   cargo run --release --example 02_pre2018_ring_states

use std::fs::File;
use std::process::Command;
use rand::rngs::StdRng;
use rand::SeedableRng;
// =========================================================================
// 💡 模式切换：
// 1. 默认使用你在 swarm_core 中亲手实现的算法
// 2. 若想先看效果或卡壳对比，可取消注释下方的 reference 引用：
// =========================================================================
use swarm_core::integrator::Rk4Integrator;
use swarm_core::models::SwarmalatorRing1D;

// 参考答案引用（需要时取消注释）：
// use swarm_core::reference::integrator::Rk4Integrator;
// use swarm_core::reference::ring_1d::SwarmalatorRing1D;

use swarm_core::types::DynamicalSystem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📖 [02] Phys. Rev. E 2018: Ring states in swarmalator systems");
    println!("📄 PDF 本地路径: papers/1d-ring/01-ring-states-pre2018/paper.pdf\n");

    // =========================================================================
    // ✏️ 【参数设置区域】对照论文参数调参：
    // - 同步相 (Sync State):       J = 0.5, K = 1.0  (S=1, R=1)
    // - 静态相位波 (Phase Wave):   J = 0.5, K = -0.5 (S=0, R=0, S_- -> 1)
    // - 异步相 (Async State):      J = -0.5, K = -0.5
    // =========================================================================
    let n = 120;        // 粒子总数
    let j = 0.5;        // 空间耦合 J
    let k = -0.5;       // 相位耦合 K
    let dt = 0.05;      // 积分步长
    let steps = 2000;   // 仿真总步数

    let model = SwarmalatorRing1D::new(n, j, k);
    let mut rng = StdRng::seed_from_u64(42);
    let mut state = model.random_initial_state(&mut rng);
    let mut integrator = Rk4Integrator::new(model.dimension());

    std::fs::create_dir_all("data")?;
    std::fs::create_dir_all("output")?;

    let traj_path = "data/02_pre2018_ring_traj.csv";
    let metric_path = "data/02_pre2018_ring_metrics.csv";

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

    println!("🎉 作图完成！可在 output/ 查看：");
    println!("   - output/final_state.png       (1D 圆环位置与相位相关散点图)");
    println!("   - output/metrics_evolution.png (序参量收敛曲线: S_-, R, S)");
    Ok(())
}
