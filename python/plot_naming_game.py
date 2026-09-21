#!/usr/bin/env python3
"""
Scientific plotting script for Naming Game paper reproduction:
"Microscopic activity patterns in the Naming Game" (cond-mat/0606125)
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

def plot_macro_dynamics(csv_path="data/naming_game_macro.csv", out_path="output/naming_game_macro.png"):
    if not os.path.exists(csv_path):
        print(f"⚠️ 文件未找到: {csv_path}，跳过宏观演化绘图。")
        return

    df = pd.read_csv(csv_path)
    topologies = ["Complete", "ErdosRenyi", "BarabasiAlbert"]
    titles = [
        "Complete Graph (Mean-Field, $N=400$)",
        "Erdős–Rényi Graph (Homogeneous, $\\langle k\\rangle=20$)",
        "Barabási–Albert Graph (Scale-Free, $m=10$)"
    ]

    fig, axes = plt.subplots(1, 3, figsize=(16, 4.5), sharey=False)

    for ax, topo, title in zip(axes, topologies, titles):
        sub = df[df["topology"] == topo]
        if sub.empty:
            continue

        # 双 y 轴：左侧绘制 Nw 和 Nd，右侧绘制成功率 S(t)
        color_w = "#1f77b4"
        color_d = "#ff7f0e"
        color_s = "#2ca02c"

        ax2 = ax.twinx()

        l1 = ax.plot(sub["step"], sub["nw"], label="Total Words $N_w(t)$", color=color_w, lw=2.0)
        l2 = ax.plot(sub["step"], sub["nd"], label="Distinct Words $N_d(t)$", color=color_d, lw=1.8, linestyle="--")
        l3 = ax2.plot(sub["step"], sub["success_rate"], label="Success Rate $S(t)$", color=color_s, lw=1.5, linestyle=":")

        ax.set_title(title, fontsize=12, fontweight="bold", pad=10)
        ax.set_xlabel("Time step $t$", fontsize=11)
        ax.set_ylabel("Word Counts ($N_w, N_d$)", fontsize=11, color="#222222")
        ax2.set_ylabel("Success Rate $S(t)$", fontsize=11, color=color_s)
        ax2.set_ylim(0, 1.05)
        ax.grid(True, alpha=0.3)

        # 标注峰值
        max_nw_idx = sub["nw"].idxmax()
        peak_step = sub.loc[max_nw_idx, "step"]
        peak_nw = sub.loc[max_nw_idx, "nw"]
        ax.annotate(
            f"Peak $N_w={peak_nw}$",
            xy=(peak_step, peak_nw),
            xytext=(peak_step, peak_nw * 1.15),
            arrowprops=dict(facecolor="black", shrink=0.08, width=1, headwidth=5),
            fontsize=9,
            fontweight="semibold",
            ha="center"
        )

        lines = l1 + l2 + l3
        labels = [l.get_label() for l in lines]
        ax.legend(lines, labels, loc="upper right" if topo != "Complete" else "center right", framealpha=0.9, fontsize=9)

    plt.suptitle("Fig 1: Macroscopic Dynamics of Naming Game across Topologies", fontsize=14, fontweight="bold", y=1.02)
    plt.tight_layout()
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    plt.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"📊 宏观序参量图已保存至: {out_path}")

def plot_micro_distributions(csv_path="data/naming_game_dist.csv", out_path="output/naming_game_micro.png"):
    if not os.path.exists(csv_path):
        print(f"⚠️ 文件未找到: {csv_path}，跳过微观分布绘图。")
        return

    df = pd.read_csv(csv_path)
    fig, axes = plt.subplots(1, 3, figsize=(16, 4.5))

    # 1. ER 随机图 (指数分布检验)
    ax1 = axes[0]
    sub_er = df[(df["topology"] == "ErdosRenyi") & (df["prob"] > 0)]
    if not sub_er.empty:
        x = sub_er["inventory_size"].values
        y = sub_er["prob"].values
        ax1.semilogy(x, y, "o-", color="#d62728", lw=2, markersize=6, label="Simulation Data")
        
        # 指数拟合 ln(P) = -c * n + b
        if len(x) >= 3:
            fit = np.polyfit(x[:min(len(x), 8)], np.log(y[:min(len(x), 8)]), 1)
            x_fit = np.linspace(x.min(), x.max(), 50)
            y_fit = np.exp(fit[1]) * np.exp(fit[0] * x_fit)
            ax1.semilogy(x_fit, y_fit, "--", color="#333333", lw=1.5, label=f"Exponential fit: $\\propto e^{{{fit[0]:.2f}n}}$")

        ax1.set_title("ER Random Graph (Homogeneous)\n$\\mathcal{P}_n \\propto e^{-c n}$ (Exponential)", fontsize=11, fontweight="bold")
        ax1.set_xlabel("Inventory size $n$", fontsize=11)
        ax1.set_ylabel("$\\mathcal{P}_n(k|t)$ (Log Scale)", fontsize=11)
        ax1.legend(framealpha=0.9, fontsize=9)
        ax1.grid(True, alpha=0.3)

    # 2. BA 无标度网络 (Hub 节点半正态高斯衰减)
    ax2 = axes[1]
    sub_ba_hubs = df[(df["topology"] == "BarabasiAlbert") & (df["group"] == "Hubs") & (df["prob"] > 0)]
    sub_ba_reg = df[(df["topology"] == "BarabasiAlbert") & (df["group"] == "Regular") & (df["prob"] > 0)]
    
    if not sub_ba_hubs.empty:
        xh = sub_ba_hubs["inventory_size"].values
        yh = sub_ba_hubs["prob"].values
        ax2.semilogy(xh, yh, "s-", color="#9467bd", lw=2, markersize=6, label="Hub nodes ($k \\gg \\langle k \\rangle$)")

        # 半正态高斯拟合: ln(P) = -n^2 / (2C) + const
        if len(xh) >= 3:
            fit_g = np.polyfit(xh**2, np.log(yh), 1)
            xh_fit = np.linspace(xh.min(), xh.max(), 50)
            yh_fit = np.exp(fit_g[1]) * np.exp(fit_g[0] * (xh_fit**2))
            ax2.semilogy(xh_fit, yh_fit, "--", color="#111111", lw=1.5, label="Half-Normal fit: $\\propto e^{-n^2 / 2C}$")

    if not sub_ba_reg.empty:
        xr = sub_ba_reg["inventory_size"].values
        yr = sub_ba_reg["prob"].values
        ax2.semilogy(xr, yr, "^--", color="#8c564b", lw=1.5, markersize=5, label="Regular nodes")

    ax2.set_title("BA Scale-Free Network (Heterogeneous)\nHubs exhibit Half-Normal $\\mathcal{P}_n \\propto e^{-\\frac{n^2}{2C(t)}}$", fontsize=11, fontweight="bold")
    ax2.set_xlabel("Inventory size $n$", fontsize=11)
    ax2.set_ylabel("$\\mathcal{P}_n(k|t)$ (Log Scale)", fontsize=11)
    ax2.legend(framealpha=0.9, fontsize=9)
    ax2.grid(True, alpha=0.3)

    # 3. 完全图 (Mean-field 峰值与衰减)
    ax3 = axes[2]
    sub_mf = df[(df["topology"] == "Complete") & (df["prob"] > 0)]
    if not sub_mf.empty:
        xm = sub_mf["inventory_size"].values
        ym = sub_mf["prob"].values
        ax3.plot(xm, ym, "D-", color="#17becf", lw=2, markersize=6, label="Complete Graph $K_{400}$")

        ax3.set_title("Complete Graph (Mean-Field)\nSuperposition of Peak at $\\sim\\sqrt{N}$ & Tail", fontsize=11, fontweight="bold")
        ax3.set_xlabel("Inventory size $n$", fontsize=11)
        ax3.set_ylabel("$\\mathcal{P}_n(t)$", fontsize=11)
        ax3.legend(framealpha=0.9, fontsize=9)
        ax3.grid(True, alpha=0.3)

    plt.suptitle("Fig 3 & 5: Microscopic Inventory Size Distributions $\\mathcal{P}_n(k|t)$ (Dall'Asta & Baronchelli 2006)", fontsize=13, fontweight="bold", y=1.02)
    plt.tight_layout()
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    plt.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"📊 微观分布图已保存至: {out_path}")

if __name__ == "__main__":
    plot_macro_dynamics()
    plot_micro_distributions()
