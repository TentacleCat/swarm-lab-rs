//! # 💡 [参考答案区] 蚂蚁趋化觅食偏微分方程标准参考实现 (Reference Implementation)
//!
//! 论文: "Modeling ant foraging: a chemotaxis approach with pheromones and trail formation" (arXiv:1409.3808)
//!
//! 本文件包含经过完备单测验证的标准理论计算与基准实现，供闯关学习者对比查阅。

use crate::ant_foraging::grid::Grid2D;
use crate::ant_foraging::metrics::AntForagingMetrics;
use crate::ant_foraging::model::{AntChemotaxisConfig, FoodSource};

/// 标量场五点中心差分拉普拉斯标准实现
pub fn ref_compute_laplacian(grid: &Grid2D, out: &mut [f64]) {
    assert_eq!(out.len(), grid.data.len());
    let inv_dx2 = 1.0 / (grid.dx * grid.dx);
    let inv_dy2 = 1.0 / (grid.dy * grid.dy);
    let nx = grid.nx;
    let ny = grid.ny;

    for j in 0..ny {
        let j_prev = if j == 0 { 1 } else { j - 1 };
        let j_next = if j == ny - 1 { ny - 2 } else { j + 1 };
        let row_offset = j * nx;
        let row_prev = j_prev * nx;
        let row_next = j_next * nx;

        for i in 0..nx {
            let i_prev = if i == 0 { 1 } else { i - 1 };
            let i_next = if i == nx - 1 { nx - 2 } else { i + 1 };

            let c = grid.data[row_offset + i];
            let l = grid.data[row_offset + i_prev];
            let r = grid.data[row_offset + i_next];
            let d = grid.data[row_prev + i];
            let u = grid.data[row_next + i];

            let d2x = (r - 2.0 * c + l) * inv_dx2;
            let d2y = (u - 2.0 * c + d) * inv_dy2;
            out[row_offset + i] = d2x + d2y;
        }
    }
}

/// 守恒型一阶迎风对流散度标准实现
pub fn ref_compute_upwind_divergence(grid: &Grid2D, vx: &[f64], vy: &[f64], div_out: &mut [f64]) {
    assert_eq!(vx.len(), grid.data.len());
    assert_eq!(vy.len(), grid.data.len());
    assert_eq!(div_out.len(), grid.data.len());

    let nx = grid.nx;
    let ny = grid.ny;
    let inv_dx = 1.0 / grid.dx;
    let inv_dy = 1.0 / grid.dy;

    for j in 0..ny {
        let row = j * nx;
        for i in 0..nx {
            let f_west = if i == 0 {
                0.0
            } else {
                let v_face = 0.5 * (vx[row + i - 1] + vx[row + i]);
                if v_face >= 0.0 {
                    v_face * grid.data[row + i - 1]
                } else {
                    v_face * grid.data[row + i]
                }
            };

            let f_east = if i == nx - 1 {
                0.0
            } else {
                let v_face = 0.5 * (vx[row + i] + vx[row + i + 1]);
                if v_face >= 0.0 {
                    v_face * grid.data[row + i]
                } else {
                    v_face * grid.data[row + i + 1]
                }
            };

            let f_south = if j == 0 {
                0.0
            } else {
                let v_face = 0.5 * (vy[(j - 1) * nx + i] + vy[j * nx + i]);
                if v_face >= 0.0 {
                    v_face * grid.data[(j - 1) * nx + i]
                } else {
                    v_face * grid.data[j * nx + i]
                }
            };

            let f_north = if j == ny - 1 {
                0.0
            } else {
                let v_face = 0.5 * (vy[j * nx + i] + vy[(j + 1) * nx + i]);
                if v_face >= 0.0 {
                    v_face * grid.data[j * nx + i]
                } else {
                    v_face * grid.data[(j + 1) * nx + i]
                }
            };

            div_out[row + i] = (f_east - f_west) * inv_dx + (f_north - f_south) * inv_dy;
        }
    }
}

/// 蚂蚁趋化觅食动力学系统标准参考实现
#[derive(Debug, Clone)]
pub struct AntChemotaxisModelReference {
    pub config: AntChemotaxisConfig,
    pub time: f64,
    pub initial_food_mass: f64,
    pub u: Grid2D,
    pub w: Grid2D,
    pub v: Grid2D,
    pub c: Grid2D,
    pub n_field: Grid2D,
    pub p_field: Grid2D,
    pub va_x: Vec<f64>,
    pub va_y: Vec<f64>,
    buf_lap: Vec<f64>,
    buf_grad_x: Vec<f64>,
    buf_grad_y: Vec<f64>,
    buf_div: Vec<f64>,
    buf_vx: Vec<f64>,
    buf_vy: Vec<f64>,
}

