//! 史书天文志月、五星「犯、入、乘、守、掩、食、在、合」记录的核验：《汉书》《后汉书》《晋书》
//! （data/phenomena/{hanshu,houhanshu,jinshu}_star_records.csv，由 scripts/calendar_data/qinhan_eval/*_star_records.py
//! 只按历法换算日期、按陈卓体系匹配星官）。逐条结果写 data/phenomena/checks/<书>_star_records.csv（PHENOMENA_CHECKS=1 时）。
//!
//! 每条记录：在记载日期范围（单日，或只记到月、旬时的整月、整旬）内逐夜推算，取「天黑（太阳 < −6°）且所涉天体都在地平上」
//! 时段内的最小角距（月取站心）：
//! - 单天体对星官：天体与星官各成员星（合成星官取各垣成员合集；积尸取 M44）的最小角距；
//! - 天体对天体（「月犯岁星」）或多天体相合（「金、木合於东井」）：天体两两最大角距，另给各天体到目标星官的距离；
//! - 「在某宿若干度」另给推算入宿度。
//! 一夜自当地正午起计（入夜那天的黄昏至次晨记为同日，见 doc/astronomy/celestial-phenomena.md）。

use super::chinese::*;
use crate::astronomy::apparent::{body_apparent_equatorial, horizontal, star_apparent_equatorial, topocentric, ApparentPipelineOptions};
use crate::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
use crate::astronomy::ephemeris::vsop87_patch::PatchTarget;
use crate::astronomy::ephemeris::{load_earth_vsop87_corrected_from_repo, load_planet_vsop87_corrected_from_repo};
use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
use crate::astronomy::geodesy::Observer;
use crate::astronomy::phenomena::track::Track;
use crate::astronomy::pipeline::{Body, Planet, Vsop87Elp, Vsop87Planets};
use crate::astronomy::stars::{load_hipparcos_bright_from_repo, CatalogStar, StarCatalog};
use crate::astronomy::time::delta_t::{DeltaTModel, MorrisonStephenson2021};
use crate::astronomy::time::earth_rotation::gast;
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, Real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::epoch::Epoch;
use crate::quantity::length::Length;
use crate::quantity::position::Position;
use crate::quantity::reference_frame::ReferenceFrame;
use crate::quantity::unit::LengthUnit;
use std::collections::HashMap;

fn body_of(name: &str) -> Option<Body> {
    Some(match name {
        "Moon" => Body::Moon,
        "Mercury" => Body::Planet(Planet::Mercury),
        "Venus" => Body::Planet(Planet::Venus),
        "Mars" => Body::Planet(Planet::Mars),
        "Jupiter" => Body::Planet(Planet::Jupiter),
        "Saturn" => Body::Planet(Planet::Saturn),
        _ => return None,
    })
}

fn cn(b: &str) -> &str {
    match b {
        "Moon" => "月",
        "Mercury" => "辰星",
        "Venus" => "太白",
        "Mars" => "荧惑",
        "Jupiter" => "岁星",
        "Saturn" => "填星",
        o => o,
    }
}

fn unit(v: [Real; 3]) -> [f64; 3] {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [(v[0] / n).as_f64(), (v[1] / n).as_f64(), (v[2] / n).as_f64()]
}

