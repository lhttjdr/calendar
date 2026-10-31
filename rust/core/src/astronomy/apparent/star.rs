//! 恒星地心视位置：星表方向（含空间运动）→ 周年视差 → 周年光行差（一阶）→ 当日真赤道（岁差章动，同行星管线）。
//!
//! 未计太阳引力偏折（离日 10° 处约 0.05″、90° 处 0.004″）。地球位置、速度取「地心太阳的反向」，即以日心代质心，
//! 速度差约 0.04%（光行差 20.5″ 中约 0.01″），位置差约 0.005 AU（视差中可忽略）。

use crate::astronomy::constant::{light_speed_au_per_day, AU_METERS};
use crate::astronomy::pipeline::{Body, EphemerisProvider, Fk5ToIcrsBias, FrameMapper, State6, TransformGraph};
use crate::astronomy::stars::CatalogStar;
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, Real};
use crate::quantity::epoch::Epoch;
use crate::quantity::position::Position;
use crate::quantity::reference_frame::ReferenceFrame;
use crate::quantity::velocity::Velocity;

use super::ApparentPipelineOptions;

fn star_graph(options: &ApparentPipelineOptions) -> TransformGraph {
    TransformGraph::default_graph()
        .with_fk5_to_icrs_mapper(|s, jd_tt| Fk5ToIcrsBias.apply(s, TimePoint::new(TimeScale::TT, jd_tt)))
        .with_precession_model(options.effective_precession_model())
}

/// 恒星地心视方向（当日真赤道架；矢量长度取 1 AU，仅方向有意义）。
pub fn star_apparent_equatorial<P: EphemerisProvider>(eph: &P, star: &CatalogStar, t: TimePoint, options: &ApparentPipelineOptions) -> Position {
    let t_tt = t.to_scale(TimeScale::TT);
    let jd = t_tt.jd;
    let graph = star_graph(options);
    // 地球位置、速度（ICRS）= −地心太阳位置、速度（日心代质心）
    let sun = graph.transform_to(eph.compute_state(Body::Sun, t_tt.to_scale(eph.evaluation_time_scale())), ReferenceFrame::ICRS, jd);
    let (p_sun, v_sun) = (sun.position.to_meters(), sun.velocity.to_m_per_s());
    let c = light_speed_au_per_day() * AU_METERS / real(86400.0);
    let u = star.icrs_direction(jd);
    // 周年视差：星方向（以 AU 计距离 1/ϖ）减去地球日心位置，u − ϖ·r_E/AU，ϖ 为弧度
    let plx = star.parallax_mas * real(core::f64::consts::PI / (180.0 * 3600.0 * 1000.0));
    let u = [u[0] + plx * p_sun[0] / AU_METERS, u[1] + plx * p_sun[1] / AU_METERS, u[2] + plx * p_sun[2] / AU_METERS];
    // 一阶光行差：u' ∝ u + v/c（v = 地球速度），误差 ~ (v/c)² ≈ 1e-8 rad
    let w = [u[0] - v_sun[0] / c, u[1] - v_sun[1] / c, u[2] - v_sun[2] / c];
    let n = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
    let au = AU_METERS;
    let pos = Position::from_si_meters_in_frame(ReferenceFrame::ICRS, w[0] / n * au, w[1] / n * au, w[2] / n * au);
    let vel = Velocity::from_si_m_per_s_in_frame(ReferenceFrame::ICRS, real(0.0), real(0.0), real(0.0));
    graph.transform_to(State6::new(pos, vel), ReferenceFrame::TrueEquator(Epoch::new(jd)), jd).position
}


