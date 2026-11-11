//! 民国历书定日基准核验：张表（vendor/calendar/chinese_calendar/实历朔日表.csv）1914—1948 年各月朔日，
//! 与现代天文合朔、中气（与网页同一组历表：VSOP87B 地球 + DE431 改正、ELPMPP02 缺省常数集、IAU2000 章动）
//! 分别按北京地方平时（东经 116°25′，UT+7h45m40s）与东经 120° 标准时（UT+8）定日、按无中气置闰排月比较。
//! 依据：陈展云《中国近代天文事迹》——民国三年起「以太阳平时为标准」，北京时代以北京地方平时所在日为准，
//! 南京时代以东经 120° 标准时所在日为准；《民国十七年国民历》是否已改，陈氏存疑。
//!
//! 用法（仓库根）：
//!   cargo run --release -p lunar-core --no-default-features --features real-f64 --example minguo_meridian_check
//! 输出 data/calendar/minguo_meridian_check.csv
use lunar_core::astronomy::aspects::solar_term_jds_for_year_cached;
use lunar_core::astronomy::ephemeris::vsop87_patch::Vsop87DePatch;
use lunar_core::astronomy::ephemeris::{load_all_from_binary_with_fit, Elpmpp02Correction, Vsop87};
use lunar_core::astronomy::frame::nutation;
use lunar_core::astronomy::time::{TimePoint, TimeScale};
use lunar_core::calendar::chinese_lunar::compute_year_data;
use lunar_core::calendar::gregorian::{civil_from_jdn, civil_to_jdn};
use lunar_core::math::real::{real, RealOps};
use lunar_core::quantity::angle::PlaneAngle;
use std::collections::BTreeMap;
use std::fs;

fn read(p: &str) -> Vec<u8> {
    fs::read(p).unwrap_or_else(|e| panic!("读 {p}: {e}"))
}

/// 北京地方平时与 UT 之差（日）：东经 116°25′ → 7h45m40s
const BEIJING: f64 = (7.0 * 3600.0 + 45.0 * 60.0 + 40.0) / 86400.0;
const E120: f64 = 8.0 / 24.0;

fn day(jd_ut: f64, off: f64) -> i64 {
    (jd_ut + 0.5 + off).floor() as i64
}

/// 冬至岁 y（天正冬至在 y−1 年 12 月）：(朔日 JDN, 农历年, 月, 闰)，无中气置闰（两冬至间 13 月，以第一个无中气之月为闰）
fn sui_months(y: i32, nm: &[f64], zq: &[f64], off: f64) -> Vec<(i64, i32, u8, bool)> {
    let nmd: Vec<i64> = nm.iter().map(|&t| day(t, off)).collect();
    let zqd: Vec<i64> = zq.iter().map(|&t| day(t, off)).collect();
    // 冬至：中气序列中 12 月 20 日前后者
    let dz = |yy: i32| *zqd.iter().find(|&&d| { let (gy, gm, gd) = civil_from_jdn(d); gy == yy && gm == 12 && gd >= 19 }).unwrap();
    let (z0, z1) = (dz(y - 1), dz(y));
    let i0 = nmd.iter().rposition(|&d| d <= z0).unwrap();
    let i1 = nmd.iter().rposition(|&d| d <= z1).unwrap();
    let n = i1 - i0;
    let zin: Vec<i64> = zqd.iter().copied().filter(|&d| d >= z0 && d <= z1).collect();
    let has = |i: usize| zin.iter().any(|&z| nmd[i] <= z && z < nmd[i + 1]);
    let leap = if n == 13 { (i0 + 1..i1).find(|&i| !has(i)) } else { None };
    let mut out = Vec::new();
    let mut m = 11u8;
    for i in i0..i1 {
        if Some(i) == leap {
            let (_, ly, pm, _) = *out.last().unwrap();
            out.push((nmd[i], ly, pm, true));
            continue;
        }
        let ly = if m >= 11 { y - 1 } else { y };
        out.push((nmd[i], ly, m, false));
        m = m % 12 + 1;
    }
    out
}

