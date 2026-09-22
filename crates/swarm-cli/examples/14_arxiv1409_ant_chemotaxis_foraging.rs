//! # Example 14: Ant Foraging via Chemotaxis & Trail Formation
//!
//! **论文**: *Modeling ant foraging: a chemotaxis approach with pheromones and trail formation*  
//! **作者**: Paulo Amorim (*Journal of Theoretical Biology*, 2015 / [arXiv:1409.3808](https://arxiv.org/abs/1409.3808))  
//!
//! ## 物理场景与模拟目标
//! 1. **双食物源竞争与觅食**:
//!    - 巢穴置于原点 $(0, 0)$，四周随机搜寻；
//!    - 食物源 1（高价值丰盛源）: 位于 $(10.0, 8.0)$，初始峰值浓度 $c_0 = 12.0$；
//!    - 食物源 2（较小近距离源）: 位于 $(-2.0, -8.0)$，初始峰值浓度 $c_0 = 6.0$。
//! 2. **自发路径涌现 (Trail Formation)**:
//!    - 观察随机扩散的觅食蚁 $u$ 发现食物后转变为搬运蚁 $w$；
//!    - 搬运蚁沿指向巢穴的势场 $\nabla a$ 回归并沿途铺设信息素 $v$；
//!    - 信息素引导更多 $u$ 蚁沿 $\nabla v$ 梯度前进，形成强正反馈双向路径。
//! 3. **食物枯竭后的路径自发消散 (Trail Dissipation)**:
//!    - 小食物源耗尽后，信息素自然挥发衰减，对应的蚁道自动解体。
//! 4. **参数空间与觅食效率对照**:
//!    - 对比标准趋化参数（$\chi_u = 50.0, \varepsilon = 0.5$）与低敏感/无路径参数（$\chi_u = 5.0, \varepsilon = 0.5$）的食物消耗速度。

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::process::Command;
use std::time::Instant;
use swarm_core::ant_foraging::{AntChemotaxisConfig, AntChemotaxisModel, FoodSource, Grid2D};

