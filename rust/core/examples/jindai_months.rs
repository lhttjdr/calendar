//! 中原一线 1914—2016 年按当时编算规则现算的逐月历日，供第 4 层仲裁取作「按术推步」，不再取今人历表。
//!
//! 规则（依据见 doc/calendar/jindai-calendar.md）：定气（太阳视黄经每 15°）、定朔、两冬至间 13 月则以第一个无中气之月为闰；
//! 定日基准：1914—1927 年北京地方平时（东经 116°25′，UT+7h45m40s；《三年历书·凡例》「以太阳平时为标准」，陈展云），
//! 1929 年起东经 120° 标准时（UT+8）；1928 年两种基准逐月相同（陈展云存疑之年，无判别之月，程序核对）。
//! 天文与网页同一组：VSOP87B 地球 + DE431 拟合改正、ELPMPP02 缺省常数集、IAU2000 章动，ΔT 用项目 TimeScale。
//!
//! 输出：
//!   data/calendar/jindai_months.csv —— 逐月朔日（rust/core/src/calendar/jindai.rs 编入），并列张表之日作对照；
//!   doc/calendar/jindai-checks.md —— 逐月检验表（与张表、与另一定日基准之差，合朔、中气近子夜者）。
//! 用法（仓库根）：
//!   cargo run --manifest-path rust/Cargo.toml --release -p lunar-core --no-default-features --features real-f64 --example jindai_months
use lunar_core::astronomy::aspects::solar_term_jds_for_year_cached;
use lunar_core::astronomy::ephemeris::vsop87_patch::Vsop87DePatch;
use lunar_core::astronomy::ephemeris::{load_all_from_binary_with_fit, Elpmpp02Correction, Vsop87};
use lunar_core::astronomy::frame::nutation;
use lunar_core::astronomy::time::{TimePoint, TimeScale};
use lunar_core::calendar::chinese_lunar::compute_year_data;
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
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
const Y0: i32 = 1914;
const Y1: i32 = 2016;

fn day(jd_ut: f64, off: f64) -> i64 {
    (jd_ut + 0.5 + off).floor() as i64
}

/// 距最近子夜（分钟，带号：正 = 子夜后）
fn from_midnight(jd_ut: f64, off: f64) -> f64 {
    let f = (jd_ut + 0.5 + off).rem_euclid(1.0);
    (if f > 0.5 { f - 1.0 } else { f }) * 1440.0
}

