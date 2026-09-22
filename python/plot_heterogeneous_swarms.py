#!/usr/bin/env python3
"""
Scientific plotting script for:
"Emergence of Specialised Collective Behaviors in Evolving Heterogeneous Swarms"
(PPSN XVIII 2024 / Springer LNCS 14965, DOI: 10.1007/978-3-031-70068-2_4)

Reproduces:
1. Learning convergence curves of CMA-ES (Heterogeneous vs Baseline)
2. Sub-group interaction matrix across ratios and initial distances (Table 3)
3. Scalability (N=10, 20, 50) and Robustness (Center, Bi-modal, Linear, Banana) (Table 4)
4. Temporal dynamics: Mean light intensity, swarm order Phi(t), and spatial distribution
"""

import os
import glob
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

# Matplotlib 样式设置
plt.style.use("seaborn-v0_8-whitegrid" if "seaborn-v0_8-whitegrid" in plt.style.available else "default")
plt.rcParams["font.sans-serif"] = ["DejaVu Sans", "Arial", "Helvetica"]
plt.rcParams["axes.edgecolor"] = "#333333"
plt.rcParams["axes.linewidth"] = 1.0

COLOR_HETERO = "#2ca02c"   # Green
COLOR_BASE = "#1f77b4"     # Blue
COLOR_ADAPTIVE = "#d62728" # Red/Orange
COLOR_RED_SUB = "#d62728"

def plot_learning_curves(data_path="data/heterogeneous_swarms_learning_curve.csv", out_path="output/heterogeneous_swarms_learning_curves.png"):
    """绘制 CMA-ES 进化优化学习曲线 (复现异构控制器与同质基准收敛对比)"""
    if not os.path.exists(data_path):
        print(f"⚠️ 未找到 {data_path}，跳过学习曲线绘制。")
        return

    df = pd.read_csv(data_path)
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(13, 5))

    # 1. 最优适应度曲线
    ax1.plot(df["generation"], df["hetero_best_fit"], "o-", color=COLOR_HETERO, linewidth=2.5, markersize=6, label="Heterogeneous Swarm (36-D)")
    ax1.plot(df["generation"], df["baseline_best_fit"], "s--", color=COLOR_BASE, linewidth=2.0, markersize=5, label="Baseline Homogeneous (18-D)")
    ax1.set_title("Best Population Fitness per Generation", fontsize=12, fontweight="bold")
    ax1.set_xlabel("Generation (CMA-ES)", fontsize=11)
    ax1.set_ylabel("Fitness f (Cumulative Light)", fontsize=11)
    ax1.legend(loc="lower right", framealpha=0.9, fontsize=10)
    ax1.grid(True, linestyle="--", alpha=0.6)

    # 2. 群体均值适应度曲线
    ax2.plot(df["generation"], df["hetero_mean_fit"], "o-", color=COLOR_HETERO, linewidth=2.0, alpha=0.85, label="Hetero Population Mean")
    ax2.plot(df["generation"], df["baseline_mean_fit"], "s--", color=COLOR_BASE, linewidth=2.0, alpha=0.85, label="Baseline Population Mean")
    ax2.set_title("Mean Population Fitness per Generation", fontsize=12, fontweight="bold")
    ax2.set_xlabel("Generation (CMA-ES)", fontsize=11)
    ax2.set_ylabel("Mean Fitness", fontsize=11)
    ax2.legend(loc="lower right", framealpha=0.9, fontsize=10)
    ax2.grid(True, linestyle="--", alpha=0.6)

    plt.suptitle("CMA-ES Black-Box Optimization: Emergence of Heterogeneous Controllers (PPSN 2024)", fontsize=13, fontweight="bold", y=0.98)
    plt.tight_layout()
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    plt.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"📊 学习曲线已保存至: {out_path}")

