//! 主序列逐段取法表：按第 4 层仲裁的缺省取法（[MainSeriesPreference::default]），逐年看 `main_series_year_months` 实际给出的来源，
//! 按（行用之历, 来源）连成年段，并附本纪明书朔检验合数（取自 `data/calendar/main_series_reckoning_eval.csv`，须先跑 main_series_reckoning_eval）。
//! 输出 `data/calendar/main_series_qufa.csv`；`scripts/calendar_data/qufa_to_md.py` 据此改写知识库与读者站《历代历法总录》的取法表，
//! 免得「各历取法」在代码、文档、读者站各写一遍而不同步。
//!
//! 用法（rust/ 目录）：cargo run --release -p lunar-core --no-default-features --features real-f64 --example main_series_qufa

use lunar_core::calendar::historical::ShuoriTable;
use lunar_core::calendar::lifa::{main_series_year_months, RiliSource, MainSeriesCalendarPeriods, MainSeriesPreference};
use std::collections::HashMap;

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    let zhang = ShuoriTable::from_csv(&read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let periods = MainSeriesCalendarPeriods::embedded();
    // 逐历检验：历名 → [推步月数, 与张表同, 本纪条数, 合推步, 合张表, 从本纪修正]
    let eval = read("data/calendar/main_series_reckoning_eval.csv");
    let mut lines = eval.lines();
    let head: Vec<&str> = lines.next().unwrap().split(',').collect();
    let col = |n: &str| head.iter().position(|h| *h == n).unwrap();
    let (cn, cm, cs) = (col("历"), col("月数"), col("与张表同"));
    let (cb, cr, ct, cf) = (col("本纪条数"), col("本纪合推步"), col("本纪合张表"), col("从本纪修正"));
    let mut benji: HashMap<String, [String; 6]> = HashMap::new();
    for l in lines {
        let c: Vec<&str> = l.split(',').collect();
        benji.insert(c[cn].to_string(), [cm, cs, cb, cr, ct, cf].map(|k| c[k].to_string()));
    }
    // (历, 来源类别, 来源名, 起, 止)
    let mut segs: Vec<(&'static str, &'static str, String, i32, i32)> = Vec::new();
    for y in -220..=2100 {
        let name: &'static str = periods.name_for_lunar_year(y);
        let (kind, src) = match main_series_year_months(Some(&zhang), y, MainSeriesPreference::default()) {
            Some((_, RiliSource::Artifact(n))) => ("实物历日", n.to_string()),
            Some((_, RiliSource::Reckoned(n))) => ("按术推步", n.to_string()),
            Some((_, RiliSource::AncientTable(n))) => ("古人长历", n.to_string()),
            Some((_, RiliSource::ModernTable(n))) => ("今人历表", n.to_string()),
            Some((_, RiliSource::Computed)) | None => ("现代定气定朔现算", String::new()),
        };
        match segs.last_mut() {
            Some(s) if s.0 == name && s.1 == kind && s.2 == src && s.4 == y - 1 => s.4 = y,
            _ => segs.push((name, kind, src, y, y)),
        }
    }
    let mut out = String::from(
        "# 主序列逐段取法（缺省取法 MainSeriesPreference::default，第 4 层 lifa::main_series_year_months 实际所给来源，逐年连段）。\n\
         # 由 rust/core/examples/main_series_qufa.rs 生成，勿手改；推步月数以下各列取自 main_series_reckoning_eval.csv（全历合计，同一历名各段共用一组数；推步月数含实物年照推之月）。\n\
         历,起,止,来源,来源名,推步月数,推步与张表同月,本纪条数,本纪合推步,本纪合张表,从本纪修正\n",
    );
    for (name, kind, src, a, b) in &segs {
        let v = benji.get(*name).cloned().unwrap_or_default();
        out += &format!("{name},{a},{b},{kind},{src},{}\n", v.join(","));
    }
    std::fs::write(root.join("data/calendar/main_series_qufa.csv"), &out).unwrap();
    println!("写 data/calendar/main_series_qufa.csv：{} 段", segs.len());
    for (name, kind, src, a, b) in &segs {
        println!("{a:>5}—{b:<5} {name:<24} {kind} {src}");
    }
}