fn export_snapshot_csv(path: &str, u: &Grid2D, w: &Grid2D, v: &Grid2D, c: &Grid2D) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writeln!(writer, "x,y,u,w,v,c")?;
    for j in 0..u.ny {
        for i in 0..u.nx {
            let (x, y) = u.coord(i, j);
            writeln!(
                writer,
                "{:.3},{:.3},{:.5},{:.5},{:.5},{:.5}",
                x,
                y,
                u.get(i, j),
                w.get(i, j),
                v.get(i, j),
                c.get(i, j)
            )?;
        }
    }
    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🐜 =================================================================");
    println!("   arXiv:1409.3808 Ant Foraging via Chemotaxis & Trail Formation");
    println!("   PDE 连续介质趋化动力学与自组织蚁道涌现仿真");
    println!("=================================================================\n");

    let out_dir = "data";
    fs::create_dir_all(out_dir)?;
    fs::create_dir_all("output")?;

    let grid_size = 90;
    let l_half: f64 = 16.0; // 模拟区域 [-16, 16] x [-16, 16]
    let dt: f64 = 0.002;
    let total_time: f64 = 25.0; // 无量纲总时长

    let food_sources = vec![
        FoodSource {
            x: 9.0,
            y: 7.0,
            radius: 2.2,
            initial_density: 12.0,
        },
        FoodSource {
            x: -2.0,
            y: -8.0,
            radius: 1.8,
            initial_density: 6.0,
        },
    ];

    let base_config = AntChemotaxisConfig {
        chi_u: 60.0,
        d_w: 0.1,
        d_v: 0.1,
        epsilon: 0.5,
        lambda: 70.0,
        return_speed: 16.0,
        nest_radius: 1.6,
        phero_fade_radius: 5.0,
        emerge_time: 2.5,
        emerge_rate: 60.0,
    };

    println!("📍 空间与物理参数:");
    println!("   - 网格分辨率: {} x {} (总点数: {})", grid_size, grid_size, grid_size * grid_size);
    println!("   - 物理区域: [ -{:.1}, {:.1} ] x [ -{:.1}, {:.1} ]", l_half, l_half, l_half, l_half);
    println!("   - 时间步长 dt = {:.4}, 模拟总时长 T = {:.1}", dt, total_time);
    println!("   - 趋化敏感度 chi_u = {:.1}, 信息素挥发率 epsilon = {:.2}", base_config.chi_u, base_config.epsilon);

    let start_time = Instant::now();

    // 1. 运行核心基准实验（完整时空四场输出）
    println!("\n🚀 [实验 1/2] 运行标准双食物源自组织觅食实验...");
    let mut model = AntChemotaxisModel::new(grid_size, grid_size, l_half, base_config.clone(), &food_sources);

    let timeseries_file = File::create(format!("{}/ant_chemotaxis_timeseries.csv", out_dir))?;
    let mut ts_writer = BufWriter::new(timeseries_file);
    writeln!(ts_writer, "time,total_food,depletion_ratio,foraging_mass,returning_mass,total_ants,max_phero,total_phero")?;

    let snapshot_targets = [1.0, 5.0, 12.0, 24.0];
    let mut next_snapshot_idx = 0;

    let total_steps = (total_time / dt).ceil() as usize;
    let log_interval = total_steps / 20;

    for step in 0..=total_steps {
        let t = model.time;

        // 检查快照导出
        if next_snapshot_idx < snapshot_targets.len() && t >= snapshot_targets[next_snapshot_idx] {
            let snap_time = snapshot_targets[next_snapshot_idx];
            let snap_path = format!("{}/ant_chemotaxis_snapshot_t{:.0}.csv", out_dir, snap_time);
            export_snapshot_csv(&snap_path, &model.u, &model.w, &model.v, &model.c)?;
            println!("   📸 已导出时空快照 t = {:.1} -> {}", snap_time, snap_path);
            next_snapshot_idx += 1;
        }

        // 记录时间序列
        if step % 25 == 0 {
            let m = model.metrics();
            writeln!(
                ts_writer,
                "{:.3},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}",
                m.time,
                m.total_food,
                m.food_depletion_ratio,
                m.foraging_ants_mass,
                m.returning_ants_mass,
                m.total_ants_mass,
                m.max_pheromone,
                m.total_pheromone
            )?;
        }

        if step % log_interval == 0 || step == total_steps {
            let m = model.metrics();
            println!(
                "   [进度 {:3.0}%] t = {:5.2} | 剩余食物: {:6.1} (消耗 {:4.1}%) | 觅食蚁: {:5.1} | 搬运蚁: {:5.1} | 峰值信息素: {:5.2}",
                (step as f64 / total_steps as f64) * 100.0,
                m.time,
                m.total_food,
                m.food_depletion_ratio * 100.0,
                m.foraging_ants_mass,
                m.returning_ants_mass,
                m.max_pheromone
            );
        }

        if step < total_steps {
            model.step(dt);
        }
    }
    ts_writer.flush()?;

    // 2. 运行对比实验：不同参数下的食物运送效率（复现论文 Section 5 核心论点：路径形成显著提升搬运效率）
    println!("\n🔍 [实验 2/2] 对比不同趋化敏感度 chi_u 下的食物搬运效率...");
    let efficiency_file = File::create(format!("{}/ant_chemotaxis_efficiency.csv", out_dir))?;
    let mut eff_writer = BufWriter::new(efficiency_file);
    writeln!(eff_writer, "time,standard_trail,low_sensitivity,high_evaporation")?;

    let mut model_low_chi = AntChemotaxisModel::new(
        grid_size,
        grid_size,
        l_half,
        AntChemotaxisConfig {
            chi_u: 4.0, // 极低敏感度，无法形成有效蚁道
            ..base_config.clone()
        },
        &food_sources,
    );

    let mut model_high_evap = AntChemotaxisModel::new(
        grid_size,
        grid_size,
        l_half,
        AntChemotaxisConfig {
            chi_u: 60.0,
            epsilon: 3.5, // 极高挥发率，信息素来不及积累
            ..base_config.clone()
        },
        &food_sources,
    );

    // 重新运行对比模型的演化
    let compare_steps = (total_time / dt).ceil() as usize;
    // 使用先前记录的标准曲线数据回放，或同步记录
    // 为准确对齐，我们同步步进对比模型并从前面的 model 时序对比
    println!("   正在计算对照组模型...");
    // 重新加载标准组用于对齐
    let mut model_std = AntChemotaxisModel::new(grid_size, grid_size, l_half, base_config.clone(), &food_sources);

    for step in 0..=compare_steps {
        if step % 50 == 0 {
            let m_std = model_std.metrics();
            let m_low = model_low_chi.metrics();
            let m_high = model_high_evap.metrics();

            writeln!(
                eff_writer,
                "{:.3},{:.4},{:.4},{:.4}",
                model_std.time,
                m_std.food_depletion_ratio,
                m_low.food_depletion_ratio,
                m_high.food_depletion_ratio
            )?;
        }

        if step < compare_steps {
            model_std.step(dt);
            model_low_chi.step(dt);
            model_high_evap.step(dt);
        }
    }
    eff_writer.flush()?;

    let elapsed = start_time.elapsed();
    println!("\n✅ 仿真全部完成！耗时: {:.2} 秒", elapsed.as_secs_f64());

    // 3. 自动调用 Python 科学可视化绘图
    println!("\n📊 正在启动 Python 生成高清论文复现图表...");
    let py_status = Command::new("uv")
        .args(["run", "python", "python/plot_ant_chemotaxis.py"])
        .status();

    match py_status {
        Ok(status) if status.success() => {
            println!("🎉 图像生成成功！请在 output/ 目录查看结果图表。");
        }
        _ => {
            println!("⚠️ uv 运行失败，尝试 python3...");
            let _ = Command::new("python3")
                .arg("python/plot_ant_chemotaxis.py")
                .status();
        }
    }

    Ok(())
}
