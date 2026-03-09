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
