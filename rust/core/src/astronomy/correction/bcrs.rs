//! 质心/BCRS 地心速度，供光行差等可选使用。

use crate::astronomy::ephemeris::Vsop87;
use crate::math::real::{real, Real};

/// 地心在 J2000 平黄道架下的速度（m/s），在日心质心系（BCRS）中。在日心质心系（BCRS）中。
/// vsop_earth：地球 VSOP87（日心系下地球位置/速度）；barycentric_planets：其余质心行星 (Vsop87, M_planet/M_sun)。
/// v_earth_bcrs = v_earth_helio + v_sun_bcrs，其中 v_sun_bcrs = - Σ (μ_i * v_i_helio)。
pub fn earth_velocity_ecliptic_j2000_bcrs(
    vsop_earth: &Vsop87,
    barycentric_planets: &[(&Vsop87, Real)],
    jd_tdb: Real,
) -> [Real; 3] {
    let v_earth = vsop_earth.velocity_ecliptic_j2000_m_per_s(jd_tdb);
    let mut v_sun_bcrs: [Real; 3] = [
        real(0.0),
        real(0.0),
        real(0.0),
    ];
    for (vsop, mass_ratio) in barycentric_planets {
        let v = vsop.velocity_ecliptic_j2000_m_per_s(jd_tdb);
        v_sun_bcrs[0] = v_sun_bcrs[0] - mass_ratio * v[0];
        v_sun_bcrs[1] = v_sun_bcrs[1] - mass_ratio * v[1];
        v_sun_bcrs[2] = v_sun_bcrs[2] - mass_ratio * v[2];
    }
    [
        v_earth[0] + v_sun_bcrs[0],
        v_earth[1] + v_sun_bcrs[1],
        v_earth[2] + v_sun_bcrs[2],
    ]
}
