//! 古六历书朔逐历分析：对黄帝、颛顼、夏（冬至、雨水两版）、殷、周、鲁七套参数逐历推平朔（`calendar::ancient_restore`），
//! 与「明书朔（晦）」记录、实历朔日表、秦汉初复原历及天文真朔逐条比较。
//!
//! 输入 `data/calendar/guliuli_shuo_records.csv`（`scripts/calendar_data/guliuli_shuo_records.py` 生成）；
//! 输出逐条表 `data/calendar/guliuli_shuo_check.csv`，并在标准输出打印汇总（Markdown 表，抄入 doc/calendar/guliuli-shuo.md）。
//!
//! 天文真朔：DE431（缺省，本地有则用；否则 DE406）定朔，ΔT 取本仓库缺省模型，换成地方平时取日：春秋（前 722—前 468 年）
//! 取曲阜 116.98°E，其余取长安 108.94°E。
//!
//! 用法（仓库根）：cargo run --release -p lunar-core --no-default-features --features real-f64 --example guliuli_shuo

use std::collections::BTreeMap;
use std::fmt::Write as _;

use lunar_core::astronomy::aspects::new_moon_jds_in_range_de406;
use lunar_core::astronomy::ephemeris::De406Kernel;
use lunar_core::astronomy::frame::nutation::try_init_full_nutation_from_repo;
use lunar_core::astronomy::time::delta_t;
use lunar_core::calendar::ancient_restore::{guliuli_all, mean_new_moon, mean_new_moon_offset, nearest_mean_new_moon};
use lunar_core::math::real::{real, Real, RealOps};
use lunar_core::quantity::angle::PlaneAngle;
use lunar_core::repo::{paths, repo_root};
use num_traits::ToPrimitive;

const QUFU_LON: f64 = 116.98;
const CHANGAN_LON: f64 = 108.94;

struct Rec {
    group: String,
    year: i32,
    month: String,
    ganzhi: String,
    position: String,
    day: i64,
    note: String,
    text: String,
}

fn load_records() -> Vec<Rec> {
    let path = repo_root().join("data/calendar/guliuli_shuo_records.csv");
    let s = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    s.lines()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
        .filter(|l| !l.is_empty())
        .map(|l| {
            let f: Vec<&str> = l.splitn(8, ',').collect();
            Rec {
                group: f[0].into(),
                year: f[1].parse().unwrap(),
                month: f[2].into(),
                ganzhi: f[3].into(),
                position: f[4].into(),
                day: f[5].parse().unwrap(),
                note: f[6].into(),
                text: f[7].into(),
            }
        })
        .collect()
}

fn lon_of(year: i32) -> f64 {
    if year <= -467 {
        QUFU_LON
    } else {
        CHANGAN_LON
    }
}

/// 天文真朔：TT 时刻 → 地方平时 JD（日界夜半）。
fn local_jd(jd_tt: Real, lon_deg: f64) -> Real {
    jd_tt - delta_t(jd_tt).in_days_value() + real(lon_deg / 360.0)
}

fn to_real(x: &num_rational::BigRational) -> Real {
    real(x.to_f64().expect("有理数转实数"))
}

