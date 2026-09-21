# Paper Reproduction Template

Use this template when starting the reproduction and theoretical study of a new Swarmalator paper.

---

## 1. Paper Overview
- **Title**: 
- **Authors**: 
- **Journal / Conference**: 
- **Year**: 
- **Links**:
  - Paper: 
  - arXiv: 
  - Upstream Code (Khev/swarmalators): 
- **Status**: `[ ] Reading` -> `[ ] Derivations` -> `[ ] Simulation Code` -> `[ ] Validation & Plots`

---

## 2. Core Concepts & Motivation
*Why was this paper written? What new physics, topology, or coupling does it introduce to the Kuramoto / Vicsek / Swarmalator family?*

---

## 3. Mathematical Model & Governing Equations

### Coordinates & Dimensions
- Spatial coordinates $\mathbf{x}_i \in \dots$
- Phase coordinates $\theta_i \in [-\pi, \pi)$
- Number of agents $N$

### Equations of Motion
$$
\dot{\mathbf{x}}_i = \dots
$$

$$
\dot{\theta}_i = \dots
$$

### Parameters
| Parameter | Symbol | Meaning | Typical Range |
| :--- | :--- | :--- | :--- |
| Spatial attraction/coupling | $J$ | ... | ... |
| Phase synchronization | $K$ | ... | ... |
| Natural frequency | $\omega_i$ | ... | ... |

---

## 4. Asymptotic States & Phase Diagram
*List and describe the stable collective states discovered in this paper:*
1. **State 1**: Description and conditions.
2. **State 2**: Description and conditions.
3. **State 3**: Description and conditions.

---

## 5. Order Parameters & Diagnostic Metrics
*Mathematical definitions of order parameters used to classify states:*
$$
R = \left| \frac{1}{N} \sum_{j=1}^N e^{i \theta_j} \right|
$$
$$
S_{\pm} = \dots
$$

---

## 6. Reproduction Plan & Checklist
- [ ] Read and annotate paper
- [ ] Verify linear stability analysis / continuum limit
- [ ] Implement model in Rust (`crates/swarm-core`)
- [ ] Run benchmark simulation with CLI
- [ ] Reproduce Figure X (Phase diagram / Order parameters vs parameters)
- [ ] Write conclusion and insights
