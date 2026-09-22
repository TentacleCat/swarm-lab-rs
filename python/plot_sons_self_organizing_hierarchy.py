#!/usr/bin/env python3
"""
Scientific plotting script for:
"Self-organizing nervous systems for robot swarms" (arXiv:2401.13103 / Science Robotics 2024)
Focus: Section 4.1 SoNS Control & Section 4.2 Analysis Metrics

Reproduces:
1. Multi-stage spatial evolution of heterogeneous SoNS swarm (Initial -> Tree Self-Organization -> Final Morphology)
2. Equation (1) Position tracking error E(t) vs Equation (2) Theoretical lower bound B(t)
3. Dynamic merge of independent swarms and subtree hierarchy scale aggregation
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

COLOR_DRONE = "#1f77b4"      # Blue: Aerial Quadrotor Drone
COLOR_GROUND = "#2ca02c"     # Green: Ground Robot (Pi-puck / e-puck)
COLOR_BRAIN = "#d62728"      # Red: Root Brain
COLOR_ERROR = "#ff7f0e"      # Orange: Tracking Error E(t)
COLOR_BOUND = "#7f7f7f"      # Gray dashed: Lower Bound B(t)
COLOR_TREE_LINK = "#8c564b"  # Brown: Hierarchy Link

def plot_sons_hierarchy_lab(
    csv_path="data/sons_self_organizing_hierarchy.csv",
    out_path="output/sons_self_organizing_hierarchy.png",
):
    if not os.path.exists(csv_path):
        print(f"⚠️ Data file not found: {csv_path}, skipping plot generation.")
        return

    df = pd.read_csv(csv_path)
    os.makedirs(os.path.dirname(out_path), exist_ok=True)

    fig = plt.figure(figsize=(16, 10), constrained_layout=True)
    gs = fig.add_gridspec(2, 3)

    # -------------------------------------------------------------
    # Row 1: Spatial snapshots at 3 key time points
    # -------------------------------------------------------------
    unique_steps = sorted(df["step"].unique())
    selected_steps = [
        unique_steps[0],
        unique_steps[len(unique_steps) // 2],
        unique_steps[-1],
    ]
    step_titles = [
        f"(a) Initial Scattered State (t={df[df['step']==selected_steps[0]]['time'].iloc[0]:.2f}s)",
        f"(b) Self-Organized Hierarchy (t={df[df['step']==selected_steps[1]]['time'].iloc[0]:.2f}s)",
        f"(c) Converged Stable Morphology (t={df[df['step']==selected_steps[2]]['time'].iloc[0]:.2f}s)",
    ]

    for col_idx, s in enumerate(selected_steps):
        ax = fig.add_subplot(gs[0, col_idx])
        sub = df[df["step"] == s]

        # Draw obstacles
        obstacles = [[0.4, 0.9], [-0.5, -0.8]]
        for ox, oy in obstacles:
            circ = Circle((ox, oy), 0.15, color="#d62728", alpha=0.3, hatch="//")
            ax.add_patch(circ)
            ax.text(ox, oy, "Obs", color="#900", fontsize=8, ha="center", va="center", weight="bold")

        # Draw hierarchy links
        robot_dict = {row["robot_id"]: (row["pos_x"], row["pos_y"]) for _, row in sub.iterrows()}
        for _, row in sub.iterrows():
            pid = int(row["parent_id"])
            if pid >= 0 and pid in robot_dict:
                px, py = robot_dict[pid]
                cx, cy = row["pos_x"], row["pos_y"]
                ax.plot([px, cx], [py, cy], color=COLOR_TREE_LINK, linestyle="-", linewidth=1.5, alpha=0.7, zorder=2)

        # Draw robots
        for _, row in sub.iterrows():
            rid = int(row["robot_id"])
            rx, ry = row["pos_x"], row["pos_y"]
            is_brain = (row["brain_id"] == rid and row["parent_id"] < 0)

            if row["robot_type"] == "Drone":
                marker = "D"
                color = COLOR_BRAIN if is_brain else COLOR_DRONE
                size = 120 if is_brain else 90
                label = f"Brain D{rid}" if is_brain else f"D{rid}"
            else:
                marker = "o"
                color = COLOR_GROUND
                size = 70
                label = f"R{rid}"

            ax.scatter(rx, ry, s=size, c=color, marker=marker, edgecolors="black", linewidths=1.2, zorder=5)
            ax.text(rx + 0.05, ry + 0.05, label, fontsize=8, weight="bold", color="#222", zorder=6)

        ax.set_title(step_titles[col_idx], fontsize=12, pad=10)
        ax.set_xlabel("X Position (m)", fontsize=10)
        ax.set_ylabel("Y Position (m)", fontsize=10)
        ax.set_xlim(-2.2, 2.2)
        ax.set_ylim(-2.2, 2.2)
        ax.set_aspect("equal")
        ax.grid(True, linestyle="--", alpha=0.5)

    # -------------------------------------------------------------
    # Row 2, Col 0: Tracking Error E(t) vs Theoretical Lower Bound B(t)
    # -------------------------------------------------------------
    ax_err = fig.add_subplot(gs[1, 0])
    step_metrics = df.drop_duplicates(subset=["step"]).sort_values("step")

    ax_err.plot(
        step_metrics["time"],
        step_metrics["tracking_error"],
        color=COLOR_ERROR,
        linewidth=2.2,
        label=r"Tracking Error $E(t)$ (Eq. 1)",
    )
    ax_err.plot(
        step_metrics["time"],
        step_metrics["theoretical_lower_bound"],
        color=COLOR_BOUND,
        linestyle="--",
        linewidth=2.0,
        label=r"Lower Bound $B(t)$ (Eq. 2)",
    )

    ax_err.set_title("(d) Tracking Error & Lower Bound Convergence", fontsize=12)
    ax_err.set_xlabel("Time (s)", fontsize=10)
    ax_err.set_ylabel("Error Metric (m)", fontsize=10)
    ax_err.legend(loc="upper right", frameon=True, fontsize=9)
    ax_err.grid(True, linestyle="--", alpha=0.5)

    # -------------------------------------------------------------
    # Row 2, Col 1: Hierarchy Tree Merging & Scale Growth
    # -------------------------------------------------------------
    ax_tree = fig.add_subplot(gs[1, 1])
    ax_tree.plot(
        step_metrics["time"],
        step_metrics["num_swarms"],
        color="#9467bd",
        linewidth=2.0,
        marker="s",
        markevery=30,
        label="Independent SoNS Swarms",
    )
    ax_tree.plot(
        step_metrics["time"],
        step_metrics["max_swarm_size"],
        color="#1f77b4",
        linewidth=2.2,
        label="Dominant SoNS Size (Robots)",
    )
    ax_tree.plot(
        step_metrics["time"],
        step_metrics["max_depth"],
        color="#8c564b",
        linewidth=1.8,
        linestyle=":",
        label="Hierarchy Depth",
    )

    ax_tree.set_title("(e) Swarm Merging & Hierarchy Scale Evolution", fontsize=12)
    ax_tree.set_xlabel("Time (s)", fontsize=10)
    ax_tree.set_ylabel("Count / Depth", fontsize=10)
    ax_tree.legend(loc="center right", frameon=True, fontsize=9)
    ax_tree.grid(True, linestyle="--", alpha=0.5)

    # -------------------------------------------------------------
    # Row 2, Col 2: Robot Velocity Distribution
    # -------------------------------------------------------------
    ax_vel = fig.add_subplot(gs[1, 2])
    df["speed"] = np.sqrt(df["vel_x"] ** 2 + df["vel_y"] ** 2)

    drone_data = df[df["robot_type"] == "Drone"]
    ground_data = df[df["robot_type"] == "Ground"]

    drone_mean_speed = drone_data.groupby("time")["speed"].mean()
    ground_mean_speed = ground_data.groupby("time")["speed"].mean()

    ax_vel.plot(drone_mean_speed.index, drone_mean_speed.values, color=COLOR_DRONE, linewidth=2.0, label="Drones Mean Speed")
    ax_vel.plot(ground_mean_speed.index, ground_mean_speed.values, color=COLOR_GROUND, linewidth=2.0, label="Ground Robots Mean Speed")

    ax_vel.set_title("(f) Kinematic Velocity Profiles", fontsize=12)
    ax_vel.set_xlabel("Time (s)", fontsize=10)
    ax_vel.set_ylabel("Linear Speed (m/s)", fontsize=10)
    ax_vel.legend(loc="upper right", frameon=True, fontsize=9)
    ax_vel.grid(True, linestyle="--", alpha=0.5)

    plt.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close()
    print(f"✅ Scientific plot successfully saved to: {out_path}")

if __name__ == "__main__":
    plot_sons_hierarchy_lab()
