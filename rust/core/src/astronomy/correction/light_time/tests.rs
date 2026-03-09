use super::*;
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, RealOps};
use crate::quantity::length::Length;
use crate::quantity::unit::LengthUnit;

#[test]
fn light_time_1au() {
    let dist = Length::from_value(real(149_597_870_700.0), LengthUnit::Meter);
    let days = light_time(dist).in_days_value();
    assert!(days > real(0.005) && days < real(0.006));
}

#[test]
fn retarded_time_point_constant_distance() {
    let jd = real(2451545.0);
    let d = real(1.5e11);
    let t = TimePoint::new(TimeScale::TT, jd);
    let tr = retarded_time_point(t, |_| Length::from_value(d, LengthUnit::Meter), 3);
    let tau = light_time(Length::from_value(d, LengthUnit::Meter)).in_days_value();
    let expected_jd = jd - tau;
    assert!(tr.jd.is_near(expected_jd, 1e-12));
}
