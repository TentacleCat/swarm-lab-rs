# Paper Reproduction: Minority game with local interactions due to the presence of herding behavior

> **Paper**: *Minority game with local interactions due to the presence of herding behavior*  
> **Authors**: A. L. M. Vilela, D. O. Cajueiro et al.  
> **arXiv**: [physics/0512087](https://arxiv.org/abs/physics/0512087)  
> **Local PDF**: [`papers/minority-game/01-herding-behavior-physics0512087/paper.pdf`](paper.pdf)  
> **Journal**: *Physica A: Statistical Mechanics and its Applications* (2006)  

---

## 1. Paper Overview
- **Title**: Minority game with local interactions due to the presence of herding behavior
- **Field**: Econophysics, Statistical Physics of Disordered Systems, Complex Networks
- **Core Topics**:
  - Arthur's El Farol Bar Problem & Challet-Zhang Minority Game (MG)
  - Herding behavior (从众羊群效应 / 盲目跟风)
  - Phase transitions: Volatility $\sigma^2 / N$ vs Information Ratio $\alpha = 2^M / N$
  - Network Topologies: Regular Ring Lattice, Erdős-Rényi Random Graph, Watts-Strogatz Small-World
- **Status**: `[x] Reading` -> `[x] Mathematical Formulation` -> `[x] Rust Implementation` -> `[x] Example Simulation & Scientific Visualization`

---

## 2. Theoretical Background & Physical Motivation

### 2.1 经典少数派博弈 (Standard Minority Game, Challet & Zhang 1997)
- 群体中有 $N$ 个竞争有限资源的智能体（如金融市场买/卖方，或酒吧常客），$N$ 通常为奇数；
- 每次每个个体独立选择行动 $a_i \in \{-1, +1\}$；
- **博弈收益机制**：只有处于**少数派（Minority）**的一方获胜（例如买方多时卖方获利；去酒吧人少时获得舒适体验）；
- **公开信息**：所有人仅能看到过去 $M$ 轮的历史胜负序列（共有 $2^M$ 种历史状态）；
- **策略表 (Strategy Table)**：将 $2^M$ 种历史映射为 $\pm 1$。每个人预先随机分配 $S$ 个策略表（通常 $S=2$）；
- **虚拟打分 (Virtual Scoring)**：每个策略根据历史预测准确度累积虚拟分数 $U_{i, s}(t)$，个体每次执行自己手头分数最高的策略。

#### 经典相变现象 ($\sigma^2 / N$ vs $\alpha = 2^M / N$):
- **拥挤相 / 羊群相 (Low-$M$, $\alpha < \alpha_c$)**: 历史状态少，不同个体的策略重合度高，集体盲目同向决策，导致市场波动率 $\sigma^2 / N > 1$（表现比抛硬币还差）；
- **临界高效协调点 ($\alpha_c \approx 0.34$)**: 波动率达到极小值 $\sigma^2 / N \ll 1$，有限资源被最大化利用，智能体通过博弈自发形成最优分工；
- **独立相 (High-$M$, $\alpha > \alpha_c$)**: 策略空间巨大，个体行为趋于互不相关的随机游走，$\sigma^2 / N \to 1.0$（硬币市场极限）。

---

### 2.2 本论文的核心扩展：引入网络邻域从众效应 (Herding Behavior)

现实金融市场中，个体常常认为某些邻居（分析师、大V、身边赚了钱的朋友）“信息更灵通”，从而选择**放弃自己的策略，盲从邻居的决定**。

#### 局域从众决策规则：
每个个体置于无向网络（规则网络、小世界网络、随机网络）节点上：
1. **寻找邻域“最懂行者” (Most Informed Neighbor)**:
   $$j^* = \arg\max_{j \in \mathcal{N}(i)} \left( \max_{s} U_{j, s}(t) \right)$$
2. **比较与跟风决策**:
   - 若智能体自身最高策略分 $\ge$ 邻居最高策略分：智能体坚持自己的策略；
   - 若智能体自身最高策略分 $<$ 邻居最高策略分：智能体**盲从 (Imitate)** 邻居 $j^*$ 的动作！

#### 核心物理结论：
1. **波动率暴增 (Volatility Explosion)**：从众效应在网络局部制造了庞大的“共谋群体（Crowd）”，使得集体净动作 $A(t) = \sum a_i$ 出现剧烈震荡；
2. **明星破灭与逆转机制**：当大量邻居跟风模仿 $j^*$ 时，瞬间将 $j^*$ 的选择推到了**多数派 (Majority)**，导致 $j^*$ 本人在下一轮惨败并被拉下神坛！
3. **高效合作相被抹平**：在强连接网络（如小世界网络）中，经典的 U 型波动率曲线转变为单调下降曲线，极小值协作区被完全摧毁。

---

## 3. Reproduction Command & Visualization

### 一键仿真
```bash
cargo run --release --example 13_physics0512087_minority_game_herding
```

### 科学可视化作图
```bash
uv run python python/plot_minority_game.py
```
生成图表位于 `output/`:
- `output/minority_game_volatility.png`：复现论文图 1~4，展示波动率 $\sigma^2 / N$ 随记忆 $M$ 与从众强度的相变；
- `output/minority_game_timeseries.png`：展示标准 MG 与从众 MG 下市场净动作 $A(t)$ 的实时时间序列对比（再现从众引发的市场剧烈震荡）。
