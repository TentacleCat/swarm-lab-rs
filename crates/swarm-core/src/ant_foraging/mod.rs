//! # Ant Foraging via Chemotaxis (蚂蚁趋化觅食与路径自组织涌现)
//!
//! 基于 Paulo Amorim (2015, *Journal of Theoretical Biology*, [arXiv:1409.3808](https://arxiv.org/abs/1409.3808)) 的
//! 连续介质非线性偏微分方程组：
//!
//! - `grid`: 二维连续网格 `Grid2D` 与守恒型一阶迎风对流/拉普拉斯算子
//! - `model`: `AntChemotaxisModel` 四组分动力学与有限差分积分器
//! - `metrics`: 觅食效率、食物消耗率与信息素统计

pub mod grid;
pub mod metrics;
pub mod model;

pub use grid::Grid2D;
pub use metrics::AntForagingMetrics;
pub use model::{AntChemotaxisConfig, AntChemotaxisModel, FoodSource};

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_grid_creation_and_indexing() {
        let mut grid = Grid2D::new(10, 10, -5.0, 5.0, -5.0, 5.0, 0.0);
        assert_eq!(grid.nx, 10);
        assert_eq!(grid.ny, 10);
        assert_eq!(grid.data.len(), 100);

        grid.set(0, 0, 1.5);
        grid.set(9, 9, 3.5);
        assert!((grid.get(0, 0) - 1.5).abs() < 1e-10);
        assert!((grid.get(9, 9) - 3.5).abs() < 1e-10);

        let (x0, y0) = grid.coord(0, 0);
        assert!((x0 - (-5.0)).abs() < 1e-10);
        assert!((y0 - (-5.0)).abs() < 1e-10);

        let (x9, y9) = grid.coord(9, 9);
        assert!((x9 - 5.0).abs() < 1e-10);
        assert!((y9 - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_laplacian_constant_zero() {
        let grid = Grid2D::new(20, 20, -10.0, 10.0, -10.0, 10.0, 42.0);
        let mut lap = vec![0.0; 400];
        grid.compute_laplacian(&mut lap);
        for &val in &lap {
            assert!(val.abs() < 1e-10, "常数场的拉普拉斯必须为 0，得到 {}", val);
        }
    }

    #[test]
    fn test_upwind_divergence_conservation() {
        // 在带零流出边界的区域内，散度积分 sum (div * dx * dy) 应该严格等于 0
        let mut grid = Grid2D::new(25, 25, -5.0, 5.0, -5.0, 5.0, 1.0);
        // 设置一个非均匀分布
        for j in 5..20 {
            for i in 5..20 {
                grid.set(i, j, 2.5);
            }
        }
        let vx = vec![1.2; 625];
        let vy = vec![-0.8; 625];
        let mut div = vec![0.0; 625];
        grid.compute_upwind_divergence(&vx, &vy, &mut div);

        let sum_div: f64 = div.iter().sum::<f64>() * grid.dx * grid.dy;
        assert!(sum_div.abs() < 1e-8, "零通量边界下的迎风散度全场积分必须严格为 0，得到 {}", sum_div);
    }

    #[test]
    fn test_ant_chemotaxis_simulation_step() {
        let config = AntChemotaxisConfig {
            emerge_rate: 20.0,
            emerge_time: 0.1,
            ..Default::default()
        };
        let food = vec![FoodSource {
            x: 5.0,
            y: 5.0,
            radius: 2.0,
            initial_density: 10.0,
        }];

        let mut model = AntChemotaxisModel::new(30, 30, 10.0, config, &food);
        let m0 = model.metrics();
        assert!(m0.total_food > 0.0);
        assert_eq!(m0.foraging_ants_mass, 0.0);

        // 步进若干步
        let dt = 0.002;
        for _ in 0..50 {
            model.step(dt);
        }

        let m1 = model.metrics();
        assert!(model.time > 0.09);
        assert!(m1.foraging_ants_mass > 0.0, "经过涌出后，觅食蚁质量应大于 0");
        // 保证所有场的值均为非负
        for &u in &model.u.data {
            assert!(u >= 0.0);
        }
        for &w in &model.w.data {
            assert!(w >= 0.0);
        }
        for &v in &model.v.data {
            assert!(v >= 0.0);
        }
        for &c in &model.c.data {
            assert!(c >= 0.0);
        }
    }
}
