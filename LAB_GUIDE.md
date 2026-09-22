# 🚀 Rust 与 Swarmalator 算法实战闯关指南 (Lab Guide)

> **“基础设施可以由工具搭建，核心算法必须自己一行一行敲。”**  
> 本指南专为你量身设计：在不被 Python 环境、数据存储和画图代码分散精力的前提下，**循序渐进地掌握 Rust 核心语法，并亲手复现物理论文中的所有核心算法**。

---

## 🧭 学习架构地图

```text
crates/swarm-core/src/
├── reference/        # 💡 参考答案区 (已包含完整且验证过的标准实现，卡壳时查阅)
│   ├── metrics.rs
│   ├── integrator.rs
│   ├── nature2017_2d.rs
│   ├── ring_1d.rs
│   └── naming_game.rs
│
├── metrics.rs        # 🎯 [实战关卡 1] 切片借用与序参量计算 (从这里开始！)
├── integrator.rs     # 🎯 [实战关卡 2] 可变借用、内存复用与 RK4 积分器
├── models/
│   ├── nature2017_2d.rs  # 🎯 [实战关卡 3] 2017 Nature Comms 2D 经典模型
│   └── ring_1d.rs        # 🎯 [实战关卡 4] 2018 PRE 一维圆环模型
├── naming_game/      # 🎯 [实战关卡 6 & 7] 命名博弈与大模型多智能体 (微观统计与四通道解耦)
│   ├── metrics.rs    # 宏观 Nw, Nd 与微观度分布 P_n(k)
│   ├── model.rs      # Direct Naming Game 谈判更新规则
│   ├── llm_model.rs  # LLM 四通道随机转移模型
│   └── network.rs    # 完全图、ER 随机图、BA 无标度、WS 小世界网络拓扑
├── minority_game/    # 🎯 [实战关卡 8] 2005 Minority Game 少数派博弈与从众羊群效应
│   ├── strategy.rs   # 记忆位运算映射、策略表与虚拟打分机制
│   ├── metrics.rs    # 归一化市场波动率 sigma^2 / N 与信息比率 alpha
│   └── model.rs      # 网络局域模仿决策状态机
├── ant_foraging/     # 🎯 [实战关卡 9] 2015 Ant Foraging 趋化偏微分方程与蚁道自组织涌现
│   ├── grid.rs       # 二维空间离散网格、五点中心拉普拉斯与守恒型一阶迎风对流
│   ├── metrics.rs    # 觅食效率、食物消耗率与信息素统计
│   └── model.rs      # 四组分反应-扩散-对流连续介质动力学系统
├── swarm_robotics/   # 🎯 [实战关卡 10] 2019 Springer 极简群体机器人可证明自组织构型
│   ├── state.rs      # Moore 邻域 8 方向、256 局部状态位域与单纯形分类 (Simplicial Vertex)
│   ├── policy.rs     # 离线全策略综合、碰撞与拓扑连通保持、方向与频次启发式矩阵
│   └── simulator.rs  # 异步离散格点世界、多智能体非阻塞步进与目标构型自组织收敛
├── turing_morphogenesis/ # 🎯 [实战关卡 11] 2018 Science Robotics 反应-扩散与群体形态发生
│   ├── morphogen.rs  # 激活子-抑制子分段线性动力学、通信图拉普拉斯与 LED 浓度映射
│   ├── edge_detector.rs # 局域加权邻居之比自适应外边缘探测器
│   ├── robot.rs      # Kilobot 三态行为状态机 (Wait / Orbit / Follow) 与斑点捕获
│   └── simulator.rs  # 多智能体连续空间世界、轮廓环绕物理运动学与截肢自愈再生
├── heterogeneous_swarms/ # 🎯 [实战关卡 12] 2024 Springer/PPSN 异构演化群体与表型可塑性
│   ├── environment.rs    # 30x30m 竞技场、4 大标量光强场 (Center, Bimodal, Linear, Banana)
│   ├── sensors.rs        # 4 象限 360° 方位与相对航向感知、无邻居默认值与光强归一化
│   ├── controller.rs     # 冻结隐层储备池 (Reservoir NN) 与 36 维全基因型协同表达
│   ├── robot.rs          # Thymio II 差速驱动动力学、硬核排斥碰撞与边界约束
│   ├── regulatory.rs     # 去中心化局域光强概率状态机 (P_green 调控表型可塑性)
│   ├── metrics.rs        # 累积光强适应度 f 与群体运动对齐序参量 \Phi
│   ├── cma_es.rs         # 协方差矩阵自适应进化策略 (CMA-ES) 优化器
│   └── simulator.rs      # 10Hz 多智能体高精度仿真引擎与评测流水线
├── morphological_swarms/ # 🎯 [实战关卡 13] 2026 arXiv 形态计算与自对齐聚集 (Sweet Spot & MIPS)
│   ├── model.rs      # 周期性边界、WCA 软排斥碰撞、速度弛豫与形态自对齐力矩更新
│   └── metrics.rs    # 光照成核聚集率 N_circ / N 与宏观极化对齐度 <Psi>
└── sons_hierarchy/       # 🎯 [实战关卡 14] 2024 Science Robotics 自组织神经系统 (SoNS) 动态多层级控制
    ├── types.rs          # 异构空中-地面机器人、目标形态槽位与相对几何位姿
    ├── allocator.rs      # Section 4.1 节点分配与自组织就近替换 (Dynamic Replacement)
    ├── controller.rs     # Section 4.1 质量-弹簧-阻尼运动学、减速死区与安全区视距保底
    ├── tree.rs           # Section 4.1 树状网络递归划分、断链自立 Brain 与规模破偶合并
    └── metrics.rs        # Section 4.2 公式 (1) 构型跟踪误差 E(t) 与公式 (2) 理论收敛下界 B(t)
```

