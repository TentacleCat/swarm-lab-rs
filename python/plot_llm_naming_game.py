#!/usr/bin/env python3
"""
Scientific plotting script for:
"Microscopic dynamics of consensus formation in multi-agent LLM Naming Games" (arXiv:2608.02178)
"""

import os
import sys
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

plt.style.use("seaborn-v0_8-whitegrid" if "seaborn-v0_8-whitegrid" in plt.style.available else "default")
plt.rcParams["font.sans-serif"] = ["DejaVu Sans", "Arial", "Helvetica"]
plt.rcParams["axes.edgecolor"] = "#333333"
plt.rcParams["axes.linewidth"] = 1.0

def plot_macroscopic_trajectories(traj_csv="data/llm_naming_game_trajectories.csv", out_path="output/llm_naming_game_trajectories.png"):
    if not os.path.exists(traj_csv):
        print(f"⚠️ 文件未找到: {traj_csv}，跳过轨迹图绘制。")
        return

    df = pd.read_csv(traj_csv)
    architectures = ["LLaMA-3.1:8B", "Mistral:7B", "Phi-3:14B"]
    titles = [
        "LLaMA-3.1:8B (Permissive)\nStandard Temp Ordering ($t_c \\sim e^{0.67 T}$)",
        "Mistral:7B (Near-Deterministic)\nTemperature Blindness ($\\alpha \\approx 0$)",
        "Phi-3:14B (Conservative)\nInverted Temp Ordering ($k$-diversity bottleneck)",
    ]

    fig, axes = plt.subplots(1, 3, figsize=(16, 4.8), sharey=True)

    colors = {
        0.1: "#1f77b4",  # Blue (low T)
        1.0: "#ff7f0e",  # Orange (mid T)
        2.0: "#d62728",  # Red (high T)
    }

    for ax, arch, title in zip(axes, architectures, titles):
        sub_arch = df[df["arch"] == arch]
        temps = sorted(sub_arch["temperature"].unique())

        for t in temps:
            sub = sub_arch[sub_arch["temperature"] == t]
            c = colors.get(float(f"{t:.1f}"), "#333333")
            label = f"$T = {t:.1f}$"
            ax.plot(sub["step"], sub["nd"], label=label, color=c, lw=2.2)

        ax.set_title(title, fontsize=11, fontweight="bold", pad=10)
        ax.set_xlabel("Time step $t$", fontsize=11)
        if ax == axes[0]:
            ax.set_ylabel("Distinct Words $N_d(t)$", fontsize=12)
        ax.grid(True, alpha=0.3)
        ax.legend(framealpha=0.9, fontsize=10, loc="upper right")

    plt.suptitle("Fig 5: Macroscopic Consensus Trajectories in Multi-Agent LLM Naming Games (arXiv:2608.02178)", fontsize=13, fontweight="bold", y=1.02)
    plt.tight_layout()
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    plt.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"📊 宏观轨迹图已保存至: {out_path}")

def plot_phase_diagram(summary_csv="data/llm_naming_game_summary.csv", out_path="output/llm_naming_game_phase_diagram.png"):
    fig, ax = plt.subplots(figsize=(8, 6.5))

    # 1. 绘制 (pi, phi) 相图背景与临界线: 3*pi - 2*phi - 1 = 0 => pi_c = (1 + 2*phi) / 3
    phi_grid = np.linspace(0, 0.6, 200)
    pi_crit = (1.0 + 2.0 * phi_grid) / 3.0

    ax.plot(phi_grid, pi_crit, color="#000000", lw=2.5, linestyle="--", label="Critical Line: $3\\pi - 2\\phi - 1 = 0$")
    ax.fill_between(phi_grid, pi_crit, 1.05, color="#2ca02c", alpha=0.15, label="Ordered Phase ($R > 0$, Consensus)")
    ax.fill_between(phi_grid, 0, pi_crit, color="#d62728", alpha=0.15, label="Disordered Phase ($R < 0$, Repaint Noise)")

    # 标记特殊点
    ax.scatter([0.0], [1.0], color="#1f77b4", s=100, zorder=5, marker="*")
    ax.annotate("Deterministic NG\n$(\\pi=1, \\phi=0)$", xy=(0.0, 1.0), xytext=(0.04, 0.98),
                fontsize=9, fontweight="bold")

    ax.scatter([0.0], [1.0 / 3.0], color="#333333", s=70, zorder=5, marker="x")
    ax.annotate("Baronchelli 2006\nThreshold $\\beta_c = 1/3$", xy=(0.0, 1.0 / 3.0), xytext=(0.03, 0.32),
                fontsize=9)

    # 2. 如果存在汇总数据，绘制实测模型的轨迹
    if os.path.exists(summary_csv):
        df_sum = pd.read_csv(summary_csv)
        arch_markers = {
            "LLaMA-3.1:8B": ("o-", "#9467bd"),
            "Mistral:7B": ("s-", "#17becf"),
            "Phi-3:14B": ("^-", "#8c564b"),
        }

        for arch, (marker, color) in arch_markers.items():
            sub = df_sum[df_sum["arch"] == arch].sort_values("temperature")
            if not sub.empty:
                ax.plot(sub["phi"], sub["pi"], marker, color=color, lw=2.5, markersize=8, label=f"{arch} ($T: 0.1 \\to 2.0$)")
                # 在最高温度点加箭头指示
                ax.annotate(f"{arch}\n(High T)", xy=(sub["phi"].iloc[-1], sub["pi"].iloc[-1]),
                            xytext=(sub["phi"].iloc[-1] + 0.02, sub["pi"].iloc[-1] - 0.05),
                            arrowprops=dict(arrowstyle="->", color=color),
                            fontsize=8, color=color, fontweight="semibold")

    ax.set_xlim(-0.02, 0.58)
    ax.set_ylim(0.15, 1.05)
    ax.set_xlabel("Repaint Rate $\\phi(T) \\equiv P(\\text{YES} \\mid w \\notin P_j)$ (Out-inventory noise)", fontsize=11)
    ax.set_ylabel("Consolidation Rate $\\pi(T) \\equiv P(\\text{YES} \\mid w \\in P_j)$ (In-inventory)", fontsize=11)
    ax.set_title("Fig 2 & 4: Two-Rate $(\\pi, \\phi)$ Phase Diagram & LLM Architecture Trajectories", fontsize=12, fontweight="bold", pad=12)
    ax.grid(True, alpha=0.3)
    ax.legend(loc="lower right", framealpha=0.95, fontsize=9.5)

    plt.tight_layout()
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    plt.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"📊 相图已保存至: {out_path}")

if __name__ == "__main__":
    plot_macroscopic_trajectories()
    plot_phase_diagram()
