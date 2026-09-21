//! [Reference] 序参量计算的标准参考实现

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

pub fn ring_spatial_order_parameter(positions: &[f64]) -> f64 {
    kuramoto_order_parameter(positions)
}

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