fn main() {
    assert!(try_init_full_nutation_from_repo(), "未能加载 IAU 2000A 章动表");
    let root = repo_root();
    let bsp = [root.join(paths::DE431_BSP)]
        .into_iter()
        .chain(paths::DE406_BSP_CANDIDATES.iter().map(|p| root.join(p)))
        .find(|p| p.exists())
        .expect("需 DE431 或 DE406 BSP");
    let kernel = De406Kernel::open(&bsp).expect("打开 BSP");
    eprintln!("天文真朔历表：{}", bsp.display());
    let cals = guliuli_all();
    let recs = load_records();

    // 天文真朔（TT），前 730—前 95 年
    let jd0 = real(1454400.0);
    let jd1 = real(1686400.0);
    let tol = PlaneAngle::from_rad(real(1e-9));
    let true_nm: Vec<Real> = new_moon_jds_in_range_de406(&kernel, jd0, jd1, tol, 20);
    eprintln!("真朔 {} 个", true_nm.len());
    // 真朔所在之日（地方平时）：最接近 day 者
    let nearest_true = |day: i64, lon: f64| -> (Real, i64) {
        let x = real(day as f64);
        let i = true_nm.partition_point(|&t| t < x);
        let cand = [i.saturating_sub(1), i.min(true_nm.len() - 1)];
        cand.iter()
            .map(|&k| {
                let lj = local_jd(true_nm[k], lon);
                (true_nm[k], (lj + real(0.5)).floor().as_f64() as i64)
            })
            .min_by_key(|&(_, d)| (d - day).abs())
            .unwrap()
    };

    // 1. 逐条表
    let mut csv = String::new();
    csv += "# 古六历书朔逐历分析（rust/core/examples/guliuli_shuo.rs 生成）：各列为「记录朔日 − 该历最近平朔日」（日）；真朔列为「记录朔日 − 天文真朔日（地方平时）」\n";
    csv += "group,year,month,ganzhi,position,shuo_jdn,真朔";
    for (n, _) in &cals {
        let _ = write!(csv, ",{n}");
    }
    csv += ",note,text\n";
    // 分组统计：group → 历名 → 差 → 次数
    let mut stats: BTreeMap<String, BTreeMap<String, BTreeMap<i64, usize>>> = BTreeMap::new();
    let mut order: Vec<String> = vec![];
    for r in &recs {
        if !order.contains(&r.group) {
            order.push(r.group.clone());
        }
        let (_, tday) = nearest_true(r.day, lon_of(r.year));
        let dt = r.day - tday;
        // 汉书日食当日确有日食者另成一组：记录日由天象独立确定，不依赖复原历排月
        let sub = (r.group == "汉书日食朔晦" && r.note != "无日食").then(|| "汉书日食朔晦·当日确有日食".to_string());
        if let Some(sg) = &sub {
            if !order.contains(sg) {
                order.push(sg.clone());
            }
            let g = stats.entry(sg.clone()).or_default();
            *g.entry("天文真朔".into()).or_default().entry(dt).or_default() += 1;
            for (n, c) in &cals {
                *g.entry((*n).into()).or_default().entry(r.day - nearest_mean_new_moon(c, r.day).1).or_default() += 1;
            }
        }
        let g = stats.entry(r.group.clone()).or_default();
        *g.entry("天文真朔".into()).or_default().entry(dt).or_default() += 1;
        let _ = write!(csv, "{},{},{},{},{},{},{}", r.group, r.year, r.month, r.ganzhi, r.position, r.day, dt);
        for (n, c) in &cals {
            let d = r.day - nearest_mean_new_moon(c, r.day).1;
            *g.entry((*n).into()).or_default().entry(d).or_default() += 1;
            let _ = write!(csv, ",{d}");
        }
        let _ = writeln!(csv, ",{},{}", r.note, r.text.replace(',', "，"));
    }
    std::fs::write(root.join("data/calendar/guliuli_shuo_check.csv"), csv.clone()).unwrap();

    // 1b. 逐条 Markdown（记录少于 100 条的组）
    println!("## 逐条（记录朔日 − 该历朔日，日；0 为恰合）\n");
    for grp in &order {
        let rows: Vec<&str> = csv.lines().skip(2).filter(|l| l.starts_with(&format!("{grp},"))).collect();
        if rows.is_empty() || rows.len() >= 100 {
            continue;
        }
        println!("### {grp}\n");
        let mut h = "| 记录 | 朔日 JDN | 备注 | 天文真朔 |".to_string();
        let mut sep = "|---|---|---|---|".to_string();
        for (n, _) in &cals {
            let _ = write!(h, " {n} |");
            sep += "---|";
        }
        println!("{h}\n{sep}");
        for l in rows {
            let f: Vec<&str> = l.split(',').collect();
            let k = 7 + cals.len();
            let mut line = format!("| {} | {} | {} | {} |", f[k + 1], f[5], f[k], f[6]);
            for v in &f[7..k] {
                let _ = write!(line, " {v} |");
            }
            println!("{line}");
        }
        println!();
    }

    // 2. 汇总：各组各历「记录 − 历」的分布
    let mut names: Vec<String> = vec!["天文真朔".into()];
    names.extend(cals.iter().map(|(n, _)| n.to_string()));
    println!("## 逐组分布（记录朔日 − 该历朔日，日：−1 / 0 / +1 / 其他）\n");
    for grp in &order {
        let g = &stats[grp];
        let total: usize = g["天文真朔"].values().sum();
        println!("### {grp}（{total} 条）\n");
        println!("| 历 | −2 及更早 | −1 | 0（恰合） | +1 | +2 及更晚 |");
        println!("|---|---|---|---|---|---|");
        for n in &names {
            let h = &g[n];
            let le = |lo: i64, hi: i64| h.iter().filter(|(k, _)| **k >= lo && **k <= hi).map(|(_, v)| *v).sum::<usize>();
            println!("| {n} | {} | {} | {} | {} | {} |", le(i64::MIN, -2), le(-1, -1), le(0, 0), le(1, 1), le(2, i64::MAX));
        }
        println!();
    }

    // 3. 各历平朔的固定相差（以殷历为准）
    let yin = &cals.iter().find(|(n, _)| *n == "殷").unwrap().1;
    println!("## 平朔相差（该历平朔 − 殷历最近平朔，日）\n");
    println!("| 历 | 相差（日） | 相差（时） | 朔日与殷历同日的比例 |");
    println!("|---|---|---|---|");
    for (n, c) in &cals {
        let off = to_real(&mean_new_moon_offset(c, yin)).as_f64();
        let frac = off - off.floor();
        // 时刻差 n + f：朔日差只取 n、n+1，n 的比例为 1 − f
        let same = if off.floor() == 0.0 { 1.0 - frac } else if off.floor() == -1.0 { frac } else { 0.0 };
        println!("| {n} | {off:+.4} | {:+.2} | {:.1}% |", off * 24.0, same * 100.0);
    }
    println!();

    // 4. 各时段：平朔时刻 − 天文真朔（地方平时），平均与朔日相合率
    println!("## 各时段平朔 − 天文真朔（地方平时；平均时刻差，日；平朔日与真朔日同日 / 平朔日早 / 平朔日晚 之比例）\n");
    let spans: [(&str, f64, f64, f64); 5] = [
        ("春秋前期 前 722—前 621", 1457000.0, 1494200.0, QUFU_LON),
        ("春秋后期 前 620—前 468", 1494200.0, 1550000.0, QUFU_LON),
        ("战国 前 467—前 222", 1550000.0, 1640100.0, CHANGAN_LON),
        ("秦汉初 前 221—前 105", 1640100.0, 1682750.0, CHANGAN_LON),
        ("太初前后 前 104—前 95", 1682750.0, 1686300.0, CHANGAN_LON),
    ];
    let mut head = "| 时段 |".to_string();
    let mut sep = "|---|".to_string();
    for (n, _) in &cals {
        let _ = write!(head, " {n} |");
        sep += "---|";
    }
    println!("{head}\n{sep}");
    for (label, a, b, lon) in spans {
        let mut row = format!("| {label} |");
        let in_span: Vec<Real> = true_nm.iter().copied().filter(|t| t.as_f64() >= a && t.as_f64() < b).collect();
        for (_, c) in &cals {
            let (mut sum, mut same, mut early, mut late) = (0.0, 0usize, 0usize, 0usize);
            for &t in &in_span {
                let lj = local_jd(t, lon);
                let tday = (lj + real(0.5)).floor().as_f64() as i64;
                let (k, cday) = nearest_mean_new_moon(c, tday);
                sum += (to_real(&mean_new_moon(c, k)) - lj).as_f64();
                match cday - tday {
                    0 => same += 1,
                    d if d < 0 => early += 1,
                    _ => late += 1,
                }
            }
            let n = in_span.len() as f64;
            let _ = write!(row, " {:+.2}；{:.0}/{:.0}/{:.0}% |", sum / n, same as f64 / n * 100.0, early as f64 / n * 100.0, late as f64 / n * 100.0);
        }
        println!("{row}");
    }
}
