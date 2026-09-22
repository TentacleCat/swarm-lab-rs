//! 图灵反应-扩散形态素动力学 (Turing Reaction-Diffusion Morphogen Dynamics)
//!
//! 依据 Science Robotics 2018 原作 C 语言实现：
//! - 包含激活子 u (Activator) 与抑制子 v (Inhibitor) 的非线性饱和动力学；
//! - 局域红外通信网络图上的离散图拉普拉斯算子 (Graph Laplacian)；
//! - 极化判定与 Kilobot 经典 LED RGB 多色映射。

use serde::{Deserialize, Serialize};

/// 图灵形态素动力学核心物理与算法参数
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MorphogenParams {
    /// 激活子自催化项系数 (原代码 A_VAL = 0.08)
    pub a: f64,
    /// 抑制子对激活子的负反馈系数 (原代码 B_VAL = -0.08)
    pub b: f64,
    /// 激活子基础生成常数 (原代码 C_VAL = 0.03)
    pub c: f64,
    /// 激活子自身降解率 (原代码 D_VAL = 0.03)
    pub d: f64,
    /// 激活子诱导抑制子合成系数 (原代码 E_VAL = 0.10)
    pub e: f64,
    /// 抑制子基础阈值偏移 (原代码 F_VAL = 0.12)
    pub f: f64,
    /// 抑制子自身降解率 (原代码 G_VAL = 0.06)
    pub g: f64,
    /// 激活子扩散系数 (原代码 D_u = 0.5)
    pub d_u: f64,
    /// 抑制子扩散系数 (原代码 D_v = 10.0, 远大于 D_u 保证图灵失稳)
    pub d_v: f64,
    /// 反应速率全局时间缩放系数 (原代码 LINEAR_R = 160.0)
    pub r_scale: f64,
    /// 激活子合成速率饱和上限 (原代码 SYNTH_U_MAX = 0.23)
    pub synth_u_max: f64,
    /// 抑制子合成速率饱和上限 (原代码 SYNTH_V_MAX = 0.50)
    pub synth_v_max: f64,
    /// 反应-扩散通信与扩散半径 (原代码 DIFF_R = 85.0 mm)
    pub diff_r: f64,
    /// 通信半径 (原代码 COMM_R = 85.0 mm)
    pub comm_r: f64,
    /// 极化斑点阈值 (原代码 POLAR_TH = 4.0)
    pub polar_th: f64,
}

impl Default for MorphogenParams {
    fn default() -> Self {
        Self {
            a: 0.08,
            b: -0.08,
            c: 0.03,
            d: 0.03,
            e: 0.10,
            f: 0.12,
            g: 0.06,
            d_u: 0.5,
            d_v: 10.0,
            r_scale: 160.0,
            synth_u_max: 0.23,
            synth_v_max: 0.50,
            diff_r: 85.0,
            comm_r: 85.0,
            polar_th: 4.0,
        }
    }
}

/// Kilobot 显示状态对应的 LED 颜色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LedColor {
    /// 熄灭 (u <= 1.0)
    Black,
    /// 粉色 (1.0 < u <= 2.0)
    Pink,
    /// 蓝色 (2.0 < u <= 3.0)
    Blue,
    /// 青色 (3.0 < u <= 4.0)
    Cyan,
    /// 绿色 (极化态 u > 4.0, 图灵激活斑点中心)
    Green,
    /// 白色 (ORBIT 边缘沿轮廓巡航移动)
    White,
    /// 红色 (FOLLOW 掉队靠拢模式)
    Red,
}

/// 机器人体内虚拟形态素浓度 (u: 激活子, v: 抑制子)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MorphogenConcentration {
    /// 激活子浓度 (Activator)
    pub u: f64,
    /// 抑制子浓度 (Inhibitor)
    pub v: f64,
}

impl MorphogenConcentration {
    /// 创建新形态素浓度
    pub fn new(u: f64, v: f64) -> Self {
        Self { u, v }
    }

    /// 判定是否达到极化状态 (即是否处于图灵斑点核心)
    pub fn is_polarized(&self, threshold: f64) -> bool {
        self.u > threshold
    }

    /// 计算分段线性饱和反应动力学生成速率 (synth_u, synth_v)
    pub fn reaction_rates(&self, p: &MorphogenParams) -> (f64, f64) {
        let mut rate_u = p.a * self.u + p.b * self.v + p.c;
        if rate_u < 0.0 {
            rate_u = 0.0;
        } else if rate_u > p.synth_u_max {
            rate_u = p.synth_u_max;
        }
        rate_u -= p.d * self.u;

        let mut rate_v = p.e * self.u - p.f;
        if rate_v < 0.0 {
            rate_v = 0.0;
        } else if rate_v > p.synth_v_max {
            rate_v = p.synth_v_max;
        }
        rate_v -= p.g * self.v;

        (rate_u, rate_v)
    }

    /// 单步欧拉显式数值积分更新形态素浓度
    ///
    /// du = r_scale * synth_u + D_u * lap_u
    /// dv = r_scale * synth_v + D_v * lap_v
    pub fn step(&mut self, lap_u: f64, lap_v: f64, dt: f64, p: &MorphogenParams) {
        let (rate_u, rate_v) = self.reaction_rates(p);
        let du = p.r_scale * rate_u + p.d_u * lap_u;
        let dv = p.r_scale * rate_v + p.d_v * lap_v;

        self.u += dt * du;
        self.v += dt * dv;

        // 保证浓度非负
        if self.u < 0.0 {
            self.u = 0.0;
        }
        if self.v < 0.0 {
            self.v = 0.0;
        }
    }

    /// 根据当前浓度获得 LED 颜色 (静态未移动状态下)
    pub fn gradient_color(&self, polar_th: f64) -> LedColor {
        if self.u > polar_th {
            LedColor::Green
        } else if self.u > polar_th - 1.0 {
            LedColor::Cyan
        } else if self.u > polar_th - 2.0 {
            LedColor::Blue
        } else if self.u > polar_th - 3.0 {
            LedColor::Pink
        } else {
            LedColor::Black
        }
    }
}
