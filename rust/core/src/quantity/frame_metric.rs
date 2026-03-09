//! 度规与 scale factors：用于坐标基分量 ↔ 正交归一化基（矢量）的换算。
//!
//! 参考架可携带度规；三分量元组（坐标基下的分量）与矢量（正交归一化基下的分量）的转换依赖 scale factors。
//! 直角坐标：scale factors = (1,1,1)。球坐标（r, lon, lat）：h_r=1, h_lon=r·cos(lat), h_lat=r。

use crate::math::real::{one, Real, RealOps};

/// 正交坐标系下度规的 scale factors：物理分量 = h_i × 坐标分量（逆变）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaleFactors3(pub [Real; 3]);

/// 坐标种类：决定 scale factors 的形式。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoordKind {
    /// 直角坐标，正交归一化基与坐标基一致，度规为单位阵。
    Cartesian,
    /// 球坐标 (r, lon, lat)，弧度。
    Spherical,
}

impl CoordKind {
    /// 速度分量的 scale factors；`position` 为 (r, lon, lat) 的数值（SI：m, rad, rad）。
    /// Cartesian 恒为 (1,1,1)；Spherical 为 (1, r·cos(lat), r)。
    pub fn velocity_scale_factors(self, position: (Real, Real, Real)) -> ScaleFactors3 {
        match self {
            CoordKind::Cartesian => ScaleFactors3([one(), one(), one()]),
            CoordKind::Spherical => {
                let (r, _lon, lat) = position;
                let r = RealOps::abs(r); // 避免负 r 导致负 h
                let h_lon = r * lat.cos();
                let h_lat = r;
                ScaleFactors3([one(), h_lon, h_lat])
            }
        }
    }
}