fn main() {
    let patch = Vsop87DePatch::from_binary(&read("data/fit/vsop87-de431-earth.bin")).expect("地球改正");
    let vsop = Vsop87::from_binary(&read("vendor/vsop87/VSOP87B.ear.bin")).expect("VSOP87").with_patch(patch.body("ear").expect("ear").clone());
    let corr = Elpmpp02Correction::DEFAULT;
    let fit = corr.builtin_fit().expect("ELP 常数集");
    let names = ["ELP_MAIN.S1", "ELP_MAIN.S2", "ELP_MAIN.S3", "ELP_PERT.S1", "ELP_PERT.S2", "ELP_PERT.S3"];
    let bufs: Vec<Vec<u8>> = names.iter().map(|n| read(&format!("vendor/elpmpp02/{n}.bin"))).collect();
    let elp = load_all_from_binary_with_fit([&bufs[0], &bufs[1], &bufs[2], &bufs[3], &bufs[4], &bufs[5]], &fit, corr).expect("ELPMPP02");
    assert!(nutation::try_init_full_nutation_from_binary(&read("vendor/IAU2000/tab5.3a.bin")), "章动表");
    let ut = |tt: f64| TimePoint::new(TimeScale::TT, real(tt)).to_scale(TimeScale::UTC).jd.as_f64();

    let tol = PlaneAngle::from_rad(real(1e-9));
    let mut nm: Vec<f64> = Vec::new();
    for ly in 1912..=1950 {
        let yd = compute_year_data(&vsop, &elp, ly, tol, 40).unwrap_or_else(|e| panic!("{ly}: {e}"));
        for t in &yd.new_moon_jds {
            let u = ut(t.as_f64());
            if !nm.iter().any(|x| (x - u).abs() < 1.0) {
                nm.push(u);
            }
        }
    }
    nm.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut zq: Vec<f64> = Vec::new();
    for gy in 1911..=1951 {
        let terms = solar_term_jds_for_year_cached(&vsop, gy);
        for k in (0..24).step_by(2) {
            zq.push(ut(terms[k].as_f64()));
        }
    }
    zq.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // 张表：JDN → (农历年, 月, 闰)
    let txt = String::from_utf8(read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let mut zh: BTreeMap<i64, (i32, u8, bool)> = BTreeMap::new();
    for line in txt.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        let (gy, gm, gd): (i32, i32, i32) = (f[0].parse().unwrap(), f[1].parse().unwrap(), f[2].parse().unwrap());
        zh.insert(civil_to_jdn(gy, gm, gd), (f[3].parse().unwrap(), f[4].parse().unwrap(), f[5] == "1"));
    }

    // 两基准逐月
    let mut bj: BTreeMap<(i32, u8, bool), i64> = BTreeMap::new();
    let mut e8: BTreeMap<(i32, u8, bool), i64> = BTreeMap::new();
    for y in 1914..=1949 {
        for (d, ly, m, l) in sui_months(y, &nm, &zq, BEIJING) {
            bj.insert((ly, m, l), d);
        }
        for (d, ly, m, l) in sui_months(y, &nm, &zq, E120) {
            e8.insert((ly, m, l), d);
        }
    }
    let zh_by_label: BTreeMap<(i32, u8, bool), i64> = zh.iter().map(|(&d, &k)| (k, d)).collect();
    let fmt = |d: i64| {
        let (y, m, dd) = civil_from_jdn(d);
        format!("{y}-{m:02}-{dd:02}")
    };
    let hm = |t: f64, off: f64| {
        let x = (t + 0.5 + off).rem_euclid(1.0) * 24.0;
        format!("{:02}:{:02}", x.floor() as i64, ((x - x.floor()) * 60.0).floor() as i64)
    };
    let mut out = String::from(
        "# 张表 1914—1948 各月朔日与现代天文（VSOP87B+DE431 改正、ELPMPP02、IAU2000 章动；ΔT 用项目 TimeScale）按北京地方平时（UT+7h45m40s）、\
东经 120°（UT+8）定日排月之比较（rust/core/examples/minguo_meridian_check.rs）。合朔时刻为两基准下的地方时；近子夜 = 任一基准下距子夜 15 分钟内。\n\
农历年,月,闰,张表朔日,北京平时朔日,东八区朔日,合朔北京平时,合朔东八区,近子夜,张表合北京平时,张表合东八区\n",
    );
    let mut stat: BTreeMap<&str, (usize, usize, usize)> = BTreeMap::new();
    for (&(ly, m, l), &zd) in zh_by_label.range((1914, 0, false)..(1949, 0, false)) {
        let (b, e) = (bj.get(&(ly, m, l)).copied(), e8.get(&(ly, m, l)).copied());
        // 本月合朔时刻：张表朔日附近之合朔
        let t = nm.iter().copied().find(|&t| (day(t, E120) - zd).abs() <= 1).unwrap();
        let near = [BEIJING, E120].iter().any(|&o| {
            let f = (t + 0.5 + o).rem_euclid(1.0);
            f < 15.0 / 1440.0 || f > 1.0 - 15.0 / 1440.0
        });
        let (hb, he) = (b == Some(zd), e == Some(zd));
        let seg = if ly <= 1927 { "1914—1927" } else if ly == 1928 { "1928" } else { "1929—1948" };
        let s = stat.entry(seg).or_default();
        s.0 += 1;
        s.1 += hb as usize;
        s.2 += he as usize;
        out += &format!(
            "{ly},{m},{},{},{},{},{},{},{},{},{}\n",
            l as u8,
            fmt(zd),
            b.map_or("—".into(), fmt),
            e.map_or("—".into(), fmt),
            hm(t, BEIJING),
            hm(t, E120),
            near as u8,
            hb as u8,
            he as u8
        );
        if !hb || !he || near {
            println!(
                "{ly}年{}{m}月 张表 {} | 北京平时 {}（合朔 {}） 东八区 {}（合朔 {}）{}",
                if l { "闰" } else { "" },
                fmt(zd),
                b.map_or("—".into(), fmt),
                hm(t, BEIJING),
                e.map_or("—".into(), fmt),
                hm(t, E120),
                if near { " 近子夜" } else { "" }
            );
        }
    }
    for (seg, (n, b, e)) in &stat {
        println!("{seg}: {n} 月，合北京平时 {b}，合东八区 {e}");
    }
    fs::write("data/calendar/minguo_meridian_check.csv", out).unwrap();
}
