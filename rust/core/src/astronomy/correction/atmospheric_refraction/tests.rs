use super::*;
use crate::math::angle::deg2rad;
use crate::math::real::{real, RealOps};
use crate::math::series::arcsec_to_rad;
use crate::quantity::angle::PlaneAngle;
use crate::quantity::{pressure::Pressure, thermodynamic_temperature::ThermodynamicTemperature};

const PI: f64 = core::f64::consts::PI;

fn std_pressure() -> Pressure {
    Pressure::from_kpa(real(101.0))
}
fn std_temperature() -> ThermodynamicTemperature {
    ThermodynamicTemperature::from_celsius(real(10.0))
}

#[test]
fn bennett_positive_altitude() {
    let alt = PlaneAngle::from_rad(deg2rad(30.0));
    let r = bennett_refraction_default(alt);
    assert!(r.rad() > real(0) && r.rad() < deg2rad(1.0));
}

#[test]
fn bennett_at_90_deg_abs_r_leq_0_9_arcsec() {
    let alt = PlaneAngle::parse("90°").unwrap();
    let r = bennett_refraction_default(alt);
    let bound_rad = arcsec_to_rad(0.9);
    assert!(r.rad().abs() <= bound_rad);
}

#[test]
fn bennett_improved_at_90_deg_abs_r_leq_0_9_arcsec() {
    let alt = PlaneAngle::parse("90°").unwrap();
    let r = bennett_improved_refraction_default(alt);
    let bound_rad = arcsec_to_rad(0.9);
    assert!(r.rad().abs() <= bound_rad);
}

#[test]
fn xjw_positive_altitude() {
    let alt = PlaneAngle::from_rad(deg2rad(30.0));
    let r = xjw_refraction(alt);
    assert!(r.rad() > real(0) && r.rad() < deg2rad(1.0));
}

fn assert_near_rad(a: crate::math::real::Real, b: crate::math::real::Real, eps_rad: f64) {
    assert!(a.is_near(b, eps_rad), "{:?} vs {:?} (eps {})", a, b, eps_rad);
}

#[test]
fn saemundsson_consistent_with_bennett_within_0_1_arcmin_at_23_deg() {
    let ha = PlaneAngle::parse("23°").unwrap();
    let r1 = bennett_refraction_default(ha);
    let h = PlaneAngle::from_rad(ha.rad() - r1.rad());
    let r2 = saemundsson_refraction(h, std_pressure(), std_temperature());
    let ha1 = h.rad() + r2.rad();
    let eps = 0.1 * (PI / 180.0 / 60.0);
    assert_near_rad(ha.rad(), ha1, eps);
}

#[test]
fn saemundsson_consistent_with_bennett_within_0_1_arcmin_dms() {
    let ha = PlaneAngle::parse("56°34'23\".5").unwrap();
    let r1 = bennett_refraction_default(ha);
    let h = PlaneAngle::from_rad(ha.rad() - r1.rad());
    let r2 = saemundsson_refraction(h, std_pressure(), std_temperature());
    let ha1 = h.rad() + r2.rad();
    let eps = 0.1 * (PI / 180.0 / 60.0);
    assert_near_rad(ha.rad(), ha1, eps);
}

#[test]
fn smart_vs_bennett_within_1_9_arcsec() {
    let eps_rad = arcsec_to_rad(1.9).as_f64();
    for s in &["45°", "75°", "95°"] {
        let ha = PlaneAngle::parse(s).unwrap();
        let b = bennett_refraction_default(ha);
        let sm = smart_refraction(ha, std_pressure(), std_temperature());
        assert_near_rad(b.rad(), sm.rad(), eps_rad);
    }
}

#[test]
fn meeus_consistent_with_smart_within_0_2_arcsec() {
    let ha = PlaneAngle::parse("56°34'23\".5").unwrap();
    let r1 = smart_refraction(ha, std_pressure(), std_temperature());
    let h = PlaneAngle::from_rad(ha.rad() - r1.rad());
    let r2 = meeus_refraction(h, std_pressure(), std_temperature());
    let ha1 = h.rad() + r2.rad();
    let eps_rad = arcsec_to_rad(0.2).as_f64();
    assert_near_rad(ha.rad(), ha1, eps_rad);
}