fn sep_deg(a: [f64; 3], b: [f64; 3]) -> f64 {
    let c = [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt().atan2(a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).to_degrees()
}

/// M44（鬼宿积尸气）以 0 号代表：J2000 中心 RA 8h40m24s、Dec +19°40′（SIMBAD），自行略去。
const M44_ID: u32 = 0;

fn m44() -> CatalogStar {
    CatalogStar {
        hip: M44_ID,
        ra_deg: real(130.1),
        dec_deg: real(19.6667),
        parallax_mas: real(0.0),
        pm_ra_mas_yr: real(0.0),
        pm_dec_mas_yr: real(0.0),
        vmag: real(3.7),
        bv: None,
    }
}

/// 限定语指明的具体星：心大星 α Sco、心前星 σ Sco、心后星 τ Sco、轩辕大星 α Leo、房北星 β¹ Sco、舆鬼质星（积尸气）M44。
/// 房第二星取 δ Sco（自北数第二）：建武八年四月辛未「月犯房第二星」当夜月距 δ Sco 0.17°，距清代「房宿二」ρ Sco 6.3°。
fn specific_star(target: &str, qualifier: &str) -> Option<(u32, &'static str)> {
    match (target, qualifier) {
        ("心", q) if q.starts_with("大星") || q.starts_with("中央大星") => Some((80763, "心大星 α Sco")),
        ("心", q) if q.starts_with("前星") => Some((80112, "心前星 σ Sco")),
        ("心", q) if q.starts_with("后星") => Some((81266, "心后星 τ Sco")),
        ("轩辕", q) if q.starts_with("大星") => Some((49669, "轩辕大星 α Leo")),
        ("房", q) if q.starts_with("第二星") => Some((78401, "房第二星 δ Sco")),
        ("房", q) if q.starts_with("北星") => Some((78820, "房北星 β¹ Sco")),
        ("舆鬼", q) if q.starts_with("质星") || q.starts_with("积尸") => Some((M44_ID, "舆鬼质星 M44")),
        ("积尸", _) | ("积尸（舆鬼）", _) => Some((M44_ID, "积尸 M44")),
        // 太微垣：清代星名「太微右垣一 右执法」β Vir、「太微左垣一 左执法」η Vir、「太微右垣二 西上将」σ Leo、「太微左垣五 东上将」
        ("右执法", _) => Some((57757, "右执法 β Vir")),
        ("左执法", _) => Some((60129, "左执法 η Vir")),
        ("西蕃上将", _) | ("西藩上将", _) => Some((55434, "西上将 σ Leo")),
        ("东蕃上将", _) | ("东藩上将", _) => Some((64241, "东上将 α Com")),
        _ => None,
    }
}

/// 都城表 `data/phenomena/capitals.csv`（与日月食表生成脚本共用）：(政权, 起年, 止年, 都城, 东经°, 北纬°)。
fn capitals() -> &'static [(String, Option<i64>, Option<i64>, String, f64, f64)] {
    static T: std::sync::OnceLock<Vec<(String, Option<i64>, Option<i64>, String, f64, f64)>> = std::sync::OnceLock::new();
    T.get_or_init(|| {
        let lines = crate::repo::read_lines("data/phenomena/capitals.csv").unwrap();
        lines
            .iter()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .skip(1)
            .map(|l| {
                let f: Vec<&str> = l.split(',').collect();
                let y = |s: &str| if s.is_empty() { None } else { Some(s.parse::<i64>().unwrap()) };
                (f[0].to_string(), y(f[1]), y(f[2]), f[3].to_string(), f[4].parse().unwrap(), f[5].parse().unwrap())
            })
            .collect()
    })
}

/// 都城（观测地）：按政权与年份查都城表；政权未知返回 None。
fn capital(regime: &str, year: i64) -> Option<(f64, f64, &'static str)> {
    capitals()
        .iter()
        .find(|(r, a, b, ..)| r == regime && a.is_none_or(|a| year >= a) && b.is_none_or(|b| year <= b))
        .map(|(_, _, _, name, lon, lat)| (*lon, *lat, name.as_str()))
}

/// 观测地：记录带政权（regime 列）者按 [capital]；无 regime 列者《汉书》按西汉、《后汉书》按东汉。
fn site(book: &str, year: i64, regime: &str) -> (f64, f64, &'static str) {
    let regime = if regime.is_empty() { if book == "hanshu" { "西汉" } else { "东汉" } } else { regime };
    capital(regime, year).unwrap_or_else(|| panic!("都城表 data/phenomena/capitals.csv 缺政权「{regime}」{year} 年"))
}

