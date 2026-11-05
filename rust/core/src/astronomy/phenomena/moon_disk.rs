//! 某地某刻实际看到的月面：月亮的站心视高度与方位、天顶方向在月面上的位置角、月面天平动（地球在月面上的经纬度）与
//! 月轴位置角、大气折射把月面上下压扁的比例。供月食图「月面实况」按观测地画月面（贴图、朝向、地平）用。
//!
//! 天平动与月轴位置角按 Meeus《Astronomical Algorithms》第 2 版第 53 章：卡西尼定律给出月面平赤道（对黄道倾角
//! I = 1.54242°，升交点在月球轨道升交点 Ω 处），光学天平动由月亮实际视黄经黄纬求出，物理天平动 ρ、σ、τ 按该章级数。
//! 不用 IAU WGCCRE 的月球自转模型（α₀、δ₀、W 线性加周期项）：那是近代拟合，往前推两千年月亮平运动的长期加速会累积到约 1°；
//! 这里的光学天平动直接跟随历表中的月亮位置，古今同样适用。月面经纬度为平地球/极轴（ME）坐标，与 LRO 月面图一致。

use crate::astronomy::apparent::{right_ascension_declination, topocentric};
use crate::astronomy::correction::atmospheric_refraction::bennett_refraction_default;
use crate::astronomy::frame::nutation::{fundamental_arguments, nutation_for_apparent};
use crate::astronomy::frame::precession::mean_obliquity;
use crate::astronomy::geodesy::Observer;
use crate::astronomy::time::earth_rotation::gast;
use crate::math::real::{real, real_const, Real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::duration::Duration;
use crate::quantity::epoch::Epoch;
use crate::quantity::position::Position;
use crate::quantity::reference_frame::ReferenceFrame;

use super::solar_eclipse::SunMoonSource;

/// 某地某刻看到的月面几何。角度均为弧度量。
#[derive(Clone, Copy, Debug)]
pub struct MoonDiskView {
    pub jd_tt: Real,
    /// 月心站心几何高度（不含折射）
    pub altitude: PlaneAngle,
    /// 月心视高度（含折射，Bennett 公式、标准大气）；几何高度低于 −2° 时不加折射，取几何高度
    pub apparent_altitude: PlaneAngle,
    /// 方位角，自北向东
    pub azimuth: PlaneAngle,
    /// 天顶方向在月面上的位置角（自天北向东量，即星位角）
    pub zenith_position_angle: PlaneAngle,
    /// 站心天平动：地球（观测者）在月面上的经度、纬度（含物理天平动）
    pub libration_longitude: PlaneAngle,
    pub libration_latitude: PlaneAngle,
    /// 月球自转轴北极的位置角（自天北向东量）
    pub axis_position_angle: PlaneAngle,
    /// 大气折射使月面竖直方向缩成原来的比例（≤ 1；地平以下取 1）
    pub vertical_scale: Real,
}

fn deg(x: f64) -> Real {
    real(x.to_radians())
}

/// 月面平赤道对黄道的倾角 I（Meeus 53）。
fn lunar_inclination() -> Real {
    deg(1.54242)
}

/// 由站心（或地心）视黄经 λ、视黄纬 β、赤经 α 与时刻求天平动 (l, b) 与月轴位置角 P。Meeus 第 53 章。
fn libration_and_axis(t: Real, lambda: Real, beta: Real, alpha: Real, dpsi: Real, eps: Real) -> (Real, Real, Real) {
    let [mp, m, f, d, om] = fundamental_arguments(t).map(|a| a.rad());
    let i = lunar_inclination();
    let w = lambda - dpsi - om;
    let a = (w.sin() * beta.cos() * i.cos() - beta.sin() * i.sin()).atan2(w.cos() * beta.cos());
    let l_opt = a - f;
    let b_opt = (-(w.sin()) * beta.cos() * i.sin() - beta.sin() * i.cos()).asin();

    // 物理天平动（度）
    let tc = t;
    let k1 = deg(119.75) + deg(131.849) * tc;
    let k2 = deg(72.56) + deg(20.186) * tc;
    let e = real_const(1.0) - real(0.002516) * tc - real(0.0000074) * tc * tc;
    let two = real_const(2.0);
    let c = |x: f64| real(x);
    let rho = c(-0.02752) * mp.cos() - c(0.02245) * f.sin() + c(0.00684) * (mp - two * f).cos() - c(0.00293) * (two * f).cos()
        - c(0.00085) * (two * f - two * d).cos()
        - c(0.00054) * (mp - two * d).cos()
        - c(0.00020) * (mp + f).sin()
        - c(0.00020) * (mp + two * f).cos()
        - c(0.00020) * (mp - f).cos()
        + c(0.00014) * (mp + two * f - two * d).cos();
    let sigma = c(-0.02816) * mp.sin() + c(0.02244) * f.cos() - c(0.00682) * (mp - two * f).sin() - c(0.00279) * (two * f).sin()
        - c(0.00083) * (two * f - two * d).sin()
        + c(0.00069) * (mp - two * d).sin()
        + c(0.00040) * (mp + f).cos()
        - c(0.00025) * (two * mp).sin()
        - c(0.00023) * (mp + two * f).sin()
        + c(0.00020) * (mp - f).cos()
        + c(0.00019) * (mp - f).sin()
        + c(0.00013) * (mp + two * f - two * d).sin()
        - c(0.00010) * (mp - c(3.0) * f).cos();
    let tau = c(0.02520) * e * m.sin() + c(0.00473) * (two * mp - two * f).sin() - c(0.00467) * mp.sin()
        + c(0.00396) * k1.sin()
        + c(0.00276) * (two * mp - two * d).sin()
        + c(0.00196) * om.sin()
        - c(0.00183) * (mp - f).cos()
        + c(0.00115) * (mp - two * d).sin()
        - c(0.00096) * (mp - d).sin()
        + c(0.00046) * (two * f - two * d).sin()
        - c(0.00039) * (mp - f).sin()
        - c(0.00032) * (mp - m - d).sin()
        + c(0.00027) * (two * mp - m - two * d).sin()
        + c(0.00023) * k2.sin()
        - c(0.00014) * (two * d).sin()
        + c(0.00014) * (two * mp - two * f).cos()
        - c(0.00012) * (mp - two * f).sin()
        - c(0.00012) * (two * mp).sin()
        + c(0.00011) * (two * mp - two * m - two * d).sin();
    let to_rad = Real::pi() / real_const(180.0);
    let (rho, sigma, tau) = (rho * to_rad, sigma * to_rad, tau * to_rad);
    let l_phys = -tau + (rho * a.cos() + sigma * a.sin()) * b_opt.tan();
    let b_phys = sigma * a.cos() - rho * a.sin();
    let l = l_opt + l_phys;
    let b = b_opt + b_phys;

    // 月轴位置角
    let v = om + dpsi + sigma / i.sin();
    let x = (i + rho).sin() * v.sin();
    let y = (i + rho).sin() * v.cos() * eps.cos() - (i + rho).cos() * eps.sin();
    let omega = x.atan2(y);
    let p = ((x * x + y * y).sqrt() * (alpha - omega).cos() / b.cos()).asin();
    let two_pi = two * Real::pi();
    let wrap = |x: Real| {
        let y = x - two_pi * ((x + Real::pi()) / two_pi).floor();
        y
    };
    (wrap(l), b, p)
}

/// 某地某 TT 时刻看到的月面几何。`delta_t` 为 TT − UT1。
pub fn moon_disk_view_in(src: &dyn SunMoonSource, observer: &Observer, jd_tt: Real, delta_t: Duration) -> MoonDiskView {
    let jd_ut1 = jd_tt - delta_t.seconds() / real_const(86400.0);
    let g = gast(jd_ut1, jd_tt);
    let (_, m) = src.sun_moon_m(jd_tt);
    let geo = Position::from_si_meters_in_frame(ReferenceFrame::TrueEquator(Epoch::new(jd_tt)), m[0], m[1], m[2]);
    let topo = topocentric(geo, observer, g);
    let [x, y, z] = topo.to_meters();
    let r = (x * x + y * y + z * z).sqrt();
    let u = [x / r, y / r, z / r];

    // 天北、天东切向量；天顶（大地法向）在真赤道架中的方向
    let nz = [-u[2] * u[0], -u[2] * u[1], real_const(1.0) - u[2] * u[2]];
    let nn = (nz[0] * nz[0] + nz[1] * nz[1] + nz[2] * nz[2]).sqrt();
    let n = [nz[0] / nn, nz[1] / nn, nz[2] / nn];
    let e = [n[1] * u[2] - n[2] * u[1], n[2] * u[0] - n[0] * u[2], n[0] * u[1] - n[1] * u[0]];
    let theta = g.rad() + observer.longitude.rad();
    let (sp, cp) = (observer.latitude.rad().sin(), observer.latitude.rad().cos());
    let zen = [cp * theta.cos(), cp * theta.sin(), sp];
    let dot = |a: [Real; 3], b: [Real; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let zenith_pa = dot(zen, e).atan2(dot(zen, n));
    let sin_alt = dot(zen, u);
    let alt = sin_alt.asin();
    // 方位角：地平北向 = 天北极在地平面上的投影
    let pole = [real_const(0.0), real_const(0.0), real_const(1.0)];
    let hn_raw = [pole[0] - sp * zen[0], pole[1] - sp * zen[1], pole[2] - sp * zen[2]];
    let hnn = (hn_raw[0] * hn_raw[0] + hn_raw[1] * hn_raw[1] + hn_raw[2] * hn_raw[2]).sqrt();
    let hn = [hn_raw[0] / hnn, hn_raw[1] / hnn, hn_raw[2] / hnn];
    let he = [hn[1] * zen[2] - hn[2] * zen[1], hn[2] * zen[0] - hn[0] * zen[2], hn[0] * zen[1] - hn[1] * zen[0]];
    let az = dot(u, he).atan2(dot(u, hn));
    let two_pi = real_const(2.0) * Real::pi();
    let az = if az < real_const(0.0) { az + two_pi } else { az };

    // 视高度与折射压扁：h_true = h_app − R(h_app)，竖直方向缩放 dh_app/dh_true = 1/(1 − R′(h_app))
    let refr = |h_app: Real| bennett_refraction_default(PlaneAngle::from_rad(h_app)).rad();
    let (app, vscale) = if alt < deg(-2.0) {
        (alt, real_const(1.0))
    } else {
        let mut h = alt;
        for _ in 0..8 {
            h = alt + refr(h);
        }
        let dh = deg(0.05);
        let dr = (refr(h + dh) - refr(h - dh)) / (real_const(2.0) * dh);
        (h, real_const(1.0) / (real_const(1.0) - dr))
    };

    // 站心视黄经黄纬（真黄道），天平动与月轴位置角
    let t = (jd_tt - real_const(2451545.0)) / real_const(36525.0);
    let (dpsi, deps) = nutation_for_apparent(t);
    let eps = mean_obliquity(t).rad() + deps.rad();
    let lambda = (u[1] * eps.cos() + u[2] * eps.sin()).atan2(u[0]);
    let beta = (u[2] * eps.cos() - u[1] * eps.sin()).asin();
    let (alpha, _) = right_ascension_declination(&topo);
    let (l, b, p) = libration_and_axis(t, lambda, beta, alpha.rad(), dpsi.rad(), eps);

    MoonDiskView {
        jd_tt,
        altitude: PlaneAngle::from_rad(alt),
        apparent_altitude: PlaneAngle::from_rad(app),
        azimuth: PlaneAngle::from_rad(az),
        zenith_position_angle: PlaneAngle::from_rad(zenith_pa),
        libration_longitude: PlaneAngle::from_rad(l),
        libration_latitude: PlaneAngle::from_rad(b),
        axis_position_angle: PlaneAngle::from_rad(p),
        vertical_scale: vscale,
    }
}

#[cfg(test)]
pub(crate) fn libration_and_axis_for_test(t: Real, lambda: Real, beta: Real, alpha: Real, dpsi: Real, eps: Real) -> (Real, Real, Real) {
    libration_and_axis(t, lambda, beta, alpha, dpsi, eps)
}
