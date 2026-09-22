# Paper Reproduction: Microscopic dynamics of consensus formation in multi-agent LLM Naming Games

> **Paper**: *Microscopic dynamics of consensus formation in multi-agent LLM Naming Games*  
> **arXiv**: [arXiv:2608.02178](https://arxiv.org/abs/2608.02178)  
> **Local PDF**: [`papers/naming-game/02-llm-naming-game-arxiv2026/paper.pdf`](paper.pdf)  
> **Date**: August 2026  

---

## 1. Paper Overview
- **Title**: Microscopic dynamics of consensus formation in multi-agent LLM Naming Games
- **Authors**: Multidisciplinary AI & Statistical Physics Research Group
- **arXiv Link**: [https://arxiv.org/abs/2608.02178](https://arxiv.org/abs/2608.02178)
- **Status**: `[x] Reading` -> `[x] Mathematical Analysis` -> `[x] Implementation in Rust` -> `[x] Example Runner & Scientific Visualization`

---

## 2. Core Concepts & Motivation

近年来，以大语言模型（LLM）为基础的多智能体系统（Multi-Agent Systems）发展迅猛。在没有中心协调者的情况下，去中心化的 LLM 群体能否通过局部交互自发达成共同约定（Linguistic Conventions / Consensus）？其内部的随机性（如解码温度 $T$）如何影响宏观相变？

过去的研究要么将 LLM 当作黑盒，要么仅在单一固定温度（如 $T=0.5$）下观察宏观结果。  
**本文的核心创新在于**：
1. **微观通道分解**：将经典 Naming Game 中的确定性词库检索，替换为温度 $T$ 下单 Token 的 LLM 判断（YES/NO），并首次将每次交互解耦为**库内通道（In-inventory）**与**库外通道（Out-inventory）**；
2. **定义两个条件概率速率**：
   - 固化率 $\pi(T) \equiv P(\text{YES} \mid w \in P_j)$（正确触发坍缩的概率）；
   - 重涂率（Repaint Rate）$\phi(T) \equiv P(\text{YES} \mid w \notin P_j)$（错误触发坍缩、丢失原有词库的概率）；
3. **推导微观漂移与平均场临界线**：
   - 漂移算子：$\Delta(t) = m(t) \pi - (1 - m(t)) \phi$；
   - 临界有序相条件：$$3\pi - 2\phi - 1 > 0$$
   （推广了 Baronchelli 2006 随机命名博弈的 $\pi > 1/3$ 阈值）；
4. **发现三大架构截然不同的监听者机制**：
   - **宽容型 (Permissive, LLaMA-3.1:8B)**：噪声重涂主导，温度升高导致收敛显著变慢（$t_c \sim e^{\alpha T}, \alpha \approx 0.67$）；
   - **近确定型 (Near-deterministic, Mistral:7B)**：$\pi \approx 1, \phi \approx 0$，呈现奇特的**温度盲性（Temperature Blindness, $\alpha \approx 0$）**；
   - **保守型 (Conservative, Phi-3:14B)**：漏坍缩主导（$\phi \approx 0$ 但 $\pi$ 随温度剧烈下降），呈现令人惊叹的**逆温度序（Inverted Temperature Ordering）**：低温反而因为词库急剧膨胀（$\bar{k} \approx 34$）而难以收敛，高温反而加速收敛！

---

## 3. Mathematical Model & Microscopic Channels

### 3.1 离散博弈微观四通道 (Channels)
每次 Speaker $i$ 向 Listener $j$ 传达词汇 $w$ 时：

| 真实状态 | LLM 监听者输出 | 微观通道类别 | 发生概率 | 状态更新动作 |
| :--- | :---: | :--- | :---: | :--- |
| $w \in P_j$ (在库中) | **YES** | **True Positive (TP)** | $\pi(T)$ | **合法坍缩（有序通道）**：$P_i = \{w\}, P_j = \{w\}$ |
| $w \in P_j$ (在库中) | **NO** | **False Negative (FN)** | $1 - \pi(T)$ | **漏坍缩（保守误差）**：$P_j$ 不变，错失共识时机 |
| $w \notin P_j$ (不在库中) | **YES** | **False Positive (FP)** | $\phi(T)$ | **非法重涂（Repaint / 无序噪声）**：$P_i = \{w\}, P_j = \{w\}$ 强行清洗掉听者原词库 |
| $w \notin P_j$ (不在库中) | **NO** | **True Negative (TN)** | $1 - \phi(T)$ | **正常学习**：$P_j \leftarrow P_j \cup \{w\}$ |

### 3.2 宏观序参量与两词微观平均场
设在库比例为 $m(t) \equiv P(w \in P_j)$，净有序漂移为：
$$\Delta(t) = m(t) \pi - (1 - m(t)) \phi$$

在竞争双词 $A, B$ 的截断平均场中，磁化序参量 $u = x - y$ 满足：
$$\dot{u} = \frac{u z}{2} (3\pi - 2\phi - 1) + \mathcal{O}(u^3)$$
系统自发破缺收敛到单一词汇的充要条件为：
$$R \equiv 3\pi - 2\phi - 1 > 0$$

---

## 4. 三大 LLM 架构特性对比矩阵

| 架构类别 | 代表模型 | $\pi(T)$ 范围 | $\phi(T)$ 范围 | 温度敏感度 $\alpha$ ($t_c \sim e^{\alpha T}$) | 宏观收敛特征 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **宽容型** (Permissive) | LLaMA-3.1:8B | $1.0 \to 0.55$ | $0.05 \to 0.50$ | $\alpha \approx 0.67$ | 标准温度序：$T$ 越高，重涂噪声越大，收敛越慢 |
| **近确定型** (Near-deterministic) | Mistral:7B | $\approx 1.0$ | $\approx 0$ | $\alpha \approx 0.01 \approx 0$ | **温度盲性**：所有温度曲线近乎重叠，极快收敛 |
| **保守型** (Conservative) | Phi-3:14B | $0.75 \to 0.30$ | $\approx 0$ | $\alpha \approx 0.43$ (中介机制) | **逆温度序**：低温下词库膨胀至 30+ 词形成瓶颈，高温反而更快打破僵局 |

---

## 5. 一键复现与实验脚本

### 运行仿真
```bash
cargo run --release --example 12_arxiv2026_llm_naming_game
```

### 科学可视化作图
```bash
uv run python python/plot_llm_naming_game.py
```
生成图表位于 `output/`:
- `output/llm_naming_game_trajectories.png`：复现论文图 5，展示三大架构在不同温度下的 $N_d(t)$ 演化与逆温度序；
- `output/llm_naming_game_phase_diagram.png`：复现论文图 2/4，展示 $(\pi, \phi)$ 相平面上的临界线 $3\pi - 2\phi - 1 = 0$ 与架构轨迹。