impl AntChemotaxisModelReference {
    pub fn new(
        nx: usize,
        ny: usize,
        half_box: f64,
        config: AntChemotaxisConfig,
        food_sources: &[FoodSource],
    ) -> Self {
        let x_min = -half_box;
        let x_max = half_box;
        let y_min = -half_box;
        let y_max = half_box;

        let u = Grid2D::new(nx, ny, x_min, x_max, y_min, y_max, 0.0);
        let w = Grid2D::new(nx, ny, x_min, x_max, y_min, y_max, 0.0);
        let v = Grid2D::new(nx, ny, x_min, x_max, y_min, y_max, 0.0);
        let mut c = Grid2D::new(nx, ny, x_min, x_max, y_min, y_max, 0.0);
        let mut n_field = Grid2D::new(nx, ny, x_min, x_max, y_min, y_max, 0.0);
        let mut p_field = Grid2D::new(nx, ny, x_min, x_max, y_min, y_max, 0.0);

        for j in 0..ny {
            for i in 0..nx {
                let (x, y) = u.coord(i, j);
                let mut food_val = 0.0f64;
                for f in food_sources {
                    let d = ((x - f.x).powi(2) + (y - f.y).powi(2)).sqrt();
                    if d < f.radius {
                        let bell = (1.0 - (d / f.radius).powi(2)).powi(2);
                        food_val = food_val.max(f.initial_density * bell);
                    }
                }
                c.set(i, j, food_val);

                let r_nest = (x * x + y * y).sqrt();
                if r_nest <= config.nest_radius {
                    let w_nest = (1.0 - (r_nest / config.nest_radius).powi(2)).powi(2);
                    n_field.set(i, j, w_nest);
                }

                let p_val = (r_nest / config.phero_fade_radius).min(1.0);
                p_field.set(i, j, p_val);
            }
        }

        let n_norm = n_field.integrate();
        if n_norm > 1e-12 {
            for val in &mut n_field.data {
                *val /= n_norm;
            }
        }

        let initial_food_mass = c.integrate();
        let n_cells = nx * ny;
        let mut va_x = vec![0.0; n_cells];
        let mut va_y = vec![0.0; n_cells];

        for j in 0..ny {
            let row = j * nx;
            for i in 0..nx {
                let (x, y) = u.coord(i, j);
                let r = (x * x + y * y).sqrt();
                if r > 1e-6 {
                    va_x[row + i] = -config.return_speed * (x / r);
                    va_y[row + i] = -config.return_speed * (y / r);
                }
            }
        }

        Self {
            config,
            time: 0.0,
            initial_food_mass,
            u,
            w,
            v,
            c,
            n_field,
            p_field,
            va_x,
            va_y,
            buf_lap: vec![0.0; n_cells],
            buf_grad_x: vec![0.0; n_cells],
            buf_grad_y: vec![0.0; n_cells],
            buf_div: vec![0.0; n_cells],
            buf_vx: vec![0.0; n_cells],
            buf_vy: vec![0.0; n_cells],
        }
    }

    pub fn step(&mut self, dt: f64) {
        let n_cells = self.u.nx * self.u.ny;

        let m_t = if self.time <= self.config.emerge_time {
            self.config.emerge_rate
        } else {
            0.0
        };

        // 1. 食物场 c
        for idx in 0..n_cells {
            let u_val = self.u.data[idx];
            let c_val = self.c.data[idx];
            self.c.data[idx] = (c_val / (1.0 + dt * u_val)).max(0.0);
        }

        // 2. 趋化速度场
        self.v.compute_gradient(&mut self.buf_grad_x, &mut self.buf_grad_y);
        let chi_u = self.config.chi_u;
        for idx in 0..n_cells {
            self.buf_vx[idx] = chi_u * self.buf_grad_x[idx];
            self.buf_vy[idx] = chi_u * self.buf_grad_y[idx];
        }

        ref_compute_upwind_divergence(&self.u, &self.buf_vx, &self.buf_vy, &mut self.buf_div);
        ref_compute_laplacian(&self.u, &mut self.buf_lap);

        let lambda = self.config.lambda;
        for idx in 0..n_cells {
            let u_val = self.u.data[idx];
            let w_val = self.w.data[idx];
            let c_val = self.c.data[idx];
            let n_val = self.n_field.data[idx];
            let lap_u = self.buf_lap[idx];
            let div_chem = self.buf_div[idx];

            let du_dt = lap_u - div_chem - u_val * c_val + lambda * w_val * n_val + m_t * n_val;
            self.buf_grad_x[idx] = (u_val + dt * du_dt).max(0.0);
        }

        // 3. 搬运蚁 w
        ref_compute_upwind_divergence(&self.w, &self.va_x, &self.va_y, &mut self.buf_div);
        ref_compute_laplacian(&self.w, &mut self.buf_lap);

        let d_w = self.config.d_w;
        for idx in 0..n_cells {
            let u_val = self.u.data[idx];
            let w_val = self.w.data[idx];
            let c_val = self.c.data[idx];
            let n_val = self.n_field.data[idx];
            let lap_w = self.buf_lap[idx];
            let div_ret = self.buf_div[idx];

            let dw_dt = d_w * lap_w - div_ret + u_val * c_val - lambda * w_val * n_val;
            self.buf_grad_y[idx] = (w_val + dt * dw_dt).max(0.0);
        }

        // 4. 信息素 v
        ref_compute_laplacian(&self.v, &mut self.buf_lap);
        let d_v = self.config.d_v;
        let eps = self.config.epsilon;

        for idx in 0..n_cells {
            let v_val = self.v.data[idx];
            let w_val = self.w.data[idx];
            let p_val = self.p_field.data[idx];
            let lap_v = self.buf_lap[idx];

            let dv_dt = d_v * lap_v + p_val * w_val - eps * v_val;
            self.buf_div[idx] = (v_val + dt * dv_dt).max(0.0);
        }

        self.u.data.copy_from_slice(&self.buf_grad_x);
        self.w.data.copy_from_slice(&self.buf_grad_y);
        self.v.data.copy_from_slice(&self.buf_div);

        self.time += dt;
    }

    pub fn metrics(&self) -> AntForagingMetrics {
        AntForagingMetrics::compute(
            self.time,
            &self.u,
            &self.w,
            &self.v,
            &self.c,
            self.initial_food_mass,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ref_laplacian() {
        let grid = Grid2D::new(10, 10, -5.0, 5.0, -5.0, 5.0, 3.0);
        let mut lap = vec![0.0; 100];
        ref_compute_laplacian(&grid, &mut lap);
        for &val in &lap {
            assert!(val.abs() < 1e-10);
        }
    }
}
