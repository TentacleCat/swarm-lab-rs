//! # Ant Chemotaxis Foraging PDE Model
//!
//! 基于 arXiv:1409.3808 (*Journal of Theoretical Biology*, 2015) 的
//! 四组分非线性趋化偏微分方程组：
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

    /// 执行一次显式守恒有限差分迎风积分步
    ///
    /// 满足稳定 CFL 条件（推荐 $dt \approx 0.001 \sim 0.002$）
    pub fn step(&mut self, dt: f64) {
        let n_cells = self.u.nx * self.u.ny;

        // 计算当前涌出源 $M(t)$
        let m_t = if self.time <= self.config.emerge_time {
            self.config.emerge_rate
        } else {
            0.0
        };

        // --- 1. 更新食物场 c: \partial_t c = - u * c ---
        // c^{n+1} = c^n / (1 + dt * u^n) 采用半隐式解析解，保证 c 严格非负单调递减
        for idx in 0..n_cells {
            let u_val = self.u.data[idx];
            let c_val = self.c.data[idx];
            self.c.data[idx] = (c_val / (1.0 + dt * u_val)).max(0.0);
        }

        // --- 2. 计算信息素场梯度并合成趋化对流速度场 V_chem = chi_u * \nabla v ---
        self.v.compute_gradient(&mut self.buf_grad_x, &mut self.buf_grad_y);
        let chi_u = self.config.chi_u;
        for idx in 0..n_cells {
            self.buf_vx[idx] = chi_u * self.buf_grad_x[idx];
            self.buf_vy[idx] = chi_u * self.buf_grad_y[idx];
        }

        // 计算趋化散度 \nabla \cdot (u * V_chem)
        self.u.compute_upwind_divergence(&self.buf_vx, &self.buf_vy, &mut self.buf_div);

        // 计算拉普拉斯 \Delta u
        self.u.compute_laplacian(&mut self.buf_lap);

        // --- 3. 准备更新 u: \partial_t u = \Delta u - \nabla\cdot(u V_chem) - u*c + \lambda*w*N + M*N ---
        // 先暂存在 buf_grad_x 中
        let lambda = self.config.lambda;
        for idx in 0..n_cells {
            let u_val = self.u.data[idx];
            let w_val = self.w.data[idx];
            let c_val = self.c.data[idx];
            let n_val = self.n_field.data[idx];
            let lap_u = self.buf_lap[idx];
            let div_chem = self.buf_div[idx];

            let du_dt = lap_u - div_chem - u_val * c_val + lambda * w_val * n_val + m_t * n_val;
            self.buf_grad_x[idx] = (u_val + dt * du_dt).max(0.0);
        }

        // --- 4. 准备更新 w: \partial_t w = D_w \Delta w - \nabla\cdot(w \nabla a) + u*c - \lambda*w*N ---
        // 计算回巢对流散度 \nabla \cdot (w \nabla a)
        self.w.compute_upwind_divergence(&self.va_x, &self.va_y, &mut self.buf_div);
        self.w.compute_laplacian(&mut self.buf_lap);

        let d_w = self.config.d_w;
        for idx in 0..n_cells {
            let u_val = self.u.data[idx];
            let w_val = self.w.data[idx];
            let c_val = self.c.data[idx];
            let n_val = self.n_field.data[idx];
            let lap_w = self.buf_lap[idx];
            let div_ret = self.buf_div[idx];

            let dw_dt = d_w * lap_w - div_ret + u_val * c_val - lambda * w_val * n_val;
            self.buf_grad_y[idx] = (w_val + dt * dw_dt).max(0.0);
        }

        // --- 5. 准备更新 v: \partial_t v = D_v \Delta v + P(x) * w - \varepsilon * v ---
        self.v.compute_laplacian(&mut self.buf_lap);
        let d_v = self.config.d_v;
        let eps = self.config.epsilon;

        for idx in 0..n_cells {
            let v_val = self.v.data[idx];
            let w_val = self.w.data[idx];
            let p_val = self.p_field.data[idx];
            let lap_v = self.buf_lap[idx];

            let dv_dt = d_v * lap_v + p_val * w_val - eps * v_val;
            self.v.data[idx] = (v_val + dt * dv_dt).max(0.0);
        }

        // --- 6. 将新状态同步写回 u 和 w ---
        self.u.data.copy_from_slice(&self.buf_grad_x);
        self.w.data.copy_from_slice(&self.buf_grad_y);

        self.time += dt;
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
