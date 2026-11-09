//! IAU 天体自转模型（WGCCRE，IAU 天体制图坐标与自转要素工作组）：自转轴北极 (α₀, δ₀)（ICRF）与本初子午线角 W，
//! 由此把 ICRS 方向换到行星本体坐标，求直下点（观测者、太阳）的经纬度。
//!
//! 版本：Archinal B.A. et al. 2018, *Report of the IAU Working Group on Cartographic Coordinates and Rotational Elements:
//! 2015*, *Celest. Mech. Dyn. Astr.* 130:22（下称 WGCCRE 2015），表 2（行星）。其火星式取自 Kuchynka et al. 2014
//! （*Icarus* 229, 340），较 WGCCRE 2009（Archinal et al. 2011, *CMDA* 109, 101：α₀ = 317.68143 − 0.1061T，
//! δ₀ = 52.88650 − 0.0609T，W = 176.630 + 350.89198226d）多出岁差的长周期项与短周期章动项；两式在 J2000 相合到 0.002°
//! （长周期项在 J2000 之值：α₀ 317.269202 + 0.41190 = 317.68110，δ₀ 54.432516 − 1.54622 = 52.88630，W 176.049863 + 0.58205
//! = 176.63191），见测试。
//!
//! 系数核对：与 NAIF 通用 PCK `pck00011.tpc`（据 WGCCRE 2015 编；BODY499_POLE_RA/DEC/PM、BODY499_NUT_PREC_*、BODY4_NUT_PREC_ANGLES
//! 第 11—26 角）逐项相同（2026-11-09 下载核对）。
//!
//! 约定（WGCCRE）：d 为自 J2000.0 起的 TDB 日数，T 为儒略世纪数；本体坐标 = R_z(W) · R_x(90° − δ₀) · R_z(90° + α₀) · ICRS。
//! 火星自转为顺行，行星面图（planetographic）经度向西为正：λ_W = 360° − λ_E（λ_E 为本体坐标中的面心东经）。
//! 自转角 W 应取光到达观测者之前离开行星的时刻（光行时修正后的时刻）。

use crate::math::real::{real, Real, RealOps};
use crate::quantity::angle::PlaneAngle;

/// 天体某时刻的自转取向。
#[derive(Clone, Copy, Debug)]
pub struct BodyOrientation {
    /// 自转轴北极赤经 α₀（ICRF）
    pub pole_ra: PlaneAngle,
    /// 自转轴北极赤纬 δ₀（ICRF）
    pub pole_dec: PlaneAngle,
    /// 本初子午线角 W（自 ICRF 赤道与本体赤道的升交点沿本体赤道东量至本初子午线）
    pub prime_meridian: PlaneAngle,
}

fn sin_deg(x: Real) -> Real {
    PlaneAngle::from_deg(x).rad().sin()
}
fn cos_deg(x: Real) -> Real {
    PlaneAngle::from_deg(x).rad().cos()
}

/// Σ a·sin(φ₀ + φ̇T)，(a, φ₀, φ̇)，度、度/儒略世纪。
fn sin_series(terms: &[(f64, f64, f64)], t: Real) -> Real {
    terms.iter().fold(real(0.0), |s, &(a, p0, p1)| s + real(a) * sin_deg(real(p0) + real(p1) * t))
}
fn cos_series(terms: &[(f64, f64, f64)], t: Real) -> Real {
    terms.iter().fold(real(0.0), |s, &(a, p0, p1)| s + real(a) * cos_deg(real(p0) + real(p1) * t))
}

/// 火星 α₀ 周期项（WGCCRE 2015 表 2；末项 0.5042615T 为岁差长周期项，周期约 7.1 万年）。
const MARS_RA_TERMS: [(f64, f64, f64); 5] = [
    (0.000068, 198.991226, 19139.4819985),
    (0.000238, 226.292679, 38280.8511281),
    (0.000052, 249.663391, 57420.7251593),
    (0.000009, 266.183510, 76560.6367950),
    (0.419057, 79.398797, 0.5042615),
];
/// 火星 δ₀ 周期项（余弦）。
const MARS_DEC_TERMS: [(f64, f64, f64); 5] = [
    (0.000051, 122.433576, 19139.9407476),
    (0.000141, 43.058401, 38280.8753272),
    (0.000031, 57.663379, 57420.7517205),
    (0.000005, 79.476401, 76560.6495004),
    (1.591274, 166.325722, 0.5042615),
];
/// 火星 W 周期项（正弦）。
const MARS_W_TERMS: [(f64, f64, f64); 6] = [
    (0.000145, 129.071773, 19140.0328244),
    (0.000157, 36.352167, 38281.0473591),
    (0.000040, 56.668646, 57420.9295360),
    (0.000001, 67.364003, 76560.2552215),
    (0.000001, 104.792680, 95700.4387578),
    (0.584542, 95.391654, 0.5042615),
];

/// 火星本初子午线角速率 Ẇ（WGCCRE 2015，度/日）。
pub const MARS_ROTATION_DEG_PER_DAY: f64 = 350.891982443297;

