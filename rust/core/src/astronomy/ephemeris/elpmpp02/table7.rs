//! ELP-MPP02 论文 §5.2 Table 7：J2000 平黄道 → ICRS(GCRF)。
//!
//! 用于将历表输出的 (x,y,z) 从「J2000 平黄道平春分」转到 ICRS，与 DE406 等在同一架下比较。
//! 公式：v_icrs = Rz(φ) Rx(−ε) v_ecl；v_icrs = Rz(φ) Rx(−ε) v_ecl。

const ARCSEC_TO_RAD: f64 = std::f64::consts::PI / (180.0 * 3600.0);

/// Table 7 ICRS 行：ε = 23°26′21″ + 0.41100″（黄赤交角），φ = -0.05542″。
const EPSILON_ARCSEC: f64 = 23.0 * 3600.0 + 26.0 * 60.0 + 21.0 + 0.41100;
const PHI_ARCSEC: f64 = -0.05542;

/// J2000 平黄道直角坐标 → ICRS(GCRF) 直角坐标；单位不变（调用方保证一致，如 m 或 km）。
#[inline]
pub fn ecliptic_j2000_to_icrs(x_ecl: f64, y_ecl: f64, z_ecl: f64) -> (f64, f64, f64) {
    let eps = EPSILON_ARCSEC * ARCSEC_TO_RAD;
    let phi = PHI_ARCSEC * ARCSEC_TO_RAD;
    let (ce, se) = (eps.cos(), eps.sin());
    // Rx(−ε) v_ecl
    let x1 = x_ecl;
    let y1 = y_ecl * ce - z_ecl * se;
    let z1 = y_ecl * se + z_ecl * ce;
    // Rz(φ)
    let (cphi, sphi) = (phi.cos(), phi.sin());
    let x2 = x1 * cphi - y1 * sphi;
    let y2 = x1 * sphi + y1 * cphi;
    (x2, y2, z1)
}

/// J2000 平黄道 → ICRS 的旋转矩阵 Rz(φ) Rx(−ε)（Real），供历表提供者把月亮状态转到 ICRS。
///
/// ELP/MPP02 的 J2000 平黄道由 Table 7 相对 ICRS 定位，本项目拟合 DE431/DE441 也按此架（冗余旋转 ≤ 0.0013″ 验证其适用）；
/// 不可与 VSOP87 的「J2000 动力学黄道 → FK5（ε₀ = 84381.406″）→ ICRS」混用，两者春分点差约 0.044″
/// （doc/astronomy/model-pairing.md）。
pub fn ecliptic_j2000_to_icrs_matrix() -> crate::quantity::rotation_matrix::RotationMatrix {
    use crate::math::real::{real, RealOps};
    let (eps, phi) = (real(EPSILON_ARCSEC * ARCSEC_TO_RAD), real(PHI_ARCSEC * ARCSEC_TO_RAD));
    let (se, ce) = (eps.sin(), eps.cos());
    let (sp, cp) = (phi.sin(), phi.cos());
    let z = real(0.0);
    // Rx(−ε)：[[1,0,0],[0,cε,−sε],[0,sε,cε]]；Rz(φ)：[[cφ,−sφ,0],[sφ,cφ,0],[0,0,1]]
    crate::quantity::rotation_matrix::RotationMatrix::from_array([
        [cp, -sp * ce, sp * se],
        [sp, cp * ce, -cp * se],
        [z, se, ce],
    ])
}

/// ICRS(GCRF) 直角坐标 → J2000 平黄道直角坐标：[ecliptic_j2000_to_icrs] 之逆，v_ecl = Rx(ε) Rz(−φ) v_icrs。
#[inline]
pub fn icrs_to_ecliptic_j2000(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let eps = EPSILON_ARCSEC * ARCSEC_TO_RAD;
    let phi = PHI_ARCSEC * ARCSEC_TO_RAD;
    let (cphi, sphi) = (phi.cos(), phi.sin());
    // Rz(−φ)
    let x1 = x * cphi + y * sphi;
    let y1 = -x * sphi + y * cphi;
    // Rx(ε)
    let (ce, se) = (eps.cos(), eps.sin());
    (x1, y1 * ce + z * se, -y1 * se + z * ce)
}

#[cfg(test)]
mod tests {
    #[test]
    fn matrix_matches_f64_function() {
        use crate::math::real::{real, RealOps};
        let m = super::ecliptic_j2000_to_icrs_matrix();
        let v = m.mul_vec([real(1.0e5), real(-2.0e5), real(3.0e4)]);
        let (x, y, z) = super::ecliptic_j2000_to_icrs(1.0e5, -2.0e5, 3.0e4);
        assert!((v[0].as_f64() - x).abs() < 1e-8 && (v[1].as_f64() - y).abs() < 1e-8 && (v[2].as_f64() - z).abs() < 1e-8);
    }

    #[test]
    fn inverse_round_trip() {
        let (x, y, z) = super::ecliptic_j2000_to_icrs(1.0e5, -2.0e5, 3.0e4);
        let (a, b, c) = super::icrs_to_ecliptic_j2000(x, y, z);
        assert!((a - 1.0e5).abs() < 1e-9 && (b + 2.0e5).abs() < 1e-9 && (c - 3.0e4).abs() < 1e-9);
    }
}
