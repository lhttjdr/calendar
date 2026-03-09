//! 数学工具：角度、级数、代数（矩阵/向量）、四元数/对偶四元数、数值抽象。
//!
//! 标量：**Real**（类型，基于 twofloat 双字浮点）+ **RealOps**（trait）；Real 为适配层，全库仅通过 Real/RealOps 操作。
//! - **real**：`Real` 类型、`RealOps` trait、`real_const`（const 构造）。
//! - **algebra::mat**：Scalar、ScalarNorm、Mat。
//! - **algebra::vec**：Vec、Vec3。
//! - **Quaternion / DualQuaternion**：标量用 Real。

pub mod algebra;
pub mod angle;
pub mod descartes;
pub mod dual_quaternion;
pub mod quaternion;
pub mod real;
pub mod series;

