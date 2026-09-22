#!/usr/bin/env python3
"""
Scientific plotting script for:
"Provable self-organizing pattern formation by a swarm of robots with limited knowledge"
(Swarm Intelligence 2019 / TU Delft / DOI: 10.1007/s11721-019-00163-0)

Reproduces:
1. Spatial Grid snapshots of pattern self-organization (Initial -> Intermediate -> P_des)
2. Normalized histograms of steps to completion across patterns (Fig 11)
3. Performance comparison of Baseline vs ALT1 vs ALT2 (Fig 12)
"""

import os
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.patches import Circle, FancyArrowPatch

plt.style.use("seaborn-v0_8-whitegrid" if "seaborn-v0_8-whitegrid" in plt.style.available else "default")
plt.rcParams["font.sans-serif"] = ["DejaVu Sans", "Arial", "Helvetica"]
plt.rcParams["axes.edgecolor"] = "#333333"
plt.rcParams["axes.linewidth"] = 1.0

COLOR_DESIRED = "#2ca02c"   # Green: Happy (in S_des)
COLOR_ACTIVE = "#ff7f0e"    # Orange: Active (in S_active)
COLOR_BLOCKED = "#d62728"   # Red: Blocked (in S_blocked)

def plot_spatial_trajectories(traj_path="data/swarm_pattern_trajectories.csv",
                              out_path="output/swarm_pattern_formation_grid.png"):
    """绘制构型自组织网格演化快照 (从无序连通初始状态到期望构型)"""
    if not os.path.exists(traj_path):
        print(f"⚠️ 文件未找到: {traj_path}，跳过空间演化图绘制。")
        return

    df = pd.read_csv(traj_path)
    patterns = ["Triangle-4", "Square-4", "Hexagon-6"]
    patterns = [p for p in patterns if p in df["pattern"].values]

    if not patterns:
        return

    n_rows = len(patterns)
    fig, axes = plt.subplots(n_rows, 4, figsize=(15, 3.8 * n_rows))
    if n_rows == 1:
        axes = axes.reshape(1, -1)

    for r_idx, pat in enumerate(patterns):
        sub = df[df["pattern"] == pat]
        steps = sorted(sub["step"].unique())
        if len(steps) > 4:
            # 选 4 个
            steps = [steps[0], steps[len(steps)//3], steps[2*len(steps)//3], steps[-1]]

        # 计算该构型所有点全局外包矩形以便统一比例
        x_min, x_max = sub["x"].min() - 1.5, sub["x"].max() + 1.5
        y_min, y_max = sub["y"].min() - 1.5, sub["y"].max() + 1.5
        span = max(x_max - x_min, y_max - y_min, 5.0)
        cx = 0.5 * (x_min + x_max)
        cy = 0.5 * (y_min + y_max)
        xlim = (cx - 0.5 * span, cx + 0.5 * span)
        ylim = (cy - 0.5 * span, cy + 0.5 * span)

        for c_idx, st in enumerate(steps[:4]):
            ax = axes[r_idx, c_idx]
            slice_df = sub[sub["step"] == st]

            # 绘制离散网格线
            ax.set_xticks(np.arange(int(xlim[0]), int(xlim[1]) + 1))
            ax.set_yticks(np.arange(int(ylim[0]), int(ylim[1]) + 1))
            ax.grid(True, color="#dcdcdc", linestyle="--", lw=0.8)

            # 绘制北向指南针指示器 (Assumption A1)
            if c_idx == 0:
                ax.annotate("N", xy=(0.08, 0.90), xycoords="axes fraction", fontsize=10, fontweight="bold", color="#025e8d")
                ax.annotate("↑", xy=(0.08, 0.82), xycoords="axes fraction", fontsize=13, fontweight="bold", color="#025e8d")

            # 绘制各机器人
            for _, row in slice_df.iterrows():
                rx, ry = row["x"], row["y"]
                state_tp = row["state_type"]
                color = COLOR_DESIRED if state_tp == "desired" else (COLOR_ACTIVE if state_tp == "active" else COLOR_BLOCKED)

                circle = Circle((rx, ry), 0.38, facecolor=color, edgecolor="#222222", lw=1.5, zorder=4)
                ax.add_patch(circle)
                ax.text(rx, ry, str(int(row["robot_id"])), color="white", fontsize=8.5,
                        ha="center", va="center", fontweight="bold", zorder=5)

            ax.set_xlim(xlim)
            ax.set_ylim(ylim)
            ax.set_aspect("equal")

            phase_label = "Initial $P_0$" if c_idx == 0 else (f"Final $P_{{des}}$" if c_idx == 3 else f"Step {st}")
            ax.set_title(f"{pat} | {phase_label}", fontsize=11, fontweight="bold")

    plt.suptitle("Provable Swarm Robotics: Spatial Self-Organization on 2D Moore Grid (Swarm Intelligence 2019)",
                 fontsize=14, fontweight="bold", y=0.99)
    plt.tight_layout()
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    plt.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"📊 构型自组织网格演化切片已保存至: {out_path}")

def plot_step_histograms(mc_path="data/swarm_pattern_formation_steps.csv",
                         out_path="output/swarm_pattern_formation_histograms.png"):
    """绘制收敛步数统计直方图与启发式对比 (复现 Fig 11 & Fig 12)"""
    if not os.path.exists(mc_path):
        print(f"⚠️ 文件未找到: {mc_path}，跳过步数直方图绘制。")
        return

    df = pd.read_csv(mc_path)
    df_conv = df[df["converged"] == True]

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 5.2))

    # 1. 图 11: 各构型在 ALT1 模式下的收敛步数概率密度分布
    patterns = ["Triangle-4", "Square-4", "Cross-5", "Hexagon-6"]
    colors = ["#1f77b4", "#2ca02c", "#ff7f0e", "#9467bd"]

    df_alt1 = df_conv[df_conv["mode"] == "ALT1"]
    for pat, color in zip(patterns, colors):
        sub = df_alt1[df_alt1["pattern"] == pat]
        if not sub.empty:
            steps = sub["steps"].values
            ax1.hist(steps, bins=15, alpha=0.55, density=True, label=f"{pat} (N={len(steps)})", color=color, edgecolor=color)
            median_val = np.median(steps)
            ax1.axvline(median_val, color=color, linestyle="--", lw=1.5, alpha=0.8)

    ax1.set_xlabel("Steps to Form Pattern", fontsize=11)
    ax1.set_ylabel("Probability Density", fontsize=11)
    ax1.set_title("Fig 11: Steps to Completion across Target Patterns (ALT1)", fontsize=12, fontweight="bold")
    ax1.legend(framealpha=0.9, fontsize=9.5)
    ax1.grid(True, alpha=0.3)

    # 2. 图 12: Baseline vs ALT1 vs ALT2 对比 (以 Triangle-4 为例)
    sub_tri = df_conv[df_conv["pattern"] == "Triangle-4"]
    modes = ["Baseline", "ALT1", "ALT2"]
    mode_colors = ["#7f7f7f", "#1f77b4", "#2ca02c"]

    box_data = [sub_tri[sub_tri["mode"] == m]["steps"].values for m in modes if not sub_tri[sub_tri["mode"] == m].empty]
    valid_modes = [m for m in modes if not sub_tri[sub_tri["mode"] == m].empty]

    if box_data:
        bp = ax2.boxplot(box_data, patch_artist=True, widths=0.5,
                         medianprops=dict(color="black", lw=2.0))
        ax2.set_xticks(range(1, len(valid_modes) + 1))
        ax2.set_xticklabels(valid_modes)
        for patch, color in zip(bp["boxes"], mode_colors):
            patch.set_facecolor(color)
            patch.set_alpha(0.65)

        # 叠加抖动散点
        for i, data in enumerate(box_data):
            y = data
            x = np.random.normal(i + 1, 0.04, size=len(y))
            ax2.plot(x, y, "k.", alpha=0.4, markersize=6)

        ax2.set_ylabel("Steps to Completion", fontsize=11)
        ax2.set_title("Fig 12: Optimization Heuristics Comparison (Triangle-4)", fontsize=12, fontweight="bold")
        ax2.grid(True, alpha=0.3)

    plt.suptitle("Provable Swarm Self-Organization: Convergence Statistics (DOI: 10.1007/s11721-019-00163-0)",
                 fontsize=13, fontweight="bold", y=1.02)
    plt.tight_layout()
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    plt.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"📊 收敛步数直方图与启发式对比已保存至: {out_path}")

if __name__ == "__main__":
    plot_spatial_trajectories()
    plot_step_histograms()
