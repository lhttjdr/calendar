//! 中原一线定朔诸历（619—1911）按术推步与张培瑜历表朔日不同之月逐月考证。
//!
//! 对推步与张表朔日相差一日之月，逐月列：定朔所在日与定小余、本历进朔限、距限、经朔，张表与推步的进退关系，以及证据——
//! 本纪明书朔日（`data/calendar/benji_shuo_check.csv`，已考讹误者另注）、本纪纪日（`data/calendar/本纪纪日/*.csv`：某日干支只在
//! 一家的该月或前月之内者）、已有考证与史载例外表；再按规则给出判定，人工按语取 `data/calendar/dingshuo_zhangbiao_notes.csv`。
//! 闰位不同之岁另表。推步取法同 `examples/zhongyuan_reckoning_eval.rs`（直接按术推步，不经仲裁；实物年也照推）。
//!
//! 输出 `data/calendar/dingshuo_zhangbiao_diffs.csv`，并改写 `doc/calendar/dingshuo-zhangbiao-diffs.md` 中
//! `<!-- 生成开始 -->`、`<!-- 生成结束 -->` 之间的部分（其余为手写）。
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example dingshuo_zhangbiao_diffs
use lunar_core::calendar::dingshuo::{dingshuo_months, DingshuoMonth};
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::historical::ShuoriTable;
use lunar_core::calendar::lifa::{zhongyuan_year_months, RiliSource, ZhongyuanCalendarPeriods, ZhongyuanPreference};
use lunar_core::calendar::month_labels::{standard_month_name, MonthLabel, MonthLabelReforms};
use lunar_core::calendar::push_calendar;
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// 带引号与换行的 CSV
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

/// 中原一线历名 → 定朔细目所用之历名（[dingshuo_months]）；None 为不在 dingshuo 模块之历
fn ding_name(name: &str) -> Option<&str> {
    Some(match name {
        "戊寅元历（武德术不传，以贞观所较戊寅元历代推）" => "戊寅元历（代推武德）",
        "会天历（术不传，以开禧历代推）" => "开禧历（代推会天）",
        "占天历（术不传，以纪元历代推）" => "纪元历（代推占天）",
        "淳祐历（术不传，以开禧历代推）" => "开禧历（代推淳祐）",
        "本天历（术不传，以成天历代推）" => "成天历（代推本天）",
        "奉元历（术不传，李锐补修气朔、借观天日月改正）" => "奉元历（李锐补修气朔，借观天日月改正）",
        "历术失载（以纪元历代推）" => "纪元历（代推南渡初）",
        "大统历（康熙复用）" => "大统历",
        "应天历" | "乾元历" | "仪天历" | "钦天历（日躔月离表不存，以应天定朔代推）" | "时宪历" | "重修大明历" | "时宪历（历象考成）"
        | "时宪历（历象考成后编）" => return None,
        n => n,
    })
}

#[derive(Clone)]
struct Diff {
    li: &'static str,
    year: i32,
    month: u8,
    leap: bool,
    /// 张表该月月名不同（闰位所致）时之张表月名
    t_label: Option<(u8, bool)>,
    r: i64,
    t: i64,
    /// 定朔细目（定小余、进朔限等，时）
    ding: Option<(i64, f64, Option<f64>, bool, f64)>,
    /// 经朔入交入食限（「朔望有交」，大衍一系此等月用精法）；None 为未复原交会
    youjiao: Option<bool>,
    /// 本纪明书朔日：(书, 本纪朔 JDN, 已考讹误)
    shuo: Vec<(String, i64, bool)>,
    /// 纪日：(书, 支持推步, 原文)
    jiri: Vec<(String, bool, String)>,
    /// 已有考证、例外表
    refs: Vec<String>,
}

