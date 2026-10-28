//! 伏见检验。

use super::visibility::night_sky_from_zenith_limit;
use super::*;
use crate::astronomy::apparent::{body_apparent_ecliptic, ApparentPipelineOptions};
use crate::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
use crate::astronomy::ephemeris::vsop87_patch::PatchTarget;
use crate::astronomy::ephemeris::{load_earth_vsop87_corrected_from_repo, load_planet_vsop87_corrected_from_repo};
use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
use crate::astronomy::geodesy::Observer;
use crate::astronomy::pipeline::{Body, EphemerisProvider, Planet, Vsop87Elp, Vsop87Planets};
use crate::astronomy::stars::load_hipparcos_bright_from_repo;
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::duration::Duration;
use crate::quantity::length::Length;
use crate::quantity::unit::LengthUnit;

fn with_eph<F: FnOnce(&Vsop87Elp)>(f: F) {
    let _ = try_init_full_nutation_from_repo();
    let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
    let loaded: Vec<_> = Planet::ALL.iter().map(|p| (*p, load_planet_vsop87_corrected_from_repo(*p, PatchTarget::De406).unwrap())).collect();
    let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DE406).unwrap();
    let eph = Vsop87Elp { planets: Vsop87Planets { earth: &earth, planets: loaded.iter().map(|(p, v)| (*p, v)).collect() }, moon: &elp };
    f(&eph)
}

fn observer(lon: f64, lat: f64, h_m: f64) -> Observer {
    Observer::new(PlaneAngle::from_deg(real(lon)), PlaneAngle::from_deg(real(lat)), Length::from_value(real(h_m), LengthUnit::Meter))
}

fn jd_to_ymd(jd: f64) -> String {
    let z = (jd + 0.5).floor();
    let f = jd + 0.5 - z;
    let a = if z < 2299161.0 {
        z
    } else {
        let al = ((z - 1867216.25) / 36524.25).floor();
        z + 1.0 + al - (al / 4.0).floor()
    };
    let b = a + 1524.0;
    let c = ((b - 122.1) / 365.25).floor();
    let d = (365.25 * c).floor();
    let e = ((b - d) / 30.6001).floor();
    let day = b - d - (30.6001 * e).floor() + f;
    let m = if e < 14.0 { e - 1.0 } else { e - 13.0 };
    let y = if m > 2.0 { c - 4716.0 } else { c - 4715.0 };
    format!("{y}-{m:02}-{:05.2}", day)
}

/// 太阳视黄经为 0（春分）的 TT 儒略日：`guess` 前后 60 日内按 2 日扫描找 − → + 变号，再二分。
fn vernal_equinox<P: EphemerisProvider>(eph: &P, guess: f64) -> f64 {
    let o = ApparentPipelineOptions::default();
    let f = |jd: f64| body_apparent_ecliptic(eph, Body::Sun, TimePoint::new(TimeScale::TT, real(jd)), &o).0.rad().wrap_to_signed_pi().as_f64();
    let mut a = guess - 60.0;
    while !(f(a) < 0.0 && f(a + 2.0) >= 0.0 && f(a) > -0.5) {
        a += 2.0;
        assert!(a < guess + 60.0, "春分不在 {guess} ± 60 日内");
    }
    let mut b = a + 2.0;
    for _ in 0..50 {
        let m = 0.5 * (a + b);
        if f(m) < 0.0 {
            a = m
        } else {
            b = m
        }
    }
    0.5 * (a + b)
}

/// 由 m_z、k 设定 Schaefer 1987 式的天空（夜天光由 m_z 反推，月光不另计——m_z 已含）。
fn options_for(model: SkyModel, threshold: ThresholdModel, k: f64, mz: f64) -> HeliacalOptions {
    let eye = Eye { threshold, ..Default::default() };
    let night = night_sky_from_zenith_limit(real(mz), real(k), &eye);
    HeliacalOptions {
        sky: SkyConditions { extinction: ExtinctionModel::Fixed { k: real(k) }, night_sky_nl: night, model, ..Default::default() },
        eye,
        include_moon: false,
        ..Default::default()
    }
}

