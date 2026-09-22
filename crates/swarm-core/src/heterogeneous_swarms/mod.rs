//! # 2024 Springer/PPSN 异构演化群体与表型可塑性集体感知
//!
//! 论文: *Emergence of Specialised Collective Behaviors in Evolving Heterogeneous Swarms* (PPSN 2024 / Springer)
//!
//! 该模块包含完整的理论建模、储备池神经网络控制器、差速运动学小车物理、
//! 协方差矩阵自适应进化策略 (CMA-ES)、在线表型可塑性调控机制与群体感知仿真器。

pub mod cma_es;
pub mod controller;
pub mod environment;
pub mod metrics;
pub mod regulatory;
pub mod robot;
pub mod sensors;
pub mod simulator;

pub use cma_es::{CmaEsConfig, CmaEsOptimizer};
pub use controller::{BaselineGenotype, HeterogeneousGenotype, Reservoir, ReservoirNN};
pub use environment::{ArenaType, Environment};
pub use metrics::{
    compute_mean_intensity, compute_spatial_stats, compute_subgroup_order, compute_swarm_order,
    SwarmMetricsSnapshot,
};
pub use regulatory::{prob_green, update_swarm_phenotypes, RegulatoryConfig};
pub use robot::{resolve_swarm_collisions, Robot, SubGroup};
pub use sensors::{
    bearing_to_quadrant, compute_robot_sensor_inputs, wrap_to_pi, Quadrant, SensorConfig,
};
pub use simulator::{SimulationResult, SwarmSimConfig, SwarmSimulator};
