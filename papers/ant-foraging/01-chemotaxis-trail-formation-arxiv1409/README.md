# Paper Reproduction: Modeling ant foraging: a chemotaxis approach with pheromones and trail formation

> **Paper**: *Modeling ant foraging: a chemotaxis approach with pheromones and trail formation*  
> **Author**: Paulo Amorim  
> **arXiv**: [arXiv:1409.3808](https://arxiv.org/abs/1409.3808)  
> **Journal**: *Journal of Theoretical Biology*, 385:160–173 (2015)  
> **DOI**: [10.1016/j.jtbi.2015.08.026](https://doi.org/10.1016/j.jtbi.2015.08.026)  
> **Local PDF**: [`papers/ant-foraging/01-chemotaxis-trail-formation-arxiv1409/paper.pdf`](paper.pdf)  

---

## 1. 论文核心概述 (Overview)

自然界中，真社会性昆虫（如蚂蚁）通过极简单的局域行为规则与化学信息素交流，能够自组织涌现出高度优化的群体觅食网络（Ant Foraging Trails）。过去的研究大多采用基于个体的元胞自动机（Cellular Automaton）或多智能体离散移动模拟，难以刻画连续介质极限和进行严格的偏微分方程（PDE）偏分析。

本论文将经典的 **Keller-Segel 趋化模型（Chemotaxis）** 进行多组分扩展，首次提出了一个**涵盖“搜寻-发现-招募-路径涌现-食物运回-食物耗尽后路径消散”全生命周期**的连续非线性偏微分方程组。

---

## 2. 偏微分方程动力学模型 (The Mathematical Model)

### 2.1 四大耦合连续场变量
设在二维物理区域 $\Omega \subset \mathbb{R}^2$ 中：
- $u(t, x, y)$：**觅食蚁密度 (Foraging Ants)**，尚未携带食物、处于随机搜寻或循信息素追踪状态；
- $w(t, x, y)$：**搬运蚁密度 (Returning Ants)**，已取得食物、正沿导向势场回巢；
- $v(t, x, y)$：**信息素浓度 (Pheromone Concentration)**，由搬运蚁释放并向空间扩散与挥发；
- $c(t, x, y)$：**食物源浓度 (Food Source)**，位于特定区域，可被 $u$ 蚂蚁接触消耗。

### 2.2 无量纲偏微分控制方程组 (Nondimensional System)
$$
\begin{cases}
\partial_t u - \Delta u + \nabla \cdot \big(u \, \chi_u \nabla v\big) = - u c + \lambda w N(x) + M(t) N(x) \\[6pt]
\partial_t w - D_w \Delta w + \nabla \cdot \big(w \nabla a\big) = u c - \lambda w N(x) \\[6pt]
\partial_t v - D_v \Delta v = P(x) w - \varepsilon v \\[6pt]
\partial_t c = - u c
\end{cases}
$$

### 2.3 物理机制项的微观含义
1. **觅食蚁方程（$\partial_t u$）**：
   - $\Delta u$：各向同性随机扩散（Fick 定律，无定向探索）；
   - $\nabla \cdot (u \, \chi_u \nabla v)$：**趋化平流漂移项**，蚂蚁感知信息素梯度并沿着 $\nabla v$ 浓度递增方向移动（$\chi_u$ 为趋化敏感度）；
   - $- u c$：觅食蚁到达食物源处拾取食物，转化为搬运蚁；
   - $+\lambda w N(x)$：搬运蚁抵达巢穴入口 $N(x)$ 后，迅速卸下食物重新转化为觅食蚁继续外出；
   - $+M(t) N(x)$：起始阶段从巢穴涌出的觅食蚁源项。
2. **搬运蚁方程（$\partial_t w$）**：
   - $D_w \Delta w$：搬运蚁扩散（$D_w = 0.1 \ll 1$，回巢蚂蚁路径较确定）；
   - $\nabla \cdot (w \nabla a)$：**回巢导向平流项**。$\nabla a(x)$ 为指向巢穴的方向场（模拟路标导航、路径整合 Path Integration 与地磁/光照偏振引导）；
   - $+ u c - \lambda w N(x)$：食物点转化生成与巢穴处卸货消失。
3. **信息素方程（$\partial_t v$）**：
   - $D_v \Delta v$：信息素在空间扩散（$D_v = 0.1$）；
   - $+ P(x) w$：搬运蚁在回程中释放信息素。其中 $P(x) \in [0, 1]$ 具有巢穴近旁衰减抑制特性（接近巢穴处释放量渐降为 0），**有效避免巢穴入口附近信息素严重饱和堵塞**；
   - $-\varepsilon v$：信息素自然化学衰减与挥发（$\varepsilon$ 为无量纲挥发率）。
4. **食物消耗方程（$\partial_t c$）**：
   - 食物被觅食蚁局域按接触质量作用定律单调消耗。

### 2.4 边界条件与质量守恒
在区域边界 $\partial \Omega$ 采用零通量 Neumann 边界条件：
$$
(\nabla u - \chi_u u \nabla v) \cdot \mathbf{n} = 0, \quad (D_w \nabla w - w \nabla a) \cdot \mathbf{n} = 0, \quad \nabla v \cdot \mathbf{n} = 0
$$
保证在蚂蚁完全出巢后，全场蚂蚁总质量严格守恒：
$$
\int_\Omega [u(t, x) + w(t, x)] \, dx = C_{\text{total}} = \text{const}.
$$

---

## 3. 数值方案设计 (Conservative Upwind Finite Difference)

为了保证计算精度、严格质量守恒并防止出现非物理的负密度：
1. **拉普拉斯项 $\Delta$**：标准五点中心差分格式；
2. **对流/趋化散度项 $\nabla \cdot (\phi \mathbf{V})$**：采用**守恒型一阶迎风格式（First-Order Upwind Scheme）**：
   - 对于网格交界面 $(i+1/2, j)$，依据速度分量符号决定选取上游网格值：
     $$F^x_{i+1/2, j} = \begin{cases} V^x_{i+1/2, j} \phi_{i, j} & V^x \ge 0 \\ V^x_{i+1/2, j} \phi_{i+1, j} & V^x < 0 \end{cases}$$
   - 离散散度 $\nabla \cdot (\phi \mathbf{V})_{i,j} \approx \frac{F^x_{i+1/2, j} - F^x_{i-1/2, j}}{\Delta x} + \frac{F^y_{i, j+1/2} - F^y_{i, j-1/2}}{\Delta y}$；
3. **时间推进**：显式前向欧拉法或四阶龙格库塔法（RK4），取稳定步长 $dt \le \frac{dx^2}{4 \max(1, D_w, D_v)}$。

---

## 4. 关键物理发现与相变探索

1. **自发路径涌现（Spontaneous Trail Formation）**：
   虽然方程并未直接规定蚂蚁运动轨迹，但通过“随机发现食物 $\to$ 搬运蚁回巢释放信息素 $\to$ 趋化正反馈招募更多觅食蚁”的自催化机制，系统在巢穴与各食物源之间自发建立起致密、清晰的双向蚁道。
2. **食物耗尽后的自发消散（Trail Dissipation）**：
   当某处食物耗尽，该处向搬运蚁的转化终止，信息素迅速自然挥发，蚁道自动解体，群体重新散开寻找新食物源。
3. **$(\varepsilon, \chi_u)$ 参数空间与觅食效率最优解**：
   - **$\varepsilon$ 过大 / $\chi_u$ 过小**：信息素瞬间挥发，蚂蚁几乎无法感应，退化为低效的纯布朗扩散；
   - **$\varepsilon$ 过小 / $\chi_u$ 过大**：旧信息素弥散全图不散，导致严重误报与蚂蚁局部打转锁死；
   - **最佳协作带**：只有在特定的中适参数脊线上，蚁道最明显，食物运载耗时最短（觅食效率最高）。

---

## 5. 快速运行与可视化

```bash
# 1. 运行核心单元测试
cargo test -p swarm-core -- ant_foraging

# 2. 运行双食物源自组织觅食仿真
cargo run --release --example 14_arxiv1409_ant_chemotaxis_foraging

# 3. 生成 4 场时空演化热力图与食物消耗曲线
uv run python python/plot_ant_chemotaxis.py
```