---

## 🏆 闯关任务分解

### 🎯 关卡 1: 切片借用与函数式迭代器（序参量计算）
- **代码文件**: [`crates/swarm-core/src/metrics.rs`](crates/swarm-core/src/metrics.rs)
- **学习的核心 Rust 语法**:
  1. 只读切片借用: `phases: &[f64]`（不发生任何数据拷贝）；
  2. 数学标量运算: `.cos()`, `.sin()`, `.powi(2)`, `.sqrt()`；
  3. 两种遍历风格对比：
     - 经典循环: `for &th in phases { ... }`
     - 函数式迭代: `phases.iter().map(...).sum()`
  4. 双切片对齐遍历: `positions.iter().zip(phases.iter())`
- **任务目标**:
  1. 实现 `kuramoto_order_parameter`
  2. 实现 `ring_spatiotemporal_order_parameters`
- **验证命令**:
  ```bash
  cargo test -p swarm-core -- metrics
  ```
  *(当终端打印 `test result: ok. 3 passed` 时，即通关本卡！)*

---

### 🎯 关卡 2: 可变引用与高性能内存复用（数值积分器）
- **代码文件**: [`crates/swarm-core/src/integrator.rs`](crates/swarm-core/src/integrator.rs)
- **学习的核心 Rust 语法**:
  1. 可变借用: `state: &mut [f64]`（Rust 编译器保证同一时间排他可变，杜绝并发竞争）；
  2. **高性能系统设计模式**: 在 `Rk4Integrator` 结构体中预分配 `k1, k2, k3, k4, tmp` 缓冲区，避免在每一步迭代中反复动态申请堆内存；
  3. 面向特征（Trait）的多态调用: `system.derivative(...)`。
- **任务目标**:
  1. 先写暖身任务：一阶欧拉法 `EulerIntegrator::step`
  2. 再挑战核心任务：四阶龙格-库塔 `Rk4Integrator::step`
- **验证命令**:
  ```bash
  cargo test -p swarm-core -- integrator
  ```
  *(内置了微分方程 $\dot{y} = -y$ 的解析解高精度对比测试)*

---

### 🎯 关卡 3: 结构体与物理受力双重循环（2017 Nature 2D 模型）
- **代码文件**: [`crates/swarm-core/src/models/nature2017_2d.rs`](crates/swarm-core/src/models/nature2017_2d.rs)
- **学习的核心 Rust 语法**:
  1. 状态数组切片拆分: `let (dx, rest) = ds.split_at_mut(n);`
  2. 论文求和公式 $\sum_{j \neq i} \dots$ 转换为代码中的两层循环；
  3. 边界与距离除零防护: 欧氏距离 $r_{ij} = \sqrt{\Delta x^2 + \Delta y^2 + \epsilon}$。
- **任务目标**:
  - 在 `derivative` 函数中实现论文核心控制方程：
    $$\dot{\mathbf{x}}_i = \frac{1}{N} \sum_{j \neq i} \left[ \frac{\mathbf{x}_j - \mathbf{x}_i}{|\mathbf{x}_j - \mathbf{x}_i|} (1 + J \cos(\theta_j - \theta_i)) - \frac{\mathbf{x}_j - \mathbf{x}_i}{|\mathbf{x}_j - \mathbf{x}_i|^2} \right]$$
    $$\dot{\theta}_i = \omega_i + \frac{K}{N} \sum_{j \neq i} \frac{\sin(\theta_j - \theta_i)}{|\mathbf{x}_j - \mathbf{x}_i|}$$
