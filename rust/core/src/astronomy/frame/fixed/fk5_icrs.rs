//! J2000.0 动力学赤道/春分（FK5）→ ICRS 的固定旋转矩阵。矩阵与向量统一 Real。
//! 数值来源：IAU SOFA 的 FK5→Hipparcos(ICRS) 偏置矩阵（J2000.0 适用）。

use crate::math::real::{real_const, real, Real, ToReal};

/// 3×3 旋转矩阵 R：v_ICRS = R · v_FK5（列向量，J2000 平赤道下）。近似单位阵 + 亚角秒级 off-diagonal。
#[inline]
pub fn rotation_matrix() -> [[Real; 3]; 3] {
    [
        [
            real_const(1.0),
            real_const(-8.82462042e-8),
            real_const(-3.85466334e-8),
        ],
        [
            real_const(8.82462042e-8),
            real_const(1.0),
            real_const(-3.30116133e-8),
        ],
        [
            real_const(3.85466334e-8),
            real_const(3.30116133e-8),
            real_const(1.0),
        ],
    ]
}

/// 将 J2000 平赤道下的直角矢量 (x,y,z) 从 FK5 转到 ICRS，返回 (x',y',z')。单位不变。
#[inline]
pub fn rotate_equatorial(x: impl ToReal, y: impl ToReal, z: impl ToReal) -> (Real, Real, Real) {
    let r = rotation_matrix();
    let (x, y, z) = (real(x), real(y), real(z));
    (
        r[0][0] * x + r[0][1] * y + r[0][2] * z,
        r[1][0] * x + r[1][1] * y + r[1][2] * z,
        r[2][0] * x + r[2][1] * y + r[2][2] * z,
    )
}

/// ICRS → J2000 平赤道(FK5)：即 R^T，岁差公式入口用。
#[inline]
pub fn rotate_equatorial_icrs_to_fk5(x: impl ToReal, y: impl ToReal, z: impl ToReal) -> (Real, Real, Real) {
    let r = rotation_matrix();
    let (x, y, z) = (real(x), real(y), real(z));
    (
        r[0][0] * x + r[1][0] * y + r[2][0] * z,
        r[0][1] * x + r[1][1] * y + r[2][1] * z,
        r[0][2] * x + r[1][2] * y + r[2][2] * z,
    )
}
