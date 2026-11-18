//! 唐宋诸历交会规则（`dingshuo::JiaoOptions`）各选项之检验：逐历与张表逐月同日数、本纪明书朔日合数（取本纪之法同
//! `zhongyuan_reckoning_eval`：除辽金北朝本纪、「其他」类），供定缺省。输出 data/calendar/jiaohui_options_eval.csv。
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example jiaohui_options_eval
use lunar_core::calendar::dingshuo::{reckon_year_with, registry, JiaoBujin, JiaoOptions, ZhengyueJiao};
use lunar_core::calendar::historical::ShuoriTable;
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

const LIS: &[&str] = &["大衍历", "五纪历", "正元历", "宣明历", "崇玄历（宣明气朔）", "调元历（宣明气朔）", "崇天历", "观天历", "纪元历", "统元历", "乾道历", "淳熙历", "会元历", "统天历", "开禧历", "成天历"];

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    let zhang = ShuoriTable::from_csv(&read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let rows = parse_csv(&read("data/calendar/benji_shuo_check.csv"));
    let head = rows.iter().find(|r| r.first().is_some_and(|c| c == "书")).unwrap().clone();
    let col = |n: &str| head.iter().position(|h| h == n).unwrap();
    let (cb, cy, cm, cr, ck, cs) = (col("书"), col("年"), col("月"), col("本纪朔 JDN"), col("类"), col("政权"));
    let skip_books = ["liaoshi_benji", "jinshi_benji", "weishu_benji", "beiqishu_benji", "zhoushu_benji"];
    // (年, 月, 闰, 本纪朔)
    let mut benji: Vec<(i32, u8, bool, i64)> = vec![];
    for r in rows.iter().filter(|r| r.len() == head.len() && r[0] != "书") {
        if skip_books.contains(&r[cb].as_str()) || ["吴", "蜀"].contains(&r[cs].as_str()) || r[ck] == "其他" {
            continue;
        }
        let (Ok(y), Ok(rec)) = (r[cy].parse::<i32>(), r[cr].parse::<i64>()) else { continue };
        let (leap, m) = match r[cm].strip_prefix('闰').or_else(|| r[cm].strip_prefix('閏')) {
            Some(x) => (true, x),
            None => (false, r[cm].as_str()),
        };
        let Ok(m) = m.parse::<u8>() else { continue };
        benji.push((y, m, leap, rec));
    }
    let d = JiaoOptions::default();
    let variants: Vec<(&str, JiaoOptions)> = vec![
        ("缺省", d),
        ("交会规则全不用", JiaoOptions { xuanming: None, song: None, zhengyue: ZhengyueJiao::Off }),
        ("宣明：夜食亦不进（阴历）", JiaoOptions { xuanming: Some(JiaoBujin { yin: true, wan_min: None }), ..d }),
        ("宣明：夜食亦不进（不限阴阳历）", JiaoOptions { xuanming: Some(JiaoBujin { yin: false, wan_min: None }), ..d }),
        ("宣明：不限阴阳历", JiaoOptions { xuanming: Some(JiaoBujin { yin: false, wan_min: Some(80) }), ..d }),
        ("宣明：亏初取食甚减 40 分", JiaoOptions { xuanming: Some(JiaoBujin { yin: true, wan_min: Some(40) }), ..d }),
        ("宣明：亏初取食甚减 120 分", JiaoOptions { xuanming: Some(JiaoBujin { yin: true, wan_min: Some(120) }), ..d }),
        ("宋：只取阴历", JiaoOptions { song: Some(JiaoBujin { yin: true, wan_min: Some(0) }), ..d }),
        ("宋：夜食亦不进", JiaoOptions { song: Some(JiaoBujin { yin: false, wan_min: None }), ..d }),
        ("正月有交：不用", JiaoOptions { zhengyue: ZhengyueJiao::Off, ..d }),
        ("正月有交：大衍一系全程", JiaoOptions { zhengyue: ZhengyueJiao::All, ..d }),
    ];
    let reg = registry();
    let mut out = String::from("选项,历,月数,与张表同,本纪条数,本纪合推步\n");
    for (label, o) in &variants {
        let mut line = format!("{label}：");
        for &name in LIS {
            let (_, spans) = reg.iter().find(|(n, _)| *n == name).unwrap();
            let (mut months, mut same, mut n, mut hit) = (0, 0, 0, 0);
            for &(a, b) in spans.iter() {
                for y in a..=b {
                    let Some(r) = reckon_year_with(name, y, o) else { continue };
                    let t = zhang.year_months(y).unwrap();
                    months += r.len();
                    same += r.iter().filter(|m| t.contains(m)).count();
                    for &(_, m, l, rec) in benji.iter().filter(|b| b.0 == y) {
                        n += 1;
                        hit += usize::from(r.iter().any(|e| e.1 == m && e.2 == l && e.0 == rec));
                    }
                }
            }
            let _ = writeln!(out, "{label},{name},{months},{same},{n},{hit}");
            let _ = write!(line, " {name} {same}/{hit}");
        }
        println!("{line}");
    }
    std::fs::write(root.join("data/calendar/jiaohui_options_eval.csv"), out).unwrap();
}
