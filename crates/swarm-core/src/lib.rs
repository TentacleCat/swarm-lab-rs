//! # Swarm-Core
//!
//! 群聚同步振子（Swarmalator）动力学仿真核心库。
//!
//! - **实战练习区**: `integrator`, `metrics`, `models`
//! - **参考答案区**: `reference` (包含已验证的标准实现，卡壳时可查阅)

pub mod integrator;
pub mod metrics;
pub mod models;
pub mod ant_foraging;
pub mod minority_game;
pub mod naming_game;
pub mod swarm_robotics;
pub mod turing_morphogenesis;
pub mod heterogeneous_swarms;
pub mod reference;
pub mod types;

// 默认导出你的练习模块
pub use integrator::{EulerIntegrator, Rk4Integrator};
pub use metrics::{
    kuramoto_order_parameter, radius_of_gyration_2d, ring_spatial_order_parameter,
    ring_spatiotemporal_order_parameters,
};
pub use models::{Swarmalator2D, SwarmalatorRing1D};
pub use types::{DynamicalSystem, MetricsSnapshot};