fn model_combos() -> Vec<(&'static str, SkyModel, ThresholdModel)> {
    let ks = ScatteringFunction::KrisciunasSchaefer1991;
    let t93 = SkyModel::Schaefer1993 { twilight: TwilightModel::Schaefer1993 { direction: TwilightDirectionTerm::AsPrinted }, scattering: ks };
    let t87 = SkyModel::Schaefer1993 { twilight: TwilightModel::Schaefer1987, scattering: ks };
    let crumey = ThresholdModel::Crumey2014 { field_factor: real(2.0) };
    let mut v = vec![
        ("Hecht+1993蒙影", t93, ThresholdModel::Schaefer1993),
        ("Hecht+1987蒙影", t87, ThresholdModel::Schaefer1993),
        ("Schaefer1998整套", SkyModel::Schaefer1998, ThresholdModel::Schaefer1998 { snellen: real(1.0) }),
        ("Crumey(F=2)+1993蒙影", t93, crumey),
    ];
    // 诊断：HEL_CRUMEY_F="3,5" 追加其它 F 的 Crumey + 1993 蒙影
    if let Ok(list) = std::env::var("HEL_CRUMEY_F") {
        for f in list.split(',') {
            let name: &'static str = Box::leak(format!("Crumey(F={f})+1993蒙影").into_boxed_str());
            v.push((name, t93, ThresholdModel::Crumey2014 { field_factor: real(f.parse::<f64>().unwrap()) }));
        }
    }
    v
}

struct Row {
    star: String,
    hip: u32,
    rising: bool,
    lat: f64,
    k: f64,
    mz: f64,
    delta_obs: f64,
    symbol: String,
    delta_pred: f64,
    comments: String,
}

fn table1() -> Vec<Row> {
    let lines = crate::repo::read_lines("data/phenomena/schaefer1987_table1.csv").unwrap();
    lines
        .iter()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
        .map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            Row {
                star: f[0].into(),
                hip: f[1].parse().unwrap(),
                rising: f[2] == "R",
                lat: f[3].parse().unwrap(),
                k: f[4].parse().unwrap(),
                mz: f[5].parse().unwrap(),
                delta_obs: f[6].parse().unwrap(),
                symbol: f[7].into(),
                delta_pred: f[8].parse().unwrap(),
                comments: f.get(9).map(|s| s.to_string()).unwrap_or_default(),
            }
        })
        .collect()
}

/// 观测地（Schaefer 1987 §2 所列台站，按纬度对应）：经度、海拔。经度只影响当地日的划分，海拔只影响折射。
fn site(lat: f64) -> (f64, f64) {
    match lat {
        l if (l - 32.0).abs() < 0.1 => (-111.6, 2100.0), // KPNO
        l if (l + 30.2).abs() < 0.1 => (-70.8, 2200.0),  // CTIO
        l if (l - 34.1).abs() < 0.1 => (-107.6, 2100.0), // VLA
        l if (l - 39.0).abs() < 0.1 => (-76.8, 50.0),    // Greenbelt / Bowie, MD
        l if (l - 19.8).abs() < 0.1 => (-155.5, 4200.0), // IRTF（Mauna Kea）
        _ => (0.0, 0.0),
    }
}

