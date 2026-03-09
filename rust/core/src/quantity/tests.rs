use super::angle::PlaneAngle;
use super::coord_components::SphericalVelocityCoordComponents;
use super::displacement::Displacement;
use super::duration::Duration;
use super::frame_metric::CoordKind;
use super::length::Length;
use super::position::Position;
use super::reference_frame::ReferenceFrame;
use super::unit::LengthUnit;
use super::velocity::Velocity;
use crate::math::real::{real, RealOps};
use crate::quantity::epoch::Epoch;

#[test]
fn length_from_meters_and_km() {
    let l = Length::from_value(real(1000.0), LengthUnit::Meter);
    assert!(l.meters().is_near(real(1000.0), 1e-10));
    assert!(l.km().is_near(real(1.0), 1e-10));
    let l2 = Length::from_value(real(1.0), LengthUnit::Kilometer);
    assert!(l2.meters().is_near(real(1000.0), 1e-10));
}

#[test]
fn length_div_duration_is_speed() {
    let len = Length::from_value(real(100.0), LengthUnit::Meter);
    let dur = Duration::in_seconds(real(10.0));
    let speed = len.div_duration(dur);
    assert!(speed.m_per_s().is_near(real(10.0), 1e-10));
}

#[test]
fn plane_angle_div_duration_is_angular_rate() {
    let angle = PlaneAngle::from_rad(real(core::f64::consts::TAU));
    let dur = Duration::in_seconds(real(1.0));
    let rate = angle.div_duration(dur);
    assert!(rate.rad_per_second().is_near(real(core::f64::consts::TAU), 1e-10));
}

#[test]
fn position_from_si_to_meters() {
    let p = Position::from_si_meters(real(1.0), real(2.0), real(3.0));
    let [x, y, z] = p.to_meters();
    assert!(x.is_near(real(1.0), 1e-10) && y.is_near(real(2.0), 1e-10) && z.is_near(real(3.0), 1e-10));
    assert_eq!(p.frame, ReferenceFrame::FK5);
}

#[test]
fn position_same_frame_and_distance_to() {
    let p1 = Position::from_si_meters_in_frame(ReferenceFrame::FK5, real(1.0), real(0.0), real(0.0));
    let p2 = Position::from_si_meters_in_frame(ReferenceFrame::FK5, real(2.0), real(0.0), real(0.0));
    assert!(p1.same_frame_as(p2));
    let d = p1.distance_to(p2);
    assert!(d.meters().is_near(real(1.0), 1e-10));
}

#[test]
fn velocity_from_si() {
    let v = Velocity::from_si_m_per_s(real(1.0), real(2.0), real(3.0));
    let [a, b, c] = v.to_m_per_s();
    assert!(a.is_near(real(1.0), 1e-10) && b.is_near(real(2.0), 1e-10) && c.is_near(real(3.0), 1e-10));
}

#[test]
fn displacement_magnitude_and_div_duration() {
    let d = Displacement::from_si_meters_in_frame(ReferenceFrame::FK5, real(3.0), real(4.0), real(0.0));
    assert!(d.magnitude().meters().is_near(real(5.0), 1e-10));
    let dur = Duration::in_seconds(real(2.0));
    let v = d.div_duration(dur);
    assert!(v.same_frame_as(Velocity::from_si_m_per_s_in_frame(
        ReferenceFrame::FK5, real(1.5), real(2.0), real(0.0)
    )));
    assert!(v.vx.m_per_s().is_near(real(1.5), 1e-10));
}

#[test]
fn position_apply_transform() {
    let p = Position::from_si_meters_in_frame(ReferenceFrame::FK5, real(1.0), real(0.0), real(0.0));
    let q = p.apply_transform(ReferenceFrame::FK5, |[x, y, z]| [x, z, -y]);
    assert_eq!(q.frame, ReferenceFrame::FK5);
    assert!(q.x.meters().is_near(real(1.0), 1e-10) && q.z.meters().is_near(real(0), 1e-10));
}

