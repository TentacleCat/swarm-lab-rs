//! # 示例 17: Springer/PPSN 2024 异构演化群体与表型可塑性集体感知实验
//!
//! **论文**: *Emergence of Specialised Collective Behaviors in Evolving Heterogeneous Swarms*  
//! **会议**: International Conference on Parallel Problem Solving from Nature (PPSN XVIII 2024, Springer LNCS 14965, pp. 53–69)  
//! **DOI**: `10.1007/978-3-031-70068-2_4` | **arXiv**: `2402.04763`  
//! **作者**: Fuda van Diggelen, Matteo de Carlo, Nicolas Cambier, Eliseo Ferrante, A. E. Eiben (VU Amsterdam)  
//!
//! ## 实验内容与复现目标
//! 1. **黑盒进化学习 (CMA-ES / 储备池神经网络 RNN)**:
//!    - 对比演化异构控制器 (36 维全基因型，分别赋给 Green 与 Red 储备池) 与同质基准 Baseline (18 维)；
//!    - 仅使用全局标量梯度爬升任务累积光强为适应度函数，无任何子任务先验引导；
//! 2. **子群专门化行为与协同相互作用 (复现原论文 Table 3)**:
//!    - 扫掠子群比例 {4:0, 3:1, 2:2, 1:3, 0:4} 与起始距离 r_{dist} \in {0.0, 0.25, 0.5, 0.75, 1.0} * 12m；
//!    - 验证中心强光区绿色剥削利用 (Exploitative) 占优，远距离弱光区红色协同探索 (Exploratory) 与两群混合涌现更强适应度；
//! 3. **在线自适应调控机制 / 表型可塑性 (复现原论文 Table 4)**:
//!    - 局部光强阈值概率有限状态机: P_{green}(light) 在 5.0s 周期下动态切换；
//!    - 可扩展性 (Scalability, N=10, 20, 50) 与鲁棒性 (Robustness: Center, Bi-modal, Linear, Banana) 全面基准对比；
//! 4. **空间动态轨迹与序参量时序**:
//!    - 导出 CSV 数据并自动调用 Python 绘制发表级科学复现全景图。

use rand::rngs::StdRng;
use rand::SeedableRng;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::process::Command;
use std::time::Instant;

