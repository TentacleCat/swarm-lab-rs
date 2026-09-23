//! # Ant Chemotaxis Foraging PDE Model
//!
//! 基于 arXiv:1409.3808 (*Journal of Theoretical Biology*, 2015) 的
//! 四组分非线性趋化偏微分方程组：

#![allow(unused_variables, dead_code)]
//!
//! $$
//! \begin{cases}
//! \partial_t u = \Delta u - \nabla \cdot (u \, \chi_u \nabla v) - u c + \lambda w N(x) + M(t) N(x) \\
//! \partial_t w = D_w \Delta w - \nabla \cdot (w \nabla a) + u c - \lambda w N(x) \\
//! \partial_t v = D_v \Delta v + P(x) w - \varepsilon v \\
//! \partial_t c = - u c
//! \end{cases}
//! $$

use super::grid::Grid2D;
use super::metrics::AntForagingMetrics;

/// 趋化觅食模型参数配置
#[derive(Debug, Clone)]
pub struct AntChemotaxisConfig {
    /// 觅食蚁对信息素梯度的趋化敏感度 $\chi_u$（论文典型取值 20 ~ 100，默认 50.0）
    pub chi_u: f64,
    /// 搬运蚁扩散率相对于觅食蚁的比例 $D_w = \alpha_w / \alpha_u$（论文取 0.1）
    pub d_w: f64,
    /// 信息素扩散率相对于觅食蚁的比例 $D_v = \alpha_v / \alpha_u$（论文取 0.1）
    pub d_v: f64,
    /// 信息素挥发/降解率 $\varepsilon$（论文典型取值 0.1 ~ 5.0，默认 0.5）
    pub epsilon: f64,
    /// 搬运蚁在巢穴快速转化为觅食蚁的速率 $\lambda$（默认 70.0）
    pub lambda: f64,
    /// 搬运蚁回巢目标速度标量 $s_{\text{ret}}$（论文无量纲值约为 18.35）
    pub return_speed: f64,
    /// 巢穴半径（无量纲，对应物理约 10 cm）
    pub nest_radius: f64,
    /// 靠近巢穴抑制信息素释放的距离尺度 $r_{\text{fade}}$
    pub phero_fade_radius: f64,
    /// 从巢穴涌出蚂蚁的持续时间 $T_{\text{emerge}}$
    pub emerge_time: f64,
    /// 从巢穴涌出蚂蚁的速率 $C_M$
    pub emerge_rate: f64,
}

impl Default for AntChemotaxisConfig {
    fn default() -> Self {
        Self {
            chi_u: 50.0,
            d_w: 0.1,
            d_v: 0.1,
            epsilon: 0.5,
            lambda: 70.0,
            return_speed: 18.35,
            nest_radius: 1.8,
            phero_fade_radius: 6.0,
            emerge_time: 2.5,
            emerge_rate: 54.0,
        }
    }
}

/// 食物源定义
#[derive(Debug, Clone)]
pub struct FoodSource {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub initial_density: f64,
}

/// 蚂蚁趋化觅食动力学系统
#[derive(Debug, Clone)]
pub struct AntChemotaxisModel {
    pub config: AntChemotaxisConfig,
    /// 当前时间 $t$
    pub time: f64,
    /// 初始食物总质量（用于计算消耗百分比）
    pub initial_food_mass: f64,
    /// 觅食蚁场 $u(t, x, y)$
    pub u: Grid2D,
    /// 搬运蚁场 $w(t, x, y)$
    pub w: Grid2D,
    /// 信息素场 $v(t, x, y)$
    pub v: Grid2D,
    /// 食物源场 $c(t, x, y)$
    pub c: Grid2D,
    /// 巢穴入口空间权重场 $N(x, y)$
    pub n_field: Grid2D,
    /// 信息素抑制场 $P(x, y) \in [0, 1]$
    pub p_field: Grid2D,
    /// 回巢速度场分量 $V_{a, x} = \partial_x a$
    pub va_x: Vec<f64>,
    /// 回巢速度场分量 $V_{a, y} = \partial_y a$
    pub va_y: Vec<f64>,
    // 预分配复用临时缓冲区，杜绝步进中的内存申请
    buf_lap: Vec<f64>,
    buf_grad_x: Vec<f64>,
    buf_grad_y: Vec<f64>,
    buf_div: Vec<f64>,
    buf_vx: Vec<f64>,
    buf_vy: Vec<f64>,
}

