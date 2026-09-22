#!/usr/bin/env python3
"""
Scientific plotting script for:
"Aggregating swarms through morphology handling design contingencies: from the sweet spot to a rich expressivity"
(arXiv:2601.07610, Jan 2026 / Jeremy Fersula, Nicolas Bredeche, Olivier Dauchot)

Reproduces:
1. Figure 3: Phase diagram of fraction in light (N_circ / N) and polar alignment (<Psi>) vs epsilon/tau_n
2. Spatial Snapshots and Trajectories for 4 representative morphological regimes:
   - Jamming (kappa = -3.5)
   - Sweet Spot (kappa = -1.2)
   - Active Brownian (kappa = 0.0)
   - Polar Flocking (kappa = +2.5)
"""

import os
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.patches import Circle

plt.style.use("seaborn-v0_8-whitegrid" if "seaborn-v0_8-whitegrid" in plt.style.available else "default")
plt.rcParams["font.sans-serif"] = ["DejaVu Sans", "Arial", "Helvetica"]
plt.rcParams["axes.edgecolor"] = "#333333"
plt.rcParams["axes.linewidth"] = 1.0


def plot_phase_diagram(scan_path="data/morphological_swarms_phase_scan.csv",
                       out_path="output/morphological_swarms_phase_diagram.png"):
    """复现论文 Figure 3: 随 epsilon / tau_n 变化的聚集率与极化度双轴相图"""
    if not os.path.exists(scan_path):
        print(f"⚠️ 文件未找到: {scan_path}，跳过相图绘制。")
        return

    df = pd.read_csv(scan_path)
    if df.empty:
        return

    # 按 kappa 聚合平均值与中位数
    df_mean = df.groupby("kappa", as_index=False)[["light_ratio", "polar_alignment"]].mean()
    df_median = df.groupby("kappa", as_index=False)[["light_ratio", "polar_alignment"]].median()

    fig, ax1 = plt.subplots(figsize=(11, 6))
    ax2 = ax1.twinx()

    # 背景相态区域着色
    ax1.axvspan(-5.2, -2.2, color="#ffebee", alpha=0.35, label="Jamming / Freeze")
    ax1.axvspan(-2.2, -0.5, color="#e8f5e9", alpha=0.45, label="MIPS Sweet Spot")
    ax1.axvspan(-0.5, 0.5, color="#f5f5f5", alpha=0.4, label="ABP Baseline")
    ax1.axvspan(0.5, 5.2, color="#e3f2fd", alpha=0.35, label="Polar Flocking")

    # 散点：单次试验数据
    ax1.scatter(df["kappa"], df["light_ratio"], color="#c62828", alpha=0.45, s=36, zorder=3)
    ax2.scatter(df["kappa"], df["polar_alignment"], color="#1565c0", alpha=0.45, s=36, zorder=3)

    # 主实线：时间平均与系综中位数
    line_light, = ax1.plot(df_median["kappa"], df_median["light_ratio"],
                           color="#b71c1c", linewidth=2.8, marker="o", markersize=6,
                           label=r"Light Fraction $N_\circ / N$ (Aggregation)")
    line_align, = ax2.plot(df_median["kappa"], df_median["polar_alignment"],
                           color="#0d47a1", linewidth=2.8, marker="s", markersize=6,
                           label=r"Polar Alignment $\langle \Psi \rangle$ (Flocking)")

    # 理论基线: 随机均匀游走在光照区的预期占比 (sigma * v_bullet / v_circ = 0.06 * 3.0 = 0.18)
    baseline_line = ax1.axhline(y=0.18, color="#212121", linestyle="--", linewidth=1.8, zorder=2,
                               label="0.18 Baseline (Random Walk)")
    ax1.annotate("0.18 (Random exploration baseline)", xy=(-4.9, 0.195),
                 color="#212121", fontsize=11, fontweight="bold")

    # 甜点区高亮标注
    sweet_spot_x = df_median.loc[df_median["light_ratio"].idxmax(), "kappa"]
    sweet_spot_y = df_median["light_ratio"].max()
    ax1.annotate(f"Sweet Spot\n($N_\\circ/N$ peak: {sweet_spot_y * 100.0:.1f}%)",
                 xy=(sweet_spot_x, sweet_spot_y),
                 xytext=(sweet_spot_x - 1.2, sweet_spot_y + 0.12),
                 arrowprops=dict(facecolor="#b71c1c", shrink=0.08, width=1.8, headwidth=7),
                 color="#b71c1c", fontsize=11, fontweight="bold",
                 bbox=dict(boxstyle="round,pad=0.3", facecolor="#ffffff", edgecolor="#b71c1c", alpha=0.9))

    # 轴属性设置
    ax1.set_xlabel(r"Self-Alignment Strength $\epsilon / \tau_n$", fontsize=15, fontweight="bold", labelpad=8)
    ax1.set_ylabel(r"Fraction in Light $N_\circ / N$", fontsize=15, fontweight="bold", color="#b71c1c", labelpad=8)
    ax2.set_ylabel(r"Global Polar Alignment $\langle \Psi \rangle$", fontsize=15, fontweight="bold", color="#0d47a1", labelpad=8)

    ax1.set_xlim(-5.1, 5.1)
    ax1.set_ylim(0.0, 0.8)
    ax2.set_ylim(0.0, 1.0)

    ax1.tick_params(axis="y", labelsize=12, labelcolor="#b71c1c")
    ax1.tick_params(axis="x", labelsize=12)
    ax2.tick_params(axis="y", labelsize=12, labelcolor="#0d47a1")
    ax1.grid(True, linestyle=":", alpha=0.6)

    # 标题与合并图例
    plt.title("arXiv:2601.07610 Reproduction (Figure 3):\nSwarm Aggregation and Polar Alignment across Morphology Space",
              fontsize=14, fontweight="bold", pad=12)

    # 合并图例
    lines = [line_light, line_align, baseline_line]
    labels = [line.get_label() if line.get_label() else "0.18 Baseline" for line in lines]
    ax1.legend(lines, labels, loc="upper right", framealpha=0.95, fontsize=11)

    plt.tight_layout()
    plt.savefig(out_path, dpi=300)
    plt.close()
    print(f"📊 相图已保存至: {out_path}")


