use crate::types::DynamicalSystem;

/// Classical 4th-order Runge-Kutta (RK4) integrator with reusable buffers
pub struct Rk4Integrator {
    k1: Vec<f64>,
    k2: Vec<f64>,
    k3: Vec<f64>,
    k4: Vec<f64>,
    tmp: Vec<f64>,
}

impl Rk4Integrator {
    pub fn new(dim: usize) -> Self {
        Self {
            k1: vec![0.0; dim],
            k2: vec![0.0; dim],
            k3: vec![0.0; dim],
            k4: vec![0.0; dim],
            tmp: vec![0.0; dim],
        }
    }

    pub fn step<S: DynamicalSystem>(&mut self, system: &S, state: &mut [f64], dt: f64) {
        let dim = state.len();
        debug_assert_eq!(dim, self.k1.len());

        // k1 = f(s)
        system.derivative(state, &mut self.k1);

        // k2 = f(s + 0.5 * dt * k1)
        for i in 0..dim {
            self.tmp[i] = state[i] + 0.5 * dt * self.k1[i];
        }
        system.derivative(&self.tmp, &mut self.k2);

        // k3 = f(s + 0.5 * dt * k2)
        for i in 0..dim {
            self.tmp[i] = state[i] + 0.5 * dt * self.k2[i];
        }
        system.derivative(&self.tmp, &mut self.k3);

        // k4 = f(s + dt * k3)
        for i in 0..dim {
            self.tmp[i] = state[i] + dt * self.k3[i];
        }
        system.derivative(&self.tmp, &mut self.k4);

        // s_next = s + dt/6 * (k1 + 2*k2 + 2*k3 + k4)
        let dt6 = dt / 6.0;
        for i in 0..dim {
            state[i] += dt6 * (self.k1[i] + 2.0 * self.k2[i] + 2.0 * self.k3[i] + self.k4[i]);
        }

        system.post_step(state);
    }
}

/// Simple forward Euler integrator for fast testing / sanity checking
pub struct EulerIntegrator {
    deriv: Vec<f64>,
}

impl EulerIntegrator {
    pub fn new(dim: usize) -> Self {
        Self {
            deriv: vec![0.0; dim],
        }
    }

    pub fn step<S: DynamicalSystem>(&mut self, system: &S, state: &mut [f64], dt: f64) {
        let dim = state.len();
        system.derivative(state, &mut self.deriv);
        for i in 0..dim {
            state[i] += dt * self.deriv[i];
        }
        system.post_step(state);
    }
}