fn label(m: u8, leap: bool) -> String {
    format!("{}{}", if leap { "闰" } else { "" }, m)
}

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    let zhang = ShuoriTable::from_csv(&read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let periods = ZhongyuanCalendarPeriods::embedded();
    let reforms = MonthLabelReforms::from_csv(&read("data/calendar/month_label_reforms.csv")).unwrap();
    let (y_from, y_to) = (619, 1911);

    // 逐年推步与张表
    let mut reck: BTreeMap<i32, Vec<(i64, u8, bool)>> = BTreeMap::new();
    let mut names: BTreeMap<i32, &'static str> = BTreeMap::new();
    for y in y_from..=y_to {
        let name: &'static str = periods.name_for_lunar_year(y);
        let r = match zhongyuan_year_months(None, y, ZhongyuanPreference::Reckoning) {
            Some((r, RiliSource::Reckoned(_))) => r,
            Some((_, RiliSource::Artifact(_))) => match push_calendar::reckon_lifa_year(name, y) {
                Some(r) => r,
                None => continue,
            },
            _ => continue,
        };
        reck.insert(y, r);
        names.insert(y, name);
    }
    let table = |y: i32| -> Vec<(i64, u8, bool)> { zhang.year_months(y).map(|t| t.to_vec()).unwrap_or_default() };

    // 定朔细目
    let mut details: HashMap<&str, Vec<DingshuoMonth>> = HashMap::new();
    for n in names.values() {
        if let Some(d) = ding_name(n) {
            details.entry(d).or_insert_with(|| dingshuo_months(d).unwrap_or_else(|| panic!("{d}")));
        }
    }
    // 宋初三历：song_early_check.csv（定朔小余、进朔限，时）
    let mut song_early: HashMap<(String, i32, String), (f64, f64)> = HashMap::new();
    for r in parse_csv(&read("data/calendar/song_early_check.csv")).into_iter().filter(|r| r.len() > 11 && r[0] == "月") {
        let lab = if r[4] == "1" { format!("闰{}", r[3]) } else { r[3].clone() };
        if let (Ok(a), Ok(b)) = (r[10].parse::<f64>(), r[11].parse::<f64>()) {
            song_early.insert((r[1].clone(), r[2].parse().unwrap(), lab), (a, b));
        }
    }

    // 逐月比较：按朔日相近配对（相差至多一日），同日而月名不同属闰位，另计
    let mut diffs: Vec<Diff> = Vec::new();
    let mut leap_years: Vec<(i32, &'static str, String, String)> = Vec::new();
    let mut stat: BTreeMap<&'static str, (i32, i32, usize, usize, usize)> = BTreeMap::new();
    for (&y, r) in &reck {
        let name = names[&y];
        let t = table(y);
        let s = stat.entry(name).or_insert((y, y, 0, 0, 0));
        s.1 = y;
        s.2 += r.len();
        let lr: Vec<_> = r.iter().filter(|m| m.2).map(|m| m.1).collect();
        let lt: Vec<_> = t.iter().filter(|m| m.2).map(|m| m.1).collect();
        if lr != lt {
            s.4 += 1;
            let f = |v: &[u8]| if v.is_empty() { "无".to_string() } else { v.iter().map(|m| format!("闰{m}")).collect::<Vec<_>>().join("、") };
            leap_years.push((y, name, f(&lr), f(&lt)));
        }
        for &(rj, m, leap) in r {
            let Some(&(tj, tm, tl)) = t.iter().min_by_key(|e| (e.0 - rj).abs()) else { continue };
            if tj == rj || (tj - rj).abs() > 2 {
                continue;
            }
            s.3 += 1;
            let dn = ding_name(name);
            let ding = dn.and_then(|d| {
                let v = &details[d];
                let x = v.iter().find(|x| x.day == rj).or_else(|| v.iter().find(|x| (x.ding_jdn - rj.min(tj)).abs() <= 1))?;
                Some((x.ding_jdn, x.frac * 24.0, x.limit.map(|l| l * 24.0), x.jin, x.jing_frac * 24.0 + (x.jing_jdn - x.ding_jdn) as f64 * 24.0))
            });
            let youjiao = dn.and_then(|d| {
                let v = &details[d];
                v.iter().find(|x| x.day == rj).or_else(|| v.iter().find(|x| (x.ding_jdn - rj.min(tj)).abs() <= 1)).and_then(|x| x.youjiao)
            });
            let ding = ding.or_else(|| {
                let (a, b) = song_early.get(&(name.to_string(), y, label(m, leap)))?;
                // 定朔所在日：小余以时计，朔日 = 所在日 + 进
                let jin = a >= b;
                Some((rj - i64::from(jin), *a, Some(*b), jin, f64::NAN))
            });
            diffs.push(Diff {
                li: name,
                year: y,
                month: m,
                leap,
                t_label: ((tm, tl) != (m, leap)).then_some((tm, tl)),
                r: rj,
                t: tj,
                ding,
                youjiao,
                shuo: vec![],
                jiri: vec![],
                refs: vec![],
            });
        }
    }

    // 本纪明书朔日
    let rows = parse_csv(&read("data/calendar/benji_shuo_check.csv"));
    let head = rows.iter().find(|r| r.first().is_some_and(|c| c == "书")).unwrap().clone();
    let col = |n: &str| head.iter().position(|h| h == n).unwrap();
    let (cb, cy, cm, cr, cs) = (col("书"), col("年"), col("月"), col("本纪朔 JDN"), col("政权"));
    let errata: BTreeSet<(String, String, String)> = parse_csv(&read("data/calendar/benji_shuo_errata.csv"))
        .into_iter()
        .filter(|r| r.len() >= 3 && !r[0].starts_with('#') && r[0] != "书")
        .map(|r| (r[0].clone(), r[1].clone(), r[2].clone()))
        .collect();
    let skip_books = ["liaoshi_benji", "jinshi_benji", "weishu_benji", "beiqishu_benji", "zhoushu_benji"];
    let to_table = |regime: &str, y: i32, name: &str, leap: bool| -> Option<(i32, u8, bool)> {
        let name = name.replace('臘', "腊");
        let l = MonthLabel { year: y, name, is_leap: leap };
        reforms.table_of(regime, &l).map(|t| (t.lunar_year, t.month, t.is_leap))
    };
    for r in rows.iter().filter(|r| r.len() == head.len() && r[0] != "书") {
        if skip_books.contains(&r[cb].as_str()) || ["吴", "蜀"].contains(&r[cs].as_str()) {
            continue;
        }
        let (Ok(y), Ok(rec)) = (r[cy].parse::<i32>(), r[cr].parse::<i64>()) else { continue };
        if r[cs] == "元" && y < 1280 {
            continue;
        }
        let (leap, m) = match r[cm].strip_prefix('闰').or_else(|| r[cm].strip_prefix('閏')) {
            Some(x) => (true, x),
            None => (false, r[cm].as_str()),
        };
        let Ok(m) = m.parse::<u8>() else { continue };
        let Some((ty, tm, tl)) = to_table(&r[cs], y, standard_month_name(m), leap) else { continue };
        let er = errata.contains(&(r[cb].clone(), r[cy].clone(), r[cm].clone()));
        for d in diffs.iter_mut().filter(|d| d.year == ty && d.month == tm && d.leap == tl) {
            d.shuo.push((r[cb].replace("_benji", ""), rec, er));
        }
    }

    // 本纪纪日：日干支为两家较早之朔者，在该月则只合较早一家，在前月则只合较晚一家（非书朔之条）
    let books = ["jiutangshu", "xintangshu", "jiuwudaishi", "xinwudaishi", "songshi", "yuanshi", "mingshi", "qingshigao"];
    // (表年, 月, 闰) → 前一月 (表年, 月, 闰)，按推步排
    let mut prev: HashMap<(i32, u8, bool), (i32, u8, bool)> = HashMap::new();
    let mut last: Option<(i32, u8, bool)> = None;
    for (&y, r) in &reck {
        for &(_, m, l) in r {
            if let Some(p) = last {
                prev.insert((y, m, l), p);
            }
            last = Some((y, m, l));
        }
    }
    let mut idx: HashMap<(i32, u8, bool), Vec<usize>> = HashMap::new();
    for (i, d) in diffs.iter().enumerate() {
        if d.t_label.is_none() {
            idx.entry((d.year, d.month, d.leap)).or_default().push(i);
        }
    }
    let mut next_of: HashMap<(i32, u8, bool), (i32, u8, bool)> = HashMap::new();
    for (k, p) in &prev {
        next_of.insert(*p, *k);
    }
    for b in books {
        let recs = parse_csv(&read(&format!("data/calendar/本纪纪日/{b}_benji.csv")));
        for r in recs.iter().skip(1).filter(|r| r.len() >= 15) {
            let (Ok(y), Ok(m)) = (r[3].parse::<i32>(), r[4].parse::<u8>()) else { continue };
            if r[12] == "朔" || (r[0] == "元" && y < 1280) {
                continue;
            }
            let leap = r[6] == "1";
            let name = if r[5].is_empty() { standard_month_name(m).to_string() } else { r[5].clone() };
            let Some(tab) = to_table(&r[0], y, &name, leap).or_else(|| to_table(&r[0], y, standard_month_name(m), leap)) else { continue };
            let Some(g) = GAN_ZHI_60.iter().position(|x| *x == r[7]) else { continue };
            let text: String = r[14..].join("，").chars().take(60).collect();
            // 本月或下月为考证之月
            for (k, in_prev) in [(tab, false), (next_of.get(&tab).copied().unwrap_or((0, 0, false)), true)] {
                let Some(is) = idx.get(&k) else { continue };
                for &i in is {
                    let d = &mut diffs[i];
                    let e = d.r.min(d.t);
                    if (g as i64 - (e + 49)).rem_euclid(60) != 0 {
                        continue;
                    }
                    // 在本月：只合较早一家；在前月：只合较晚一家
                    let early_is_r = d.r < d.t;
                    let sup_r = if in_prev { !early_is_r } else { early_is_r };
                    let tag = format!("{b}{}", if in_prev { "（前月）" } else { "" });
                    d.jiri.push((tag, sup_r, text.clone()));
                }
            }
        }
    }

    // 节日纪日：节日在月内日序固定者（中和节二月一日、上巳三月三日、重阳九月九日），由日干支逆推该月朔。只认明书节名之条，
    // 罢、停、禁、贡、改者不算（曲江赐宴、「九日宴」未必在节日当天：《旧唐书》794 年「賜百僚九日宴」、795 年九月曲江宴皆不在九日）。
    let fest_day = |_y: i32, m: u8, text: &str| -> Option<i64> {
        if ['罷', '停', '禁', '貢', '改'].iter().any(|c| text.contains(*c)) {
            return None;
        }
        match m {
            2 if text.contains("中和節") => Some(1),
            3 if text.contains("上巳") => Some(3),
            9 if text.contains("重陽") || text.contains("九月九日") => Some(9),
            _ => None,
        }
    };
    let mut fest_stat = (0usize, 0usize, 0usize); // (条数, 合张表, 合推步)
    let mut fest_rows = String::new();
    for b in books {
        let recs = parse_csv(&read(&format!("data/calendar/本纪纪日/{b}_benji.csv")));
        for r in recs.iter().skip(1).filter(|r| r.len() >= 15) {
            let (Ok(y), Ok(m)) = (r[3].parse::<i32>(), r[4].parse::<u8>()) else { continue };
            if r[6] == "1" || (r[0] == "元" && y < 1280) {
                continue;
            }
            let text = r[14..].join("，");
            let Some(n) = fest_day(y, m, &text) else { continue };
            let Some(tab) = to_table(&r[0], y, standard_month_name(m), false) else { continue };
            let Some(g) = GAN_ZHI_60.iter().position(|x| *x == r[7]) else { continue };
            let hit = |ms: &[(i64, u8, bool)]| ms.iter().find(|e| e.1 == tab.1 && !e.2).is_some_and(|e| (g as i64 - (e.0 + 49)).rem_euclid(60) == n - 1);
            let Some(rm) = reck.get(&tab.0) else { continue };
            let (ht, hr) = (hit(&table(tab.0)), hit(rm));
            fest_stat.0 += 1;
            fest_stat.1 += ht as usize;
            fest_stat.2 += hr as usize;
            let short: String = text.chars().take(40).collect();
            if ht != hr || (!ht && !hr) {
                fest_rows += &format!("| {b} | {} | {} | {} | {n} | {} | {} | {} |\n", tab.0, tab.1, r[7], if hr { "合" } else { "否" }, if ht { "合" } else { "否" }, short.replace('|', "／"));
            }
            if let Some(is) = idx.get(&tab) {
                for &i in is {
                    let d = &mut diffs[i];
                    let sr = (g as i64 - (d.r + 49)).rem_euclid(60) == n - 1;
                    let st = (g as i64 - (d.t + 49)).rem_euclid(60) == n - 1;
                    if sr != st {
                        d.jiri.push((format!("{b}（节日{n}日）"), sr, short.clone()));
                    }
                }
            }
        }
    }

    // 已有考证与例外表
    for (f, li_col, y_col, m_col, l_col) in [
        ("tang_shuo_kaozheng", Some(0), 1, 2, Some(3)),
        ("tangsong_cunyi_kaozheng", Some(0), 1, 2, Some(3)),
        ("wudai_shuo_kaozheng", Some(0), 1, 2, Some(3)),
        ("song_shuo_kaozheng", Some(0), 1, 2, Some(3)),
        ("yuan_shuo_kaozheng", None, 0, 1, None),
        ("dingshuo_exceptions", Some(0), 1, 2, Some(3)),
        ("chutu_rili_shuo", None, 1, 2, Some(3)),
    ] {
        for r in parse_csv(&read(&format!("data/calendar/{f}.csv"))).iter().filter(|r| r.len() > m_col && !r[0].starts_with('#')) {
            let (Ok(y), Ok(m)) = (r[y_col].parse::<i32>(), r[m_col].parse::<u8>()) else { continue };
            let leap = l_col.is_some_and(|c| r[c] == "1");
            // 有历名列者须同历（分政权之修正「政权：历名」不算）
            let li_ok = |d: &Diff| match li_col {
                None => true,
                Some(c) => !r[c].contains('：') && (d.li.starts_with(r[c].as_str()) || ding_name(d.li).is_some_and(|n| n.starts_with(r[c].as_str()))),
            };
            for d in diffs.iter_mut().filter(|d| d.year == y && d.month == m && d.leap == leap && li_ok(d)) {
                if !d.refs.contains(&f.to_string()) {
                    d.refs.push(f.to_string());
                }
            }
        }
    }

    // 人工按语
    let mut notes: HashMap<(i32, String), (String, String)> = HashMap::new();
    if let Ok(t) = std::fs::read_to_string(root.join("data/calendar/dingshuo_zhangbiao_notes.csv")) {
        for r in parse_csv(&t).into_iter().filter(|r| r.len() >= 5 && !r[0].starts_with('#') && r[0] != "历") {
            notes.insert((r[1].parse().unwrap(), r[2].clone()), (r[3].clone(), r[4].clone()));
        }
    }

    // 判定
    let classify = |d: &Diff| -> (String, String) {
        let sr: Vec<_> = d.shuo.iter().filter(|s| !s.2 && s.1 == d.r).collect();
        let st: Vec<_> = d.shuo.iter().filter(|s| !s.2 && s.1 == d.t).collect();
        let jr = d.jiri.iter().filter(|j| j.1).count();
        let jt = d.jiri.iter().filter(|j| !j.1).count();
        // 进退关系
        let cause = match d.ding {
            Some((base, h, lim, jin, _)) => {
                let (ro, to) = (d.r - base, d.t - base);
                let lim_s = lim.map_or("术文不进朔".to_string(), |l| format!("限 {l:.2} 时"));
                match (ro, to) {
                    (0, 1) if h >= 18.0 && lim.is_none_or(|l| h < l) => format!("张表进、推步不进：定小余 {h:.2} 时过四分之三（18 时），未过本历{lim_s}——张表统一四分之三进朔"),
                    (0, 1) if h >= 18.0 => format!("张表进、推步不进：定小余 {h:.2} 时过本历{lim_s}，推步依排月细则或例外表不进"),
                    (0, 1) => format!("张表进、推步不进：定小余 {h:.2} 时不及四分之三，{lim_s}；张表另有所据"),
                    (1, 0) if h < 18.0 => format!("推步进、张表不进：定小余 {h:.2} 时过本历{lim_s}而不及四分之三——张表统一四分之三进朔"),
                    (1, 0) if jin => format!("推步进、张表不进：定小余 {h:.2} 时过本历{lim_s}与四分之三；张表另有所据（或有交不进）"),
                    (1, 0) => format!("推步进、张表不进：定小余 {h:.2} 时未过本历{lim_s}，推步依排月细则或例外表进"),
                    (-1, 0) | (0, -1) | (1, 2) | (2, 1) => format!("差在定朔所在日：定小余 {h:.2} 时（{lim_s}）"),
                    _ => format!("推步较定朔日 {ro:+}、张表 {to:+}（定小余 {h:.2} 时，{lim_s}）"),
                }
            }
            None => "（本历无定朔细目）".to_string(),
        };
        let verdict = if !sr.is_empty() && st.is_empty() {
            "本纪证推步（张表疑误）".to_string()
        } else if !st.is_empty() && sr.is_empty() {
            "本纪证张表（推步临限或不合）".to_string()
        } else if !sr.is_empty() && !st.is_empty() {
            "本纪两歧".to_string()
        } else if jr > 0 && jt == 0 {
            "纪日证推步（张表疑误）".to_string()
        } else if jt > 0 && jr == 0 {
            "纪日证张表（推步临限或不合）".to_string()
        } else if jr > 0 && jt > 0 {
            "纪日两歧".to_string()
        } else if cause.contains("张表统一四分之三进朔") {
            "无证；出于张表统一进朔，从术文".to_string()
        } else {
            "无证".to_string()
        };
        (cause, verdict)
    };

    // 三分类：①有当时文字证张表误（判定含「证推步」）；②有文字证推步误（含「证张表」）；③无据待核（无证、两歧）
    let fenlei = |v: &str| -> &'static str {
        if v.contains("证推步") {
            "①文字证张表误"
        } else if v.contains("证张表") {
            "②文字证推步误"
        } else {
            "③无据待核"
        }
    };

    // CSV
    let mut csv = String::from("历,年,月,张表月名,推步朔,张表朔,张表−推步,定朔所在日,定小余（时）,进朔限（时）,距限（时）,经朔（时，距定朔所在日夜半）,有交,进退关系,本纪明书朔,纪日证据,已有考证,判定,分类,按语\n");
    let fmt_shuo = |d: &Diff| -> String {
        d.shuo.iter().map(|s| format!("{} {}{}", s.0, gz(s.1), if s.2 { "（已考讹）" } else { "" })).collect::<Vec<_>>().join("；")
    };
    let fmt_jiri = |d: &Diff| -> String {
        d.jiri.iter().map(|j| format!("{}：{}合{}", j.0, j.2, if j.1 { "推步" } else { "张表" })).collect::<Vec<_>>().join("；")
    };
    let mut rows_out = Vec::new();
    for d in &diffs {
        let (cause, mut verdict) = classify(d);
        let lab = label(d.month, d.leap);
        let note = notes.get(&(d.year, lab.clone())).cloned();
        let mut an = String::new();
        if let Some((v, n)) = &note {
            if !v.is_empty() {
                verdict = v.clone();
            }
            an = n.clone();
        }
        let (dd, h, lim, dist, jing) = match d.ding {
            Some((base, h, lim, _, jing)) => (
                gz(base).to_string(),
                format!("{h:.2}"),
                lim.map_or("—".into(), |l| format!("{l:.2}")),
                lim.map_or("—".into(), |l| format!("{:+.2}", h - l)),
                if jing.is_nan() { "—".into() } else { format!("{jing:.2}") },
            ),
            None => ("—".into(), "—".into(), "—".into(), "—".into(), "—".into()),
        };
        let tl = d.t_label.map_or(String::new(), |(m, l)| label(m, l));
        csv += &format!(
            "{},{},{},{},{},{},{:+},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            d.li,
            d.year,
            lab,
            tl,
            gz(d.r),
            gz(d.t),
            d.t - d.r,
            dd,
            h,
            lim,
            dist,
            jing,
            d.youjiao.map_or("—", |b| if b { "有" } else { "" }),
            cause,
            fmt_shuo(d).replace(',', "，"),
            fmt_jiri(d).replace(',', "，"),
            d.refs.join(" "),
            verdict,
            fenlei(&verdict),
            an.replace(',', "，")
        );
        rows_out.push((d.clone(), cause, verdict, an, dd, h, lim, dist));
    }
    std::fs::write(root.join("data/calendar/dingshuo_zhangbiao_diffs.csv"), csv).unwrap();

    // 文档生成部分
    let mut md = String::new();
    md += "### 逐历汇总\n\n（脚本生成：`rust/core/examples/dingshuo_zhangbiao_diffs.rs`；逐月 CSV 见 `data/calendar/dingshuo_zhangbiao_diffs.csv`。）\n\n";
    md += "| 历 | 年段 | 月数 | 朔日不同之月 | 张表晚一日 | 张表早一日 | 闰位不同之岁 | 其中统一进朔所致 | ①文字证张表误 | ②文字证推步误 | ③中两歧 | ③无据待核（含两歧） |\n|---|---|---|---|---|---|---|---|---|---|---|---|\n";
    let mut total = [0usize; 9];
    let mut stat_sorted: Vec<_> = stat.iter().collect();
    stat_sorted.sort_by_key(|(_, s)| s.0);
    for (name, s) in stat_sorted {
        let rs: Vec<_> = rows_out.iter().filter(|x| x.0.li == *name).collect();
        if rs.is_empty() && s.4 == 0 {
            continue;
        }
        let late = rs.iter().filter(|x| x.0.t > x.0.r).count();
        let early = rs.len() - late;
        let uni = rs.iter().filter(|x| x.1.contains("张表统一四分之三进朔")).count();
        let pr = rs.iter().filter(|x| x.2.contains("证推步")).count();
        let pt = rs.iter().filter(|x| x.2.contains("证张表")).count();
        let bo = rs.iter().filter(|x| x.2.contains("两歧")).count();
        let no = rs.len() - pr - pt;
        for (i, v) in [rs.len(), late, early, s.4, uni, pr, pt, bo, no].iter().enumerate() {
            total[i] += v;
        }
        md += &format!("| {name} | {}—{} | {} | {} | {late} | {early} | {} | {uni} | {pr} | {pt} | {bo} | {no} |\n", s.0, s.1, s.2, rs.len(), s.4);
    }
    md += &format!(
        "| **合计** | | | **{}** | {} | {} | **{}** | {} | **{}** | **{}** | {} | {} |\n\n",
        total[0], total[1], total[2], total[3], total[4], total[5], total[6], total[7], total[8]
    );
    md += "分类：①有当时文字（本纪明书朔、纪日、写本、册文、题记等）证张表误、推步合；②有文字证推步误、张表合；③无据待核（无本纪纪日可判，或两歧）。三类之和即朔日不同之月。\n\n";
    md += "「其中统一进朔所致」：张表与推步之差恰可由张表「朔日小余在日法四分之三已上者，虚进一日」（李勇、张培瑜 1996 自述）与本历术文进朔限之不同解释者。\n\n";
    md += "### 有证之月\n\n| 历 | 年 | 月 | 推步 | 张表 | 定小余（时） | 限（时） | 距限 | 本纪明书朔 | 纪日 | 判定 | 分类 | 按语 |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|\n";
    let cell = |s: String| if s.is_empty() { "—".to_string() } else { s.replace('|', "／") };
    for x in rows_out.iter().filter(|x| !x.0.shuo.is_empty() || !x.0.jiri.is_empty() || !x.3.is_empty()) {
        let d = &x.0;
        md += &format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            d.li,
            d.year,
            label(d.month, d.leap),
            gz(d.r),
            gz(d.t),
            x.5,
            x.6,
            x.7,
            cell(fmt_shuo(d)),
            cell(fmt_jiri(d)),
            x.2,
            fenlei(&x.2),
            cell(x.3.clone())
        );
    }
    md += "\n### 逐月表（全部朔日不同之月）\n\n「定朔」为定朔所在日干支；「距限」= 定小余 − 进朔限（正为过限）；「有交」= 经朔入交入食限（大衍一系此等月定朔用二次差精法；— 为未复原交会）；「经朔」为经朔距定朔所在日夜半之时（负为前一日）。\n\n";
    md += "| 历 | 年 | 月 | 推步 | 张表 | 定朔 | 定小余 | 限 | 距限 | 有交 | 进退关系 | 证据 | 已有考证 | 判定 | 分类 |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n";
    for x in &rows_out {
        let d = &x.0;
        let lab = match d.t_label {
            Some((m, l)) => format!("{}（张表{}）", label(d.month, d.leap), label(m, l)),
            None => label(d.month, d.leap),
        };
        let ev = [fmt_shuo(d), d.jiri.iter().map(|j| format!("{}合{}", j.0, if j.1 { "推步" } else { "张表" })).collect::<Vec<_>>().join("；")]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("；");
        md += &format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            d.li,
            d.year,
            lab,
            gz(d.r),
            gz(d.t),
            x.4,
            x.5,
            x.6,
            x.7,
            d.youjiao.map_or("—", |b| if b { "有" } else { "" }),
            x.1.split('：').next().unwrap_or(""),
            cell(ev),
            cell(d.refs.join(" ")),
            x.2,
            fenlei(&x.2)
        );
    }
    md += &format!(
        "\n### 节日纪日逆推之朔\n\n本纪纪日中明书节名之条（中和节二月一日、上巳三月三日、重阳九月九日；罢、停、禁、贡、改者不算）凡 {} 条，\n逆推之朔合张表 {}、合推步 {}。两家不同或皆不合者：\n\n| 书 | 年 | 月 | 日干支 | 节日日序 | 推步 | 张表 | 原文 |\n|---|---|---|---|---|---|---|---|\n{}",
        fest_stat.0, fest_stat.1, fest_stat.2, fest_rows
    );
    md += "\n### 闰位不同之岁\n\n| 年 | 历 | 推步闰 | 张表闰 |\n|---|---|---|---|\n";
    for (y, n, a, b) in &leap_years {
        md += &format!("| {y} | {n} | {a} | {b} |\n");
    }
    let doc_path = root.join("doc/calendar/dingshuo-zhangbiao-diffs.md");
    let doc = std::fs::read_to_string(&doc_path).unwrap_or_else(|_| "<!-- 生成开始 -->\n<!-- 生成结束 -->\n".into());
    let (a, b) = (doc.find("<!-- 生成开始 -->").expect("生成开始"), doc.find("<!-- 生成结束 -->").expect("生成结束"));
    let new = format!("{}<!-- 生成开始 -->\n\n{}\n{}", &doc[..a], md, &doc[b..]);
    std::fs::write(doc_path, new).unwrap();
    println!("朔日不同之月 {}（①文字证张表误 {}、②文字证推步误 {}、③无据待核 {}），闰位不同之岁 {}", total[0], total[5], total[6], total[8], total[3]);
}
