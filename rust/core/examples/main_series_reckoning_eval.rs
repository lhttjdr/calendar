//! 主序列按术推步逐历检验：与张培瑜历表逐月比较（朔日、闰位），并以正史本纪明书朔日（`data/calendar/benji_shuo_check.csv`）
//! 逐条检验推步与张表；秦汉初（前 221—太初元年四月）张表以前的今人历表为李忠林复原历，本纪取《汉书》本纪书朔、晦
//! （`data/calendar/hanshu_qinhan_benji.csv`，逐条结果写 data/calendar/qinhan_benji_check.csv）。结果定第 4 层逐历缺省（`lifa::reckoning_preferred_by_evidence`，见 doc/calendar/rili-sources.md）。
//!
//! 输出 data/calendar/main_series_reckoning_eval.csv（逐历汇总）、data/calendar/main_series_reckoning_benji.csv（推步与张表不同、
//! 或本纪只合其一之条）、data/calendar/main_series_reckoning_diffs.csv（推步与张表互异之月逐月三分类）。
//!
//! 三分类（推步与张表互异之月）：①有硬据证张表误、推步合；②有硬据证推步误、张表合；③无据待核（无证、证据冲突）。
//! 证据等级：A 实物历日（`artifacts` 整年实物、`data/calendar/chutu_rili_shuo.csv` 出土具注历）；B 当时文字；C 当朝或近时所修
//! 史书明书朔（含明、清实录书朔，《汉书》秦汉初书朔晦）；D 当朝史书纪日与「閏月」；E 后人追修史书（《后汉书》《晋书》，《南史》《北史》
//! 《新五代史》前史无同条者）。D、E 须异源两条（同一部书——含同源诸书——同月诸事只算一源），一见、一源者归③。619—1911 年朔日之异
//! （定朔考证按朔日相近配月而张表月名相同者）只取 `data/calendar/dingshuo_zhangbiao_diffs.csv`（`examples/dingshuo_zhangbiao_diffs.rs`，
//! 含明书朔、实录、纪日、写本与人工按语）之分类（另加实物），故须先跑该程序；其余互异之月本程序按月名收本纪明书朔、纪日、「閏月」、实录书朔。
//! 复原性质材料（《通鉴目录》、今人诸表）不作证据。有 A 级者从 A；余者诸证一致即从之，相冲突则归③。
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example main_series_reckoning_eval
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::historical::ShuoriTable;
use lunar_core::calendar::lifa::{main_series_year_months, RiliSource, MainSeriesCalendarPeriods, MainSeriesPreference};
use lunar_core::calendar::month_labels::{standard_month_name, MonthLabel, MonthLabelReforms};
use lunar_core::calendar::artifacts::main_series_artifact_year;
use lunar_core::calendar::push_calendar;
use lunar_core::calendar::qinhan_restore;
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// 同一史源之书（旧、新唐书；旧、新五代史；北史与北朝四史；南史与南朝四史；晋书与三国志魏书）互见之条不算独立
fn lineage(book: &str) -> &str {
    match book.trim_end_matches("_benji") {
        "jiutangshu" | "xintangshu" => "唐",
        "jiuwudaishi" | "xinwudaishi" => "五代",
        "beishi" | "weishu" | "beiqishu" | "zhoushu" | "suishu" => "北朝",
        "nanshi" | "songshu" | "nanqishu" | "liangshu" | "chenshu" => "南朝",
        "jinshu" | "sanguozhi" => "魏晋",
        x => x,
    }
}

/// 检验开关（只供分解各项改动之效，缺省全开）：环境变量 `ABLATE` 列出要关掉的项，逗号分隔——`jia`（D、E 级判①②须异源两条）、
/// `e2`（E 级明书朔亦须两条，与分政权同）、`jiri`（定朔考证不及之互异月收本纪纪日、「閏月」与《汉书》秦汉初书朔晦纪日）、
/// `shilu`（明、清实录书朔，C 级；619 年后朔日之异由定朔考证收入）
fn on(f: &str) -> bool {
    !std::env::var("ABLATE").unwrap_or_default().split(',').any(|x| x.trim() == f)
}

/// 互异之月的一条证据
#[derive(Clone)]
struct ZEv {
    /// 支持推步（否则支持张表）
    sup_r: bool,
    lv: char,
    what: String,
    /// 书（史源按 [`lineage`] 归并；实物、考证结论为空）
    book: String,
    /// 一条即可判：A、B、C 级，或定朔考证已判之①②
    hard: bool,
}

/// 一月诸证之分类：有 A 级者只看 A 级；余者诸证须同向（两向为冲突）；一条即可判者（A、B、C、定朔考证已判）一条即判；
/// D、E 级须异源两条（`jia` 关时同源内取条数最多之书计条数）。→ (0 ①／1 ②／2 ③, 等级或注)
fn classify_z(ev: &[ZEv]) -> (usize, String) {
    let top: Vec<&ZEv> = if ev.iter().any(|e| e.lv == 'A') { ev.iter().filter(|e| e.lv == 'A').collect() } else { ev.iter().collect() };
    if top.is_empty() {
        return (2, "—".into());
    }
    let side_r = top.iter().all(|e| e.sup_r);
    if !side_r && !top.iter().all(|e| !e.sup_r) {
        return (2, "—（证据冲突）".into());
    }
    let k = if side_r { 0 } else { 1 };
    let best = top.iter().map(|e| e.lv).min().unwrap().to_string();
    if top.iter().any(|e| e.hard) {
        return (k, best);
    }
    let lins: BTreeSet<&str> = top.iter().map(|e| lineage(&e.book)).collect();
    let mut per: BTreeMap<(&str, &str), BTreeSet<&str>> = BTreeMap::new();
    for e in &top {
        per.entry((lineage(&e.book), e.book.as_str())).or_default().insert(e.what.as_str());
    }
    let mut by: BTreeMap<&str, usize> = BTreeMap::new();
    for ((l, _), v) in &per {
        let x = by.entry(l).or_default();
        *x = (*x).max(v.len());
    }
    let n_old: usize = by.values().sum();
    let n = if on("jia") { lins.len() } else { n_old };
    if n >= 2 {
        return (k, best);
    }
    let why = if n_old > 1 { format!("史书只一源 {n_old} 条") } else { "纪日一见".to_string() };
    (2, format!("—（{why}，倾向{}）", if side_r { "推步" } else { "张表" }))
}