def plot_subgroup_ratios(data_path="data/heterogeneous_swarms_ratios_table3.csv", out_path="output/heterogeneous_swarms_subgroup_ratios.png"):
    """绘制子群比例与生成距离相互作用矩阵 (复现论文 Table 3)"""
    if not os.path.exists(data_path):
        print(f"⚠️ 未找到 {data_path}，跳过子群协同分析图绘制。")
        return

    df = pd.read_csv(data_path)
    ratios = df["ratio_name"].unique()
    dist_ratios = sorted(df["dist_ratio"].unique())

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 5.5))

    # 1. 多折线图对比各比例在不同初始距离下的适应度
    palette = ["#2ca02c", "#8c564b", "#1f77b4", "#ff7f0e", "#d62728"]
    for i, ratio in enumerate(ratios):
        sub = df[df["ratio_name"] == ratio].sort_values("dist_ratio")
        ax1.plot(sub["dist_ratio"], sub["mean_fitness"], "o-", linewidth=2.2, color=palette[i % len(palette)], label=ratio)
        ax1.fill_between(sub["dist_ratio"], sub["mean_fitness"] - sub["std_fitness"], sub["mean_fitness"] + sub["std_fitness"], alpha=0.12, color=palette[i % len(palette)])

    ax1.set_title("Performance vs Spawn Distance Across Subgroup Ratios (Table 3)", fontsize=12, fontweight="bold")
    ax1.set_xlabel(r"Normalized Distance to Center $r_{dist} / 12\mathrm{m}$", fontsize=11)
    ax1.set_ylabel("Mean Fitness f", fontsize=11)
    ax1.legend(loc="upper right", framealpha=0.9, fontsize=9.5)
    ax1.grid(True, linestyle="--", alpha=0.6)

    # 2. 二维热力矩阵 (Pivot Table: Distance x Ratio)
    pivot_df = df.pivot(index="ratio_name", columns="dist_ratio", values="mean_fitness")
    im = ax2.imshow(pivot_df.values, cmap="YlGnBu", aspect="auto")
    ax2.set_xticks(range(len(dist_ratios)))
    ax2.set_xticklabels([f"{d:.2f}" for d in dist_ratios])
    ax2.set_yticks(range(len(pivot_df.index)))
    ax2.set_yticklabels(pivot_df.index)
    ax2.set_title("Fitness Heatmap (Green:Red Specialization Synergy)", fontsize=12, fontweight="bold")
    ax2.set_xlabel(r"Distance Ratio $r_{dist}$", fontsize=11)
    ax2.set_ylabel("Subgroup Ratio (Green:Red)", fontsize=11)

    for i in range(len(pivot_df.index)):
        for j in range(len(dist_ratios)):
            val = pivot_df.values[i, j]
            text_color = "white" if val > pivot_df.values.max() * 0.75 else "black"
            ax2.text(j, i, f"{val:.3f}", ha="center", va="center", color=text_color, fontweight="bold", fontsize=9.5)

    fig.colorbar(im, ax=ax2, shrink=0.85, pad=0.03, label="Fitness f")

    plt.suptitle("Emergence of Specialised Roles: Exploitative (Green) vs Exploratory (Red) Synergy", fontsize=13, fontweight="bold", y=0.98)
    plt.tight_layout()
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    plt.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"📊 子群比例矩阵图已保存至: {out_path}")

def plot_scalability_robustness(data_path="data/heterogeneous_swarms_scalability_robustness_table4.csv", out_path="output/heterogeneous_swarms_scalability_robustness.png"):
    """绘制可扩展性与跨环境鲁棒性对比柱状图 (复现论文 Table 4)"""
    if not os.path.exists(data_path):
        print(f"⚠️ 未找到 {data_path}，跳过可扩展性与鲁棒性图绘制。")
        return

    df = pd.read_csv(data_path)
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 5.5))

    controllers = ["Baseline", "Best Hetero", "Adaptive"]
    colors = [COLOR_BASE, COLOR_HETERO, COLOR_ADAPTIVE]

    # 1. 可扩展性 (Scalability: Swarm Size in 10, 20, 50)
    df_scale = df[df["exp_type"] == "Scalability"]
    conditions_scale = df_scale["condition_label"].unique()
    x = np.arange(len(conditions_scale))
    width = 0.25

    for idx, ctrl in enumerate(controllers):
        sub = df_scale[df_scale["controller_type"] == ctrl]
        sub = sub.set_index("condition_label").reindex(conditions_scale).reset_index()
        ax1.bar(x + (idx - 1) * width, sub["mean_fitness"], width, yerr=sub["std_fitness"], capsize=4, label=ctrl, color=colors[idx], alpha=0.85, edgecolor="#222")

    ax1.set_title("Scalability Across Swarm Sizes (N = 10, 20, 50)", fontsize=12, fontweight="bold")
    ax1.set_xticks(x)
    ax1.set_xticklabels(conditions_scale, fontsize=10.5)
    ax1.set_ylabel("Mean Fitness f", fontsize=11)
    ax1.legend(loc="upper left", framealpha=0.9, fontsize=10)
    ax1.grid(True, linestyle="--", alpha=0.5)

    # 2. 鲁棒性 (Robustness: Center, Bi-modal, Linear, Banana)
    df_robust = df[df["exp_type"] == "Robustness"]
    conditions_robust = ["Center", "Bi-modal", "Linear", "Banana"]
    x2 = np.arange(len(conditions_robust))

    for idx, ctrl in enumerate(controllers):
        sub = df_robust[df_robust["controller_type"] == ctrl]
        sub = sub.set_index("condition_label").reindex(conditions_robust).reset_index()
        ax2.bar(x2 + (idx - 1) * width, sub["mean_fitness"], width, yerr=sub["std_fitness"], capsize=4, label=ctrl, color=colors[idx], alpha=0.85, edgecolor="#222")

    ax2.set_title("Robustness Across Challenging Environments (Table 4)", fontsize=12, fontweight="bold")
    ax2.set_xticks(x2)
    ax2.set_xticklabels(conditions_robust, fontsize=10.5)
    ax2.set_ylabel("Mean Fitness f", fontsize=11)
    ax2.legend(loc="upper right", framealpha=0.9, fontsize=10)
    ax2.grid(True, linestyle="--", alpha=0.5)

    plt.suptitle("Validation of Online Regulatory Mechanism: Significant Gains in Scalability & Robustness", fontsize=13, fontweight="bold", y=0.98)
    plt.tight_layout()
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    plt.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"📊 可扩展性与鲁棒性图已保存至: {out_path}")