/// 宿度体系：（距星体系 id，是否黄道，周天度数）。清时宪历（顺治二年 1645 起）以黄经计宿度、周天 360°，距星 1754 年前
/// 取《灵台仪象志》（1645—1673 年《崇祯历书》距星未单列，以之近似），乾隆十九年（1754）参改 ζ Ori 后取《仪象考成》；
/// 其余各书陈卓距星、赤经、周天 365.25 度。
fn lodge_model(book: &str, year: i64) -> (&'static str, bool, f64) {
    match book {
        "qingshigao" if year >= 1754 => ("qing", true, 360.0),
        "qingshigao" if year >= 1645 => ("qing_lingtai", true, 360.0),
        _ => ("chenzhuo", false, 365.25),
    }
}

struct Rec {
    f: HashMap<String, String>,
}

impl Rec {
    fn get(&self, k: &str) -> &str {
        self.f.get(k).map(|s| s.as_str()).unwrap_or("")
    }
}

fn read_records(path: &str) -> Vec<Rec> {
    let lines = crate::repo::read_lines(path).unwrap();
    let header: Vec<String> = lines[0].split(',').map(|s| s.to_string()).collect();
    lines[1..]
        .iter()
        .map(|l| Rec { f: header.iter().cloned().zip(l.split(',').map(|s| s.to_string())).collect() })
        .collect()
}

/// 一夜（自 `night_tt` 起 1 日）中「天黑且所涉天体都在地平上」时段的最小量；无此时段返回 None。
/// `metric(t, 各天体站心单位向量)` 给出该时刻要最小化的量。
#[allow(clippy::too_many_arguments)]
fn night_min(
    night_tt: f64,
    dt_days: f64,
    obs: &Observer,
    sun: &Track,
    tracks: &[Track],
    metric: &dyn Fn(&[[f64; 3]]) -> f64,
    require_visible: bool,
) -> Option<f64> {
    let mut best: Option<f64> = None;
    for k in 0..144 {
        let t = real(night_tt + k as f64 / 144.0);
        let g = gast(t - real(dt_days), t);
        let to_topo = |v: [Real; 3]| {
            let p = Position::from_si_meters_in_frame(ReferenceFrame::TrueEquator(Epoch::new(t)), v[0], v[1], v[2]);
            topocentric(p, obs, g)
        };
        let alt = |p: &Position| horizontal(p, obs, g).altitude.rad().as_f64().to_degrees();
        if require_visible && alt(&to_topo(sun.at(t))) >= -6.0 {
            continue;
        }
        let ps: Vec<Position> = tracks.iter().map(|tr| to_topo(tr.at(t))).collect();
        if require_visible && ps.iter().any(|p| alt(p) <= 0.0) {
            continue;
        }
        let us: Vec<[f64; 3]> = ps.iter().map(|p| unit(p.to_meters())).collect();
        let m = metric(&us);
        if best.is_none_or(|b| m < b) {
            best = Some(m);
        }
    }
    best
}