#[test]
fn reference_frame_is_epoch_dependent_and_id_str() {
    let ep = Epoch::j2000();
    assert!(!ReferenceFrame::ICRS.is_epoch_dependent());
    assert!(!ReferenceFrame::FK5.is_epoch_dependent());
    assert!(ReferenceFrame::MeanEcliptic(ep).is_epoch_dependent());
    assert!(ReferenceFrame::MeanEquator(ep).is_epoch_dependent());
    assert!(ReferenceFrame::TrueEquator(ep).is_epoch_dependent());
    assert!(ReferenceFrame::TrueEcliptic(ep).is_epoch_dependent());
    assert!(ReferenceFrame::ApparentEcliptic(ep).is_epoch_dependent());
    assert!(ReferenceFrame::ApparentEquator(ep).is_epoch_dependent());
    assert!(!ReferenceFrame::Elpmpp02MeanLunar.is_epoch_dependent());
    assert!(!ReferenceFrame::Elpmpp02LaskarCartesian.is_epoch_dependent());

    assert_eq!(ReferenceFrame::ICRS.id_str(), "ICRS");
    assert_eq!(ReferenceFrame::FK5.id_str(), "FK5");
    assert_eq!(ReferenceFrame::MeanEcliptic(ep).id_str(), "MeanEcliptic(epoch)");
    assert_eq!(ReferenceFrame::MeanEquator(ep).id_str(), "MeanEquator(epoch)");
    assert_eq!(ReferenceFrame::TrueEquator(ep).id_str(), "TrueEquator(epoch)");
    assert_eq!(ReferenceFrame::TrueEcliptic(ep).id_str(), "TrueEcliptic(epoch)");
    assert_eq!(ReferenceFrame::ApparentEcliptic(ep).id_str(), "ApparentEcliptic(epoch)");
    assert_eq!(ReferenceFrame::ApparentEquator(ep).id_str(), "ApparentEquator(epoch)");
    assert_eq!(ReferenceFrame::Elpmpp02MeanLunar.id_str(), "ELPMPP02_MEAN_LUNAR");
    assert_eq!(ReferenceFrame::Elpmpp02LaskarCartesian.id_str(), "ELPMPP02_LASKAR_CARTESIAN");
}

#[test]
fn frame_metric_coord_kind_velocity_scale_factors() {
    let one = real(1.0);
    let cart = CoordKind::Cartesian.velocity_scale_factors((one, one, one));
    assert!(cart.0[0].is_near(one, 1e-10) && cart.0[1].is_near(one, 1e-10) && cart.0[2].is_near(one, 1e-10));

    let r = real(6371e3);
    let lat0 = real(0.0);
    let spherical = CoordKind::Spherical.velocity_scale_factors((r, real(0.0), lat0));
    assert!(spherical.0[0].is_near(one, 1e-10));
    assert!(spherical.0[1].is_near(r, 1e-10));
    assert!(spherical.0[2].is_near(r, 1e-10));
}

#[test]
fn coord_components_spherical_velocity_roundtrip() {
    let r = Length::from_value(real(1.0), LengthUnit::Kilometer);
    let lon = PlaneAngle::from_rad(real(0.1));
    let lat = PlaneAngle::from_rad(real(0.2));
    let v = Velocity::from_si_m_per_s(real(100.0), real(-50.0), real(20.0));
    let vec_speed = crate::quantity::vector3::Vector3::from_speeds([v.vx, v.vy, v.vz]);
    let comp = SphericalVelocityCoordComponents::from_vector3_at(vec_speed, r, lon, lat);
    let back = comp.to_vector3_at(r, lon, lat);
    assert!(back.x().m_per_s().is_near(v.vx.m_per_s(), 1e-8));
    assert!(back.y().m_per_s().is_near(v.vy.m_per_s(), 1e-8));
    assert!(back.z().m_per_s().is_near(v.vz.m_per_s(), 1e-8));
}
