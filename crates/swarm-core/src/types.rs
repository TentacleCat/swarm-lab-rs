use serde::{Deserialize, Serialize};

/// Trait representing a dynamical swarmalator system
pub trait DynamicalSystem: Send + Sync {
    /// Dimension of the flattened state vector
    fn dimension(&self) -> usize;

    /// Number of swarmalator agents
    fn num_agents(&self) -> usize;

    /// Compute time derivative ds/dt at state s
    fn derivative(&self, s: &[f64], ds: &mut [f64]);

    /// Compute order parameters / diagnostic metrics at current state
    fn metrics(&self, s: &[f64]) -> MetricsSnapshot;

    /// Normalize periodic angles or wrap coordinates if applicable
    fn post_step(&self, _s: &mut [f64]) {}
}

/// Snapshot of diagnostic order parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricsSnapshot {
    /// Simulation time
    pub t: f64,
    /// Kuramoto phase coherence order parameter R in [0, 1]
    pub order_r: f64,
    /// Spatial order parameter S in [0, 1] (if applicable)
    pub order_s: Option<f64>,
    /// Spatio-temporal order parameter S_+ in [0, 1]
    pub order_s_plus: Option<f64>,
    /// Spatio-temporal order parameter S_- in [0, 1]
    pub order_s_minus: Option<f64>,
}

/// Helper to wrap an angle to [-pi, pi)
#[inline]
pub fn wrap_to_pi(angle: f64) -> f64 {
    let pi = std::f64::consts::PI;
    let two_pi = 2.0 * pi;
    let mut a = angle % two_pi;
    if a >= pi {
        a -= two_pi;
    } else if a < -pi {
        a += two_pi;
    }
    a
}
