//! 行星视星等（V 波段）：Mallama & Hilton (2018)《Computing apparent planetary magnitudes for The Astronomical Almanac》，
//! *Astronomy and Computing* 25, 10（arXiv:1808.01973），即《天文年历》现行算法；作者程序 Ap_Mag V3 与测试数据原件在
//! `vendor/planetary_magnitudes/`。
//!
//! 通式（论文式 1）V = 5 log₁₀(r Δ) + V₁(0) + 相位函数；r、Δ 为行星到太阳、到观测者的距离（AU），α 为相位角（行星处
//! 太阳与观测者的夹角，度）。各行星公式见论文式 2—17，系数照录原文；土星式 12 的三次项取作者程序的 −1.506×10⁻⁶
//! （论文正文作 −1.505×10⁻⁶，与作者程序的官方测试输出不符，见测试）。
//!
//! 公式分支与作者程序一致：相位角超出观测范围时仍按相应公式计算（作者程序只给警告，见 [`within_observed_phase_limits`]）；
//! 只有两种情形不给星等（返回 `None`）：土星含环而相位角 > 6.5° 或环倾角 > 27°；海王星相位角 > 1.9° 而在 2000 年以前。
//!
//! 火星（式 6、7）的自转修正 L(λe) 与季节修正 L(Ls) 取作者程序的两张表（Mallama 2007, *Icarus* 192, 404，表 6、表 8，
//! 10° 间隔）以 Stirling 四阶插值。作者程序的差分循环只算到 δ(2)，四阶差分用到未赋值的 δ(3)（注释原意为 range(4)），
//! 此处按完整的 Stirling 公式实现，与官方输出之差见测试。由历表算星等时只能给出 L(Ls)（需日心黄经）；L(λe) 需火星直下点
//! 经度（IAU 火星自转模型），尚未接入，记为 0（幅度约 ±0.06 星等）。
//!
//! 已知省略：土星、天王星自转轴取 IAU J2000 值（固定），未计极轴的长期变化（土星约 −0.036°/cy，到前 200 年约 0.8°，
//! 对土星星等影响 ≤ 0.02）。

use crate::astronomy::pipeline::{Body, EphemerisProvider, FrameMapper, Fk5ToIcrsBias, LightTimeCorrector, Planet, TransformGraph};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, Real, RealOps};
use crate::quantity::reference_frame::ReferenceFrame;

/// 计算星等所需的几何量。
#[derive(Clone, Copy, Debug)]
pub struct MagnitudeGeometry {
    /// 行星到太阳距离（AU）
    pub r_au: Real,
    /// 行星到观测者距离（AU）
    pub delta_au: Real,
    /// 相位角 α（度）
    pub phase_deg: Real,
    /// 行星面心纬度：太阳直射点（度；土星、天王星用）
    pub sun_sub_lat_deg: Real,
    /// 行星面心纬度：观测者直下点（度；土星、天王星用）
    pub observer_sub_lat_deg: Real,
    /// 儒略年（海王星用，式 16）
    pub year: Real,
    /// 行星日心黄经（J2000 黄道，度；火星季节修正用）
    pub helio_ecl_long_deg: Real,
    /// 火星直下点经度：(观测者, 太阳)（度；火星自转修正用，未知为 None）
    pub mars_sub_longitudes_deg: Option<(Real, Real)>,
}

/// 天王星扁率（论文 §3.7）。
const URANUS_FLATTENING: f64 = 0.0022927;

