//! 主序列定朔诸历（619—1911）按术推步与张培瑜历表朔日不同之月逐月考证。
//!
//! 对推步与张表朔日相差一日之月，逐月列：定朔所在日与定小余、本历进朔限、距限、经朔，张表与推步的进退关系，以及证据——
//! 本纪明书朔日（`data/calendar/benji_shuo_check.csv`，已考讹误者另注）、本纪纪日（`data/calendar/本纪纪日/*.csv`：某日干支只在
//! 一家的该月或前月之内者）、已有考证与史载例外表；再按规则给出判定，人工按语取 `data/calendar/dingshuo_zhangbiao_notes.csv`。
//! 闰位不同之岁另表。推步取法同 `examples/main_series_reckoning_eval.rs`（直接按术推步，不经仲裁；实物年也照推）。
//!
//! 输出 `data/calendar/dingshuo_zhangbiao_diffs.csv`，并改写 `doc/calendar/dingshuo-zhangbiao-diffs.md` 中
//! `<!-- 生成开始 -->`、`<!-- 生成结束 -->` 之间的部分（其余为手写）。
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example dingshuo_zhangbiao_diffs
use lunar_core::calendar::dingshuo::{dingshuo_months, Biaoduan, DingshuoMonth};
use lunar_core::calendar::song_early;
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::historical::ShuoriTable;
use lunar_core::calendar::lifa::{main_series_year_months, RiliSource, MainSeriesCalendarPeriods, MainSeriesPreference};
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

/// 主序列历名 → 定朔细目所用之历名（[dingshuo_months]）；None 为不在 dingshuo 模块之历
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
    /// 经朔所入表段（[Biaoduan]）
    bd: Option<Biaoduan>,
    /// 本纪明书朔日：(书, 本纪朔 JDN, 已考讹误)
    shuo: Vec<(String, i64, bool)>,
    /// 纪日：(书, 支持推步, 原文)
    jiri: Vec<(String, bool, String)>,
    /// 已有考证、例外表
    refs: Vec<String>,
    /// 出土历日（`data/calendar/chutu_rili_shuo.csv`）：(件, 支持推步)
    art: Vec<(String, bool)>,
}

/// 证据等级（高到低）：A 实物历日；B 当时文字（碑志、题记、册文、诏令原文等）；C 当朝或近时所修史书明书朔；
/// D 当朝史书纪日干支；E 后人追修史书（多转录前史，非独立）。复原性质材料（《通鉴目录》、今人诸表）不作证据。
/// `book` 为本纪纪日表名（去 `_benji`），`shuo` 为明书朔（否则纪日）。
/// 《新唐书》本纪多据实录，与《旧唐书》同级（两书同条互见只算一证，见 [`lineage`]）；《南史》《北史》《新五代史》节录前史，
/// 与前史同条者按前史计、不重计（同一史源取最高一级），前史无者才为 E；《后汉书》《晋书》为 E。
fn book_level(book: &str, shuo: bool) -> char {
    if ["houhanshu", "jinshu", "nanshi", "beishi", "xinwudaishi"].iter().any(|b| book.starts_with(b)) {
        'E'
    } else if shuo {
        'C'
    } else {
        'D'
    }
}

/// 史源：同一史源之书（旧、新唐书；旧、新五代史；北史与魏、北齐、周、隋书；南史与宋、南齐、梁、陈书）互见之条不算独立
fn lineage(book: &str) -> &str {
    let b = book.split('（').next().unwrap_or(book);
    match b.trim_end_matches("_benji") {
        "jiutangshu" | "xintangshu" => "唐",
        "jiuwudaishi" | "xinwudaishi" => "五代",
        "beishi" | "weishu" | "beiqishu" | "zhoushu" | "suishu" => "北朝",
        "nanshi" | "songshu" | "nanqishu" | "liangshu" | "chenshu" => "南朝",
        x => x,
    }
}

/// 检验开关（只供分解各项改动之效，缺省全开）：环境变量 `ABLATE` 列出要关掉的项——`jia`（纪日判①②须异源两条）、
/// `shilu`（明、清实录书朔，C 级）、`e2`（E 级明书朔只一见不判）
fn on(f: &str) -> bool {
    !std::env::var("ABLATE").unwrap_or_default().split(',').any(|x| x.trim() == f)
}

/// 支持某一家之纪日的独立条数：同一部书（含同源诸书，见 [`lineage`]）同月诸事只算一个史源，各史源相加——同书系月之误会连带
/// 同月诸条，不能以同书不同日之事凑足两条。开关 `jia` 关时为旧规则：同一史源内各书取条数最多者，各史源相加；同书同文（前 20 字）不重计
fn jiri_events(d: &Diff, sup_r: bool) -> usize {
    if on("jia") {
        return d.jiri.iter().filter(|j| j.1 == sup_r).map(|j| lineage(j.0.split('（').next().unwrap_or(&j.0)).to_string()).collect::<BTreeSet<_>>().len();
    }
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut per: BTreeMap<(String, String), usize> = BTreeMap::new();
    for j in d.jiri.iter().filter(|j| j.1 == sup_r) {
        let b = j.0.split('（').next().unwrap_or(&j.0).to_string();
        if seen.insert((b.clone(), j.2.chars().take(20).collect())) {
            *per.entry((lineage(&b).to_string(), b)).or_default() += 1;
        }
    }
    let mut by_lin: BTreeMap<String, usize> = BTreeMap::new();
    for ((l, _), n) in per {
        let e = by_lin.entry(l).or_default();
        *e = (*e).max(n);
    }
    by_lin.values().sum()
}

