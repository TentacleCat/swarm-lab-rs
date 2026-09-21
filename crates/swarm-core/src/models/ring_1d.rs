use crate::metrics::{
    kuramoto_order_parameter, ring_spatial_order_parameter, ring_spatiotemporal_order_parameters,
};
use crate::types::{wrap_to_pi, DynamicalSystem, MetricsSnapshot};
use rand::Rng;
use rand_distr::{Distribution, Uniform};
use rayon::prelude::*;

/// 1D Swarmalator model on a Ring (PRE 2018 / PRE 2022)
/// Positions phi_i in [-pi, pi), phases theta_i in [-pi, pi)
#[derive(Clone, Debug)]
pub struct SwarmalatorRing1D {
    pub n: usize,
    /// Spatial coupling parameter J
    pub j: f64,
    /// Phase coupling parameter K
    pub k: f64,
    /// Natural spatial drift velocities nu_i
    pub nu: Vec<f64>,
    /// Natural frequencies omega_i
    pub omega: Vec<f64>,
}

impl SwarmalatorRing1D {
    pub fn new(n: usize, j: f64, k: f64) -> Self {
        Self {
            n,
            j,
            k,
            nu: vec![0.0; n],
            omega: vec![0.0; n],
        }
    }

    pub fn with_drifts_and_frequencies(mut self, nu: Vec<f64>, omega: Vec<f64>) -> Self {
        assert_eq!(nu.len(), self.n);
        assert_eq!(omega.len(), self.n);
        self.nu = nu;
        self.omega = omega;
        self
    }

    /// Random uniform initial conditions for phi and theta in [-pi, pi)
    pub fn random_initial_state<R: Rng>(&self, rng: &mut R) -> Vec<f64> {
        let dist = Uniform::new(-std::f64::consts::PI, std::f64::consts::PI);
        let mut state = vec![0.0; 2 * self.n];
        for i in 0..(2 * self.n) {
            state[i] = dist.sample(rng);
        }
        state
    }
}

impl DynamicalSystem for SwarmalatorRing1D {
    fn dimension(&self) -> usize {
        2 * self.n
    }

    fn num_agents(&self) -> usize {
        self.n
    }

    fn derivative(&self, s: &[f64], ds: &mut [f64]) {
        let n = self.n;
        let phi = &s[0..n];
        let theta = &s[n..2 * n];

        let inv_n = 1.0 / (n as f64);
        let j_param = self.j;
        let k_param = self.k;
        let nu = &self.nu;
        let omega = &self.omega;

        let (dphi, dtheta) = ds.split_at_mut(n);

        (0..n).into_par_iter().map(|i| {
            let phii = phi[i];
            let thi = theta[i];

            let mut sum_phi = 0.0;
            let mut sum_th = 0.0;

            for j in 0..n {
                let dphi_ji = phi[j] - phii;
                let dth_ji = theta[j] - thi;

                // dot(phi_i) force: sin(phi_j - phi_i) * (1 + J * cos(theta_j - theta_i))
                sum_phi += dphi_ji.sin() * (1.0 + j_param * dth_ji.cos());

                // dot(theta_i) force: sin(theta_j - theta_i) * (1 + J * cos(phi_j - phi_i))
                sum_th += dth_ji.sin() * (1.0 + j_param * dphi_ji.cos());
            }

            let dphi_i = nu[i] + inv_n * sum_phi;
            let dth_i = omega[i] + (k_param * inv_n) * sum_th;

            (dphi_i, dth_i)
        }).collect::<Vec<_>>()
        .into_iter()
        .enumerate()
        .for_each(|(i, (dp, dt))| {
            dphi[i] = dp;
            dtheta[i] = dt;
        });
    }

    fn metrics(&self, s: &[f64]) -> MetricsSnapshot {
        let n = self.n;
        let phi = &s[0..n];
        let theta = &s[n..2 * n];

        let r = kuramoto_order_parameter(theta);
        let s_order = ring_spatial_order_parameter(phi);
        let (s_plus, s_minus) = ring_spatiotemporal_order_parameters(phi, theta);

        MetricsSnapshot {
            t: 0.0,
            order_r: r,
            order_s: Some(s_order),
            order_s_plus: Some(s_plus),
            order_s_minus: Some(s_minus),
        }
    }

    fn post_step(&self, s: &mut [f64]) {
        for val in s.iter_mut() {
            *val = wrap_to_pi(*val);
        }
    }
}