/// 一批恒星的地心视方向（当日真赤道架；矢量长度取 1 AU）：与逐颗调用 [`star_apparent_equatorial`] 等价，但地球状态与
/// 「ICRS → 当日真赤道」变换只算一次。该变换由偏差、岁差、章动三个旋转组成，取三个基矢量的像（减去原点之像）为矩阵列。
/// 星图要一次算近万颗星，逐颗走管线会重复构造岁差章动矩阵。
pub fn stars_apparent_equatorial<P: EphemerisProvider>(
    eph: &P,
    stars: &[CatalogStar],
    t: TimePoint,
    options: &ApparentPipelineOptions,
) -> Vec<Position> {
    let t_tt = t.to_scale(TimeScale::TT);
    let jd = t_tt.jd;
    let graph = star_graph(options);
    let sun = graph.transform_to(eph.compute_state(Body::Sun, t_tt.to_scale(eph.evaluation_time_scale())), ReferenceFrame::ICRS, jd);
    let (p_sun, v_sun) = (sun.position.to_meters(), sun.velocity.to_m_per_s());
    let c = light_speed_au_per_day() * AU_METERS / real(86400.0);
    let au = AU_METERS;
    let zero_vel = || Velocity::from_si_m_per_s_in_frame(ReferenceFrame::ICRS, real(0.0), real(0.0), real(0.0));
    let image = |v: [Real; 3]| {
        let pos = Position::from_si_meters_in_frame(ReferenceFrame::ICRS, v[0] * au, v[1] * au, v[2] * au);
        graph.transform_to(State6::new(pos, zero_vel()), ReferenceFrame::TrueEquator(Epoch::new(jd)), jd).position.to_meters()
    };
    let o = image([real(0.0), real(0.0), real(0.0)]);
    let cols: [[Real; 3]; 3] = core::array::from_fn(|k| {
        let mut e = [real(0.0); 3];
        e[k] = real(1.0);
        let m = image(e);
        [(m[0] - o[0]) / au, (m[1] - o[1]) / au, (m[2] - o[2]) / au]
    });
    let frame = ReferenceFrame::TrueEquator(Epoch::new(jd));
    let plx_unit = real(core::f64::consts::PI / (180.0 * 3600.0 * 1000.0));
    stars
        .iter()
        .map(|star| {
            let u = star.icrs_direction(jd);
            let plx = star.parallax_mas * plx_unit;
            let u = [u[0] + plx * p_sun[0] / AU_METERS, u[1] + plx * p_sun[1] / AU_METERS, u[2] + plx * p_sun[2] / AU_METERS];
            let w = [u[0] - v_sun[0] / c, u[1] - v_sun[1] / c, u[2] - v_sun[2] / c];
            let n = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
            let w = [w[0] / n, w[1] / n, w[2] / n];
            let r: [Real; 3] = core::array::from_fn(|i| cols[0][i] * w[0] + cols[1][i] * w[1] + cols[2][i] * w[2]);
            Position::from_si_meters_in_frame(frame, o[0] + r[0] * au, o[1] + r[1] * au, o[2] + r[2] * au)
        })
        .collect()
}

/// 恒星平位置：星表方向（含空间运动）→ 当日平赤道与平春分点（只计岁差，不计章动、光行差与视差）。
/// 古代星表与近代星表校算多用平位置（如潘鼐对《仪象考成》1744.0 的「今测计算值」）。矢量长度取 1 AU，仅方向有意义。
pub fn star_mean_equatorial(star: &CatalogStar, t: TimePoint, options: &ApparentPipelineOptions) -> Position {
    let jd = t.to_scale(TimeScale::TT).jd;
    let graph = TransformGraph::default_graph()
        .with_fk5_to_icrs_mapper(|s, jd_tt| Fk5ToIcrsBias.apply(s, TimePoint::new(TimeScale::TT, jd_tt)))
        .with_precession_model(options.effective_precession_model());
    let u = star.icrs_direction(jd);
    let au = AU_METERS;
    let pos = Position::from_si_meters_in_frame(ReferenceFrame::ICRS, u[0] * au, u[1] * au, u[2] * au);
    let vel = Velocity::from_si_m_per_s_in_frame(ReferenceFrame::ICRS, real(0.0), real(0.0), real(0.0));
    graph.transform_to(State6::new(pos, vel), ReferenceFrame::MeanEquator(Epoch::new(jd)), jd).position
}

