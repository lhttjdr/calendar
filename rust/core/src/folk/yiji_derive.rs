//! 协纪月表婚事（結婚姻、嫁娶）宜忌之推法，可略去若干神煞重推。
//!
//! 月表（`data/folk/xieji_yuebiao.csv`）由 `scripts/folk/xieji_yuebiao.py` 按卷十一（各事宜神忌神）、卷十（相併相抵、
//! 宜忌等第表、鋪註條例）推出。本模块把其中与婚事有关的部分移植过来：当日诸神照月表所列，去掉调用方「不看」之神后
//! 重推結婚姻、嫁娶之宜忌。诸神全看时与月表 720 日全同（`tests::matches_yuebiao_all_days`）。
//!
//! 之所以只移植婚事：月表诸事之间牵连（鋪註條例「吉凶相抵不註忌甲亦不註忌乙」），婚事一组（結婚姻→嫁娶）自成一环，
//! 其余诸事不影响嫁娶之注。

use super::yiji::{shi_shen, yuebiao_day, YueBiaoDay};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

const GAN: &str = "甲乙丙丁戊己庚辛壬癸";
const ZHI: &str = "子丑寅卯辰巳午未申酉戌亥";
const DENGDI_CSV: &str = include_str!("../../../../data/folk/yiji_dengdi.csv");

/// 婚事两事
pub const HUN_SHI: [&str; 2] = ["結婚姻", "嫁娶"];

fn norm(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '刼' => '劫',
            '夘' => '卯',
            '隂' => '陰',
            '徳' => '德',
            '虚' => '虛',
            '摇' => '搖',
            '絶' => '絕',
            c => c,
        })
        .collect()
}

/// 卷十宜忌等第表：(主凶神, 月支) → 等第
static DENGDI: Lazy<HashMap<(String, char), &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    for line in DENGDI_CSV.lines().skip(1) {
        let f: Vec<&'static str> = line.split(',').collect();
        if f.len() < 4 {
            continue;
        }
        for z in f[1].chars() {
            m.insert((norm(f[0]), z), f[3]);
        }
    }
    m
});

const DE: [&str; 6] = ["天德", "月德", "天德合", "月德合", "天赦", "天願"];
const DE4: [&str; 4] = ["天德", "月德", "天德合", "月德合"];
const MAJOR: [&str; 11] = ["平日", "收日", "閉日", "劫煞", "災煞", "月煞", "月刑", "月害", "月厭", "大時", "天吏"];
const MAJOR_ALIAS: [&str; 10] = ["死神", "天火", "月虛", "大敗", "咸池", "致死", "地火", "血支", "河魁", "天罡"];
const GRADE_ORDER: [&str; 6] = ["上", "上次", "中", "中次", "下", "下下"];
/// 月厭遇德「止忌行幸……結婚姻……栽種」中之婚事
const RESIDUAL_YAN: [&str; 2] = ["結婚姻", "嫁娶"];
const ALLBAD: [&str; 14] = [
    "小會", "單陰", "純陽", "孤陽", "純陰", "陰位", "陰陽交破", "陰陽擊衝", "陽破陰衝", "陰道衝陽", "三陰", "陰陽俱錯", "絕陰", "絕陽",
];
/// 婚事「遇亥日厭對八專四忌四窮而仍註忌嫁娶者，止註所忌之事，其不忌者仍不註忌」
const HUN_ZHUANJI: [&str; 5] = ["亥日", "厭對", "八專", "四忌", "四窮"];

/// 卷十各条「與某併則不忌」（只取婚事忌神中有者）
fn jie_of(n: &str) -> Vec<&'static str> {
    let mut v: Vec<&'static str> = DE4.to_vec();
    match n {
        "厭對" => v.push("天赦"),
        "五離" => v.extend(["天赦", "三合", "六合"]),
        "八專" => return vec!["天願"],
        _ => return vec![],
    }
    v
}

/// 婚事之注
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zhu {
    Yi,
    Ji,
    /// 两不注
    Neither,
    /// 諸事不宜（特定日、皆忌）
    AllBad,
}

