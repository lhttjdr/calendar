//! 史书日月食记录检验：逐条计算记录日附近的日食或月食，以及在都城所见。
//!
//! 输入由 scripts/calendar_data/qinhan_eval/ 下的脚本生成（记录日的儒略日数只由历法换算得出），格式与输出列见
//! data/phenomena/README.md。ΔT 取 Morrison–Stephenson 2021；与月历表 ELPMPP02 的配套修正可选（第 6 个参数）：
//! `offset`（缺省，按 ELPMPP02 − DE431 实测月亮黄经差，`delta_t::lunar_longitude_offset`）或 `ndot`（按 ṅ = −25.857 的二次修正，
//! 2026-10-26 前的做法）。都城所见在 ΔT 及 ±σ、±2σ 下各算一次。
//!
//! 运行（仓库根下）：缺省用 f64 构建（2026-10-28 对《汉书》《后汉书》《清史稿》四表，f64 与 TwoFloat 在输出精度下逐列全同，
//! f64 约快十倍，见 doc/engineering/project-and-implementation.md §11.2.2）；需对照时把 real-f64 换成 twofloat。
//!   cargo run --release -p lunar-core --example historical_eclipses --no-default-features --features real-f64 -- \
//!     <solar|lunar> <输入 csv> <输出 csv> <东经°> <北纬°> [offset|ndot]
//! 输入有 site_lon、site_lat 列者逐行取观测地（命令行经纬度为缺省）。
//! 例：《汉书》日食在长安：solar data/phenomena/hanshu_solar_eclipses.csv data/phenomena/hanshu_solar_eclipses_check.csv 108.88 34.30
//!     《后汉书》在洛阳：solar|lunar data/phenomena/houhanshu_{solar,lunar}_eclipses.csv … 112.62 34.73

use lunar_core::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
use lunar_core::astronomy::ephemeris::load_earth_vsop87_corrected_from_repo;
use lunar_core::astronomy::ephemeris::vsop87_patch::PatchTarget;
use lunar_core::astronomy::frame::nutation::try_init_full_nutation_from_repo;
use lunar_core::astronomy::geodesy::Observer;
use lunar_core::astronomy::phenomena::{
    eclipse_window, lunar_eclipse_geometry, lunar_eclipse_in, moon_altitude_in, solar_eclipse_global_in, solar_eclipse_local_in,
    EclipseEphemeris, InterpolatedSunMoon, LunarEclipseKind, SolarEclipseKind,
};
use lunar_core::astronomy::time::delta_t::{
    delta_t_for_ephemeris, delta_t_for_lunar_offset, lunar_longitude_offset, lunar_tidal_acceleration, MorrisonStephenson2021,
};
use lunar_core::math::real::{real, Real, RealOps};
use lunar_core::quantity::angle::PlaneAngle;
use lunar_core::quantity::duration::Duration;
use lunar_core::quantity::length::Length;
use lunar_core::quantity::unit::LengthUnit;

/// ΔT 偏移（以 σ 计）。
const SIGMAS: [f64; 5] = [-2.0, -1.0, 0.0, 1.0, 2.0];
/// 月亮上缘可见的站心几何高度阈值：大气折射 34′ + 月亮视半径约 15.5′。
const MOON_HORIZON_DEG: f64 = -0.825;

fn deg(d: f64) -> PlaneAngle {
    PlaneAngle::from_rad(real(d.to_radians()))
}

struct Row {
    cols: Vec<String>,
}

impl Row {
    fn get(&self, header: &[String], name: &str) -> String {
        header.iter().position(|h| h == name).map(|i| self.cols[i].clone()).unwrap_or_default()
    }
}

/// ΔT 与月历表的配套方式：true 为按 ṅ 二次修正（旧），false 为按实测黄经差（缺省）。
static PAIRING_NDOT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