- **验证与出图**:
  ```bash
  cargo run --release --example 01_natcomm2017_oscillators_sync_and_swarm
  ```
  *(如果算法正确，程序运行完毕后会自动调用 Python，打开 `output/final_state.png` 即可看到由你亲手算出的彩虹相位波！)*

---

### 🎯 关卡 4: 圆环拓扑动力学（2018 PRE 1D 模型）
- **代码文件**: [`crates/swarm-core/src/models/ring_1d.rs`](crates/swarm-core/src/models/ring_1d.rs)
- **学习的核心 Rust 语法**:
  - 圆环上角度的周期性归一化与紧凑数组表达；
  - 空间极角与内部相位的对称性双向耦合。
- **任务目标**:
  - 实现 `ring_1d.rs` 中的微分方程。
- **验证与出图**:
  ```bash
  cargo run --release --example 02_pre2018_ring_states
  ```

---

### 🚀 关卡 5: 并发与并行计算（体验 Rayon 的威力）
当你用单线程循环完成关卡 3 和 4 后，你会发现 $N=500$ 时稍有卡顿。
此时你将学习 Rust 的“无畏并发”：
1. 引入 `rayon::prelude::*`；
2. 将最外层粒子循环 `(0..n)` 改为 `(0..n).into_par_iter()`；
3. 亲身体验在无需加锁（Mutex）的情况下，多核 CPU 瞬间跑满、提速 10 倍的震撼快感。

---

### 🎯 关卡 6: 离散多智能体博弈与微观统计动力学（2006 Naming Game 命名博弈）
- **论文**: *"Microscopic activity patterns in the Naming Game"*, L. Dall'Asta, A. Baronchelli, [cond-mat/0606125](https://arxiv.org/abs/cond-mat/0606125)
- **本地文档**: [`papers/naming-game/01-microscopic-activity-condmat2006/README.md`](papers/naming-game/01-microscopic-activity-condmat2006/README.md)
- **代码文件**:
  - 宏观与微观度分布计算: [`crates/swarm-core/src/naming_game/metrics.rs`](crates/swarm-core/src/naming_game/metrics.rs)
  - Direct 博弈单步状态机: [`crates/swarm-core/src/naming_game/model.rs`](crates/swarm-core/src/naming_game/model.rs)
- **学习的核心 Rust 语法**:
  1. **集合容器**: `std::collections::HashSet` 与其集合操作（`insert`, `contains`, `clear`）；
  2. **泛型结构体与 Trait 约束**: `struct NamingGame<G: Network>`；
  3. **随机抽样与离散事件逻辑**: 从 `HashSet` 迭代器中随机抽词并根据谈判结果更新；
  4. **有序映射与直方图归一化**: 使用 `BTreeMap<usize, f64>` 构建无偏离散概率分布 $\mathcal{P}_n(k | t)$。
- **任务目标**:
  1. 在 `metrics.rs` 中实现 `total_words`, `distinct_words`, `inventory_size_distribution`, `is_consensus`；
  2. 在 `model.rs` 中实现 `NamingGame::step` 的 4 步谈判交互规则。
- **验证命令**:
  ```bash
  cargo test -p swarm-core -- naming_game::tests
  ```
  *(当看到 `test result: ok. 5 passed` 时，即通关本卡！)*
- **运行实验与出图**:
  ```bash
  cargo run --release --example 11_condmat2006_naming_game_activity
  ```
  *(程序运行完毕后会自动调用 Python，生成 `output/naming_game_macro.png` 与 `output/naming_game_micro.png`，完美复现原论文图 1、图 3、图 5！)*

---