fn check_book(book: &'static str, title: &str) {
    let _ = try_init_full_nutation_from_repo();
    let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
    let loaded: Vec<_> = Planet::ALL.iter().map(|p| (*p, load_planet_vsop87_corrected_from_repo(*p, PatchTarget::De406).unwrap())).collect();
    let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DE406).unwrap();
    let eph = Vsop87Elp { planets: Vsop87Planets { earth: &earth, planets: loaded.iter().map(|(p, v)| (*p, v)).collect() }, moon: &elp };
    let cat: StarCatalog = load_hipparcos_bright_from_repo().unwrap();
    let asterisms: HashMap<String, Asterism> =
        load_asterisms_from_repo().unwrap().into_iter().filter(|a| a.system == "chenzhuo").map(|a| (a.id.clone(), a)).collect();
    let systems = load_lunar_mansion_systems_from_repo().unwrap();
    let chenzhuo = systems.iter().find(|s| s.id == "chenzhuo").unwrap();
    let recs = read_records(&format!("data/phenomena/{book}_star_records.csv"));
    let opts = ApparentPipelineOptions::default();
    let tp = |t: f64| TimePoint::new(TimeScale::TT, real(t));
    let m44s = m44();

    let mut tab = crate::test_util::CheckTable::new(
        Box::leak(format!("{book}_star_records").into_boxed_str()),
        &[
            "序", "纪年日期", "公历(儒略)", "干支在月", "记载", "观测地", "推算夜数", "天体间最大角距°", "距星官最小角距°", "推算所在宿", "最近成员 HIP", "具体星", "具体星角距°",
            "月最近之日偏移", "该日角距°", "推算入宿", "原文",
        ],
    );
    tab.meta(format!("{title}月、五星凌犯与相合记录逐条推算（VSOP87 + ELPMPP02，ΔT Morrison–Stephenson 2021，星官取陈卓体系；"));
    tab.meta(format!(
        "天黑即太阳 < −6° 且所涉天体都在地平上；日界取当地{}；只记到月或旬者在整月、整旬内逐夜取最小；月另在 ±3 日内求最接近之日）。",
        if std::env::var("PHENOMENA_NIGHT").as_deref() == Ok("midnight") { "子夜（民用日）" } else { "正午（一夜记为入夜之日）" }
    ));
    tab.meta(format!("解析见 data/phenomena/{book}_star_records.csv；「—」表示该日期范围内无可见时段或无可比对象"));

    /// 一条记录的推算结果：表格行与统计量（各记录独立，可并行求，再按原序汇总）。
    #[derive(Default)]
    struct RecOut {
        row: Vec<String>,
        moon_day: Option<f64>,
        planet_day: Option<f64>,
        multi: Option<f64>,
        baseline: Vec<f64>,
        moon_offset: Option<i32>,
        lodge: Option<(bool, bool)>, // (同宿, 邻宿)
    }
    let process = |ri: usize, r: &Rec| -> Option<RecOut> {
        let mut out = RecOut::default();
        let bodies: Vec<Body> = r.get("bodies").split_whitespace().filter_map(body_of).collect();
        if bodies.is_empty() {
            return None;
        }
        let (j0, j1): (f64, f64) = (r.get("jdn_start").parse().unwrap(), r.get("jdn_end").parse().unwrap());
        let year: i64 = r.get("year").parse().unwrap_or(0);
        let (lon, lat, place) = site(book, year, r.get("regime"));
        let obs = Observer::new(PlaneAngle::from_deg(real(lon)), PlaneAngle::from_deg(real(lat)), Length::from_value(real(100.0), LengthUnit::Meter));
        let mid_ut0 = j0 - 0.5 - lon / 360.0;
        let dt_days = MorrisonStephenson2021.estimate(real(mid_ut0)).value.seconds().as_f64() / 86400.0;
        // 日界：缺省正午（入夜那天的黄昏至次晨记为同日）；PHENOMENA_NIGHT=midnight 改为子夜（民用日），供比较各书记日约定
        let day_start = if std::env::var("PHENOMENA_NIGHT").as_deref() == Ok("midnight") { 0.0 } else { 0.5 };
        let first_night = mid_ut0 + dt_days + day_start;
        let nights = (j1 - j0) as usize + 1;
        let single_day = nights == 1;
        let (t0, t1) = (real(first_night - 4.0), real(first_night + nights as f64 + 4.0));
        let sun = Track::build(t0, t1, |t| body_apparent_equatorial(&eph, Body::Sun, TimePoint::new(TimeScale::TT, t), &opts).to_meters());
        let mut tracks: Vec<Track> = bodies
            .iter()
            .map(|b| Track::build(t0, t1, |t| body_apparent_equatorial(&eph, *b, TimePoint::new(TimeScale::TT, t), &opts).to_meters()))
            .collect();
        // 目标：星官成员（合成星官「+」取合集），或天体
        let kind = r.get("target_kind");
        let tid = r.get("target_id");
        let mut target_body = false;
        if kind == "body" {
            if let Some(tb) = body_of(tid) {
                tracks.push(Track::build(t0, t1, |t| body_apparent_equatorial(&eph, tb, TimePoint::new(TimeScale::TT, t), &opts).to_meters()));
                target_body = true;
            }
        }
        let n_bodies = bodies.len() + target_body as usize;
        let mut member_hips: Vec<u32> = vec![];
        if kind == "asterism" {
            for id in tid.split('+') {
                if let Some(a) = asterisms.get(id) {
                    member_hips.extend(a.member_hips());
                    if a.members.iter().any(|m| matches!(m, AsterismMember::Other(s) if s == "DSO:M44")) {
                        member_hips.push(M44_ID);
                    }
                }
            }
        }
        let spec = specific_star(r.get("target_name"), r.get("qualifier"));
        let star_dir = |h: u32, t: f64| {
            let st = if h == M44_ID { Some(&m44s) } else { cat.by_hip(h) };
            st.map(|st| unit(star_apparent_equatorial(&eph, st, tp(t), &opts).to_meters()))
        };
        let members: Vec<(u32, [f64; 3])> = member_hips.iter().filter_map(|h| star_dir(*h, first_night).map(|u| (*h, u))).collect();
        let spec_dir = spec.and_then(|(h, _)| star_dir(h, first_night));
        let nb = bodies.len();
        // 各量的逐夜最小
        let pair_max = |us: &[[f64; 3]]| {
            let mut m: f64 = 0.0;
            for i in 0..us.len() {
                for j in i + 1..us.len() {
                    m = m.max(sep_deg(us[i], us[j]));
                }
            }
            m
        };
        let to_ast = |us: &[[f64; 3]]| {
            // 各天体到星官最近成员的距离，取最大（多天体时要求都在其旁）
            us[..nb].iter().map(|u| members.iter().map(|(_, s)| sep_deg(*u, *s)).fold(f64::MAX, f64::min)).fold(0.0, f64::max)
        };
        let nearest = |us: &[[f64; 3]]| {
            members.iter().min_by(|a, b| sep_deg(us[0], a.1).partial_cmp(&sep_deg(us[0], b.1)).unwrap()).map(|m| m.0).unwrap_or(0)
        };
        let (mut best_pair, mut best_ast, mut best_spec) = (None::<f64>, None::<f64>, None::<f64>);
        let mut near_hip = 0u32;
        let mut pair_night = first_night;
        for n in 0..nights {
            let night = first_night + n as f64;
            if n_bodies >= 2 {
                if let Some(v) = night_min(night, dt_days, &obs, &sun, &tracks, &pair_max, true) {
                    if best_pair.is_none_or(|b| v < b) {
                        best_pair = Some(v);
                        pair_night = night;
                    }
                }
            }
            if !members.is_empty() {
                if let Some(v) = night_min(night, dt_days, &obs, &sun, &tracks[..nb], &to_ast, true) {
                    if best_ast.is_none_or(|b| v < b) {
                        best_ast = Some(v);
                        // 最近成员（首个天体，取当夜中点地心方向近似；月的视差可达 1°，只作指认参考）
                        near_hip = nearest(&[unit(tracks[0].at(real(night + 0.5)))]);
                    }
                }
            }
            if let Some(sd) = spec_dir {
                if let Some(v) = night_min(night, dt_days, &obs, &sun, &tracks[..1], &|us: &[[f64; 3]]| sep_deg(us[0], sd), true) {
                    best_spec = Some(best_spec.map_or(v, |b: f64| b.min(v)));
                }
            }
        }
        // 天黑时段内不可见（近日或在地平下）：给出不论可见与否的最小天体间角距，注明
        let mut invisible = false;
        if n_bodies >= 2 && best_pair.is_none() {
            for n in 0..nights {
                let night = first_night + n as f64;
                if let Some(v) = night_min(night, dt_days, &obs, &sun, &tracks, &pair_max, false) {
                    if best_pair.is_none_or(|b| v < b) {
                        best_pair = Some(v);
                        pair_night = night;
                    }
                }
            }
            invisible = best_pair.is_some();
        }
        // 推算所在宿（陈卓体系，赤经区间）：多天体取天体间最接近之夜，其余取首夜，均取夜半；记载目标为二十八宿时比对
        let (lodge_sys, ecliptic, zhou_tian) = lodge_model(book, year);
        let lodge_sys = systems.iter().find(|s| s.id == lodge_sys).unwrap();
        let lodge_of = |night: f64| {
            let t = real(night + 0.5);
            let frame = MansionFrame::new(&eph, &cat, lodge_sys, TimePoint::new(TimeScale::TT, t), &opts).unwrap();
            let v = tracks[0].at(t);
            let (a, dcl) = crate::astronomy::apparent::right_ascension_declination(&Position::from_si_meters_in_frame(
                ReferenceFrame::TrueEquator(Epoch::new(t)),
                v[0],
                v[1],
                v[2],
            ));
            if ecliptic {
                // 平黄赤交角（P03）；章动项 <10″，于度级比较可略
                let eps = crate::astronomy::frame::precession::mean_obliquity((t - real(2451545.0)) / real(36525.0)).rad();
                frame.to_ecliptic(eps).locate(ecliptic_longitude(a.rad(), dcl.rad(), eps), dcl.rad(), real(zhou_tian))
            } else {
                frame.locate(a.rad(), dcl.rad(), real(zhou_tian))
            }
        };
        // 记载所在宿：目标为二十八宿者；否则「犯歲星於昴」之「於某宿」（lodge 列有宿无度）
        let rec_lodge = if kind == "asterism" && tid.len() == 3 && tid.ends_with('A') {
            tid[..2].parse::<usize>().ok().map(|i| i - 1)
        } else if r.get("lodge_deg").is_empty() && !r.get("lodge").is_empty() {
            MANSIONS.iter().position(|m| *m == r.get("lodge"))
        } else {
            None
        };
        let lodge_cell = rec_lodge.map(|li| {
            let loc = lodge_of(if n_bodies >= 2 { pair_night } else { first_night });
            let d = (loc.mansion as i32 - li as i32).rem_euclid(28);
            let verdict = match d {
                0 => "同宿",
                1 | 27 => "邻宿",
                _ => "不合",
            };
            format!("{}{:.1}（{verdict}）", MANSIONS[loc.mansion], loc.ru_xiu_du.as_f64())
        });
        if let Some(c) = &lodge_cell {
            out.lodge = Some((c.contains("同宿"), c.contains("邻宿")));
        }
        // 月亮单日记录：±3 日内最接近之日（对星官或天体目标）
        let (mut best_off, mut best_off_sep) = (None::<i32>, None::<f64>);
        if single_day && bodies.len() == 1 && bodies[0] == Body::Moon {
            for d in -3..=3 {
                let night = first_night + d as f64;
                let v = if target_body {
                    night_min(night, dt_days, &obs, &sun, &tracks, &pair_max, true)
                } else if !members.is_empty() {
                    night_min(night, dt_days, &obs, &sun, &tracks[..1], &to_ast, true)
                } else {
                    None
                };
                if let Some(v) = v {
                    if best_off_sep.is_none_or(|b| v < b) {
                        best_off = Some(d);
                        best_off_sep = Some(v);
                    }
                }
            }
            out.moon_offset = best_off;
        }
        // 统计用：记载当日（或范围内）的主要角距
        let main = if n_bodies >= 2 { best_pair } else { best_ast };
        if let Some(v) = main {
            if n_bodies >= 3 || (n_bodies == 2 && bodies.len() >= 2) {
                out.multi = Some(v);
            } else if bodies[0] == Body::Moon {
                if single_day {
                    out.moon_day = Some(v);
                }
            } else if single_day {
                out.planet_day = Some(v);
                // 随机日期基线：错开 ±60—240 日，单天体对星官
                if !members.is_empty() && !target_body {
                    for d in [-240.0, -180.0, -120.0, -60.0, 60.0, 120.0, 180.0, 240.0] {
                        let bt = Track::build(real(first_night + d - 1.0), real(first_night + d + 2.0), |t| {
                            body_apparent_equatorial(&eph, bodies[0], TimePoint::new(TimeScale::TT, t), &opts).to_meters()
                        });
                        let bu = unit(bt.at(real(first_night + d + 0.5)));
                        out.baseline.push(members.iter().map(|(_, u)| sep_deg(bu, *u)).fold(f64::MAX, f64::min));
                    }
                }
            }
        }
        // 入宿度
        let mut ruxiu = String::new();
        if !r.get("lodge").is_empty() {
            let loc = lodge_of(first_night);
            ruxiu = format!("{}{:.2}（史载 {}{}）", MANSIONS[loc.mansion], loc.ru_xiu_du.as_f64(), r.get("lodge"), r.get("lodge_deg"));
        }
        let fmt = |x: Option<f64>| x.map(|x| format!("{x:.2}")).unwrap_or("—".into());
        let body_names: Vec<&str> = r.get("bodies").split_whitespace().map(cn).collect();
        let tgt = if kind == "body" { cn(tid).to_string() } else { r.get("target_name").to_string() };
        let era = r.get("era");
        let reign = if era.is_empty() || era == "中" || era == "後" { r.get("reign") } else { "" };
        let month = r.get("month").replace("leap", "闰");
        let month = match month.find("旬") {
            Some(_) => format!("{}月{}", &month[..month.len() - "上旬".len()], &month[month.len() - "上旬".len()..]),
            None => format!("{month}月"),
        };
        let date = format!("{reign}{era}{}年{month}{}", r.get("year_in_era"), r.get("ganzhi"));
        out.row = vec![
            (ri + 1).to_string(),
            date,
            r.get("julian").into(),
            if r.get("in_month") == "1" { "是".into() } else { "否".into() },
            format!("{}{}{}{}", body_names.join("、"), r.get("verb"), tgt, r.get("qualifier")),
            place.into(),
            nights.to_string(),
            if n_bodies >= 2 { format!("{}{}", fmt(best_pair), if invisible { "（不可见）" } else { "" }) } else { String::new() },
            if members.is_empty() { String::new() } else { fmt(best_ast) },
            lodge_cell.unwrap_or_default(),
            if near_hip == 0 { String::new() } else { near_hip.to_string() },
            spec.map(|s| s.1.to_string()).unwrap_or_default(),
            if spec.is_some() { fmt(best_spec) } else { String::new() },
            best_off.map(|o| format!("{o:+}")).unwrap_or_default(),
            if best_off.is_some() { fmt(best_off_sep) } else { String::new() },
            ruxiu,
            r.get("text").into(),
        ];
        Some(out)
    };
    let (mut moon_day, mut planet_day, mut multi) = (vec![], vec![], vec![]);
    let mut planet_baseline: Vec<f64> = vec![];
    let mut moon_offsets: Vec<i32> = vec![];
    let mut lodge_hits = (0usize, 0usize, 0usize); // 同宿、邻宿、总数
    for out in crate::test_util::par_map(&recs, process).into_iter().flatten() {
        moon_day.extend(out.moon_day);
        planet_day.extend(out.planet_day);
        multi.extend(out.multi);
        planet_baseline.extend(out.baseline);
        moon_offsets.extend(out.moon_offset);
        if let Some((same, adj)) = out.lodge {
            lodge_hits.0 += same as usize;
            lodge_hits.1 += adj as usize;
            lodge_hits.2 += 1;
        }
        tab.row(out.row);
    }
    tab.write();
    let pct = |v: &[f64], x: f64| 100.0 * v.iter().filter(|s| **s <= x).count() as f64 / v.len().max(1) as f64;
    let median = |v: &[f64]| {
        let mut v = v.to_vec();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v.get(v.len() / 2).copied().unwrap_or(f64::NAN)
    };
    let hist: Vec<(i32, usize)> = (-3..=3).map(|d| (d, moon_offsets.iter().filter(|x| **x == d).count())).collect();
    println!("{title}：{} 条", recs.len());
    println!("  月（单日）{} 条：≤1° {:.0}%、≤2° {:.0}%，中位 {:.2}°；最接近之日偏移 {hist:?}", moon_day.len(), pct(&moon_day, 1.0), pct(&moon_day, 2.0), median(&moon_day));
    println!("  行星（单日）{} 条：≤1° {:.0}%、≤2° {:.0}%，中位 {:.2}°", planet_day.len(), pct(&planet_day, 1.0), pct(&planet_day, 2.0), median(&planet_day));
    println!("  行星随机日期基线 {} 个：≤1° {:.0}%、≤2° {:.0}%，中位 {:.2}°", planet_baseline.len(), pct(&planet_baseline, 1.0), pct(&planet_baseline, 2.0), median(&planet_baseline));
    println!("  推算所在宿（记载目标为二十八宿者）{} 条：同宿 {}、邻宿 {}", lodge_hits.2, lodge_hits.0, lodge_hits.1);
    println!("  多天体相合 {} 条（天体间最大角距）：≤1° {:.0}%、≤3° {:.0}%，中位 {:.2}°", multi.len(), pct(&multi, 1.0), pct(&multi, 3.0), median(&multi));
}