/// ΔT 中值与 σ（秒）。
fn delta_t(jd_tt: Real) -> (f64, f64) {
    let dt = if *PAIRING_NDOT.get().unwrap_or(&false) {
        delta_t_for_ephemeris(&MorrisonStephenson2021, jd_tt, real(lunar_tidal_acceleration::ELPMPP02_DE405))
    } else {
        delta_t_for_lunar_offset(&MorrisonStephenson2021, jd_tt, &lunar_longitude_offset::ELPMPP02_DE406_MINUS_DE431)
    };
    (dt.value.seconds().as_f64(), dt.sigma.map(|s| s.seconds().as_f64()).unwrap_or(0.0))
}

/// 某地以正午为日界的「夜日」（儒略日数）：当地正午至次日正午记为前一日（史书按夜记月食之日）。
fn night_day(jd_tt: f64, dt: f64, lon: f64) -> i64 {
    (jd_tt - dt / 86400.0 + lon / 360.0).floor() as i64
}

/// 某地地方平太阳时的日期（儒略日数）。
fn civil_day(jd_tt: f64, dt: f64, lon: f64) -> i64 {
    (jd_tt - dt / 86400.0 + lon / 360.0 + 0.5).floor() as i64
}

fn verdict(mags: &[f64], site: &str) -> String {
    match mags.iter().filter(|m| **m > 0.0).count() {
        0 => format!("{site}不可见"),
        n if n == mags.len() => format!("{site}可见"),
        _ => format!("{site}可见与否取决于ΔT"),
    }
}

fn solar(w: &InterpolatedSunMoon, row: &Row, header: &[String], observer: &Observer, lon: f64) -> (String, String) {
    let jdn: f64 = row.get(header, "jdn").parse().unwrap();
    let Some(g) = solar_eclipse_global_in(w, real(jdn)) else {
        return ("无日食".into(), ",,,,,,,,,,,,,无日食,,,,".into());
    };
    let (dt0, sigma) = delta_t(g.greatest_tt);
    let day_offset = civil_day(g.greatest_tt.as_f64(), dt0, lon) - jdn as i64;
    let mut mags = vec![];
    let mut alt0 = f64::NAN;
    let (mut lmt0, mut geo0, mut lmt1, mut lmt4) = (f64::NAN, 0.0, f64::NAN, f64::NAN);
    for k in SIGMAS {
        let local = solar_eclipse_local_in(w, observer, g.greatest_tt, Duration::in_seconds(real(dt0 + k * sigma)));
        mags.push(local.as_ref().and_then(|l| l.max_visible_magnitude).map(|m: Real| m.as_f64()).unwrap_or(0.0));
        if k == 0.0 {
            alt0 = local.as_ref().map(|l| l.sun_altitude_greatest.rad().as_f64().to_degrees()).unwrap_or(f64::NAN);
            // 本地食甚、初亏、复圆的地方平太阳时（时）与几何食分（不论太阳是否在地平以上），供与史载时辰、食分比较
            if let Some(l) = local.as_ref() {
                // 地方平太阳时（时）：TT 儒略日减 ΔT、加经度
                let lmt = |jd: f64| {
                    let ut = jd - dt0 / 86400.0 + lon / 360.0 + 0.5;
                    (ut - ut.floor()) * 24.0
                };
                lmt0 = lmt(l.greatest.as_f64());
                lmt1 = lmt(l.first_contact.as_f64());
                lmt4 = lmt(l.last_contact.as_f64());
                geo0 = l.magnitude.as_f64();
            }
        }
    }
    let kind = match g.kind {
        SolarEclipseKind::Partial => "偏",
        SolarEclipseKind::Annular => "环",
        SolarEclipseKind::Total => "全",
        SolarEclipseKind::Hybrid => "全环",
    };
    let v = verdict(&mags, "都城");
    let msg = format!("{kind}食 γ={:+.3} 日差 {day_offset:+} ΔT={dt0:.0}±{sigma:.0}s 都城食分 {:.2?} {v}", g.gamma.as_f64(), mags);
    let csv = format!(
        "{:.5},{kind},{:.4},{:.4},{day_offset},{dt0:.0},{sigma:.0},{:.3},{:.3},{:.3},{:.3},{:.3},{alt0:.1},{v},{lmt0:.2},{geo0:.3},{lmt1:.2},{lmt4:.2}",
        g.greatest_tt.as_f64(), g.gamma.as_f64(), g.magnitude.as_f64(), mags[0], mags[1], mags[2], mags[3], mags[4]
    );
    (msg, csv)
}

