//! 恒星地心视位置：星表方向（含空间运动）→ 周年视差 → 太阳引力光线偏折 → 周年光行差（一阶）→ 当日真赤道（岁差章动，同行星管线）。
//!
//! 太阳引力偏折按 [`ApparentPipelineOptions::light_deflection`]（缺省开，SOFA `iauLdsun` 同式；离日 1° 处 0.47″、10° 处 0.047″、
//! 90° 处 0.004″）。地球位置、速度取「地心太阳的反向」，即以日心代质心，速度差约 0.04%（光行差 20.5″ 中约 0.01″），
//! 位置差约 0.005 AU（视差中可忽略；偏折中 e 的方向差约 0.3°，相对影响 < 10⁻²）。

use crate::astronomy::constant::AU_METERS;
use crate::astronomy::correction::aberration::annual_aberration_direction;
use crate::astronomy::pipeline::{Body, EphemerisProvider, Fk5ToIcrsBias, FrameMapper, State6, TransformGraph};
use crate::astronomy::stars::CatalogStar;
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, Real};
use crate::quantity::epoch::Epoch;
use crate::quantity::position::Position;
use crate::quantity::reference_frame::ReferenceFrame;
use crate::quantity::velocity::Velocity;

use super::{ApparentPipelineOptions, LightDeflection};
use crate::astronomy::correction::light_deflection::{solar_deflection_star, unit};

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
    let u = deflected_direction(direction_with_parallax(star, jd, p_sun), p_sun, options);
    let au = AU_METERS;
    let pos = Position::from_si_meters_in_frame(ReferenceFrame::ICRS, u[0] * au, u[1] * au, u[2] * au);
    let vel = Velocity::from_si_m_per_s_in_frame(ReferenceFrame::ICRS, real(0.0), real(0.0), real(0.0));
    let epoch = Epoch::new(jd);
    let earth_velocity = [-v_sun[0], -v_sun[1], -v_sun[2]];
    let [x, y, z] = graph.transform_with_earth_velocity(State6::new(pos, vel), ReferenceFrame::ApparentEquator(epoch), earth_velocity, jd).position.to_meters();
    // 轴向即当日真赤道，后续站心平移按 TrueEquator 接
    Position::from_si_meters_in_frame(ReferenceFrame::TrueEquator(epoch), x, y, z)
}

/// 星表方向（含空间运动）加周年视差后的单位矢量：u − ϖ·r_E/AU，ϖ 为弧度；地心太阳位置即 −r_E。
fn direction_with_parallax(star: &CatalogStar, jd: Real, p_sun: [Real; 3]) -> [Real; 3] {
    let u = star.icrs_direction(jd);
    let plx = star.parallax_mas * real(core::f64::consts::PI / (180.0 * 3600.0 * 1000.0));
    let w = [u[0] + plx * p_sun[0] / AU_METERS, u[1] + plx * p_sun[1] / AU_METERS, u[2] + plx * p_sun[2] / AU_METERS];
    let n = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
    [w[0] / n, w[1] / n, w[2] / n]
}

