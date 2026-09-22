//! # Morphological Swarm Aggregation & Expressivity (arXiv:2601.07610)
//!
//! 系统复现 Jeremy Fersula, Nicolas Bredeche, Olivier Dauchot 2026 年工作：
//! *Aggregating swarms through morphology handling design contingencies: from the sweet spot to a rich expressivity*
//!
//! - `model`: 质点动力学状态、周期性边界、WCA 软排斥势能、自主推进及形态自对齐力矩更新
//! - `metrics`: 光照成核聚集率 $N_\circ / N$、宏观极化对齐度 $\langle \Psi \rangle$、接触连通网络统计

pub mod metrics;
pub mod model;

pub use metrics::SwarmMetrics;
pub use model::{MorphologicalSwarm, MorphologyType, Particle, SwarmParams, WCA_CUTOFF};

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_periodic_boundary_displacement_and_wrapping() {
        let half_l = 10.0;
        // 跨越右边界测试: x_i = 9.0, x_j = -9.0 -> 实际最短位移 dx = (-9 - 9) + 20 = 2.0
        let (dx, dy) = MorphologicalSwarm::minimum_image_displacement(
            [9.0, 0.0],
            [-9.0, 0.0],
            half_l,
        );
        assert!((dx - 2.0).abs() < 1e-10);
        assert!(dy.abs() < 1e-10);

        // 坐标周期性映射
        assert!((MorphologicalSwarm::wrap_coordinate(11.5, half_l) - (-8.5)).abs() < 1e-10);
        assert!((MorphologicalSwarm::wrap_coordinate(-12.0, half_l) - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_wca_force_repulsion_and_cutoff() {
        let mut params = SwarmParams::default();
        params.n_particles = 2;
        params.half_box_l = 10.0;

        // 两个粒子间距设为 0.8 (< WCA_CUTOFF = 1.12246)
        let mut p1 = Particle::new([0.0, 0.0], 0.0);
        let mut p2 = Particle::new([0.8, 0.0], 0.0);
        p1.pos = [0.0, 0.0];
        p2.pos = [0.8, 0.0];

        let swarm = MorphologicalSwarm {
            params: params.clone(),
            particles: vec![p1, p2],
            current_time: 0.0,
            step_count: 0,
        };

        let forces = swarm.compute_wca_forces();
        // p1 在 p2 左边，受到 p2 施加的向左排斥力 (fx < 0)
        assert!(forces[0][0] < 0.0, "WCA 排斥力应当向左推动粒子 1");
        // p2 在 p1 右边，受到向右排斥力 (fx > 0)
        assert!(forces[1][0] > 0.0, "WCA 排斥力应当向右推动粒子 2");
        // 牛顿第三定律: 作用力与反作用力大小相等方向相反
        assert!((forces[0][0] + forces[1][0]).abs() < 1e-10);

        // 两个粒子间距设为 1.5 (> WCA_CUTOFF = 1.12246)，力应当严格为 0
        let p3 = Particle::new([0.0, 0.0], 0.0);
        let p4 = Particle::new([1.5, 0.0], 0.0);
        let swarm_far = MorphologicalSwarm {
            params,
            particles: vec![p3, p4],
            current_time: 0.0,
            step_count: 0,
        };
        let forces_far = swarm_far.compute_wca_forces();
        assert_eq!(forces_far[0], [0.0, 0.0]);
        assert_eq!(forces_far[1], [0.0, 0.0]);
    }

    #[test]
    fn test_self_alignment_torque_symmetry() {
        // 测试 Aligner (+1) 与 Fronter (-1) 面对碰撞时的不同转矩响应
        // 粒子 1 主动朝向沿 +x: mu = [1, 0]
        // 受到外力向后阻碍碰撞: F_ext = [-5.0, 1.0] (带有轻微 +y 偏向)
        let mut p_aligner = Particle::new([0.0, 0.0], 0.0);
        p_aligner.vel = [p_aligner.mu[0] + (-5.0), p_aligner.mu[1] + 1.0];

        let p_fronter = p_aligner.clone();

        // 计算 Aligner 力矩项 (kappa = +1.0)
        let kappa_align = 1.0;
        let muy_align = kappa_align
            * (p_aligner.mu[0] * p_aligner.mu[0] * p_aligner.vel[1]
                - p_aligner.mu[0] * p_aligner.mu[1] * p_aligner.vel[0]);

        // 计算 Fronter 力矩项 (kappa = -1.0)
        let kappa_front = -1.0;
        let muy_front = kappa_front
            * (p_fronter.mu[0] * p_fronter.mu[0] * p_fronter.vel[1]
                - p_fronter.mu[0] * p_fronter.mu[1] * p_fronter.vel[0]);

        // 两者转向趋势完全相反
        assert!(muy_align > 0.0);
        assert!(muy_front < 0.0);
        assert!((muy_align + muy_front).abs() < 1e-12);
    }

    #[test]
    fn test_swarm_simulation_numerical_stability() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut params = SwarmParams::default();
        params.n_particles = 16;
        params.half_box_l = 8.0;

        let mut swarm = MorphologicalSwarm::new_random(params, &mut rng);
        assert_eq!(swarm.particles.len(), 16);

        // 运行 200 步
        for _ in 0..200 {
            swarm.step(&mut rng);
        }

        // 验证物理量合规性
        for p in &swarm.particles {
            assert!(p.pos[0] >= -swarm.params.half_box_l && p.pos[0] <= swarm.params.half_box_l);
            assert!(p.pos[1] >= -swarm.params.half_box_l && p.pos[1] <= swarm.params.half_box_l);
            let mu_len = (p.mu[0].powi(2) + p.mu[1].powi(2)).sqrt();
            assert!((mu_len - 1.0).abs() < 1e-6, "朝向单位向量必须归一化");
            assert!(p.vel[0].is_finite() && p.vel[1].is_finite());
        }

        let metrics = swarm.evaluate_metrics();
        assert!(metrics.light_ratio >= 0.0 && metrics.light_ratio <= 1.0);
        assert!(metrics.polar_alignment >= 0.0 && metrics.polar_alignment <= 1.0);
    }
}
