#!/usr/bin/env python3
"""
Visualization script for Swarmalator simulations.
Plots final spatial and phase configurations, plus order parameter time series.
"""

import argparse
import os
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd


def plot_simulation(traj_path: str, metrics_path: str, output_dir: str, model: str):
    os.makedirs(output_dir, exist_ok=True)

    # 1. Plot Order Parameter Metrics Time Series
    if os.path.exists(metrics_path):
        df_metrics = pd.read_csv(metrics_path)
        plt.figure(figsize=(9, 5))
        plt.plot(df_metrics['time'], df_metrics['order_r'], label='Phase Coherence $R(t)$', color='crimson', lw=2)
        
        if 'order_s' in df_metrics.columns and df_metrics['order_s'].notna().any():
            plt.plot(df_metrics['time'], df_metrics['order_s'], label='Spatial Order $S(t)$', color='royalblue', lw=2)
        if 'order_s_plus' in df_metrics.columns and df_metrics['order_s_plus'].notna().any():
            plt.plot(df_metrics['time'], df_metrics['order_s_plus'], label='$S_+(t)$', color='darkorange', lw=1.5, ls='--')
        if 'order_s_minus' in df_metrics.columns and df_metrics['order_s_minus'].notna().any():
            plt.plot(df_metrics['time'], df_metrics['order_s_minus'], label='$S_-(t)$', color='purple', lw=1.5, ls=':')

        plt.title(f"Swarmalator Order Parameters Evolution ({model.upper()})", fontsize=14, pad=12)
        plt.xlabel("Time $t$", fontsize=12)
        plt.ylabel("Order Parameter Value", fontsize=12)
        plt.ylim(-0.05, 1.05)
        plt.grid(True, linestyle='--', alpha=0.5)
        plt.legend(frameon=True, fontsize=11)
        plt.tight_layout()
        metric_out = os.path.join(output_dir, "metrics_evolution.png")
        plt.savefig(metric_out, dpi=300)
        plt.close()
        print(f" Saved metrics plot: {metric_out}")

    # 2. Plot Final State Snapshot
    if os.path.exists(traj_path):
        df_traj = pd.read_csv(traj_path)
        last_step = df_traj['step'].max()
        df_final = df_traj[df_traj['step'] == last_step]

        plt.figure(figsize=(7, 6))
        if model.lower() == '2d':
            scatter = plt.scatter(
                df_final['c1'],
                df_final['c2'],
                c=df_final['phase'],
                cmap='hsv',
                vmin=-np.pi,
                vmax=np.pi,
                s=60,
                edgecolors='black',
                linewidth=0.5,
                alpha=0.9,
            )
            cbar = plt.colorbar(scatter)
            cbar.set_label("Phase $\\theta$", fontsize=12)
            cbar.set_ticks([-np.pi, -np.pi/2, 0, np.pi/2, np.pi])
            cbar.set_ticklabels(['$-\\pi$', '$-\\pi/2$', '$0$', '$\\pi/2$', '$\\pi$'])
            plt.title(f"2D Swarmalators State at t={df_final['time'].iloc[0]:.2f}", fontsize=13)
            plt.xlabel("Position $x$", fontsize=12)
            plt.ylabel("Position $y$", fontsize=12)
            plt.axis('equal')
            plt.grid(True, linestyle=':', alpha=0.6)
        else: # Ring model
            # Scatter of phi vs theta
            scatter = plt.scatter(
                df_final['c1'],
                df_final['phase'],
                c=df_final['phase'],
                cmap='hsv',
                vmin=-np.pi,
                vmax=np.pi,
                s=50,
                edgecolors='black',
                linewidth=0.5,
            )
            cbar = plt.colorbar(scatter)
            cbar.set_label("Phase $\\theta$", fontsize=12)
            cbar.set_ticks([-np.pi, 0, np.pi])
            cbar.set_ticklabels(['$-\\pi$', '$0$', '$\\pi$'])
            plt.title(f"1D Ring Swarmalators $(\\phi, \\theta)$ at t={df_final['time'].iloc[0]:.2f}", fontsize=13)
            plt.xlabel("Position $\\phi$", fontsize=12)
            plt.ylabel("Phase $\\theta$", fontsize=12)
            plt.xlim(-np.pi, np.pi)
            plt.ylim(-np.pi, np.pi)
            plt.grid(True, linestyle=':', alpha=0.6)

        plt.tight_layout()
        state_out = os.path.join(output_dir, "final_state.png")
        plt.savefig(state_out, dpi=300)
        plt.close()
        print(f" Saved state snapshot plot: {state_out}")


def main():
    parser = argparse.ArgumentParser(description="Plot Swarmalator simulation results")
    parser.add_argument("--traj", default="data/trajectory.csv", help="Path to trajectory CSV")
    parser.add_argument("--metrics", default="data/metrics.csv", help="Path to metrics CSV")
    parser.add_argument("--outdir", default="output", help="Output directory for plots")
    parser.add_argument("--model", default="2d", choices=["2d", "ring"], help="Model type")
    args = parser.parse_args()

    plot_simulation(args.traj, args.metrics, args.outdir, args.model)


if __name__ == "__main__":
    main()