/// 两家相同之月的证据
#[derive(Default)]
struct SameEv {
    /// 实物：(合两家, 说明)
    art: Vec<(bool, String)>,
    /// 明书朔：(合两家, 等级, 判史书误, 说明)
    shuo: Vec<(bool, char, bool, String)>,
    /// 纪日（只计能分辨之条：朔日、晦日，或书日序者）：(合两家, 书, 文首)
    jiri: Vec<(bool, String, String)>,
    /// 不合两家之纪日的独立错误：书 → 该书该月诸条（行号, 类：0 在两家月内或只在一家月内，1 两家月外，2 月内而日序、晦不合）
    jiri_rows: BTreeMap<String, Vec<(usize, u8)>>,
}

/// 不合两家之纪日的独立错误数与等级。同书同月诸条按原文行序排，两家月外之条连续成串者（中间无月内之条隔开）多出一处脱月名、
/// 月名错位或系月之误，一串只算一处；月内而日序、晦不合者各算一处。同源诸书（见 [`lineage`]）取处数最多之书，各史源相加
fn jiri_count_con(rows: &BTreeMap<String, Vec<(usize, u8)>>) -> (usize, char) {
    let mut by: BTreeMap<String, usize> = BTreeMap::new();
    let mut lv = 'E';
    for (b, v) in rows {
        let mut v = v.clone();
        v.sort();
        let mut n = 0;
        let mut prev_out = false;
        for &(_, k) in &v {
            match k {
                1 => {
                    if !prev_out {
                        n += 1;
                    }
                    prev_out = true;
                }
                2 => {
                    n += 1;
                    prev_out = false;
                }
                _ => prev_out = false,
            }
        }
        if n > 0 {
            let e = by.entry(lineage(b).to_string()).or_default();
            *e = (*e).max(n);
            lv = lv.min(book_level(b, false));
        }
    }
    (by.values().sum(), lv)
}

/// 主序列之条：北魏、东魏、西魏、北齐、北周与隋平陈（589）以前，元 1280 年以前（金重修大明历）不是主序列所行之历，
/// 其书（《北史》《隋书》等）之条不以中原诸历衡之（另由 regime_benji_eval 以分政权推步检验）；明 1644 年后、清 1645 年前同
fn main_series_regime(regime: &str, y: i32) -> bool {
    match regime {
        "西汉" | "东汉" | "魏" | "晋" | "宋" | "齐" | "梁" | "陈" | "唐" | "武周" | "后梁" | "后唐" | "后晋" | "后汉" | "后周" => true,
        "隋" => y >= 590,
        "元" => y >= 1280,
        "明" => y <= 1644,
        "清" => y >= 1645,
        _ => false,
    }
}

/// 纪日独立条数与等级：同源内取条数最多之书，等级取同源最高者
fn jiri_count(js: &[&(bool, String, String)]) -> (usize, char) {
    let mut seen = BTreeSet::new();
    let mut per: BTreeMap<(String, String), usize> = BTreeMap::new();
    for j in js {
        if seen.insert((j.1.clone(), j.2.clone())) {
            *per.entry((lineage(&j.1).to_string(), j.1.clone())).or_default() += 1;
        }
    }
    let mut by: BTreeMap<String, usize> = BTreeMap::new();
    let mut lv = 'E';
    for ((l, b), n) in per {
        let e = by.entry(l).or_default();
        *e = (*e).max(n);
        lv = lv.min(book_level(&b, false));
    }
    (by.values().sum(), lv)
}

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

#[derive(Default)]
struct Stat {
    years: (i32, i32),
    months: usize,
    same: usize,
    leap_diff: Vec<i32>,
    n: usize,
    hit_r: usize,
    hit_t: usize,
    only_r: usize,
    only_t: usize,
    /// 已考为讹误（data/calendar/benji_shuo_errata.csv）而不计的本纪条数
    errata: usize,
    /// 合推步之条中，靠史载例外表「本纪」类修正（只有本纪明书、推步临限）才合者
    benji_fix: usize,
    /// 推步与张表互异之月：①、②、③
    cls: [usize; 3],
    /// 两家相同之月：有证、同误、无证
    same_cls: [usize; 3],
}

