//! Paper: Solvable model of non-identical swarmalators (Phys. Rev. Lett. 2022)
//! PDF 本地路径: papers/1d-ring/03-non-identical-prl2022/paper.pdf
//! arXiv 链接: https://arxiv.org/abs/2207.03920
//!
//! 一键运行命令:
//!   cargo run --release --example 04_prl2022_solvable_non_identical

use std::fs::File;
use std::process::Command;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::{Cauchy, Distribution};
use swarm_core::integrator::Rk4Integrator;
use swarm_core::models::SwarmalatorRing1D;
use swarm_core::types::DynamicalSystem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📖 [04] Phys. Rev. Lett. 2022: Solvable model of non-identical swarmalators");
    println!("📄 PDF 本地路径: papers/1d-ring/03-non-identical-prl2022/paper.pdf\n");

    // =========================================================================
    // ✏️ 【参数与频率分布设置区】
    // 本论文核心创新：引入非全同自然频率 omega_i ~ g(omega) 与漂移速度 nu_i
    // 论文使用柯西分布 (Lorentzian distribution) 实现解析可解性 (Ott-Antonsen Ansatz)
    // =========================================================================
    let n = 200;
    let j = 0.5;
    let k = 1.0;
    let dt = 0.05;
    let steps = 2500;

    let mut rng = StdRng::seed_from_u64(42);

    // 柯西分布生成固有频率与空间漂移:
    let delta = 0.1; // 频率分散度 (半高宽)
    let cauchy = Cauchy::new(0.0, delta).unwrap();
    let omega: Vec<f64> = (0..n).map(|_| cauchy.sample(&mut rng)).collect();
    let nu: Vec<f64> = vec![0.0; n]; // 空间速度设为 0 或同样采样

    let model = SwarmalatorRing1D::new(n, j, k).with_drifts_and_frequencies(nu, omega);
    let mut state = model.random_initial_state(&mut rng);
    let mut integrator = Rk4Integrator::new(model.dimension());

    std::fs::create_dir_all("data")?;
    std::fs::create_dir_all("output")?;

    let traj_path = "data/04_prl2022_non_identical_traj.csv";
    let metric_path = "data/04_prl2022_non_identical_metrics.csv";

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

    println!("🎉 作图完成！可在 output/ 查看非全同频率下的相态破缺与去相干。");
    Ok(())
}
