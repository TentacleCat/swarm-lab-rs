use super::types::{vec2_length, vec2_sub, RobotNode, Vec2};
use serde::{Deserialize, Serialize};

/// Metrics snapshot for evaluating SoNS swarm performance according to paper Section 4.2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoNSMetrics {
    /// Time elapsed (seconds)
    pub time: f64,
    /// Mean position tracking error E (Equation 1 in Section 4.2)
    pub tracking_error: f64,
    /// Theoretical lower bound B of position tracking error (Equation 2 in Section 4.2)
    pub theoretical_lower_bound: f64,
    /// Maximum hierarchy depth in the dominant SoNS tree
    pub max_depth: usize,
    /// Number of independent SoNS trees currently existing
    pub num_swarms: usize,
    /// Number of robots in the largest SoNS tree
    pub max_swarm_size: usize,
}

/// Compute position tracking error E according to Equation (1) of Section 4.2:
///
/// E = 1/n * sum_{i=1}^n | d(p_i - p_1) - d(f_i - f_1) |
///
/// where p_1 is the brain's current position, f_1 is the brain's target position,
/// p_i is robot i's position, and f_i is robot i's target position.
/// For the brain (i=1), E_1 is identically 0.
pub fn compute_tracking_error(
    robots: &[RobotNode],
    brain_id: usize,
    target_world_positions: &[Option<Vec2>],
) -> f64 {
    let n = robots.len();
    if n == 0 {
        return 0.0;
    }

    let p1 = robots[brain_id].position;
    // Brain target is always its own current position in relative tracking
    let f1 = target_world_positions[brain_id].unwrap_or(p1);

    let mut sum_error = 0.0;
    let mut count = 0;

    for (i, robot) in robots.iter().enumerate() {
        if i == brain_id {
            count += 1;
            continue; // E_1 = 0
        }

        if let Some(fi) = target_world_positions[i] {
            let actual_dist = vec2_length(vec2_sub(robot.position, p1));
            let target_dist = vec2_length(vec2_sub(fi, f1));
            let ei = (actual_dist - target_dist).abs();
            sum_error += ei;
            count += 1;
        }
    }

    if count > 0 {
        sum_error / (count as f64)
    } else {
        0.0
    }
}

/// Compute theoretical lower bound B(t) according to Equation (2) of Section 4.2:
///
/// B = 1/n * sum_{i=1}^n max(0, |d(p_{eps, i} - f_{eps, i})| - kappa_i * (t - t_eps))
///
/// where p_{eps, i} is start position when target was set, f_{eps, i} is target position,
/// kappa_i is max speed, and (t - t_eps) is elapsed time.
pub fn compute_theoretical_lower_bound(
    start_positions: &[Vec2],
    target_positions: &[Option<Vec2>],
    max_speeds: &[f64],
    elapsed_time: f64,
) -> f64 {
    let n = start_positions.len();
    if n == 0 {
        return 0.0;
    }

    let mut sum_bound = 0.0;
    let mut count = 0;

    for i in 0..n {
        if let Some(target) = target_positions[i] {
            let initial_dist = vec2_length(vec2_sub(start_positions[i], target));
            let travel_dist = max_speeds[i] * elapsed_time;
            let bi = (initial_dist - travel_dist).max(0.0);
            sum_bound += bi;
            count += 1;
        }
    }

    if count > 0 {
        sum_bound / (count as f64)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sons_hierarchy::types::RobotType;

    #[test]
    fn test_tracking_error_zero_when_perfect() {
        let brain = RobotNode::new(0, RobotType::Drone, [0.0, 0.0], 1.0);
        let follower = RobotNode::new(1, RobotType::Ground, [1.0, 0.0], 0.5);
        let robots = vec![brain, follower];

        let targets = vec![Some([0.0, 0.0]), Some([1.0, 0.0])];
        let err = compute_tracking_error(&robots, 0, &targets);
        assert!(err < 1e-9);
    }

    #[test]
    fn test_theoretical_lower_bound_decreases_linearly() {
        let starts = vec![[0.0, 0.0], [2.0, 0.0]];
        let targets = vec![Some([0.0, 0.0]), Some([0.0, 0.0])];
        let speeds = vec![0.1, 0.2];

        // At t=0, initial dist of robot 1 is 2.0. Mean bound = (0 + 2)/2 = 1.0
        let b0 = compute_theoretical_lower_bound(&starts, &targets, &speeds, 0.0);
        assert!((b0 - 1.0).abs() < 1e-9);

        // At t=5, robot 1 traveled 0.2 * 5 = 1.0. Remaining dist = 1.0. Mean bound = 0.5
        let b5 = compute_theoretical_lower_bound(&starts, &targets, &speeds, 5.0);
        assert!((b5 - 0.5).abs() < 1e-9);

        // At t=10, robot 1 traveled 2.0. Remaining dist = 0.0. Mean bound = 0.0
        let b10 = compute_theoretical_lower_bound(&starts, &targets, &speeds, 10.0);
        assert_eq!(b10, 0.0);
    }
}
