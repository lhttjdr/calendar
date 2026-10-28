//! 观测者：大地坐标 → 地心直角坐标（与 SOFA `iauGd2gc` 同式），并经地球自转转到当日真赤道架。
//!
//! 默认椭球 GRS80（IERS Conventions 2010 推荐）；WGS84 与之扁率差约 1e-10，供对照。
//! 极移对古代天象可忽略，地固系 → 当日真赤道只绕 z 轴转 GAST（[`crate::astronomy::time::earth_rotation::gast`]）。

use crate::math::real::{real_const, Real};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::epoch::Epoch;
use crate::quantity::length::Length;
use crate::quantity::position::Position;
use crate::quantity::reference_frame::ReferenceFrame;
use crate::quantity::unit::LengthUnit;

/// 参考椭球。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Ellipsoid {
    /// GRS80：a = 6378137 m，1/f = 298.257222101。
    #[default]
    Grs80,
    /// WGS84：a = 6378137 m，1/f = 298.257223563。
    Wgs84,
}

impl Ellipsoid {
    /// (赤道半径 a [m], 扁率 f)。
    pub fn parameters(self) -> (Real, Real) {
        match self {
            Ellipsoid::Grs80 => (real_const(6378137.0), real_const(1.0) / real_const(298.257222101)),
            Ellipsoid::Wgs84 => (real_const(6378137.0), real_const(1.0) / real_const(298.257223563)),
        }
    }
}

/// 地面观测者。经度东为正；纬度为大地纬度（椭球法线与赤道面夹角）；高度为椭球高。
#[derive(Clone, Copy, Debug)]
pub struct Observer {
    pub longitude: PlaneAngle,
    pub latitude: PlaneAngle,
    pub height: Length,
    pub ellipsoid: Ellipsoid,
}

impl Observer {
    pub fn new(longitude: PlaneAngle, latitude: PlaneAngle, height: Length) -> Self {
        Self { longitude, latitude, height, ellipsoid: Ellipsoid::default() }
    }

    /// 地固系（ITRS）地心直角坐标 [m]。
    pub fn geocentric_itrs_meters(&self) -> [Real; 3] {
        let (a, f) = self.ellipsoid.parameters();
        let e2 = f.clone() * (real_const(2.0) - f);
        let (sp, cp) = (self.latitude.rad().sin(), self.latitude.rad().cos());
        let (sl, cl) = (self.longitude.rad().sin(), self.longitude.rad().cos());
        let n = a / (real_const(1.0) - e2.clone() * sp.clone() * sp.clone()).sqrt();
        let h = self.height.meters();
        let r_xy = (n.clone() + h.clone()) * cp;
        [r_xy.clone() * cl, r_xy * sl, (n * (real_const(1.0) - e2) + h) * sp]
    }

    /// 当日真赤道架下的地心位置：地固系绕 z 轴转 +GAST（观测者子午圈赤经 = GAST + 经度）。
    pub fn position_true_equator(&self, gast: PlaneAngle, epoch: Epoch) -> Position {
        let [x, y, z] = self.geocentric_itrs_meters();
        let (sg, cg) = (gast.rad().sin(), gast.rad().cos());
        Position::from_si_meters_in_frame(
            ReferenceFrame::TrueEquator(epoch),
            x.clone() * cg.clone() - y.clone() * sg.clone(),
            x * sg + y * cg,
            z,
        )
    }

    /// 地心距 [m]（诊断用）。
    pub fn geocentric_distance(&self) -> Length {
        let [x, y, z] = self.geocentric_itrs_meters();
        Length::from_value((x.clone() * x + y.clone() * y + z.clone() * z).sqrt(), LengthUnit::Meter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::{real, RealOps};

    /// x、y 对照 SOFA t_sofa_c.c 中 iauGd2gc(n, 3.1, −0.5, 2500.0) 的值；z 用独立几何检验：沿椭球法线退回 h 后的点
    /// 必须落在椭球面上（p²/a² + z²/b² = 1），且法线方向的倾角等于大地纬度。
    #[test]
    fn geodetic_to_geocentric_matches_sofa_and_lies_on_ellipsoid() {
        for (ell, expect_xy) in [
            (Ellipsoid::Wgs84, [-5599000.5577049947, 233011.67223479203]),
            (Ellipsoid::Grs80, [-5599000.5577260738, 233011.6722356703]),
        ] {
            let (lon, lat, h) = (3.1_f64, -0.5_f64, 2500.0_f64);
            let o = Observer {
                longitude: PlaneAngle::from_rad(real(lon)),
                latitude: PlaneAngle::from_rad(real(lat)),
                height: Length::from_value(real(h), LengthUnit::Meter),
                ellipsoid: ell,
            };
            let v: Vec<f64> = o.geocentric_itrs_meters().iter().map(|x| x.as_f64()).collect();
            for i in 0..2 {
                assert!((v[i] - expect_xy[i]).abs() < 1e-6, "{ell:?} [{i}] {} vs {}", v[i], expect_xy[i]);
            }
            let (a, f) = ell.parameters();
            let (a, f) = (a.as_f64(), f.as_f64());
            let b = a * (1.0 - f);
            // 退回椭球面
            let n = [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()];
            let s = [v[0] - h * n[0], v[1] - h * n[1], v[2] - h * n[2]];
            let p = (s[0] * s[0] + s[1] * s[1]).sqrt();
            let on_surface = p * p / (a * a) + s[2] * s[2] / (b * b);
            assert!((on_surface - 1.0).abs() < 1e-12, "{ell:?} 不在椭球面上：{on_surface}");
            // 椭球面该点外法线倾角 = atan2(z/b², p/a²) 应等于大地纬度
            let normal_lat = (s[2] / (b * b)).atan2(p / (a * a));
            assert!((normal_lat - lat).abs() < 1e-12, "{ell:?} 法线倾角 {normal_lat} ≠ {lat}");
        }
    }
}
