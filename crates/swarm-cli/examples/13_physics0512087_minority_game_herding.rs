//! Paper: Minority game with local interactions due to the presence of herding behavior (arXiv:physics/0512087)
//! PDF 本地路径: papers/minority-game/01-herding-behavior-physics0512087/paper.pdf
//! 论文链接: https://arxiv.org/abs/physics/0512087
//!
//! 一键运行命令:
//!   cargo run --release --example 13_physics0512087_minority_game_herding

use std::fs::File;
use std::process::Command;
use rand::rngs::StdRng;
use rand::SeedableRng;

// =========================================================================
// 💡 模式切换：
// 1. 默认使用 reference 参考答案，确保一键直接出效果和复现论文结果；
// 2. 当你完成了关卡 8 的练习后，可取消注释下方、注释上方，测试自己手写的代码！
// =========================================================================
use swarm_core::reference::minority_game::{
    alpha_parameter, normalized_volatility, MinorityGameReference as MinorityGame,
};

// 练习区模式（完成实战后取消注释）：
// use swarm_core::minority_game::{alpha_parameter, normalized_volatility, MinorityGame};

use swarm_core::naming_game::network::AdjacencyGraph;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!(" 📈 Minority Game with Herding Behavior (physics/0512087)");
    println!(" 📄 Local Paper: papers/minority-game/01-herding-behavior-physics0512087/paper.pdf");
    println!("============================================================\n");

    std::fs::create_dir_all("data")?;
    std::fs::create_dir_all("output")?;

    let vol_csv_path = "data/minority_game_volatility.csv";
    let ts_csv_path = "data/minority_game_timeseries.csv";

    let mut vol_writer = csv::Writer::from_writer(File::create(vol_csv_path)?);
    let mut ts_writer = csv::Writer::from_writer(File::create(ts_csv_path)?);

    vol_writer.write_record(&["config", "memory", "alpha", "volatility"])?;
    ts_writer.write_record(&["config", "step", "attendance"])?;

    let n = 101; // 智能体总数 N = 101 (奇数)
    let s = 2;   // 策略数 S = 2
    let memories = [2, 3, 4, 5, 6, 7, 8, 9];
    let total_steps = 4000;
    let warmup_steps = 1000;

    let mut rng = StdRng::seed_from_u64(42);

    println!("▶ [1/4] 模拟经典标准少数派博弈 (Standard MG)...");
    for &m in &memories {
        let alpha = alpha_parameter(m, n);
        let mut mg = MinorityGame::<AdjacencyGraph>::new(n, m, s, None, false, &mut rng);

        // 预热期
        let _ = mg.run_simulation(warmup_steps, &mut rng);
        // 采样期
        let samples = mg.run_simulation(total_steps - warmup_steps, &mut rng);
        let vol = normalized_volatility(&samples, n);

        vol_writer.write_record(&[
            "Standard MG",
            &m.to_string(),
            &format!("{:.4}", alpha),
            &format!("{:.4}", vol),
        ])?;

        // 记录 M=5 (临界点附近) 的时间序列
        if m == 5 {
            for (step, &att) in samples.iter().take(300).enumerate() {
                ts_writer.write_record(&["Standard MG (M=5)", &step.to_string(), &att.to_string()])?;
            }
        }
    }

    println!("▶ [2/4] 模拟规则网络从众博弈 (Regular Ring, K=8)...");
    for &m in &memories {
        let alpha = alpha_parameter(m, n);
        let net = AdjacencyGraph::regular_ring(n, 8);
        let mut mg = MinorityGame::new(n, m, s, Some(net), true, &mut rng);

        let _ = mg.run_simulation(warmup_steps, &mut rng);
        let samples = mg.run_simulation(total_steps - warmup_steps, &mut rng);
        let vol = normalized_volatility(&samples, n);

        vol_writer.write_record(&[
            "Regular Ring (K=8)",
            &m.to_string(),
            &format!("{:.4}", alpha),
            &format!("{:.4}", vol),
        ])?;

        if m == 5 {
            for (step, &att) in samples.iter().take(300).enumerate() {
                ts_writer.write_record(&["Regular Ring (K=8)", &step.to_string(), &att.to_string()])?;
            }
        }
    }

    println!("▶ [3/4] 模拟小世界网络从众博弈 (Small World, K=8, p=0.1)...");
    for &m in &memories {
        let alpha = alpha_parameter(m, n);
        let net = AdjacencyGraph::watts_strogatz(n, 8, 0.1, &mut rng);
        let mut mg = MinorityGame::new(n, m, s, Some(net), true, &mut rng);

        let _ = mg.run_simulation(warmup_steps, &mut rng);
        let samples = mg.run_simulation(total_steps - warmup_steps, &mut rng);
        let vol = normalized_volatility(&samples, n);

        vol_writer.write_record(&[
            "Small World (K=8, p=0.1)",
            &m.to_string(),
            &format!("{:.4}", alpha),
            &format!("{:.4}", vol),
        ])?;
    }

    println!("▶ [4/4] 模拟强从众小世界网络 (Small World, K=8, p=0.5)...");
    for &m in &memories {
        let alpha = alpha_parameter(m, n);
        let net = AdjacencyGraph::watts_strogatz(n, 8, 0.5, &mut rng);
        let mut mg = MinorityGame::new(n, m, s, Some(net), true, &mut rng);

        let _ = mg.run_simulation(warmup_steps, &mut rng);
        let samples = mg.run_simulation(total_steps - warmup_steps, &mut rng);
        let vol = normalized_volatility(&samples, n);

        vol_writer.write_record(&[
            "Small World (K=8, p=0.5)",
            &m.to_string(),
            &format!("{:.4}", alpha),
            &format!("{:.4}", vol),
        ])?;

        if m == 5 {
            for (step, &att) in samples.iter().take(300).enumerate() {
                ts_writer.write_record(&["Small World (K=8, p=0.5)", &step.to_string(), &att.to_string()])?;
            }
        }
    }

    vol_writer.flush()?;
    ts_writer.flush()?;

    println!("\n✅ 少数派博弈仿真完成！数据已输出至:");
    println!("  - 波动率相变数据: {}", vol_csv_path);
    println!("  - 时间序列波动数据: {}", ts_csv_path);

    // 调用 Python 自动绘制科学作图
    println!("\n🎨 正在调用 Python 生成科学作图 (output/minority_game_*.png)...");
    let py_status = Command::new("uv")
        .args(["run", "python", "python/plot_minority_game.py"])
        .status()
        .or_else(|_| Command::new("python3").arg("python/plot_minority_game.py").status());

    match py_status {
        Ok(st) if st.success() => {
            println!("🎉 作图完成！请查看生成图表:");
            println!("  - 波动率相变曲线 sigma^2/N vs alpha: output/minority_game_volatility.png");
            println!("  - 市场净动作时间序列对比: output/minority_game_timeseries.png");
        }
        _ => {
            println!("💡 可手动运行以下命令出图:\n  uv run python python/plot_minority_game.py");
        }
    }

    Ok(())
}