/// 月食在某地可见的最大食分（本影食取本影食分，半影食取半影食分）：在食的起讫之间每 2 分钟取样，月亮上缘在地平以上才计。
fn lunar_visible_magnitude(w: &InterpolatedSunMoon, observer: &Observer, span: (Real, Real), umbral: bool, dt: f64) -> f64 {
    let d = Duration::in_seconds(real(dt));
    let (a, b) = (span.0.as_f64(), span.1.as_f64());
    let step = 2.0 / 1440.0;
    let mut best = 0.0_f64;
    let mut t = a;
    while t <= b {
        let tt = real(t);
        if moon_altitude_in(w, observer, tt, d).rad().as_f64().to_degrees() > MOON_HORIZON_DEG {
            let g = lunar_eclipse_geometry(w, tt);
            let m = if umbral { g.umbral_magnitude() } else { g.penumbral_magnitude() };
            best = best.max(m.as_f64());
        }
        t += step;
    }
    best
}

fn lunar(w: &InterpolatedSunMoon, row: &Row, header: &[String], observer: &Observer, lon: f64) -> (String, String) {
    let guess: f64 = row.get(header, "guess_jdn").parse().unwrap();
    let Some(e) = lunar_eclipse_in(w, real(guess)) else {
        return ("±1.5 日内无月食".into(), ",,,,,,,,,,,,,,,无月食".into());
    };
    let (dt0, sigma) = delta_t(e.greatest_tt);
    // 月食按夜记日：一夜记为入夜之日（日界取当地正午，与凌犯检验同），子夜后的食甚不算次日
    let day = night_day(e.greatest_tt.as_f64(), dt0, lon);
    let day_offset = row.get(header, "jdn").parse::<i64>().map(|j| (day - j).to_string()).unwrap_or_default();
    let umbral = e.partial.is_some();
    let span = e.partial.unwrap_or(e.penumbral);
    let mags: Vec<f64> = SIGMAS.iter().map(|k| lunar_visible_magnitude(w, observer, span, umbral, dt0 + k * sigma)).collect();
    let kind = match e.kind {
        LunarEclipseKind::Penumbral => "半影",
        LunarEclipseKind::Partial => "月偏",
        LunarEclipseKind::Total => "月全",
    };
    let local_hours = |t: Real| (t.as_f64() - dt0 / 86400.0 + lon / 360.0 + 0.5).rem_euclid(1.0) * 24.0;
    let v = verdict(&mags, "都城");
    let msg = format!(
        "{kind}食 本影食分 {:.3} 地方时 {:.2}–{:.2}h 日差 {day_offset} ΔT={dt0:.0}±{sigma:.0}s 都城可见食分 {:.2?} {v}",
        e.umbral_magnitude.as_f64(),
        local_hours(span.0),
        local_hours(span.1),
        mags
    );
    let csv = format!(
        "{:.5},{kind},{:.4},{:.4},{:.4},{:.2},{:.2},{day_offset},{dt0:.0},{sigma:.0},{:.3},{:.3},{:.3},{:.3},{:.3},{v}",
        e.greatest_tt.as_f64(),
        e.gamma.as_f64(),
        e.penumbral_magnitude.as_f64(),
        e.umbral_magnitude.as_f64(),
        local_hours(span.0),
        local_hours(span.1),
        mags[0],
        mags[1],
        mags[2],
        mags[3],
        mags[4]
    );
    (msg, csv)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let usage = "参数：<solar|lunar> <输入 csv> <输出 csv> <东经°> <北纬°> [offset|ndot]";
    assert!(args.len() == 6 || args.len() == 7, "{usage}");
    let pairing = args.get(6).map(String::as_str).unwrap_or("offset");
    assert!(pairing == "offset" || pairing == "ndot", "{usage}");
    PAIRING_NDOT.set(pairing == "ndot").unwrap();
    let (mode, input, output) = (args[1].as_str(), &args[2], &args[3]);
    let (lon, lat): (f64, f64) = (args[4].parse().expect(usage), args[5].parse().expect(usage));
    assert!(mode == "solar" || mode == "lunar", "{usage}");

    let vsop = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).expect("VSOP87 地球数据及 DE406 改正");
    let elp = load_all(&lunar_core::repo::default_loader(), lunar_core::repo::paths::ELPMPP02, Elpmpp02Correction::DE406)
        .expect("ELPMPP02 数据");
    assert!(try_init_full_nutation_from_repo(), "未能加载 IAU 2000A 章动表");
    let eph = EclipseEphemeris::new(&vsop, &elp);
    let observer = Observer::new(deg(lon), deg(lat), Length::from_value(real(150.0), LengthUnit::Meter));

    let text = std::fs::read_to_string(input).unwrap_or_else(|e| panic!("{input}: {e}（工作目录须为仓库根）"));
    let mut lines = text.lines();
    let header_line = lines.next().unwrap();
    let header: Vec<String> = header_line.split(',').map(String::from).collect();
    let extra = if mode == "solar" {
        "greatest_tt_jd,kind,gamma,global_magnitude,day_offset,delta_t_s,sigma_s,site_magnitude_m2s,site_magnitude_m1s,\
         site_magnitude_0,site_magnitude_p1s,site_magnitude_p2s,sun_altitude_greatest_deg,verdict,local_greatest_lmt_h,site_magnitude_geometric,local_first_lmt_h,local_last_lmt_h"
    } else {
        "greatest_tt_jd,kind,gamma,penumbral_magnitude,umbral_magnitude,local_start_h,local_end_h,day_offset,delta_t_s,sigma_s,\
         site_magnitude_m2s,site_magnitude_m1s,site_magnitude_0,site_magnitude_p1s,site_magnitude_p2s,verdict"
    };
    // 输出保留输入的关键列（不含原文），后接计算列
    let keep: Vec<usize> = header.iter().enumerate().filter(|(_, h)| *h != "text" && *h != "note").map(|(i, _)| i).collect();
    let mut out = keep.iter().map(|i| header[*i].as_str()).collect::<Vec<_>>().join(",") + "," + extra + "\n";
    for line in lines {
        let row = Row { cols: line.split(',').map(String::from).collect() };
        let center: f64 = row.get(&header, if mode == "solar" { "jdn" } else { "guess_jdn" }).parse().unwrap();
        let w = eclipse_window(&eph, real(center));
        // 逐行观测地（site_lon、site_lat 列，一书跨数都者由都城表 data/phenomena/capitals.csv 给出）；缺则用命令行经纬度
        let (row_lon, row_lat) = match (row.get(&header, "site_lon").parse::<f64>(), row.get(&header, "site_lat").parse::<f64>()) {
            (Ok(a), Ok(b)) => (a, b),
            _ => (lon, lat),
        };
        let row_observer = if (row_lon, row_lat) == (lon, lat) {
            observer.clone()
        } else {
            Observer::new(deg(row_lon), deg(row_lat), Length::from_value(real(150.0), LengthUnit::Meter))
        };
        let (msg, csv) = if mode == "solar" {
            solar(&w, &row, &header, &row_observer, row_lon)
        } else {
            lunar(&w, &row, &header, &row_observer, row_lon)
        };
        let label: Vec<&str> = keep.iter().take(8).map(|i| row.cols[*i].as_str()).collect();
        println!("{}：{msg}", label.join(" "));
        out += &(keep.iter().map(|i| row.cols[*i].as_str()).collect::<Vec<_>>().join(",") + "," + &csv + "\n");
    }
    std::fs::write(output, out).unwrap();
    println!("→ {output}");
}
