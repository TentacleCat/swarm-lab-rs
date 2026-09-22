//! Paper: Microscopic dynamics of consensus formation in multi-agent LLM Naming Games (arXiv:2608.02178)
//! PDF 本地路径: papers/naming-game/02-llm-naming-game-arxiv2026/paper.pdf
//! 论文链接: https://arxiv.org/abs/2608.02178
//!
//! 一键运行命令:
//!   cargo run --release --example 12_arxiv2026_llm_naming_game

use std::fs::File;
use std::process::Command;
use rand::rngs::StdRng;
use rand::SeedableRng;

use swarm_core::naming_game::llm_model::{LlmArchitecture, LlmNamingGame};
use swarm_core::naming_game::network::CompleteGraph;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!(" 🤖 Multi-Agent LLM Naming Game: Microscopic Dynamics (arXiv:2608.02178)");
    println!(" 📄 Local Paper: papers/naming-game/02-llm-naming-game-arxiv2026/paper.pdf");
    println!("============================================================\n");

    std::fs::create_dir_all("data")?;
    std::fs::create_dir_all("output")?;

    let traj_csv_path = "data/llm_naming_game_trajectories.csv";
    let summary_csv_path = "data/llm_naming_game_summary.csv";

    let mut traj_writer = csv::Writer::from_writer(File::create(traj_csv_path)?);
    let mut sum_writer = csv::Writer::from_writer(File::create(summary_csv_path)?);

    traj_writer.write_record(&[
        "arch",
        "temperature",
        "step",
        "nd",
        "nw",
        "avg_k",
        "drift",
        "tp",
        "fn",
        "fp",
        "tn",
    ])?;

    sum_writer.write_record(&[
        "arch",
        "temperature",
        "consensus_time",
        "pi",
        "phi",
        "order_r",
    ])?;

    let architectures = [
        ("LLaMA-3.1:8B", LlmArchitecture::Llama3_1_8B),
        ("Mistral:7B", LlmArchitecture::Mistral7B),
        ("Phi-3:14B", LlmArchitecture::Phi3_14B),
    ];

    let temperatures = [0.1, 1.0, 2.0];
    let n = 100; // 智能体数量 N=100
    let max_steps = 80_000;
    let sample_interval = 200;

    for (name, arch) in &architectures {
        println!("▶ 模拟架构: {} (N={})", name, n);

        for &temp in &temperatures {
            let mut rng = StdRng::seed_from_u64(42);
            let graph = CompleteGraph::new(n);
            let mut game = LlmNamingGame::new(graph, *arch, temp);

            let r_param = game.ordering_parameter_r();
            println!(
                "   [T={:.1}] pi={:.3}, phi={:.3}, 理论有序度 R={:.3}",
                temp, game.pi, game.phi, r_param
            );

            let mut consensus_step = max_steps;

            for step in 1..=max_steps {
                game.step(&mut rng);

                if step % sample_interval == 0 || step == 1 {
                    let nd = game.distinct_words();
                    let nw = game.total_words();
                    let avg_k = game.average_inventory_size();
                    let drift = game.drift_proxy();

                    traj_writer.write_record(&[
                        *name,
                        &format!("{:.1}", temp),
                        &step.to_string(),
                        &nd.to_string(),
                        &nw.to_string(),
                        &format!("{:.3}", avg_k),
                        &format!("{:.4}", drift),
                        &game.tp_count.to_string(),
                        &game.fn_count.to_string(),
                        &game.fp_count.to_string(),
                        &game.tn_count.to_string(),
                    ])?;
                }

                if game.is_consensus() {
                    consensus_step = step;
                    println!("     ✨ 达成共识时刻 tc = {}", step);
                    break;
                }
            }

            sum_writer.write_record(&[
                *name,
                &format!("{:.1}", temp),
                &consensus_step.to_string(),
                &format!("{:.3}", game.pi),
                &format!("{:.3}", game.phi),
                &format!("{:.3}", r_param),
            ])?;
        }
    }

    traj_writer.flush()?;
    sum_writer.flush()?;

    println!("\n✅ 仿真数据已写入:");
    println!("  - 轨迹演化数据: {}", traj_csv_path);
    println!("  - 实验汇总数据: {}", summary_csv_path);

    // 调用 Python 自动绘制科学作图
    println!("\n🎨 正在调用 Python 生成科学作图 (output/llm_naming_game_*.png)...");
    let py_status = Command::new("uv")
        .args(["run", "python", "python/plot_llm_naming_game.py"])
        .status()
        .or_else(|_| Command::new("python3").arg("python/plot_llm_naming_game.py").status());

    match py_status {
        Ok(st) if st.success() => {
            println!("🎉 作图完成！请查看生成图表:");
            println!("  - 宏观收敛轨迹与逆温度序: output/llm_naming_game_trajectories.png");
            println!("  - (pi, phi) 相图与临界线: output/llm_naming_game_phase_diagram.png");
        }
        _ => {
            println!("💡 可手动运行以下命令出图:\n  uv run python python/plot_llm_naming_game.py");
        }
    }

    Ok(())
}
