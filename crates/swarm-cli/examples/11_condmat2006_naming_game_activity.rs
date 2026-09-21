//! Paper: Microscopic activity patterns in the Naming Game (J. Phys. A / cond-mat/0606125)
//! PDF 本地路径: papers/naming-game/01-microscopic-activity-condmat2006/paper.pdf
//! 论文链接: https://arxiv.org/abs/cond-mat/0606125
//!
//! 一键运行命令:
//!   cargo run --release --example 11_condmat2006_naming_game_activity

use std::fs::File;
use std::process::Command;
use rand::rngs::StdRng;
use rand::SeedableRng;

// =========================================================================
// 💡 模式切换：
// 1. 默认使用 reference 参考答案，确保一键直接出效果和复现论文结果；
// 2. 当你完成了关卡 6 的练习后，可取消注释上方、注释下方，测试自己敲的代码！
// =========================================================================
use swarm_core::reference::naming_game::{
    distinct_words_ref as distinct_words,
    inventory_size_distribution_ref as inventory_size_distribution,
    is_consensus_ref as is_consensus,
    total_words_ref as total_words,
    NamingGameReference as NamingGame,
};

// 练习区模式（完成实战后取消注释）：
// use swarm_core::naming_game::{
//     distinct_words, inventory_size_distribution, is_consensus, total_words,
//     NamingGame,
// };

