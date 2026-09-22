#!/usr/bin/env python3
"""
plot_turing_morphogenesis.py

复现 Science Robotics 2018 论文核心图表：
- 论文: Morphogenesis in robot swarms (DOI: 10.1126/scirobotics.aau9178)
- 输出图表:
  1. output/turing_morphogenesis_snapshots.png: 空间形态发生多阶段演化与自愈切片 (对应 Fig. 2 & Fig. 3)
  2. output/turing_morphogenesis_metrics.png: 形态测量时序、迁移机器人数与断肢自愈动力学 (对应 Fig. 4)
"""

import os
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
from matplotlib.patches import Circle
from matplotlib.lines import Line2D

# 样式设置
plt.rcParams["font.sans-serif"] = ["DejaVu Sans", "Arial", "Helvetica"]
plt.rcParams["axes.unicode_minus"] = False
os.makedirs("output", exist_ok=True)

# 颜色映射字典 (匹配 Kilobot LED 原文设计)
COLOR_MAP = {
    "Green": "#2ca02c",   # 极化斑点核心 (Activator u > 4.0)
    "Cyan": "#17becf",    # 高浓度梯度
    "Blue": "#1f77b4",    # 中浓度梯度
    "Pink": "#e377c2",    # 低浓度梯度
    "Black": "#333333",   # 静止基质/熄灭
    "White": "#f5f5f5",   # ORBIT 边缘沿轮廓迁移
    "Red": "#d62728",     # FOLLOW 掉队重连
}


def plot_snapshots():
    snap_path = "data/turing_morphogenesis_snapshots.csv"
    if not os.path.exists(snap_path):
        print(f"❌ 未找到 {snap_path}")
        return

    df = pd.read_csv(snap_path)
    phases = [
        ("0_Initial", "Phase 0: Initial Cluster (t=0s)", "#666666"),
        ("1_TuringSpots", "Phase 1: Turing Spots Emergence", "#2ca02c"),
        ("2_EarlyGrowth", "Phase 2: Active Tissue Migration (ORBIT)", "#e6550d"),
        ("3_MatureShape", "Phase 3: Mature Multi-Lobed Morphology", "#31a354"),
        ("4_Amputated", "Phase 4: Targeted Amputation (x > 65mm)", "#d62728"),
        ("5_Regrown", "Phase 5: Reorganized Self-Healing", "#756bb1"),
    ]

    fig, axes = plt.subplots(2, 3, figsize=(16, 11), sharex=True, sharey=True)
    axes = axes.flatten()

    bot_radius = 16.5  # mm (Kilobot 物理半径)
    comm_radius = 85.0 # mm (红外通信/扩散半径)

    for idx, (phase_key, title_str, title_color) in enumerate(phases):
        ax = axes[idx]
        sub = df[df["phase"] == phase_key]

        if sub.empty:
            ax.set_title(title_str, fontsize=11, fontweight="bold", color=title_color)
            continue

        xs = sub["x"].values
        ys = sub["y"].values
        colors = [COLOR_MAP.get(c, "#555555") for c in sub["color"]]
        states = sub["state"].values

        # 绘制近邻通信网络拓扑连边 (轻度半透明)
        n = len(xs)
        for i in range(n):
            for j in range(i + 1, n):
                dist = np.hypot(xs[i] - xs[j], ys[i] - ys[j])
                if dist <= comm_radius:
                    ax.plot([xs[i], xs[j]], [ys[i], ys[j]], color="#cccccc", lw=0.4, alpha=0.35, zorder=1)

        # 绘制机器人圆盘
        for i in range(n):
            c_fill = colors[i]
            edge_c = "#ffffff" if c_fill == "#f5f5f5" else "#222222"
            lw = 1.2 if states[i] in ["ORBIT", "FOLLOW"] else 0.6
            circle = Circle((xs[i], ys[i]), bot_radius, facecolor=c_fill, edgecolor=edge_c,
                            lw=lw, alpha=0.9, zorder=3)
            ax.add_patch(circle)

        # 标记突起轴线与原点参考
        ax.axhline(0, color="gray", ls=":", lw=0.5, alpha=0.3)
        ax.axvline(0, color="gray", ls=":", lw=0.5, alpha=0.3)

        # 标注统计信息
        n_orbit = sum(states == "ORBIT")
        n_polar = sum(sub["is_polarized"] == 1)
        info_text = f"N={n} | Polarized: {n_polar} | Orbiting: {n_orbit}"
        ax.text(0.03, 0.04, info_text, transform=ax.transAxes, fontsize=9,
                bbox=dict(boxstyle="round,pad=0.25", facecolor="white", alpha=0.8, edgecolor="#cccccc"))

        ax.set_title(title_str, fontsize=11.5, fontweight="bold", color=title_color, pad=8)
        ax.set_aspect("equal")
        ax.grid(True, ls="--", alpha=0.25)
        ax.set_xlim(-220, 220)
        ax.set_ylim(-220, 220)

        if idx >= 3:
            ax.set_xlabel("X Position (mm)", fontsize=10.5)
        if idx % 3 == 0:
            ax.set_ylabel("Y Position (mm)", fontsize=10.5)

    # 图例
    legend_elements = [
        Line2D([0], [0], marker='o', color='w', markerfacecolor='#2ca02c', markersize=10, label='Polarized Spot (u > 4.0)'),
        Line2D([0], [0], marker='o', color='w', markerfacecolor='#17becf', markersize=9, label='Morphogen High Gradient'),
        Line2D([0], [0], marker='o', color='w', markerfacecolor='#1f77b4', markersize=8, label='Morphogen Mid Gradient'),
        Line2D([0], [0], marker='o', color='k', markerfacecolor='#f5f5f5', markersize=9, label='Migrating (ORBIT White LED)'),
        Line2D([0], [0], marker='o', color='w', markerfacecolor='#d62728', markersize=9, label='Follow/Rejoining'),
        Line2D([0], [0], color='#cccccc', lw=1.5, label='Infrared Graph Edge (R ≤ 85mm)'),
    ]
    fig.legend(handles=legend_elements, loc="lower center", ncol=6, fontsize=9.5,
               frameon=True, framealpha=0.9, bbox_to_anchor=(0.5, -0.01))

    plt.suptitle("Science Robotics 2018: Morphogenesis in Robot Swarms (Slavkov et al.)\n"
                 "Reaction-Diffusion Patterning Coupled to Differential Tissue Growth & Amputation Self-Healing",
                 fontsize=14, fontweight="bold", y=0.98)

    out_file = "output/turing_morphogenesis_snapshots.png"
    plt.tight_layout(rect=[0, 0.03, 1, 0.95])
    plt.savefig(out_file, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"📊 空间演化与自愈切片图已保存至: {out_file}")