/// 火星自转修正表 L1（作者程序，Mallama 2007 表 6）：下标 i+2 对应有效中央经度 10i°，两端各补两项以便插值。
const MARS_L1: [f64; 40] = [
    0.024, 0.034, 0.036, 0.045, 0.038, 0.023, 0.015, 0.011, 0.000, -0.012, -0.018, -0.036, -0.044, -0.059, -0.060, -0.055,
    -0.043, -0.041, -0.041, -0.036, -0.036, -0.018, -0.038, -0.011, 0.002, 0.004, 0.018, 0.019, 0.035, 0.050, 0.035, 0.027,
    0.037, 0.048, 0.025, 0.022, 0.024, 0.034, 0.036, 0.045,
];
/// 火星季节修正表 L2（作者程序，Mallama 2007 表 8）：下标 i+2 对应 Ls = 10i°。
const MARS_L2: [f64; 40] = [
    -0.030, -0.017, -0.029, -0.017, -0.014, -0.006, -0.018, -0.020, -0.014, -0.030, -0.008, -0.040, -0.024, -0.037, -0.036,
    -0.032, 0.010, 0.010, -0.001, 0.044, 0.025, -0.004, -0.016, -0.008, 0.029, -0.054, -0.033, 0.055, 0.017, 0.052, 0.006,
    0.087, 0.006, 0.064, 0.030, 0.019, -0.030, -0.017, -0.029, -0.017,
];
/// 日心黄经 + 此值 = 火星季节角 Ls（作者程序 `Ls_offset`）。
const MARS_LS_OFFSET_DEG: f64 = -85.0;

fn deg_to_rad(d: Real) -> Real {
    d * real(core::f64::consts::PI / 180.0)
}

fn rad_to_deg(r: Real) -> Real {
    r * real(180.0 / core::f64::consts::PI)
}

/// 行星面心纬度 → 行星面图纬度（论文式 13）：φ′ = atan(tan φ / (1 − f)²)。度。
pub fn planetographic_latitude_deg(planetocentric_deg: Real, flattening: f64) -> Real {
    let k = real((1.0 - flattening) * (1.0 - flattening));
    let phi = deg_to_rad(planetocentric_deg);
    rad_to_deg((phi.sin() / k).atan2(phi.cos()))
}

/// 10° 间隔表的 Stirling 四阶插值（Duncombe, *Explanatory Supplement* §14.2.4），角度取 [0, 360)。
fn stirling_10deg(table: &[f64; 40], angle_deg: Real) -> Real {
    let a = angle_deg.as_f64().rem_euclid(360.0);
    let zp = (a / 10.0).floor() as usize;
    let p = real(a / 10.0 - zp as f64);
    let f = |i: usize| real(table[zp + i]);
    // 五点 f(zp..zp+4)，中心为 f(zp+2)
    let d: [Real; 4] = std::array::from_fn(|i| f(i + 1) - f(i));
    let d2: [Real; 3] = std::array::from_fn(|i| d[i + 1] - d[i]);
    let d3: [Real; 2] = [d2[1] - d2[0], d2[2] - d2[1]];
    let d4 = d3[1] - d3[0];
    let a4 = d4 / real(24.0);
    let a3 = (d3[0] + d3[1]) / real(12.0);
    let a2 = d2[1] / real(2.0) - a4;
    let a1 = (d[1] + d[2]) / real(2.0) - a3;
    f(2) + p * (a1 + p * (a2 + p * (a3 + p * a4)))
}

/// 火星有效中央经度（作者程序）：观测者与太阳直下点经度之均，二者相差超过 180° 时加 180°。
pub fn mars_effective_central_meridian_deg(sub_observer_long_deg: Real, sub_sun_long_deg: Real) -> Real {
    let mut e = (sub_observer_long_deg + sub_sun_long_deg) / real(2.0);
    if (sub_observer_long_deg - sub_sun_long_deg).abs() > real(180.0) {
        e = e + real(180.0);
    }
    if e > real(360.0) {
        e = e - real(360.0);
    }
    e
}

/// 火星自转修正 L(λe)（星等）。
pub fn mars_rotation_correction(effective_cm_deg: Real) -> Real {
    stirling_10deg(&MARS_L1, effective_cm_deg)
}

/// 火星季节修正 L(Ls)（星等），自变量为日心黄经（J2000 黄道，度）。
pub fn mars_orbital_correction(helio_ecl_long_deg: Real) -> Real {
    stirling_10deg(&MARS_L2, helio_ecl_long_deg + real(MARS_LS_OFFSET_DEG))
}

