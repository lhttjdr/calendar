//! 《左传》《国语》所载具体天象的检验。
//!
//! - 《左传·僖公五年》卜偃：「丙子旦，日在尾，月在策，鶉火中」，即晋灭虢之「冬十二月丙子朔」（前 655 年 11 月 15 日，儒略历）。
//!   策按杜预注为傅说星（陈卓 06E，HIP 87261）。旦取日出前 2.5 刻，地点取虢（上阳，约东经 111.2°、北纬 34.8°）。
//! - 《国语·周语下》伐纣天象：「歲在鶉火，月在天駟，日在析木之津，辰在斗柄，星在天黿」。按韦昭注：木星在鹑火（柳、星、张），
//!   日月合朔（辰）于斗，水星（星）在玄枵（女、虚、危）；「日在析木之津」即箕斗之间，合朔于斗已大致满足；「月在天驷」（房）每月都有，不作区分。
//!   逐个合朔扫描前 1130 至前 1000 年，列出合朔于斗且木星在柳、星、张者，并给出合朔前后 20 日内水星是否到女、虚、危。
//! 宿按陈卓体系距星当日视赤经划界（赤道宿度）。

use super::chinese::*;
use crate::astronomy::apparent::{body_apparent_equatorial, right_ascension_declination, star_apparent_equatorial, ApparentPipelineOptions};
use crate::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
use crate::astronomy::ephemeris::vsop87_patch::PatchTarget;
use crate::astronomy::ephemeris::{load_earth_vsop87_corrected_from_repo, load_planet_vsop87_corrected_from_repo};
use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
use crate::astronomy::frame::precession::{mean_obliquity_for, PrecessionModel};
use crate::astronomy::pipeline::{Body, Planet, Vsop87Elp, Vsop87Planets};
use crate::astronomy::stars::load_hipparcos_bright_from_repo;
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, RealOps};

fn wrap180(x: f64) -> f64 {
    (x + 540.0).rem_euclid(360.0) - 180.0
}

