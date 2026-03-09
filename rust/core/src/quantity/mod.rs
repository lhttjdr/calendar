//! 量纲与物理量类型：量纲、带量纲标量、长度、时间、平面角、位置、速度、位移等。
//! 概念按子模块分层，不在此层 re-export。
//!
//! **层级约束**：凡物理概念一律用本层类型（PlaneAngle、Length、Position、Velocity、Duration、Epoch 等），
//! 不越级使用下层的 Vec/Scalar，更不直接使用 f64/数组表示物理量（含与代数层的“边界”也不暴露 f64/数组）；若实现中必须与代数运算交互，转换封装在实现内部最小范围，对外 API 与数据类型仍只用物理量类型。
//!
//! **为何 Position/Velocity/Displacement 不用 math::Vec**：
//! 1. **量纲**：math::Vec 要求分量类型满足 `Scalar`（含 `Mul<Output = Self>` / `Div<Output = Self>`）。Length×Length→面积、Length/Length→无量纲，故 Length 不是 Scalar，`Vec<Length, 3>` 在类型上不成立。
//! 2. **参考架**：位置/速度/位移都带 `ReferenceFrame`，是「某架下的三分量」，不是无架纯向量；类型上应为「架 + 三分量」，而非裸 Vec。
//! 3. **语义区分**：Position / Velocity / Displacement 是不同物理概念，用独立类型可避免误混（如位置+位置无定义）；若用 Vec 仍需 newtype 包装。
//! 4. **与矩阵交互**：3×3 旋转等通过 `Mat::mul_vec_typed` 接受 `[Length; 3]` / `[Speed; 3]`，由 `Position::from_lengths_in_frame` 等组装回类型，不依赖 Vec。
//!
//! **物理矢量与 LinearComponent / ScaledBy**：数学上矢量空间只需「矢量加法 + 域上数乘」。标量用 **Real** 抽象，不直接写死 f64。`ScaledBy&lt;R&gt;`（R: Real）表示「可被 R 数乘」；`LinearComponent&lt;R&gt;` = Add + Sub + ScaledBy&lt;R&gt;，由 blanket 自动实现。物理层为分量类型实现 Add、Sub、ScaledBy&lt;R&gt;（R: Real）一次即可。三分量矢量用 [Vector3](vector3::Vector3) = math::Vec&lt;T, 3&gt;（T: LinearComponent&lt;Real&gt;）在物理层实现加减与数乘；`magnitude()` 等具名量在 `Vector3&lt;Length&gt;` 等上于本层 impl。
//!
//! **矢量 vs 三分量元组**：矢量 = 某架**正交归一化基**下的三分量，同量纲，可加可数乘（[Vector3](vector3::Vector3) = math::Vec&lt;T, 3&gt;，物理层补 Add/Sub/ScaledBy）。  
//! 三分量元组 = 某架**坐标基**下的分量，量纲可不同，须与带度规的架配合（[coord_components](coord_components)、[frame_metric](frame_metric)）。
//!
//! **vec / Vector3 / Displacement 关系**：
//! - **math::algebra::vec::Vec&lt;T, N&gt;**：代数层 N 维向量，T: Copy；T: Scalar 时 dot/cross/norm。数学层不依赖 Real/物理量。
//! - **quantity::vector3::Vector3&lt;T&gt;**：类型别名 = Vec&lt;T, 3&gt;；物理层为 T: LinearComponent&lt;Real&gt; 实现 Add/Sub/ScaledBy&lt;Real&gt;，复用 math::Vec。
//! - **quantity::coord_components**：**三分量元组**（坐标基分量），量纲可不同，与 [frame_metric](frame_metric) 配合。
//! - **Displacement**：`{ frame: ReferenceFrame, vec: Vector3&lt;Length&gt; }`，即架 + 位移矢量。
//!
//! **单位管理**：单位以参数形式提供，见 [unit] 模块（如 [SpeedUnit](unit::SpeedUnit)、[LengthUnit](unit::LengthUnit)）。构造用 `X::from_value(value, unit)`，取值用 `x.in_unit(unit)`；新增单位时在枚举中加变体并实现换算，不再增加 `from_km_per_xxx` 等硬编码函数。

pub mod angle;
pub mod angle_parse;
pub mod angular_rate;
pub mod coord_components;
pub mod dimension;
pub mod dimensionless;
pub mod displacement;
pub mod frame_metric;
pub mod vector3;
pub mod duration;
pub mod epoch;
pub mod julian_centuries;
pub mod length;
pub mod position;
pub mod pressure;
pub mod pressure_temperature_factor;
pub mod reference_frame;
pub mod rotation_matrix;
pub mod spherical;
pub mod speed;
pub mod thermodynamic_temperature;
pub mod unit;
pub mod velocity;

#[cfg(test)]
mod tests;
