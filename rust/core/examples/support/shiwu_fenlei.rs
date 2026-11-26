//! 实物比对的分类（与 scripts/calendar_data/shiwu_fenlei.py 同一规则）：推步（或该段所用复原）与张表逐月相同、互异，按实物分类。
//! 互异：①实物证张表误（合推步）、②实物证推步误（合张表）、③无据待核；相同：有证、同误（实物两家皆不合）、无证。
//! 证据等级 A 实物历日、B 当时文字、C 史书明书朔一条即可判，D 纪日须两条以上独立。
#![allow(dead_code)]

use lunar_core::calendar::gan_zhi::GAN_ZHI_60;

pub const CATS: [&str; 6] = ["有证", "同误", "无证", "①", "②", "③"];

fn idx(g: &str) -> Option<i64> {
    GAN_ZHI_60.iter().position(|x| *x == g).map(|i| i as i64)
}

/// 所记朔与两家之朔相差几日 → 判语
pub fn off_note(o: &str, r: &str) -> String {
    match (idx(o), idx(r)) {
        (None, _) => "待核".into(),
        (_, None) => "两家皆无此月，待核".into(),
        (Some(a), Some(b)) => {
            let d = (a - b + 30).rem_euclid(60) - 30;
            if d.abs() == 29 || d.abs() == 30 {
                "合前后月：月名、年号抄误或闰位异".into()
            } else if d.abs() <= 2 {
                format!("差 {d:+} 日，待核")
            } else {
                format!("差 {d:+} 日，判抄误或释读误")
            }
        }
    }
}

/// tui、zhang：两家该月朔干支（空 = 无此月）；obs：独立证据所记之朔（已剔除不作证据之条）。→ (分类, 注)；两家缺一返回 None。
pub fn classify(tui: &str, zhang: &str, obs: &[&str], grade: char, what: &str) -> Option<(&'static str, String)> {
    if tui.is_empty() || zhang.is_empty() {
        return None;
    }
    let need = if grade == 'D' { 2 } else { 1 };
    let n = obs.len();
    let p = obs.iter().filter(|o| **o == tui).count();
    let z = obs.iter().filter(|o| **o == zhang).count();
    let rest = |k: usize, s: &str| if k < n { format!("另 {} 条{s}", n - k) } else { String::new() };
    Some(if tui == zhang {
        if n == 0 {
            ("无证", String::new())
        } else if p >= need {
            ("有证", rest(p, "不合"))
        } else if p == 0 && n >= need {
            ("同误", format!("{what}与两家皆不合，{}", off_note(obs[0], tui)))
        } else {
            ("无证", format!("{what}一见"))
        }
    } else if p > 0 && z > 0 {
        ("③", format!("证据冲突：合推步 {p}、合张表 {z}"))
    } else if p >= need {
        ("①", rest(p, "两家皆不合"))
    } else if z >= need {
        ("②", rest(z, "两家皆不合"))
    } else if p + z > 0 {
        ("③", format!("{what}一见，倾向{}", if p > 0 { "推步" } else { "张表" }))
    } else if n > 0 {
        ("③", format!("{what}与两家皆不合"))
    } else {
        ("③", String::new())
    })
}

pub fn label(c: &(&'static str, String)) -> String {
    if c.1.is_empty() { c.0.to_string() } else { format!("{}（{}）", c.0, c.1) }
}

/// [(分类, 等级)] → 一句汇总
pub fn headline(cats: &[(&str, char)], tui: &str) -> String {
    headline_unit(cats, tui, "月")
}

pub fn headline_unit(cats: &[(&str, char)], tui: &str, u: &str) -> String {
    let c = |k: &str| cats.iter().filter(|x| x.0 == k).count();
    format!(
        "{tui}与张表**相同 {}** {u}（有证 {}／同误 {}／无证 {}），**互异 {}** {u}（①证张表误 {}／②证{tui}误 {}／③无据待核 {}）",
        c("有证") + c("同误") + c("无证"),
        c("有证"),
        c("同误"),
        c("无证"),
        c("①") + c("②") + c("③"),
        c("①"),
        c("②"),
        c("③")
    )
}

/// 按证据等级给数（等级「—」= 无证据之月）
pub fn grade_table(cats: &[(&str, char)]) -> Vec<String> {
    let mut gs: Vec<char> = cats.iter().map(|x| x.1).collect();
    gs.sort();
    gs.dedup();
    let mut out = vec![
        "| 等级 | 有证 | 同误 | 无证 | ①证张表误 | ②证推步误 | ③无据待核 |".to_string(),
        "|---|---|---|---|---|---|---|".into(),
    ];
    let row = |f: &dyn Fn(&(&str, char)) -> bool| CATS.iter().map(|k| cats.iter().filter(|x| x.0 == *k && f(x)).count().to_string()).collect::<Vec<_>>().join(" | ");
    for g in gs {
        out.push(format!("| {g} | {} |", row(&|x| x.1 == g)));
    }
    out.push(format!("| 合计 | {} |", row(&|_| true)));
    out
}

pub const RULE: &str = "分类（每月一个）：两家**相同**之月——有证（实物合两家）、同误（实物与两家皆不合，判实物抄误或待核）、无证（两家相同而已，\
或只有推得、定年等不作证据之条）；两家**互异**之月——①实物证张表误（合推步）、②实物证推步误（合张表）、③无据待核\
（实物两家皆不合、证据冲突，或只一见而等级不足）。实物本身是证据，复原（今人诸表、外推）只作比对。";
