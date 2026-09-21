# Swarmalator Theory Primer

## 1. Motivation: Merging Swarming with Synchronization

In nature, systems often exhibit both **spatial self-organization** (flocking of birds, swarming of bacteria, schooling of fish) and **phase synchronization** (synchronous flashing of fireflies, beating of heart cells, chirp rhythms of crickets).

Traditionally, theoretical physics treated these two phenomena separately:
- **Synchronization Models** (e.g. Kuramoto model): Fixed spatial topology or all-to-all coupling, agents update only internal phases $\theta_i \in S^1$.
- **Swarming Models** (e.g. Vicsek model, Couzin model): Agents update spatial coordinates $\mathbf{x}_i \in \mathbb{R}^d$ and velocity directions, without internal phase oscillators.

**Swarmalators** (coined by Kevin O'Keeffe, Hyunsuk Hong, and Steven H. Strogatz in 2017) unify both:
> **Swarmalator = Swarm + Oscillator**
> Particles have both a spatial location $\mathbf{x}_i$ and an internal phase $\theta_i$, with bidirectional coupling:
> 1. Spatial motion depends on relative internal phases.
> 2. Phase synchronization depends on spatial proximity.

---

## 2. General Mathematical Formulation

For $N$ swarmalators in $\mathbb{R}^d$:

$$
\dot{\mathbf{x}}_i = \mathbf{v}_i + \frac{1}{N} \sum_{j \neq i}^N \mathbf{F}(\mathbf{x}_j - \mathbf{x}_i, \theta_j - \theta_i)
$$

$$
\dot{\theta}_i = \omega_i + \frac{1}{N} \sum_{j \neq i}^N H(\theta_j - \theta_i) G(\|\mathbf{x}_j - \mathbf{x}_i\|)
$$

Where:
- $\mathbf{F}(\mathbf{x}_{ij}, \theta_{ij})$ is the spatial interaction force (typically attraction-repulsion modulated by phase correlation $1 + J \cos(\theta_j - \theta_i)$).
- $G(r_{ij})$ is spatial proximity weighting (e.g. $1/r_{ij}$ or $e^{-r_{ij}/\sigma}$).
- $H(\theta_{ij})$ is phase interaction (e.g. $K \sin(\theta_j - \theta_i)$ as in Kuramoto).

---

## 3. Order Parameters

To quantitatively diagnose and classify macroscopic emergent phases:

### 1. Kuramoto Phase Coherence $R$
$$
Z = \frac{1}{N} \sum_{j=1}^N e^{i \theta_j} = R e^{i \psi}, \quad R \in [0, 1]
$$
- $R \approx 1$: Complete phase synchrony.
- $R \approx 0$: Incoherent / asynchronous phases.

### 2. Spatial Coherence $S$ (1D Ring)
$$
S = \left| \frac{1}{N} \sum_{j=1}^N e^{i \phi_j} \right| \in [0, 1]
$$
- $S \approx 1$: Particles concentrated at a single spatial cluster.
- $S \approx 0$: Particles uniformly distributed around the ring.

### 3. Spatio-Temporal Order Parameters $S_{\pm}$
$$
W_{\pm} = \frac{1}{N} \sum_{j=1}^N e^{i (\phi_j \pm \theta_j)}, \quad S_{\pm} = |W_{\pm}| \in [0, 1]
$$
- $S_+ \approx 1$ or $S_- \approx 1$: Pure **Phase Wave** state! The internal phase $\theta$ is perfectly correlated with spatial position $\phi$ ($\theta_i \equiv \mp \phi_i + C$).

---

## 4. Macroscopic States Summary Table

| State | Spatial Order $S$ | Phase Order $R$ | Phase Wave $S_{\pm}$ | Motion |
| :--- | :--- | :--- | :--- | :--- |
| **Static Synchrony** | Clustered ($S \approx 1$) | Coherent ($R \approx 1$) | $S_{\pm} \approx 0$ | Static |
| **Static Asynchrony** | Spread ($S \approx 0$) | Incoherent ($R \approx 0$) | $S_{\pm} \approx 0$ | Static |
| **Static Phase Wave** | Spread ($S \approx 0$) | Incoherent ($R \approx 0$) | High ($S_+ \approx 1$ or $S_- \approx 1$) | Static (frozen rainbow) |
| **Active Phase Wave** | Swirling / Ring | Dynamic | Fluctuating | Continuous periodic motion |
| **Splintered Phase Wave** | Broken clusters | Partially coherent | Clustered | Dynamic / Chimera-like |
