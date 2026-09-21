# Ring states in swarmalator systems (Phys. Rev. E 2018)

Exploration of Swarmalators confined to 1D circular topology (a ring).

---

## 1. Paper Reference
- **Title**: Ring states in swarmalator systems
- **Authors**: Kevin P. O'Keeffe, Steven H. Strogatz
- **Journal**: *Physical Review E* 98, 022203 (2018)
- **DOI**: [10.1103/PhysRevE.98.022203](https://journals.aps.org/pre/abstract/10.1103/PhysRevE.98.022203)
- **Upstream Code**: [Khev/swarmalators/1D/on-ring/regular](https://github.com/Khev/swarmalators/tree/master/1D/on-ring/regular)

---

## 2. Model Equations on a 1D Ring
Agents have circular spatial coordinates $\phi_i \in [-\pi, \pi)$ and phases $\theta_i \in [-\pi, \pi)$:

$$
\dot{\phi}_i = \nu_i + \frac{1}{N} \sum_{j=1}^N \sin(\phi_j - \phi_i)(1 + J \cos(\theta_j - \theta_i))
$$

$$
\dot{\theta}_i = \omega_i + \frac{K}{N} \sum_{j=1}^N \sin(\theta_j - \theta_i)(1 + J \cos(\phi_j - \phi_i))
$$

---

## 3. Order Parameters
- **Kuramoto phase coherence**:
  $$R = \left| \frac{1}{N} \sum_{j=1}^N e^{i \theta_j} \right|$$
- **Spatial order parameter**:
  $$S = \left| \frac{1}{N} \sum_{j=1}^N e^{i \phi_j} \right|$$
- **Spatio-temporal correlation / Phase wave order parameter**:
  $$S_{\pm} = \left| \frac{1}{N} \sum_{j=1}^N e^{i (\phi_j \pm \theta_j)} \right|$$

---

## 4. Asymptotic Ring States
1. **Sync State**: $S=1, R=1$ (all particles clumping at the same point in space and in phase).
2. **Phase Wave State**: $S=0, R=0, S_+ \approx 1$ or $S_- \approx 1$. Particles are uniformly spread around the ring, with phase strictly locked to position $\theta_i = \pm \phi_i + C$.
3. **Async State**: $S=0, R=0, S_{\pm}=0$. Uniform distribution in both space and phase without correlation.

---

## 5. How to Run Reproduction
```bash
# Ring simulation (J = 0.5, K = -0.5)
cargo run --release -p swarm-cli -- --model ring -n 150 -j 0.5 -k -0.5 --steps 2000 --output data/ring_wave.csv

# Visualize
python3 python/plot_phases.py --traj data/ring_wave.csv --metrics data/metrics.csv --model ring
```