// =========================================================================
// 💡 [闯关模式开关]
// 默认使用参考实现 (Reference)，保证开箱即用并可作为对照。
// 当你在 crates/swarm-core/src/heterogeneous_swarms/ 中亲手编写完成练习任务后，
// 注释掉下面这行，取消注释下一行，即可切换为你自己的实战代码检验成果！
// =========================================================================
use swarm_core::reference::heterogeneous_swarms::{
    ArenaTypeReference as ArenaType,
    CmaEsConfigReference as CmaEsConfig,
    CmaEsOptimizerReference as CmaEsOptimizer,
    HeterogeneousGenotypeReference as HeterogeneousGenotype,
    ReservoirReference as Reservoir,
    SwarmSimConfigReference as SwarmSimConfig,
    SwarmSimulatorReference as SwarmSimulator,
};
// use swarm_core::heterogeneous_swarms::{
//     ArenaType, CmaEsConfig, CmaEsOptimizer, HeterogeneousGenotype, Reservoir,
//     SwarmSimConfig, SwarmSimulator,
// };

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 ======================================================================");
    println!("   PPSN XVIII (2024) / Springer: Evolving Heterogeneous Swarms");
    println!("   异构演化群体与表型可塑性自组织涌现感知仿真实验 (VU Amsterdam)");
    println!("   DOI: 10.1007/978-3-031-70068-2_4");
    println!("======================================================================\n");

    let out_dir = "data";
    fs::create_dir_all(out_dir)?;
    fs::create_dir_all("output")?;

    let start_total = Instant::now();
    let mut rng = StdRng::seed_from_u64(2024);

    // 1. 初始化两套独立的冻结隐层储备池 Reservoir 1 (Green) 与 Reservoir 2 (Red)
    println!("🧠 [步骤 1/4] 初始化 9x9 随机隐层储备池 (Reservoirs frozen U[-1, 1])...");
    let res_green = Reservoir::new_random(&mut rng);
    let res_red = Reservoir::new_random(&mut rng);
    println!("   ✅ 储备池参数已锁定: 9 维感知输入 -> 2 层 ReLU 隐层 -> 2 维输出 (v, w)\n");

    // -------------------------------------------------------------------------
    // 步骤 2: CMA-ES 进化优化学习 (演化异构控制器 vs 同质基准)
    // -------------------------------------------------------------------------
    println!("🧬 [步骤 2/4] 执行 CMA-ES 进化学习实验 (Heterogeneous vs Baseline)...");
    let n_generations = 15; // 演示实验代数，保证秒级收敛并保留完整进化动力学
    let lambda = 16;
    let eval_sim_time = 30.0; // 进化评估时长

    let mut sim_cfg_evo = SwarmSimConfig::default();
    sim_cfg_evo.arena_type = ArenaType::Center;
    sim_cfg_evo.swarm_size = 20;
    sim_cfg_evo.ratio = (10, 10);
    sim_cfg_evo.spawn_distance = 12.0;
    sim_cfg_evo.simulation_time = eval_sim_time;
    sim_cfg_evo.dt = 0.1;

    let sim_evo = SwarmSimulator::new(sim_cfg_evo, res_green.clone(), res_red.clone());

    // 2.1 优化异构控制器 (D = 36)
    let cma_cfg_hetero = CmaEsConfig {
        dim: 36,
        lambda,
        mu: lambda / 2,
        sigma0: 1.0,
        bounds: (-5.0, 5.0),
        max_generations: n_generations,
        repeats_per_eval: 2,
    };
    let mut opt_hetero = CmaEsOptimizer::new(cma_cfg_hetero, &mut rng);

    // 2.2 优化同质基准控制器 (D = 18, 复制至两个子群)
    let cma_cfg_base = CmaEsConfig {
        dim: 18,
        lambda,
        mu: lambda / 2,
        sigma0: 1.0,
        bounds: (-5.0, 5.0),
        max_generations: n_generations,
        repeats_per_eval: 2,
    };
    let mut opt_base = CmaEsOptimizer::new(cma_cfg_base, &mut rng);

    let curve_path = format!("{}/heterogeneous_swarms_learning_curve.csv", out_dir);
    let mut curve_writer = BufWriter::new(File::create(&curve_path)?);
    writeln!(
        curve_writer,
        "generation,baseline_best_fit,baseline_mean_fit,hetero_best_fit,hetero_mean_fit"
    )?;

    for gen in 0..n_generations {
        let t_gen = Instant::now();

        // 异构个体采样与评估
        let pop_hetero = opt_hetero.sample_population(&mut rng);
        let mut fits_hetero = Vec::with_capacity(lambda);
        for ind in &pop_hetero {
            let geno = HeterogeneousGenotype::from_flat_slice(ind);
            let f = sim_evo.evaluate_median(&geno, 1000 + (gen as u64) * 31, 2);
            fits_hetero.push(f);
        }
        opt_hetero.update(&pop_hetero, &fits_hetero);

        // 同质个体采样与评估
        let pop_base = opt_base.sample_population(&mut rng);
        let mut fits_base = Vec::with_capacity(lambda);
        for ind in &pop_base {
            // 将 18 维权重复制给 Green 与 Red
            let mut w36 = vec![0.0; 36];
            w36[0..18].copy_from_slice(ind);
            w36[18..36].copy_from_slice(ind);
            let geno = HeterogeneousGenotype::from_flat_slice(&w36);
            let f = sim_evo.evaluate_median(&geno, 2000 + (gen as u64) * 31, 2);
            fits_base.push(f);
        }
        opt_base.update(&pop_base, &fits_base);

        let hetero_best = opt_hetero.best_fitness;
        let hetero_mean: f64 = fits_hetero.iter().sum::<f64>() / (lambda as f64);
        let base_best = opt_base.best_fitness;
        let base_mean: f64 = fits_base.iter().sum::<f64>() / (lambda as f64);

        writeln!(
            curve_writer,
            "{},{},{},{},{}",
            gen, base_best, base_mean, hetero_best, hetero_mean
        )?;

        if gen % 3 == 0 || gen == n_generations - 1 {
            println!(
                "   [Gen {:>2}/{}] Hetero Best: {:.4} (Mean: {:.4}) | Baseline Best: {:.4} (Mean: {:.4}) [{:.2}s]",
                gen, n_generations, hetero_best, hetero_mean, base_best, base_mean, t_gen.elapsed().as_secs_f64()
            );
        }
    }
    curve_writer.flush()?;
    println!("   📊 进化收敛曲线已写入: {}\n", curve_path);

    // 最优控制器基因型
    let best_hetero_geno = HeterogeneousGenotype::from_flat_slice(&opt_hetero.best_candidate);
    let mut best_base_36 = vec![0.0; 36];
    best_base_36[0..18].copy_from_slice(&opt_base.best_candidate);
    best_base_36[18..36].copy_from_slice(&opt_base.best_candidate);
    let best_base_geno = HeterogeneousGenotype::from_flat_slice(&best_base_36);

    // -------------------------------------------------------------------------
    // 步骤 3: 子群比例与距离相互作用验证 (复现 Table 3)
    // -------------------------------------------------------------------------
    println!("🔬 [步骤 3/4] 评估子群协同效应 (复现论文 Table 3 比例 x 距离网格)...");
    let table3_path = format!("{}/heterogeneous_swarms_ratios_table3.csv", out_dir);
    let mut table3_writer = BufWriter::new(File::create(&table3_path)?);
    writeln!(
        table3_writer,
        "ratio_name,green_count,red_count,dist_ratio,spawn_dist,mean_fitness,std_fitness,mean_order"
    )?;

    let ratio_configs = [
        ("4:0 (All Green)", 20, 0),
        ("3:1", 15, 5),
        ("2:2 (Equal)", 10, 10),
        ("1:3", 5, 15),
        ("0:4 (All Red)", 0, 20),
    ];
    let dist_ratios = [0.0, 0.25, 0.50, 0.75, 1.0];
    let n_eval_trials = 5;

    for &(r_name, n_g, n_r) in &ratio_configs {
        for &r_frac in &dist_ratios {
            let spawn_d = r_frac * 12.0;

            let mut sim_cfg = SwarmSimConfig::default();
            sim_cfg.arena_type = ArenaType::Center;
            sim_cfg.swarm_size = 20;
            sim_cfg.ratio = (n_g, n_r);
            sim_cfg.spawn_distance = spawn_d;
            sim_cfg.simulation_time = 45.0; // 评测时长
            sim_cfg.dt = 0.1;
            sim_cfg.regulatory_enabled = false;

            let sim = SwarmSimulator::new(sim_cfg, res_green.clone(), res_red.clone());

            let mut fits = Vec::with_capacity(n_eval_trials);
            let mut orders = Vec::with_capacity(n_eval_trials);

            for trial in 0..n_eval_trials {
                let seed = 3000 + (trial as u64) * 101;
                let res = sim.run_trial(&best_hetero_geno, seed, false);
                fits.push(res.fitness);
                orders.push(res.final_order);
            }

            let mean_fit: f64 = fits.iter().sum::<f64>() / (n_eval_trials as f64);
            let var_fit: f64 = fits.iter().map(|&x| (x - mean_fit).powi(2)).sum::<f64>()
                / (n_eval_trials as f64);
            let std_fit = var_fit.sqrt();
            let mean_ord: f64 = orders.iter().sum::<f64>() / (n_eval_trials as f64);

            writeln!(
                table3_writer,
                "{},{},{},{:.2},{:.1},{:.4},{:.4},{:.4}",
                r_name, n_g, n_r, r_frac, spawn_d, mean_fit, std_fit, mean_ord
            )?;
        }
    }
    table3_writer.flush()?;
    println!("   📊 Table 3 比例矩阵数据已写入: {}\n", table3_path);

    // -------------------------------------------------------------------------
    // 步骤 4: 可扩展性与环境鲁棒性对比 (复现 Table 4)
    // -------------------------------------------------------------------------
    println!("🌍 [步骤 4/4] 验证可扩展性与跨环境鲁棒性 (复现论文 Table 4)...");
    let table4_path = format!("{}/heterogeneous_swarms_scalability_robustness_table4.csv", out_dir);
    let mut table4_writer = BufWriter::new(File::create(&table4_path)?);
    writeln!(
        table4_writer,
        "exp_type,condition_label,controller_type,mean_fitness,std_fitness,mean_order,std_order"
    )?;

    // 4.1 可扩展性实验: Swarm size in {10, 20, 50}
    let swarm_sizes = [10, 20, 50];
    let controllers = ["Baseline", "Best Hetero", "Adaptive"];

    for &size in &swarm_sizes {
        for &ctrl_name in &controllers {
            let mut sim_cfg = SwarmSimConfig::default();
            sim_cfg.arena_type = ArenaType::Center;
            sim_cfg.swarm_size = size;
            sim_cfg.ratio = (size / 2, size - size / 2);
            sim_cfg.spawn_distance = 12.0;
            sim_cfg.simulation_time = 45.0;
            sim_cfg.dt = 0.1;
            sim_cfg.regulatory_enabled = ctrl_name == "Adaptive";

            let sim = SwarmSimulator::new(sim_cfg, res_green.clone(), res_red.clone());
            let target_geno = if ctrl_name == "Baseline" {
                &best_base_geno
            } else {
                &best_hetero_geno
            };

            let mut fits = Vec::with_capacity(n_eval_trials);
            let mut orders = Vec::with_capacity(n_eval_trials);

            for trial in 0..n_eval_trials {
                let seed = 4000 + (trial as u64) * 233;
                let res = sim.run_trial(target_geno, seed, false);
                fits.push(res.fitness);
                orders.push(res.final_order);
            }

            let mean_fit: f64 = fits.iter().sum::<f64>() / (n_eval_trials as f64);
            let std_fit: f64 = (fits.iter().map(|&x| (x - mean_fit).powi(2)).sum::<f64>()
                / (n_eval_trials as f64))
                .sqrt();
            let mean_ord: f64 = orders.iter().sum::<f64>() / (n_eval_trials as f64);
            let std_ord: f64 = (orders.iter().map(|&x| (x - mean_ord).powi(2)).sum::<f64>()
                / (n_eval_trials as f64))
                .sqrt();

            writeln!(
                table4_writer,
                "Scalability,Size {},{},{:.4},{:.4},{:.4},{:.4}",
                size, ctrl_name, mean_fit, std_fit, mean_ord, std_ord
            )?;
        }
    }

    // 4.2 鲁棒性实验: 4 种竞技场环境 (Center, Bimodal, Linear, Banana)
    let arenas = [
        (ArenaType::Center, "Center"),
        (ArenaType::Bimodal, "Bi-modal"),
        (ArenaType::Linear, "Linear"),
        (ArenaType::Banana, "Banana"),
    ];

    for &(arena, arena_name) in &arenas {
        for &ctrl_name in &controllers {
            let mut sim_cfg = SwarmSimConfig::default();
            sim_cfg.arena_type = arena;
            sim_cfg.swarm_size = 20;
            sim_cfg.ratio = (10, 10);
            sim_cfg.spawn_distance = 12.0;
            sim_cfg.simulation_time = 45.0;
            sim_cfg.dt = 0.1;
            sim_cfg.regulatory_enabled = ctrl_name == "Adaptive";

            let sim = SwarmSimulator::new(sim_cfg, res_green.clone(), res_red.clone());
            let target_geno = if ctrl_name == "Baseline" {
                &best_base_geno
            } else {
                &best_hetero_geno
            };

            let mut fits = Vec::with_capacity(n_eval_trials);
            let mut orders = Vec::with_capacity(n_eval_trials);

            for trial in 0..n_eval_trials {
                let seed = 5000 + (trial as u64) * 317;
                let res = sim.run_trial(target_geno, seed, false);
                fits.push(res.fitness);
                orders.push(res.final_order);
            }

            let mean_fit: f64 = fits.iter().sum::<f64>() / (n_eval_trials as f64);
            let std_fit: f64 = (fits.iter().map(|&x| (x - mean_fit).powi(2)).sum::<f64>()
                / (n_eval_trials as f64))
                .sqrt();
            let mean_ord: f64 = orders.iter().sum::<f64>() / (n_eval_trials as f64);
            let std_ord: f64 = (orders.iter().map(|&x| (x - mean_ord).powi(2)).sum::<f64>()
                / (n_eval_trials as f64))
                .sqrt();

            writeln!(
                table4_writer,
                "Robustness,{},{},{:.4},{:.4},{:.4},{:.4}",
                arena_name, ctrl_name, mean_fit, std_fit, mean_ord, std_ord
            )?;
        }
    }
    table4_writer.flush()?;
    println!("   📊 Table 4 可扩展性与鲁棒性数据已写入: {}\n", table4_path);

    // -------------------------------------------------------------------------
    // 步骤 5: 记录高分辨率空间轨迹与宏观时序
    // -------------------------------------------------------------------------
    println!("🗺️  [步骤 5/5] 记录单次 Adaptive Swarm 空间轨迹与序参量时序...");
    let mut sim_cfg_track = SwarmSimConfig::default();
    sim_cfg_track.arena_type = ArenaType::Center;
    sim_cfg_track.swarm_size = 20;
    sim_cfg_track.ratio = (10, 10);
    sim_cfg_track.spawn_distance = 12.0;
    sim_cfg_track.simulation_time = 60.0;
    sim_cfg_track.dt = 0.1;
    sim_cfg_track.regulatory_enabled = true;

    let sim_track = SwarmSimulator::new(sim_cfg_track, res_green.clone(), res_red.clone());
    let track_res = sim_track.run_trial(&best_hetero_geno, 9999, true);

    let ts_path = format!("{}/heterogeneous_swarms_timeseries.csv", out_dir);
    let mut ts_writer = BufWriter::new(File::create(&ts_path)?);
    writeln!(
        ts_writer,
        "step,time,mean_intensity,green_intensity,red_intensity,swarm_order,green_order,red_order,green_ratio,center_dist,gyration_radius"
    )?;
    for snap in &track_res.metrics_history {
        writeln!(
            ts_writer,
            "{},{:.2},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}",
            snap.step,
            snap.time,
            snap.mean_intensity,
            snap.green_intensity,
            snap.red_intensity,
            snap.swarm_order,
            snap.green_order,
            snap.red_order,
            snap.green_ratio,
            snap.center_distance,
            snap.gyration_radius
        )?;
    }
    ts_writer.flush()?;
    println!("   📊 时序数据已写入: {}", ts_path);

    // 记录最终机器人位置快照
    let snap_path = format!("{}/heterogeneous_swarms_final_snapshot.csv", out_dir);
    let mut snap_writer = BufWriter::new(File::create(&snap_path)?);
    writeln!(snap_writer, "robot_id,x,y,heading,subgroup")?;
    for bot in &track_res.final_robots {
        writeln!(
            snap_writer,
            "{},{:.4},{:.4},{:.4},{}",
            bot.id, bot.position[0], bot.position[1], bot.heading, bot.subgroup.name()
        )?;
    }
    snap_writer.flush()?;
    println!("   📊 最终位置快照已写入: {}\n", snap_path);

    println!("✨ Rust 仿真核心全部完成！耗时: {:.2}s", start_total.elapsed().as_secs_f64());
    println!("🎨 正在调用 Python 生成发表级科学图表...");

    // 尝试调用 python 绘图脚本
    let py_status = Command::new("uv")
        .args(["run", "python", "python/plot_heterogeneous_swarms.py"])
        .status()
        .or_else(|_| {
            Command::new("python3")
                .args(["python/plot_heterogeneous_swarms.py"])
                .status()
        });

    match py_status {
        Ok(status) if status.success() => {
            println!("✅ 科学图表绘制成功！已保存在 output/ 目录。");
        }
        _ => {
            println!("ℹ️  提示: 可手动运行 `uv run python python/plot_heterogeneous_swarms.py` 出图。");
        }
    }

    Ok(())
}