impl AntChemotaxisModel {
    /// 初始化仿真系统
    ///
    /// - `nx`, `ny`: 网格尺寸（例如 100x100 或 160x160）
    /// - `l_half`: 区域半宽（区域为 $[-L, L] \times [-L, L]$，论文中约为 18.0）
    /// - `config`: 模型物理参数
    /// - `food_sources`: 食物源列表
    pub fn new(
        nx: usize,
        ny: usize,
        l_half: f64,
        config: AntChemotaxisConfig,
        food_sources: &[FoodSource],
    ) -> Self {
        let x_min = -l_half;
        let x_max = l_half;
        let y_min = -l_half;
        let y_max = l_half;

        let u = Grid2D::new(nx, ny, x_min, x_max, y_min, y_max, 0.0);
        let w = Grid2D::new(nx, ny, x_min, x_max, y_min, y_max, 0.0);
        let v = Grid2D::new(nx, ny, x_min, x_max, y_min, y_max, 0.0);
        let mut c = Grid2D::new(nx, ny, x_min, x_max, y_min, y_max, 0.0);
        let mut n_field = Grid2D::new(nx, ny, x_min, x_max, y_min, y_max, 0.0);
        let mut p_field = Grid2D::new(nx, ny, x_min, x_max, y_min, y_max, 1.0);

        let n_cells = nx * ny;
        let mut va_x = vec![0.0; n_cells];
        let mut va_y = vec![0.0; n_cells];

        // 1. 初始化食物场 $c_0(x, y)$
        for j in 0..ny {
            for i in 0..nx {
                let (x, y) = c.coord(i, j);
                let mut food_val = 0.0;
                for src in food_sources {
                    let d2 = (x - src.x).powi(2) + (y - src.y).powi(2);
                    let val = src.initial_density * (-d2 / (2.0 * src.radius.powi(2))).exp();
                    if val > 1e-4 {
                        food_val += val;
                    }
                }
                c.set(i, j, food_val);
            }
        }
        let initial_food_mass = c.integrate();

        // 2. 初始化巢穴场 $N(x, y)$、信息素抑制场 $P(x, y)$ 与回巢势场速度 $\nabla a$
        let nest_r = config.nest_radius;
        let fade_r = config.phero_fade_radius;
        let s_ret = config.return_speed;
        let smoothing_delta = 0.5 * nest_r;

        for j in 0..ny {
            for i in 0..nx {
                let (x, y) = u.coord(i, j);
                let r2 = x * x + y * y;
                let r = r2.sqrt();
                let idx = j * nx + i;

                // 巢穴指示函数：平滑高斯型分布，归一化使其在巢穴内有效
                let n_val = (-r2 / (2.0 * nest_r.powi(2))).exp();
                n_field.set(i, j, n_val);

                // 信息素抑制场：近巢穴处降为 0
                let p_val = if r < fade_r {
                    (r / fade_r).powi(2)
                } else {
                    1.0
                };
                p_field.set(i, j, p_val);

                // 回巢速度场：指向原点 (0, 0)
                // a(x) 引导速度场 \nabla a = - s_ret * (x, y) / sqrt(r^2 + delta^2)
                let denom = (r2 + smoothing_delta.powi(2)).sqrt();
                va_x[idx] = -s_ret * x / denom;
                va_y[idx] = -s_ret * y / denom;
            }
        }

        Self {
            config,
            time: 0.0,
            initial_food_mass,
            u,
            w,
            v,
            c,
            n_field,
            p_field,
            va_x,
            va_y,
            buf_lap: vec![0.0; n_cells],
            buf_grad_x: vec![0.0; n_cells],
            buf_grad_y: vec![0.0; n_cells],
            buf_div: vec![0.0; n_cells],
            buf_vx: vec![0.0; n_cells],
            buf_vy: vec![0.0; n_cells],
        }
    }

    /// ## 任务 3: 执行一次显式守恒有限差分迎风积分步
    ///
    /// ## 动力学控制方程 (arXiv:1409.3808):
    /// 1. **食物消耗场 $c$**: $\partial_t c = - u c$
    ///    采用半隐式稳定格式：$c^{n+1} = \max(0, c^n / (1 + dt \cdot u^n))$；
    /// 2. **信息素场梯度与趋化平流速度**: $\mathbf{V}_{\text{chem}} = \chi_u \nabla v$
    ///    - 调用 `self.v.compute_gradient`；
    ///    - 趋化散度项：$\nabla \cdot (u \mathbf{V}_{\text{chem}})$ 调用 `compute_upwind_divergence`；
    /// 3. **觅食蚁场 $u$**: $\partial_t u = \Delta u - \nabla \cdot (u \mathbf{V}_{\text{chem}}) - u c + \lambda w N(x) + M(t) N(x)$；
    /// 4. **搬运蚁场 $w$**: $\partial_t w = D_w \Delta w - \nabla \cdot (w \nabla a) + u c - \lambda w N(x)$；
    /// 5. **信息素场 $v$**: $\partial_t v = D_v \Delta v + P(x) w - \varepsilon v$；
    /// 6. **同步更新状态数组与时间**: `self.time += dt`。
    pub fn step(&mut self, dt: f64) {
        // TODO: 请实现四场非线性 PDE 的显式守恒迎风积分步
        todo!("【关卡 9 - 任务 3】请实现四组分偏微分动力学显式时间积分 step");
    }

    /// 计算当前时刻系统的各项物理与觅食统计指标
    pub fn metrics(&self) -> AntForagingMetrics {
        AntForagingMetrics::compute(
            self.time,
            &self.u,
            &self.w,
            &self.v,
            &self.c,
            self.initial_food_mass,
        )
    }
}