/// Schaefer 1987 表 1：作者 1983—1986 年肉眼实测的 45 条恒星伏见记录（已转录为 data/phenomena/schaefer1987_table1.csv）。
/// 对每条记录，按其纬度、k、m_z 计算该日（春分后 Δ_obs 日，取 1985 年）晨（升）或昏（落）可见与否，与实测比对；
/// 并在 Δ_obs ± 30 日内求模型伏见日，与作者 1987 模型的 Δ_pred 比较。四种「阈值 × 蒙影」组合并列。
///
/// 实测的「可见」：升时符号为 ≫ > ≳，落时为 ≪ < ≲（原文：落时符号反向）。
#[test]
#[ignore]
fn schaefer1987_table1_observations() {
    let _ = try_init_full_nutation_from_repo();
    let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
    let eph = Vsop87Planets { earth: &earth, planets: vec![] };
    let cat = load_hipparcos_bright_from_repo().unwrap();
    let rows = table1();
    assert_eq!(rows.len(), 45);
    let eq = vernal_equinox(&eph, 2446145.0); // 1985-03-20
    let dt = Duration::in_seconds(real(54.6)); // 1985 年 ΔT
    let (j0, j1) = (eq + 55.0, eq + 385.0);
    let t = std::time::Instant::now();
    let tracks = GeocentricTracks::new(&eph, real(j0), real(j1), false, &ApparentPipelineOptions::default());
    println!("春分 {}，日取样 {:.1} s", jd_to_ymd(eq), t.elapsed().as_secs_f64());
    let combos = model_combos();
    let mut header: Vec<&'static str> = vec!["星", "升落", "纬度", "k", "m_z", "Δ_obs", "实测", "Δ_pred(1987)", "注"];
    for (name, _, _) in &combos {
        header.push(Box::leak(format!("{name}：当日可见").into_boxed_str()));
        header.push(Box::leak(format!("{name}：模型伏见日 Δ").into_boxed_str()));
    }
    let mut tab = crate::test_util::CheckTable::new("schaefer1987_table1", &header);
    tab.meta("Schaefer 1987 表 1：作者肉眼实测 45 条恒星伏见（data/phenomena/schaefer1987_table1.csv）。Δ 为春分后日数（取 1985 年）；");
    tab.meta("「当日可见」为模型在 Δ_obs 当日的判定，与实测一致记 ✓；实测「可见」= 升时 ≫ > ≳，落时 ≪ < ≲。各组合夜天光由 m_z 反推，不计月光");
    let mut agree = vec![[0usize; 2]; combos.len()]; // [全部, 去掉 ZL/MW/??]
    let mut n_clean = 0;
    let mut dpred_diff: Vec<Vec<f64>> = vec![vec![]; combos.len()];
    for r in &rows {
        let star = cat.by_hip(r.hip).unwrap_or_else(|| panic!("缺 HIP {}", r.hip));
        let d_obs = eq + r.delta_obs;
        let target = PreparedTarget::new(&eph, &HeliacalTarget::Star(*star), real(d_obs - 32.0), real(d_obs + 32.0));
        let (lon, h) = site(r.lat);
        let obs_visible = if r.rising { r.symbol.starts_with('>') } else { r.symbol.starts_with('<') };
        let clean = !(r.comments.contains("ZL") || r.comments.contains("MW") || r.comments.contains("??"));
        n_clean += clean as usize;
        let mut line = format!("{:8} {} φ{:5.1} k{:.2} m{:.1} Δobs{:4.0} {:2} Δpred{:4.0} {:6}|", r.star, if r.rising { 'R' } else { 'S' }, r.lat, r.k, r.mz, r.delta_obs, r.symbol, r.delta_pred, r.comments);
        let mut cells = vec![
            r.star.clone(),
            if r.rising { "升".into() } else { "落".into() },
            format!("{:.1}", r.lat),
            format!("{:.2}", r.k),
            format!("{:.1}", r.mz),
            format!("{:.0}", r.delta_obs),
            format!("{}（{}）", r.symbol, if obs_visible { "可见" } else { "不见" }),
            format!("{:.0}", r.delta_pred),
            r.comments.clone(),
        ];
        for (ci, (_, tw, th)) in combos.iter().enumerate() {
            let sky = HeliacalSky::new(&tracks, observer(lon, r.lat, h), dt, options_for(*tw, *th, r.k, r.mz));
            let days = sky.daily_visibility(&target, real(d_obs - 30.0), real(d_obs + 30.0));
            let half = if r.rising { 0 } else { 1 };
            let vis = |d: &DailyVisibility| d.windows[half].is_some_and(|(_, m)| m >= real(0.0));
            // Δ_obs 那一当地日（子夜在 Δ_obs 前 1 日内）
            let today = days.iter().find(|d| d.midnight_tt.as_f64() <= d_obs && d.midnight_tt.as_f64() > d_obs - 1.0).unwrap();
            let model_visible = vis(today);
            // 模型伏见日：升取最后一个由不见转见的日子，落取最后一个可见日之后
            let delta_model = if r.rising {
                days.windows(2).filter(|w| !vis(&w[0]) && vis(&w[1])).last().map(|w| w[1].midnight_tt.as_f64() + 0.5 - eq)
            } else {
                days.windows(2).filter(|w| vis(&w[0]) && !vis(&w[1])).last().map(|w| w[0].midnight_tt.as_f64() + 0.5 - eq)
            };
            if model_visible == obs_visible {
                agree[ci][0] += 1;
                if clean {
                    agree[ci][1] += 1;
                }
            }
            if let Some(dm) = delta_model {
                dpred_diff[ci].push(dm - r.delta_pred);
            }
            line += &format!(" {}{:>6}", if model_visible == obs_visible { "✓" } else { "✗" }, delta_model.map(|d| format!("{d:.0}")).unwrap_or("—".into()));
            cells.push(format!("{}{}", if model_visible { "可见" } else { "不见" }, if model_visible == obs_visible { " ✓" } else { " ✗" }));
            cells.push(delta_model.map(|d| format!("{d:.0}")).unwrap_or("—".into()));
        }
        println!("{line}");
        tab.row(cells);
    }
    tab.write();
    println!("可见与否与实测一致（全部 45 / 去掉 ZL、MW、?? 共 {n_clean}）：");
    for (ci, (name, _, _)) in combos.iter().enumerate() {
        let d = &dpred_diff[ci];
        let mut ad: Vec<f64> = d.iter().map(|x| x.abs()).collect();
        ad.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let med = if ad.is_empty() { f64::NAN } else { ad[ad.len() / 2] };
        let mean = d.iter().sum::<f64>() / d.len().max(1) as f64;
        println!("  {name:24} {:2}/45  {:2}/{n_clean}   模型日 − Δpred：均值 {mean:+.1}，|中位| {med:.1}（{} 条有解）", agree[ci][0], agree[ci][1], d.len());
    }
    println!("用时 {:.1} s", t.elapsed().as_secs_f64());
}