/// 火星自转取向（WGCCRE 2015）。`jd_tdb`：TDB 儒略日（应为光行时修正后的发光时刻）。
pub fn mars_orientation_wgccre2015(jd_tdb: Real) -> BodyOrientation {
    let d = jd_tdb - real(2451545.0);
    let t = d / real(36525.0);
    let ra = real(317.269202) - real(0.10927547) * t + sin_series(&MARS_RA_TERMS, t);
    let dec = real(54.432516) - real(0.05827105) * t + cos_series(&MARS_DEC_TERMS, t);
    // W 的线性部分先对 360° 取余，免得 d 大时与周期项相加损失有效位
    let w_lin = (real(MARS_ROTATION_DEG_PER_DAY) * d).as_f64().rem_euclid(360.0);
    let w = real(176.049863) + real(w_lin) + sin_series(&MARS_W_TERMS, t);
    BodyOrientation { pole_ra: PlaneAngle::from_deg(ra), pole_dec: PlaneAngle::from_deg(dec), prime_meridian: PlaneAngle::from_deg(w) }
}

/// R_z(θ) 坐标旋转作用于 v。
fn rot_z(theta: Real, v: [Real; 3]) -> [Real; 3] {
    let (s, c) = (theta.sin(), theta.cos());
    [c * v[0] + s * v[1], -s * v[0] + c * v[1], v[2]]
}
/// R_x(θ) 坐标旋转作用于 v。
fn rot_x(theta: Real, v: [Real; 3]) -> [Real; 3] {
    let (s, c) = (theta.sin(), theta.cos());
    [v[0], c * v[1] + s * v[2], -s * v[1] + c * v[2]]
}

impl BodyOrientation {
    /// ICRS 向量 → 本体坐标（x 指本初子午线，z 指北极）。
    pub fn icrs_to_body_fixed(&self, v: [Real; 3]) -> [Real; 3] {
        let half_pi = Real::pi() / real(2.0);
        let v = rot_z(half_pi + self.pole_ra.rad(), v);
        let v = rot_x(half_pi - self.pole_dec.rad(), v);
        rot_z(self.prime_meridian.rad(), v)
    }

    /// 行星中心指向 `dir_icrs` 方向的直下点：(面心东经, 面心纬度)。东经取 [0, 2π)。
    pub fn sub_point(&self, dir_icrs: [Real; 3]) -> (PlaneAngle, PlaneAngle) {
        let b = self.icrs_to_body_fixed(dir_icrs);
        let h = (b[0] * b[0] + b[1] * b[1]).sqrt();
        (PlaneAngle::from_rad(b[1].atan2(b[0])).wrap_to_2pi(), PlaneAngle::from_rad(b[2].atan2(h)))
    }

    /// 顺行天体的行星面图西经（度，[0, 360)）：λ_W = 360° − λ_E。
    pub fn sub_point_west_longitude_deg(&self, dir_icrs: [Real; 3]) -> Real {
        let east = self.sub_point(dir_icrs).0;
        let w = PlaneAngle::from_rad(-east.rad()).wrap_to_2pi();
        w.rad() * real(180.0) / Real::pi()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deg(a: PlaneAngle) -> f64 {
        a.rad().as_f64().to_degrees()
    }

    /// WGCCRE 2015 与 2009 的火星式在 J2000 相合（长周期项吸收了 2009 式的常数），以防抄错系数；
    /// 百年后两式之差仍在 0.05° 内（2009 式只是线性近似）。
    #[test]
    fn mars_wgccre2015_agrees_with_2009_near_j2000() {
        for (jd, tol) in [(2451545.0, 0.003), (2451545.0 + 36525.0, 0.05), (2451545.0 - 36525.0, 0.05)] {
            let o = mars_orientation_wgccre2015(real(jd));
            let d = jd - 2451545.0;
            let t = d / 36525.0;
            let ra09 = 317.68143 - 0.1061 * t;
            let dec09 = 52.88650 - 0.0609 * t;
            let w09 = (176.630 + 350.89198226 * d).rem_euclid(360.0);
            let dw = (deg(o.prime_meridian) - w09 + 540.0).rem_euclid(360.0) - 180.0;
            assert!((deg(o.pole_ra) - ra09).abs() < tol, "{jd} α₀ {} vs {ra09}", deg(o.pole_ra));
            assert!((deg(o.pole_dec) - dec09).abs() < tol, "{jd} δ₀ {} vs {dec09}", deg(o.pole_dec));
            assert!(dw.abs() < tol.max(0.01), "{jd} W 差 {dw}");
        }
    }

    /// 北极方向的直下点纬度为 +90°；本初子午线方向东经 0。
    #[test]
    fn body_fixed_axes() {
        let o = mars_orientation_wgccre2015(real(2460000.5));
        let (ra, dec) = (o.pole_ra.rad(), o.pole_dec.rad());
        let pole = [dec.cos() * ra.cos(), dec.cos() * ra.sin(), dec.sin()];
        let b = o.icrs_to_body_fixed(pole);
        assert!((b[2] - real(1.0)).abs().as_f64() < 1e-14);
        assert!((deg(o.sub_point(pole).1) - 90.0).abs() < 1e-9);
    }
}