def plot_spatial_snapshots(snap_path="data/morphological_swarms_snapshots.csv",
                           traj_path="data/morphological_swarms_trajectories.csv",
                           out_path="output/morphological_swarms_spatial_snapshots.png"):
    """绘制 4 种典型物理相态的空间瞬态快照与运动迹线"""
    if not os.path.exists(snap_path):
        print(f"⚠️ 文件未找到: {snap_path}，跳过空间快照绘制。")
        return

    df_snap = pd.read_csv(snap_path)
    df_traj = pd.read_csv(traj_path) if os.path.exists(traj_path) else None

    cases = [
        ("Fronter_Jammed", "A. Fronter Jammed (κ = -3.5)\nDark-Zone Micro-Cluster Freezing"),
        ("Fronter_SweetSpot", "B. Fronter Sweet Spot (κ = -1.2)\nMIPS Nucleation in Lit Region"),
        ("Active_Brownian", "C. Active Brownian Particles (κ = 0.0)\nIsotropic Random Exploration"),
        ("Aligner_Flocking", "D. Aligner Flocking (κ = +2.5)\nPolar Coherent Collective Flow"),
    ]

    fig, axes = plt.subplots(2, 2, figsize=(13, 13))
    axes = axes.flatten()

    half_l = 13.85
    light_radius = 3.83

    for idx, (case_name, title) in enumerate(cases):
        ax = axes[idx]
        sub_snap = df_snap[df_snap["case_name"] == case_name]
        if sub_snap.empty:
            continue

        # 1. 绘制周期性盒子背景与中心光照区
        ax.set_facecolor("#fafafa")
        light_circle = Circle((0, 0), light_radius, facecolor="#c8e6c9", alpha=0.7,
                              linestyle="--", edgecolor="#2e7d32", linewidth=2.0, zorder=1)
        ax.add_patch(light_circle)
        ax.text(0, 0, "Lit Region\n(6% Area)", color="#1b5e20", fontsize=9,
                ha="center", va="center", fontweight="bold", alpha=0.8, zorder=2)

        # 2. 绘制最近轨迹尾迹 (若有)
        if df_traj is not None:
            sub_traj = df_traj[df_traj["case_name"] == case_name]
            if not sub_traj.empty:
                max_time = sub_traj["time"].max()
                tail_traj = sub_traj[sub_traj["time"] >= max_time - 15.0]  # 最近 15 tau
                for robot_id in tail_traj["robot_id"].unique():
                    rt = tail_traj[tail_traj["robot_id"] == robot_id]
                    # 避免跨越周期性边界绘制穿屏长线
                    dx = np.diff(rt["x"])
                    dy = np.diff(rt["y"])
                    jump = (np.abs(dx) > half_l) | (np.abs(dy) > half_l)
                    if not jump.any():
                        ax.plot(rt["x"], rt["y"], color="#78909c", alpha=0.35, linewidth=1.0, zorder=2)

        # 3. 绘制机器人质点 (按是否在光区着色)
        in_light = sub_snap["in_light"] == 1
        # 光照区粒子 (红色/绿色) 与暗区粒子 (灰色/蓝色)
        if case_name == "Aligner_Flocking":
            # 极化流统一显眼亮蓝
            node_colors = ["#1565c0" for _ in range(len(sub_snap))]
        else:
            node_colors = ["#d32f2f" if il else "#546e7a" for il in in_light]

        ax.scatter(sub_snap["x"], sub_snap["y"], c=node_colors, s=55,
                   edgecolor="#212121", linewidth=0.8, zorder=4)

        # 4. 绘制朝向速度向量 (quiver)
        ax.quiver(sub_snap["x"], sub_snap["y"], sub_snap["mu_x"], sub_snap["mu_y"],
                  color="#212121", scale=30, width=0.005, headwidth=4, headlength=5, zorder=5)

        # 统计在光照区的数量
        n_in = in_light.sum()
        pct = (n_in / len(sub_snap)) * 100.0
        ax.text(-half_l + 0.8, half_l - 1.5, f"In Light: {n_in}/{len(sub_snap)} ({pct:.1f}%)",
                fontsize=11, fontweight="bold",
                bbox=dict(boxstyle="round,pad=0.3", facecolor="white", edgecolor="#666666", alpha=0.9),
                zorder=6)

        ax.set_xlim(-half_l, half_l)
        ax.set_ylim(-half_l, half_l)
        ax.set_aspect("equal")
        ax.set_title(title, fontsize=12, fontweight="bold", pad=8)
        ax.set_xlabel("x (body diameters)", fontsize=10)
        ax.set_ylabel("y (body diameters)", fontsize=10)
        ax.grid(True, linestyle=":", alpha=0.5)

    plt.suptitle("Emergent Collective Spatial Modes in Swarms with Morphological Self-Alignment (arXiv:2601.07610)",
                 fontsize=14, fontweight="bold", y=0.99)
    plt.tight_layout()
    plt.savefig(out_path, dpi=300)
    plt.close()
    print(f"🗺️ 空间快照已保存至: {out_path}")


if __name__ == "__main__":
    out_dir = "output"
    os.makedirs(out_dir, exist_ok=True)

    print("🚀 开始绘制 arXiv:2601.07610 论文复现科研图表...")
    plot_phase_diagram()
    plot_spatial_snapshots()
    print("✨ 图表绘制完成！")
