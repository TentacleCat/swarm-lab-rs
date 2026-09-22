//! # 2D Field Grid and Spatial Differential Operators
//!
//! 提供二维结构化网格表示 `Grid2D`，支持高性能内存连续布局、
//! 五点中心差分拉普拉斯算子 $\Delta$、中心差分梯度 $(\partial_x, \partial_y)$ 以及
//! 守恒型一阶迎风对流散度算子 $\nabla \cdot (\phi \mathbf{V})$。

use std::ops::{Index, IndexMut};

/// 二维标量场连续网格
#[derive(Debug, Clone)]
pub struct Grid2D {
    pub nx: usize,
    pub ny: usize,
    pub dx: f64,
    pub dy: f64,
    pub x_min: f64,
    pub y_min: f64,
    pub data: Vec<f64>,
}

impl Grid2D {
    /// 创建给定尺寸与物理范围的二维网格
    pub fn new(nx: usize, ny: usize, x_min: f64, x_max: f64, y_min: f64, y_max: f64, init_val: f64) -> Self {
        assert!(nx >= 3 && ny >= 3, "网格分辨率至少为 3x3");
        assert!(x_max > x_min && y_max > y_min, "物理范围必须满足 max > min");
        let dx = (x_max - x_min) / (nx as f64 - 1.0);
        let dy = (y_max - y_min) / (ny as f64 - 1.0);
        Self {
            nx,
            ny,
            dx,
            dy,
            x_min,
            y_min,
            data: vec![init_val; nx * ny],
        }
    }

    /// 一维扁平索引
    #[inline(always)]
    pub fn idx(&self, i: usize, j: usize) -> usize {
        debug_assert!(i < self.nx && j < self.ny, "网格索引越界: ({}, {}) vs ({}, {})", i, j, self.nx, self.ny);
        j * self.nx + i
    }

    /// 获取网格点对应的物理连续坐标 $(x, y)$
    #[inline(always)]
    pub fn coord(&self, i: usize, j: usize) -> (f64, f64) {
        (self.x_min + i as f64 * self.dx, self.y_min + j as f64 * self.dy)
    }

    /// 获取指定网格点的值
    #[inline(always)]
    pub fn get(&self, i: usize, j: usize) -> f64 {
        self.data[self.idx(i, j)]
    }

    /// 设置指定网格点的值
    #[inline(always)]
    pub fn set(&mut self, i: usize, j: usize, val: f64) {
        let idx = self.idx(i, j);
        self.data[idx] = val;
    }

    /// 全场标量积分 $\iint \phi(x, y) \, dx dy$
    pub fn integrate(&self) -> f64 {
        let sum: f64 = self.data.iter().sum();
        sum * self.dx * self.dy
    }

    /// 计算五点中心差分拉普拉斯算子 $\Delta \phi$，带齐次零通量 Neumann 边界条件
    ///
    /// 边界处理使用虚节点镜像：
    /// - 左边界 $i=0$: $\phi_{-1, j} = \phi_{1, j}$
    /// - 右边界 $i=nx-1$: $\phi_{nx, j} = \phi_{nx-2, j}$
    pub fn compute_laplacian(&self, out: &mut [f64]) {
        assert_eq!(out.len(), self.data.len());
        let inv_dx2 = 1.0 / (self.dx * self.dx);
        let inv_dy2 = 1.0 / (self.dy * self.dy);
        let nx = self.nx;
        let ny = self.ny;

        for j in 0..ny {
            let j_prev = if j == 0 { 1 } else { j - 1 };
            let j_next = if j == ny - 1 { ny - 2 } else { j + 1 };
            let row_offset = j * nx;
            let row_prev = j_prev * nx;
            let row_next = j_next * nx;

            for i in 0..nx {
                let i_prev = if i == 0 { 1 } else { i - 1 };
                let i_next = if i == nx - 1 { nx - 2 } else { i + 1 };

                let c = self.data[row_offset + i];
                let l = self.data[row_offset + i_prev];
                let r = self.data[row_offset + i_next];
                let d = self.data[row_prev + i];
                let u = self.data[row_next + i];

                let d2x = (r - 2.0 * c + l) * inv_dx2;
                let d2y = (u - 2.0 * c + d) * inv_dy2;
                out[row_offset + i] = d2x + d2y;
            }
        }
    }

