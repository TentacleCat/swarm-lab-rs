#!/usr/bin/env python3
"""
Scientific plotting script for:
"Minority game with local interactions due to the presence of herding behavior" (arXiv:physics/0512087)
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

def plot_volatility(csv_path="data/minority_game_volatility.csv", out_path="output/minority_game_volatility.png"):
    if not os.path.exists(csv_path):
        print(f"⚠️ 文件未找到: {csv_path}，跳过波动率相图绘制。")
        return

    df = pd.read_csv(csv_path)
    fig, ax = plt.subplots(figsize=(8.5, 6.0))

    # 1. 抛硬币随机决策基准线 sigma^2 / N = 1.0
    ax.axhline(1.0, color="#666666", linestyle="--", lw=1.8, label="Random Choice (Coin-Toss Benchmark, $\\sigma^2/N = 1$)")

    # 2. 绘制各配置曲线
    styles = {
        "Standard MG": ("o-", "#1f77b4", 2.5, 7),
        "Regular Ring (K=8)": ("s-", "#2ca02c", 2.0, 6),
        "Small World (K=8, p=0.1)": ("^-", "#ff7f0e", 2.0, 6),
        "Small World (K=8, p=0.5)": ("D-", "#d62728", 2.2, 6),
    }

    for config, (fmt, color, lw, ms) in styles.items():
        sub = df[df["config"] == config].sort_values("alpha")
        if not sub.empty:
            ax.plot(sub["alpha"], sub["volatility"], fmt, color=color, lw=lw, markersize=ms, label=config)

    # 标注临界点 alpha_c
    ax.axvline(0.34, color="#9467bd", linestyle=":", lw=1.5, alpha=0.7)
    ax.text(0.35, 3.8, "Critical Point $\\alpha_c \\approx 0.34$\n(Standard MG Minimum)", color="#9467bd", fontsize=9, fontweight="bold")

    ax.set_xscale("log")
    ax.set_xlabel("Information Ratio $\\alpha = 2^M / N$ (Log Scale)", fontsize=11)
    ax.set_ylabel("Normalized Volatility $\\sigma^2 / N$", fontsize=11)
    ax.set_title("Fig 1-4: Volatility $\\sigma^2/N$ vs $\\alpha$ with Herding Behavior (physics/0512087)", fontsize=12, fontweight="bold", pad=12)
    ax.grid(True, alpha=0.3, which="both")
    ax.legend(framealpha=0.95, fontsize=10, loc="upper right")

    plt.tight_layout()
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    plt.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"📊 少数派博弈波动率相变图已保存至: {out_path}")

def plot_timeseries(csv_path="data/minority_game_timeseries.csv", out_path="output/minority_game_timeseries.png"):
    if not os.path.exists(csv_path):
        print(f"⚠️ 文件未找到: {csv_path}，跳过时间序列绘制。")
        return

    df = pd.read_csv(csv_path)
    configs = ["Standard MG (M=5)", "Regular Ring (K=8)", "Small World (K=8, p=0.5)"]
    colors = ["#1f77b4", "#2ca02c", "#d62728"]
    titles = [
        "Standard MG (Efficient Cooperation, $\\sigma^2/N \\ll 1$)",
        "Regular Ring with Herding ($K=8$)",
        "Small World with Herding ($K=8, p=0.5$, Violent Crowd)",
    ]

    fig, axes = plt.subplots(1, 3, figsize=(16, 4.5), sharey=True)

    for ax, cfg, color, title in zip(axes, configs, colors, titles):
        sub = df[df["config"] == cfg]
        if not sub.empty:
            ax.plot(sub["step"], sub["attendance"], color=color, lw=1.5, alpha=0.85)
            ax.axhline(0, color="#333333", linestyle="--", lw=1.0)
            ax.set_title(title, fontsize=10.5, fontweight="bold")
            ax.set_xlabel("Time step $t$", fontsize=11)
            if ax == axes[0]:
                ax.set_ylabel("Net Attendance $A(t) = \\sum a_i$", fontsize=11)
            ax.grid(True, alpha=0.3)

    plt.suptitle("Real-Time Market Fluctuation: Breakdown of Efficiency under Herding Behavior", fontsize=13, fontweight="bold", y=1.02)
    plt.tight_layout()
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    plt.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"📊 市场净动作波动时间序列已保存至: {out_path}")

if __name__ == "__main__":
    plot_volatility()
    plot_timeseries()