/// 推一日（月序 `m` 寅月 = 0，干支序 `k`）婚事之注。`names` 为当日所值诸神（月表吉神凶神所列，加建除「某日」）；
/// 返回 (結婚姻, 嫁娶, 等第说明)
pub fn derive(m: u8, k: u8, names: &HashSet<String>) -> (Zhu, Zhu, String) {
    let has = |n: &str| names.contains(n);
    let any = |l: &[&str]| l.iter().any(|n| names.contains(*n));
    let gz: String = [GAN.chars().nth(k as usize % 10).unwrap(), ZHI.chars().nth(k as usize % 12).unwrap()].iter().collect();
    let mz = ZHI.chars().nth((m as usize + 2) % 12).unwrap();
    // 德
    let mut de: HashSet<&str> = DE.iter().copied().filter(|n| has(n)).collect();
    if any(&["陽錯", "陰錯", "陰陽俱錯"]) || (has("大會") && (m == 4 || m == 10)) {
        de.retain(|n| *n == "天赦");
    }
    if any(&["歲薄", "逐陣"]) {
        de.clear();
    }
    // 特定日：諸事皆忌
    if any(&ALLBAD) || (has("大會") && !((m == 4 || m == 10) && has("天赦"))) {
        return (Zhu::AllBad, Zhu::AllBad, "特定日→皆忌".into());
    }
    if has("四廢") && has("月破") {
        return (Zhu::AllBad, Zhu::AllBad, "四廢併月破→皆忌".into());
    }
    // 宜、忌之源
    let lists: Vec<(Vec<&'static str>, Vec<&'static str>)> = HUN_SHI.iter().map(|s| shi_shen(s)).collect();
    let mut yi: HashSet<usize> = (0..2).filter(|&i| lists[i].0.iter().any(|n| has(n))).collect();
    let mut src: Vec<HashSet<String>> = (0..2)
        .map(|i| lists[i].1.iter().filter(|n| has(n)).map(|n| n.to_string()).collect())
        .collect();
    // 百忌日「亥不嫁娶」（「亥日」由 names_of 按日支加入，可略去）
    if has("亥日") {
        src[1].insert("亥日".into());
    }
    // 等第
    let present: Vec<&str> = MAJOR.iter().copied().filter(|n| has(n)).collect();
    let present_g: Vec<&str> = if any(&["四廢", "歲薄", "逐陣"]) {
        present.iter().copied().filter(|n| *n != "天吏" && *n != "大時").collect()
    } else {
        present.clone()
    };
    let grade = present_g
        .iter()
        .filter_map(|n| DENGDI.get(&(n.to_string(), mz)).copied())
        .max_by_key(|g| GRADE_ORDER.iter().position(|x| x == g).unwrap_or(0));
    let mut has_de = !de.is_empty();
    if present.contains(&"月厭") && (mz == '子' || mz == '午') {
        has_de = has("天赦");
    }
    let mode = match grade {
        None => "從忌",
        Some("上") => "從宜",
        Some("上次" | "中") if has_de => "從宜",
        Some("上次") if !has_de => "相抵",
        Some("中次") if has_de => "相抵",
        Some("中" | "中次") => "從忌",
        Some("下") if has_de => "從忌",
        _ => "皆忌",
    };
    // 次要凶神之解（德作建論之日，德不算）
    let jnames: HashSet<&str> = names.iter().map(|s| s.as_str()).filter(|n| !DE.contains(n) || de.contains(n)).collect();
    for s in src.iter_mut() {
        s.retain(|n| {
            let j = jie_of(n);
            !(j.iter().any(|x| jnames.contains(x)))
        });
    }
    // 四忌四窮：惟正月乙亥與天願併，止忌軍事
    if m == 0 && gz == "乙亥" && has("天願") {
        for s in src.iter_mut() {
            s.remove("四窮");
            s.remove("四忌");
        }
    }
    // 五墓：五月十一月與月德併則不忌
    if has("五墓") && (m == 4 || m == 10) && has("月德") {
        for s in src.iter_mut() {
            s.remove("五墓");
        }
    }
    // 月建與德等併止忌土功（德作建論之日不解）
    if has("月建")
        && ["天德", "月德", "天德合", "月德合", "天赦", "月恩", "四相"].iter().any(|n| jnames.contains(n))
        && !any(&MAJOR)
        && !any(&["陽錯", "陰錯", "陰陽俱錯"])
    {
        for s in src.iter_mut() {
            s.remove("月建");
            s.remove("小時");
        }
    }
    // 滿日：寅申月值守日、子午卯酉月值相日，與月德天德合月德合月恩四相併則不忌
    if has("滿日") && "寅申子午卯酉".contains(mz) && any(&["月德", "天德合", "月德合", "月恩", "四相"]) {
        for s in src.iter_mut() {
            s.remove("滿日");
        }
    }
    let mut major_set: HashSet<&str> = MAJOR.iter().chain(MAJOR_ALIAS.iter()).copied().collect();
    if grade.is_some() {
        major_set.extend(["月建", "小時", "土府", "月破", "大耗", "滿日"]);
    }
    let is_major = |n: &str| major_set.contains(n);
    let mut ji_major: HashSet<usize> = (0..2).filter(|&i| src[i].iter().any(|n| is_major(n))).collect();
    let ji_minor: HashSet<usize> = (0..2).filter(|&i| src[i].iter().any(|n| !is_major(n))).collect();
    let mut note = String::new();
    if let Some(g) = grade {
        note = format!("{g}{}→{mode}", if has_de { "遇德" } else { "" });
        if mode == "皆忌" {
            return (Zhu::AllBad, Zhu::AllBad, note);
        }
        // 月厭遇德仍忌婚事；大時只忌军事；余止忌求醫療病（非婚事）
        let residual: Vec<usize> = if present.contains(&"月厭") { (0..2).filter(|&i| RESIDUAL_YAN.contains(&HUN_SHI[i])).collect() } else { vec![] };
        match mode {
            "從宜" => ji_major.retain(|i| residual.contains(i)),
            "相抵" => {
                let both: HashSet<usize> = ji_major.intersection(&yi).copied().collect();
                for i in &both {
                    yi.remove(i);
                }
                ji_major = ji_major.iter().copied().filter(|i| !both.contains(i) || residual.contains(i)).collect();
                if present.contains(&"大時") && present.iter().all(|n| *n == "大時" || *n == "收日") {
                    ji_major.retain(|&i| src[i].iter().any(|n| !["大時", "大敗", "咸池"].contains(&n.as_str())));
                }
                let mut freed: HashSet<usize> = both
                    .iter()
                    .copied()
                    .filter(|i| !residual.contains(i))
                    .filter(|&i| !ji_minor.contains(&i) || src[i].iter().filter(|n| !is_major(n)).all(|n| HUN_ZHUANJI.contains(&n.as_str())))
                    .collect();
                // 「遇不將而不註忌嫁娶者亦仍註忌」：嫁娶只由不將放开时不连带
                if freed.contains(&1) && !lists[1].0.iter().any(|n| *n != "不將" && has(n)) {
                    freed.remove(&1);
                }
                // 吉凶相抵不註忌甲亦不註忌乙：結婚姻 ⇄ 嫁娶
                if freed.contains(&0) {
                    ji_major.remove(&1);
                }
                if freed.contains(&1) {
                    ji_major.remove(&0);
                }
            }
            _ => {
                for i in &ji_major {
                    yi.remove(i);
                }
            }
        }
    } else {
        for i in &ji_major {
            yi.remove(i);
        }
    }
    for i in &ji_minor {
        yi.remove(i);
    }
    let mut ji: HashSet<usize> = ji_major.union(&ji_minor).copied().collect();
    // 鋪註條例：德合與赦願所會之辰，諸事不忌
    const ZHUSHI_BUJI: [(u8, &str); 8] = [(1, "甲戌"), (3, "丙申"), (5, "甲午"), (6, "戊申"), (7, "庚辰"), (8, "辛卯"), (9, "甲子"), (11, "甲子")];
    if ZHUSHI_BUJI.iter().any(|(mm, g)| *mm == m && *g == gz) {
        for i in &ji {
            yi.remove(i);
        }
        ji.clear();
    }
    for i in &ji {
        yi.remove(i);
    }
    // 歲薄、逐陣：所宜事照月厭所忌刪（婚事皆月厭所忌）
    if any(&["歲薄", "逐陣"]) {
        yi.clear();
    }
    // 癸亥干支俱盡，止註祭祀等，婚事不註宜
    if gz == "癸亥" {
        yi.clear();
    }
    let zhu = |i: usize| if yi.contains(&i) { Zhu::Yi } else if ji.contains(&i) { Zhu::Ji } else { Zhu::Neither };
    (zhu(0), zhu(1), note)
}

