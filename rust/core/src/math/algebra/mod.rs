//! 代数结构：矩阵、向量；物理量系统应建在此层之上（Scalar/ScalarNorm + Vec），不越过 Scalar 用 Real。
//!
//! - **mat::Scalar**：四则运算标量，用于 Mat、Vec（分量类型 = 数乘的「标量」时，即 T 同时当域元素用）。
//! - **mat::ScalarNorm**：Scalar + sqrt/zero/one，用于 Vec::norm/normalize。
//! - **mat::LinearComponent&lt;R&gt;**（R: Real）：数学上矢量空间只需「矢量加法 + 域上数乘」；若分量是物理量（如 Length），
//!   只需 Add + Sub + ScaledBy&lt;R&gt;，标量用 Real 抽象不固定 f64。Scalar 比这更严，用于纯数向量。
//! - **mat::Mat&lt;T,R,C&gt;**：R×C 矩阵。
//! - **vec::Vec&lt;T,N&gt;**：N 维向量，T: Copy；T: Scalar 时 dot/cross/norm。物理层 quantity::vector3::Vector3&lt;T&gt; = Vec&lt;T, 3&gt;（T: LinearComponent&lt;Real&gt;），在本层仅复用容器，不依赖 Real/物理量。

pub mod mat;
pub mod vec;
