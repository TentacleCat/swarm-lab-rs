# Oscillators that sync and swarm (Nature Communications 2017)

Foundational paper introducing the concept of **Swarmalators** (Swarming Oscillators).

---

## 1. Paper Reference
- **Title**: Oscillators that sync and swarm
- **Authors**: Kevin P. O'Keeffe, Hyunsuk Hong, Steven H. Strogatz
- **Journal**: *Nature Communications* 8, 1504 (2017)
- **DOI**: [10.1038/s41467-017-01190-3](https://www.nature.com/articles/s41467-017-01190-3)
- **Upstream Code**: [Khev/swarmalators/2D/unit-vector-model](https://github.com/Khev/swarmalators/tree/master/2D/unit-vector-model)

---

## 2. Governing Equations
For $N$ identical oscillators ($i = 1, \dots, N$) with positions $\mathbf{x}_i \in \mathbb{R}^2$ and phases $\theta_i \in [-\pi, \pi)$:

$$
\dot{\mathbf{x}}_i = \frac{1}{N} \sum_{j \neq i}^N \left[ \frac{\mathbf{x}_j - \mathbf{x}_i}{|\mathbf{x}_j - \mathbf{x}_i|} (1 + J \cos(\theta_j - \theta_i)) - \frac{\mathbf{x}_j - \mathbf{x}_i}{|\mathbf{x}_j - \mathbf{x}_i|^2} \right]
$$

$$
\dot{\theta}_i = \omega_i + \frac{K}{N} \sum_{j \neq i}^N \frac{\sin(\theta_j - \theta_i)}{|\mathbf{x}_j - \mathbf{x}_i|}
$$

- $J$: Modulates spatial attraction by phase similarity (like attracts like when $J > 0$).
- $K$: Modulates phase synchronization, decaying with Euclidean distance $1/r_{ij}$.

---

## 3. Five Asymptotic States
In the $(J, K)$ parameter plane, the model exhibits five distinct states:
1. **Static Synchrony** ($J > 0, K > 0$):
   Particles form a dense circular cluster and achieve complete phase locking ($R \to 1$).
2. **Static Asynchrony** ($J < 0, K < 0$):
   Particles form an annulus or disk with completely desynchronized phases uniformly distributed in space ($R \approx 0$).
3. **Static Phase Wave** ($J > 0, K < 0$):
   Particles freeze into a disk where phase is correlated with spatial polar angle: $\theta_i \approx \phi_i + \text{const}$. A frozen rainbow disk!
4. **Active Phase Wave** ($J < 0, K > 0$ or transition boundaries):
   Non-stationary state where particles continuously circulate and swirl around the center in space while running in phase.
5. **Splintered Phase Wave**:
   A chimera-like state where a cluster splits into coherent moving subgroups.

---

## 4. How to Run Reproduction
Run the simulation via CLI:
```bash
# Static Phase Wave (J = 1.0, K = -0.1)
cargo run --release -p swarm-cli -- --model 2d -n 200 -j 1.0 -k -0.1 --steps 3000 --output data/natcomm_wave.csv

# Visualize
python3 python/plot_phases.py --traj data/natcomm_wave.csv --metrics data/metrics.csv --model 2d
```
