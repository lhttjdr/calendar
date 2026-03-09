//! 天文模块统一入口：时间、岁差、参考架、历表、管线与视位置等。
//!
//! **物理量约定**：涉及物理概念时一律用 quantity 层类型，不暴露裸 `Real`/`f64` 表示物理量。应使用的类型包括：
//! - **长度** [`Length`](crate::quantity::length::Length)、**位移** [`Displacement`](crate::quantity::displacement::Displacement)、**位置** [`Position`](crate::quantity::position::Position)
//! - **速度** [`Speed`](crate::quantity::speed::Speed)、**速度矢量** [`Velocity`](crate::quantity::velocity::Velocity)
//! - **平面角** [`PlaneAngle`](crate::quantity::angle::PlaneAngle)、**角速度** [`AngularRate`](crate::quantity::angular_rate::AngularRate)
//! - **时长** [`Duration`](crate::quantity::duration::Duration)、**历元** [`Epoch`](crate::quantity::epoch::Epoch)
//! - **光速等常数** 用 [`constant::light_speed`](constant::light_speed) 等物理量或由其导出的数值。
//! 仅在矩阵/向量运算边界处临时用 `.meters()`、`.m_per_s()`、`.rad()` 等取出数值，对外 API 与数据类型仍只用物理量类型。
//!
//! ## 管线（pipeline）中的角色
//!
//! 视位置计算是一条管线，各层对应关系：
//!
//! - **时间** [`time`]：历元、时标（TT/TDB/UTC），管线输入。
//! - **星历表提供者** [`EphemerisProvider`](pipeline::EphemerisProvider)：给时刻 → 6D 状态；由 [`ephemeris`]（Vsop87、Elpmpp02）实现。
//! - **架映射** [`FrameMapper`](pipeline::FrameMapper)：跨架拟合/改正；用 [`frame`]（fk5_icrs、vsop87_de406_icrs_patch）。
//! - **岁差 / 章动** [`precession`]、[`nutation`]：架随时间的旋转（平架→真架），在 [`TransformGraph`](pipeline::TransformGraph) 中组合。
//! - **光行时** [`LightTimeCorrector`](pipeline::LightTimeCorrector)：用 [`light_time`]。
//! - **光学改正** [`OpticalCorrector`](pipeline::OpticalCorrector)：光行差等，用 [`aberration`]。
//! - **视位置 API** [`apparent`]：太阳/月球视黄经等，内部用管线。
//!
//! ## 上位概念（逻辑分组）
//!
//! - **参考架与架变换**：固定旋转 [`frame`] + **岁差** [`precession`] + **章动** [`nutation`]（架随时间的变化）。
//! - **历表** [`ephemeris`]：EphemerisProvider 的数据源。
//! - **观测改正**：光行时 [`light_time`]、光行差 [`aberration`]、大气折射 [`atmospheric_refraction`]。
//! - **日月关系** [`aspects`]：黄经、节气、月相。

pub mod apparent;
pub mod aspects;
pub mod constant;
pub mod correction;
pub mod ephemeris;
pub mod frame;
pub mod pipeline;
pub mod time;



