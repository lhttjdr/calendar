use crate::astronomy::ephemeris::{
    position_velocity, position_velocity_mean_only, position_velocity_with_max_terms, Elpmpp02Data,
    Vsop87,
};
use crate::astronomy::time::TimePoint;
use crate::math::real::{Real, RealOps};
use crate::quantity::angle::PlaneAngle;

/// 历表返回 Real。
pub fn sun_ecliptic_longitude(vsop: &Vsop87, t: TimePoint) -> PlaneAngle {
    let pos = vsop.position(t);
    let rad_r = (pos.L.rad() + Real::pi()).wrap_to_2pi();
    PlaneAngle::from_rad(rad_r)
}

/// 历表返回 Real。
pub fn moon_ecliptic_longitude(elp: &Elpmpp02Data, t: TimePoint) -> PlaneAngle {
    let (pos, _vel) = position_velocity(elp, t);
    let rad_r = pos.y.meters().atan2(pos.x.meters()).wrap_to_2pi();
    PlaneAngle::from_rad(rad_r)
}

/// 同上，可限制 ELP 级数项数（粗算时用 coarse_max_terms 加速）。
pub fn moon_ecliptic_longitude_with_max_terms(
    elp: &Elpmpp02Data,
    t: TimePoint,
    max_terms: Option<u32>,
) -> PlaneAngle {
    let (pos, _vel) = position_velocity_with_max_terms(elp, t, max_terms);
    let rad_r = pos.y.meters().atan2(pos.x.meters()).wrap_to_2pi();
    PlaneAngle::from_rad(rad_r)
}

pub fn moon_ecliptic_longitude_mean_only(elp: &Elpmpp02Data, t: TimePoint) -> PlaneAngle {
    let (pos, _vel) = position_velocity_mean_only(elp, t);
    let rad_r = pos.y.meters().atan2(pos.x.meters()).wrap_to_2pi();
    PlaneAngle::from_rad(rad_r)
}
