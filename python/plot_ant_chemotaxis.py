#!/usr/bin/env python3
"""
Scientific plotting script for:
"Modeling ant foraging: a chemotaxis approach with pheromones and trail formation" (arXiv:1409.3808)

Reproduces:
1. Spatial 2D heatmaps of the 4 coupled fields (u: foraging ants, w: returning ants, v: pheromone, c: food)
2. Trail emergence and dissipation upon food exhaustion across snapshots
3. Foraging efficiency comparison (Trail vs Non-trail parameter regimes)
"""

import os
import glob
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.colors import LinearSegmentedColormap

# Matplotlib 样式设置
plt.style.use("seaborn-v0_8-whitegrid" if "seaborn-v0_8-whitegrid" in plt.style.available else "default")
plt.rcParams["font.sans-serif"] = ["DejaVu Sans", "Arial", "Helvetica"]
plt.rcParams["axes.edgecolor"] = "#333333"
plt.rcParams["axes.linewidth"] = 1.0

# 自定义连续科学配色
CMAP_ANTS = plt.cm.YlOrRd
CMAP_PHERO = plt.cm.viridis
CMAP_FOOD = plt.cm.Greens

def plot_spatial_fields(snapshot_path="data/ant_chemotaxis_snapshot_t12.csv", out_path="output/ant_chemotaxis_spatial_fields.png"):
    """绘制成熟蚁道阶段 (t=12) 的 4 场空间分布图 (复现论文 Fig 1-3)"""
    if not os.path.exists(snapshot_path):
        candidates = sorted(glob.glob("data/ant_chemotaxis_snapshot_t*.csv"))
        if not candidates:
            print(f"⚠️ 未找到快照文件，跳过空间分布图绘制。")
            return
        snapshot_path = candidates[-1]

    df = pd.read_csv(snapshot_path)
    x_unique = np.sort(df["x"].unique())
    y_unique = np.sort(df["y"].unique())
    nx = len(x_unique)
    ny = len(y_unique)

    extent = [x_unique.min(), x_unique.max(), y_unique.min(), y_unique.max()]

    U = df["u"].values.reshape(ny, nx)
    W = df["w"].values.reshape(ny, nx)
    V = df["v"].values.reshape(ny, nx)
    C = df["c"].values.reshape(ny, nx)

    fig, axes = plt.subplots(2, 2, figsize=(11, 10))

    # 1. 觅食蚁密度 u(x, y)
    im0 = axes[0, 0].imshow(U, origin="lower", extent=extent, cmap=CMAP_ANTS, aspect="equal")
    axes[0, 0].set_title(r"Foraging Ants $u(t, x, y)$ (Emergence of Trails)", fontsize=12, fontweight="bold")
    axes[0, 0].plot(0, 0, "k*", markersize=12, label="Nest")
    fig.colorbar(im0, ax=axes[0, 0], shrink=0.8, pad=0.03)

    # 2. 搬运蚁密度 w(x, y)
    im1 = axes[0, 1].imshow(W, origin="lower", extent=extent, cmap=plt.cm.magma, aspect="equal")
    axes[0, 1].set_title(r"Returning Ants $w(t, x, y)$ (Nest-Bound Flow)", fontsize=12, fontweight="bold")
    axes[0, 1].plot(0, 0, "w*", markersize=12, label="Nest")
    fig.colorbar(im1, ax=axes[0, 1], shrink=0.8, pad=0.03)

    # 3. 信息素浓度 v(x, y)
    im2 = axes[1, 0].imshow(V, origin="lower", extent=extent, cmap=CMAP_PHERO, aspect="equal")
    axes[1, 0].set_title(r"Pheromone Field $v(t, x, y)$ (Chemical Highway)", fontsize=12, fontweight="bold")
    axes[1, 0].plot(0, 0, "r*", markersize=12, label="Nest")
    fig.colorbar(im2, ax=axes[1, 0], shrink=0.8, pad=0.03)

    # 4. 食物源 c(x, y)
    im3 = axes[1, 1].imshow(C, origin="lower", extent=extent, cmap=CMAP_FOOD, aspect="equal")
    axes[1, 1].set_title(r"Food Concentration $c(t, x, y)$ (Resource Depletion)", fontsize=12, fontweight="bold")
    axes[1, 1].plot(0, 0, "k*", markersize=12, label="Nest")
    fig.colorbar(im3, ax=axes[1, 1], shrink=0.8, pad=0.03)

    for ax in axes.flat:
        ax.set_xlabel("x (dimensionless)", fontsize=10)
        ax.set_ylabel("y (dimensionless)", fontsize=10)
        ax.legend(loc="upper right", framealpha=0.8, fontsize=9)

    plt.suptitle("PDE Ant Foraging: Spatial Field Distributions with Dual Food Sources (arXiv:1409.3808)", fontsize=14, fontweight="bold", y=0.98)
    plt.tight_layout()
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    plt.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"📊 4 场空间分布图已保存至: {out_path}")