/// 气名（冬至起）；麟德、戊寅气序启蛰在雨水前
fn qi_name(li: &str, k: usize) -> &'static str {
    const A: [&str; 24] =
        ["冬至", "小寒", "大寒", "立春", "雨水", "惊蛰", "春分", "清明", "谷雨", "立夏", "小满", "芒种", "夏至", "小暑", "大暑", "立秋", "处暑", "白露", "秋分", "寒露", "霜降", "立冬", "小雪", "大雪"];
    if (li.starts_with("麟德") || li.starts_with("戊寅")) && (k == 4 || k == 5) {
        return if k == 4 { "启蛰" } else { "雨水" };
    }
    A[k % 24]
}

/// 表段三栏：(月离所入, 日躔所入, 距月离段界（时，入转之时）)
fn fmt_bd(li: &str, bd: &Option<Biaoduan>) -> (String, String, Option<f64>) {
    let Some(b) = bd else { return ("—".into(), "—".into(), None) };
    let (yue, jie) = match b.yue {
        Some((n, yu, chu)) => {
            let mut bounds = vec![yu, 1.0 - yu];
            let seg = chu.map_or(String::new(), |c| {
                bounds.push((yu - c).abs());
                format!("{}段，初数 {c:.3}", if yu <= c { "初" } else { "末" })
            });
            let j = bounds.iter().cloned().fold(f64::INFINITY, f64::min).max(0.0) * 24.0;
            (if seg.is_empty() { format!("第{n}日 余{yu:.3}") } else { format!("第{n}日 余{yu:.3}（{seg}）") }, Some(j))
        }
        None => ("—".into(), None),
    };
    let (k, d, len) = b.ri;
    (yue, format!("{} {d:.2}/{len:.2}日", qi_name(li, k)), jie)
}

/// 疑格表（`data/calendar/dingshuo_biao_yige.csv`）一行
struct Yige {
    li: String,
    biao: String,
    ge: String,
    yiju: String,
    /// 满一日（月离）或满一气（日躔、进朔限）之影响上限（时）
    shang: Option<f64>,
    shuo: String,
}

