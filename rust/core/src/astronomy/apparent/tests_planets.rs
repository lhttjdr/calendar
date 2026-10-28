//! 行星视位置：VSOP87 与 DE406 互校（同一管线，只换历表），前 2000 年至 2000 年。

use super::*;
use crate::astronomy::ephemeris::vsop87_patch::PatchTarget;
use crate::astronomy::ephemeris::{
    load_earth_vsop87_corrected_from_repo, load_earth_vsop87_from_repo, load_planet_vsop87_corrected_from_repo, load_planet_vsop87_from_repo,
    De406Kernel, Vsop87,
};
use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
use crate::astronomy::pipeline::{Body, FrameMapper, Fk5ToIcrsBias, LightTimeCorrector, Planet, State6, TransformGraph, Vsop87Planets};
use crate::quantity::{epoch::Epoch, position::Position, reference_frame::ReferenceFrame, velocity::Velocity};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, RealOps};

fn sep_arcsec(a: [f64; 3], b: [f64; 3]) -> f64 {
    let na = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    let nb = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
    let c = [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let s = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
    let d = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    (s / (na * nb)).atan2(d / (na * nb)).to_degrees() * 3600.0
}

fn to_f64(p: crate::quantity::position::Position) -> [f64; 3] {
    let m = p.to_meters();
    [m[0].as_f64(), m[1].as_f64(), m[2].as_f64()]
}

/// VSOP87（原样、加改正）与 JPL 历表互校，前 2000 年至 2000 年每 125 年一点（加小偏移避开整年）：
/// - 日心方向（ICRS）：原样 VSOP87 只打印（其常数拟合自 DE200，远离 DE406，见 Chapront 2000 表 II、III）；改正后断言
///   不超过拟合脚本独立检验所得上限（各体见 `patched_limit`）；
/// - 地心视位置（当日真赤道）：改正后断言 ≤ 1.5″（地球与行星误差合成后除以地心距，近地时放大）。
#[test]
fn planets_vsop87_vs_de406() {
    let Some(bsp) = crate::test_util::de406_bsp_path_if_exists() else {
        println!("skipped: 无 DE406 BSP");
        return;
    };
    check_vsop87_patch_against(&De406Kernel::open(&bsp).unwrap(), PatchTarget::De406);
}

/// 同上，目标历表 DE441（`vendor/jpl/de441/de441.bsp` 本地验证用，不入库；缺文件时跳过）。
#[test]
fn planets_vsop87_vs_de441() {
    let Ok(kernel) = De406Kernel::open(crate::repo::repo_root().join(crate::repo::paths::DE441_BSP)) else {
        println!("skipped: 无 DE441 BSP");
        return;
    };
    check_vsop87_patch_against(&kernel, PatchTarget::De441);
}

fn check_vsop87_patch_against(kernel: &De406Kernel, target: PatchTarget) {
    assert!(try_init_full_nutation_from_repo(), "未能加载 IAU 2000A 章动表");
    let earth = load_earth_vsop87_from_repo().unwrap();
    let earth_fix = load_earth_vsop87_corrected_from_repo(target).unwrap_or_else(|e| panic!("{}: {e}", target.repo_path()));
    let loaded: Vec<(Planet, Vsop87, Vsop87)> = Planet::ALL
        .iter()
        .map(|p| (*p, load_planet_vsop87_from_repo(*p).unwrap(), load_planet_vsop87_corrected_from_repo(*p, target).unwrap()))
        .collect();
    let raw = Vsop87Planets { earth: &earth, planets: loaded.iter().map(|(p, v, _)| (*p, v)).collect() };
    let fixed = Vsop87Planets { earth: &earth_fix, planets: loaded.iter().map(|(p, _, v)| (*p, v)).collect() };
    let opts = ApparentPipelineOptions::pipeline_default();
    let graph = TransformGraph::default_graph()
        .with_fk5_to_icrs_mapper(|s, jd_tt| Fk5ToIcrsBias.apply(s, TimePoint::new(TimeScale::TT, jd_tt)));
    let helio_icrs = |v: &Vsop87, jd: f64| {
        let (hp, hv) = v.heliocentric_rectangular(real(jd));
        let f = ReferenceFrame::MeanEcliptic(Epoch::j2000());
        let st = State6::new(Position::from_si_meters_in_frame(f, hp[0], hp[1], hp[2]), Velocity::from_si_m_per_s_in_frame(f, hv[0], hv[1], hv[2]));
        to_f64(graph.transform_to(st, ReferenceFrame::ICRS, real(jd)).position)
    };
    let mut failures = vec![];
    for (p, pv, pv_fix) in &loaded {
        let (mut h_raw, mut h_fix, mut g_raw, mut g_fix) = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
        for k in 0..=32 {
            let year = -2000.0 + 125.0 * k as f64;
            let jd = 2451545.0 + (year - 2000.0) * 365.25 + 37.3 * k as f64;
            let (bp, _) = kernel.compute_state_km(0, p.naif_barycenter(), jd).unwrap();
            let (sp, _) = kernel.compute_state_km(0, 10, jd).unwrap();
            let de = [bp[0] - sp[0], bp[1] - sp[1], bp[2] - sp[2]];
            h_raw = h_raw.max(sep_arcsec(helio_icrs(pv, jd), de));
            h_fix = h_fix.max(sep_arcsec(helio_icrs(pv_fix, jd), de));
            let t = TimePoint::new(TimeScale::TT, real(jd));
            let d = to_f64(planet_apparent_equatorial(kernel, *p, t, &opts));
            g_raw = g_raw.max(sep_arcsec(to_f64(planet_apparent_equatorial(&raw, *p, t, &opts)), d));
            g_fix = g_fix.max(sep_arcsec(to_f64(planet_apparent_equatorial(&fixed, *p, t, &opts)), d));
        }
        println!("[{target:?}] {p:?}: 日心 原样 {h_raw:.2}″ → 改正 {h_fix:.3}″；地心视位置 原样 {g_raw:.2}″ → 改正 {g_fix:.3}″");
        if h_fix > patched_limit(*p) {
            failures.push(format!("{p:?} 改正后日心差 {h_fix:.3}″ > {}″", patched_limit(*p)));
        }
        if g_fix > 1.5 {
            failures.push(format!("{p:?} 改正后地心差 {g_fix:.3}″"));
        }
    }
    assert!(failures.is_empty(), "{failures:?}");
}

/// 改正后日心方向差上限：拟合脚本在前 3000—3000 年独立检验点上的最大残差（见 data/fit 文件生成日志）取整加余量。
fn patched_limit(p: Planet) -> f64 {
    match p {
        Planet::Saturn => 1.2,
        Planet::Mercury | Planet::Uranus => 0.5,
        _ => 0.4,
    }
}

/// Skyfield 1.x 读同一 de406.bsp 算得的行星地心视位置（当日真赤道与春分点）：(JD TT, 行星, α rad, δ rad, 距角 °)。
/// 由 `scripts/ephemeris/skyfield_planet_apparent.py` 生成（2026-10-26）。
#[rustfmt::skip]
const SKYFIELD: [(f64, Planet, f64, f64, f64); 35] = [
    (2451545.0, Planet::Mercury, 4.74857631998049, -0.4261892588111112, 8.54),
    (2451545.0, Planet::Venus, 4.186902659926794, -0.3219908889145466, 38.85),
    (2451545.0, Planet::Mars, 5.768597010120934, -0.23008133236155615, 47.60),
    (2451545.0, Planet::Jupiter, 0.4165722648224803, 0.1499979742143498, 104.88),
    (2451545.0, Planet::Saturn, 0.6765841594621419, 0.22016935243614066, 120.00),
    (2451545.0, Planet::Uranus, 5.54098080018725, -0.2970606344450442, 34.45),
    (2451545.0, Planet::Neptune, 5.330808393900074, -0.3353341897462795, 22.83),
    (2460000.5, Planet::Mercury, 5.6413527480462715, -0.2903812472640552, 16.06),
    (2460000.5, Planet::Venus, 0.09774299378908421, 0.026276814921479208, 29.64),
    (2460000.5, Planet::Mars, 1.330339404240349, 0.44155786156099464, 101.44),
    (2460000.5, Planet::Jupiter, 0.18383519371987878, 0.05789107252782281, 34.89),
    (2460000.5, Planet::Saturn, 5.782923722279688, -0.22865775052454287, 7.47),
    (2460000.5, Planet::Uranus, 0.7513239440364764, 0.2816408810845822, 69.31),
    (2460000.5, Planet::Neptune, 6.201643830567838, -0.05751532262984227, 18.34),
    (2086308.0, Planet::Mercury, 4.93509228758143, -0.4339997282452922, 5.05),
    (2086308.0, Planet::Venus, 5.64304136697294, -0.28469233983752185, 34.11),
    (2086308.0, Planet::Mars, 2.086022312421996, 0.4385706644739756, 168.88),
    (2086308.0, Planet::Jupiter, 4.717744878946304, -0.4103468366695348, 16.00),
    (2086308.0, Planet::Saturn, 0.7927794322044329, 0.2626425233622202, 120.98),
    (2086308.0, Planet::Uranus, 5.854034064841007, -0.19351323006096763, 46.90),
    (2086308.0, Planet::Neptune, 4.633985781936442, -0.3918984041206018, 20.46),
    (1757585.0, Planet::Mercury, 5.145262648442978, -0.4099075756796034, 11.02),
    (1757585.0, Planet::Venus, 5.650863387808059, -0.2788875594535854, 39.14),
    (1757585.0, Planet::Mars, 4.752559975215986, -0.4279232796279439, 9.70),
    (1757585.0, Planet::Jupiter, 5.263019538466243, -0.37170201270829956, 17.44),
    (1757585.0, Planet::Saturn, 3.6846879815939055, -0.17969266734096281, 69.23),
    (1757585.0, Planet::Uranus, 1.0145344136604377, 0.3559774263980401, 138.58),
    (1757585.0, Planet::Neptune, 1.5108570899250404, 0.398590083101893, 165.06),
    (1355818.0, Planet::Mercury, 4.837948877725603, -0.44735481408099437, 5.73),
    (1355818.0, Planet::Venus, 5.539190440715606, -0.3154755721758883, 32.57),
    (1355818.0, Planet::Mars, 5.3989837203960835, -0.34787944625436296, 24.75),
    (1355818.0, Planet::Jupiter, 0.3470639754747778, 0.13102882417975403, 99.32),
    (1355818.0, Planet::Saturn, 1.1065642209664794, 0.3599483177608708, 143.35),
    (1355818.0, Planet::Uranus, 0.18527816300115338, 0.07056963094669726, 89.48),
    (1355818.0, Planet::Neptune, 3.310188945186798, -0.035012144321482114, 92.21),
];

/// 视位置管线外部核对：同一 DE406，本管线 vs Skyfield。两者算法差别：
/// - Skyfield 另计太阳等引力光线偏折（约 0.00407″·cot(ψ/2)，ψ 为距角）及光行差的相对论高阶项；
/// - 岁差同为 IAU 2006（P03），但本管线用 ζ_A、θ_A、z_A 三角式，Skyfield 用 ψ_A、ω_A、χ_A 四角式（Capitaine et al. 2003），
///   两者都是截断多项式，远端略有分歧：按各自系数直接构造矩阵比较，t = −19 世纪差 0.09″、t = −30 世纪差 1.25″。
/// 断言：去掉偏折量级后，|t| ≤ 20 世纪剩余差 ≤ 0.1″；前 1000 年（t ≈ −30）≤ 1.3″。
/// 本测试发现并验证了 2026-10-26 修正的 P03 ζ/z 对调（修正前公元 100 年差 51″、前 1000 年差 196″）。
#[test]
fn planets_pipeline_vs_skyfield() {
    let Some(bsp) = crate::test_util::de406_bsp_path_if_exists() else {
        println!("skipped: 无 DE406 BSP");
        return;
    };
    let kernel = De406Kernel::open(&bsp).unwrap();
    assert!(try_init_full_nutation_from_repo(), "未能加载 IAU 2000A 章动表");
    // Skyfield 用 IAU 2006（P03）岁差，同模型比较
    let mut opts = ApparentPipelineOptions::pipeline_default();
    opts.precession_model = Some(crate::astronomy::frame::precession::PrecessionModel::P03);
    let mut failures = vec![];
    for (jd, p, ra, dec, elong) in SKYFIELD {
        let t = TimePoint::new(TimeScale::TT, real(jd));
        let ours = to_f64(planet_apparent_equatorial(&kernel, p, t, &opts));
        let sky = [dec.cos() * ra.cos(), dec.cos() * ra.sin(), dec.sin()];
        let d = sep_arcsec(ours, sky);
        // 太阳引力偏折：δ = (4GM/c²R)·cot(ψ/2)/2·… 近似 0.00407″·cot(ψ/2)（距地 1 au）
        let psi = elong.to_radians();
        let deflection = 0.00407 / (psi / 2.0).tan();
        println!("JD {jd} {p:?}：差 {d:.4}″（距角 {elong}°，太阳偏折量级 {deflection:.4}″）");
        let tol = if ((jd - 2451545.0) / 36525.0).abs() <= 20.0 { 0.1 } else { 1.3 };
        if d - deflection > tol {
            failures.push(format!("JD {jd} {p:?}：{d:.4}″"));
        }
    }
    assert!(failures.is_empty(), "{failures:?}");
}


/// 太阳视黄经与 DE406 之差：原样 VSOP87 与施加 DE406 版地球改正（`data/fit/vsop87-de406-planets.txt` 的 `ear` 段，
/// Chapront 2000 方法）的 VSOP87，同一管线（`sun_apparent_ecliptic_longitude_with_options`），前 3000—3000 年每 100 年。
/// 基准 `sun_apparent_ecliptic_longitude_de406_with_options`（同一光行时、岁差章动）。
/// 2026-10-26 前节气管线用的旧 patch（`vsop87-de406-icrs.txt`，地心太阳赤道坐标，拟合区间前 1000—3000 年、带人为边界）
/// 在此测试中现代约 0.14″、前 3000 年 5.3″；新改正全区间 ≤ 0.16″，故以新改正取代（见 doc/astronomy/celestial-phenomena.md §九 P3）。
#[test]
fn earth_patch_comparison_vs_de406() {
    let Some(bsp) = crate::test_util::de406_bsp_path_if_exists() else {
        println!("skipped: 无 DE406 BSP");
        return;
    };
    let kernel = De406Kernel::open(&bsp).unwrap();
    assert!(try_init_full_nutation_from_repo(), "未能加载 IAU 2000A 章动表");
    let raw = load_earth_vsop87_from_repo().unwrap();
    let fixed = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
    let opts = ApparentPipelineOptions::pipeline_default();
    let wrap = |d: f64| (d + 180.0).rem_euclid(360.0) - 180.0;
    let (mut rows, mut worst) = (vec![], [0.0_f64; 2]);
    for k in 0..=60 {
        let year = -3000.0 + 100.0 * k as f64;
        let jd = (2451545.0 + (year - 2000.0) * 365.25 + 17.3 * k as f64).clamp(625362.0, 2816847.0);
        let t = TimePoint::new(TimeScale::TT, real(jd));
        let de = sun_apparent_ecliptic_longitude_de406_with_options(&kernel, t, &opts).rad().as_f64().to_degrees();
        let a = sun_apparent_ecliptic_longitude_with_options(&raw, t, &opts).rad().as_f64().to_degrees();
        let b = sun_apparent_ecliptic_longitude_with_options(&fixed, t, &opts).rad().as_f64().to_degrees();
        let (d_raw, d_fix) = (wrap(a - de).abs() * 3600.0, wrap(b - de).abs() * 3600.0);
        worst[0] = worst[0].max(d_raw);
        worst[1] = worst[1].max(d_fix);
        rows.push(format!("{year}:{d_raw:.2}/{d_fix:.2}"));
    }
    println!("原样 / DE406 版改正（″）：{}", rows.join(" "));
    println!("最大：原样 {:.3}″，DE406 版改正 {:.3}″", worst[0], worst[1]);
    assert!(worst[1] < 0.2, "改正后最大差 {:.3}″", worst[1]);
}


/// 节气基准对照：《月相和二十四节气的计算》§7.4 扩展表 `TDBtimes_extended.txt`（自称 DE441 + Vondrák 2011 岁差 + IAU 2000A 章动），
/// 在其节气时刻（TDB）上，太阳视黄经按定义恰为 15° 的整数倍；算本仓库各配置在该时刻的视黄经残差，换算成节气时刻差（秒）。
/// 前 2900—2900 年（DE406 覆盖内）每 100 年一年 × 24 节气，岁差均为 Vondrák 2011。配置：
/// A VSOP87 + DE406 版改正；B VSOP87 + DE441 版改正；C DE406；D DE441（缺 DE441 文件时略去 B 以外的 D）。
/// 只打印：扩展表在古代与论文定义的 Vondrák 模型不符（见 `vondrak_longitude_vs_ecliptic_pole_method`），公元 300 年后可作参照。
#[test]
fn solar_terms_vs_extended_reference() {
    let Some(bsp) = crate::test_util::de406_bsp_path_if_exists() else {
        println!("skipped: 无 DE406 BSP");
        return;
    };
    let path = crate::repo::repo_root().join("vendor/月相和二十四节气的计算/TDBtimes_extended.txt");
    let Ok(text) = std::fs::read_to_string(&path) else {
        println!("skipped: 无 {path:?}");
        return;
    };
    let k406 = De406Kernel::open(&bsp).unwrap();
    let k441 = De406Kernel::open(crate::repo::repo_root().join(crate::repo::paths::DE441_BSP)).ok();
    assert!(try_init_full_nutation_from_repo(), "未能加载 IAU 2000A 章动表");
    let e406 = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
    let e441 = load_earth_vsop87_corrected_from_repo(PatchTarget::De441).unwrap();
    use crate::astronomy::frame::precession::PrecessionModel;
    let mut vond = ApparentPipelineOptions::pipeline_default();
    vond.precession_model = Some(PrecessionModel::Vondrak2011);
    let to_s = |d: f64| (d + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI;
    let sec = |d: f64| to_s(d) / (std::f64::consts::TAU / 365.2422) * 86400.0;
    let mut per_era = vec![];
    for line in text.lines().skip(1) {
        let tok: Vec<&str> = line.split_whitespace().collect();
        let Ok(year) = tok[0].parse::<i32>() else { continue };
        if year < -2900 || year > 2900 || year % 100 != 0 {
            continue;
        }
        let jd0: f64 = tok[1].parse().unwrap();
        let mut m = [0.0_f64; 4];
        // 第 4–27 栏：J12(小寒, 285°) 起每 15°
        for k in 0..24 {
            let jd = jd0 + tok[3 + k].parse::<f64>().unwrap();
            let target = ((285.0 + 15.0 * k as f64) % 360.0).to_radians();
            let t = TimePoint::new(TimeScale::TDB, real(jd));
            let mut vals = vec![
                sun_apparent_ecliptic_longitude_with_options(&e406, t, &vond).rad().as_f64(),
                sun_apparent_ecliptic_longitude_with_options(&e441, t, &vond).rad().as_f64(),
                sun_apparent_ecliptic_longitude_de406_with_options(&k406, t, &vond).rad().as_f64(),
            ];
            if let Some(k) = &k441 {
                vals.push(sun_apparent_ecliptic_longitude_de406_with_options(k, t, &vond).rad().as_f64());
            }
            for (i, v) in vals.into_iter().enumerate() {
                m[i] = m[i].max(sec(v - target).abs());
            }
        }
        per_era.push(format!("{year}:{:.1}/{:.1}/{:.1}/{:.1}", m[0], m[1], m[2], m[3]));
    }
    println!("节气时刻最大差（秒）A VSOP87+DE406 版 / B VSOP87+DE441 版 / C DE406 / D DE441（均 Vondrák）：");
    for chunk in per_era.chunks(10) {
        println!("  {}", chunk.join("  "));
    }
}


/// 视黄经（Vondrák 2011 岁差）的独立核对：不经变换图与黄赤交角，直接用 Vondrák et al. 2011 的黄道极、赤道极
/// （`vondrak2011::{ecliptic_pole, equator_pole}`，已对照论文附录 A.5 标准值）：春分点 = 赤道极 × 黄道极，把太阳视方向
/// （光行时后，ICRS → J2000 平赤道）投影到黄道面得平黄经，加黄经章动 Δψ。与管线之差只来自管线变换到视黄道时用 P03 平黄赤交角
/// 等二阶量，前 2900 年至 2000 年 ≤ 0.03″。
/// 2026-10-26 以此确认：《月相和二十四节气的计算》扩展表（自称 DE441 + Vondrák）在古代与论文定义的 Vondrák 模型不符
/// （同用 DE441，前 200 年差约 28 s、前 2900 年约 474 s），见 doc/astronomy/celestial-phenomena.md §九 P3。
#[test]
fn vondrak_longitude_vs_ecliptic_pole_method() {
    let Some(bsp) = crate::test_util::de406_bsp_path_if_exists() else {
        println!("skipped: 无 DE406 BSP");
        return;
    };
    let kernel = De406Kernel::open(&bsp).unwrap();
    assert!(try_init_full_nutation_from_repo(), "未能加载 IAU 2000A 章动表");
    use crate::astronomy::frame::precession::{vondrak2011, PrecessionModel};
    let mut vond = ApparentPipelineOptions::pipeline_default();
    vond.precession_model = Some(PrecessionModel::Vondrak2011);
    for jd in [2451545.0, 1648000.0, 1355818.0, 662000.0] {
        let t = TimePoint::new(TimeScale::TDB, real(jd));
        let lam_pipe = sun_apparent_ecliptic_longitude_de406_with_options(&kernel, t, &vond).rad().as_f64();
        let corrector: LightTimeCorrector<'_, De406Kernel, Fk5ToIcrsBias> = LightTimeCorrector { ephemeris: &kernel, mapper: None, max_iter: 2 };
        let (tr, st) = corrector.retarded_state(t.to_scale(TimeScale::TT), Body::Sun);
        let jd_tr = tr.to_scale(TimeScale::TT).jd;
        let [x, y, z] = st.position.to_meters();
        let (a, b, c) = crate::astronomy::frame::fixed::fk5_icrs::rotate_equatorial_icrs_to_fk5(x, y, z);
        let v = [a.as_f64(), b.as_f64(), c.as_f64()];
        let tc = (jd_tr - real(2451545.0)) / real(36525.0);
        let pe = vondrak2011::equator_pole(tc).map(|q| q.as_f64());
        let pc = vondrak2011::ecliptic_pole(tc).map(|q| q.as_f64());
        let cr = |a: [f64; 3], b: [f64; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
        let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        let mut eqx = cr(pe, pc);
        let n = dot(eqx, eqx).sqrt();
        eqx = eqx.map(|q| q / n);
        let yax = cr(pc, eqx);
        let lam_mean = dot(v, yax).atan2(dot(v, eqx));
        let dpsi = crate::astronomy::frame::nutation::nutation_for_apparent(tc).0.rad().as_f64();
        let d = ((lam_mean + dpsi - lam_pipe + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI).to_degrees() * 3600.0;
        println!("JD {jd}：黄道极法 − 管线 = {d:+.4}″");
        assert!(d.abs() < 0.03, "JD {jd}：{d}″");
    }
}


/// Skyfield 1.55 读同一 de406.bsp 的行星地心视星等（土星含环）：(JD TT, 行星, `planetary_magnitude` 原值, 真太阳几何值)。
/// 原值把太阳当作位于太阳系质心（Skyfield 源码注释所言），近日行星有 0.01—0.04 星等之差；真太阳几何值用 eph[10] 太阳位置、
/// 调用 Skyfield 同一套公式函数，与本仓库同法，作断言基准。由 `scripts/ephemeris/skyfield_planet_magnitude.py` 生成（2026-10-26）。
#[rustfmt::skip]
const SKYFIELD_MAG: [(f64, Planet, f64, f64); 28] = [
    (2451545.0, Planet::Mercury, -0.6908490795413624, -0.734184259033362),
    (2451545.0, Planet::Venus, -4.048360473507462, -4.066173941243579),
    (2451545.0, Planet::Mars, 1.0229943281987608, 1.0358375956120414),
    (2451545.0, Planet::Jupiter, -2.5235006360506063, -2.5205596110668),
    (2451545.0, Planet::Saturn, 0.1023840521546937, 0.10359714658515351),
    (2451545.0, Planet::Uranus, 5.942679908339012, 5.943041717207734),
    (2451545.0, Planet::Neptune, 7.854722181335482, 7.854841981376007),
    (2455000.5, Planet::Mercury, 0.13459197389177158, 0.16945139855526437),
    (2455000.5, Planet::Venus, -4.302376827062061, -4.286292024563007),
    (2455000.5, Planet::Mars, 1.1374577201782716, 1.1433809513657085),
    (2455000.5, Planet::Jupiter, -2.565428517029098, -2.5632330737593025),
    (2455000.5, Planet::Saturn, 0.9599024601051163, 0.9599370791639323),
    (2455000.5, Planet::Uranus, 5.9116355510641645, 5.912008440155584),
    (2455000.5, Planet::Neptune, 7.736584531244095, 7.7369246867556445),
    (2460000.5, Planet::Mercury, -0.4314909340812967, -0.4527549370210464),
    (2460000.5, Planet::Venus, -3.9437236064130725, -3.9339657205249154),
    (2460000.5, Planet::Mars, 0.3314520200961102, 0.32317070957791394),
    (2460000.5, Planet::Jupiter, -2.1098134066981507, -2.106199500259014),
    (2460000.5, Planet::Saturn, 0.8689239456212885, 0.8691337458561588),
    (2460000.5, Planet::Uranus, 5.812667467596567, 5.813373971666812),
    (2460000.5, Planet::Neptune, 7.8247475687089345, 7.825394554231806),
    (2458850.5, Planet::Mercury, -0.9558225475561494, -0.9434242563703543),
    (2458850.5, Planet::Venus, -3.9726899981645314, -3.9755981522402255),
    (2458850.5, Planet::Mars, 1.5736517385381643, 1.5701943042917996),
    (2458850.5, Planet::Jupiter, -1.8422037522769956, -1.8389868238966371),
    (2458850.5, Planet::Saturn, 0.5852864870858614, 0.5873449676980211),
    (2458850.5, Planet::Uranus, 5.780020667808113, 5.779903787145604),
    (2458850.5, Planet::Neptune, 7.789803826077332, 7.790188382735533),
];

/// 星等几何（r、Δ、相位角、土星/天王星直下点纬度）与公式整体对照 Skyfield（真太阳几何值）。两者公式同源（Ap_Mag V3）；
/// 差别：天王星本仓库按论文式 13 换算行星面图纬度（Skyfield 未换算，差约 1e-4 星等）；火星本仓库计季节修正 L(Ls)（Skyfield
/// 不计，比较前扣除）；光行时与位置的细小差异。容差 0.002。
#[test]
fn planet_magnitude_vs_skyfield() {
    let Some(bsp) = crate::test_util::de406_bsp_path_if_exists() else {
        println!("skipped: 无 DE406 BSP");
        return;
    };
    let kernel = De406Kernel::open(&bsp).unwrap();
    let mut failures = vec![];
    let mut tab = crate::test_util::CheckTable::new("planet_magnitude_vs_skyfield", &["行星", "历元(TT)", "Skyfield（真太阳）", "Skyfield 原值（太阳取质心）", "本实现", "差"]);
    tab.meta("行星视星等对 Skyfield（DE406，scripts/ephemeris/skyfield_planet_magnitude.py）逐点；火星已扣除 Skyfield 未计的季节修正 L(Ls)");
    for (jd, p, sf_barycentric, expect) in SKYFIELD_MAG {
        let t = TimePoint::new(TimeScale::TT, real(jd));
        // Skyfield 不计火星季节修正 L(Ls)，比较前从本仓库结果中扣除
        let ls_corr = if p == Planet::Mars {
            mars_orbital_correction(planet_magnitude_geometry(&kernel, p, t).helio_ecl_long_deg).as_f64()
        } else {
            0.0
        };
        let v = planet_magnitude(&kernel, p, t).map(|v| v.as_f64() - ls_corr);
        println!("JD {jd} {p:?}: 本仓库 {v:?}，Skyfield 真太阳 {expect:.4}，Skyfield 原值（太阳取质心）{sf_barycentric:.4}");
        let fmtv = |x: Option<f64>| x.map(|x| format!("{x:.4}")).unwrap_or("（不给）".into());
        tab.row(vec![
            format!("{p:?}"),
            crate::test_util::jd_to_calendar(jd),
            if expect.is_nan() { "（不给）".into() } else { format!("{expect:.4}") },
            if sf_barycentric.is_nan() { "—".into() } else { format!("{sf_barycentric:.4}") },
            fmtv(v),
            match v { Some(v) if !expect.is_nan() => format!("{:+.4}", v - expect), _ => "—".into() },
        ]);
        match v {
            Some(v) if (v - expect).abs() <= 0.002 => {}
            None if expect.is_nan() => {}
            _ => failures.push(format!("JD {jd} {p:?}: {v:?} vs {expect}")),
        }
    }
    tab.write();
    assert!(failures.is_empty(), "{failures:?}");
}
