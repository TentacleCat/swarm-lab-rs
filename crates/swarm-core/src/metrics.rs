/// Compute Kuramoto phase coherence order parameter R in [0, 1]
pub fn kuramoto_order_parameter(phases: &[f64]) -> f64 {
    let n = phases.len() as f64;
    if n == 0.0 {
        return 0.0;
    }
    let (mut sum_cos, mut sum_sin) = (0.0, 0.0);
    for &th in phases {
        sum_cos += th.cos();
        sum_sin += th.sin();
    }
    ((sum_cos / n).powi(2) + (sum_sin / n).powi(2)).sqrt()
}

/// Compute circular spatial coherence order parameter S in [0, 1] for ring coordinates
pub fn ring_spatial_order_parameter(positions: &[f64]) -> f64 {
    kuramoto_order_parameter(positions)
}

/// Compute spatio-temporal correlation order parameters S_+ and S_- on a ring
/// S_+ = |(1/N) sum exp(i (phi_j + theta_j))|
/// S_- = |(1/N) sum exp(i (phi_j - theta_j))|
pub fn ring_spatiotemporal_order_parameters(positions: &[f64], phases: &[f64]) -> (f64, f64) {
    let n = positions.len() as f64;
    if n == 0.0 {
        return (0.0, 0.0);
    }
    let mut sum_cos_plus = 0.0;
    let mut sum_sin_plus = 0.0;
    let mut sum_cos_minus = 0.0;
    let mut sum_sin_minus = 0.0;

    for (&phi, &theta) in positions.iter().zip(phases.iter()) {
        let plus = phi + theta;
        let minus = phi - theta;
        sum_cos_plus += plus.cos();
        sum_sin_plus += plus.sin();
        sum_cos_minus += minus.cos();
        sum_sin_minus += minus.sin();
    }

    let s_plus = ((sum_cos_plus / n).powi(2) + (sum_sin_plus / n).powi(2)).sqrt();
    let s_minus = ((sum_cos_minus / n).powi(2) + (sum_sin_minus / n).powi(2)).sqrt();
    (s_plus, s_minus)
}

/// Compute radius of gyration for 2D particles
pub fn radius_of_gyration_2d(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    if n == 0.0 {
        return 0.0;
    }
    let mean_x: f64 = x.iter().sum::<f64>() / n;
    let mean_y: f64 = y.iter().sum::<f64>() / n;

    let var: f64 = x.iter().zip(y.iter())
        .map(|(&xi, &yi)| (xi - mean_x).powi(2) + (yi - mean_y).powi(2))
        .sum::<f64>() / n;

    var.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_kuramoto_identical_phases() {
        let phases = vec![0.5, 0.5, 0.5, 0.5];
        let r = kuramoto_order_parameter(&phases);
        assert!((r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_kuramoto_uniform_phases() {
        let phases = vec![0.0, PI / 2.0, PI, 3.0 * PI / 2.0];
        let r = kuramoto_order_parameter(&phases);
        assert!(r < 1e-6);
    }

    #[test]
    fn test_phase_wave_order_parameter() {
        // When theta_i = phi_i, phi - theta = 0, so S_- should be 1.0
        let n = 20;
        let positions: Vec<f64> = (0..n).map(|i| (i as f64) * 2.0 * PI / (n as f64) - PI).collect();
        let phases = positions.clone();
        let (s_plus, s_minus) = ring_spatiotemporal_order_parameters(&positions, &phases);
        assert!(s_plus < 1e-6);
        assert!((s_minus - 1.0).abs() < 1e-6);
    }
}
