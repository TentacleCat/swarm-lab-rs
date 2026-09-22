# Paper Reproduction: Provable self-organizing pattern formation by a swarm of robots with limited knowledge

> **Paper**: *Provable self-organizing pattern formation by a swarm of robots with limited knowledge*  
> **Authors**: Mario Coppola, Jian Guo, Eberhard Gill, Guido C. H. E. de Croon (Delft University of Technology / TU Delft)  
> **Journal**: *Swarm Intelligence*, 13(1):59–94 (2019)  
> **DOI**: [10.1007/s11721-019-00163-0](https://doi.org/10.1007/s11721-019-00163-0)  
> **Local PDF**: [`papers/swarm-robotics/01-provable-pattern-formation-swarm-intel2019/paper.pdf`](paper.pdf)  

---

## 1. 核心理论背景 (Background & Motivation)

在微型无人机群（Micro Air Vehicles, MAVs）、纳米机器人与微纳卫星集群中，个体往往受到极致的算力、能耗与载荷限制：
- **无法搭载昂贵的高精度全局定位系统（如 GPS 拒止环境）**；
- **难以承受高带宽、高功耗的机间无线通信（通信拥塞与干扰）**；
- **计算芯片仅能执行最简单的反应式逻辑（无复杂历史记忆）**。

传统集群构型控制多依赖全局坐标、中心化指派、种子机器人（Seed/Leader）或分布式共识通信。而本论文由荷兰代尔夫特理工大学（TU Delft）团队提出，开创性地解决了**在极度受限（无通信、无身份、无记忆、无全局坐标、无种子）条件下，如何由局部随机行为映射自发涌现全局期望构型，并给出了严格的形式化可证明性（Provable Liveness & Safety）**。

---

## 2. 机器人认知约束与物理假设

### 2.1 极简认知约束 (Constraints C1 ~ C10)
1. **同构 (Homogeneous)**：所有机器人完全一致；
2. **匿名 (Anonymous)**：无法区分邻居机器人的身份 ID；
3. **反应式 (Reactive)**：仅基于当前局部感知选择动作；
4. **无记忆 (Memoryless)**：不记录历史轨迹与过去状态；
5. **无种子 / 无领航者 (No Leader/Seed)**：没有特殊参考机器人；
6. **无通信 (Non-communicating)**：机器人之间完全不收发任何信息包；
7. **纯局部状态 (Local State Only)**：仅感知紧邻视距内的邻居存在性；
8. **无全局坐标 (No Global Position)**：不知晓自己在空间中的绝对坐标；
9. **无界空间 (Unbounded Space)**：在自由二维平面中作业；
10. **极短探测距 (Short Range)**：仅感知邻近 8-邻域网格（Moore 邻域）。

### 2.2 基础假设 (Assumptions A1 ~ A4)
- **A1 共同方向参考**：全体知晓共同北向（由简易板载磁力计或陀螺仪提供）；
- **A2 平面移动**：机器人在二维平面移动；
- **A3 相对局部动静检测**：能以一定频率探测邻居当前是悬停还是在移动；
- **A4 初始连通拓扑**：初始形态 $P_0$ 的感知图必须是单一连通分量（否则无界空间中失散群体无法相聚）。

---

## 3. 概率局域状态-动作映射策略 ($\Pi_f$)

每个机器人周围有 8 个方向：
$$l_1(\text{N}), \, l_2(\text{NE}), \, l_3(\text{E}), \, l_4(\text{SE}), \, l_5(\text{S}), \, l_6(\text{SW}), \, l_7(\text{W}), \, l_8(\text{NW})$$
局部状态空间共 $|S| = 2^8 = 256$ 种二值邻域分布。动作空间包含 8 个网格平移步 $A = \{a_1, \dots, a_8\}$。

策略设计基于三大生物学启发的拟人原则：
1. **谨慎原则 (Be Careful - 防碰撞)**：
   若方向 $a_k$ 上已有邻居占据（$l_k = 1$），则该动作属于碰撞集合 $\Pi_{\text{collision}}$，禁止执行；
2. **社交原则 (Be Social - 维系集群连通)**：
   若机器人从 $(0, 0)$ 移向 $a_k$ 后，其所有原有邻居与自身无法在局部子图中保持连通，则该动作属于分裂集合 $\Pi_{\text{separation}}$，禁止执行；
   $$\Pi_{\text{safe}} = \Pi \setminus \big(\Pi_{\text{collision}} \cup \Pi_{\text{separation}}\big)$$
3. **满意原则 (Be Happy - 目标构型编码)**：
   对于设计的目标构型 $P_{\text{des}}$（如三角形、四边形、六边形），提取该构型中所有节点呈现的局部期望状态集合 $S_{\text{des}}$。当机器人的局部状态 $s \in S_{\text{des}}$ 时，它处于“满意状态”，**禁止执行任何移动**：
   $$\Pi_f = \Pi_{\text{safe}} \setminus \big(S_{\text{des}} \times A\big)$$

---

## 4. 可证明性分析 (Formal Verification)

- **安全性 (Safety)**：在 $\Pi_{\text{safe}}$ 约束下，异步单步执行绝不产生碰撞，且集群始终维持单连通拓扑；
- **活锁证明 (Livelock Freedom)**：
  定义**单纯形状态 (Simplicial State, $S_{\text{simplicial}}$)**，即邻居构成单一团簇的活跃状态。处于单纯形状态的边缘个体可自由在集群外缘爬行滑动，从而打破死循环；
- **死锁证明 (Deadlock Freedom)**：
  构建**匹配方向矩阵 $D(S_{\text{des}})$** 与**匹配度矩阵 $M(S_{\text{des}})$**，利用握手定理（Handshaking Theorem）与生成树拓扑搜索，严格证明除 $P_{\text{des}}$ 之外不存在其他全体满足 $S_{\text{des}}$ 的伪构型（Spurious Patterns）。

---

## 5. 构型案例与优化加速启发式 (ALT1 / ALT2)

论文在第 5 节中验证了多种经典构型：
- **Triangle-4 (4 机器人三角形)**：底层 3 个，顶层 1 个；
- **Square-4 (4 机器人正方形)**：$2 \times 2$ 紧密阵列；
- **Hexagon-6 (6 机器人正六边形)**：环状闭合回路；
- **Triangle-9 (9 机器人大型三角形)**：底层 5 个，中层 3 个，顶层 1 个。

为减少构型耗时，提出了 **ALT1**（刚移动过的机器人下一时间步冷却降权）与 **ALT2**（限制高度拥挤节点移动），显著降低收敛步数。

---

## 6. 一键运行与出图指令

```bash
# 1. 运行核心单元测试 (状态分解、防碰撞、防断连、构型收敛)
cargo test -p swarm-core -- swarm_robotics

# 2. 运行构型自组织 Monte Carlo 仿真与时序导出
cargo run --release --example 15_springer2019_swarm_robotics_pattern_formation

# 3. 生成高清构型空间演化图与收敛步数直方图
uv run python python/plot_swarm_pattern_formation.py
```