/// Schaefer 1987 表 7：尤卡坦（φ = 20°）金星可见/不可见时段（日），模型取 m = −3.4、k = 0.30、m_z = 6.0。
/// 表 7 为作者 1987 模型的计算值（非观测），此处对照各组合的同条件结果。5 个会合周期均值：
/// 下合不可见 11.2、晨星 251.6、上合不可见 69.4、昏星 250。
#[test]
#[ignore]
fn schaefer1987_table7_venus_yucatan() {
    let _ = try_init_full_nutation_from_repo();
    let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
    let venus = load_planet_vsop87_corrected_from_repo(Planet::Venus, PatchTarget::De406).unwrap();
    let eph = Vsop87Planets { earth: &earth, planets: vec![(Planet::Venus, &venus)] };
    let (j0, j1) = (2443144.5, 2446066.5); // 1977-01-01 … 1985-01-01
    let tracks = GeocentricTracks::new(&eph, real(j0), real(j1), false, &ApparentPipelineOptions::default());
    let target = PreparedTarget::new(&eph, &HeliacalTarget::PlanetFixedMagnitude(Planet::Venus, real(-3.4)), real(j0), real(j1));
    let mut tab = crate::test_util::CheckTable::new("schaefer1987_table7_venus", &["组合", "伏见事件", "日期", "视弧°", "距前一事件 日", "所属时段"]);
    tab.meta("Schaefer 1987 表 7 条件（尤卡坦 89°W、20°N，m = −3.4，k = 0.30，m_z = 6.0）下 1977—1984 金星伏见逐次；");
    tab.meta("表 7 五周期均值：下合伏 11.2、晨星 251.6、上合伏 69.4、昏星 250 日");
    for (name, tw, th) in model_combos() {
        let sky = HeliacalSky::new(&tracks, observer(-89.0, 20.0, 10.0), Duration::in_seconds(real(50.0)), options_for(tw, th, 0.30, 6.0));
        let ev = sky.events(&target, real(j0), real(j1));
        use HeliacalEventKind::*;
        let mut sums = [(0.0, 0); 4]; // 下合伏、晨星、上合伏、昏星
        for (i, e) in ev.iter().enumerate() {
            let (gap, seg) = if i == 0 {
                (String::from("—"), String::new())
            } else {
                let p = &ev[i - 1];
                let seg = match (p.kind, e.kind) {
                    (EveningLast, MorningFirst) => "下合伏",
                    (MorningFirst, MorningLast) => "晨星",
                    (MorningLast, EveningFirst) => "上合伏",
                    (EveningFirst, EveningLast) => "昏星",
                    _ => "（次序异常）",
                };
                (format!("{:.1}", e.jd_tt.as_f64() - p.jd_tt.as_f64()), seg.to_string())
            };
            tab.row(vec![name.to_string(), format!("{:?}", e.kind), crate::test_util::jd_to_calendar(e.jd_tt.as_f64()), format!("{:.1}", e.arcus_visionis_deg().as_f64()), gap, seg]);
        }
        for w in ev.windows(2) {
            let d = w[1].jd_tt.as_f64() - w[0].jd_tt.as_f64();
            let slot = match (w[0].kind, w[1].kind) {
                (EveningLast, MorningFirst) => 0,
                (MorningFirst, MorningLast) => 1,
                (MorningLast, EveningFirst) => 2,
                (EveningFirst, EveningLast) => 3,
                _ => continue,
            };
            sums[slot].0 += d;
            sums[slot].1 += 1;
        }
        let avg = |i: usize| sums[i].0 / sums[i].1.max(1) as f64;
        println!("{name:24} 下合伏 {:5.1}  晨星 {:6.1}  上合伏 {:5.1}  昏星 {:6.1}（表 7：11.2 / 251.6 / 69.4 / 250）", avg(0), avg(1), avg(2), avg(3));
    }
    tab.write();
}