/// 月表一日所列诸神（吉神、凶神，加建除「某日」；亥日加「亥日」，百忌日「亥不嫁娶」）
pub fn names_of(d: &YueBiaoDay, k: u8) -> HashSet<String> {
    let mut s: HashSet<String> = d.ji_shen.iter().chain(d.xiong_shen.iter()).map(|n| norm(n)).collect();
    s.insert(format!("{}日", d.jian_chu));
    if k % 12 == 11 {
        s.insert("亥日".into());
    }
    s
}

/// 可略去（「不看」）之神：婚事两事之宜神、忌神（卷十一），末加卷十月厭條「諸事皆忌」之日（大會、小會、單陰……各名分列）。
/// 下标即 [`JiaquOptions::ignore_shen`](super::jiaqu::JiaquOptions) 位
pub static HUN_SHEN: Lazy<Vec<(&'static str, bool)>> = Lazy::new(|| {
    let mut v: Vec<(&'static str, bool)> = Vec::new();
    for s in HUN_SHI {
        let (yi, ji) = shi_shen(s);
        for n in yi {
            if !v.iter().any(|(x, _)| *x == n) {
                v.push((n, true));
            }
        }
        for n in ji {
            if !v.iter().any(|(x, _)| *x == n) {
                v.push((n, false));
            }
        }
    }
    // 卷十月厭條：陰陽大會（與月德併仍諸事皆忌）、小會、單陰、純陽……「亦諸事皆忌」
    v.push(("大會", false));
    for n in ALLBAD {
        if !v.iter().any(|(x, _)| *x == n) {
            v.push((n, false));
        }
    }
    assert!(v.len() <= 64, "{}", v.len());
    v
});