def plot_trails_evolution(out_path="output/ant_chemotaxis_trails_evolution.png"):
    """展示从早期探索 (t=5)、蚁道贯通 (t=12) 到小食物源枯竭消散 (t=24) 的演化过程"""
    time_points = [5.0, 12.0, 24.0]
    files = [f"data/ant_chemotaxis_snapshot_t{int(t)}.csv" for t in time_points]

    valid = [f for f in files if os.path.exists(f)]
    if len(valid) < 2:
        print("⚠️ 快照数量不足，跳过时空演化对比图绘制。")
        return

    fig, axes = plt.subplots(2, len(valid), figsize=(4.5 * len(valid), 8.5), sharex=True, sharey=True)

    for col_idx, fpath in enumerate(valid):
        df = pd.read_csv(fpath)
        x_u = np.sort(df["x"].unique())
        y_u = np.sort(df["y"].unique())
        nx, ny = len(x_u), len(y_u)
        extent = [x_u.min(), x_u.max(), y_u.min(), y_u.max()]

        U = df["u"].values.reshape(ny, nx)
        V = df["v"].values.reshape(ny, nx)
        C = df["c"].values.reshape(ny, nx)

        t_val = time_points[col_idx]

        # 第一行: 觅食蚁密度 + 食物等高线
        ax_top = axes[0, col_idx] if len(valid) > 1 else axes[0]
        im_u = ax_top.imshow(U, origin="lower", extent=extent, cmap=CMAP_ANTS, aspect="equal")
        ax_top.contour(C, levels=[0.5, 2.0, 5.0], extent=extent, colors="green", linewidths=1.2)
        ax_top.plot(0, 0, "k*", markersize=10)
        ax_top.set_title(f"t = {t_val:.1f}: Foraging Ants $u$", fontsize=11, fontweight="bold")
        if col_idx == 0:
            ax_top.set_ylabel("y (dimensionless)", fontsize=10)

        # 第二行: 信息素浓度场
        ax_bot = axes[1, col_idx] if len(valid) > 1 else axes[1]
        im_v = ax_bot.imshow(V, origin="lower", extent=extent, cmap=CMAP_PHERO, aspect="equal")
        ax_bot.plot(0, 0, "r*", markersize=10)
        ax_bot.set_title(f"t = {t_val:.1f}: Pheromone $v$", fontsize=11, fontweight="bold")
        ax_bot.set_xlabel("x (dimensionless)", fontsize=10)
        if col_idx == 0:
            ax_bot.set_ylabel("y (dimensionless)", fontsize=10)

    plt.suptitle("Spatiotemporal Evolution: Recruitment, Trail Emergence, and Food Depletion Fading", fontsize=13, fontweight="bold", y=0.99)
    plt.tight_layout()
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    plt.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"📊 蚁道时空演化对比图已保存至: {out_path}")

def plot_efficiency(timeseries_path="data/ant_chemotaxis_timeseries.csv",
                    eff_path="data/ant_chemotaxis_efficiency.csv",
                    out_path="output/ant_chemotaxis_efficiency.png"):
    """绘制群体宏观时序与不同参数下食物消耗效率曲线 (复现 Section 5 核心发现)"""
    if not os.path.exists(timeseries_path):
        print(f"⚠️ 未找到时间序列文件 {timeseries_path}，跳过效率曲线绘制。")
        return

    df_ts = pd.read_csv(timeseries_path)

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 5.2))

    # 1. 宏观质量与信息素演化
    ax1.plot(df_ts["time"], df_ts["total_food"], label="Total Food $C(t)$", color="#2ca02c", lw=2.2)
    ax1.plot(df_ts["time"], df_ts["foraging_mass"], label="Foraging Ants $U(t)$", color="#ff7f0e", lw=1.8)
    ax1.plot(df_ts["time"], df_ts["returning_mass"], label="Returning Ants $W(t)$", color="#d62728", lw=1.8)
    ax1.set_xlabel("Time $t$ (dimensionless)", fontsize=11)
    ax1.set_ylabel("Total Mass / Density Integral", fontsize=11)
    ax1.set_title("Population & Food Mass Evolution", fontsize=12, fontweight="bold")
    ax1.grid(True, alpha=0.3)
    ax1.legend(loc="center right", framealpha=0.9, fontsize=9.5)

    # 2. 觅食效率对比 (Trail vs Non-Trail)
    if os.path.exists(eff_path):
        df_eff = pd.read_csv(eff_path)
        ax2.plot(df_eff["time"], df_eff["standard_trail"] * 100.0,
                 label=r"Optimal Trail ($\chi_u = 60, \varepsilon = 0.5$)", color="#1f77b4", lw=2.5)
        ax2.plot(df_eff["time"], df_eff["low_sensitivity"] * 100.0,
                 label=r"Low Sensitivity ($\chi_u = 4.0$, No Trails)", color="#7f7f7f", linestyle="--", lw=2.0)
        ax2.plot(df_eff["time"], df_eff["high_evaporation"] * 100.0,
                 label=r"Fast Evaporation ($\varepsilon = 3.5$, Broken Trails)", color="#e377c2", linestyle=":", lw=2.0)
        ax2.set_xlabel("Time $t$ (dimensionless)", fontsize=11)
        ax2.set_ylabel("Food Removed (%)", fontsize=11)
        ax2.set_title("Food Removal Efficiency: Trail vs Non-Trail Regimes", fontsize=12, fontweight="bold")
        ax2.grid(True, alpha=0.3)
        ax2.legend(loc="lower right", framealpha=0.9, fontsize=9.5)
    else:
        ax2.plot(df_ts["time"], df_ts["depletion_ratio"] * 100.0, color="#1f77b4", lw=2.2)
        ax2.set_title("Cumulative Food Removal (%)", fontsize=12, fontweight="bold")

    plt.suptitle("arXiv:1409.3808: Trail Formation Significantly Accelerates Food Removal Efficiency", fontsize=13, fontweight="bold", y=1.02)
    plt.tight_layout()
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    plt.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"📊 觅食效率与宏观动力学曲线已保存至: {out_path}")

if __name__ == "__main__":
    plot_spatial_fields()
    plot_trails_evolution()
    plot_efficiency()