/// Schaefer 2000（JHA 31, 149）表 1：埃及（φ = 30°N，k = 0.35）天狼星偕日升，春分后日数 DSVE。作者以其 1998 程序计算，
/// 此处对照各组合（m_z 取标准 6.0）。
#[test]
#[ignore]
fn schaefer2000_sirius_egypt() {
    use crate::astronomy::time::delta_t::{DeltaTModel, MorrisonStephenson2021};
    let _ = try_init_full_nutation_from_repo();
    let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
    let eph = Vsop87Planets { earth: &earth, planets: vec![] };
    let sirius = *load_hipparcos_bright_from_repo().unwrap().by_hip(32349).unwrap();
    let table = [(-3499, 87.8), (-2999, 92.3), (-2499, 95.8), (-1999, 100.3), (-1499, 104.8), (-999, 108.2), (-499, 112.9), (1, 117.3), (500, 123.0)];
    let combos = model_combos();
    let mut header: Vec<&'static str> = vec!["天文纪年", "Schaefer 2000 DSVE"];
    for (name, _, _) in &combos {
        header.push(Box::leak(format!("{name}：DSVE").into_boxed_str()));
        header.push(Box::leak(format!("{name}：差").into_boxed_str()));
        header.push(Box::leak(format!("{name}：视弧°").into_boxed_str()));
    }
    let mut tab = crate::test_util::CheckTable::new("schaefer2000_sirius", &header);
    tab.meta("Schaefer 2000 表 1：埃及（31.25°E、30°N）、k = 0.35 天狼星偕日升，春分后日数 DSVE；本实现 m_z = 6.0、ΔT 取 Morrison–Stephenson 2021");
    for (year, dsve) in table {
        // 春分约在儒略历 3 月 21 日前后（远古偏早），取该年 1 月 1 日（儒略历）+ 79 日为初值
        let jan1 = 1721423.5 + 365.25 * (year as f64 - 1.0);
        let jan1 = (jan1 - 0.5).floor() + 0.5;
        let eq = vernal_equinox(&eph, jan1 + 79.0);
        let dt = MorrisonStephenson2021.estimate(real(eq)).value;
        let (j0, j1) = (eq + dsve - 25.0, eq + dsve + 25.0);
        let tracks = GeocentricTracks::new(&eph, real(j0), real(j1), false, &ApparentPipelineOptions::default());
        let target = PreparedTarget::new(&eph, &HeliacalTarget::Star(sirius), real(j0), real(j1));
        let mut line = format!("{year:6} DSVE {dsve:6.1} |");
        let mut cells = vec![year.to_string(), format!("{dsve:.1}")];
        for (_, tw, th) in &combos {
            let sky = HeliacalSky::new(&tracks, observer(31.25, 30.0, 20.0), dt, options_for(*tw, *th, 0.35, 6.0));
            let first = sky.events(&target, real(j0 + 2.0), real(j1 - 2.0)).into_iter().find(|e| e.kind == HeliacalEventKind::MorningFirst);
            line += &match first {
                Some(e) => format!(" {:6.1} (AV {:4.1})", e.jd_tt.as_f64() - eq, e.arcus_visionis_deg().as_f64()),
                None => "      —          ".into(),
            };
            match first {
                Some(e) => {
                    let d = e.jd_tt.as_f64() - eq;
                    cells.extend([format!("{d:.1}"), format!("{:+.1}", d - dsve), format!("{:.1}", e.arcus_visionis_deg().as_f64())]);
                }
                None => cells.extend(["—".to_string(), "—".to_string(), "—".to_string()]),
            }
        }
        println!("{line}");
        tab.row(cells);
    }
    tab.write();
}