/// 冬至岁 y（天正冬至在 y−1 年 12 月）各月：(合朔 UT, 朔日 JDN, 农历年, 月, 闰)
fn sui_months(y: i32, nm: &[f64], zq: &[f64], off: f64) -> Vec<(f64, i64, i32, u8, bool)> {
    let nmd: Vec<i64> = nm.iter().map(|&t| day(t, off)).collect();
    let zqd: Vec<i64> = zq.iter().map(|&t| day(t, off)).collect();
    let dz = |yy: i32| {
        *zqd.iter()
            .find(|&&d| {
                let (gy, gm, gd) = civil_from_jdn(d);
                gy == yy && gm == 12 && gd >= 19
            })
            .unwrap()
    };
    let (z0, z1) = (dz(y - 1), dz(y));
    let i0 = nmd.iter().rposition(|&d| d <= z0).unwrap();
    let i1 = nmd.iter().rposition(|&d| d <= z1).unwrap();
    let n = i1 - i0;
    let zin: Vec<i64> = zqd.iter().copied().filter(|&d| d >= z0 && d <= z1).collect();
    let has = |i: usize| zin.iter().any(|&z| nmd[i] <= z && z < nmd[i + 1]);
    let leap = if n == 13 { (i0 + 1..i1).find(|&i| !has(i)) } else { None };
    let mut out: Vec<(f64, i64, i32, u8, bool)> = Vec::new();
    let mut m = 11u8;
    for i in i0..i1 {
        if Some(i) == leap {
            let &(_, _, ly, pm, _) = out.last().unwrap();
            out.push((nm[i], nmd[i], ly, pm, true));
            continue;
        }
        let ly = if m >= 11 { y - 1 } else { y };
        out.push((nm[i], nmd[i], ly, m, false));
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
    for ly in Y0 - 2..=Y1 + 2 {
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
    for gy in Y0 - 3..=Y1 + 2 {
        let terms = solar_term_jds_for_year_cached(&vsop, gy);
        for k in (0..24).step_by(2) {
            zq.push(ut(terms[k].as_f64()));
        }
    }
    zq.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // 两种基准逐月：(农历年, 月, 闰) → (合朔 UT, 朔日)
    type Key = (i32, u8, bool);
    let mut bj: BTreeMap<Key, (f64, i64)> = BTreeMap::new();
    let mut e8: BTreeMap<Key, (f64, i64)> = BTreeMap::new();
    for y in Y0..=Y1 + 1 {
        for (t, d, ly, m, l) in sui_months(y, &nm, &zq, BEIJING) {
            bj.insert((ly, m, l), (t, d));
        }
        for (t, d, ly, m, l) in sui_months(y, &nm, &zq, E120) {
            e8.insert((ly, m, l), (t, d));
        }
    }
    // 1928 年两基准逐月相同（不同则须另定依据）
    let k28: Vec<Key> = bj.keys().filter(|k| k.0 == 1928).copied().collect();
    assert_eq!(k28, e8.keys().filter(|k| k.0 == 1928).copied().collect::<Vec<_>>(), "1928 年月序");
    for k in &k28 {
        assert_eq!(bj[k].1, e8[k].1, "1928 年 {k:?} 两基准朔日不同");
    }

    // 张表：(农历年, 月, 闰) → 朔日
    let txt = String::from_utf8(read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let mut zh: BTreeMap<Key, i64> = BTreeMap::new();
    for line in txt.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        let (gy, gm, gd): (i32, i32, i32) = (f[0].parse().unwrap(), f[1].parse().unwrap(), f[2].parse().unwrap());
        zh.insert((f[3].parse().unwrap(), f[4].parse().unwrap(), f[5] == "1"), civil_to_jdn(gy, gm, gd));
    }
    let fmt = |d: i64| {
        let (y, m, dd) = civil_from_jdn(d);
        format!("{y}-{m:02}-{dd:02}")
    };
    let hms = |t: f64, off: f64| {
        let s = ((t + 0.5 + off).rem_euclid(1.0) * 86400.0).round() as i64 % 86400;
        format!("{:02}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
    };
    let base_of = |ly: i32| if ly <= 1927 { ("北京地方平时", BEIJING) } else { ("东经120°", E120) };

    let mut csv = String::from(
        "# 中原一线 1914—2016 年按当时编算规则现算之逐月朔日（rust/core/examples/jindai_months.rs 生成，勿手改；rust/core/src/calendar/jindai.rs 编入）。\n\
# 规则：定气定朔、无中气置闰；定日基准 1914—1927 北京地方平时（UT+7h45m40s），1928 起东经 120° 标准时（1928 年两基准逐月相同）。依据与检验见 doc/calendar/jindai-calendar.md、jindai-checks.md。\n\
# 合朔时刻为定日基准之地方时；距子夜 = 合朔距最近子夜（分，正为子夜后）；另一基准朔日 = 改用另一定日基准所得（同则空）；张表只作对照。\n\
农历年,月,闰,大小,朔日,朔日JDN,朔干支,定日基准,合朔时刻,距子夜(分),另一基准朔日,张表朔日,合张表\n",
    );
    let mut md = vec![
        "# 近现代中原一线历日逐月检验（自动生成）".to_string(),
        String::new(),
        "> 由 `rust/core/examples/jindai_months.rs` 生成，勿手改。规则与依据见 [jindai-calendar.md](jindai-calendar.md)。".to_string(),
        String::new(),
    ];
    let mut rows: Vec<String> = Vec::new();
    let mut near_rows: Vec<String> = Vec::new();
    let mut stat: BTreeMap<&str, (usize, usize, usize)> = BTreeMap::new();
    for ly in Y0..=Y1 {
        let (bname, off) = base_of(ly);
        let (own, other) = if off == BEIJING { (&bj, &e8) } else { (&e8, &bj) };
        let keys: Vec<Key> = own.keys().filter(|k| k.0 == ly).copied().collect();
        assert!(keys.len() == 12 || keys.len() == 13, "{ly} 年 {} 月", keys.len());
        // 下一月朔（定月大小）
        let next_of = |k: &Key| -> i64 {
            let all: Vec<(&Key, &(f64, i64))> = own.iter().collect();
            let i = all.iter().position(|(kk, _)| *kk == k).unwrap();
            // 本岁最后一月之后：下一岁正月（同一基准）
            all.iter().skip(i + 1).map(|(_, v)| v.1).find(|&d| d > own[k].1).unwrap()
        };
        for k in keys {
            let (t, d) = own[&k];
            let len = next_of(&k) - d;
            assert!(len == 29 || len == 30, "{k:?} 月长 {len}");
            let od = other.get(&k).map(|v| v.1);
            let z = zh.get(&k).copied();
            let mn = from_midnight(t, off);
            let seg = if ly <= 1927 { "1914—1927" } else if ly == 1928 { "1928" } else if ly <= 1948 { "1929—1948" } else { "1949—2016" };
            let s = stat.entry(seg).or_default();
            s.0 += 1;
            s.1 += (z == Some(d)) as usize;
            s.2 += (od == Some(d)) as usize;
            let other_s = match od {
                Some(x) if x == d => String::new(),
                Some(x) => fmt(x),
                None => "（月序不同）".into(),
            };
            let line = format!(
                "{ly},{},{},{},{},{d},{},{bname},{},{:+.1},{other_s},{},{}",
                k.1,
                k.2 as u8,
                if len == 30 { "大" } else { "小" },
                fmt(d),
                GAN_ZHI_60[(d + 49).rem_euclid(60) as usize],
                hms(t, off),
                mn,
                z.map_or("—".into(), fmt),
                (z == Some(d)) as u8
            );
            csv += &line;
            csv += "\n";
            let cells = format!(
                "| {ly} | {}{} | {} | {} | {} | {bname} | {} | {:+.1} | {} | {} | {} |",
                if k.2 { "闰" } else { "" },
                k.1,
                if len == 30 { "大" } else { "小" },
                fmt(d),
                GAN_ZHI_60[(d + 49).rem_euclid(60) as usize],
                hms(t, off),
                mn,
                if other_s.is_empty() { "同".into() } else { other_s.clone() },
                z.map_or("—".into(), fmt),
                if z == Some(d) { "✓" } else { "✗" }
            );
            if z != Some(d) || mn.abs() < 30.0 {
                near_rows.push(cells.clone());
            }
            rows.push(cells);
        }
    }
    // 中气近子夜（影响置闰）
    let mut zq_rows = Vec::new();
    for &t in &zq {
        let d = day(t, E120);
        let (gy, ..) = civil_from_jdn(d);
        if !(Y0..=Y1 + 1).contains(&gy) {
            continue;
        }
        let off = if gy <= 1927 { BEIJING } else { E120 };
        let mn = from_midnight(t, off);
        if mn.abs() < 10.0 {
            zq_rows.push(format!("| {} | {} | {:+.1} |", fmt(day(t, off)), hms(t, off), mn));
        }
    }
    let total: usize = stat.values().map(|s| s.0).sum();
    let agree: usize = stat.values().map(|s| s.1).sum();
    md.push(format!("共 {total} 月，与张表同日 {agree}。分段（月数 / 合张表 / 两种定日基准同日）："));
    md.push(String::new());
    md.push("| 年段 | 月数 | 合张表 | 两基准同日 |".into());
    md.push("|---|---|---|---|".into());
    for (seg, (n, a, o)) in &stat {
        md.push(format!("| {seg} | {n} | {a} | {o} |"));
    }
    md.push(String::new());
    let head = "| 农历年 | 月 | 大小 | 朔日 | 干支 | 定日基准 | 合朔（基准地方时） | 距子夜（分） | 另一基准朔日 | 张表朔日 | 合张表 |\n|---|---|---|---|---|---|---|---|---|---|---|";
    md.push("## 与张表不同或合朔距子夜 30 分钟以内之月".into());
    md.push(String::new());
    md.push(head.into());
    md.extend(near_rows);
    md.push(String::new());
    md.push("## 中气距子夜 10 分钟以内（定日基准地方时）".into());
    md.push(String::new());
    md.push("| 日期 | 时刻 | 距子夜（分） |\n|---|---|---|".into());
    md.extend(zq_rows);
    md.push(String::new());
    md.push("## 逐月".into());
    md.push(String::new());
    md.push(head.into());
    md.extend(rows);
    md.push(String::new());
    fs::write("data/calendar/jindai_months.csv", csv).unwrap();
    fs::write("doc/calendar/jindai-checks.md", md.join("\n")).unwrap();
    for (seg, (n, a, o)) in &stat {
        println!("{seg}: {n} 月，合张表 {a}，两基准同日 {o}");
    }
}