def plot_trajectories_and_dynamics(ts_path="data/heterogeneous_swarms_timeseries.csv", snap_path="data/heterogeneous_swarms_final_snapshot.csv", out_path="output/heterogeneous_swarms_trajectories.png"):
    """绘制群体宏观时序与最终空间分布快照"""
    if not os.path.exists(ts_path):
        print(f"⚠️ 未找到 {ts_path}，跳过时序动态图绘制。")
        return

    df_ts = pd.read_csv(ts_path)
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 5.5))

    # 1. 光强与序参量动态时序
    ax1_twin = ax1.twinx()
    l1 = ax1.plot(df_ts["time"], df_ts["mean_intensity"], "-", color="#1f77b4", linewidth=2.5, label="Mean Light $l_t$")
    l2 = ax1.plot(df_ts["time"], df_ts["green_ratio"] * 255.0, "--", color="#2ca02c", linewidth=1.8, label="Green Ratio (% x 255)")
    l3 = ax1_twin.plot(df_ts["time"], df_ts["swarm_order"], "-.", color="#d62728", linewidth=2.2, label=r"Swarm Order $\Phi(t)$")

    ax1.set_title("Temporal Dynamics: Gradient Sensing & Phenotypic Switching", fontsize=12, fontweight="bold")
    ax1.set_xlabel("Simulation Time (s)", fontsize=11)
    ax1.set_ylabel("Light Intensity / Ratio", fontsize=11)
    ax1_twin.set_ylabel(r"Alignment Order Parameter $\Phi$", fontsize=11, color="#d62728")
    ax1_twin.set_ylim(0.0, 1.05)

    lines = l1 + l2 + l3
    labels = [l.get_label() for l in lines]
    ax1.legend(lines, labels, loc="lower right", framealpha=0.9, fontsize=9.5)
    ax1.grid(True, linestyle="--", alpha=0.5)

    # 2. 空间竞技场二维光强背景与机器人位置
    if os.path.exists(snap_path):
        df_snap = pd.read_csv(snap_path)
        # 生成 30x30m 背景标量场
        grid_x, grid_y = np.meshgrid(np.linspace(0, 30, 150), np.linspace(0, 30, 150))
        dist_c = np.sqrt((grid_x - 15.0)**2 + (grid_y - 15.0)**2)
        field = 255.0 * np.clip(1.0 - dist_c / 14.5, 0.0, 1.0)

        im = ax2.imshow(field, origin="lower", extent=[0, 30, 0, 30], cmap="copper", alpha=0.75)
        fig.colorbar(im, ax=ax2, shrink=0.85, pad=0.03, label="Scalar Light Intensity G")

        # 绘制机器人位置
        green_bots = df_snap[df_snap["subgroup"] == "Green"]
        red_bots = df_snap[df_snap["subgroup"] == "Red"]

        ax2.scatter(green_bots["x"], green_bots["y"], c="#2ca02c", s=70, edgecolor="white", linewidth=1.2, label=f"Green Robots ({len(green_bots)})", zorder=4)
        ax2.scatter(red_bots["x"], red_bots["y"], c="#d62728", s=70, edgecolor="white", linewidth=1.2, label=f"Red Robots ({len(red_bots)})", zorder=4)

        # 绘制航向箭头
        for _, row in df_snap.iterrows():
            dx = 0.8 * np.cos(row["heading"])
            dy = 0.8 * np.sin(row["heading"])
            ax2.arrow(row["x"], row["y"], dx, dy, color="white", head_width=0.3, alpha=0.8, zorder=5)

        ax2.plot(15.0, 15.0, "w*", markersize=14, label="Light Center (Optimum)", zorder=6)
        ax2.set_title("Spatial Swarm Distribution in 30x30m Arena", fontsize=12, fontweight="bold")
        ax2.set_xlabel("X (meters)", fontsize=11)
        ax2.set_ylabel("Y (meters)", fontsize=11)
        ax2.set_xlim(0, 30)
        ax2.set_ylim(0, 30)
        ax2.legend(loc="upper right", framealpha=0.9, fontsize=9.5)
    else:
        ax2.text(0.5, 0.5, "No snapshot found", ha="center", va="center")

    plt.suptitle("Swarm Spatial Organization and Emergent Gradient Tracking", fontsize=13, fontweight="bold", y=0.98)
    plt.tight_layout()
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    plt.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"📊 时序与空间分布图已保存至: {out_path}")

def main():
    print("🎨 [Python] 开始生成发表级科学复现全景图...")
    plot_learning_curves()
    plot_subgroup_ratios()
    plot_scalability_robustness()
    plot_trajectories_and_dynamics()
    print("🎉 [Python] 所有科学图表绘制完成！")

if __name__ == "__main__":
    main()
