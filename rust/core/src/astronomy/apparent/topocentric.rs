//! 站心视位置与地平坐标。
//!
//! - [`topocentric`]：当日真赤道架下，地心视位置减去观测者位置（观测者由 GAST 转到同一架）。周日光行差（≤0.32″）未计。
//! - [`horizontal`]：真赤道坐标 → 方位角（正北起、向东为正）与几何高度角（未含大气折射）。
//!   出没、带食判断请用标准地平高度阈值（如太阳 −0°50′）而非对负高度施折射——现有 Saemundsson 折射在真高度 < 0 时返回 0。

use crate::astronomy::geodesy::Observer;
use crate::math::real::{Real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::position::Position;
use crate::quantity::reference_frame::ReferenceFrame;

/// 地平坐标：方位角（正北起向东，[0, 2π)）与高度角。
#[derive(Clone, Copy, Debug)]
pub struct HorizontalCoords {
    pub azimuth: PlaneAngle,
    pub altitude: PlaneAngle,
}

fn true_equator_epoch(p: &Position) -> crate::quantity::epoch::Epoch {
    match p.frame {
        ReferenceFrame::TrueEquator(e) => e,
        other => panic!("站心/地平计算要求当日真赤道架（TrueEquator），实为 {other:?}"),
    }
}

/// 站心视位置（真赤道架）。`gast` 须与 `geocentric` 同一时刻（UT1 经 ΔT 由该 TT 得到）。
pub fn topocentric(geocentric: Position, observer: &Observer, gast: PlaneAngle) -> Position {
    let epoch = true_equator_epoch(&geocentric);
    let [gx, gy, gz] = geocentric.to_meters();
    let [ox, oy, oz] = observer.position_true_equator(gast, epoch).to_meters();
    Position::from_si_meters_in_frame(geocentric.frame, gx - ox, gy - oy, gz - oz)
}

/// 赤经、赤纬（真赤道架）。
pub fn right_ascension_declination(p: &Position) -> (PlaneAngle, PlaneAngle) {
    let [x, y, z] = p.to_meters();
    let rho = (x.clone() * x.clone() + y.clone() * y.clone()).sqrt();
    (PlaneAngle::from_rad(y.atan2(x)).wrap_to_2pi(), PlaneAngle::from_rad(z.atan2(rho)))
}

/// 地方时角 H = GAST + 东经 − 赤经（[0, 2π)）。
pub fn hour_angle(p: &Position, observer: &Observer, gast: PlaneAngle) -> PlaneAngle {
    let (ra, _) = right_ascension_declination(p);
    PlaneAngle::from_rad(gast.rad() + observer.longitude.rad() - ra.rad()).wrap_to_2pi()
}

/// 由时角、赤纬与大地纬度求地平坐标（几何高度，未含折射）。
pub fn horizontal_from_hour_angle(h: PlaneAngle, dec: PlaneAngle, latitude: PlaneAngle) -> HorizontalCoords {
    let (sh, ch) = (h.rad().sin(), h.rad().cos());
    let (sd, cd) = (dec.rad().sin(), dec.rad().cos());
    let (sp, cp) = (latitude.rad().sin(), latitude.rad().cos());
    let sin_alt: Real = sp.clone() * sd.clone() + cp.clone() * cd.clone() * ch.clone();
    // 方位角自北向东：tan A = −cosδ sinH / (sinδ cosφ − cosδ sinφ cosH)；两者平方和即 cos²h
    let num = -(cd.clone() * sh);
    let den = sd * cp - cd * sp * ch;
    // 用 atan2 而非 asin：天顶附近 sin h 舍入后可略大于 1，asin 得 NaN
    let altitude = PlaneAngle::from_rad(sin_alt.atan2((num.clone() * num.clone() + den.clone() * den.clone()).sqrt()));
    HorizontalCoords { azimuth: PlaneAngle::from_rad(num.atan2(den)).wrap_to_2pi(), altitude }
}

/// 真赤道架位置（地心或站心）→ 地平坐标。站心高度请先用 [`topocentric`]。
pub fn horizontal(p: &Position, observer: &Observer, gast: PlaneAngle) -> HorizontalCoords {
    let (_, dec) = right_ascension_declination(p);
    horizontal_from_hour_angle(hour_angle(p, observer, gast), dec, observer.latitude)
}