use swarm_core::naming_game::network::{AdjacencyGraph, CompleteGraph, Network};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!(" 🗣️ Naming Game: Microscopic Activity Patterns (cond-mat/0606125)");
    println!(" 📄 Local Paper: papers/naming-game/01-microscopic-activity-condmat2006/paper.pdf");
    println!("============================================================\n");

    std::fs::create_dir_all("data")?;
    std::fs::create_dir_all("output")?;

    let macro_csv_path = "data/naming_game_macro.csv";
    let dist_csv_path = "data/naming_game_dist.csv";

    let mut macro_writer = csv::Writer::from_writer(File::create(macro_csv_path)?);
    let mut dist_writer = csv::Writer::from_writer(File::create(dist_csv_path)?);

    macro_writer.write_record(&["topology", "step", "nw", "nd", "success_rate"])?;
    dist_writer.write_record(&["topology", "group", "inventory_size", "prob"])?;

    let mut rng = StdRng::seed_from_u64(42);

    // =========================================================================
    // 1. 完全图 (Complete Graph KN, N=400)
    // =========================================================================
    println!("▶ [1/3] 模拟完全图 (Complete Graph KN, N=400)...");
    {
        let n = 400;
        let graph = CompleteGraph::new(n);
        let mut game = NamingGame::new(graph);

        let max_steps = 60_000;
        let sample_interval = 100;
        let mut recent_success = 0;
        let mut dist_sampled = false;

        for step in 1..=max_steps {
            let res = game.step(&mut rng);
            if res.success {
                recent_success += 1;
            }

            if step % sample_interval == 0 {
                let nw = total_words(&game.inventories);
                let nd = distinct_words(&game.inventories);
                let rate = recent_success as f64 / sample_interval as f64;
                recent_success = 0;

                macro_writer.write_record(&[
                    "Complete",
                    &step.to_string(),
                    &nw.to_string(),
                    &nd.to_string(),
                    &format!("{:.4}", rate),
                ])?;

                // 在重构区采样微观词汇分布 P_n (约在 step = 8000 左右)
                if !dist_sampled && step >= 8_000 {
                    dist_sampled = true;
                    let dist = inventory_size_distribution(&game.inventories, None);
                    for (sz, prob) in dist {
                        dist_writer.write_record(&[
                            "Complete",
                            "All",
                            &sz.to_string(),
                            &format!("{:.6}", prob),
                        ])?;
                    }
                }

                if is_consensus(&game.inventories) {
                    println!("  ✨ 完全图在 step = {} 达成全网共识！", step);
                    break;
                }
            }
        }
    }

    // =========================================================================
    // 2. 同质随机网络 (Erdös-Rényi Graph, N=500, <k>=20)
    // =========================================================================
    println!("▶ [2/3] 模拟同质随机网络 (Erdös-Rényi Graph, N=500, <k>=20)...");
    {
        let n = 500;
        let avg_k = 20.0;
        let graph = AdjacencyGraph::erdos_renyi(n, avg_k, &mut rng);
        let mut game = NamingGame::new(graph);

        let max_steps = 70_000;
        let sample_interval = 100;
        let mut recent_success = 0;
        let mut dist_sampled = false;

        for step in 1..=max_steps {
            let res = game.step(&mut rng);
            if res.success {
                recent_success += 1;
            }

            if step % sample_interval == 0 {
                let nw = total_words(&game.inventories);
                let nd = distinct_words(&game.inventories);
                let rate = recent_success as f64 / sample_interval as f64;
                recent_success = 0;

                macro_writer.write_record(&[
                    "ErdosRenyi",
                    &step.to_string(),
                    &nw.to_string(),
                    &nd.to_string(),
                    &format!("{:.4}", rate),
                ])?;

                // 在重构区采样微观词汇分布 P_n (step = 10000 左右)
                if !dist_sampled && step >= 10_000 {
                    dist_sampled = true;
                    let dist = inventory_size_distribution(&game.inventories, None);
                    for (sz, prob) in dist {
                        dist_writer.write_record(&[
                            "ErdosRenyi",
                            "Typical",
                            &sz.to_string(),
                            &format!("{:.6}", prob),
                        ])?;
                    }
                }

                if is_consensus(&game.inventories) {
                    println!("  ✨ ER 随机网络在 step = {} 达成全网共识！", step);
                    break;
                }
            }
        }
    }

    // =========================================================================
    // 3. 异质无标度网络 (Barabási-Albert Graph, N=500, m0=10, m=10)
    // =========================================================================
    println!("▶ [3/3] 模拟异质无标度网络 (Barabási-Albert Graph, N=500, m=10)...");
    {
        let n = 500;
        let m0 = 10;
        let m = 10;
        let graph = AdjacencyGraph::barabasi_albert(n, m0, m, &mut rng);

        // 识别网络中的 Hub 节点 (度前 10% 的高度数节点) 与 普通节点
        let mut node_degrees: Vec<(usize, usize)> = (0..n).map(|i| (i, graph.degree(i))).collect();
        node_degrees.sort_by_key(|&(_, deg)| std::cmp::Reverse(deg));
        let num_hubs = n / 10;
        let hub_nodes: Vec<usize> = node_degrees[..num_hubs].iter().map(|&(i, _)| i).collect();
        let regular_nodes: Vec<usize> = node_degrees[num_hubs..].iter().map(|&(i, _)| i).collect();

        let mut game = NamingGame::new(graph);

        let max_steps = 70_000;
        let sample_interval = 100;
        let mut recent_success = 0;
        let mut dist_sampled = false;

        for step in 1..=max_steps {
            let res = game.step(&mut rng);
            if res.success {
                recent_success += 1;
            }

            if step % sample_interval == 0 {
                let nw = total_words(&game.inventories);
                let nd = distinct_words(&game.inventories);
                let rate = recent_success as f64 / sample_interval as f64;
                recent_success = 0;

                macro_writer.write_record(&[
                    "BarabasiAlbert",
                    &step.to_string(),
                    &nw.to_string(),
                    &nd.to_string(),
                    &format!("{:.4}", rate),
                ])?;

                // 在重构区分别采样 Hub 节点与普通节点的微观分布 P_n(k)
                if !dist_sampled && step >= 10_000 {
                    dist_sampled = true;
                    // Hub 节点
                    let hub_dist = inventory_size_distribution(&game.inventories, Some(&hub_nodes));
                    for (sz, prob) in hub_dist {
                        dist_writer.write_record(&[
                            "BarabasiAlbert",
                            "Hubs",
                            &sz.to_string(),
                            &format!("{:.6}", prob),
                        ])?;
                    }
                    // 普通节点
                    let reg_dist = inventory_size_distribution(&game.inventories, Some(&regular_nodes));
                    for (sz, prob) in reg_dist {
                        dist_writer.write_record(&[
                            "BarabasiAlbert",
                            "Regular",
                            &sz.to_string(),
                            &format!("{:.6}", prob),
                        ])?;
                    }
                }

                if is_consensus(&game.inventories) {
                    println!("  ✨ BA 无标度网络在 step = {} 达成全网共识！", step);
                    break;
                }
            }
        }
    }

    macro_writer.flush()?;
    dist_writer.flush()?;

    println!("\n✅ 仿真数据已写入:");
    println!("  - 宏观时间演化序列: {}", macro_csv_path);
    println!("  - 微观词汇分布数据: {}", dist_csv_path);

    // 调用 Python 自动绘制高清对比图表
    println!("\n🎨 正在调用 Python 生成科学作图 (output/naming_game_*.png)...");
    let py_status = Command::new("uv")
        .args(["run", "python", "python/plot_naming_game.py"])
        .status()
        .or_else(|_| Command::new("python3").arg("python/plot_naming_game.py").status());

    match py_status {
        Ok(st) if st.success() => {
            println!("🎉 作图完成！请查看生成图表:");
            println!("  - 宏观序参量对比: output/naming_game_macro.png");
            println!("  - 微观词汇分布对比: output/naming_game_micro.png");
        }
        _ => {
            println!("💡 可手动运行以下命令出图:\n  uv run python python/plot_naming_game.py");
        }
    }

    Ok(())
}
