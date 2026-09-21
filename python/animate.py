#!/usr/bin/env python3
"""
Animation script for Swarmalators.
Generates an animated MP4 / GIF showing the evolution of particles in space colored by phase.
"""

import argparse
import os
import matplotlib.animation as animation
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd


def create_animation(traj_path: str, output_path: str, model: str = "2d", fps: int = 30):
    if not os.path.exists(traj_path):
        print(f"Trajectory file {traj_path} not found.")
        return

    df = pd.read_csv(traj_path)
    steps = sorted(df['step'].unique())

    fig, ax = plt.subplots(figsize=(7, 7))

    if model.lower() == "2d":
        # Determine fixed bounds from data
        max_coord = max(df['c1'].abs().max(), df['c2'].abs().max()) * 1.15
        ax.set_xlim(-max_coord, max_coord)
        ax.set_ylim(-max_coord, max_coord)
        ax.set_xlabel("Position $x$", fontsize=12)
        ax.set_ylabel("Position $y$", fontsize=12)
        ax.set_title("2D Swarmalator Dynamic Evolution", fontsize=14)
        ax.grid(True, linestyle=':', alpha=0.5)

        step0 = steps[0]
        df0 = df[df['step'] == step0]
        scatter = ax.scatter(
            df0['c1'],
            df0['c2'],
            c=df0['phase'],
            cmap='hsv',
            vmin=-np.pi,
            vmax=np.pi,
            s=50,
            edgecolors='black',
            linewidth=0.5,
        )
        cbar = plt.colorbar(scatter, ax=ax)
        cbar.set_label("Phase $\\theta$", fontsize=11)
        time_text = ax.text(0.03, 0.95, "", transform=ax.transAxes, fontsize=12,
                            bbox=dict(boxstyle="round,pad=0.3", fc="white", ec="gray", alpha=0.8))

        def update(frame_step):
            sub = df[df['step'] == frame_step]
            coords = np.column_stack([sub['c1'], sub['c2']])
            scatter.set_offsets(coords)
            scatter.set_array(sub['phase'].values)
            time_text.set_text(f"Step: {frame_step} | Time: {sub['time'].iloc[0]:.2f}")
            return scatter, time_text

    else:
        ax.set_xlim(-np.pi, np.pi)
        ax.set_ylim(-np.pi, np.pi)
        ax.set_xlabel("Position $\\phi$", fontsize=12)
        ax.set_ylabel("Phase $\\theta$", fontsize=12)
        ax.set_title("1D Ring Swarmalator Evolution $(\\phi, \\theta)$", fontsize=14)
        ax.grid(True, linestyle=':', alpha=0.5)

        step0 = steps[0]
        df0 = df[df['step'] == step0]
        scatter = ax.scatter(
            df0['c1'],
            df0['phase'],
            c=df0['phase'],
            cmap='hsv',
            vmin=-np.pi,
            vmax=np.pi,
            s=50,
            edgecolors='black',
            linewidth=0.5,
        )
        cbar = plt.colorbar(scatter, ax=ax)
        cbar.set_label("Phase $\\theta$", fontsize=11)
        time_text = ax.text(0.03, 0.95, "", transform=ax.transAxes, fontsize=12,
                            bbox=dict(boxstyle="round,pad=0.3", fc="white", ec="gray", alpha=0.8))

        def update(frame_step):
            sub = df[df['step'] == frame_step]
            coords = np.column_stack([sub['c1'], sub['phase']])
            scatter.set_offsets(coords)
            scatter.set_array(sub['phase'].values)
            time_text.set_text(f"Step: {frame_step} | Time: {sub['time'].iloc[0]:.2f}")
            return scatter, time_text

    ani = animation.FuncAnimation(fig, update, frames=steps, blit=True, interval=1000/fps)
    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    if output_path.endswith(".gif"):
        ani.save(output_path, writer='pillow', fps=fps)
    else:
        ani.save(output_path, writer='ffmpeg', fps=fps)
    plt.close()
    print(f" Animation saved: {output_path}")


def main():
    parser = argparse.ArgumentParser(description="Create Swarmalators animation")
    parser.add_argument("--traj", default="data/trajectory.csv", help="Path to trajectory CSV")
    parser.add_argument("--output", default="output/evolution.gif", help="Output GIF/MP4 file path")
    parser.add_argument("--model", default="2d", choices=["2d", "ring"], help="Model type")
    parser.add_argument("--fps", type=int, default=24, help="Frames per second")
    args = parser.parse_args()

    create_animation(args.traj, args.output, args.model, args.fps)


if __name__ == "__main__":
    main()
