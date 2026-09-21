//! # Swarm-Core
//!
//! Core library for high-performance simulation of Swarmalator dynamical systems.
//! Provides ODE integrators, order parameters, and model definitions.

pub mod integrator;
pub mod metrics;
pub mod models;
pub mod types;

pub use integrator::{EulerIntegrator, Rk4Integrator};
pub use metrics::{
    kuramoto_order_parameter, radius_of_gyration_2d, ring_spatial_order_parameter,
    ring_spatiotemporal_order_parameters,
};
pub use models::{Swarmalator2D, SwarmalatorRing1D};
pub use types::{DynamicalSystem, MetricsSnapshot};