#[test]
#[ignore]
fn houhanshu_star_records_check() {
    check_book("houhanshu", "《后汉书·天文志》");
}

#[test]
#[ignore]
fn hanshu_star_records_check() {
    check_book("hanshu", "《汉书·天文志》");
}

#[test]
#[ignore]
fn jinshu_star_records_check() {
    check_book("jinshu", "《晋书·天文志》");
}

#[test]
#[ignore]
fn songshu_star_records_check() {
    check_book("songshu", "《宋书·天文志》");
}

#[test]
#[ignore]
fn nanqishu_star_records_check() {
    check_book("nanqishu", "《南齐书·天文志》");
}

#[test]
#[ignore]
fn weishu_star_records_check() {
    check_book("weishu", "《魏书·天象志》");
}

#[test]
#[ignore]
fn suishu_star_records_check() {
    check_book("suishu", "《隋书·天文志》");
}

#[test]
#[ignore]
fn xintangshu_star_records_check() {
    check_book("xintangshu", "《新唐书·天文志》");
}

#[test]
#[ignore]
fn jiutangshu_star_records_check() {
    check_book("jiutangshu", "《旧唐书·天文志》");
}

#[test]
#[ignore]
fn jiuwudaishi_star_records_check() {
    check_book("jiuwudaishi", "《旧五代史·天文志》");
}

#[test]
#[ignore]
fn xinwudaishi_star_records_check() {
    check_book("xinwudaishi", "《新五代史·司天考》");
}

#[test]
#[ignore]
fn songshi_star_records_check() {
    check_book("songshi", "《宋史·天文志》");
}

#[test]
#[ignore]
fn jinshi_star_records_check() {
    check_book("jinshi", "《金史·天文志》");
}

#[test]
#[ignore]
fn yuanshi_star_records_check() {
    check_book("yuanshi", "《元史·天文志》");
}

#[test]
#[ignore]
fn mingshi_star_records_check() {
    check_book("mingshi", "《明史·天文志》");
}

#[test]
#[ignore]
fn qingshigao_star_records_check() {
    check_book("qingshigao", "《清史稿·天文志》");
}