fn label(m: u8, leap: bool) -> String {
    format!("{}{}", if leap { "闰" } else { "" }, m)
}

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    let zhang = ShuoriTable::from_csv(&read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let periods = MainSeriesCalendarPeriods::embedded();
    let reforms = MonthLabelReforms::from_csv(&read("data/calendar/month_label_reforms.csv")).unwrap();
    let (y_from, y_to) = (619, 1911);

    // 逐年推步与张表
    let mut reck: BTreeMap<i32, Vec<(i64, u8, bool)>> = BTreeMap::new();
    let mut names: BTreeMap<i32, &'static str> = BTreeMap::new();
    for y in y_from..=y_to {
        let name: &'static str = periods.name_for_lunar_year(y);
        let r = match main_series_year_months(None, y, MainSeriesPreference::Reckoning) {
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
            let bd = dn
                .and_then(|d| {
                    let v = &details[d];
                    v.iter().find(|x| x.day == rj).or_else(|| v.iter().find(|x| (x.ding_jdn - rj.min(tj)).abs() <= 1)).and_then(|x| x.biaoduan.clone())
                })
                .or_else(|| {
                    let n = if name.starts_with("钦天历") { "应天历（代推钦天）" } else { name };
                    song_early::biaoduan(n, rj).or_else(|| song_early::biaoduan(n, rj - 1)).or_else(|| song_early::biaoduan(n, rj + 1))
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
                bd,
                shuo: vec![],
                jiri: vec![],
                refs: vec![],
                art: vec![],
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

    // 明、清实录月首书朔（C 级，当朝实录）：明实录（data/calendar/mingshilu_shuo.csv）、清实录（qingshilu_shuo_xinfa.csv）；
    // 明实录校勘表（mingshilu_jiaokan.csv）已判馆本讹、诸本同讹、录文抽取误或存疑者照其结论不计（列入书朔栏并注「校勘表」）
    if on("shilu") {
        let jiaokan: HashMap<(i32, u8, bool), String> = parse_csv(&read("data/calendar/mingshilu_jiaokan.csv"))
            .into_iter()
            .filter(|r| r.len() > 10 && !r[0].starts_with('#') && r[0] != "年")
            .filter_map(|r| Some(((r[0].parse().ok()?, r[1].parse().ok()?, r[2] == "1"), r[10].clone())))
            .collect();
        let mut lu: Vec<(i32, u8, bool, String, &str)> = Vec::new();
        for r in parse_csv(&read("data/calendar/mingshilu_shuo.csv")).into_iter().filter(|r| r.len() > 5 && !r[0].starts_with('#') && r[0] != "年") {
            if let (Ok(y), Ok(m)) = (r[0].parse::<i32>(), r[1].parse::<u8>()) {
                lu.push((y, m, r[2] == "1", r[3].clone(), "明实录"));
            }
        }
        for r in parse_csv(&read("data/calendar/qingshilu_shuo_xinfa.csv")).into_iter().filter(|r| r.len() > 8 && !r[0].starts_with('#') && r[0] != "年") {
            if let (Ok(y), Ok(m)) = (r[0].parse::<i32>(), r[2].parse::<u8>()) {
                lu.push((y, m, r[3] == "1", r[4].clone(), "清实录"));
            }
        }
        // 校勘表判「张表误」而录文书朔表未收之月（1378 年八月「八月庚子朔」，太祖实录卷一百十九，录文抽取漏）：以校勘表所录馆本朔补
        for r in parse_csv(&read("data/calendar/mingshilu_jiaokan.csv")).into_iter().filter(|r| r.len() > 10 && !r[0].starts_with('#') && r[0] != "年") {
            if let (Ok(y), Ok(m)) = (r[0].parse::<i32>(), r[1].parse::<u8>()) {
                if r[10].contains("张表误") && !lu.iter().any(|x| x.0 == y && x.1 == m && x.2 == (r[2] == "1") && x.4 == "明实录") {
                    lu.push((y, m, r[2] == "1", r[3].clone(), "明实录"));
                }
            }
        }
        for (y, m, l, sh, book) in lu {
            let Some(gi) = GAN_ZHI_60.iter().position(|g| *g == sh) else { continue };
            for d in diffs.iter_mut().filter(|d| d.year == y && d.month == m && d.leap == l) {
                // 所书干支取最近推步朔之日（两家只差一二日）
                let rec = d.r + ((gi as i64 - (d.r + 49)).rem_euclid(60) + 30) % 60 - 30;
                let jk = jiaokan.get(&(y, m, l)).filter(|j| book == "明实录" && !j.contains("张表误") && (rec == d.r || rec == d.t));
                d.shuo.push((format!("{book}{}", jk.map_or(String::new(), |j| format!("（校勘表：{}）", j.chars().take(24).collect::<String>()))), rec, jk.is_some()));
            }
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
                // 出土历日：朔干支只合一家者为 A 级证据
                if f == "chutu_rili_shuo" {
                    let (hr, ht) = (r[4] == gz(d.r), r[4] == gz(d.t));
                    if hr != ht {
                        d.art.push((r[0].split('（').next().unwrap_or(&r[0]).to_string(), hr));
                    }
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

    // 疑格：定朔所用之格为校改格、版本异文或疑讹之格者
    let mut yige: Vec<Yige> = Vec::new();
    if let Ok(t) = std::fs::read_to_string(root.join("data/calendar/dingshuo_biao_yige.csv")) {
        for r in parse_csv(&t).into_iter().filter(|r| r.len() >= 8 && !r[0].starts_with('#') && r[0] != "历") {
            yige.push(Yige { li: r[0].clone(), biao: r[1].clone(), ge: r[2].clone(), yiju: r[5].clone(), shang: r[6].parse().ok(), shuo: r[7].clone() });
        }
    }
    // 一月所用之疑格：(说明, 影响上限（时）)
    let hits = |d: &Diff| -> Vec<(String, Option<f64>)> {
        let Some(b) = &d.bd else { return vec![] };
        let dn = ding_name(d.li).unwrap_or(d.li);
        let mut out = Vec::new();
        for y in yige.iter().filter(|y| d.li.starts_with(y.li.as_str()) || dn.starts_with(y.li.as_str())) {
            let hit = match y.biao.as_str() {
                "月离" => b.yue.and_then(|(n, yu, chu)| {
                    // 格：「7」整日，「7初」「7末」分段
                    let (num, part) = y.ge.split_at(y.ge.find(|c: char| !c.is_ascii_digit()).unwrap_or(y.ge.len()));
                    let num: u32 = num.parse().ok()?;
                    if num != n {
                        return None;
                    }
                    let frac = match (part, chu) {
                        ("初", Some(c)) if yu <= c => yu / c,
                        ("末", Some(c)) if yu > c => (yu - c) / (1.0 - c),
                        ("", _) => yu,
                        _ => return None,
                    };
                    Some(frac)
                }),
                "日躔" => (qi_name(d.li, b.ri.0) == y.ge).then(|| b.ri.1 / b.ri.2),
                // 进朔限：疑格在气初（节点），气内线性内插，前后两气皆用之，取满值为上限
                // 格作「某气初」者节点在本气之初（权 1 − 入气比），「某气末」者在本气之末（权 = 入气比）
                "进朔限" => {
                    let x = b.ri.1 / b.ri.2;
                    let q = qi_name(d.li, b.ri.0);
                    if y.ge == format!("{q}初") {
                        Some(1.0 - x)
                    } else if y.ge == format!("{q}末") {
                        Some(x)
                    } else {
                        (y.ge == q).then_some(1.0)
                    }
                }
                _ => None,
            };
            if let Some(fr) = hit {
                let shuo = if y.shuo.is_empty() { String::new() } else { format!("：{}", y.shuo) };
                out.push((format!("{}{}「{}」（{}）{shuo}", y.biao, if y.biao == "月离" { "第" } else { "" }, y.ge, y.yiju), y.shang.map(|s| s * fr)));
            }
        }
        out
    };

    // 判定
    let classify = |d: &Diff| -> (String, String) {
        let sr: Vec<_> = d.shuo.iter().filter(|s| !s.2 && s.1 == d.r).collect();
        let st: Vec<_> = d.shuo.iter().filter(|s| !s.2 && s.1 == d.t).collect();
        let jr = d.jiri.iter().filter(|j| j.1).count();
        let jt = d.jiri.iter().filter(|j| !j.1).count();
        // 明书朔之所出：全为明清实录者称「实录」，否则「本纪」（「本纪、实录」并出者亦以「本纪」起，等级照书）
        let src = |v: &[&(String, i64, bool)]| -> &'static str {
            if v.iter().all(|s| s.0.starts_with("明实录") || s.0.starts_with("清实录")) {
                "实录"
            } else if v.iter().any(|s| s.0.starts_with("明实录") || s.0.starts_with("清实录")) {
                "本纪、实录"
            } else {
                "本纪"
            }
        };
        // E 级（《新五代史》前史无同条者）明书朔只一见：与分政权、主序列逐月分类同，须两条（开关 e2）
        let e_only = |v: &[&(String, i64, bool)]| -> &'static str {
            if on("e2") && v.iter().all(|s| book_level(&s.0, true) == 'E') && v.iter().map(|s| lineage(&s.0)).collect::<BTreeSet<_>>().len() < 2 {
                "；E 级明书朔一见"
            } else {
                ""
            }
        };
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
        let ar = d.art.iter().filter(|a| a.1).count();
        let at = d.art.len() - ar;
        let verdict = if ar > 0 && at == 0 {
            "实物证推步（张表疑误）".to_string()
        } else if at > 0 && ar == 0 {
            "实物证张表（推步不合）".to_string()
        } else if ar > 0 && at > 0 {
            "实物两歧".to_string()
        } else if !sr.is_empty() && st.is_empty() && jt > 0 {
            // 单条明书朔与纪日冲突：不判（归③）
            format!("{}证推步而纪日证张表，冲突不判", src(&sr))
        } else if !st.is_empty() && sr.is_empty() && jr > 0 {
            format!("{}证张表而纪日证推步，冲突不判", src(&st))
        } else if !sr.is_empty() && st.is_empty() {
            format!("{}证推步（张表疑误）{}", src(&sr), e_only(&sr))
        } else if !st.is_empty() && sr.is_empty() {
            format!("{}证张表（推步临限或不合）{}", src(&st), e_only(&st))
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

    // 三分类：①有当时文字证张表误（判定含「证推步」）；②有文字证推步误（含「证张表」）；③无据待核（无证、两歧、冲突）
    let fenlei = |v: &str| -> &'static str {
        if v.contains("冲突") || v.contains("一见") {
            "③无据待核"
        } else if v.contains("证推步") {
            "①文字证张表误"
        } else if v.contains("证张表") {
            "②文字证推步误"
        } else {
            "③无据待核"
        }
    };

    // CSV
    // 证据等级（只对①②）：判定所依之证中最高一级。判定以「实物」起为 A，「写本、题记、碑、墓志、册、诏、祭文」起为 B，
    // 「本纪证」为明书朔（C，追修史书 E），「纪日证」为纪日（D，追修史书 E）。
    let dengji = |d: &Diff, v: &str| -> String {
        let f = fenlei(v);
        if f.starts_with('③') {
            return "—".into();
        }
        let sup_r = f.starts_with('①');
        if v.starts_with("实物") {
            return "A".into();
        }
        if ["写本", "题记", "碑", "墓志", "册", "诏", "祭文"].iter().any(|k| v.starts_with(k)) {
            return "B".into();
        }
        // 本纪以外之当朝或近时史书（《长编》等）明书朔
        if v.starts_with("他书明书朔") {
            return "C".into();
        }
        let lv = if v.starts_with("本纪") || v.starts_with("实录") {
            let want = if sup_r { d.r } else { d.t };
            d.shuo.iter().filter(|s| !s.2 && s.1 == want).map(|s| book_level(&s.0, true)).min()
        } else {
            d.jiri.iter().filter(|j| j.1 == sup_r).map(|j| book_level(&j.0, false)).min()
        };
        lv.map_or("—".into(), |c| c.to_string())
    };
    // 推步须移多少时（定朔）方与张表同：张表晚一日者须过进朔限（无限者过子夜），张表早一日者须退过限或子夜；
    // 推步依排月细则或例外表而与定小余所示相反者为 None（不是表格之事）
    let need = |d: &Diff| -> Option<f64> {
        let (base, h, lim, _, _) = d.ding?;
        if h.is_nan() {
            return None;
        }
        let (ro, to) = (d.r - base, d.t - base);
        match (ro, to) {
            (0, 1) => {
                let thr = lim.unwrap_or(24.0);
                (h < thr).then(|| thr - h)
            }
            // 麟德 684—708 年正月过戌初（19 时）进（排月细则，限不在术文）
            (1, 0) => lim.or((d.li.starts_with("麟德") && d.month == 1 && !d.leap).then_some(19.0)).filter(|l| h >= *l).map(|l| h - l),
            (0, -1) => Some(h),
            (-1, 0) | (1, 2) => Some(24.0 - h),
            (2, 1) => Some(h),
            _ => None,
        }
    };
    let fuhe = |d: &Diff| -> String {
        let nd = need(d);
        let nd_s = match (nd, d.ding.is_some_and(|x| !x.1.is_nan())) {
            (Some(n), _) => format!("推步须移 {n:.2} 时方同张表"),
            (None, true) => "差由排月细则（有交不进、三大三小等）或例外表，不在表格".to_string(),
            (None, false) => "本历无定朔细目".to_string(),
        };
        let hs = hits(d);
        if d.bd.is_none() {
            return format!("无表段细目；{nd_s}");
        }
        if hs.is_empty() {
            return format!("所用格无校改、异文；{nd_s}");
        }
        let parts: Vec<String> = hs
            .iter()
            .map(|(t, u)| match (u, nd) {
                (Some(u), Some(n)) => format!("{t}，影响上限 {u:.2} 时{}", if *u < n { "，不足以致异" } else { "，或可致异（须核方向）" }),
                (Some(u), None) => format!("{t}，影响上限 {u:.2} 时"),
                _ => t.clone(),
            })
            .collect();
        format!("{}；{nd_s}", parts.join("；"))
    };
    let mut csv = String::from("历,年,月,张表月名,推步朔,张表朔,张表−推步,定朔所在日,定小余（时）,进朔限（时）,距限（时）,经朔（时，距定朔所在日夜半）,有交,进退关系,本纪明书朔,纪日证据,已有考证,判定,分类,证据等级,按语,距子夜（时）,月离所入,日躔所入,距月离段界（时）,复核\n");
    let fmt_shuo = |d: &Diff| -> String {
        d.shuo.iter().map(|s| format!("{} {}{}", s.0, gz(s.1), if s.2 && s.0.contains("校勘表") { "（照校勘表不计）" } else if s.2 { "（已考讹）" } else { "" })).collect::<Vec<_>>().join("；")
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
        // 纪日只一见（独立条数不足二）不足以判①②：归③，注倾向（人工按语所判亦同）
        if verdict.starts_with("纪日证推步") || verdict.starts_with("纪日证张表") {
            let sup_r = verdict.starts_with("纪日证推步");
            if jiri_events(d, sup_r) < 2 {
                verdict = format!("纪日一见，倾向{}（{}）", if sup_r { "推步" } else { "张表" }, verdict);
            }
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
        let ziye = d.ding.map_or("—".to_string(), |(_, h, ..)| format!("{:.2}", h.min(24.0 - h)));
        let (yue, ri, jie) = fmt_bd(d.li, &d.bd);
        let jie_s = jie.map_or("—".to_string(), |j| format!("{j:.2}"));
        let fh = fuhe(d);
        csv += &format!(
            "{},{},{},{},{},{},{:+},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
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
            dengji(d, &verdict),
            an.replace(',', "，"),
            ziye,
            yue,
            ri,
            jie_s,
            fh.replace(',', "，")
        );
        let lv = dengji(d, &verdict);
        rows_out.push((d.clone(), cause, verdict, an, dd, h, lim, dist, lv, (ziye, yue, ri, jie_s, fh)));
    }
    std::fs::write(root.join("data/calendar/dingshuo_zhangbiao_diffs.csv"), csv).unwrap();

    // 表段分布：全部推步之月（有定朔细目者）按「月离入分段之日」「定小余距进朔限（无限者距子夜）≤ 0.5 时」分，与张表朔日不同者各占几何。
    // 用以检验「与张表之异多在进朔临界或月离分段处」之说
    let diff_set: BTreeSet<(&str, i64)> = diffs.iter().map(|d| (d.li, d.r)).collect();
    let mut fenbu: BTreeMap<&str, (i32, [usize; 8])> = BTreeMap::new();
    for (&y, r) in &reck {
        let name = names[&y];
        let Some(dn) = ding_name(name) else { continue };
        let e = fenbu.entry(name).or_insert((y, [0; 8]));
        for &(rj, _, _) in r {
            let Some(x) = details[dn].iter().find(|x| x.day == rj) else { continue };
            let df = diff_set.contains(&(name, rj)) as usize;
            let h = x.frac * 24.0;
            let near = match x.limit {
                Some(l) => (h - l * 24.0).abs() <= 0.5,
                None => h.min(24.0 - h) <= 0.5,
            };
            let split = x.biaoduan.as_ref().and_then(|b| b.yue).is_some_and(|(_, _, c)| c.is_some());
            let c = &mut e.1;
            c[0] += 1;
            c[1] += df;
            if split {
                c[2] += 1;
                c[3] += df;
            }
            if near {
                c[4] += 1;
                c[5] += df;
            }
            if !split && !near {
                c[6] += 1;
                c[7] += df;
            }
        }
    }
    let mut fenbu_md = String::from(
        "\n### 表段分布\n\n有定朔细目之历（宋初三历、钦天、时宪不在内）全部推步之月，按经朔所入月离是否为分段之日（七、十四、二十一、二十八日等初末分段），\n与定小余距本历进朔限（术文不进朔者距子夜）是否在 0.5 时以内，分别计与张表朔日不同者。\n\n| 历 | 月数 | 其中异 | 分段日之月 | 其中异 | 距限 ≤0.5 时之月 | 其中异 | 二者皆否之月 | 其中异 |\n|---|---|---|---|---|---|---|---|---|\n",
    );
    let mut ft = [0usize; 8];
    let mut fs: Vec<_> = fenbu.iter().collect();
    fs.sort_by_key(|(_, v)| v.0);
    for (n, (_, c)) in fs {
        for i in 0..8 {
            ft[i] += c[i];
        }
        fenbu_md += &format!("| {n} | {} | {} | {} | {} | {} | {} | {} | {} |\n", c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]);
    }
    let pct = |a: usize, b: usize| if b == 0 { "—".to_string() } else { format!("{:.1}%", 100.0 * a as f64 / b as f64) };
    fenbu_md += &format!(
        "| **合计** | {} | {}（{}） | {} | {}（{}） | {} | {}（{}） | {} | {}（{}） |\n",
        ft[0], ft[1], pct(ft[1], ft[0]), ft[2], ft[3], pct(ft[3], ft[2]), ft[4], ft[5], pct(ft[5], ft[4]), ft[6], ft[7], pct(ft[7], ft[6])
    );

    // 文档生成部分
    let mut md = String::new();
    md += "### 逐历汇总\n\n（脚本生成：`rust/core/examples/dingshuo_zhangbiao_diffs.rs`；逐月 CSV 见 `data/calendar/dingshuo_zhangbiao_diffs.csv`。）\n\n";
    md += "| 历 | 年段 | 月数 | 朔日不同之月 | 张表晚一日 | 张表早一日 | 闰位不同之岁 | 其中统一进朔所致 | ①文字证张表误 | ②文字证推步误 | ③中两歧、冲突或纪日一见 | ③无据待核（含两歧、冲突、一见） | 相同之月：有证 | 同误 | 无证 |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n";
    // 两家相同之月：取 main_series_reckoning_months.csv（examples/main_series_reckoning_eval.rs，须在本程序之后跑，再跑本程序一次以更新此栏）
    let mut same: HashMap<String, [usize; 3]> = HashMap::new();
    let mut same_lv: BTreeMap<(usize, String), usize> = BTreeMap::new();
    if let Ok(t) = std::fs::read_to_string(root.join("data/calendar/main_series_reckoning_months.csv")) {
        for r in parse_csv(&t).into_iter().skip(1).filter(|r| r.len() >= 8 && r[5] == "相同") {
            let Ok(y) = r[1].parse::<i32>() else { continue };
            if !(y_from..=y_to).contains(&y) || !stat.contains_key(r[0].as_str()) {
                continue;
            }
            let k = ["有证", "同误", "无证"].iter().position(|x| *x == r[6]).unwrap_or(2);
            same.entry(r[0].clone()).or_default()[k] += 1;
            if k < 2 {
                *same_lv.entry((k, r[7].clone())).or_default() += 1;
            }
        }
    }
    let mut same_tot = [0usize; 3];
    let mut total = [0usize; 9];
    let mut stat_sorted: Vec<_> = stat.iter().collect();
    stat_sorted.sort_by_key(|(_, s)| s.0);
    for (name, s) in stat_sorted {
        let rs: Vec<_> = rows_out.iter().filter(|x| x.0.li == *name).collect();
        if rs.is_empty() && s.4 == 0 && !same.contains_key(*name) {
            continue;
        }
        let late = rs.iter().filter(|x| x.0.t > x.0.r).count();
        let early = rs.len() - late;
        let uni = rs.iter().filter(|x| x.1.contains("张表统一四分之三进朔")).count();
        let pr = rs.iter().filter(|x| fenlei(&x.2).starts_with('①')).count();
        let pt = rs.iter().filter(|x| fenlei(&x.2).starts_with('②')).count();
        let bo = rs.iter().filter(|x| x.2.contains("两歧") || x.2.contains("冲突") || x.2.contains("一见")).count();
        let no = rs.len() - pr - pt;
        for (i, v) in [rs.len(), late, early, s.4, uni, pr, pt, bo, no].iter().enumerate() {
            total[i] += v;
        }
        let sm = same.get(*name).copied().unwrap_or_default();
        for i in 0..3 {
            same_tot[i] += sm[i];
        }
        md += &format!("| {name} | {}—{} | {} | {} | {late} | {early} | {} | {uni} | {pr} | {pt} | {bo} | {no} | {} | {} | {} |\n", s.0, s.1, s.2, rs.len(), s.4, sm[0], sm[1], sm[2]);
    }
    md += &format!(
        "| **合计** | | | **{}** | {} | {} | **{}** | {} | **{}** | **{}** | {} | {} | **{}** | **{}** | **{}** |\n\n",
        total[0], total[1], total[2], total[3], total[4], total[5], total[6], total[7], total[8], same_tot[0], same_tot[1], same_tot[2]
    );
    md += "相同之月（推步与张表朔日、月名皆同）：**有证**——实物、明书朔合两家，或两条以上独立纪日（落在朔日、晦日或书日序者）合两家；**同误**——史载与两家皆不合（注是否已判史书误）；**无证**——只是两家相同，不算证实（纪日只一见者亦归此，注倾向）。逐月见 `data/calendar/main_series_reckoning_months.csv`「分类」列。\n\n";
    md += "| 相同之月 | A | B | C | D | E | 计 |\n|---|---|---|---|---|---|---|\n";
    for (k, nm) in [(0usize, "有证"), (1, "同误")] {
        let c: Vec<usize> = ["A", "B", "C", "D", "E"].iter().map(|l| same_lv.get(&(k, l.to_string())).copied().unwrap_or(0)).collect();
        md += &format!("| {nm} | {} | {} |\n", c.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" | "), c.iter().sum::<usize>());
    }
    md += &format!("| 无证 | — | — | — | — | — | {} |\n\n", same_tot[2]);
    md += "分类：①有当时文字（本纪明书朔、纪日、写本、册文、题记等）证张表误、推步合；②有文字证推步误、张表合；③无据待核（无本纪纪日可判，或两歧；单条明书朔与纪日相冲突者亦不判，归③；纪日须异源两条一致才判——同一部书（含同源诸书：旧、新唐书，旧、新五代史等）同月诸事只算一源，2026-11-27 起——只一源者归③并注「纪日一见，倾向推步／张表」；E 级明书朔（《新五代史》前史无同条者）只一见亦归③；明清实录书朔为 C 级，明实录书朔经校勘表判讹或存疑者照其结论不计，书朔栏注「校勘表」）。三类之和即朔日不同之月。\n\n";
    // 三类 × 证据等级
    let lvls = ["A", "B", "C", "D", "E"];
    md += "证据等级：A 实物历日（具注历、历谱；抄本讹误另核）；B 当时文字（碑志、题记、简牍文书、册文、诏令原文）；C 当朝或近时所修史书明书「某月某干支朔」；D 当朝史书纪日干支（能分辨两家者）；E 后人追修史书（《后汉书》《晋书》；《南史》《北史》《新五代史》前史无同条者）。《新唐书》本纪多据实录，与《旧唐书》同级，两书同条只算一证；《南史》《北史》《新五代史》与前史同条者按前史计、不重计。复原性质材料（《通鉴目录》、今人诸表）不作证据。\n\n| 分类 | A | B | C | D | E | 计 |\n|---|---|---|---|---|---|---|\n";
    for (k, name) in [('①', "①文字证张表误"), ('②', "②文字证推步误")] {
        let rs: Vec<_> = rows_out.iter().filter(|x| fenlei(&x.2).starts_with(k)).collect();
        let c: Vec<String> = lvls.iter().map(|l| rs.iter().filter(|x| x.8 == *l).count().to_string()).collect();
        md += &format!("| {name} | {} | {} |\n", c.join(" | "), rs.len());
    }
    md += &format!("| ③无据待核 | — | — | — | — | — | {} |\n\n", rows_out.iter().filter(|x| fenlei(&x.2).starts_with('③')).count());
    md += "「其中统一进朔所致」：张表与推步之差恰可由张表「朔日小余在日法四分之三已上者，虚进一日」（李勇、张培瑜 1996 自述）与本历术文进朔限之不同解释者。\n\n";
    md += "### 有证之月\n\n| 历 | 年 | 月 | 推步 | 张表 | 定小余（时） | 限（时） | 距限 | 本纪明书朔 | 纪日 | 判定 | 分类 | 等级 | 按语 |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n";
    let cell = |s: String| if s.is_empty() { "—".to_string() } else { s.replace('|', "／") };
    for x in rows_out.iter().filter(|x| !x.0.shuo.is_empty() || !x.0.jiri.is_empty() || !x.0.art.is_empty() || !x.3.is_empty()) {
        let d = &x.0;
        md += &format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
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
            x.8,
            cell(x.3.clone())
        );
    }
    md += "\n### 逐月表（全部朔日不同之月）\n\n「定朔」为定朔所在日干支；「距限」= 定小余 − 进朔限（正为过限）；「有交」= 经朔入交入食限（大衍一系此等月定朔用二次差精法；— 为未复原交会）；「经朔」为经朔距定朔所在日夜半之时（负为前一日）。「距子夜」= 定小余距最近夜半之时；「月离」「日躔」为经朔所入之格（月离第几日、入余，分段之日注初末段与初数；日躔所入之气、入气日／本气日数）；「段界」= 入余距本日首尾或初末分界之时（按入转之时计）；「复核」见上「方法·逐月复核」：所用之格是否为校改、异文或疑讹之格（[dingshuo_biao_yige.csv](../../data/calendar/dingshuo_biao_yige.csv)），其影响上限是否足以致异，及推步须移几时方同张表。\n\n";
    md += "| 历 | 年 | 月 | 推步 | 张表 | 定朔 | 定小余 | 限 | 距限 | 距子夜 | 有交 | 月离 | 日躔 | 段界 | 进退关系 | 证据 | 已有考证 | 判定 | 分类 | 等级 | 复核 |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n";
    for x in &rows_out {
        let d = &x.0;
        let lab = match d.t_label {
            Some((m, l)) => format!("{}（张表{}）", label(d.month, d.leap), label(m, l)),
            None => label(d.month, d.leap),
        };
        let ev = [d.art.iter().map(|a| format!("{}合{}", a.0, if a.1 { "推步" } else { "张表" })).collect::<Vec<_>>().join("；"), fmt_shuo(d), d.jiri.iter().map(|j| format!("{}合{}", j.0, if j.1 { "推步" } else { "张表" })).collect::<Vec<_>>().join("；")]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("；");
        md += &format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            d.li,
            d.year,
            lab,
            gz(d.r),
            gz(d.t),
            x.4,
            x.5,
            x.6,
            x.7,
            x.9.0,
            d.youjiao.map_or("—", |b| if b { "有" } else { "" }),
            x.9.1,
            x.9.2,
            x.9.3,
            x.1.split('：').next().unwrap_or(""),
            cell(ev),
            cell(d.refs.join(" ")),
            x.2,
            fenlei(&x.2),
            x.8,
            cell(x.9.4.clone())
        );
    }
    md += &fenbu_md;
    // 复核归类：每月按先后次序归入一类（逐月见上表「复核」列）
    let mut gl: Vec<(&str, usize, usize, usize, usize)> = [
        "本历无定朔细目",
        "排月细则（有交不进、三大三小、正月之规等）或例外表",
        "张表统一四分之三进朔与术文之限不同",
        "麟德 709—728 年常月进朔（推步进、张表不进）",
        "近子夜（差在定朔所在日）",
        "临限：推步须移 ≤ 0.1 时",
        "推步须移 0.1—0.5 时",
        "推步须移 0.5—2 时",
        "推步须移 > 2 时",
    ]
    .iter()
    .map(|n| (*n, 0, 0, 0, 0))
    .collect();
    for x in &rows_out {
        let d = &x.0;
        let nd = need(d);
        let k = if d.ding.is_none_or(|v| v.1.is_nan()) {
            0
        } else if nd.is_none() {
            1
        } else if x.1.contains("张表统一四分之三进朔") {
            2
        } else if d.li.starts_with("麟德") && d.year >= 709 && d.r > d.t {
            3
        } else if x.1.starts_with("差在定朔所在日") {
            4
        } else {
            match nd.unwrap() {
                n if n <= 0.1 => 5,
                n if n <= 0.5 => 6,
                n if n <= 2.0 => 7,
                _ => 8,
            }
        };
        let hs = hits(d);
        let g = &mut gl[k];
        g.1 += 1;
        g.2 += (!hs.is_empty()) as usize;
        g.3 += hs.iter().any(|(_, u)| u.zip(nd).is_some_and(|(u, n)| u >= n)) as usize;
        g.4 += fenlei(&x.2).starts_with('①') as usize + fenlei(&x.2).starts_with('②') as usize;
    }
    md += "\n### 复核归类\n\n朔日不同之月逐月按下列次序归入一类；「用疑格」为所用日躔、月离之格或进朔限节点是校改、异文或疑讹之格（`data/calendar/dingshuo_biao_yige.csv`），「足以致异」为其影响上限不小于推步须移之时。\n\n| 类 | 月数 | 用疑格 | 疑格足以致异 | 已有①② |\n|---|---|---|---|---|\n";
    for g in &gl {
        md += &format!("| {} | {} | {} | {} | {} |\n", g.0, g.1, g.2, g.3, g.4);
    }
    md += &format!("| **合计** | {} | {} | {} | {} |\n", gl.iter().map(|g| g.1).sum::<usize>(), gl.iter().map(|g| g.2).sum::<usize>(), gl.iter().map(|g| g.3).sum::<usize>(), gl.iter().map(|g| g.4).sum::<usize>());
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
