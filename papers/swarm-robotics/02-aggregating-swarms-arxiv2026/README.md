# Paper Reproduction: Aggregating swarms through morphology handling design contingencies: from the sweet spot to a rich expressivity

> **Paper**: *Aggregating swarms through morphology handling design contingencies: from the sweet spot to a rich expressivity*  
> **Authors**: Jeremy Fersula, Nicolas Bredeche, Olivier Dauchot (Sorbonne Université, CNRS, ESPCI Paris, PSL University)  
> **Preprint**: [arXiv:2601.07610](https://arxiv.org/abs/2601.07610) (cond-mat.soft, cs.RO, Jan 2026)  
> **Local PDF**: [`papers/swarm-robotics/02-aggregating-swarms-arxiv2026/paper.pdf`](paper.pdf)  
> **Upstream Repository (Submodule)**: [`submodules/AggregatingSwarms2026`](../../../submodules/AggregatingSwarms2026/)  

---

## 1. 核心理论背景 (Background & Motivation)

### 1.1 什么是形态计算 (Morphological Computing)?
在传统机器人学中，智能通常被定义为“微处理器执行感知-计算-驱动循环（Sense-Compute-Act Cycle）”。但在微型群体机器人（Swarm Robotics，如 Kilobot）或活性物质（Active Matter）中，个体的计算资源与传感器极其有限，无法运行复杂的轨迹规划、通信协商或 SLAM 算法。

**形态计算（Morphological Computation）** 的核心思想是：**利用物理机身结构（质量分布、接触点不对称性、材料弹性与摩擦）来分担原本需要微处理器计算的任务**，让物理定律自然而然地成为算法逻辑的执行载体。

### 1.2 本论文的突破点：非零速约束下的涌现聚集与“形态甜点区 (Sweet Spot)”
在早期研究中，群体在目标区域聚集往往依赖简单的“进光即停（Stop in light）”策略：
- 每个个体只需独立感知光强，进入光区后完全熄火刹车；
- 机器人停留在光照区边缘自然堆叠成物理壁垒（Steric Wall）。

而本论文提出了一个更深层次、更具物理本源性的问题：**如果机器人不允许停步，只能在光区减速（例如 $v_\circ / v_\bullet = 1/3$），单凭个体无法实现任务，群体能否仅通过机械形态的差异涌现出聚集？**

实验与仿真得出了令人振奋的结论：
1. **Aligner（顺应形态）**：外骨骼力矩使机器人朝向外力同向转动，碰撞后迅速顺滑掠过并对齐速度，在全场产生**宏观极化游荡流（Polar Flocking）**，在光照区聚集率仅约 16%（与随机游走无异）；
2. **Fronter（突刺形态）**：外骨骼力矩使机器人朝向外力反向转动（向接触点死锁），碰撞中实际行进速度剧烈衰减至近乎为零，与光照区慢行协同，**通过动力学诱导相分离（Motility-Induced Phase Separation, MIPS）成核机制，在光照区涌现出高达 40% 的自发强聚集**；
3. **存在严格的“形态甜点区 (Sweet Spot)”**：
   - 逆对齐过弱：不足以抗衡噪声形成稳定核；
   - 逆对齐过强：机器人在黑暗区一经碰头即形成永动机式的微死锁，导致团簇在全场冷冻冻结，反而出现“反向趋光”；
   - 只有在狭窄的甜点区（$\epsilon/\tau_n \approx -1.5 \sim -0.8$），集群才能完美聚集。

---

## 2. 动力学控制方程与数值物理模型

### 2.1 过阻尼朗之万运动微分方程
在低雷诺数/干摩擦表面上，机器人的质心位置 $\vec{r}_i$ 与机身朝向单位向量 $\vec{n}_i = (\cos\theta_i, \sin\theta_i)$ 满足过阻尼随机动力系统：

$$\frac{d\vec{r}_i}{dt} = \vec{v}_i$$

$$\tau_v \frac{d\vec{v}_i}{dt} = v_a(\vec{r}_i) \vec{n}_i - \vec{v}_i + \vec{F}_{i, ext}$$

$$\tau_n \frac{d\vec{n}_i}{dt} = \epsilon (\vec{n}_i \times \vec{v}_i) \times \vec{n}_i + \sqrt{2D} \xi_i \vec{n}_{i, \perp}$$

其中：
- $v_a(\vec{r}_i)$：局部自主推进标量速率。在中心半径 $R_{\text{light}} = 3.83$ 的光照区内为 $v_\circ = 1/3$，在外部暗区为 $v_\bullet = 1.0$；
- $\tau_v = 0.001$：瞬时速度动量弛豫时间（$\tau_v \to 0$ 时即为极限过阻尼 $\vec{v}_i = v_a \vec{n}_i + \vec{F}_{i, ext}$）；
- $\vec{F}_{i, ext} = \sum_{j \neq i} \vec{F}_{ij}$：两两软球排斥碰撞力；
- $\epsilon \in \{-1, +1\}$ 与 $\tau_n$：$\epsilon = -1$ 为 Fronter 形态，$\epsilon = +1$ 为 Aligner 形态，$\tau_n$ 为对齐力矩特征时间，统一由无量纲控制参数 $\kappa = \epsilon / \tau_n \in [-5.0, 5.0]$ 调控；
- $D = 0.01$：角高斯白噪声扩散强度，$\xi_i$ 为均值为 0、方差为 1 的随机扰动，$\vec{n}_{i, \perp} = (-\sin\theta_i, \cos\theta_i)$。

### 2.2 Weeks-Chandler-Andersen (WCA) 截断势能
机器人之间的物理排斥采用平移截断 Lennard-Jones 势能：

$$V_{\text{WCA}}(r) = 4\epsilon_{\text{LJ}} \left[ \left(\frac{\sigma}{r}\right)^{12} - \left(\frac{\sigma}{r}\right)^6 \right] + \epsilon_{\text{LJ}}, \quad r < r_c = 2^{1/6}\sigma \approx 1.1225$$

受力为解析形式：

$$\vec{F}_{ij} = -\nabla V = \frac{48\epsilon_{\text{LJ}}}{r_{ij}^2} \left[ \left(\frac{\sigma}{r_{ij}}\right)^{12} - 0.5 \left(\frac{\sigma}{r_{ij}}\right)^6 \right] (\vec{r}_i - \vec{r}_j)$$

---

## 3. 四大宏观相态与相图（复现 Figure 3）

| 相态区 | $\kappa = \epsilon / \tau_n$ 范围 | 微观力学机制 | 宏观涌现表现 | 光照区占比 $N_\circ/N$ | 极化度 $\langle \Psi \rangle$ |
| :--- | :---: | :--- | :--- | :---: | :---: |
| **Jamming / 冻结死锁** | $\kappa < -2.2$ | 逆对齐极强，撞击后粒子牢牢咬死 | 在全场（包括暗区）形成二元/三元静态冷冻死锁团簇，反向趋光 | $< 0.18$ | $\approx 0$ |
| **MIPS Sweet Spot / 最佳甜点** | $-2.0 \le \kappa \le -0.6$ | 撞击有效衰减瞬时速率，且易受噪声解离重排 | 瞬态增密触发 MIPS 相分离，在光照区形成动态自维持大团簇 | **$0.35 \sim 0.40$** | $\approx 0$ |
| **Active Brownian / ABP 基线** | $-0.5 < \kappa < 0.5$ | 转向力矩趋近于零，纯物理弹开与角扩散 | 空间各向同性随机均匀遍历，无宏观聚集 | $\approx 0.18$ | $\approx 0$ |
| **Polar Flocking / 顺应极化** | $\kappa > 0.5$ | 碰撞后机身偏转为平行速度分量 | 粒子集体转向形成沿周期性盒子的单向大尺度相干游弋流 | $\approx 0.16 \sim 0.18$ | **$0.80 \sim 0.95$** |

*注：理论随机游走基线值为 $\sigma_{\text{area}} \times (v_\bullet / v_\circ) = 0.06 \times 3.0 = 0.18$。*

---

## 4. 原始代码库精要解读 (Codebase Deep Dive)

本工作代码作为 Submodule 存放在 [`submodules/AggregatingSwarms2026/`](../../../submodules/AggregatingSwarms2026/)：

### 4.1 Kilobot 嵌入式 C 固件 (`Algorithms/phototaxisWalkInLight_preset_115.c`)
- **感知端**: 每约 1s 进行一轮环境光强采样（`QUICKLIGHTSAMPLES = 100` 次 ADC 采样平滑去噪）；
- **驱动端 (1:15 占空比间歇推进)**:
  - 振动电机具有起转电压门槛，无法通过线性调压平滑降速；
  - 固件在光照区采用时间切片：`RUN 1 单位时间 + STAND 15 单位时间`，由此等效产生 $1/3$ 的平均运动速度。

### 4.2 LAMMPS 分子动力学扩展插件 (`Simulations/LammpsPlugin/`)
- `fix_spp.cpp` / `fix_evolution.cpp`：
  ```cpp
  // 形态力矩更新 (向量三重积在 2D 上的化简):
  double mux = epsilon * (mu[1]*mu[1]*vx - mu[0]*mu[1]*vy) / tau_n;
  double muy = epsilon * (mu[0]*mu[0]*vy - mu[0]*mu[1]*vx) / tau_n;
  mu[0] += dt * mux;
  mu[1] += dt * muy;
  ```
  该向量式严格等于：
  $$\dot{\vec{n}} \propto \epsilon (\vec{F}_{ext} \cdot \vec{n}_{\perp}) \vec{n}_{\perp}$$

---

## 5. Rust 高性能复现与运行验证

在当前仓库中，我们使用原生 Rust 实现了零堆分配的高性能仿真引擎，并编写了专属实验：

### 5.1 运行专属实验
```bash
cargo run --release --example 18_arxiv2026_morphological_aggregating_swarms
```

### 5.2 绘制高分辨率科研图表
```bash
uv run python python/plot_morphological_aggregating_swarms.py
```
- 生成图表 1: [`output/morphological_swarms_phase_diagram.png`](../../../output/morphological_swarms_phase_diagram.png) (完整复现论文 Figure 3 双轴相图与甜点区)
- 生成图表 2: [`output/morphological_swarms_spatial_snapshots.png`](../../../output/morphological_swarms_spatial_snapshots.png) (4 种典型相态空间瞬态分布与流动矢量)
