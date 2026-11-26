//! 唐宋诸历步交会（入交定、入食限）与史载日食逐条检验。
//!
//! 照各历「步交會」简法（`dingshuo::Jiaohui`：入交泛 + 入气朓朒 + 入转朓朒 × 交率/交数 = 入交定；去交不过望差（后限）为入食限），
//! 逐月判定定朔是否入食限、是否阴历、加时是否在昼，与新旧《唐书》、新旧《五代史》、《宋史》日食记录（`data/phenomena/*_solar_eclipses.csv`）
//! 比对：史载日食之月推得入食限否（漏判），推得入食限、阴历且昼见之月史载否（误判，含史失载者）。
//!
//! 输出 data/calendar/jiaohui_eclipse_check.csv（逐条：史载日食之月）、终端打印各历汇总。
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example jiaohui_eclipse_check
use lunar_core::calendar::dingshuo::{jiao_months, reckon_year, registry, JiaoMonth};
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::historical::ShuoriTable;
use num_traits::ToPrimitive;
use std::collections::BTreeMap;
use std::fmt::Write as _;

fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let (mut rows, mut row, mut cell, mut quoted) = (Vec::new(), Vec::new(), String::new(), false);
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        match (c, quoted) {
            ('"', true) if it.peek() == Some(&'"') => {
                cell.push('"');
                it.next();
            }
            ('"', _) => quoted = !quoted,
            (',', false) => row.push(std::mem::take(&mut cell)),
            ('\n', false) => {
                row.push(std::mem::take(&mut cell));
                rows.push(std::mem::take(&mut row));
            }
            ('\r', false) => {}
            _ => cell.push(c),
        }
    }
    if !cell.is_empty() || !row.is_empty() {
        row.push(cell);
        rows.push(row);
    }
    rows
}

fn gz(j: i64) -> &'static str {
    GAN_ZHI_60[(j + 49).rem_euclid(60) as usize]
}

/// 史载日食（同日合并诸书）
struct Rec {
    jdn: i64,
    books: Vec<&'static str>,
    year: i32,
    month: String,
    note: String,
    text: String,
}

