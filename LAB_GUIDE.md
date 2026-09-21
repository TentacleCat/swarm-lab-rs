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
│   └── ring_1d.rs
│
├── metrics.rs        # 🎯 [实战关卡 1] 切片借用与序参量计算 (从这里开始！)
├── integrator.rs     # 🎯 [实战关卡 2] 可变借用、内存复用与 RK4 积分器
└── models/
    ├── nature2017_2d.rs  # 🎯 [实战关卡 3] 2017 Nature Comms 2D 经典模型
    └── ring_1d.rs        # 🎯 [实战关卡 4] 2018 PRE 一维圆环模型
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
