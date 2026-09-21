# Paper Reproduction: Microscopic activity patterns in the Naming Game

> **Paper**: Dall'Asta, L., & Baronchelli, A. (2006). *Microscopic activity patterns in the Naming Game*. Journal of Physics A: Mathematical and General, 39(48), 14851.  
> **arXiv**: [cond-mat/0606125](https://arxiv.org/abs/cond-mat/0606125)  
> **Local PDF**: [`papers/naming-game/01-microscopic-activity-condmat2006/paper.pdf`](paper.pdf)

---

## 1. Paper Overview
- **Title**: Microscopic activity patterns in the Naming Game
- **Authors**: Luca Dall'Asta, Andrea Baronchelli
- **Journal**: *J. Phys. A: Math. Gen.* 39 (2006) 14851–14867
- **Year**: 2006
- **Links**:
  - arXiv: [cond-mat/0606125](https://arxiv.org/abs/cond-mat/0606125)
  - DOI: [10.1088/0305-4470/39/48/002](https://doi.org/10.1088/0305-4470/39/48/002)
- **Status**: `[x] Reading` -> `[x] Mathematical Analysis` -> `[x] Hands-on Lab & Reference Code` -> `[x] Reproduction Runner & Visualization`

---

## 2. Core Concepts & Motivation

在统计物理学与复杂网络交叉领域，传统观点扩散或协同模型（如 Voter Model、Ising 自旋翻转、Axelrod 文化传播模型）通常**不具备个体记忆（Memory-less）**，智能体在任意时刻仅处于单一离散状态。

而 **Naming Game（命名博弈）**（Steels 1995, Baronchelli et al. 2006）引入了**基于记忆与协商的离散交互机制**：
1. 每个智能体拥有一个内部词汇库（Inventory），可以同时容纳多个不同状态（词汇/观点）；
2. 智能体间通过两两“说话者-听者”（Speaker-Hearer）谈判博弈，基于反馈进行词汇学习与消除；
3. 系统展现出极富特色的两阶段动力学：
   - **重构区 (Reorganization Region)**：词汇扩散，整体词汇总量 $N_w(t)$ 达到峰值后缓慢重构；
   - **收敛区 (Convergence Region)**：对称性破缺触发超指数级级联（Cascade），迅速收敛到全员共识单一词汇。

本文的核心贡献在于：**首次从微观个体活动（Microscopic Activity）角度，利用主方程（Master Equation）与绝热近似（Adiabatic Approximation），严格解析了网络拓扑结构（度均值 $\langle k \rangle$ 与度涨落 $\langle k^2 \rangle$）对个体词汇库大小分布 $\mathcal{P}_n(k | t)$ 的决定性影响！**

---

## 3. Model Definition & Update Rules (Direct Naming Game)

设群体由 $N$ 个智能体组成，置于无向图 $G = (V, E)$ 的顶点上，每个个体维护内部词汇表 $V_i \subseteq \mathbb{N}$。

### 初始条件
所有智能体词汇库均为空：
$$V_i(t=0) = \emptyset, \quad \forall i \in \{1, \dots, N\}$$

### 单步博弈交互规则 (Direct Naming Game at step $t$)
1. **随机选取交互对**：
   - 均匀随机抽取一位说话者 $S \in \{1, \dots, N\}$（概率 $p_k = 1/N$）；
   - 从 $S$ 的邻居节点集合 $\mathcal{N}(S)$ 中均匀随机抽取一位听者 $H$（度为 $k$ 的节点被选为听者的概率 $q_k = \frac{k p_k}{\langle k \rangle}$，即度越大的节点越容易成为听者）。
2. **说话者表达**：
   - 若 $S$ 词汇表为空 ($V_S = \emptyset$)，则 $S$ 发明一个全新的词汇 $w_{\text{new}}$ 加入 $V_S$；
   - $S$ 从其词汇表 $V_S$ 中均匀随机抽取一个词汇 $w \in V_S$ 并传达给 $H$。
3. **听者反馈与更新**：
   - **谈判成功 (Success)**（当且仅当 $w \in V_H$）：
     - 双方达成共识，消除多余记忆，均将词汇表收缩为仅包含该词汇：
       $$V_S \leftarrow \{w\}, \quad V_H \leftarrow \{w\}$$
   - **谈判失败 (Failure)**（若 $w \notin V_H$）：
     - 听者将新词汇添加到词汇库，说话者保持不变：
       $$V_H \leftarrow V_H \cup \{w\}, \quad V_S \text{ 保持不变}$$

---

## 4. Macroscopic & Microscopic Observables

### 宏观序参量
1. **全网词汇总量** $N_w(t) = \sum_{i=1}^N |V_i(t)|$
2. **当前不同词汇种类数** $N_d(t) = |\bigcup_{i=1}^N V_i(t)|$
3. **即时谈判成功率** $S(t) = P(w \in V_H)$

### 微观活动分布 $\mathcal{P}_n(k | t)$
度为 $k$ 的节点在时刻 $t$ 其词汇库大小 $|V_i| = n$ 的概率分布。

---

## 5. Master Equation & Topological Phase Predictions

主方程形式：
$$
\partial_t \mathcal{P}_n(k, t) = \sum_m \left[ \mathcal{W}_k(m \to n | t) \mathcal{P}_m(k, t) - \mathcal{W}_k(n \to m | t) \mathcal{P}_n(k, t) \right]
$$

### 核心结论矩阵

| 网络拓扑类型 | 代表拓扑 | 重构区分布 $\mathcal{P}_n(k)$ | 收敛区分布 $\mathcal{P}_n(k)$ | 物理机制 |
| :--- | :--- | :--- | :--- | :--- |
| **同质稀疏网络** (Homogeneous) | Erdős-Rényi (ER) 随机图 | **指数衰减** $\mathcal{P}_n \propto e^{-c n}$ | 指数衰减 (截断很早) | 所有节点度接近 $\langle k \rangle$，转移速率与词汇量 $n$ 无关 |
| **异质无标度网络** (Heterogeneous) | Barabási-Albert (BA) 网络 Hub 节点 | **半正态衰减 (高斯型)** $\mathcal{P}_n \propto e^{-\frac{n^2}{2 C(t)}}$ | 指数截断 | Hub 节点被优先选为听者 ($q_k \gg 1$)，获得词汇速率远大于消除速率 |
| **全连接平均场** (Mean-Field) | 完全图 (Complete Graph $K_N$) | **峰值叠加** $\sqrt{N}$ 峰值 + 指数尾部 | **幂律分布** $\mathcal{P}_n \propto n^{-\gamma}$ ($\gamma \approx 1$) | 初始快速累积 $\sqrt{N}$ 词汇；收敛期主导词汇级联带来幂律自相似性 |

---

## 6. One-Click Reproduction Guide

### 运行仿真
```bash
cargo run --release --example 11_condmat2006_naming_game_activity
```

### 科学可视化作图
```bash
python3 python/plot_naming_game.py
```
生成的图表保存在 `output/`:
- `output/naming_game_macro.png`：复现原论文图 1，三类拓扑下 $N_w(t), N_d(t), S(t)$ 宏观演化对比；
- `output/naming_game_micro.png`：复现原论文图 3、图 4、图 6，微观词汇分布在 ER (指数)、BA Hub (半正态) 与完全图 (幂律/峰值) 的严格验证。