#[test]
#[ignore]
fn zuozhuan_guoyu_sky() {
    let _ = try_init_full_nutation_from_repo();
    let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::De406).unwrap();
    let loaded: Vec<_> = Planet::ALL.iter().map(|p| (*p, load_planet_vsop87_corrected_from_repo(*p, PatchTarget::De406).unwrap())).collect();
    let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DE406).unwrap();
    let eph = Vsop87Elp { planets: Vsop87Planets { earth: &earth, planets: loaded.iter().map(|(p, v)| (*p, v)).collect() }, moon: &elp };
    let opts = ApparentPipelineOptions::default();
    let cat = load_hipparcos_bright_from_repo().unwrap();
    let sys = load_lunar_mansion_systems_from_repo().unwrap().into_iter().find(|s| s.id == "chenzhuo").unwrap();
    let tp = |jd: f64| TimePoint::new(TimeScale::TT, real(jd));
    let radec = |b: Body, jd: f64| {
        let (a, d) = right_ascension_declination(&body_apparent_equatorial(&eph, b, tp(jd), &opts));
        (a.rad().as_f64().to_degrees().rem_euclid(360.0), d.rad().as_f64())
    };
    let loc = |frame: &MansionFrame, ra_deg: f64| {
        let l = frame.ru_xiu(real(ra_deg.to_radians()), real(365.25));
        (MANSIONS[l.0], l.1.as_f64())
    };
    let lon = |b: Body, jd: f64| {
        let (a, d) = radec(b, jd);
        let (a, d) = (a.to_radians(), d);
        let e = mean_obliquity_for(real((jd - 2451545.0) / 36525.0), PrecessionModel::Vondrak2011).rad().as_f64();
        (a.sin() * e.cos() + d.tan() * e.sin()).atan2(a.cos()).to_degrees()
    };
    let conj = |mut t: f64| {
        for _ in 0..8 {
            t -= wrap180(lon(Body::Moon, t) - lon(Body::Sun, t)) / 12.19;
        }
        t
    };

    // 一、卜偃：前 655 年 11 月 15 日（JDN 1482503）
    let mut tab = crate::test_util::CheckTable::new("zuozhuan_buyan", &["项", "记载", "推算", "说明"]);
    tab.meta("《左传·僖公五年》卜偃「丙子旦，日在尾，月在策，鶉火中」：晋灭虢「冬十二月丙子朔」，前 655 年 11 月 15 日（儒略历）；赤道宿度（陈卓距星）；旦为日出前 2.5 刻，虢地东经 111.2°、北纬 34.8°");
    let jdn = 1482503.0;
    let tc = conj(jdn);
    let lon_e = 111.2f64;
    let lat = 34.8f64.to_radians();
    // 地方时日出：以当日太阳赤纬求日出时角，旦 = 日出 − 36 分钟
    let (a_s, d_s) = radec(Body::Sun, jdn);
    let h0 = ((-(0.8333f64).to_radians().sin() - lat.sin() * d_s.sin()) / (lat.cos() * d_s.cos())).acos().to_degrees();
    let t_dawn_local = 12.0 - h0 / 15.0 - 0.6; // 平太阳时近似（不计时差，误差 < 20 分钟，对宿度判断无碍）
    let jd_dawn = jdn - 0.5 + (t_dawn_local - lon_e / 15.0) / 24.0 + 0.4; // 约加 ΔT（前 7 世纪约 6 小时内取 0.4 日足够定宿）
    let frame = MansionFrame::new(&eph, &cat, &sys, tp(jd_dawn), &opts).unwrap();
    let (sa, _) = radec(Body::Sun, jd_dawn);
    let (ma, md) = radec(Body::Moon, jd_dawn);
    let fu = cat.by_hip(87261).unwrap();
    let (fa, fd) = right_ascension_declination(&star_apparent_equatorial(&eph, fu, tp(jd_dawn), &opts));
    let (fa, fd) = (fa.rad().as_f64().to_degrees(), fd.rad().as_f64());
    let merid = (sa - h0 - 9.02).rem_euclid(360.0);
    let s = loc(&frame, sa);
    let m = loc(&frame, ma);
    let me = loc(&frame, merid);
    let _ = a_s;
    let rows = vec![
        vec!["合朔".into(), "丙子朔".into(), format!("合朔距丙子日正午 {:+.2} 日", tc - jdn), "".into()],
        vec!["日".into(), "在尾".into(), format!("{}{:.1}", s.0, s.1), "".into()],
        vec!["月".into(), "在策（傅说）".into(), format!("{}{:.1}；距傅说赤经 {:+.1}°，赤纬差 {:+.1}°", m.0, m.1, wrap180(ma - fa), (md - fd).to_degrees()), "傅说在尾宿".into()],
        vec!["旦中".into(), "鶉火中（柳、星、张）".into(), format!("子午圈 {}{:.1}", me.0, me.1), "".into()],
    ];
    for r in rows {
        println!("{}", r.join(" | "));
        tab.row(r);
    }
    tab.write();

    // 二、伐纣天象：扫描合朔
    let mut tab2 = crate::test_util::CheckTable::new("guoyu_fazhou", &["合朔（儒略历，天文纪年）", "日", "木星", "水星（合朔 ±20 日内到女虚危者）", "五项皆合"]);
    tab2.meta("《国语·周语下》伐纣天象逐个合朔扫描（前 1130—前 1000 年）：列出合朔于斗且木星在柳、星、张（鹑火）者；水星在玄枵（女、虚、危）查合朔前后 20 日；赤道宿度（陈卓距星）");
    let start = 2451545.0 + (-1130.0 - 2000.0) * 365.25;
    let end = 2451545.0 + (-1000.0 - 2000.0) * 365.25;
    let mut t = conj(start);
    let mut hits = vec![];
    while t < end {
        let frame = MansionFrame::new(&eph, &cat, &sys, tp(t), &opts).unwrap();
        let s = loc(&frame, radec(Body::Sun, t).0);
        if s.0 == "斗" {
            let j = loc(&frame, radec(Body::Planet(Planet::Jupiter), t).0);
            if ["柳", "星", "张"].contains(&j.0) {
                let merc: Vec<String> = (-20..=20)
                    .step_by(5)
                    .filter_map(|d| {
                        let tt = t + d as f64;
                        let f = MansionFrame::new(&eph, &cat, &sys, tp(tt), &opts).unwrap();
                        let q = loc(&f, radec(Body::Planet(Planet::Mercury), tt).0);
                        ["女", "虚", "危"].contains(&q.0).then(|| format!("{d:+}日{}{:.0}", q.0, q.1))
                    })
                    .collect();
                let (y, mo, da) = {
                    let z = (t + 0.5).floor() as i64;
                    let a = z + 32082;
                    let b = (4 * a + 3) / 1461;
                    let c = a - 1461 * b / 4;
                    let m = (5 * c + 2) / 153;
                    (b - 4800 + (m + 2) / 12 + 0, m + 3 - 12 * ((m + 2) / 12), c - (153 * m + 2) / 5 + 1)
                };
                let all = !merc.is_empty();
                let row = vec![format!("{y}-{mo:02}-{da:02}"), format!("{}{:.1}", s.0, s.1), format!("{}{:.1}", j.0, j.1),
                    if merc.is_empty() { "无".into() } else { merc.join(" ") }, if all { "是".into() } else { "否".into() }];
                println!("{}", row.join(" | "));
                if all {
                    hits.push(y);
                }
                tab2.row(row);
            }
        }
        t = conj(t + 29.53);
    }
    tab2.write();
    println!("合朔于斗、木星在鹑火、水星到玄枵的年份：{hits:?}");
}