/// 证据等级：《后汉书》《晋书》与《南史》《北史》《新五代史》（前史无同条者）为 E；余者（含《新唐书》）明书朔 C、纪日 D。
/// 同一史源互见之条（如旧、新唐书同书一朔）一致时只算一证，取最高一级。
fn book_level(book: &str, shuo: bool) -> char {
    if ["houhanshu", "jinshu", "nanshi", "beishi", "xinwudaishi"].iter().any(|b| book.starts_with(b)) {
        'E'
    } else if shuo {
        'C'
    } else {
        'D'
    }
}

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    let zhang = ShuoriTable::from_csv(&read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let periods = MainSeriesCalendarPeriods::embedded();
    let mut stats: BTreeMap<(i32, &'static str), Stat> = BTreeMap::new();
    let mut first_year: BTreeMap<&'static str, i32> = BTreeMap::new();
    // 逐年：推步与张表
    let mut reck: BTreeMap<i32, Vec<(i64, u8, bool)>> = BTreeMap::new();
    // 推步与张表互异之月：(首年, 历, 年, 推步月, 张表对应月, 证据 [(支持推步, 等级, 说明)])
    #[allow(clippy::type_complexity)]
    let mut sames: Vec<(i32, &'static str, i32, (i64, u8, bool))> = Vec::new();
    let mut same_ev: HashMap<(i32, u8, bool), SameEv> = HashMap::new();
    let mut diffs: Vec<(i32, &'static str, i32, (i64, u8, bool), Option<(i64, u8, bool)>, Vec<ZEv>)> = Vec::new();
    // 今人历表：张表；张表以前（秦汉初）为李忠林复原历
    let table = |y: i32| -> Vec<(i64, u8, bool)> {
        zhang.year_months(y).map(|t| t.to_vec()).or_else(|| qinhan_restore::year_months(y)).unwrap()
    };
    for y in -220..=1913 {
        // 直接按术推步（不经仲裁）：实物年也照推，推步与张表的比较不受实物接入影响
        let name: &'static str = periods.name_for_lunar_year(y);
        let r = match main_series_year_months(None, y, MainSeriesPreference::Reckoning) {
            Some((r, RiliSource::Reckoned(_))) => r,
            Some((_, RiliSource::Artifact(_))) => match push_calendar::reckon_lifa_year(name, y) {
                Some(r) => r,
                None => continue,
            },
            _ => continue,
        };
        let t = table(y);
        let fy = *first_year.entry(name).or_insert(y);
        let s = stats.entry((fy, name)).or_default();
        if s.months == 0 {
            s.years.0 = y;
        }
        s.years.1 = y;
        s.months += r.len();
        s.same += r.iter().filter(|m| t.contains(m)).count();
        let lr: Vec<_> = r.iter().filter(|m| m.2).map(|m| m.1).collect();
        let lt: Vec<_> = t.iter().filter(|m| m.2).map(|m| m.1).collect();
        if lr != lt {
            s.leap_diff.push(y);
        }
        for &m in r.iter().filter(|m| t.contains(m)) {
            sames.push((fy, name, y, m));
        }
        for &m in r.iter().filter(|m| !t.contains(m)) {
            // 张表对应之月：同月名者（朔日之异），否则同朔日者（月名、闰位之异）
            let tm = t.iter().find(|e| e.1 == m.1 && e.2 == m.2).or_else(|| t.iter().find(|e| e.0 == m.0)).copied();
            diffs.push((fy, name, y, m, tm, Vec::new()));
        }
        reck.insert(y, r);
    }
    // 本纪明书朔日
    let rows = parse_csv(&read("data/calendar/benji_shuo_check.csv"));
    let head = rows.iter().find(|r| r.first().is_some_and(|c| c == "书")).unwrap().clone();
    let col = |n: &str| head.iter().position(|h| h == n).unwrap();
    let (cb, cy, cm, cr, ct, ck, cx, cs) = (col("书"), col("年"), col("月"), col("本纪朔 JDN"), col("张表朔 JDN"), col("类"), col("原文"), col("政权"));
    // 史载例外表中「本纪」类所改之朔日
    let benji_fix_jdn: std::collections::BTreeSet<i64> =
        lunar_core::calendar::dingshuo::shizai_liwai().iter().filter(|r| r.4 == "本纪" && !r.3).map(|r| r.2).collect();
    // 已考讹误的本纪朔日：不作证据，另计
    let errata: std::collections::BTreeSet<(String, String, String)> = parse_csv(&read("data/calendar/benji_shuo_errata.csv"))
        .into_iter()
        .filter(|r| r.len() >= 3 && !r[0].starts_with('#') && r[0] != "书")
        .map(|r| (r[0].clone(), r[1].clone(), r[2].clone()))
        .collect();
    // 讹误表每条须在本纪检验总表中（防书名、年月写错）；落在取张表之段（如观象）者不进本表计分
    let all_keys: std::collections::BTreeSet<(String, String, String)> =
        rows.iter().filter(|r| r.len() == head.len()).map(|r| (r[cb].clone(), r[cy].clone(), r[cm].clone())).collect();
    let missing: Vec<_> = errata.iter().filter(|k| !all_keys.contains(*k)).collect();
    assert!(missing.is_empty(), "benji_shuo_errata.csv 有条目不在 benji_shuo_check.csv 中：{missing:?}");
    let reforms = MonthLabelReforms::from_csv(&read("data/calendar/month_label_reforms.csv")).unwrap();
    // 「其他」类判语：(书, 年, 月) → 判
    let qita: HashMap<(String, String, String), String> = parse_csv(&read("data/calendar/benji_qita_kaozheng_notes.csv"))
        .into_iter()
        .filter(|r| r.len() >= 5 && !r[0].starts_with('#') && r[0] != "书")
        .map(|r| ((r[0].clone(), r[1].clone(), r[2].clone()), r[4].clone()))
        .collect();
    let skip_books = ["liaoshi_benji", "jinshi_benji", "weishu_benji", "beiqishu_benji", "zhoushu_benji"];
    let mut detail = String::from("历,书,年,月,本纪朔,推步朔,张表朔,本纪合推步,本纪合张表,类,原文\n");
    for r in rows.iter().filter(|r| r.len() == head.len() && r[0] != "书") {
        if skip_books.contains(&r[cb].as_str()) || ["吴", "蜀"].contains(&r[cs].as_str()) {
            continue;
        }
        let (Ok(y), Ok(rec)) = (r[cy].parse::<i32>(), r[cr].parse::<i64>()) else { continue };
        // 只检主序列之条（北朝诸国、隋 590 年前、元 1280 年前等不检中原诸历，见 main_series_regime）；宋代史书「宋」与刘宋同名，
        // 960 年后的「宋」即赵宋，亦属主序列
        if !main_series_regime(&r[cs], y) {
            continue;
        }
        let (leap, m) = match r[cm].strip_prefix('闰').or_else(|| r[cm].strip_prefix('閏')) {
            Some(x) => (true, x),
            None => (false, r[cm].as_str()),
        };
        let Ok(m) = m.parse::<u8>() else { continue };
        // 本纪记当时月名、纪年年；经月序改制表换为建寅表月（如秦汉初十月岁首：太初元年十一月 = 夏正前一年十一月）
        let regime = match r[cs].as_str() {
            "西汉" | "东汉" => "汉",
            x => x,
        };
        let label = MonthLabel { year: y, name: standard_month_name(m).to_string(), is_leap: leap };
        let (y, m, leap) = reforms.table_of(regime, &label).map_or((y, m, leap), |t| (t.lunar_year, t.month, t.is_leap));
        // 太初改历以前的《汉书》本纪另由下面秦汉初一节逐条计（hanshu_qinhan_benji.csv），此处不重计
        if r[cb] == "hanshu_benji" && y <= -104 {
            continue;
        }
        let Some(months) = reck.get(&y) else { continue };
        let name = periods.name_for_lunar_year(y);
        let fy = *first_year.iter().find(|(n, _)| **n == name).map(|(_, f)| f).unwrap();
        let rj = months.iter().find(|e| e.1 == m && e.2 == leap).map(|e| e.0);
        let tj = zhang.year_months(y).and_then(|t| t.iter().find(|e| e.1 == m && e.2 == leap).map(|e| e.0));
        let (hr, ht) = (rj == Some(rec), tj == Some(rec));
        let is_errata = errata.contains(&(r[cb].clone(), r[cy].clone(), r[cm].clone()));
        // 两家相同之月：明书朔合两家为证，不合两家为同误（已考讹或「其他」类已判致误之由者注判史书误）
        if rj.is_some() && rj == tj {
            let judged = is_errata || (r[ck] == "其他" && qita.get(&(r[cb].clone(), r[cy].clone(), r[cm].clone())).is_some_and(|j| !j.starts_with("未明")));
            same_ev.entry((y, m, leap)).or_default().shuo.push((hr, book_level(&r[cb], true), judged, format!("{}「{}朔」", r[cb].replace("_benji", ""), gz(rec))));
        }
        if r[ck] == "其他" {
            continue;
        }
        if !is_errata {
            for d in diffs.iter_mut().filter(|d| d.2 == y) {
                let sup_r = d.3 .1 == m && d.3 .2 == leap && d.3 .0 == rec;
                let sup_t = d.4.is_some_and(|t| t.1 == m && t.2 == leap && t.0 == rec);
                if sup_r != sup_t {
                    let lv = book_level(&r[cb], true);
                    d.5.push(ZEv {
                        sup_r,
                        lv,
                        what: format!("{}「{}{}月{}朔」", r[cb].replace("_benji", ""), if leap { "闰" } else { "" }, m, gz(rec)),
                        book: r[cb].clone(),
                        hard: lv == 'C' || !on("e2"),
                    });
                }
            }
        }
        let s = stats.get_mut(&(fy, name)).unwrap();
        if is_errata {
            // 已考讹误：不计分，明细照列并注明
            s.errata += 1;
        } else {
            s.n += 1;
            s.hit_r += hr as usize;
            s.hit_t += ht as usize;
            s.only_r += (hr && !ht) as usize;
            s.only_t += (ht && !hr) as usize;
            // 合推步而该月朔是史载例外表「本纪」类所改（推步临限、只有本纪可从）：另计，「本纪合推步」减去即纯推步之合数
            s.benji_fix += (hr && benji_fix_jdn.contains(&rec)) as usize;
        }
        if hr != ht || rj != tj {
            let g = |j: Option<i64>| j.map_or("—", gz);
            let kind = if is_errata { "已考讹不计" } else { r[ck].as_str() };
            detail += &format!(
                "{},{},{},{},{},{},{},{},{},{},{}\n",
                name, r[cb], y, r[cm], gz(rec), g(rj), g(tj), hr as u8, ht as u8, kind, r[cx].replace([',', '\n'], "，")
            );
        }
    }
    // 秦汉初：《汉书》本纪纪日。书朔、晦者计分（晦为该月末日）；余者只判在月内否，逐条列出
    {
        let recs = parse_csv(&read("data/calendar/hanshu_qinhan_benji.csv"));
        let h = recs.iter().find(|r| r.first().is_some_and(|c| c == "卷")).unwrap().clone();
        let c = |n: &str| h.iter().position(|x| x == n).unwrap();
        let (cv, cn, cy, cm, cl, cg, cs, cx) = (c("卷"), c("纪年年"), c("夏正年"), c("月"), c("闰"), c("干支"), c("朔晦"), c("原文"));
        let mut per = String::from("卷,纪年年,夏正年,月,干支,朔晦,推步该月朔,推步合,复原该月朔,复原合,实物合,原文\n");
        // 月 (朔, 下月朔)
        let span = |ms: &dyn Fn(i32) -> Vec<(i64, u8, bool)>, y: i32, m: u8, leap: bool| -> Option<(i64, i64)> {
            let mut all: Vec<(i64, u8, bool)> = ms(y);
            all.extend(ms(y + 1));
            let i = all.iter().position(|e| e.1 == m && e.2 == leap)?;
            Some((all[i].0, all.get(i + 1)?.0))
        };
        // 推步（不经实物）；实物年另取出土历谱（artifacts），本纪与同月实物相违者不计入推步、复原之比较（实物 > 史书）
        let reck_y = |y: i32| push_calendar::reckon_lifa_year(periods.name_for_lunar_year(y), y).unwrap();
        let art_y = |y: i32| main_series_artifact_year(y).map_or_else(|| reck_y(y), |a| a.months.clone());
        for r in recs.iter().filter(|r| r.len() == h.len() && r[0] != "卷" && !r[0].starts_with('#')) {
            let (y, m, leap): (i32, u8, bool) = (r[cy].parse().unwrap(), r[cm].parse().unwrap(), r[cl] == "1");
            let gi = GAN_ZHI_60.iter().position(|g| *g == r[cg]).unwrap() as i64;
            let judge = |sp: Option<(i64, i64)>| -> (String, bool) {
                let Some((a, b)) = sp else { return ("—".into(), false) };
                let d = a + (gi - (a + 49)).rem_euclid(60);
                let ok = d < b && match r[cs].as_str() { "朔" => d == a, "晦" => d == b - 1, _ => true };
                (gz(a).to_string(), ok)
            };
            let (gr, okr) = judge(span(&reck_y, y, m, leap));
            let (gt, okt) = judge(span(&table, y, m, leap));
            let oka = main_series_artifact_year(y).is_some().then(|| judge(span(&art_y, y, m, leap)).1);
            let ka = oka.map_or("", |o| if o { "1" } else { "0" });
            per += &format!("{},{},{},{}{},{},{},{},{},{},{},{},{}\n", r[cv], r[cn], y, if leap { "闰" } else { "" }, m, r[cg], r[cs], gr, okr as u8, gt, okt as u8, ka, r[cx]);
            // 互异之月逐月分类：书朔、晦者 C（明书朔晦），余为纪日 D；与同月实物相违者不计（实物 > 史书）
            if on("jiri") && okr != okt && oka != Some(false) && gr != "—" {
                for d in diffs.iter_mut().filter(|d| d.2 == y && d.3 .1 == m && d.3 .2 == leap && gz(d.3 .0) == gr) {
                    let lv = if r[cs].is_empty() { 'D' } else { 'C' };
                    d.5.push(ZEv {
                        sup_r: okr,
                        lv,
                        what: format!("《汉书·{}》{}{}月{}{}", r[cv], if leap { "闰" } else { "" }, m, r[cg], r[cs]),
                        book: "hanshu".into(),
                        hard: lv == 'C',
                    });
                }
            }
            if r[cs].is_empty() {
                continue;
            }
            let name = periods.name_for_lunar_year(y);
            let fy = *first_year.iter().find(|(n, _)| **n == name).map(|(_, f)| f).unwrap();
            let s = stats.get_mut(&(fy, name)).unwrap();
            if oka == Some(false) {
                // 如高帝九年「六月乙未晦」：张家山《历谱》七月丁酉（胡家草场《岁纪》改朔），乙未为六月廿九——本纪从改朔前之原历
                s.errata += 1;
                detail += &format!("{},hanshu_benji,{},{}{},{}{},{},{},{},{},实物否定不计,{}\n", name, y, if leap { "闰" } else { "" }, m, r[cg], r[cs], gr, gt, okr as u8, okt as u8, r[cx]);
                continue;
            }
            s.n += 1;
            s.hit_r += okr as usize;
            s.hit_t += okt as usize;
            s.only_r += (okr && !okt) as usize;
            s.only_t += (okt && !okr) as usize;
            if okr != okt {
                detail += &format!("{},hanshu_benji,{},{}{},{}{},{},{},{},{},书{},{}\n", name, y, if leap { "闰" } else { "" }, m, r[cg], r[cs], gr, gt, okr as u8, okt as u8, r[cs], r[cx]);
            }
        }
        std::fs::write(root.join("data/calendar/qinhan_benji_check.csv"), per).unwrap();
    }
    // 互异之月另收（定朔考证已按月判者后面清去，只留其判）：本纪纪日（不书朔之条）、本纪「閏月」、明清实录书朔。纪日按月名与两家
    // 同名之月比——干支落在推步该月之内而不在张表同名之月之内，或反之；「閏月」（不书月数，接在所记某月之后）只用于两家闰位不同之岁的
    // 闰位、月名之异之月；实录书朔为 C 级（明实录校勘表已判讹或存疑之条照其结论不计）。
    let mut znotes: HashMap<(i32, i64, u8, bool), Vec<String>> = HashMap::new();
    let zy_books = [
        "hanshu", "houhanshu", "sanguozhi", "jinshu", "songshu", "nanqishu", "liangshu", "chenshu", "suishu", "beishi", "jiutangshu", "xintangshu",
        "jiuwudaishi", "xinwudaishi", "songshi", "yuanshi", "mingshi", "qingshigao",
    ];
    let tab_months = |y: i32| -> Vec<(i64, u8, bool)> { zhang.year_months(y).map(|t| t.to_vec()).or_else(|| qinhan_restore::year_months(y)).unwrap_or_default() };
    let next_after = |ms: &dyn Fn(i32) -> Vec<(i64, u8, bool)>, y: i32, a: i64| -> Option<i64> { ms(y).into_iter().chain(ms(y + 1)).map(|e| e.0).filter(|&j| j > a).min() };
    let reck_months = |y: i32| reck.get(&y).cloned().unwrap_or_default();
    let leap_type = |d: &(i32, &'static str, i32, (i64, u8, bool), Option<(i64, u8, bool)>, Vec<ZEv>)| {
        d.3 .2 || d.4.is_none_or(|t| t.1 != d.3 .1 || t.2 != d.3 .2 || (t.0 - d.3 .0).abs() >= 15)
    };
    if on("jiri") {
        for b in zy_books {
            let Ok(t) = std::fs::read_to_string(root.join(format!("data/calendar/本纪纪日/{b}_benji.csv"))) else { continue };
            for r in parse_csv(&t).iter().skip(1) {
                if r.len() < 15 || r[0].starts_with('#') || r[12] == "朔" {
                    continue;
                }
                let (Ok(y), Ok(m)) = (r[3].parse::<i32>(), r[4].parse::<u8>()) else { continue };
                if !main_series_regime(&r[0], y) || (b == "hanshu" && y <= -104) {
                    continue;
                }
                let regime = match r[0].as_str() {
                    "西汉" | "东汉" => "汉",
                    x => x,
                };
                let leap = r[6] == "1";
                let name = if r[5].is_empty() { standard_month_name(m).to_string() } else { r[5].replace('臘', "腊") };
                let Some(tb) = reforms
                    .table_of(regime, &MonthLabel { year: y, name, is_leap: leap })
                    .or_else(|| reforms.table_of(regime, &MonthLabel { year: y, name: standard_month_name(m).to_string(), is_leap: leap }))
                else {
                    continue;
                };
                let Some(g) = GAN_ZHI_60.iter().position(|x| *x == r[7]) else { continue };
                let pos = r[12].as_str();
                let nth = pos.strip_prefix('日').and_then(|n| n.parse::<i64>().ok());
                let judge = |a: i64, e: i64| {
                    let dd = a + (g as i64 - (a + 49)).rem_euclid(60);
                    dd < e && match (pos, nth) {
                        ("晦", _) => dd == e - 1,
                        (_, Some(n)) => dd == a + n - 1,
                        _ => true,
                    }
                };
                let text: String = r[14..].join("，").chars().take(20).collect();
                for d in diffs.iter_mut().filter(|d| d.2 == tb.lunar_year && d.3 .1 == tb.month && d.3 .2 == tb.is_leap) {
                    let a = d.3 .0;
                    let Some(br) = next_after(&reck_months, d.2, a) else { continue };
                    let zs = tab_months(d.2).into_iter().find(|e| e.1 == tb.month && e.2 == tb.is_leap);
                    let okr = judge(a, br);
                    let okt = zs.is_some_and(|z| next_after(&tab_months, d.2, z.0).is_some_and(|zb| judge(z.0, zb)));
                    if okr != okt {
                        d.5.push(ZEv { sup_r: okr, lv: book_level(b, false), what: format!("{b}：{}{text}", r[7]), book: b.to_string(), hard: false });
                    }
                }
            }
            // 本纪「閏月」：接在所记某月之后（月序经月序改制换为建寅表月）
            let Ok(t) = std::fs::read_to_string(root.join(format!("data/calendar/本纪纪日/{b}_benji_閏月.csv"))) else { continue };
            let rows = parse_csv(&t);
            let Some(h) = rows.first().cloned() else { continue };
            let c = |n: &str| h.iter().position(|x| x == n).unwrap();
            let (crg, cyr, caf, ctx) = (c("regime"), c("year"), c("leap_after"), c("text"));
            for r in rows.iter().skip(1).filter(|r| r.len() == h.len()) {
                let (Ok(y), Ok(after)) = (r[cyr].parse::<i32>(), r[caf].parse::<u8>()) else { continue };
                if !main_series_regime(&r[crg], y) || (b == "hanshu" && y <= -104) {
                    continue;
                }
                let regime = match r[crg].as_str() {
                    "西汉" | "东汉" => "汉",
                    x => x,
                };
                let lab = MonthLabel { year: y, name: standard_month_name(after).to_string(), is_leap: false };
                let (ty, ta) = reforms.table_of(regime, &lab).map_or((y, after), |t| (t.lunar_year, t.month));
                let rl: Vec<u8> = reck_months(ty).iter().filter(|e| e.2).map(|e| e.1).collect();
                let zl: Vec<u8> = tab_months(ty).iter().filter(|e| e.2).map(|e| e.1).collect();
                let (sr, st) = (rl.contains(&ta), zl.contains(&ta));
                if rl == zl || sr == st {
                    continue;
                }
                let text: String = r[ctx].chars().take(20).collect();
                for d in diffs.iter_mut().filter(|d| d.2 == ty) {
                    if leap_type(d) {
                        d.5.push(ZEv { sup_r: sr, lv: book_level(b, false), what: format!("{b}：閏月接{ta}月 {text}"), book: b.to_string(), hard: false });
                    }
                }
            }
        }
    }
    if on("shilu") {
        let jiaokan: HashMap<(i32, u8, bool), String> = parse_csv(&read("data/calendar/mingshilu_jiaokan.csv"))
            .into_iter()
            .filter(|r| r.len() > 10 && !r[0].starts_with('#') && r[0] != "年")
            .filter_map(|r| Some(((r[0].parse().ok()?, r[1].parse().ok()?, r[2] == "1"), r[10].clone())))
            .collect();
        let mut lu: Vec<(i32, u8, bool, String, String, &str)> = Vec::new();
        for r in parse_csv(&read("data/calendar/mingshilu_shuo.csv")).into_iter().filter(|r| r.len() > 5 && !r[0].starts_with('#') && r[0] != "年") {
            if let (Ok(y), Ok(m)) = (r[0].parse::<i32>(), r[1].parse::<u8>()) {
                lu.push((y, m, r[2] == "1", r[3].clone(), format!("{}「{}」", r[4], r[5].chars().take(20).collect::<String>()), "mingshilu"));
            }
        }
        for r in parse_csv(&read("data/calendar/qingshilu_shuo_xinfa.csv")).into_iter().filter(|r| r.len() > 8 && !r[0].starts_with('#') && r[0] != "年") {
            if let (Ok(y), Ok(m)) = (r[0].parse::<i32>(), r[2].parse::<u8>()) {
                lu.push((y, m, r[3] == "1", r[4].clone(), format!("《清实录》第 {} 册卷 {} 书页 {}「{}」", r[5], r[6], r[7], r[8].chars().take(20).collect::<String>()), "qingshilu"));
            }
        }
        // 校勘表判「张表误」而录文书朔表未收之月（1378 年八月，录文抽取漏）：以校勘表所录馆本朔补
        for r in parse_csv(&read("data/calendar/mingshilu_jiaokan.csv")).into_iter().filter(|r| r.len() > 10 && !r[0].starts_with('#') && r[0] != "年") {
            if let (Ok(y), Ok(m)) = (r[0].parse::<i32>(), r[1].parse::<u8>()) {
                if r[10].contains("张表误") && !lu.iter().any(|x| x.0 == y && x.1 == m && x.2 == (r[2] == "1") && x.5 == "mingshilu") {
                    lu.push((y, m, r[2] == "1", r[3].clone(), format!("{}（据明实录校勘表，校勘记册{}书页{}）", r[6], r[7], r[8]), "mingshilu"));
                }
            }
        }
        for (y, m, l, sh, what, book) in lu {
            for d in diffs.iter_mut().filter(|d| d.2 == y && d.3 .1 == m && d.3 .2 == l) {
                let zs = tab_months(y).into_iter().find(|e| e.1 == m && e.2 == l);
                let (sr, st) = (sh == gz(d.3 .0), zs.is_some_and(|z| gz(z.0) == sh));
                if sr == st {
                    continue;
                }
                if let Some(j) = jiaokan.get(&(y, m, l)).filter(|j| book == "mingshilu" && !j.contains("张表误")) {
                    znotes.entry((d.2, d.3 .0, d.3 .1, d.3 .2)).or_default().push(format!("{what}合{}，明实录校勘表：{j}——照校勘表不计", if sr { "推步" } else { "张表" }));
                    continue;
                }
                d.5.push(ZEv { sup_r: sr, lv: 'C', what: what.clone(), book: book.to_string(), hard: true });
            }
        }
    }
    // 实物：整年实物（artifacts）与出土具注历（chutu_rili_shuo.csv）
    let chutu: Vec<Vec<String>> = parse_csv(&read("data/calendar/chutu_rili_shuo.csv")).into_iter().filter(|r| r.len() > 4 && !r[0].starts_with('#') && r[0] != "件").collect();
    // 619—1911 定朔诸历朔日之异：取 dingshuo_zhangbiao_diffs.csv 之分类、等级
    let ding: Vec<Vec<String>> = parse_csv(&read("data/calendar/dingshuo_zhangbiao_diffs.csv")).into_iter().skip(1).filter(|r| r.len() > 20).collect();
    let lab = |m: (i64, u8, bool)| format!("{}{}", if m.2 { "闰" } else { "" }, m.1);
    let mut dcsv = String::from("历,年,月,推步朔,张表对应月,张表朔,异之类,分类,证据等级,证据\n");
    for d in diffs.iter_mut() {
        let (y, rm, tm) = (d.2, d.3, d.4);
        // 定朔考证已逐月判者：本纪明书朔已含其中（并与纪日合判），不再另计
        let ding_rows: Vec<&Vec<String>> = ding.iter().filter(|c| c[1] == y.to_string() && c[2] == lab(rm) && c[4] == gz(rm.0)).collect();
        let mut note = String::new();
        // 定朔考证按朔日相近配月：张表月名同者（「张表月名」空）其判即本月之判，本程序所收书朔、纪日、实录清去；张表月名不同者
        // （闰位、月名之异）考证不收纪日，本程序另按月名收之，考证所判①②亦并入
        if ding_rows.iter().any(|c| c[3].is_empty()) {
            d.5.clear();
        }
        for c in &ding_rows {
            if c[18].starts_with('①') || c[18].starts_with('②') {
                d.5.push(ZEv { sup_r: c[18].starts_with('①'), lv: c[19].chars().next().unwrap_or('E'), what: format!("定朔考证：{}", c[17]), book: String::new(), hard: true });
            } else if c[17] != "无证" && !c[17].starts_with("无证；") {
                note = format!("定朔考证：{}", c[17]);
            }
        }
        if let Some(a) = main_series_artifact_year(y) {
            let (hr, ht) = (a.months.contains(&rm), tm.is_some_and(|t| a.months.contains(&t)));
            if hr != ht {
                d.5.push(ZEv { sup_r: hr, lv: 'A', what: format!("实物历日（{y} 年）"), book: String::new(), hard: true });
            }
        }
        for c in chutu.iter().filter(|c| c[1] == y.to_string()) {
            let (Ok(m), l) = (c[2].parse::<u8>(), c[3] == "1") else { continue };
            let sup_r = rm.1 == m && rm.2 == l && gz(rm.0) == c[4];
            let sup_t = tm.is_some_and(|t| t.1 == m && t.2 == l && gz(t.0) == c[4]);
            if sup_r != sup_t {
                d.5.push(ZEv { sup_r, lv: 'A', what: c[0].split('（').next().unwrap_or(&c[0]).to_string(), book: String::new(), hard: true });
            }
        }
        let ev = &d.5;
        let (k, lv) = classify_z(ev);
        stats.get_mut(&(d.0, d.1)).unwrap().cls[k] += 1;
        // 两家同名之月而朔差约一月（如 719、852 年八月差 29 日）：推步该月即张表前后一月，是闰位之异
        let kind = match tm {
            Some(t) if t.1 == rm.1 && t.2 == rm.2 && (t.0 - rm.0).abs() < 15 => "朔日之异",
            Some(_) => "月名之异（闰位）",
            None => "张表无对应月",
        };
        let mut evs: Vec<String> = ev.iter().map(|e| format!("{}{}合{}", e.what, e.lv, if e.sup_r { "推步" } else { "张表" })).collect();
        if !note.is_empty() {
            evs.push(note.clone());
        }
        evs.extend(znotes.get(&(y, rm.0, rm.1, rm.2)).cloned().unwrap_or_default());
        dcsv += &format!(
            "{},{y},{},{},{},{},{kind},{},{lv},{}\n",
            d.1,
            lab(rm),
            gz(rm.0),
            tm.map_or("—".into(), lab),
            tm.map_or("—", |t| gz(t.0)),
            ["①硬据证张表误", "②硬据证推步误", "③无据待核"][k],
            evs.join("；").replace(',', "，")
        );
    }
    std::fs::write(root.join("data/calendar/main_series_reckoning_diffs.csv"), &dcsv).unwrap();

    // 两家相同之月：纪日。主序列诸史本纪纪日（不书朔之条），只取能分辨朔日之条——日在朔日或晦日（两家下月朔同者），或书日序；
    // 落在两家该月之外者为同误之证。他国、元 1280 年前、隋 590 年前、明 1644 年后、清 1645 年前之条不计。
    let zy_books = [
        "hanshu", "houhanshu", "sanguozhi", "jinshu", "songshu", "nanqishu", "liangshu", "chenshu", "suishu", "beishi", "jiutangshu", "xintangshu",
        "jiuwudaishi", "xinwudaishi", "songshi", "yuanshi", "mingshi", "qingshigao",
    ];
    let next_shuo = |ms: &dyn Fn(i32) -> Vec<(i64, u8, bool)>, y: i32, a: i64| -> Option<i64> {
        ms(y).into_iter().chain(ms(y + 1)).map(|e| e.0).filter(|&j| j > a).min()
    };
    let reck_f = |y: i32| reck.get(&y).cloned().unwrap_or_default();
    let tab_f = |y: i32| zhang.year_months(y).map(|t| t.to_vec()).or_else(|| qinhan_restore::year_months(y)).unwrap_or_default();
    let same_set: BTreeSet<(i32, u8, bool)> = sames.iter().map(|s| (s.2, s.3 .1, s.3 .2)).collect();
    for b in zy_books {
        let Ok(t) = std::fs::read_to_string(root.join(format!("data/calendar/本纪纪日/{b}_benji.csv"))) else { continue };
        for (ri, r) in parse_csv(&t).iter().skip(1).enumerate() {
            if r.len() < 15 || r[0].starts_with('#') {
                continue;
            }
            let (Ok(y), Ok(m)) = (r[3].parse::<i32>(), r[4].parse::<u8>()) else { continue };
            let pos = r[12].as_str();
            if pos == "朔" {
                continue;
            }
            if !main_series_regime(&r[0], y) {
                continue;
            }
            let regime = match r[0].as_str() {
                "西汉" | "东汉" => "汉",
                x => x,
            };
            let leap = r[6] == "1";
            let name = if r[5].is_empty() { standard_month_name(m).to_string() } else { r[5].replace('臘', "腊") };
            let label = MonthLabel { year: y, name, is_leap: leap };
            let Some(tb) = reforms.table_of(regime, &label).or_else(|| reforms.table_of(regime, &MonthLabel { year: y, name: standard_month_name(m).to_string(), is_leap: leap })) else { continue };
            let key = (tb.lunar_year, tb.month, tb.is_leap);
            if !same_set.contains(&key) {
                continue;
            }
            let Some(a) = reck_f(key.0).iter().find(|e| e.1 == key.1 && e.2 == key.2).map(|e| e.0) else { continue };
            let (Some(br), Some(bt)) = (next_shuo(&reck_f, key.0, a), next_shuo(&tab_f, key.0, a)) else { continue };
            let Some(g) = GAN_ZHI_60.iter().position(|x| *x == r[7]) else { continue };
            let d = a + (g as i64 - (a + 49)).rem_euclid(60);
            let (in_r, in_t) = (d < br, d < bt);
            let nth = pos.strip_prefix('日').and_then(|n| n.parse::<i64>().ok());
            let text: String = r[14..].join("，").chars().take(20).collect();
            let e = same_ev.entry(key).or_default();
            let mut kind = 0u8;
            if !in_r && !in_t {
                e.jiri.push((false, b.to_string(), text));
                kind = 1;
            } else if in_r && in_t {
                let exact = match (pos, nth) {
                    ("晦", _) => Some(br == bt && d == br - 1),
                    (_, Some(n)) => Some(d == a + n - 1),
                    _ => None,
                };
                match exact {
                    Some(true) => e.jiri.push((true, b.to_string(), text)),
                    Some(false) => {
                        e.jiri.push((false, b.to_string(), text));
                        kind = 2;
                    }
                    None if d == a || (br == bt && d == br - 1) => e.jiri.push((true, b.to_string(), text)),
                    None => {}
                }
            }
            e.jiri_rows.entry(b.to_string()).or_default().push((ri, kind));
        }
    }
    // 实物
    for &(_, _, y, m) in &sames {
        if let Some(a) = main_series_artifact_year(y) {
            if let Some(am) = a.months.iter().find(|e| e.1 == m.1 && e.2 == m.2) {
                same_ev.entry((y, m.1, m.2)).or_default().art.push((am.0 == m.0, format!("实物历日（{y} 年）")));
            }
        }
        for c in chutu.iter().filter(|c| c[1] == y.to_string() && c[2] == m.1.to_string() && (c[3] == "1") == m.2) {
            same_ev.entry((y, m.1, m.2)).or_default().art.push((c[4] == gz(m.0), c[0].split('（').next().unwrap_or(&c[0]).to_string()));
        }
    }
    // 分类：有证（实物、明书朔一条即可；纪日须两条以上独立之条）、同误（史载与两家皆不合；注判史书误与否）、无证
    let mut mcsv = String::from("历,年,月,推步朔,张表朔,异同,分类,证据等级,注\n");
    let mut same_lv: BTreeMap<(usize, char), usize> = BTreeMap::new();
    for &(fy, name, y, m) in &sames {
        let e = same_ev.get(&(y, m.1, m.2));
        let mut sup: Vec<(char, String)> = Vec::new();
        let mut con: Vec<(char, bool, String)> = Vec::new();
        let mut note: Vec<String> = Vec::new();
        if let Some(e) = e {
            for a in &e.art {
                if a.0 { sup.push(('A', a.1.clone())) } else { con.push(('A', false, a.1.clone())) }
            }
            for s in &e.shuo {
                if s.0 { sup.push((s.1, s.3.clone())) } else { con.push((s.1, s.2, s.3.clone())) }
            }
            let js: Vec<_> = e.jiri.iter().filter(|j| j.0).collect();
            let jc: Vec<_> = e.jiri.iter().filter(|j| !j.0).collect();
            let (ns, ls) = jiri_count(&js);
            // 不合两家者按独立错误计（同书同月连续落在月外之一串只算一处，见 jiri_count_con）
            let (nc, lc) = if jc.is_empty() { (0, 'E') } else { jiri_count_con(&e.jiri_rows) };
            if ns >= 2 {
                sup.push((ls, format!("纪日 {ns} 条")));
            } else if ns == 1 {
                note.push("纪日一见，合两家".into());
            }
            let nrow = jc.iter().map(|j| (&j.1, &j.2)).collect::<BTreeSet<_>>().len();
            if nc >= 2 {
                con.push((lc, false, format!("纪日 {nc} 处不合两家（{nrow} 条）")));
            } else if nc == 1 {
                note.push(if nrow > 1 { format!("纪日一处不合两家（同书同月连续 {nrow} 条）") } else { "纪日一见，不合两家".into() });
            }
        }
        let best = |v: &[char]| v.iter().min().copied().map_or("—".to_string(), |c| c.to_string());
        let sup_a = sup.iter().any(|s| s.0 == 'A');
        let con_a = con.iter().any(|c| c.0 == 'A');
        let unjudged: Vec<_> = con.iter().filter(|c| !c.1).collect();
        let (k, lv) = if (sup_a && !con_a) || (!sup.is_empty() && unjudged.is_empty()) {
            if !con.is_empty() {
                note.push(format!("另有史载不合两家（{}，已判史书误）", con.iter().map(|c| c.2.as_str()).collect::<Vec<_>>().join("、")));
            }
            (0, best(&sup.iter().map(|s| s.0).collect::<Vec<_>>()))
        } else if !con.is_empty() {
            let j = if unjudged.is_empty() { "判史书误" } else if unjudged.len() == con.len() { "未判" } else { "部分判史书误" };
            note.push(format!("{}：{}", j, con.iter().map(|c| c.2.as_str()).collect::<Vec<_>>().join("、")));
            (1, best(&con.iter().map(|c| c.0).collect::<Vec<_>>()))
        } else {
            (2, "—".to_string())
        };
        if k < 2 {
            *same_lv.entry((k, lv.chars().next().unwrap())).or_default() += 1;
        }
        stats.get_mut(&(fy, name)).unwrap().same_cls[k] += 1;
        let ev_txt: Vec<String> = sup.iter().map(|s| format!("{}{}", s.1, s.0)).collect();
        mcsv += &format!(
            "{name},{y},{},{},{},相同,{},{lv},{}\n",
            lab(m),
            gz(m.0),
            gz(m.0),
            ["有证", "同误", "无证"][k],
            ev_txt.into_iter().chain(note).collect::<Vec<_>>().join("；").replace(',', "，")
        );
    }
    // 互异之月并入同一逐月表
    for line in dcsv.lines().skip(1) {
        let c: Vec<&str> = line.splitn(10, ',').collect();
        mcsv += &format!("{},{},{},{},{},互异（{}）,{},{},{}\n", c[0], c[1], c[2], c[3], c[5], c[6], c[7], c[8], c.get(9).unwrap_or(&""));
    }
    std::fs::write(root.join("data/calendar/main_series_reckoning_months.csv"), mcsv).unwrap();
    println!("相同之月 {}：有证 {:?}", sames.len(), same_lv);
    let mut out = String::from("历,起,止,月数,与张表同,闰位不同之岁,本纪条数,本纪合推步,本纪合张表,只合推步,只合张表,已考讹不计,从本纪修正,互异之月,①硬据证张表误,②硬据证推步误,③无据待核,相同有证,相同同误,相同无证\n");
    for ((_, name), s) in &stats {
        let ld: Vec<String> = s.leap_diff.iter().map(|y| y.to_string()).collect();
        out += &format!("{name},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n", s.years.0, s.years.1, s.months, s.same, ld.join(" "), s.n, s.hit_r, s.hit_t, s.only_r, s.only_t, s.errata, s.benji_fix, s.months - s.same, s.cls[0], s.cls[1], s.cls[2], s.same_cls[0], s.same_cls[1], s.same_cls[2]);
        println!("{name:<20} {}—{} 月 {} 同 {} 闰异 {:?} | 本纪 {} 推步 {} 张表 {} （只推步 {} 只张表 {}）", s.years.0, s.years.1, s.months, s.same, s.leap_diff, s.n, s.hit_r, s.hit_t, s.only_r, s.only_t);
    }
    std::fs::write(root.join("data/calendar/main_series_reckoning_eval.csv"), out).unwrap();
    std::fs::write(root.join("data/calendar/main_series_reckoning_benji.csv"), detail).unwrap();
}