    /// 计算中心差分梯度分量 $(\partial_x \phi, \partial_y \phi)$
    pub fn compute_gradient(&self, grad_x: &mut [f64], grad_y: &mut [f64]) {
        assert_eq!(grad_x.len(), self.data.len());
        assert_eq!(grad_y.len(), self.data.len());
        let inv_2dx = 1.0 / (2.0 * self.dx);
        let inv_2dy = 1.0 / (2.0 * self.dy);
        let nx = self.nx;
        let ny = self.ny;

        for j in 0..ny {
            let j_prev = if j == 0 { 0 } else { j - 1 };
            let j_next = if j == ny - 1 { ny - 1 } else { j + 1 };
            let row_offset = j * nx;
            let row_prev = j_prev * nx;
            let row_next = j_next * nx;

            for i in 0..nx {
                let i_prev = if i == 0 { 0 } else { i - 1 };
                let i_next = if i == nx - 1 { nx - 1 } else { i + 1 };

                let idx = row_offset + i;
                let dx_term = if i == 0 {
                    (self.data[row_offset + 1] - self.data[row_offset]) / self.dx
                } else if i == nx - 1 {
                    (self.data[row_offset + nx - 1] - self.data[row_offset + nx - 2]) / self.dx
                } else {
                    (self.data[row_offset + i_next] - self.data[row_offset + i_prev]) * inv_2dx
                };

                let dy_term = if j == 0 {
                    (self.data[row_next + i] - self.data[row_offset + i]) / self.dy
                } else if j == ny - 1 {
                    (self.data[row_offset + i] - self.data[row_prev + i]) / self.dy
                } else {
                    (self.data[row_next + i] - self.data[row_prev + i]) * inv_2dy
                };

                grad_x[idx] = dx_term;
                grad_y[idx] = dy_term;
            }
        }
    }

    /// 守恒型一阶迎风对流散度 $\nabla \cdot (\phi \mathbf{V})$
    ///
    /// 给定标量场 $\phi$（即 `self`）与速度场 $\mathbf{V} = (v_x, v_y)$，
    /// 在单元控制面 $(i+1/2, j)$ 和 $(i, j+1/2)$ 采用上游风迎风插值计算数值质量通量，
    /// 在区域外法向界面通量置 0（满足零通量自然边界条件），确保全局质量严格守恒：
    /// $\sum_{i,j} \nabla \cdot (\phi \mathbf{V})_{i,j} \Delta x \Delta y = 0$。
    pub fn compute_upwind_divergence(&self, vx: &[f64], vy: &[f64], div_out: &mut [f64]) {
        assert_eq!(vx.len(), self.data.len());
        assert_eq!(vy.len(), self.data.len());
        assert_eq!(div_out.len(), self.data.len());

        let nx = self.nx;
        let ny = self.ny;
        let inv_dx = 1.0 / self.dx;
        let inv_dy = 1.0 / self.dy;

        // 计算 x 方向单元界面通量 Fx_{i+1/2, j}，尺寸 (nx - 1) * ny
        // 边界 i=0 左侧与 i=nx-1 右侧通量为 0 (零流出边界)
        for j in 0..ny {
            let row = j * nx;
            for i in 0..nx {
                // x 方向通量:
                // 左面通量 F_west (即界面 i-1/2)
                let f_west = if i == 0 {
                    0.0
                } else {
                    let v_face = 0.5 * (vx[row + i - 1] + vx[row + i]);
                    if v_face >= 0.0 {
                        v_face * self.data[row + i - 1]
                    } else {
                        v_face * self.data[row + i]
                    }
                };

                // 右面通量 F_east (即界面 i+1/2)
                let f_east = if i == nx - 1 {
                    0.0
                } else {
                    let v_face = 0.5 * (vx[row + i] + vx[row + i + 1]);
                    if v_face >= 0.0 {
                        v_face * self.data[row + i]
                    } else {
                        v_face * self.data[row + i + 1]
                    }
                };

                // y 方向通量:
                // 南面通量 F_south (即界面 j-1/2)
                let f_south = if j == 0 {
                    0.0
                } else {
                    let v_face = 0.5 * (vy[(j - 1) * nx + i] + vy[j * nx + i]);
                    if v_face >= 0.0 {
                        v_face * self.data[(j - 1) * nx + i]
                    } else {
                        v_face * self.data[j * nx + i]
                    }
                };

                // 北面通量 F_north (即界面 j+1/2)
                let f_north = if j == ny - 1 {
                    0.0
                } else {
                    let v_face = 0.5 * (vy[j * nx + i] + vy[(j + 1) * nx + i]);
                    if v_face >= 0.0 {
                        v_face * self.data[j * nx + i]
                    } else {
                        v_face * self.data[(j + 1) * nx + i]
                    }
                };

                div_out[row + i] = (f_east - f_west) * inv_dx + (f_north - f_south) * inv_dy;
            }
        }
    }
}

impl Index<(usize, usize)> for Grid2D {
    type Output = f64;
    #[inline(always)]
    fn index(&self, (i, j): (usize, usize)) -> &Self::Output {
        &self.data[self.idx(i, j)]
    }
}

impl IndexMut<(usize, usize)> for Grid2D {
    #[inline(always)]
    fn index_mut(&mut self, (i, j): (usize, usize)) -> &mut Self::Output {
        let idx = self.idx(i, j);
        &mut self.data[idx]
    }
}
