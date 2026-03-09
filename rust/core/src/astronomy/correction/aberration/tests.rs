use crate::math::real::{real, Real, RealOps, ToReal};
use super::*;

fn r3(x: impl ToReal, y: impl ToReal, z: impl ToReal) -> [Real; 3] {
    [real(x), real(y), real(z)]
}

#[test]
fn annual_aberration_zero_velocity() {
    let r = r3(real(1), real(0), real(0));
    let v = r3(real(0), real(0), real(0));
    let e_app = annual_aberration_direction(r, v);
    assert!(e_app[0].is_near(real(1), 1e-15));
    assert!(e_app[1].abs().is_near(real(0), 1e-15) && e_app[2].abs().is_near(real(0), 1e-15));
}

#[test]
fn annual_aberration_unit_norm() {
    let r = r3(real(1), real(0.5), real(0.2));
    let v = r3(real(0.01), real(-0.02), real(0));
    let e_app = annual_aberration_direction(r, v);
    let n = (e_app[0] * e_app[0] + e_app[1] * e_app[1] + e_app[2] * e_app[2]).sqrt();
    assert!(n.is_near(real(1), 1e-14));
}

#[test]
fn annual_aberration_direction_derivative_zero_velocity() {
    let r = r3(real(1), real(0), real(0));
    let v = r3(real(0), real(0), real(0));
    let dr_dt = r3(real(0.01), real(0), real(0));
    let dv_dt = r3(real(0), real(0), real(0));
    let de = annual_aberration_direction_derivative(r, v, dr_dt, dv_dt);
    assert!(de[0].abs().is_near(real(0), 1e-10) && de[1].abs().is_near(real(0), 1e-10) && de[2].abs().is_near(real(0), 1e-10));
}