/// 探索输出：列出行星伏见事件与视弧。环境变量：HEL_LAT、HEL_LON、HEL_JD0、HEL_DAYS、HEL_DT、HEL_K、HEL_EPH=de441。
#[test]
#[ignore]
fn heliacal_explore() {
    if std::env::var("HEL_EPH").as_deref() == Ok("de441") {
        let _ = try_init_full_nutation_from_repo();
        let k = crate::astronomy::ephemeris::De406Kernel::open(crate::repo::repo_root().join(crate::repo::paths::DE441_BSP)).unwrap();
        explore(&k);
    } else {
        with_eph(|eph| explore(eph));
    }
}

fn explore<P: EphemerisProvider>(eph: &P) {
    let env = |k: &str, d: f64| std::env::var(k).ok().and_then(|s| s.parse().ok()).unwrap_or(d);
    let (lat, lon, y0, days, dt) = (env("HEL_LAT", 32.54), env("HEL_LON", 44.42), env("HEL_JD0", 2460676.5), env("HEL_DAYS", 730.0), env("HEL_DT", 70.0));
    let mut opts = HeliacalOptions::default();
    if let Ok(k) = std::env::var("HEL_K") {
        opts.sky.extinction = ExtinctionModel::Fixed { k: real(k.parse::<f64>().unwrap()) };
    }
    let t = std::time::Instant::now();
    let tracks = GeocentricTracks::new(eph, real(y0), real(y0 + days), true, &ApparentPipelineOptions::default());
    println!("日月取样 {:.1} s", t.elapsed().as_secs_f64());
    let sky = HeliacalSky::new(&tracks, observer(lon, lat, 50.0), Duration::in_seconds(real(dt)), opts);
    for p in [Planet::Mercury, Planet::Venus, Planet::Mars, Planet::Jupiter, Planet::Saturn] {
        let t = std::time::Instant::now();
        let target = PreparedTarget::new(eph, &HeliacalTarget::Planet(p), real(y0), real(y0 + days));
        let ev = sky.events(&target, real(y0), real(y0 + days));
        for e in &ev {
            println!(
                "{p:?}\t{:?}\t{}\tAV {:5.2}\th {:5.2}\tsun {:6.2}\tdaz {:5.1}\tm {:5.2}\tlim {:5.2}",
                e.kind,
                jd_to_ymd(e.jd_tt.as_f64()),
                e.arcus_visionis_deg().as_f64(),
                e.altitude_deg.as_f64(),
                e.sun_altitude_deg.as_f64(),
                e.azimuth_difference_deg.as_f64(),
                e.magnitude.as_f64(),
                e.limiting_magnitude.as_f64()
            );
        }
        println!("{p:?}: {} 个事件，{:.1} s", ev.len(), t.elapsed().as_secs_f64());
    }
}