/// 位掩码 → 所略诸神名
pub fn ignore_names(mask: u64) -> Vec<String> {
    HUN_SHEN.iter().enumerate().filter(|(i, _)| mask >> i & 1 == 1).map(|(_, (n, _))| n.to_string()).collect()
}

/// 略去 `ignore` 诸神后的月表一日：婚事两事之宜忌按重推，余事照月表。`ignore` 空时即月表
pub fn yuebiao_day_ignoring(m: u8, k: u8, ignore: &[String]) -> YueBiaoDay {
    let base = yuebiao_day(m, k);
    if ignore.is_empty() {
        return base.clone();
    }
    let mut names = names_of(base, k);
    for n in ignore {
        names.remove(&norm(n));
    }
    let (a, b, note) = derive(m, k, &names);
    let mut d = base.clone();
    d.deng_di = note;
    if a == Zhu::AllBad {
        // 原月表諸事不宜而今可推者，仍只改婚事，余事从缺（不注）
        d.yi = Some(Vec::new());
    }
    let yi = d.yi.get_or_insert_with(Vec::new);
    for (shi, z) in HUN_SHI.iter().zip([a, b]) {
        yi.retain(|x| x != shi);
        d.ji.retain(|x| x != shi);
        match z {
            Zhu::Yi => yi.push(shi.to_string()),
            Zhu::Ji | Zhu::AllBad => d.ji.push(shi.to_string()),
            Zhu::Neither => {}
        }
    }
    d.ji_shen.retain(|n| !ignore.iter().any(|x| norm(x) == norm(n)));
    d.xiong_shen.retain(|n| !ignore.iter().any(|x| norm(x) == norm(n)));
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zhu_of(d: &YueBiaoDay, shi: &str) -> Zhu {
        match &d.yi {
            None => Zhu::AllBad,
            Some(v) if v.iter().any(|x| x == shi) => Zhu::Yi,
            _ if d.ji.iter().any(|x| x == shi) => Zhu::Ji,
            _ => Zhu::Neither,
        }
    }

    /// 诸神全看：結婚姻、嫁娶之注与月表 720 日全同
    #[test]
    fn matches_yuebiao_all_days() {
        let mut bad = Vec::new();
        for m in 0..12u8 {
            for k in 0..60u8 {
                let d = yuebiao_day(m, k);
                let (a, b, _) = derive(m, k, &names_of(d, k));
                let (ea, eb) = (zhu_of(d, "結婚姻"), zhu_of(d, "嫁娶"));
                if a != ea || b != eb {
                    bad.push(format!("{}月{}{}：推 {a:?}/{b:?}，月表 {ea:?}/{eb:?}；{}", m + 1, GAN.chars().nth(k as usize % 10).unwrap(), ZHI.chars().nth(k as usize % 12).unwrap(), d.deng_di));
                }
            }
        }
        assert!(bad.is_empty(), "{} 日不合：\n{}", bad.len(), bad.join("\n"));
    }

    /// 不看「月厭」：原因月厭忌嫁娶之日，若无他忌神则不再忌
    #[test]
    fn ignore_changes_result() {
        let mut changed = 0;
        for m in 0..12u8 {
            for k in 0..60u8 {
                let d = yuebiao_day(m, k);
                let e = yuebiao_day_ignoring(m, k, &["月厭".to_string()]);
                if zhu_of(d, "嫁娶") != zhu_of(&e, "嫁娶") {
                    changed += 1;
                }
            }
        }
        assert!(changed > 0);
    }
}