/// Σ cₖ αᵏ（α 度）。
fn poly(a: Real, coeffs: &[f64]) -> Real {
    Real::power_series_at(coeffs, a)
}

/// 可算星等的天体（含地球，供他处观测地球时用）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MagnitudePlanet {
    Mercury,
    Venus,
    Earth,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
}

impl From<Planet> for MagnitudePlanet {
    fn from(p: Planet) -> Self {
        match p {
            Planet::Mercury => MagnitudePlanet::Mercury,
            Planet::Venus => MagnitudePlanet::Venus,
            Planet::Mars => MagnitudePlanet::Mars,
            Planet::Jupiter => MagnitudePlanet::Jupiter,
            Planet::Saturn => MagnitudePlanet::Saturn,
            Planet::Uranus => MagnitudePlanet::Uranus,
            Planet::Neptune => MagnitudePlanet::Neptune,
        }
    }
}

/// 相位角是否在论文给出的观测/可靠范围内（作者程序在范围外只给警告，照算）。
pub fn within_observed_phase_limits(planet: MagnitudePlanet, phase_deg: Real) -> bool {
    let a = phase_deg.as_f64();
    match planet {
        MagnitudePlanet::Mercury => (2.0..=170.0).contains(&a),
        MagnitudePlanet::Venus => (2.0..=179.0).contains(&a),
        MagnitudePlanet::Earth => a <= 170.0,
        MagnitudePlanet::Mars => a <= 120.0,
        MagnitudePlanet::Jupiter => a <= 130.0,
        MagnitudePlanet::Saturn => a <= 150.0,
        MagnitudePlanet::Uranus => a <= 154.0,
        MagnitudePlanet::Neptune => a <= 133.14,
    }
}

