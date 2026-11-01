//! 站心视位置与地平坐标。
//!
//! - [`topocentric`]：当日真赤道架下，地心视位置减去观测者位置（观测者由 GAST 转到同一架），结果架为 [`ReferenceFrame::TopocentricEquator`]，
//!   即变换图 TrueEquator → TopocentricEquator 边的算式；[`topocentric_state`] 连速度一并平移。
//! - [`diurnal_aberration_state`]、[`to_horizontal_state`]、[`refract_state`]：变换图站心之后三条边的算式，依次为周日光行差、转东北天、大气折射。
//!   上下文见 [`ObservationContext`]。
//! - [`horizontal`]：真赤道坐标 → 方位角（正北起、向东为正）与几何高度角（未含大气折射）。
//!   出没、带食判断请用标准地平高度阈值（如太阳 −0°50′）而非对负高度施折射——现有 Saemundsson 折射在真高度 < 0 时返回 0。

use crate::astronomy::correction::{refract_enu, Atmosphere, RefractionModel};
use crate::astronomy::geodesy::Observer;
use crate::astronomy::time::earth_rotation::earth_rotation_rate;
use crate::astronomy::pipeline::State6;
use crate::math::real::{real_const, Real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::position::Position;
use crate::quantity::reference_frame::ReferenceFrame;
use crate::quantity::velocity::Velocity;

/// 地平坐标：方位角（正北起向东，[0, 2π)）与高度角。
#[derive(Clone, Copy, Debug)]
pub struct HorizontalCoords {
    pub azimuth: PlaneAngle,
    pub altitude: PlaneAngle,
}

fn true_equator_epoch(p: &Position) -> crate::quantity::epoch::Epoch {
    match p.frame {
        ReferenceFrame::TrueEquator(e) => e,
        other => panic!("站心平移要求地心真赤道架（TrueEquator），实为 {other:?}"),
    }
}

/// 站心视位置（站心真赤道架）。`gast` 须与 `geocentric` 同一时刻（UT1 经 ΔT 由该 TT 得到）。
/// 只接受地心真赤道架，重复平移会 panic。
pub fn topocentric(geocentric: Position, observer: &Observer, gast: PlaneAngle) -> Position {
    let epoch = true_equator_epoch(&geocentric);
    let [gx, gy, gz] = geocentric.to_meters();
    let [ox, oy, oz] = observer.position_true_equator(gast, epoch).to_meters();
    Position::from_si_meters_in_frame(ReferenceFrame::TopocentricEquator(epoch), gx - ox, gy - oy, gz - oz)
}

/// 站心 6D 状态：位置同 [`topocentric`]，速度减去观测者随地球自转的速度。
pub fn topocentric_state(geocentric: &State6, observer: &Observer, gast: PlaneAngle) -> State6 {
    let position = topocentric(geocentric.position, observer, gast);
    let epoch = true_equator_epoch(&geocentric.position);
    let [vx, vy, vz] = geocentric.velocity.to_m_per_s();
    let [ox, oy, oz] = observer.velocity_true_equator(gast, epoch).to_m_per_s();
    State6::new(position, Velocity::from_si_m_per_s_in_frame(position.frame, vx - ox, vy - oy, vz - oz))
}

/// 地心之后各观测边所需的上下文。`gast` 须与所变换的 state 同一时刻（UT1 经 ΔT 由该 TT 得到）。
#[derive(Clone, Copy, Debug)]
pub struct ObservationContext {
    pub observer: Observer,
    pub gast: PlaneAngle,
    /// 只有折射边用到。
    pub atmosphere: Atmosphere,
}

impl ObservationContext {
    /// 大气取观测者海拔处的标准状态（海平面 101 kPa、10°C）。
    pub fn new(observer: Observer, gast: PlaneAngle) -> Self {
        Self { atmosphere: Atmosphere::standard_at_height(observer.height), observer, gast }
    }
}

/// 周日光行差：方向朝观测者自转速度偏转（一阶，最大约 0.32″），距离不变。
/// 速度不改：其变化量级为 v/c ≈ 1.6e-6 倍，且此后只用于方向的时间导数。
pub fn diurnal_aberration_state(topo: &State6, ctx: &ObservationContext) -> State6 {
    let ReferenceFrame::TopocentricEquator(epoch) = topo.frame() else {
        panic!("周日光行差要求站心真赤道架（TopocentricEquator），实为 {:?}", topo.frame());
    };
    let c = real_const(299792458.0);
    let [x, y, z] = topo.position.to_meters();
    let dist = (x * x + y * y + z * z).sqrt();
    let [vx, vy, vz] = ctx.observer.velocity_true_equator(ctx.gast, epoch).to_m_per_s();
    let u = [x / dist + vx / c, y / dist + vy / c, z / dist + vz / c];
    let scale = dist / (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
    let frame = ReferenceFrame::TopocentricAberratedEquator(epoch);
    let [wx, wy, wz] = topo.velocity.to_m_per_s();
    State6::from_si_in_frame(frame, u[0] * scale, u[1] * scale, u[2] * scale, wx, wy, wz)
}

/// 站心赤道 → 东北天：地方恒星时 θ = GAST + 东经，U 沿大地法线。速度含旋转项 Ṙ·r。
pub fn to_horizontal_state(equatorial: &State6, ctx: &ObservationContext) -> State6 {
    let ReferenceFrame::TopocentricAberratedEquator(epoch) = equatorial.frame() else {
        panic!("转地平要求已加周日光行差的站心赤道架（TopocentricAberratedEquator），实为 {:?}", equatorial.frame());
    };
    let theta = ctx.gast.rad() + ctx.observer.longitude.rad();
    let (st, ct) = (theta.sin(), theta.cos());
    let (sp, cp) = (ctx.observer.latitude.rad().sin(), ctx.observer.latitude.rad().cos());
    let o = real_const(0.0);
    let r = [[-st, ct, o], [-sp * ct, -sp * st, cp], [cp * ct, cp * st, sp]];
    let w = earth_rotation_rate();
    let r_dot = [
        [-(ct * w), -(st * w), o],
        [sp * st * w, -(sp * ct * w), o],
        [-(cp * st * w), cp * ct * w, o],
    ];
    equatorial.apply_transition(&r, &r_dot, ReferenceFrame::HorizontalGeometric(epoch))
}

/// 大气折射：几何地平 → 视地平。折射是方向的非线性映射，没有 6×6 转移矩阵，速度取该映射沿速度方向的中心差分（±1 s）。
pub fn refract_state(geometric: &State6, model: RefractionModel, atmosphere: &Atmosphere) -> State6 {
    let ReferenceFrame::HorizontalGeometric(epoch) = geometric.frame() else {
        panic!("折射要求几何地平架（HorizontalGeometric），实为 {:?}", geometric.frame());
    };
    let (p, v) = geometric.to_meters_and_m_per_s();
    let at = |s: Real| refract_enu([p[0] + v[0] * s, p[1] + v[1] * s, p[2] + v[2] * s], model, atmosphere);
    let (out, ahead, behind) = (at(real_const(0.0)), at(real_const(1.0)), at(real_const(-1.0)));
    let d = |i: usize| (ahead[i] - behind[i]) / real_const(2.0);
    State6::from_si_in_frame(ReferenceFrame::HorizontalApparent(epoch), out[0], out[1], out[2], d(0), d(1), d(2))
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

/// 真赤道或站心真赤道架位置 → 地平坐标（几何高度）。与管线图到 HorizontalGeometric 的路径相比不含周日光行差（≤0.32″），供逐点循环用。站心高度请先用 [`topocentric`]。
pub fn horizontal(p: &Position, observer: &Observer, gast: PlaneAngle) -> HorizontalCoords {
    let (_, dec) = right_ascension_declination(p);
    horizontal_from_hour_angle(hour_angle(p, observer, gast), dec, observer.latitude)
}