/// 检验之历：(显示名, 取入交之历名, 年段之历名)
const LIS: &[(&str, &str, &[&str])] = &[
    ("麟德历", "麟德历", &["麟德历"]),
    ("大衍历", "大衍历", &["大衍历"]),
    ("五纪历", "五纪历", &["五纪历"]),
    ("正元历", "正元历", &["正元历"]),
    ("宣明历", "宣明历", &["宣明历"]),
    ("宣明气朔（五代）", "宣明历", &["崇玄历（宣明气朔）", "调元历（宣明气朔）"]),
    ("崇玄历（术文，对照）", "崇玄历", &["崇玄历"]),
    ("崇天历", "崇天历", &["崇天历"]),
    ("观天历", "观天历", &["观天历"]),
    ("纪元历", "纪元历", &["纪元历"]),
    ("统元历", "统元历", &["统元历"]),
    ("乾道历", "乾道历", &["乾道历"]),
    ("淳熙历", "淳熙历", &["淳熙历"]),
    ("会元历", "会元历", &["会元历"]),
    ("统天历", "统天历", &["统天历"]),
    ("开禧历", "开禧历", &["开禧历"]),
    ("成天历", "成天历", &["成天历"]),
];

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    let mut recs: BTreeMap<i64, Rec> = BTreeMap::new();
    for (file, book) in [
        ("xintangshu", "新唐书"),
        ("jiutangshu", "旧唐书"),
        ("jiuwudaishi", "旧五代史"),
        ("xinwudaishi", "新五代史"),
        ("songshi", "宋史"),
    ] {
        let rows = parse_csv(&read(&format!("data/phenomena/{file}_solar_eclipses.csv")));
        let h = &rows[0];
        let col = |n: &str| h.iter().position(|x| x == n).unwrap();
        let (cj, cy, cm, cl, cn, ct) = (col("jdn"), col("year"), col("month"), col("leap"), col("note"), col("text"));
        for r in &rows[1..] {
            if r.len() <= ct || r[cj].is_empty() {
                continue;
            }
            let jdn: i64 = r[cj].parse().unwrap();
            let e = recs.entry(jdn).or_insert_with(|| Rec {
                jdn,
                books: vec![],
                year: r[cy].parse().unwrap_or(0),
                month: format!("{}{}", if r[cl] == "1" { "闰" } else { "" }, r[cm]),
                note: String::new(),
                text: r[ct].chars().take(40).collect(),
            });
            if !e.books.contains(&book) {
                e.books.push(book);
            }
            if !r[cn].is_empty() && !e.note.contains(r[cn].as_str()) {
                e.note += &r[cn];
            }
        }
    }
    let mut out = String::from(
        "# 唐宋诸历步交会与史载日食逐条检验（rust/core/examples/jiaohui_eclipse_check.rs）。入交定 = 入交泛 + 入气朓朒 + 入转朓朒 × 交率/交数（日）；\n\
         # 去交 = 距最近交点（日）；入食限 = 去交不过望差（后限）；昼 = 定朔加时在日出、日入间；史载日食合并新旧唐书、新旧五代史、宋史，同日一条。\n\
         历,年,月,朔干支,史书,注,原文,定朔JDN,定朔干支,定小余（时）,日出（时）,入交定（日）,去交（日）,阴阳历,交前后,入食限,昼,术蚀,蚀历,食甚（时）,排月朔干支\n",
    );
    println!("{:<20} {:>5} {:>6} {:>6} {:>8} {:>8} | {:>6} {:>8} {:>8} {:>8}", "历", "史载", "入食限", "+阴历", "+昼", "阴历且昼", "推入限", "推阴历", "推阴昼", "史载/推阴昼");
    let reg = registry();
    // 张表朔日、本纪明书朔日（供第二表）
    let zhang = ShuoriTable::from_csv(&read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let mut zhang_days: BTreeMap<i64, (i32, u8, bool)> = BTreeMap::new();
    for y in 660..1290 {
        for &(j, m, l) in zhang.year_months(y).unwrap_or_default().iter() {
            zhang_days.insert(j, (y, m, l));
        }
    }
    let mut benji: BTreeMap<i64, String> = BTreeMap::new();
    {
        let text = read("data/calendar/benji_shuo_check.csv");
        let rows: Vec<Vec<String>> = parse_csv(&text.lines().filter(|l| !l.starts_with('#')).collect::<Vec<_>>().join("\n"));
        let h = &rows[0];
        let col = |n: &str| h.iter().position(|x| x == n).unwrap();
        let (cb, cz) = (col("本纪朔 JDN"), col("书"));
        for r in &rows[1..] {
            if let Ok(j) = r[cb].parse::<i64>() {
                benji.entry(j).or_default().push_str(&format!("{} ", r[cz]));
            }
        }
    }
    let mut out2 = String::from(
        "# 唐宋诸历入食限之月与进朔、正月（rust/core/examples/jiaohui_eclipse_check.rs）：定小余过进朔限、或在正月者。\n\
         历,定朔JDN,定朔干支,定小余（时）,日出（时）,日入（时）,进朔,去交（日）,阴阳历,昼,术蚀,蚀历,食甚（时）,排月朔干支,张表朔干支,张表月,本纪朔干支,本纪书,史载日食\n",
    );
    for &(label, src, span_names) in LIS {
        let Some(months) = jiao_months(src) else { continue };
        // 年段之 JDN 范围
        let mut ranges = vec![];
        for (n, spans) in reg.iter().filter(|(n, _)| span_names.contains(n)) {
            for &(a, b) in spans.iter() {
                let lo = reckon_year(n, a).unwrap()[0].0;
                let hi = reckon_year(n, b).unwrap().last().unwrap().0 + 29;
                ranges.push((lo, hi));
            }
        }
        let inr = |j: i64| ranges.iter().any(|&(a, b)| (a..=b).contains(&j));
        let ms: Vec<&JiaoMonth> = months.iter().filter(|m| inr(m.ding_jdn) && m.jiao.is_some()).collect();
        let (mut n, mut hs, mut hy, mut hz, mut hyz, mut nr, mut hr) = (0, 0, 0, 0, 0, 0, 0);
        for rec in recs.values().filter(|r| inr(r.jdn)) {
            let Some(m) = ms.iter().min_by_key(|m| (m.ding_jdn - rec.jdn).abs()).filter(|m| (m.ding_jdn - rec.jdn).abs() <= 2) else { continue };
            let j = m.jiao.as_ref().unwrap();
            n += 1;
            hs += usize::from(j.shixian);
            hy += usize::from(j.shixian && j.yin);
            hz += usize::from(j.shixian && m.zhou);
            hyz += usize::from(j.shixian && j.yin && m.zhou);
            let rs = m.rishi.as_ref();
            nr += usize::from(rs.is_some());
            hr += usize::from(rs.is_some_and(|r| r.shi));
            let _ = writeln!(
                out,
                "{label},{},{},{},{},{},{},{},{},{:.2},{},{:.3},{:.3},{},{},{},{},{},{},{},{}",
                rec.year,
                rec.month,
                gz(rec.jdn),
                rec.books.join("、"),
                rec.note,
                rec.text.replace(',', "，"),
                m.ding_jdn,
                gz(m.ding_jdn),
                m.frac * 24.0,
                m.sunrise.map_or(String::new(), |s| format!("{:.2}", s * 24.0)),
                j.ru.to_f64().unwrap(),
                j.qu.to_f64().unwrap(),
                if j.yin { "阴" } else { "阳" },
                if j.qian { "交前" } else { "交后" },
                u8::from(j.shixian),
                u8::from(m.zhou),
                rs.map_or(String::new(), |r| u8::from(r.shi).to_string()),
                rs.map_or("", |r| if r.yin { "阴" } else { "阳" }),
                rs.map_or(String::new(), |r| format!("{:.2}", r.shen.to_f64().unwrap() * 24.0)),
                gz(m.day),
            );
        }
        for m in &ms {
            let j = m.jiao.as_ref().unwrap();
            let zt = (-1..=1).map(|k| m.ding_jdn + k).find(|d| zhang_days.contains_key(d));
            let zhengyue = zt.is_some_and(|d| zhang_days[&d].1 == 1 && !zhang_days[&d].2);
            if !j.shixian || !(m.jin || zhengyue) {
                continue;
            }
            let bj = (-1..=1).map(|k| m.ding_jdn + k).find(|d| benji.contains_key(d));
            let rec = (-2..=2).map(|k| m.ding_jdn + k).find(|d| recs.contains_key(d));
            let _ = writeln!(
                out2,
                "{label},{},{},{:.2},{},{},{},{:.3},{},{},{},{},{},{},{},{},{},{},{}",
                m.ding_jdn,
                gz(m.ding_jdn),
                m.frac * 24.0,
                m.sunrise.map_or(String::new(), |s| format!("{:.2}", s * 24.0)),
                m.sunrise.map_or(String::new(), |s| format!("{:.2}", 24.0 - s * 24.0)),
                u8::from(m.jin),
                j.qu.to_f64().unwrap(),
                if j.yin { "阴" } else { "阳" },
                u8::from(m.zhou),
                m.rishi.as_ref().map_or(String::new(), |r| u8::from(r.shi).to_string()),
                m.rishi.as_ref().map_or("", |r| if r.yin { "阴" } else { "阳" }),
                m.rishi.as_ref().map_or(String::new(), |r| format!("{:.2}", r.shen.to_f64().unwrap() * 24.0)),
                gz(m.day),
                zt.map_or("", gz),
                zt.map_or(String::new(), |d| format!("{}年{}{}月", zhang_days[&d].0, if zhang_days[&d].2 { "闰" } else { "" }, zhang_days[&d].1)),
                bj.map_or("", gz),
                bj.map_or(String::new(), |d| benji[&d].trim().to_string()),
                rec.map_or(String::new(), |d| format!("{}{}", gz(d), recs[&d].books.join("、"))),
            );
        }
        let ps = ms.iter().filter(|m| m.jiao.as_ref().unwrap().shixian).count();
        let py = ms.iter().filter(|m| m.jiao.as_ref().is_some_and(|j| j.shixian && j.yin)).count();
        let pyz = ms.iter().filter(|m| m.zhou && m.jiao.as_ref().is_some_and(|j| j.shixian && j.yin)).count();
        let pr = ms.iter().filter(|m| m.rishi.as_ref().is_some_and(|r| r.shi)).count();
        let rs = if nr > 0 { format!("  术蚀 {hr}/{nr}（推 {pr}）") } else { String::new() };
        println!("{label:<20} {n:>5} {hs:>6} {hy:>6} {hz:>8} {hyz:>8} | {ps:>6} {py:>8} {pyz:>8} {:>8}{rs}", format!("{hyz}/{pyz}"));
    }
    std::fs::write(root.join("data/calendar/jiaohui_eclipse_check.csv"), out).unwrap();
    std::fs::write(root.join("data/calendar/jiaohui_jinshuo_months.csv"), out2).unwrap();
}