/// 一批恒星的平位置（当日平赤道与平春分点；矢量长度取 1 AU）：与逐颗调用 [`star_mean_equatorial`] 等价，「ICRS → 当日平赤道」
/// 变换只算一次（偏差、岁差两个旋转，取基矢量之像为矩阵列）。古星表历元拟合要在许多历元上算上百颗星。
pub fn stars_mean_equatorial(stars: &[CatalogStar], t: TimePoint, options: &ApparentPipelineOptions) -> Vec<Position> {
    let jd = t.to_scale(TimeScale::TT).jd;
    let graph = star_graph(options);
    let au = AU_METERS;
    let frame = ReferenceFrame::MeanEquator(Epoch::new(jd));
    let image = |v: [Real; 3]| {
        let pos = Position::from_si_meters_in_frame(ReferenceFrame::ICRS, v[0] * au, v[1] * au, v[2] * au);
        let vel = Velocity::from_si_m_per_s_in_frame(ReferenceFrame::ICRS, real(0.0), real(0.0), real(0.0));
        graph.transform_to(State6::new(pos, vel), frame, jd).position.to_meters()
    };
    let o = image([real(0.0), real(0.0), real(0.0)]);
    let cols: [[Real; 3]; 3] = core::array::from_fn(|k| {
        let mut e = [real(0.0); 3];
        e[k] = real(1.0);
        let m = image(e);
        [(m[0] - o[0]) / au, (m[1] - o[1]) / au, (m[2] - o[2]) / au]
    });
    stars
        .iter()
        .map(|star| {
            let u = star.icrs_direction(jd);
            let r: [Real; 3] = core::array::from_fn(|i| cols[0][i] * u[0] + cols[1][i] * u[1] + cols[2][i] * u[2]);
            Position::from_si_meters_in_frame(frame, o[0] + r[0] * au, o[1] + r[1] * au, o[2] + r[2] * au)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::ephemeris::load_earth_vsop87_corrected_from_repo;
    use crate::astronomy::ephemeris::vsop87_patch::PatchTarget;
    use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
    use crate::astronomy::pipeline::Vsop87Planets;
    use crate::astronomy::stars::load_hipparcos_bright_from_repo;
    use crate::math::real::RealOps;

    /// Skyfield（DE406，IAU 2006 岁差 + 2000A 章动，`scripts/phenomena/skyfield_star_apparent.py`）的恒星视位置
    /// （当日真赤道）：(HIP, jd TT, ra rad, dec rad)。天狼星、大角、牛宿一（β Cap）、轩辕十四、δ Aur；
    /// 1985、2000、2025、−500、−900 年。
    const SKYFIELD_STARS: &[(u32, f64, f64, f64)] = &[
        (32349, 2446145.5, 1.7649184526810235, -0.2914126024599986),
        (32349, 2451545.0, 1.7678471781946739, -0.2917806948950408),
        (32349, 2460676.5, 1.772703380803702, -0.29234792693256045),
        (32349, 2268923.5, 1.6716581549732825, -0.2815391913630001),
        (32349, 1757585.0, 1.402322978394678, -0.2775595186472427),
        (69673, 2446145.5, 3.730619578664108, 0.33608163453778317),
        (69673, 2451545.0, 3.733417964556171, 0.3347969585478417),
        (69673, 2460676.5, 3.7384746952516914, 0.3324805086512692),
        (69673, 2268923.5, 3.6340661242707384, 0.38122771978792147),
        (69673, 1757585.0, 3.3547809254105347, 0.5221887570358197),
        (100345, 2446145.5, 5.323933959957875, -0.25887077481248116),
        (100345, 2451545.0, 5.327500675245552, -0.25799462003564333),
        (100345, 2460676.5, 5.333697029614506, -0.2566264614234629),
        (100345, 2268923.5, 5.204243402950171, -0.2835988921449608),
        (100345, 1757585.0, 4.851898637385805, -0.32586086618101506),
        (49669, 2446145.5, 2.651121363265958, 0.21014193894483318),
        (49669, 2451545.0, 2.65451596505418, 0.20885416311372532),
        (49669, 2460676.5, 2.660392743962722, 0.2067129291367941),
        (49669, 2268923.5, 2.5375910078523725, 0.25042167180485475),
        (49669, 1757585.0, 2.1987400946833966, 0.3479354210906351),
        (28358, 2446145.5, 1.5633279880008213, 0.9475312018232789),
        (28358, 2451545.0, 1.5688041451880428, 0.9474275584263664),
        (28358, 2460676.5, 1.577891270870065, 0.9474744094119798),
        (28358, 2268923.5, 1.389708377503389, 0.9433355897699792),
        (28358, 1757585.0, 0.9149036929837475, 0.8884610263125922),
    ];

    /// 与 Skyfield 的角距：本实现未计引力偏折、以日心代质心，空间运动同为视向速度 0 的直线运动；岁差同取 P03（Skyfield 所用）。
    /// 实测差 0.002″（1985）至 0.08″（−900），主要来自引力偏折（离日 25° 时约 0.02″）与日心/质心速度之差（约 0.01″）。
    #[test]
    fn star_apparent_vs_skyfield() {
        let _ = try_init_full_nutation_from_repo();
        let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
        let eph = Vsop87Planets { earth: &earth, planets: vec![] };
        let cat = load_hipparcos_bright_from_repo().unwrap();
        let opts = ApparentPipelineOptions { use_p03_precession: true, ..Default::default() };
        let mut worst: f64 = 0.0;
        let mut tab = crate::test_util::CheckTable::new("star_apparent_vs_skyfield", &["HIP", "历元(TT)", "Skyfield RA°", "Skyfield Dec°", "角距差″"]);
        tab.meta("恒星地心视位置（当日真赤道）对 Skyfield（DE406，P03，scripts/phenomena/skyfield_star_apparent.py）逐点对照");
        for &(hip, jd, ra, dec) in SKYFIELD_STARS {
            let p = star_apparent_equatorial(&eph, cat.by_hip(hip).unwrap(), TimePoint::new(TimeScale::TT, real(jd)), &opts).to_meters();
            let n = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            let u = [p[0] / n, p[1] / n, p[2] / n];
            let v = [dec.cos() * ra.cos(), dec.cos() * ra.sin(), dec.sin()];
            let cr = [u[1].as_f64() * v[2] - u[2].as_f64() * v[1], u[2].as_f64() * v[0] - u[0].as_f64() * v[2], u[0].as_f64() * v[1] - u[1].as_f64() * v[0]];
            let sep = (cr[0] * cr[0] + cr[1] * cr[1] + cr[2] * cr[2]).sqrt() * 206264.806;
            worst = worst.max(sep);
            tab.row(vec![hip.to_string(), crate::test_util::jd_to_calendar(jd), format!("{:.6}", ra.to_degrees()), format!("{:.6}", dec.to_degrees()), format!("{sep:.3}")]);
            assert!(sep < 0.1, "HIP {hip} jd {jd}: {sep:.3}″");
        }
        tab.write();
        println!("恒星视位置对 Skyfield 最大差 {worst:.3}″");
    }

    /// 批量与逐颗结果一致（前 500 年、2000 年各取 40 颗星）。
    #[test]
    fn batch_matches_single() {
        let _ = try_init_full_nutation_from_repo();
        let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
        let eph = Vsop87Planets { earth: &earth, planets: vec![] };
        let cat = load_hipparcos_bright_from_repo().unwrap();
        let opts = ApparentPipelineOptions::default();
        let stars: Vec<CatalogStar> = cat.stars.iter().step_by(cat.stars.len() / 40).copied().collect();
        for jd in [1538432.5, 2451545.0] {
            let t = TimePoint::new(TimeScale::TT, real(jd));
            let batch = stars_apparent_equatorial(&eph, &stars, t, &opts);
            let mut worst: f64 = 0.0;
            for (s, b) in stars.iter().zip(&batch) {
                let a = star_apparent_equatorial(&eph, s, t, &opts).to_meters();
                let b = b.to_meters();
                let na = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
                let nb = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
                let d: f64 = (0..3).map(|i| ((a[i] / na - b[i] / nb).as_f64()).powi(2)).sum::<f64>().sqrt();
                worst = worst.max(d);
            }
            println!("jd {jd}: 批量与逐颗最大差 {:.2e}″", worst * 206264.806);
            assert!(worst < 1e-12, "jd {jd}: {worst}");
        }
    }

    /// 平位置批量与逐颗一致。
    #[test]
    fn mean_batch_matches_single() {
        let cat = load_hipparcos_bright_from_repo().unwrap();
        let opts = ApparentPipelineOptions::default();
        let stars: Vec<CatalogStar> = cat.stars.iter().step_by(cat.stars.len() / 40).copied().collect();
        for jd in [1538432.5, 2451545.0] {
            let t = TimePoint::new(TimeScale::TT, real(jd));
            let batch = stars_mean_equatorial(&stars, t, &opts);
            let mut worst: f64 = 0.0;
            for (s, b) in stars.iter().zip(&batch) {
                let a = star_mean_equatorial(s, t, &opts).to_meters();
                let b = b.to_meters();
                let na = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
                let nb = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
                let d: f64 = (0..3).map(|i| ((a[i] / na - b[i] / nb).as_f64()).powi(2)).sum::<f64>().sqrt();
                worst = worst.max(d);
            }
            assert!(worst < 1e-12, "jd {jd}: {worst}");
        }
    }
}