def plot_metrics():
    metric_path = "data/turing_morphogenesis_metrics.csv"
    if not os.path.exists(metric_path):
        print(f"❌ 未找到 {metric_path}")
        return

    df = pd.read_csv(metric_path)
    if df.empty:
        return

    fig, (ax1, ax2, ax3) = plt.subplots(3, 1, figsize=(11, 10), sharex=True)

    t = df["time"]

    # 1. 机器人状态分布
    ax1.plot(t, df["polarized_count"], label="Polarized Robots (Turing Spots)", color="#2ca02c", lw=2.2)
    ax1.plot(t, df["orbiting_count"], label="Orbiting Edge Robots (Cell Migration)", color="#e6550d", lw=1.8, ls="--")
    ax1.plot(t, df["waiting_count"], label="Quiescent Waiting Robots", color="#7f7f7f", lw=1.5, alpha=0.7)
    ax1.set_ylabel("Robot Count", fontsize=11, fontweight="bold")
    ax1.set_title("(a) Swarm Differentiation & Active Tissue Migration", fontsize=12, fontweight="bold")
    ax1.legend(loc="upper right", framealpha=0.9, fontsize=9.5)
    ax1.grid(True, alpha=0.3)

    # 2. 空间几何伸展 (回转半径与最大展宽)
    ax2.plot(t, df["gyration_radius"], label="Gyration Radius Rg (mm)", color="#1f77b4", lw=2.0)
    ax2.plot(t, df["max_extent"], label="Max Swarm Span / Extension (mm)", color="#9467bd", lw=1.8, ls="-.")
    ax2.set_ylabel("Spatial Extent (mm)", fontsize=11, fontweight="bold")
    ax2.set_title("(b) Morphological Protrusion Outgrowth & Amputation Perturbation", fontsize=12, fontweight="bold")
    ax2.legend(loc="upper left", framealpha=0.9, fontsize=9.5)
    ax2.grid(True, alpha=0.3)

    # 3. 图灵斑点数与总体机器人数
    ax3.plot(t, df["turing_spots_count"], label="Identified Turing Spots Count", color="#2ca02c", lw=2.0)
    ax3.plot(t, df["total_robots"] / 10.0, label="Total Swarm Size (Scaled N / 10)", color="#333333", lw=1.5, ls=":")
    ax3.set_ylabel("Spot Count / Scaled N", fontsize=11, fontweight="bold")
    ax3.set_xlabel("Elapsed Time (s)", fontsize=11, fontweight="bold")
    ax3.set_title("(c) Self-Healing: Spot Re-emergence Post-Amputation", fontsize=12, fontweight="bold")
    ax3.legend(loc="upper right", framealpha=0.9, fontsize=9.5)
    ax3.grid(True, alpha=0.3)

    # 标注阶段边界虚线
    # 纯扩散约到 t = 350 * 0.05 = 17.5s, 生长到 t = 17.5 + 600 * 0.08 = 65.5s
    # 截肢发生在 65.5s
    t_diff = 17.5
    t_amp = 65.5

    for ax in [ax1, ax2, ax3]:
        ax.axvline(t_diff, color="#1f77b4", ls="--", lw=1.2, alpha=0.7)
        ax.axvline(t_amp, color="#d62728", ls="--", lw=1.2, alpha=0.7)

    ax1.text(t_diff / 2, ax1.get_ylim()[1] * 0.85, "Phase 1:\nDiffusion Only", ha="center", fontsize=8.5,
             color="#1f77b4", fontweight="bold")
    ax1.text((t_diff + t_amp) / 2, ax1.get_ylim()[1] * 0.85, "Phase 2:\nTuring Morphogenesis", ha="center",
             fontsize=8.5, color="#e6550d", fontweight="bold")
    ax1.text(t_amp + 15, ax1.get_ylim()[1] * 0.85, "Phase 3:\nAmputation & Healing", ha="center", fontsize=8.5,
             color="#d62728", fontweight="bold")

    plt.suptitle("Morphogenesis Quantitative Dynamics (Science Robotics 2018)", fontsize=13, fontweight="bold", y=0.99)
    out_file = "output/turing_morphogenesis_metrics.png"
    plt.tight_layout(rect=[0, 0, 1, 0.96])
    plt.savefig(out_file, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"📊 时序动力学与形态测量曲线已保存至: {out_file}")


if __name__ == "__main__":
    plot_snapshots()
    plot_metrics()