/// 行星视星等 V（分支同作者程序 Ap_Mag V3）。`saturn_rings`：土星是否含环（式 10 含环；式 11、12 只算球体）。
/// 土星含环超出 α ≤ 6.5°、β ≤ 27°，或海王星 α > 1.9° 而在 2000 年以前时返回 `None`。
pub fn planet_magnitude_v(planet: MagnitudePlanet, g: &MagnitudeGeometry, saturn_rings: bool) -> Option<Real> {
    let a = g.phase_deg;
    let dist = real(5.0) * (g.r_au * g.delta_au).log10();
    let v = match planet {
        // 式 2
        MagnitudePlanet::Mercury => poly(a, &[-0.613, 6.3280e-02, -1.6336e-03, 3.3644e-05, -3.4265e-07, 1.6893e-09, -3.0334e-12]),
        // 式 3（α < 163.7°）、式 4
        MagnitudePlanet::Venus => {
            if a < real(163.7) {
                poly(a, &[-4.384, -1.044e-03, 3.687e-04, -2.814e-06, 8.938e-09])
            } else {
                poly(a, &[236.05828, -2.81914, 8.39034e-03])
            }
        }
        // 式 5
        MagnitudePlanet::Earth => poly(a, &[-3.99, -1.060e-3, 2.054e-4]),
        // 式 6（α ≤ 50°）、式 7；L(λe) 有直下点经度时才计
        MagnitudePlanet::Mars => {
            let base = if a <= real(50.0) {
                poly(a, &[-1.601, 0.02267, -0.0001302])
            } else {
                poly(a, &[-0.367, -0.02573, 0.0003445])
            };
            let rot = g
                .mars_sub_longitudes_deg
                .map(|(o, s)| mars_rotation_correction(mars_effective_central_meridian_deg(o, s)))
                .unwrap_or(real(0.0));
            base + rot + mars_orbital_correction(g.helio_ecl_long_deg)
        }
        // 式 8（α ≤ 12°）、式 9（Mayorga et al. 2016 绿光反照率多项式）
        MagnitudePlanet::Jupiter => {
            if a <= real(12.0) {
                poly(a, &[-9.395, -3.7e-04, 6.16e-04])
            } else {
                let x = a / real(180.0);
                real(-9.428) - real(2.5) * poly(x, &[1.0, -1.507, -0.363, -0.062, 2.809, -1.876]).log10()
            }
        }
        MagnitudePlanet::Saturn => {
            // 有效倾角 β = √(βE βS)（同号），异号（环背光）时 β = 0
            let prod = g.sun_sub_lat_deg * g.observer_sub_lat_deg;
            let beta = if prod >= real(0.0) { prod.sqrt() } else { real(0.0) };
            let geocentric = a <= real(6.5) && beta <= real(27.0);
            if saturn_rings && geocentric {
                // 式 10
                let sb = deg_to_rad(beta).sin();
                real(-8.914) - real(1.825) * sb + real(0.026) * a - real(0.378) * sb * (real(-2.25) * a).exp()
            } else if !saturn_rings && geocentric {
                // 式 11
                poly(a, &[-8.95, -3.7e-04, 6.16e-04])
            } else if !saturn_rings && a > real(6.5) {
                // 式 12（三次项取作者程序的 −1.506e-6）
                poly(a, &[-8.94, 2.446e-4, 2.672e-4, -1.506e-6, 4.767e-9])
            } else {
                return None;
            }
        }
        MagnitudePlanet::Uranus => {
            // 式 14（α ≤ 3.1°）、式 15；φ′ 为太阳与观测者直下点行星面图纬度绝对值之均
            let pg = |x: Real| planetographic_latitude_deg(x, URANUS_FLATTENING).abs();
            let phi = (pg(g.sun_sub_lat_deg) + pg(g.observer_sub_lat_deg)) / real(2.0);
            let base = real(-7.110) - real(8.4e-04) * phi;
            if a <= real(3.1) {
                base
            } else {
                base + real(6.587e-3) * a + real(1.045e-4) * a * a
            }
        }
        MagnitudePlanet::Neptune => {
            // 式 16：V₁(0) 随年代（1980 年前 −6.89，1980—2000 线性变亮，2000 年后 −7.00）
            let y = g.year;
            let v0 = if y < real(1980.0) {
                real(-6.89)
            } else if y <= real(2000.0) {
                real(-6.89) - real(0.0054) * (y - real(1980.0))
            } else {
                real(-7.00)
            };
            if a <= real(1.9) {
                v0
            } else if y > real(2000.0) {
                // 式 17，只适用于 2000 年后
                v0 + real(7.944e-3) * a + real(9.617e-5) * a * a
            } else {
                return None;
            }
        }
    };
    Some(v + dist)
}

/// 自转轴北极（ICRS，J2000）：IAU WGCCRE（Archinal et al. 2018）α₀、δ₀ 的 J2000 值。
fn pole_icrs(planet: Planet) -> Option<[f64; 3]> {
    let (ra, dec): (f64, f64) = match planet {
        Planet::Saturn => (40.589, 83.537),
        Planet::Uranus => (257.311, -15.175),
        _ => return None,
    };
    let (ra, dec) = (ra.to_radians(), dec.to_radians());
    Some([dec.cos() * ra.cos(), dec.cos() * ra.sin(), dec.sin()])
}

