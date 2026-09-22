# Paper Reproduction: Emergence of Specialised Collective Behaviors in Evolving Heterogeneous Swarms

> **Paper**: *Emergence of Specialised Collective Behaviors in Evolving Heterogeneous Swarms*  
> **Authors**: Fuda van Diggelen, Matteo de Carlo, Nicolas Cambier, Eliseo Ferrante, A. E. Eiben (Vrije Universiteit Amsterdam)  
> **Conference**: International Conference on Parallel Problem Solving from Nature (*PPSN XVIII 2024*, Springer LNCS 14965, pp. 53–69)  
> **DOI**: [10.1007/978-3-031-70068-2_4](https://doi.org/10.1007/978-3-031-70068-2_4)  
> **arXiv**: [arXiv:2402.04763](https://arxiv.org/abs/2402.04763)  
> **Original Code**: [github.com/fudavd/EC_swarm (branch PPSN_2024)](https://github.com/fudavd/EC_swarm/tree/PPSN_2024)  

---

## 1. 核心理论背景 (Background & Motivation)

在自然界群体（如社会性昆虫群、鱼群和鸟群）中，**任务专门化与劳动力分工（Division of Labor）** 是解决复杂生存任务、提高群体弹性和资源效率的核心驱动力。这种机制在生物学中通常由**表型可塑性（Phenotypic Plasticity）** 所支持：即拥有相同基因组的个体，在不同环境刺激（如信息素梯度、食物密度、光敏信号）下表现出截然不同的行为表型。

在群体机器人（Swarm Robotics）领域，如何使一群**无显式通信（No Message Passing）、无记忆（Memoryless）、感知能力极度受限（Limited Sensing）** 的简单机器人自发涌现分工与协同，长期以来是一个极具挑战性的前沿难题。

阿姆斯特丹自由大学（VU Amsterdam）团队发表于 **PPSN 2024** 的这篇开创性论文，提出了**基于进化算法（CMA-ES）与储备池神经网络（Reservoir Neural Network, RNN）的异构群体自组织涌现架构**：
1. **单一全局目标**: 仅通过一个无任何任务先验知识的群体级标量梯度感知任务（Emergent Perception Task）引导进化，不设置任何子任务奖励或分工惩罚；
2. **表型可塑性基因组**: 单一全基因型 $x \in \mathbb{R}^{36}$ 分解为两组神经网络控制器权重，协同演化两个子群；
3. **自适应在线调控机制**: 启发式概率有限状态机使个体能根据局域光强以纯去中心化方式实时切换表型，大幅提升可扩展性与鲁棒性。

---

## 2. 机器人认知、运动学与感知模型

### 2.1 极简认知约束
- **无通信 (Non-communicating)**: 机器人之间不通过 WiFi、蓝牙或无线电广播任何信息或位置；
- **反应式与无记忆 (Memoryless & Reactive)**: 控制器仅依赖当前即时局部传感器数值，不保存任何历史状态；
- **无专门化认知 (Specialization Unaware)**: 机器人自身“不知道”群体中存在专门化分工，亦无预设的角色标签。

### 2.2 Thymio II 差速驱动运动学
- 小车底盘基于标准 Thymio II 规格：轮距 $L = 0.085\text{ m}$，轮半径 $R = 0.021\text{ m}$，最大线速度 $v_{\max} = \pm 0.14\text{ m/s}$；
- 控制器输出目标线速度比例 $v \in [-1, 1]$ 与目标角速度比例 $w \in [-1, 1]$；
- 差速小车运动学控制方程：
  $$\dot{x} = v \cdot v_{\max} \cos(\theta), \quad \dot{y} = v \cdot v_{\max} \sin(\theta), \quad \dot{\theta} = w \cdot \frac{2 v_{\max}}{L}$$

### 2.3 9 维受限感知向量
传感器以 $10\text{ Hz}$（$\Delta t = 0.1\text{ s}$）频率采样：
1. **4 象限方位与相对航向传感器**:
   - 覆盖机器人本体周向 $360^\circ$（前 Front、右 Right、后 Back、左 Left 四个 $90^\circ$ 象限）；
   - 在每个象限 $i \in \{0, 1, 2, 3\}$ 内，测量有效感知距离 $r_{\max} = 2.0\text{ m}$ 内的最近邻居距离 $d_i$ 与相对航向角 $\theta_i$；
   - 距离超出 $2.0\text{ m}$ 时，默认距离取 $d_i = 2.01\text{ m}$，相对航向 $\theta_i = 0.0$；
   - 归一化至 $[-1.0, 1.0]$：
     $$d_{i, \text{norm}} = \frac{d_i}{r_{\max}} \times 2 - 1, \quad \theta_{i, \text{norm}} = \frac{\theta_i}{\pi}$$
2. **局部标量光敏传感器**:
   - 测量当前位置的光强标量值 $G \in [0, 255]$；
   - 归一化至 $[-1.0, 1.0]$：
     $$G_{\text{norm}} = \frac{2 G}{255} - 1$$
3. **总输入向量**:
   $$s_{\text{in}} = [d_{\text{front}}, \theta_{\text{front}}, d_{\text{right}}, \theta_{\text{right}}, d_{\text{back}}, \theta_{\text{back}}, d_{\text{left}}, \theta_{\text{left}}, G_{\text{norm}}]^T \in [-1, 1]^9$$

---

## 3. 控制器架构与协方差矩阵自适应进化 (CMA-ES)

### 3.1 储备池神经网络 (RNN)
- 输入层: $9$ 神经元；
- 隐藏层: $2$ 层各 $9$ 神经元，ReLU 激活函数；
- 储备池权重 $W_{h1}, W_{h2} \in \mathbb{R}^{9 \times 9}$ 在初始时按均匀分布 $U[-1, 1]$ 采样并**永久冻结**；
- 偏置置 0，仅进化输出层权重 $W_{\text{out}} \in \mathbb{R}^{2 \times 9}$（$18$ 个可学习参数）；
- 控制器前向推理方程：
  $$\text{RNN} = \tanh\left( W_{\text{out}} \cdot \text{ReLU}\left( W_{h2} \cdot \text{ReLU}\left( W_{h1} s_{\text{in}} \right) \right) \right) \in [-1, 1]^2$$

### 3.2 异构群体基因型协同表达
- 全群体分为子群 1（Green）和子群 2（Red），各拥有独立的冻结储备池 Reservoir 1 与 Reservoir 2；
- 单个全基因型向量长 36 维：
  $$x = \left[ W_{\text{out}, 1}, W_{\text{out}, 2} \right] \in \mathbb{R}^{36}$$
- 采用 **CMA-ES（Covariance Matrix Adaptation Evolution Strategy）** 进行优化：
  - 种群大小 $\lambda = 30$；
  - 终止代数 $N_{\text{gen}} = 100$；
  - 步长 $\sigma_0 = 1.0$；
  - 每次评估运行 $N_{\text{repeats}} = 3$ 次独立试验，取适应度中位数以降低随机性干扰。

### 3.3 全局适应度函数与序参量指标
1. **累积光强适应度 (公式 1)**:
   $$f = \frac{\sum_{t=0}^T l_t}{G_{\max} \cdot T}, \quad \text{其中} \quad l_t = \frac{1}{N} \sum_{n=1}^N G_n(t)$$
2. **群体运动对齐序参量 (公式 2)**:
   $$\Phi = \frac{1}{N} \sum_{n=1}^N \varphi_n, \quad \varphi_n = \frac{\left\| \left( \sum_{p=1}^P e^{j \theta_p} \right) + e^{j \theta_n} \right\|}{P + 1}$$
   $\Phi \in [0, 1]$：越接近 1 说明群体运动航向高度一致对齐，接近 0 说明各向无序发散。

---

## 4. 关键科学发现与实验复现结论

### 4.1 子群专门化行为的自发涌现 (Exploitation vs Exploration)
在没有任何先验角色指定的前提下，进化算法自发促成了双子群的劳动分工：
- **子群 1 (Green - 剥削利用型 Exploitative)**:
  在接近梯度中心的高光强区表现卓越，低航向对齐度，更贪婪地盘踞中心强光区以最大化瞬时光强；
- **子群 2 (Red - 探索协调型 Exploratory)**:
  在远离中心的弱光强区表现卓越，具有显著更高的群体运动对齐度 $\Phi$，展现高度协调的集体巡航搜索行为；
- **子群协同效应 (Table 3)**:
  当群体在远距离（$r_{\text{dist}} \ge 0.5 \times 12\text{m}$）初始化时，均质混合比例（如 $2:2$ 或 $1:3$）的适应度显著高于任何单一极端子群（$4:0$ 全绿或 $0:4$ 全红），证实异构群体的相互作用产生了 $1+1 > 2$ 的涌现增益。

### 4.2 在线自适应表型调控机制 (Phenotypic Plasticity)
基于子群协同效应分析，设计去中心化概率有限状态机：
$$P_{\text{green}}(\text{light}) = \begin{cases}
    1.00 & \text{if } \text{light} > 229 \\
    0.75 & \text{if } 76 < \text{light} \le 229 \\
    0.50 & \text{if } \text{light} \le 76
\end{cases}$$
每隔 $\tau_{\text{reg}} = 5.0\text{ s}$ 独立重抽样一次表型。

**Table 4 复现验证结果对比**:
| 实验维度 | 实验条件 | Baseline (同质) | Best Hetero (固定 2:2) | Adaptive (表型可塑性) |
|---|---|---|---|---|
| **可扩展性 (Scalability)** | $N = 10$ | $0.23 \pm 0.09$ | $0.35 \pm 0.10$ | **$0.40 \pm 0.12$** |
| | $N = 20$ | $0.33 \pm 0.12$ | $0.39 \pm 0.12$ | **$0.43 \pm 0.08$** |
| | $N = 50$ | $0.45 \pm 0.05$ | $0.47 \pm 0.03$ | **$0.49 \pm 0.03$** |
| **鲁棒性 (Robustness)** | Bi-modal (双峰决策) | $0.40 \pm 0.19$ | $0.43 \pm 0.17$ | **$0.47 \pm 0.19$** |
| | Linear (弱渐变场) | $0.43 \pm 0.24$ | $0.51 \pm 0.19$ | **$0.59 \pm 0.25$** |
| | Banana (弯曲浅谷陷阱) | $0.19 \pm 0.32$ | $0.25 \pm 0.29$ | **$0.31 \pm 0.35$** |

---

## 5. 一键复现与出图指令

```bash
# 1. 运行核心单元测试 (差速物理、储备池前向推理、CMA-ES 优化、调控状态机与序参量)
cargo test -p swarm-core -- heterogeneous_swarms

# 2. 运行完整进化优化与验证仿真流水线
cargo run --release --example 17_springer2024_heterogeneous_swarms

# 3. 独立生成发表级科学全景图 (若未自动调用)
uv run python python/plot_heterogeneous_swarms.py
```
- 输出图表位于 `output/`:
  - `heterogeneous_swarms_learning_curves.png`: 进化收敛曲线对比；
  - `heterogeneous_swarms_subgroup_ratios.png`: 子群比例与距离热力矩阵 (Table 3)；
  - `heterogeneous_swarms_scalability_robustness.png`: 可扩展性与鲁棒性柱状图 (Table 4)；
  - `heterogeneous_swarms_trajectories.png`: 空间轨迹与对齐序参量动态时序。