/// 伏见与合日的次序：每次合日（外行星）、上合，其前最近的伏、其后最近的见须为对应种类，且伏见间隔合理；下合允许晨昏两见
/// 的重叠（水星允许整次晨现或昏现不可见）。合日时刻取自已对 Skyfield 验证的 [`planet_events`]。DE441（缺文件则跳过），巴比伦，2025–2026。
#[test]
fn heliacal_brackets_conjunctions() {
    let _ = try_init_full_nutation_from_repo();
    let Ok(k) = crate::astronomy::ephemeris::De406Kernel::open(crate::repo::repo_root().join(crate::repo::paths::DE441_BSP)) else {
        eprintln!("跳过：无 DE441");
        return;
    };
    let (j0, j1) = (2460676.5, 2460676.5 + 730.0);
    let tracks = GeocentricTracks::new(&k, real(j0), real(j1), true, &ApparentPipelineOptions::default());
    let sky = HeliacalSky::new(&tracks, observer(44.42, 32.54, 50.0), Duration::in_seconds(real(70.0)), HeliacalOptions::default());
    use HeliacalEventKind::*;
    let mut tab = crate::test_util::CheckTable::new(
        "heliacal_babylon_2025_2026",
        &["行星", "事件", "日期(TT)", "视弧°", "行星高度°", "太阳高度°", "方位差°", "星等", "极限星等"],
    );
    tab.meta("巴比伦（44.42°E、32.54°N）2025—2026 行星伏见逐次（DE441，缺省模型：Schaefer 1998 整套、湿度 50%、15 ℃、计月光与月光眩光），");
    tab.meta("合日时刻（planet_events，已对 Skyfield 验证）一并列出以示次序");
    for p in [Planet::Mercury, Planet::Venus, Planet::Mars, Planet::Jupiter, Planet::Saturn] {
        let target = PreparedTarget::new(&k, &HeliacalTarget::Planet(p), real(j0), real(j1));
        let hel = sky.events(&target, real(j0), real(j1));
        let conj: Vec<_> = planet_events(&k, p, real(j0), real(j1), &ApparentPipelineOptions::default())
            .into_iter()
            .filter(|e| matches!(e.kind, PlanetEventKind::Conjunction | PlanetEventKind::InferiorConjunction | PlanetEventKind::SuperiorConjunction))
            .collect();
        let mut lines: Vec<(f64, Vec<String>)> = vec![];
        for e in &hel {
            lines.push((
                e.jd_tt.as_f64(),
                vec![
                    format!("{p:?}"),
                    format!("{:?}", e.kind),
                    crate::test_util::jd_to_calendar(e.jd_tt.as_f64()),
                    format!("{:.2}", e.arcus_visionis_deg().as_f64()),
                    format!("{:.2}", e.altitude_deg.as_f64()),
                    format!("{:.2}", e.sun_altitude_deg.as_f64()),
                    format!("{:.1}", e.azimuth_difference_deg.as_f64()),
                    format!("{:.2}", e.magnitude.as_f64()),
                    format!("{:.2}", e.limiting_magnitude.as_f64()),
                ],
            ));
        }
        for c in &conj {
            let mut v = vec![format!("{p:?}"), format!("（{:?}）", c.kind), crate::test_util::jd_to_calendar(c.jd_tt.as_f64())];
            v.extend(std::iter::repeat_n(String::new(), 6));
            lines.push((c.jd_tt.as_f64(), v));
        }
        lines.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        for (_, v) in lines {
            tab.row(v);
        }
        let mut checked = 0;
        for c in &conj {
            let t = c.jd_tt.as_f64();
            if c.kind == PlanetEventKind::InferiorConjunction {
                // 下合：行星在日北（或南）若干度时，下合前后数日可晨昏两见（如 2025-03 金星在日北 8°，3/19 夕伏、3/22 晨见、
                // 3/23 下合）。只要求 [t − 60, t + 10] 内有夕伏、[t − 10, t + 60] 内有晨见（水星允许整次不可见）
                let el = hel.iter().any(|e| e.kind == EveningLast && (t - 60.0..t + 10.0).contains(&e.jd_tt.as_f64()));
                let mf = hel.iter().any(|e| e.kind == MorningFirst && (t - 10.0..t + 60.0).contains(&e.jd_tt.as_f64()));
                if p != Planet::Mercury && t - 60.0 > j0 && t + 60.0 < j1 {
                    assert!(el && mf, "{p:?} 下合 {}：夕伏 {el} 晨见 {mf}", jd_to_ymd(t));
                }
                checked += 1;
                continue;
            }
            let before = hel.iter().filter(|e| e.jd_tt.as_f64() < t).last();
            let after = hel.iter().find(|e| e.jd_tt.as_f64() > t);
            let (Some(b), Some(a)) = (before, after) else { continue };
            let (want_b, want_a) = match c.kind {
                PlanetEventKind::SuperiorConjunction => (MorningLast, EveningFirst),
                _ => (EveningLast, MorningFirst),
            };
            if p == Planet::Mercury && (b.kind, a.kind) != (want_b, want_a) {
                assert!(matches!(b.kind, MorningLast | EveningLast) && matches!(a.kind, MorningFirst | EveningFirst), "{p:?} 合 {}", jd_to_ymd(t));
                continue;
            }
            assert_eq!((b.kind, a.kind), (want_b, want_a), "{p:?} 合 {}", jd_to_ymd(t));
            let gap = a.jd_tt.as_f64() - b.jd_tt.as_f64();
            // 伏日：金星下合可短至数日；火星远日合时星等暗、相对太阳移动慢，缺省消光（k ≈ 0.35）下可达 8 个月
            assert!(gap > 1.0 && gap < 270.0, "{p:?} 合 {} 伏 {gap:.1} 日", jd_to_ymd(t));
            for e in [a, b] {
                assert!(e.sun_altitude_deg.as_f64() < -0.8 && e.sun_altitude_deg.as_f64() >= -18.0);
                assert!(e.altitude_deg.as_f64() > 0.0 && e.magnitude <= e.limiting_magnitude);
            }
            checked += 1;
        }
        assert!(checked >= 1, "{p:?} 无可检合日");
    }
    tab.write();
}