fn norm(v: [Real; 3]) -> Real {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn angle_deg(a: [Real; 3], b: [Real; 3]) -> Real {
    let cr = [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    rad_to_deg(norm(cr).atan2(dot))
}

/// 由历表算地心观测的星等几何量：行星取光行时修正后的地心位置（ICRS），太阳取地心几何位置（太阳在光行时 τ 内的移动
/// 对 r、α 的影响可忽略）。火星直下点经度未计（[`MagnitudeGeometry::mars_sub_longitudes_deg`] 为 None）。
pub fn planet_magnitude_geometry<P: EphemerisProvider>(eph: &P, planet: Planet, t: TimePoint) -> MagnitudeGeometry {
    let t_tt = t.to_scale(TimeScale::TT);
    let graph = TransformGraph::default_graph()
        .with_fk5_to_icrs_mapper(|s, jd_tt| Fk5ToIcrsBias.apply(s, TimePoint::new(TimeScale::TT, jd_tt)));
    let to_icrs_au = |state: crate::astronomy::pipeline::State6, jd: Real| -> [Real; 3] {
        let m = graph.transform_to(state, ReferenceFrame::ICRS, jd).position.to_meters();
        let au = crate::astronomy::constant::AU_METERS;
        [m[0] / au, m[1] / au, m[2] / au]
    };
    let corrector: LightTimeCorrector<'_, P, Fk5ToIcrsBias> = LightTimeCorrector { ephemeris: eph, mapper: None, max_iter: 3 };
    let (tr, pstate) = corrector.retarded_state(t_tt, Body::Planet(planet));
    // 观测者 → 行星
    let obs_to_planet = to_icrs_au(pstate, tr.to_scale(TimeScale::TT).jd);
    let sun_state = eph.compute_state(Body::Sun, t_tt.to_scale(eph.evaluation_time_scale()));
    let obs_to_sun = to_icrs_au(sun_state, t_tt.jd);
    let sun_to_planet = [obs_to_planet[0] - obs_to_sun[0], obs_to_planet[1] - obs_to_sun[1], obs_to_planet[2] - obs_to_sun[2]];
    let (sun_lat, obs_lat) = match pole_icrs(planet) {
        Some(p) => {
            let p = [real(p[0]), real(p[1]), real(p[2])];
            // 直下点面心纬度 = 90° − ∠(极, 行星→X) = ∠(极, X→行星) − 90°
            (angle_deg(p, sun_to_planet) - real(90.0), angle_deg(p, obs_to_planet) - real(90.0))
        }
        None => (real(0.0), real(0.0)),
    };
    // 日心黄经（J2000 黄道）：ICRS → FK5（偏差 0.02″ 量级，对 10° 间隔的季节修正表无影响，略去）→ R_x(ε₀)
    let eps0 = crate::astronomy::frame::precession::mean_obliquity(0.0).rad();
    let (ce, se) = (eps0.cos(), eps0.sin());
    let y_ecl = sun_to_planet[1] * ce + sun_to_planet[2] * se;
    let helio_long = rad_to_deg(y_ecl.atan2(sun_to_planet[0]).wrap_to_2pi());
    MagnitudeGeometry {
        r_au: norm(sun_to_planet),
        delta_au: norm(obs_to_planet),
        phase_deg: angle_deg(sun_to_planet, obs_to_planet),
        sun_sub_lat_deg: sun_lat,
        observer_sub_lat_deg: obs_lat,
        year: real(2000.0) + (t_tt.jd - real(2451545.0)) / real(365.25),
        helio_ecl_long_deg: helio_long,
        mars_sub_longitudes_deg: None,
    }
}

/// 行星地心视星等（土星含环）。不给星等的情形见 [`planet_magnitude_v`]。
pub fn planet_magnitude<P: EphemerisProvider>(eph: &P, planet: Planet, t: TimePoint) -> Option<Real> {
    planet_magnitude_v(planet.into(), &planet_magnitude_geometry(eph, planet, t), true)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 土星扁率换算：作者程序把输入的行星面图纬度按 b²/a² = 0.8137 换为面心纬度（`Saturn_Geodetic_Geocentric`）。
    const SATURN_B2_A2: f64 = 0.8137;

    fn month(s: &str) -> f64 {
        ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"].iter().position(|m| *m == s).unwrap() as f64
    }

    /// 「2018-Feb-21」→ 儒略年（近似，海王星式 16 的年代分段用，精度足够）。
    fn year_of(date: &str) -> f64 {
        let p: Vec<&str> = date.split('-').collect();
        p[0].parse::<f64>().unwrap() + (month(p[1]) * 30.44 + p[2].parse::<f64>().unwrap()) / 365.25
    }

    /// 作者程序 Ap_Mag V3 官方测试（`vendor/planetary_magnitudes/Ap_Mag_Current_Version/Ap_Mag_Input_V3.txt`，HORIZONS 几何量 +
    /// 预先算好的星等）。附录：与预算星等之差 ≤ 0.001 即实现通过。逐行按各行星的列解析；土星末例预算星等为空（含环超出范围），
    /// 应返回 None。
    #[test]
    fn matches_ap_mag_v3_official_test_data() {
        let path = crate::repo::repo_root().join("vendor/planetary_magnitudes/Ap_Mag_Current_Version/Ap_Mag_Input_V3.txt");
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipped: 无 {path:?}");
            return;
        };
        let mut planet: Option<MagnitudePlanet> = None;
        let (mut n, mut failures) = (0, vec![]);
        let mut tab = crate::test_util::CheckTable::new("ap_mag_v3_official", &["行星", "日期", "相角°", "官方星等", "本实现", "差"]);
        tab.meta("Mallama & Hilton 2018 作者程序 Ap_Mag V3 官方测试数据（vendor/planetary_magnitudes/Ap_Mag_Current_Version/Ap_Mag_Input_V3.txt）逐例");
        for line in text.lines().map(|l| l.trim_end_matches('\r')) {
            match line.trim() {
                "MERCURY" => planet = Some(MagnitudePlanet::Mercury),
                "VENUS" => planet = Some(MagnitudePlanet::Venus),
                "EARTH" => planet = Some(MagnitudePlanet::Earth),
                "MARS" => planet = Some(MagnitudePlanet::Mars),
                "JUPITER" => planet = Some(MagnitudePlanet::Jupiter),
                "SATURN" => planet = Some(MagnitudePlanet::Saturn),
                "URANUS" => planet = Some(MagnitudePlanet::Uranus),
                "NEPTUNE" => planet = Some(MagnitudePlanet::Neptune),
                _ => {}
            }
            let Some(p) = planet else { continue };
            let mut t: Vec<&str> = line.split_whitespace().collect();
            let rings = matches!(t.first(), Some(&"T"));
            if matches!(t.first(), Some(&"T") | Some(&"F")) {
                t.remove(0);
            }
            if t.len() < 8 || !t[0].chars().next().unwrap().is_ascii_digit() || !t[0].contains('-') {
                continue;
            }
            let f = |i: usize| t[i].parse::<f64>().unwrap();
            let base = |r: f64, d: f64, a: f64| MagnitudeGeometry {
                r_au: real(r),
                delta_au: real(d),
                phase_deg: real(a),
                sun_sub_lat_deg: real(0.0),
                observer_sub_lat_deg: real(0.0),
                year: real(year_of(t[0])),
                helio_ecl_long_deg: real(0.0),
                mars_sub_longitudes_deg: None,
            };
            let to_pc = |pg: f64, b2a2: f64| (pg.to_radians().tan() * b2a2).atan().to_degrees();
            let (geo, expect) = match p {
                MagnitudePlanet::Mercury | MagnitudePlanet::Earth | MagnitudePlanet::Jupiter => (base(f(4), f(6), f(8)), t.get(9)),
                MagnitudePlanet::Venus => (base(f(4), f(6), f(10)), t.get(11)),
                MagnitudePlanet::Mars => (
                    MagnitudeGeometry {
                        helio_ecl_long_deg: real(f(8)),
                        mars_sub_longitudes_deg: Some((real(f(4)), real(f(6)))),
                        ..base(f(10), f(12), f(16))
                    },
                    t.get(17),
                ),
                MagnitudePlanet::Saturn => (
                    MagnitudeGeometry {
                        observer_sub_lat_deg: real(to_pc(f(5), SATURN_B2_A2)),
                        sun_sub_lat_deg: real(to_pc(f(7), SATURN_B2_A2)),
                        ..base(f(8), f(10), f(14))
                    },
                    t.get(15),
                ),
                MagnitudePlanet::Uranus | MagnitudePlanet::Neptune => {
                    // 输入为行星面图纬度；函数取面心纬度（内部按式 13 换回），天王星先换为面心
                    let k = (1.0 - URANUS_FLATTENING).powi(2);
                    (
                        MagnitudeGeometry {
                            observer_sub_lat_deg: real(to_pc(f(5), k)),
                            sun_sub_lat_deg: real(to_pc(f(7), k)),
                            ..base(f(8), f(10), f(12))
                        },
                        t.get(13),
                    )
                }
            };
            n += 1;
            let ours = planet_magnitude_v(p, &geo, rings).map(|v| v.as_f64());
            match (ours, expect.map(|s| s.parse::<f64>().unwrap())) {
                (Some(v), Some(e)) => {
                    println!("{p:?} {} α={:.2}: 本实现 {v:.4}，官方 {e}", t[0], geo.phase_deg.as_f64());
                    tab.row(vec![format!("{p:?}"), t[0].to_string(), format!("{:.2}", geo.phase_deg.as_f64()), format!("{e}"), format!("{v:.4}"), format!("{:+.4}", v - e)]);
                    if (v - e).abs() > 0.001 {
                        failures.push(format!("{p:?} {}: {v:.4} vs {e}", t[0]));
                    }
                }
                (None, None) => {
                    println!("{p:?} {}: 不给星等（与官方一致）", t[0]);
                    tab.row(vec![format!("{p:?}"), t[0].to_string(), format!("{:.2}", geo.phase_deg.as_f64()), "（不给）".into(), "（不给）".into(), "一致".into()]);
                }
                (o, e) => failures.push(format!("{p:?} {}: {o:?} vs {e:?}", t[0])),
            }
        }
        tab.write();
        assert_eq!(n, 27, "官方测试应有 27 例，解析到 {n}");
        assert!(failures.is_empty(), "{failures:?}");
    }

    /// 论文正文式 12 三次项 −1.505e-6 与 Ap_Mag V3 官方输出（5.206）不符，作者程序为 −1.506e-6。
    #[test]
    fn saturn_eq12_cubic_coefficient_follows_author_code() {
        let (r, d, a) = (9.026035315474_f64, 10.1321497654765_f64, 169.8958_f64);
        let with = |c3: f64| 5.0 * (r * d).log10() - 8.94 + 2.446e-4 * a + 2.672e-4 * a * a - c3 * a.powi(3) + 4.767e-9 * a.powi(4);
        assert!((with(1.506e-6) - 5.206).abs() < 0.0005);
        assert!((with(1.505e-6) - 5.206).abs() > 0.004);
    }

    /// Stirling 插值在表格节点处取表值。
    #[test]
    fn mars_stirling_hits_table_nodes() {
        for i in 0..36 {
            let v = mars_rotation_correction(real(10.0 * i as f64)).as_f64();
            assert!((v - MARS_L1[i + 2]).abs() < 1e-12, "L1 {i}: {v}");
            let v = stirling_10deg(&MARS_L2, real(10.0 * i as f64)).as_f64();
            assert!((v - MARS_L2[i + 2]).abs() < 1e-12, "L2 {i}: {v}");
        }
    }

    /// Real 的 ln、exp、log10：TwoFloat 下须优于 f64（twofloat 0.8 原生 ln、负自变量 exp 误差约 2e-12，见 math::real）。
    #[test]
    fn real_log_exp() {
        assert!((real(10.0).log10() - real(1.0)).abs().as_f64() < 1e-15);
        assert!((real(1.0).exp().ln() - real(1.0)).abs().as_f64() < 1e-15);
        assert!((real(-2.302585092994046).exp() * real(10.0) - real(1.0)).abs().as_f64() < 1e-15);
        assert!((real(2.0).ln().as_f64() - core::f64::consts::LN_2).abs() < 1e-15);
    }
}
