//! [Reference] 2017 Nature Communications 2D 模型标准参考实现

use crate::reference::metrics::{kuramoto_order_parameter, radius_of_gyration_2d};
use crate::types::{DynamicalSystem, MetricsSnapshot, wrap_to_pi};
use rand::Rng;
use rand_distr::{Distribution, Uniform};
use rayon::prelude::*;

#[derive(Clone, Debug)]
pub struct Swarmalator2D {
    pub n: usize,
    pub j: f64,
    pub k: f64,
    pub omega: Vec<f64>,
    pub eps: f64,
}

impl Swarmalator2D {
    pub fn new(n: usize, j: f64, k: f64) -> Self {
        Self {
            n,
            j,
            k,
            omega: vec![0.0; n],
            eps: 1e-5,
        }
    }

    pub fn with_frequencies(mut self, omega: Vec<f64>) -> Self {
        assert_eq!(omega.len(), self.n);
        self.omega = omega;
        self
    }

    pub fn random_initial_state<R: Rng>(&self, rng: &mut R) -> Vec<f64> {
        let pos_dist = Uniform::new(-1.0, 1.0);
        let phase_dist = Uniform::new(-std::f64::consts::PI, std::f64::consts::PI);
        let mut state = vec![0.0; 3 * self.n];

        for i in 0..self.n {
            state[i] = pos_dist.sample(rng);
            state[self.n + i] = pos_dist.sample(rng);
            state[2 * self.n + i] = phase_dist.sample(rng);
        }
        state
    }
}

impl DynamicalSystem for Swarmalator2D {
    fn dimension(&self) -> usize {
        3 * self.n
    }

    fn num_agents(&self) -> usize {
        self.n
    }

    fn derivative(&self, s: &[f64], ds: &mut [f64]) {
        let n = self.n;
        let x = &s[0..n];
        let y = &s[n..2 * n];
        let theta = &s[2 * n..3 * n];

        let inv_n = 1.0 / (n as f64);
        let j_param = self.j;
        let k_param = self.k;
        let eps = self.eps;
        let omega = &self.omega;

        let (dx, rest) = ds.split_at_mut(n);
        let (dy, dtheta) = rest.split_at_mut(n);

        (0..n).into_par_iter().map(|i| {
            let xi = x[i];
            let yi = y[i];
            let thi = theta[i];

            let mut sum_fx = 0.0;
            let mut sum_fy = 0.0;
            let mut sum_fth = 0.0;

            for j in 0..n {
                if i == j {
                    continue;
                }
                let dx_ij = x[j] - xi;
                let dy_ij = y[j] - yi;
                let dth_ij = theta[j] - thi;

                let dist_sq = dx_ij * dx_ij + dy_ij * dy_ij + eps;
                let dist = dist_sq.sqrt();

                let cos_dth = dth_ij.cos();
                let spatial_factor = (1.0 + j_param * cos_dth) / dist - 1.0 / dist_sq;

                sum_fx += dx_ij * spatial_factor;
                sum_fy += dy_ij * spatial_factor;
                sum_fth += (k_param * dth_ij.sin()) / dist;
            }

            let dxi = inv_n * sum_fx;
            let dyi = inv_n * sum_fy;
            let dthi = omega[i] + inv_n * sum_fth;

            (dxi, dyi, dthi)
        }).collect::<Vec<_>>()
        .into_iter()
        .enumerate()
        .for_each(|(i, (dxi, dyi, dthi))| {
            dx[i] = dxi;
            dy[i] = dyi;
            dtheta[i] = dthi;
        });
    }

    fn metrics(&self, s: &[f64]) -> MetricsSnapshot {
        let n = self.n;
        let x = &s[0..n];
        let y = &s[n..2 * n];
        let theta = &s[2 * n..3 * n];

        let r = kuramoto_order_parameter(theta);
        let r_gyr = radius_of_gyration_2d(x, y);

        MetricsSnapshot {
            t: 0.0,
            order_r: r,
            order_s: Some(r_gyr),
            order_s_plus: None,
            order_s_minus: None,
        }
    }

    fn post_step(&self, s: &mut [f64]) {
        let n = self.n;
        for i in (2 * n)..(3 * n) {
            s[i] = wrap_to_pi(s[i]);
        }
    }
}