/// 按选项施太阳引力光线偏折：u 为（含视差的）星方向单位矢量，`p_sun` 为地心太阳（ICRS，米）；e = −p_sun 方向。
fn deflected_direction(u: [Real; 3], p_sun: [Real; 3], options: &ApparentPipelineOptions) -> [Real; 3] {
    match options.light_deflection {
        LightDeflection::None => u,
        LightDeflection::Sun => {
            let e = [-p_sun[0], -p_sun[1], -p_sun[2]];
            let em = (e[0] * e[0] + e[1] * e[1] + e[2] * e[2]).sqrt() / AU_METERS;
            unit(solar_deflection_star(u, unit(e), em))
        }
    }
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
    let au = AU_METERS;
    let earth_velocity_au_day: [Real; 3] = core::array::from_fn(|i| -v_sun[i] * real(86400.0) / AU_METERS);
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
    stars
        .iter()
        .map(|star| {
            let w = annual_aberration_direction(deflected_direction(direction_with_parallax(star, jd, p_sun), p_sun, options), earth_velocity_au_day);
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

    fn sep_to(p: Position, ra: f64, dec: f64) -> f64 {
        let p = p.to_meters();
        let n = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        let u = [p[0] / n, p[1] / n, p[2] / n];
        let v = [dec.cos() * ra.cos(), dec.cos() * ra.sin(), dec.sin()];
        let cr = [u[1].as_f64() * v[2] - u[2].as_f64() * v[1], u[2].as_f64() * v[0] - u[0].as_f64() * v[2], u[0].as_f64() * v[1] - u[1].as_f64() * v[0]];
        (cr[0] * cr[0] + cr[1] * cr[1] + cr[2] * cr[2]).sqrt() * 206264.806
    }

    /// 与 Skyfield 的角距：两边都计太阳引力偏折（本实现缺省开）；本实现以日心代质心，空间运动同为视向速度 0 的直线运动；
    /// 岁差同取 P03（Skyfield 所用）。另列不计偏折的差。
    #[test]
    fn star_apparent_vs_skyfield() {
        let _ = try_init_full_nutation_from_repo();
        let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
        let eph = Vsop87Planets { earth: &earth, planets: vec![] };
        let cat = load_hipparcos_bright_from_repo().unwrap();
        let opts = ApparentPipelineOptions { use_p03_precession: true, ..Default::default() };
        let off = ApparentPipelineOptions { light_deflection: LightDeflection::None, ..opts.clone() };
        let mut worst: f64 = 0.0;
        let mut tab = crate::test_util::CheckTable::new("star_apparent_vs_skyfield", &["HIP", "历元(TT)", "Skyfield RA°", "Skyfield Dec°", "角距差″", "不计偏折差″"]);
        tab.meta("恒星地心视位置（当日真赤道）对 Skyfield（DE406，P03，scripts/phenomena/skyfield_star_apparent.py）逐点对照；本实现缺省计太阳引力偏折");
        for &(hip, jd, ra, dec) in SKYFIELD_STARS {
            let t = TimePoint::new(TimeScale::TT, real(jd));
            let sep = sep_to(star_apparent_equatorial(&eph, cat.by_hip(hip).unwrap(), t, &opts), ra, dec);
            let sep_off = sep_to(star_apparent_equatorial(&eph, cat.by_hip(hip).unwrap(), t, &off), ra, dec);
            worst = worst.max(sep);
            tab.row(vec![hip.to_string(), crate::test_util::jd_to_calendar(jd), format!("{:.6}", ra.to_degrees()), format!("{:.6}", dec.to_degrees()), format!("{sep:.3}"), format!("{sep_off:.3}")]);
            assert!(sep < 0.1, "HIP {hip} jd {jd}: {sep:.3}″");
        }
        tab.write();
        println!("恒星视位置对 Skyfield 最大差 {worst:.3}″");
    }

    /// Skyfield（DE406，P03，`scripts/phenomena/skyfield_deflection_near_sun.py`）近合日恒星视位置：
    /// (HIP, jd TT, ra rad, dec rad, 距角°, 去掉偏折的 ra rad, dec rad)。轩辕十四、角宿一、心宿二、毕宿五，2000 年前后距日最近之日及其后 1—60 日。
    const SKYFIELD_NEAR_SUN: &[(u32, f64, f64, f64, f64, f64, f64)] = &[
        (49669, 2451413.8863, 2.6542867852547736, 0.20894391529637538, 0.4649, 2.654285058058016, 0.20893940746523015),
        (49669, 2451414.8863, 2.6542843885248812, 0.2089407209927712, 1.0700, 2.654285864939746, 0.20893920867679006),
        (49669, 2451415.8863, 2.654285631545413, 0.20893970304416495, 1.9828, 2.6542865865324408, 0.20893906988401686),
        (49669, 2451418.8863, 2.65428743547115, 0.20893918808837816, 4.8438, 2.654287859724702, 0.20893898503109593),
        (49669, 2451423.8863, 2.6542896632981465, 0.20893821659370135, 9.6651, 2.654289880538947, 0.20893812503585948),
        (49669, 2451443.8863, 2.65431021731153, 0.20893115886398733, 29.1161, 2.6543102893737585, 0.20893113122058532),
        (49669, 2451473.8863, 2.654355508743312, 0.20891420177151823, 58.7052, 2.654355542427187, 0.20891418921463337),
        (65474, 2451469.1605, 3.51311246671542, -0.19471095133183025, 2.0547, 3.5131128843093493, -0.19470992545066254),
        (65474, 2451470.1605, 3.513112819552564, -0.19471083835663272, 2.2818, 3.5131135679906684, -0.1947101669113655),
        (65474, 2451471.1605, 3.5131134057543396, -0.19471070963083764, 2.8569, 3.513114144745939, -0.19471038389601691),
        (65474, 2451474.1605, 3.513114693342445, -0.1947107076702625, 5.3745, 3.513115124100507, -0.19471070290438844),
        (65474, 2451479.1605, 3.513118188603725, -0.19471119733196188, 10.1554, 3.513118412851976, -0.19471123698168852),
        (65474, 2451499.1605, 3.513143082930563, -0.19471929590967083, 30.0720, 3.513143154875627, -0.194719319139317),
        (65474, 2451529.1605, 3.513200522394634, -0.19474017447396902, 60.4243, 3.5132005552565935, -0.19474018659344883),
        (80763, 2451514.9035, 4.3168980053212485, -0.4612701392151219, 4.5702, 4.316898091431121, -0.46126964340104554),
        (80763, 2451515.9035, 4.316898355178599, -0.46126993668114635, 4.6811, 4.316898554510214, -0.4612694804045464),
        (80763, 2451516.9035, 4.316898897103111, -0.4612696681418654, 4.9993, 4.316899174705906, -0.4612692825259897),
        (80763, 2451519.9035, 4.316901971235954, -0.46126894856669437, 6.8248, 4.31690228573153, -0.46126876524874916),
        (80763, 2451524.9035, 4.316909583056019, -0.4612691732048634, 11.1264, 4.3169098049475325, -0.4612691194547758),
        (80763, 2451544.9035, 4.316941021211422, -0.461271836475149, 30.8240, 4.31694110251603, -0.4612718379294642),
        (80763, 2451574.9035, 4.317008137648448, -0.46128098477990453, 61.1562, 4.317008175284643, -0.4612809885116776),
        (21421, 2451695.4324, 1.2038532551947403, 0.2881091492053456, 5.4671, 1.2038531941881867, 0.28810955278522365),
        (21421, 2451696.4324, 1.2038539023190868, 0.28810918467653684, 5.5503, 1.203853914866743, 0.2881095861047913),
        (21421, 2451697.4324, 1.2038549048628902, 0.288109277776844, 5.7926, 1.2038549822093192, 0.28810965527989396),
        (21421, 2451700.4324, 1.2038594247213628, 0.28811028485452744, 7.2648, 1.2038595992503776, 0.2881105416018592),
        (21421, 2451705.4324, 1.203864524777829, 0.2881127545809717, 11.0135, 1.203864691188586, 0.2881128779182561),
        (21421, 2451725.4324, 1.2038907403981316, 0.28811931576988, 29.1344, 1.2038908144169247, 0.28811933902651865),
        (21421, 2451755.4324, 1.2039498571481617, 0.2881337332574349, 57.4505, 1.2039498933160275, 0.28813374051825597),
    ];

    fn unit_f64(p: Position) -> [f64; 3] {
        let p = p.to_meters();
        let n = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        [(p[0] / n).as_f64(), (p[1] / n).as_f64(), (p[2] / n).as_f64()]
    }

    fn radec_unit(ra: f64, dec: f64) -> [f64; 3] {
        [dec.cos() * ra.cos(), dec.cos() * ra.sin(), dec.sin()]
    }

    /// 近合日恒星对 Skyfield。两边的偏折量逐例比较：本实现「开 − 关」与 Skyfield「apparent − 去掉偏折」两个方向差矢量之差，
    /// 与两边其余差异（以日心代质心等，约 0.03—0.08″，与距角无关）分开。断言偏折矢量差 < 1 mas、批量与逐颗一致。
    #[test]
    fn star_deflection_near_sun_vs_skyfield() {
        let _ = try_init_full_nutation_from_repo();
        let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
        let eph = Vsop87Planets { earth: &earth, planets: vec![] };
        let cat = load_hipparcos_bright_from_repo().unwrap();
        let opts = ApparentPipelineOptions { use_p03_precession: true, ..Default::default() };
        let off = ApparentPipelineOptions { light_deflection: LightDeflection::None, ..opts.clone() };
        let mut tab = crate::test_util::CheckTable::new(
            "star_deflection_near_sun_vs_skyfield",
            &["HIP", "历元(TT)", "距角°", "本实现偏折″", "Skyfield偏折″", "偏折矢量差mas", "0.00407″·cot(ψ/2)", "位置差″(计偏折)", "位置差″(不计偏折)"],
        );
        tab.meta("近合日恒星对 Skyfield（DE406，P03，scripts/phenomena/skyfield_deflection_near_sun.py）：偏折量逐例比较（本实现开−关 对 Skyfield apparent−去偏折），及两边位置差");
        let (mut worst, mut worst_pos): (f64, f64) = (0.0, 0.0);
        for &(hip, jd, ra, dec, elong, ra0, dec0) in SKYFIELD_NEAR_SUN {
            let star = cat.by_hip(hip).unwrap();
            let t = TimePoint::new(TimeScale::TT, real(jd));
            let on = star_apparent_equatorial(&eph, star, t, &opts);
            let batch = stars_apparent_equatorial(&eph, core::slice::from_ref(star), t, &opts)[0];
            let off_p = star_apparent_equatorial(&eph, star, t, &off);
            let (u_on, u_off) = (unit_f64(on), unit_f64(off_p));
            let (s_on, s_off) = (radec_unit(ra, dec), radec_unit(ra0, dec0));
            let ours: [f64; 3] = core::array::from_fn(|i| u_on[i] - u_off[i]);
            let sky: [f64; 3] = core::array::from_fn(|i| s_on[i] - s_off[i]);
            let norm = |v: [f64; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt() * 206264.806;
            let dv = norm(core::array::from_fn(|i| ours[i] - sky[i])) * 1000.0;
            let (d, d_off, d_batch) = (sep_to(on, ra, dec), sep_to(off_p, ra, dec), sep_to(batch, ra, dec));
            let approx = 0.00407 / (elong.to_radians() / 2.0).tan();
            println!("HIP {hip} jd {jd} 距角 {elong}°：偏折 本实现 {:.4}″ / Skyfield {:.4}″，矢量差 {dv:.3} mas；位置差 {d:.4}″（不计偏折 {d_off:.4}″）", norm(ours), norm(sky));
            tab.row(vec![
                hip.to_string(),
                crate::test_util::jd_to_calendar(jd),
                format!("{elong:.4}"),
                format!("{:.4}", norm(ours)),
                format!("{:.4}", norm(sky)),
                format!("{dv:.3}"),
                format!("{approx:.4}"),
                format!("{d:.4}"),
                format!("{d_off:.4}"),
            ]);
            worst = worst.max(dv);
            worst_pos = worst_pos.max(d);
            assert!((d - d_batch).abs() < 1e-6, "HIP {hip} jd {jd}: 批量 {d_batch:.4}″ 与逐颗 {d:.4}″ 不一致");
        }
        tab.write();
        println!("近合日恒星：偏折矢量差最大 {worst:.3} mas，位置差最大 {worst_pos:.4}″");
        assert!(worst < 1.0, "偏折矢量差 {worst:.3} mas");
        assert!(worst_pos < 0.1, "位置差 {worst_pos:.4}″");
    }

    /// 批量与逐颗结果一致（前 500 年、2000 年各取 40 颗星）。
    #[test]
    fn batch_matches_single() {
        let _ = try_init_full_nutation_from_repo();
        let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::DEFAULT).unwrap();
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