### 🎯 关卡 7: 大模型多智能体命名博弈与微观动力学（2026 LLM Naming Game）
- **论文**: *"Microscopic dynamics of consensus formation in multi-agent LLM Naming Games"*, [arXiv:2608.02178](https://arxiv.org/abs/2608.02178) (2026)
- **本地文档**: [`papers/naming-game/02-llm-naming-game-arxiv2026/README.md`](papers/naming-game/02-llm-naming-game-arxiv2026/README.md)
- **代码文件**: [`crates/swarm-core/src/naming_game/llm_model.rs`](crates/swarm-core/src/naming_game/llm_model.rs)
- **学习的核心物理与统计机制**:
  1. **微观四通道解耦**: 将 LLM 听者回答解耦为库内通道固化率 $\pi(T) \equiv P(\text{YES} \mid w \in P_j)$ 与库外通道重涂率 $\phi(T) \equiv P(\text{YES} \mid w \notin P_j)$；
  2. **微观漂移与平均场临界线**: 验证两词平均场临界有序相条件 $3\pi - 2\phi - 1 > 0$；
  3. **三大模型架构的温度特征**:
     - LLaMA-3.1:8B: 宽容型，重涂噪声主导，$t_c \sim e^{0.67 T}$；
     - Mistral:7B: 近确定型，温度盲性（$\alpha \approx 0$）；
     - Phi-3:14B: 保守型，漏坍缩主导，呈现显著的“逆温度序”（低温词库暴涨至 30+ 形成瓶颈）。
- **运行实验与出图**:
  ```bash
  cargo run --release --example 12_arxiv2026_llm_naming_game
  ```
  *(运行完毕自动生成 `output/llm_naming_game_trajectories.png` 与 `output/llm_naming_game_phase_diagram.png`)*

---

### 🎯 关卡 8: 少数派博弈与从众羊群效应（2005 Minority Game with Herding Behavior）
- **论文**: *"Minority game with local interactions due to the presence of herding behavior"*, A. L. M. Vilela, D. O. Cajueiro et al., [physics/0512087](https://arxiv.org/abs/physics/0512087)
- **本地文档**: [`papers/minority-game/01-herding-behavior-physics0512087/README.md`](papers/minority-game/01-herding-behavior-physics0512087/README.md)
- **代码文件**:
  - 策略表与虚拟打分: [`crates/swarm-core/src/minority_game/strategy.rs`](crates/swarm-core/src/minority_game/strategy.rs)
  - 波动率与相变统计: [`crates/swarm-core/src/minority_game/metrics.rs`](crates/swarm-core/src/minority_game/metrics.rs)
  - 网络局域模仿状态机: [`crates/swarm-core/src/minority_game/model.rs`](crates/swarm-core/src/minority_game/model.rs)
- **学习的核心物理与统计机制**:
  1. **历史状态位掩码操作**: 将长度为 $M$ 的过去胜负二值序列打包为一个 `usize`（$0 \dots 2^M - 1$），实现 $O(1)$ 常数时间查表；
  2. **虚拟打分（Virtual Scoring）**: 每个 Agent 拥有 $S$ 个独立策略表，每轮不论是否执行，均根据真实少数派胜者更新策略虚拟分数 $U_{i, s}(t+1) = U_{i, s}(t) - a_i^s(t) \cdot \text{sgn}(A(t))$；
  3. **网络局域从众（Herding Imitation）**: 遍历 `network.neighbors(agent_id)`，寻找邻域得分最高者。若自身得分低于邻居最高分，则放弃自主策略，盲从模仿该邻居动作；
  4. **相变破坏与市场振荡**: 验证经典 $\alpha_c \approx 0.34$ 最优协调相为何在强局部从众下被彻底抹平，波动率 $\sigma^2/N$ 暴涨数倍。
- **验证命令**:
  ```bash
  cargo test -p swarm-core -- minority_game
  ```
- **运行实验与出图**:
  ```bash
  cargo run --release --example 13_physics0512087_minority_game_herding
  uv run python python/plot_minority_game.py
  ```
  *(生成 `output/minority_game_volatility.png` 与 `output/minority_game_timeseries.png`)*

---

### 🎯 关卡 9: 偏微分方程连续介质趋化与自组织蚁道涌现（2015 Ant Foraging via Chemotaxis）
- **论文**: *"Modeling ant foraging: a chemotaxis approach with pheromones and trail formation"*, Paulo Amorim, [arXiv:1409.3808](https://arxiv.org/abs/1409.3808) (*J. Theor. Biol.* 2015)
- **本地文档**: [`papers/ant-foraging/01-chemotaxis-trail-formation-arxiv1409/README.md`](papers/ant-foraging/01-chemotaxis-trail-formation-arxiv1409/README.md)
- **代码文件**:
  - 空间网格与偏微分算子: [`crates/swarm-core/src/ant_foraging/grid.rs`](crates/swarm-core/src/ant_foraging/grid.rs)
  - 四场耦合动力学状态机: [`crates/swarm-core/src/ant_foraging/model.rs`](crates/swarm-core/src/ant_foraging/model.rs)
  - 统计指标与质量守恒: [`crates/swarm-core/src/ant_foraging/metrics.rs`](crates/swarm-core/src/ant_foraging/metrics.rs)
- **学习的核心物理与数值计算机制**:
  1. **连续介质 PDE 建模**:
     - 觅食蚁场 $u$（扩散 + 沿 $\nabla v$ 趋化平流 + 接触食物转化为 $w$）；
     - 搬运蚁场 $w$（向巢穴定向迁移 $\nabla a$ + 回巢在 $N(x)$ 处卸货并重新转化为 $u$）；
     - 信息素场 $v$（扩散 + 自然挥发 $-\varepsilon v$ + 铺路源项 $P(x) w$ 近巢穴抑制）；
     - 食物源场 $c$（质量作用定律单调消耗 $\partial_t c = -uc$）。
  2. **数值迎风格式（Conservative Upwind Scheme）**:
     - 在网格控制界面采用基于流向的通量选取，避免高 Peclet 数平流数值发散；
     - 保证质量严格守恒与解的物理非负性（$\iint (u+w) dx dy = \text{const}$）。
  3. **自催化正反馈与路径涌现**:
     - 初始随机扩散 $\to$ 发现食物 $\to$ 铺设信息素 $\to$ 招募更多觅食蚁 $\to$ 自发形成高速蚁道；
     - 食物耗尽后信息素自然衰减，蚁道自发解体。
  4. **参数空间与搬运效率**:
     - 验证论文核心结论：自组织蚁道的形成显著加速了食物的运载效率。
- **验证命令**:
  ```bash
  cargo test -p swarm-core -- ant_foraging
  ```
- **运行实验与出图**:
  ```bash
  cargo run --release --example 14_arxiv1409_ant_chemotaxis_foraging
  uv run python python/plot_ant_chemotaxis.py
  ```
  *(生成 `output/ant_chemotaxis_spatial_fields.png`、`output/ant_chemotaxis_trails_evolution.png` 与 `output/ant_chemotaxis_efficiency.png`)*

### 🎯 关卡 10: 极简认知群体机器人可证明自组织构型 (Provable Swarm Pattern Formation)
- **代码文件**:
  - 局域观测与拓扑单纯形分类: [`crates/swarm-core/src/swarm_robotics/state.rs`](crates/swarm-core/src/swarm_robotics/state.rs)
  - 离线全策略生成与启发式转移矩阵: [`crates/swarm-core/src/swarm_robotics/policy.rs`](crates/swarm-core/src/swarm_robotics/policy.rs)
  - 异步离散仿真与自组织收敛世界: [`crates/swarm-core/src/swarm_robotics/simulator.rs`](crates/swarm-core/src/swarm_robotics/simulator.rs)
- **学习的核心物理与机器人分布式控制机制**:
  1. **极简感知与动作模型 (Minimalist Sensor/Actuator Model)**:
     - 机器人处于 2D Moore 网格环境，仅感知 8 邻域是否有机器人，状态空间有限且封闭（$s \in \{0, 1\}^8$，共 256 种局部状态）；
     - 动作全向可选（$a \in \{1..8\}$，共 8 个平移方向）；无记忆、无全局坐标系、无通信握手。
  2. **可证明安全无死锁策略 ($\Pi_{\text{safe}}$)**:
     - 碰撞规避策略（Collision Avoidance）：禁止移向已有机器人的格子；
     - 局部拓扑连通保持（Local Separation Avoidance）：引入单纯形节点分类（Simplicial Vertex），仅当局部邻域诱导子图连通时才允许移动，严格保证整个群体网络在移动中保持全局连通图。
  3. **目标构型期望状态与对称性分解 ($S_{\text{des}}$ & $\Pi_f$)**:
     - 从几何构型模板（如 Triangle-4, Square-4, Hexagon-6 等）中提取所有构成此构型的局部状态集合 $S_{\text{des}}$；
     - 定义构型收敛准则：当且仅当所有机器人均满足 $s_i \in S_{\text{des}}$ 时，群体达到吸收态终止移动。
  4. **启发式收敛加速策略 (ALT1 / ALT2)**:
     - 匹配方向矩阵 $D(S_{\text{des}})$ 与匹配频次计数 $M(S_{\text{des}})$；
     - 引导机器人优先选择能更快形成期望邻域结构的动作，显著减少随机抖动与收敛步数。
- **验证命令**:
  ```bash
  cargo test -p swarm-core -- swarm_robotics
  ```
- **运行实验与出图**:
  ```bash
  cargo run --release --example 15_springer2019_swarm_robotics_pattern_formation
  uv run python python/plot_swarm_pattern_formation.py
  ```
  *(生成 `output/swarm_pattern_formation_grid.png` 与 `output/swarm_pattern_formation_histograms.png`)*

### 🎯 关卡 11: 图灵反应-扩散形态素与群体形态发生 (Turing Morphogenesis in Robot Swarms)
- **代码文件**:
  - 形态素动力学与图拉普拉斯: [`crates/swarm-core/src/turing_morphogenesis/morphogen.rs`](crates/swarm-core/src/turing_morphogenesis/morphogen.rs)
  - 局域加权边缘探测器: [`crates/swarm-core/src/turing_morphogenesis/edge_detector.rs`](crates/swarm-core/src/turing_morphogenesis/edge_detector.rs)
  - Kilobot 行为状态机: [`crates/swarm-core/src/turing_morphogenesis/robot.rs`](crates/swarm-core/src/turing_morphogenesis/robot.rs)
  - 连续世界多智能体仿真器: [`crates/swarm-core/src/turing_morphogenesis/simulator.rs`](crates/swarm-core/src/turing_morphogenesis/simulator.rs)
- **学习的核心物理与群体生物形态发生机制**:
  1. **动态网络上的图灵反应-扩散 (Turing Reaction-Diffusion on Dynamic Graphs)**:
     - 激活子 $u$ 与抑制子 $v$ 沿近邻红外通信网络扩散；
     - 满足长程抑制、短程激活图灵条件（$D_v / D_u = 20 \gg 1$），在初始均一网络上自发失稳形成空间周期性极化斑点（Polarized Turing Spots $u > 4.0$）；
     - 针对嵌入式低功耗微处理器的分段线性饱和动力学（Piecewise-linear Kinetics）。
  2. **完全去中心化的局域边缘识别 (Decentralized Boundary Detection)**:
     - 机器人仅依据自身邻居数与邻居的距离加权邻居均值之比（$\bar{N}_i / \bar{N}_{\mathcal{N}_i} < 0.8$）精准识别处于外边界的节点。
  3. **形态素引导的组织流动与差异性生长 (Turing-Guided Boundary Outgrowth)**:
     - 未极化边缘机器人沿外轮廓巡航环绕（`ORBIT`），直至抵达极化斑点核心区域时被捕获固化（`WAIT`）；
     - 在无中央指令与坐标系的条件下，自发沿图灵激活斑点位置生长出指状突起肢体构型（Protrusions / Lobes）。
  4. **损伤断肢切除与形态自愈再生 (Amputation & Self-Healing Regeneration)**:
     - 剪切切断部分突起指状分支后，剩余群体红外拓扑重构，自发重新孕育图灵斑点并再次启动组织流动再生新形态。
- **验证命令**:
  ```bash
  cargo test -p swarm-core -- turing_morphogenesis
  ```
- **运行实验与出图**:
  ```bash
  cargo run --release --example 16_scirobotics2018_turing_morphogenesis
  uv run python python/plot_turing_morphogenesis.py
  ```
  *(生成 `output/turing_morphogenesis_snapshots.png` 与 `output/turing_morphogenesis_metrics.png`)*

---

### 🎯 关卡 12: 异构群体演化与表型可塑性集体感知 (2024 Springer/PPSN Heterogeneous Swarms)
- **论文**: *"Emergence of Specialised Collective Behaviors in Evolving Heterogeneous Swarms"*, Fuda van Diggelen, Matteo de Carlo, Nicolas Cambier, Eliseo Ferrante, A. E. Eiben (*PPSN XVIII 2024*, Springer LNCS 14965, [DOI: 10.1007/978-3-031-70068-2_4](https://doi.org/10.1007/978-3-031-70068-2_4), [arXiv:2402.04763](https://arxiv.org/abs/2402.04763))
- **本地文档**: [`papers/heterogeneous-swarms/01-specialised-collective-behaviors-springer2024/README.md`](papers/heterogeneous-swarms/01-specialised-collective-behaviors-springer2024/README.md)
- **代码文件**:
  - 竞技场与标量光强场: [`crates/swarm-core/src/heterogeneous_swarms/environment.rs`](crates/swarm-core/src/heterogeneous_swarms/environment.rs)
  - 4 象限受限感知系统: [`crates/swarm-core/src/heterogeneous_swarms/sensors.rs`](crates/swarm-core/src/heterogeneous_swarms/sensors.rs)
  - 储备池神经网络与基因型: [`crates/swarm-core/src/heterogeneous_swarms/controller.rs`](crates/swarm-core/src/heterogeneous_swarms/controller.rs)
  - Thymio II 动力学与碰撞: [`crates/swarm-core/src/heterogeneous_swarms/robot.rs`](crates/swarm-core/src/heterogeneous_swarms/robot.rs)
  - 在线表型可塑性调控: [`crates/swarm-core/src/heterogeneous_swarms/regulatory.rs`](crates/swarm-core/src/heterogeneous_swarms/regulatory.rs)
  - 适应度与对齐序参量: [`crates/swarm-core/src/heterogeneous_swarms/metrics.rs`](crates/swarm-core/src/heterogeneous_swarms/metrics.rs)
  - CMA-ES 进化优化器: [`crates/swarm-core/src/heterogeneous_swarms/cma_es.rs`](crates/swarm-core/src/heterogeneous_swarms/cma_es.rs)
  - 高精度群体仿真引擎: [`crates/swarm-core/src/heterogeneous_swarms/simulator.rs`](crates/swarm-core/src/heterogeneous_swarms/simulator.rs)
- **学习的核心物理与进化算法机制**:
  1. **无显式通信下的分工涌现 (Division of Labor without Communication)**:
     - 仅依靠群体级累积光强适应度 $f = \frac{\sum l_t}{G_{\max} T}$ 引导进化，控制器无记忆、无领航者、不知晓分工；
  2. **储备池计算与协同进化 (Reservoir Neural Network & CMA-ES)**:
     - 随机隐层权重生成后严格冻结，CMA-ES 仅演化输出层 18 维权重（双子群共 36 维全基因型）；
  3. **子群专门化协同效应 (Table 3 复现)**:
     - 绿色子群（Exploitative 剥削利用型）在强光区高密度聚集；红色子群（Exploratory 探索协调型）在弱光区高对齐度巡航；
     - 远距离均质混合比例（2:2 / 1:3）显著优于单一极端子群（4:0 / 0:4），呈现 $1+1 > 2$ 的协同涌现；
  4. **去中心化在线表型可塑性调控 (Table 4 复现)**:
     - 依据局域标量光强以概率 $P_{\text{green}}(\text{light})$ 每 5.0s 动态重抽样表达控制器；
     - 在群体可扩展性（$N = 10, 20, 50$）与跨环境鲁棒性（Center, Bi-modal, Linear, Banana）中全面超越同质基准。
- **验证命令**:
  ```bash
  cargo test -p swarm-core -- heterogeneous_swarms
  ```
- **运行实验与出图**:
  ```bash
  cargo run --release --example 17_springer2024_heterogeneous_swarms
  uv run python python/plot_heterogeneous_swarms.py
  ```
  *(生成 `output/heterogeneous_swarms_learning_curves.png`、`output/heterogeneous_swarms_subgroup_ratios.png`、`output/heterogeneous_swarms_scalability_robustness.png` 与 `output/heterogeneous_swarms_trajectories.png`)*

---

### 🎯 关卡 13: 形态计算与自对齐聚集 (Morphological Computing & MIPS Phototaxis)
- **代码文件**:
  - 物理模型与周期性边界: [`crates/swarm-core/src/morphological_swarms/model.rs`](crates/swarm-core/src/morphological_swarms/model.rs)
  - 宏观序参量与团簇网络: [`crates/swarm-core/src/morphological_swarms/metrics.rs`](crates/swarm-core/src/morphological_swarms/metrics.rs)
- **学习的核心物理与形态学计算机制**:
  1. **形态计算（Morphological Computation）与非对称受力**:
     - 机器人机身外骨骼的非对称摩擦与质量分布，在遭遇物理接触力时产生自发偏转力矩；
     - 仅依靠 1:15 间歇启闭（PWM 占空比降低速度至 $v_\circ / v_\bullet = 1/3$），**单机器人无法独立停在光照区**，完全依赖形态力矩诱发群体动力学聚集。
  2. **向量三重积力矩与转向微分动力学**:
     - 动力学控制方程：$\tau_n \dot{\vec{n}} = \epsilon (\vec{n} \times \vec{v}) \times \vec{n} + \sqrt{2D} \xi \vec{n}_{\perp}$；
     - 2D 向量化实现：$\dot{n}_x \propto \kappa (n_y^2 v_x - n_x n_y v_y)$，$\dot{n}_y \propto \kappa (n_x^2 v_y - n_x n_y v_x)$；
     - Aligner ($\kappa > 0$) 碰撞同向对齐 $\to$ 极化游弋流（Flocking，$\langle \Psi \rangle > 0.8$）；
     - Fronter ($\kappa < 0$) 碰撞反向咬合 $\to$ 瞬时速度衰减至零。
  3. **形态“甜点区 (Sweet Spot)”与相变相图**:
     - 弱逆对齐 ($\kappa > -0.6$)：成核不足以抗衡噪声，聚集率接近随机游走基线 0.18；
     - 强逆对齐 ($\kappa < -2.2$)：在全场（包括暗区）发生碰撞冷冻死锁，导致反向趋光；
     - 最佳甜点区 ($\kappa \in [-2.0, -0.6]$)：碰撞减速与光照区慢行形成正反馈，通过动力学诱导相分离（MIPS）在光照区涌现出高达 40% 的自发聚集。
- **验证命令**:
  ```bash
  cargo test -p swarm-core -- morphological_swarms
  ```
- **运行实验与出图**:
  ```bash
  cargo run --release --example 18_arxiv2026_morphological_aggregating_swarms
  uv run python python/plot_morphological_aggregating_swarms.py
  ```
  *(生成 `output/morphological_swarms_phase_diagram.png` 与 `output/morphological_swarms_spatial_snapshots.png`)*

---

### 🎯 关卡 14: 自组织神经系统 (SoNS) 动态多层级控制与就近置换
- **代码文件**:
  - 核心类型定义: [`crates/swarm-core/src/sons_hierarchy/types.rs`](crates/swarm-core/src/sons_hierarchy/types.rs)
  - 节点分配与就近置换 (Section 4.1): [`crates/swarm-core/src/sons_hierarchy/allocator.rs`](crates/swarm-core/src/sons_hierarchy/allocator.rs)
  - 质量-弹簧-阻尼运动学控制: [`crates/swarm-core/src/sons_hierarchy/controller.rs`](crates/swarm-core/src/sons_hierarchy/controller.rs)
  - 拓扑生命周期与子树维护: [`crates/swarm-core/src/sons_hierarchy/tree.rs`](crates/swarm-core/src/sons_hierarchy/tree.rs)
  - 跟踪误差与理论下界 (Section 4.2): [`crates/swarm-core/src/sons_hierarchy/metrics.rs`](crates/swarm-core/src/sons_hierarchy/metrics.rs)
- **学习的核心理论与算法机制 (Science Robotics 2024 / arXiv:2401.13103)**:
  1. **零先验树状网络自组织建立与递归委派 (Section 4.1)**:
     - 每台机器人初始均为自身 SoNS 的独立 Brain；
     - 招募链接建立后，Parent 将子图 $G'_i \subset G$ 递归分发给 Child，实现自顶向下的分层管辖。
  2. **自组织节点匹配与就近置换机制 (Dynamic Replacement)**:
     - 传统层级网络遭遇外围新节点时需逐跳向下交接（Link-by-link handover），引发严重的级联延迟；
     - SoNS 允许 Parent 对已分配槽位执行**就近替换**：将更靠近目标槽位的新 Candidate 填入，将原 Child 降级为候选者参与下一轮重分配，群体内部产生涟漪顺移，大幅加速几何构型收敛。
  3. **集群裂变与规模破偶合并 (Splitting & Merging)**:
     - 遭遇障碍或断链时，被割离节点自立为新 Brain，并**完整保留其下游子树结构**；
     - 两群相遇时，以总子树规模 $\text{scale}$ 与内部 Rank 评估主从，劣势方整树并入优势方。
  4. **质量-弹簧-阻尼运动控制与安全区保底**:
     - 死区停止 $r_{\text{stop}}$、线性减速 $r_{\text{slow}}$、视距安全区 $r_{\text{safe}}$ 约束与势场避障叠加。
  5. **构型收敛评估指标 (Section 4.2)**:
     - 公式 (1) 平均欧氏跟踪误差 $E(t) = \frac{1}{n} \sum |d(\mathbf{p}_i - \mathbf{p}_1) - d(\mathbf{f}_i - \mathbf{f}_1)|$；
     - 公式 (2) 物理极限直线速度理论下界 $B(t) = \frac{1}{n} \sum \max(0, d_0 - \kappa_i t)$。
- **验证命令**:
  ```bash
  cargo test -p swarm-core -- sons_hierarchy
  ```
- **运行实验与出图**:
  ```bash
  cargo run --example 19_arxiv2401_sons_self_organizing_hierarchy
  uv run python python/plot_sons_self_organizing_hierarchy.py
  ```
  *(生成 `output/sons_self_organizing_hierarchy.png`，呈现三阶段空间拓扑演化、跟踪误差收敛曲线与集群合并演化)*

---

## 💡 卡壳求助指南（如何使用参考答案）

如果你在编写过程中：
- 遇到了借用检查器报错（如 `cannot borrow *state as mutable more than once at a time`）；
- 或者公式推导遇到疑问；

你可以：
1. 打开对应的 `crates/swarm-core/src/reference/` 参考代码，对比自己的实现；
2. 在 `examples/` 脚本中，随时可以临时取消注释参考答案的引用进行效果对比：
   ```rust
   // 切换为参考答案快速验证效果：
   // use swarm_core::reference::integrator::Rk4Integrator;
   // use swarm_core::reference::nature2017_2d::Swarmalator2D;
   ```
