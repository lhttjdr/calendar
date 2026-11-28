//! 分政权推步以各国史书逐条检验：`data/calendar/本纪纪日/*.csv`（各史本纪纪日，带政权列）中属分政权之条，
//! 以该政权推步（`push_calendar::regime_year_months`）与主序列今人历表（张表，作对照：看本国史书支持的是本国之历还是中原之历）
//! 两家检验：纪日干支是否落在所记之月（书「朔」者须为朔日，书「晦」者须为晦日）。另读北朝诸史（《魏书》《隋书》）与《金史》天文志之凌犯、
//! 日月食纪日（`data/phenomena/`），及实物与当时文字：走马楼吴简（`zoumalou_months.csv`）、辽代石刻（`liao_shike_shuo.csv`）、
//! 明郑大统历实物（`mingzheng_shiwu_months.csv`），并入逐月分类。
//! 输出 data/calendar/regime_benji_check.csv（本纪逐条）、regime_zhi_check.csv（天文志逐条）、regime_benji_months.csv（逐月分类）、
//! doc/calendar/regime-benji-checks.md（汇总与两家不同之条）。先跑 `scripts/calendar_data/zoumalou_shuo_check.py check`。
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example regime_benji_eval
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::historical::ShuoriTable;
use lunar_core::calendar::lifa::{main_series_year_months, MainSeriesPreference};
use lunar_core::calendar::push_calendar::{regime_year_months, regime_year_months_raw};
use std::collections::BTreeMap;

/// 史书政权名 → 分政权推步之键
fn regime_key(r: &str) -> Option<&'static str> {
    Some(match r {
        "吴" => "吴",
        "蜀" => "蜀",
        "后秦" => "后秦",
        "北凉" => "北凉",
        "北魏" | "西魏" | "北周" | "隋" => "北魏/西魏/北周/隋",
        "东魏" | "北齐" => "东魏/北齐",
        "辽" => "辽",
        "金" => "金",
        "西夏" => "西夏",
        "南明" | "明郑" | "鲁监国" => "南明/南明·绍武/鲁监国/明郑",
        _ => return None,
    })
}

/// 检验开关（只供分解各项改动之效，缺省全开）：环境变量 `ABLATE` 列出要关掉的项，逗号分隔——
/// `jia`（D、E 级判①②须异源两条；关则同源不同日之事各算一条）、`yi`（南明私家追记、《建康实录》吴部分按 E 计）、
/// `zoumalou`（走马楼吴简）、`liao`（辽代石刻）、`mingzheng`（明郑大统历实物）、`zhi`（北朝诸史与《金史》天文志纪日）
fn on(f: &str) -> bool {
    !std::env::var("ABLATE").unwrap_or_default().split(',').any(|x| x.trim() == f)
}

/// 证据等级：《后汉书》《晋书》与《南史》《北史》《新五代史》（前史无同条者）为 E；南明诸书（`nanming_*`：《小腆纪年》《海东逸史》
/// 《明季南略》等清人私家追记，及《弘光实录钞》《行在阳秋》）与《建康实录》吴部分（唐许嵩撰，记吴事后约五百年）亦为 E；
/// 余者（含《新唐书》，多据实录；含各史天文志 `*_zhi`）明书朔 C、纪日 D。
/// 同一史源之书（见 [`lineage`]）互见之条只算一证：等级取同源诸书最高者（前史有同条即按前史计）。
fn book_level(book: &str, shuo: bool) -> char {
    let later = ["houhanshu", "jinshu", "nanshi", "beishi", "xinwudaishi"];
    let later_yi = ["nanming_", "jiankang_shilu_wu"];
    if later.iter().any(|b| book.starts_with(b)) || (on("yi") && later_yi.iter().any(|b| book.starts_with(b))) {
        'E'
    } else if shuo {
        'C'
    } else {
        'D'
    }
}

/// 史源：同一史源之书（《北史》与魏、北齐、周、隋书；《南史》与南朝四书；旧、新唐书；旧、新五代史；《建康实录》吴部分与《三国志·吴书》）
/// 只算一源；同书本纪与天文志（`*_zhi`）同源
fn lineage(book: &str) -> &str {
    match book.trim_end_matches("_benji").trim_end_matches("_zhi") {
        "jiutangshu" | "xintangshu" => "唐",
        "jiuwudaishi" | "xinwudaishi" => "五代",
        "beishi" | "weishu" | "beiqishu" | "zhoushu" | "suishu" => "北朝",
        "nanshi" | "songshu" | "nanqishu" | "liangshu" | "chenshu" => "南朝",
        "sanguozhi_wushu" | "jiankang_shilu_wu" => "吴",
        x => x,
    }
}

/// 互异之月的一条证据
#[derive(Clone)]
struct Ev {
    /// 支持推步（否则支持张表）
    sup_r: bool,
    lv: char,
    /// 合推步靠史载例外表「本纪」类修正（证据与修正同源，非独立）
    benji_fixed: bool,
    /// 书或实物表名（史源按 [`lineage`] 归并）
    book: String,
    shuo: bool,
    /// 说明（实物、当时文字之条）
    what: String,
}

/// 一月诸证之分类（互异之月）：有 A 级者只看 A 级；余者诸证须同向（两向为冲突，归③）；A、B 级一条、C 级（明书朔）一条即可判；
/// D、E 级须异源两条（开关 `jia` 关时同源不同日之事各算一条）。→ (分类, 等级)
fn classify_diff(es: &[Ev]) -> (String, char) {
    if es.is_empty() {
        return ("③无据待核".into(), '—');
    }
    let top: Vec<&Ev> = if es.iter().any(|e| e.lv == 'A') { es.iter().filter(|e| e.lv == 'A').collect() } else { es.iter().collect() };
    let side_r = top.iter().all(|e| e.sup_r);
    if !side_r && !top.iter().all(|e| !e.sup_r) {
        return ("③无据待核（证据冲突）".into(), '—');
    }
    let best = top.iter().map(|e| e.lv).min().unwrap();
    let hard = top.iter().any(|e| matches!(e.lv, 'A' | 'B') || (e.lv == 'C' && e.shuo));
    let lins: std::collections::BTreeSet<&str> = top.iter().map(|e| lineage(&e.book)).collect();
    // 旧规则（`jia` 关）：同源内取条数最多之书计条数（同书同文只算一条），各史源相加
    let mut per: BTreeMap<(&str, &str), std::collections::BTreeSet<&str>> = BTreeMap::new();
    for e in &top {
        per.entry((lineage(&e.book), e.book.as_str())).or_default().insert(e.what.as_str());
    }
    let mut by_lin: BTreeMap<&str, usize> = BTreeMap::new();
    for ((l, _), v) in &per {
        let x = by_lin.entry(l).or_default();
        *x = (*x).max(v.len());
    }
    let events: usize = by_lin.values().sum();
    let n_ok = if on("jia") { lins.len() } else { events };
    let tilt = if side_r { "推步" } else { "张表" };
    if !hard && n_ok < 2 {
        let why = if events > 1 { format!("史书只一源 {events} 条") } else { "纪日一见".to_string() };
        return (format!("③无据待核（{why}，倾向{tilt}）"), '—');
    }
    ((if side_r { "①证张表误" } else { "②证推步误" }).to_string(), best)
}

fn leap_lab(who: &str, m: u8) -> String {
    if m == 0 { format!("{who}此后无闰") } else { format!("{who}闰{m}") }
}

fn gz(j: i64) -> &'static str {
    GAN_ZHI_60[(j + 49).rem_euclid(60) as usize]
}

/// 某岁某月之 [始, 终) 日：月首取本岁所列，月终取本岁、次岁所列之中首个更晚之朔
fn span(f: &dyn Fn(i32) -> Option<Vec<(i64, u8, bool)>>, y: i32, m: u8, leap: bool) -> Option<(i64, i64)> {
    let cur = f(y)?;
    let a = cur.iter().find(|e| e.1 == m && e.2 == leap)?.0;
    let b = cur.iter().chain(f(y + 1).unwrap_or_default().iter()).map(|e| e.0).filter(|&j| j > a).min()?;
    Some((a, b))
}

fn csv_rows(text: &str) -> Vec<BTreeMap<String, String>> {
    let mut rdr = text.lines().filter(|l| !l.starts_with('#'));
    let head: Vec<String> = rdr.next().unwrap_or_default().split(',').map(|s| s.to_string()).collect();
    rdr.map(|l| {
        let f: Vec<&str> = l.splitn(head.len(), ',').collect();
        head.iter().cloned().zip(f.iter().map(|s| s.to_string()).chain(std::iter::repeat(String::new()))).collect()
    })
    .collect()
}

#[derive(Default)]
struct Stat {
    n: usize,
    r: usize,
    t: usize,
    only_r: usize,
    only_t: usize,
    shuo: usize,
    shuo_r: usize,
    shuo_t: usize,
    /// 考证表「不计」之条（本纪月份、干支已考为讹，或解析之误）
    excluded: usize,
    /// 合推步之条中，纯推步不合、靠史载修正才合，且当年修正有「本纪」类者
    benji_fix: usize,
    /// 同上，当年修正只有「他书」类者
    other_fix: usize,
    /// 合主序列（本项目缺省取用之历日：实物、推步，否则张表）
    z: usize,
    only_r_z: usize,
    only_z: usize,
}

/// 分政权考证表（`data/calendar/*_regime_kaozheng.csv`，列含 书、年、月、干支、处置、改作）中处置为「改月」「不计」之条：
/// (书, 年, 月, 干支) → (处置, 改作)。「改月」= 本纪月名已考为讹（如辽 985 年「閏九月」实为九月，据同月「庚辰，重九」），按改作之月判；
/// 「不计」= 不作检验之据。其余处置（存、已入例外表等）只是考证记录，不影响检验。
fn kaozheng(root: &std::path::Path) -> BTreeMap<(String, String, String, String), (String, String)> {
    let mut out = BTreeMap::new();
    let Ok(dir) = std::fs::read_dir(root.join("data/calendar")) else { return out };
    for e in dir.filter_map(|e| e.ok()) {
        let name = e.file_name().to_string_lossy().to_string();
        if !name.ends_with("_regime_kaozheng.csv") {
            continue;
        }
        for r in csv_rows(&std::fs::read_to_string(e.path()).unwrap()) {
            let (Some(c), Some(g)) = (r.get("处置"), r.get("改作")) else { break };
            if c == "改月" || c == "不计" {
                out.insert((r["书"].clone(), r["年"].clone(), r["月"].clone(), r["干支"].clone()), (c.clone(), g.clone()));
            }
        }
    }
    out
}

fn main() {
    // 仓库根：当前目录是仓库根（有 data/calendar/本纪纪日）时取之（worktree 共用 target 时编译期路径指向别处），否则按编译期路径
    let root = std::env::current_dir()
        .ok()
        .filter(|d| d.join("data/calendar/本纪纪日").is_dir())
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    let zhang = ShuoriTable::from_csv(&read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let mut files: Vec<_> = std::fs::read_dir(root.join("data/calendar/本纪纪日"))
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "csv") && !p.to_string_lossy().contains("閏月"))
        .collect();
    files.sort();
    let mut stats: BTreeMap<(String, String, String), Stat> = BTreeMap::new();
    let mut per = String::from("书,政权,推步之历,年,月,干支,位置,推步月首,推步合,张表月首,张表合,原文,中原月首,中原合\n");
    let mut diff: Vec<String> = Vec::new();
    let kz = kaozheng(&root);
    // 推步与张表互异之月的史书证据：(推步之键, 年, 推步月首) → [(支持推步, 等级, 靠本纪修正)]
    // 证据项：(支持推步, 等级, 靠本纪修正, 书, 书朔)
    #[allow(clippy::type_complexity)]
    let mut ev: BTreeMap<(String, i32, String), Vec<Ev>> = BTreeMap::new();
    // 两家相同之月的证据：(推步之键, 年, 月首) → [(合两家, 等级, 书, 书朔, 文首)]
    #[allow(clippy::type_complexity)]
    let mut same_ev: BTreeMap<(String, i32, String), Vec<(bool, char, String, bool, String)>> = BTreeMap::new();
    // 史书来源：各史本纪纪日（逐条写 regime_benji_check.csv），另有北朝诸史与《金史》天文志之凌犯、日月食纪日（`data/phenomena/`，
    // 书名记 `<书>_zhi`，与同书本纪同源；逐条另写 regime_zhi_check.csv，不入本纪逐条表以免改动读该表的考证脚本）
    let mut sources: Vec<(String, Vec<BTreeMap<String, String>>, bool)> =
        files.iter().map(|p| (p.file_stem().unwrap().to_string_lossy().to_string(), csv_rows(&std::fs::read_to_string(p).unwrap()), false)).collect();
    if on("zhi") {
        for b in ["weishu", "suishu", "jinshi"] {
            let mut rows = Vec::new();
            for kind in ["star_records", "solar_eclipses", "lunar_eclipses"] {
                let Ok(t) = std::fs::read_to_string(root.join(format!("data/phenomena/{b}_{kind}.csv"))) else { continue };
                for mut r in csv_rows(&t) {
                    // 凌犯表月列「闰」「闰5」，无 leap、position 列；日月食表有之（月食书「望」等不作朔晦）
                    let leap = r.get("leap").is_some_and(|l| l == "1") || r["month"].starts_with('闰');
                    r.insert("leap".into(), (leap as u8).to_string());
                    let pos = r.get("position").cloned().unwrap_or_default();
                    r.insert("position".into(), if pos == "朔" || pos == "晦" { pos } else { String::new() });
                    rows.push(r);
                }
            }
            sources.push((format!("{b}_zhi"), rows, true));
        }
    }
    let mut per_zhi = String::from("书,政权,推步之历,年,月,干支,位置,推步月首,推步合,张表月首,张表合,原文,中原月首,中原合\n");
    for (book, rows, is_zhi) in sources {
        // 上一条所记之年、月（不闰）：本纪「閏月」不书月数（月列只记「闰」），只保证闰月在所记最后一月之后
        let mut last: Option<(i32, u8)> = None;
        for r in rows {
            let Ok(y) = r["year"].parse::<i32>() else { continue };
            // 月列：「6」、「闰6」（閏六月）、「闰」（閏月，不书月数）
            let ms = r["month"].trim_start_matches('闰');
            let leap = r["leap"] == "1" || r["month"].starts_with('闰');
            let m_written: Option<u8> = ms.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().ok();
            if let (Some(m), false) = (m_written, leap) {
                last = Some((y, m));
            }
            let Some(key) = regime_key(&r["regime"]) else { continue };
            let Some(gi) = GAN_ZHI_60.iter().position(|g| *g == r["ganzhi"]) else { continue };
            let fix = kz.get(&(book.clone(), r["year"].clone(), r["month"].clone(), r["ganzhi"].clone()));
            if fix.is_some_and(|f| f.0 == "不计") {
                stats.entry((key.to_string(), r["regime"].clone(), book.clone())).or_default().excluded += 1;
                continue;
            }
            // 「改月」：按考证所改之月（如「9」「闰8」）判
            let (m_written, leap) = match fix {
                Some((_, g)) => (g.trim_start_matches('闰').parse::<u8>().ok(), g.starts_with('闰')),
                None => (m_written, leap),
            };
            let reg = |yy: i32| regime_year_months(key, yy).map(|x| x.1);
            let Some(li) = regime_year_months(key, y).map(|x| x.0) else { continue };
            let tab = |yy: i32| zhang.year_months(yy).map(|t| t.to_vec());
            let zy = |yy: i32| main_series_year_months(Some(&zhang), yy, MainSeriesPreference::Evidence).map(|x| x.0).or_else(|| tab(yy));
            // 「閏月」不书月数：取该历当年闰月（须不早于上条所记之月，否则视为不合，月记 0）；两家各按其历之闰
            let leap_of = |f: &dyn Fn(i32) -> Option<Vec<(i64, u8, bool)>>| -> u8 {
                let lo = last.filter(|l| l.0 == y).map_or(1, |l| l.1);
                f(y).unwrap_or_default().iter().find(|e| e.2 && e.1 >= lo).map_or(0, |e| e.1)
            };
            let (m, m_reg, m_tab) = match m_written {
                Some(m) => (m, m, m),
                None if leap => (0, leap_of(&reg), leap_of(&tab)),
                None => continue,
            };
            let pos = r.get("position").cloned().unwrap_or_default();
            let judge = |sp: Option<(i64, i64)>| -> (String, bool) {
                let Some((a, b)) = sp else { return ("—".into(), false) };
                let d = a + (gi as i64 - (a + 49)).rem_euclid(60);
                // 「日N」：书日序与干支（如「十八日庚申」），干支须为月首起第 N 日
                let nth = pos.strip_prefix("日").and_then(|n| n.parse::<i64>().ok());
                let ok = d < b
                    && match pos.as_str() {
                        "朔" => d == a,
                        "晦" => d == b - 1,
                        _ => nth.is_none_or(|n| d == a + n - 1),
                    };
                (gz(a).to_string(), ok)
            };
            let (gr, okr) = judge(span(&reg, y, m_reg, leap));
            // 不含史载修正之推步：修正后合而修正前不合者计「史载修正而合」
            let raw = |yy: i32| regime_year_months_raw(key, yy).map(|x| x.1);
            let m_raw = if m == 0 { leap_of(&raw) } else { m_reg };
            let fixed = okr && !judge(span(&raw, y, m_raw, leap)).1;
            let (gt, okt) = judge(span(&tab, y, m_tab, leap));
            // 两家相同之月（朔日、月名同）：书朔一条可证；纪日只取能分辨朔日之条（朔日、晦日或书日序）；落在两家该月之外为同误之证
            if let (Some((a, br)), Some((at, bt))) = (span(&reg, y, m_reg, leap), span(&tab, y, m_tab, leap)) {
                if a == at && m_reg == m_tab {
                    let d = a + (gi as i64 - (a + 49)).rem_euclid(60);
                    let nth = pos.strip_prefix("日").and_then(|n| n.parse::<i64>().ok());
                    let (in_r, in_t) = (d < br, d < bt);
                    let item = if !in_r && !in_t {
                        Some(false)
                    } else if in_r && in_t {
                        match (pos.as_str(), nth) {
                            ("朔", _) => Some(d == a),
                            ("晦", _) => (br == bt).then_some(d == br - 1),
                            (_, Some(n)) => Some(d == a + n - 1),
                            _ => (d == a || (br == bt && d == br - 1)).then_some(true),
                        }
                    } else {
                        None
                    };
                    if let Some(ok) = item {
                        let t20: String = r.get("text").cloned().unwrap_or_default().chars().take(20).collect();
                        same_ev.entry((key.to_string(), y, format!("{}（{}{m_reg}）", gz(a), if leap { "闰" } else { "" }))).or_default().push((ok, book_level(&book, pos == "朔"), book.clone(), pos == "朔", t20));
                    }
                }
            }
            let m_zy = if m == 0 { leap_of(&zy) } else { m_reg };
            let (gzy, okz) = judge(span(&zy, y, m_zy, leap));
            let text = r.get("text").cloned().unwrap_or_default().replace(',', "，");
            let mn0 = if m == 0 { format!("闰（{}，{}）", leap_lab("推步", m_reg), leap_lab("张表", m_tab)) } else { format!("{}{}", if leap { "闰" } else { "" }, m) };
            let mn = if fix.is_some() { format!("{}→{mn0}（考证改月）", r["month"]) } else { mn0 };
            let line = format!("{book},{},{li},{y},{mn},{},{pos},{gr},{},{gt},{},{text},{gzy},{}\n", r["regime"], r["ganzhi"], okr as u8, okt as u8, okz as u8);
            if is_zhi {
                per_zhi += &line;
            } else {
                per += &line;
            }
            if okr != okt && gr != "—" {
                let benji_fixed = okr && fixed && lunar_core::calendar::dingshuo::shizai_liwai()
                    .iter()
                    .any(|r| r.0.starts_with(&format!("{key}：")) && (r.1 == y || r.1 == y - 1 || r.1 == y + 1) && r.4 == "本纪");
                let what: String = format!("{}{}", r["ganzhi"], text.chars().take(20).collect::<String>());
                ev.entry((key.to_string(), y, format!("{gr}（{}{m_reg}）", if leap { "闰" } else { "" }))).or_default().push(Ev {
                    sup_r: okr,
                    lv: book_level(&book, pos == "朔"),
                    benji_fixed,
                    book: book.clone(),
                    shuo: pos == "朔",
                    what,
                });
            }
            let s = stats.entry((key.to_string(), r["regime"].clone(), book.clone())).or_default();
            s.n += 1;
            s.r += okr as usize;
            s.t += okt as usize;
            s.only_r += (okr && !okt) as usize;
            s.only_t += (okt && !okr) as usize;
            if okr && fixed {
                let benji = lunar_core::calendar::dingshuo::shizai_liwai()
                    .iter()
                    .any(|r| r.0.starts_with(&format!("{key}：")) && (r.1 == y || r.1 == y - 1 || r.1 == y + 1) && r.4 == "本纪");
                if benji {
                    s.benji_fix += 1;
                } else {
                    s.other_fix += 1;
                }
            }
            s.z += okz as usize;
            s.only_r_z += (okr && !okz) as usize;
            s.only_z += (okz && !okr) as usize;
            if pos == "朔" {
                s.shuo += 1;
                s.shuo_r += okr as usize;
                s.shuo_t += okt as usize;
            }
            if okr != okt || okr != okz || !okr {
                diff.push(format!(
                    "| {} | {book} | {li} | {y} | {mn} | {}{pos} | {gr} | {} | {gt} | {} | {gzy} | {} | {} |",
                    r["regime"], r["ganzhi"], if okr { "✓" } else { "✗" }, if okt { "✓" } else { "✗" }, if okz { "✓" } else { "✗" }, text.chars().take(40).collect::<String>()
                ));
            }
        }
    }
    std::fs::write(root.join("data/calendar/regime_benji_check.csv"), &per).unwrap();
    std::fs::write(root.join("data/calendar/regime_zhi_check.csv"), &per_zhi).unwrap();
    let mut md = vec![
        "# 分政权推步之史书逐条检验（自动生成）".to_string(),
        String::new(),
        "> 由 `rust/core/examples/regime_benji_eval.rs` 生成，勿手改；逐条见 `data/calendar/regime_benji_check.csv`。方法见 [historical-calendar.md](historical-calendar.md)「分政权推步的史书检验」。".to_string(),
        String::new(),
        "纪日取自各史本纪（`data/calendar/本纪纪日/`，按政权列归属）。「推步」= 该政权所行之历按术推步（分政权推步）；「张表」= 主序列的今人历表，作对照——本国史书若合本国推步而不合张表，即本国另行一历之证。纪日合 = 干支落在所记之月（书朔者须为朔日、书晦者须为晦日，书日序者干支须为该日）。「主序列」= 本项目中原历日缺省取用之来源（实物、推步、古人长历；南明期间即清新法推步与时宪书实物），也作对照。".to_string(),
        String::new(),
        "## 汇总".to_string(),
        String::new(),
        "「合推步」含史载修正（例外表历名「政权：历名」）；其中纯推步不合、靠修正才合者另列：「从本纪修正」= 当年修正有只据本纪之条，「他书修正」= 当年修正皆有本纪以外的记录；合推步减去二者即纯按术推步之合数。「考讹不计」= 考证表处置为「不计」之条。".to_string(),
        String::new(),
        "| 推步之键 | 史书政权 | 书 | 纪日 | 合推步 | 合张表 | 只合推步 | 只合张表 | 书朔 | 朔合推步 | 朔合张表 | 考讹不计 | 从本纪修正 | 他书修正 | 合主序列 | 只合推步（对中原） | 只合中原 |".to_string(),
        "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|".to_string(),
    ];
    for ((k, rg, b), s) in &stats {
        md.push(format!(
            "| {k} | {rg} | {b} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            s.n, s.r, s.t, s.only_r, s.only_t, s.shuo, s.shuo_r, s.shuo_t, s.excluded, s.benji_fix, s.other_fix, s.z, s.only_r_z, s.only_z
        ));
        println!(
            "{k} {rg} {b}: {} 推步 {} 张表 {} 只推 {} 只张 {} | 朔 {} {} {} | 不计 {} 修正 本纪 {} 他书 {} | 中原 {} 只推 {} 只中原 {}",
            s.n, s.r, s.t, s.only_r, s.only_t, s.shuo, s.shuo_r, s.shuo_t, s.excluded, s.benji_fix, s.other_fix, s.z, s.only_r_z, s.only_z
        );
    }
    // 实物与当时文字（A、B 级，一条即可判）：走马楼吴简（逐月所据之简取 `scripts/calendar_data/zoumalou_shuo_check.py` 所写
    // data/calendar/zoumalou_months.csv，与 doc/calendar/zoumalou-checks.md 同一分类之据；推步、张表朔须与本程序所推相同）、
    // 辽代石刻（data/calendar/liao_shike_shuo.csv，书朔或由日序与日干支倒推之朔「(推)」）、明郑《大统历》实物
    // （data/calendar/mingzheng_shiwu_months.csv）。所记朔按月名与两家同名之月比。
    let mut same_hard: BTreeMap<(String, i32, String), Vec<(bool, char, String)>> = BTreeMap::new();
    let mut shiwu_stat: BTreeMap<&str, [usize; 4]> = BTreeMap::new(); // 源 → [入表之条, 落互异之月且只合一家, 落相同之月, 两家皆无此月]
    #[allow(clippy::too_many_arguments)]
    fn add_obs(
        ev: &mut BTreeMap<(String, i32, String), Vec<Ev>>,
        same: &mut BTreeMap<(String, i32, String), Vec<(bool, char, String)>>,
        zhang: &ShuoriTable,
        key: &str,
        (y, m, leap): (i32, u8, bool),
        shuo: &str,
        lv: char,
        src: &str,
        what: String,
    ) -> usize {
        let Some((_, r)) = regime_year_months(key, y) else { return 3 };
        let Some(a) = r.iter().find(|e| e.1 == m && e.2 == leap).map(|e| e.0) else { return 3 };
        let t = zhang.year_months(y).map(|t| t.to_vec()).unwrap_or_default();
        let lab = format!("{}（{}{m}）", gz(a), if leap { "闰" } else { "" });
        if t.contains(&(a, m, leap)) {
            same.entry((key.to_string(), y, lab)).or_default().push((shuo == gz(a), lv, what));
            return 2;
        }
        let tz = t.iter().find(|e| e.1 == m && e.2 == leap).map(|e| gz(e.0));
        let (sr, st) = (shuo == gz(a), tz == Some(shuo));
        if sr == st {
            return 0;
        }
        ev.entry((key.to_string(), y, lab)).or_default().push(Ev { sup_r: sr, lv, benji_fixed: false, book: src.to_string(), shuo: true, what });
        1
    }
    let read_rows = |p: &str| -> Vec<BTreeMap<String, String>> { std::fs::read_to_string(root.join(p)).map(|t| csv_rows(&t)).unwrap_or_default() };
    if on("zoumalou") {
        let wu = |y: i32, m: u8, l: bool| regime_year_months("吴", y).and_then(|x| x.1.into_iter().find(|e| e.1 == m && e.2 == l)).map(|e| gz(e.0));
        let wei = |y: i32, m: u8, l: bool| zhang.year_months(y).and_then(|t| t.iter().find(|e| e.1 == m && e.2 == l).copied()).map(|e| gz(e.0));
        for r in read_rows("data/calendar/zoumalou_months.csv") {
            let (Ok(y), Ok(m)) = (r["年"].parse::<i32>(), r["月"].parse::<u8>()) else { continue };
            let l = r["闰"] == "1";
            assert!(
                wu(y, m, l).unwrap_or("") == r["吴推步朔"] && wei(y, m, l).unwrap_or("") == r["张表朔"],
                "zoumalou_months.csv {y}-{m} 推步或张表朔与本程序不同，先重跑 scripts/calendar_data/zoumalou_shuo_check.py check"
            );
            for (i, sh) in r["简文朔"].split('；').enumerate() {
                let what = format!("走马楼吴简 {}（{sh}朔）", r["简"].split('；').nth(i).unwrap_or(""));
                let c = add_obs(&mut ev, &mut same_hard, &zhang, "吴", (y, m, l), sh, 'B', "zoumalou", what);
                let st = shiwu_stat.entry("走马楼吴简").or_default();
                st[0] += 1;
                st[c] += (c > 0) as usize;
            }
        }
    }
    if on("liao") {
        for r in read_rows("data/calendar/liao_shike_shuo.csv") {
            let Ok(y) = r["年"].parse::<i32>() else { continue };
            let mm = r["月"].trim_end_matches('月');
            let l = mm.starts_with('閏') || mm.starts_with('闰');
            let Ok(m) = mm.trim_start_matches(['閏', '闰']).parse::<u8>() else { continue };
            let sh = r["石刻朔"].split('(').next().unwrap_or("").to_string();
            if !GAN_ZHI_60.contains(&sh.as_str()) {
                continue;
            }
            let what = format!("辽代石刻《{}》{}书页 {}（{}朔）", r["原文"].chars().take(12).collect::<String>(), r["书"], r["书页"], r["石刻朔"]);
            let c = add_obs(&mut ev, &mut same_hard, &zhang, "辽", (y, m, l), &sh, 'B', "liao_shike", what);
            let st = shiwu_stat.entry("辽代石刻").or_default();
            st[0] += 1;
            st[c] += (c > 0) as usize;
        }
    }
    if on("mingzheng") {
        for r in read_rows("data/calendar/mingzheng_shiwu_months.csv") {
            let (Ok(y), Ok(m)) = (r["年"].parse::<i32>(), r["月"].parse::<u8>()) else { continue };
            let what = format!("明郑大统历实物（{}）", r.get("出处").map_or("", |s| s.as_str()).chars().take(24).collect::<String>());
            let c = add_obs(&mut ev, &mut same_hard, &zhang, "南明/南明·绍武/鲁监国/明郑", (y, m, r["闰"] == "1"), &r["朔干支"], 'A', "mingzheng_shiwu", what);
            let st = shiwu_stat.entry("明郑大统历实物").or_default();
            st[0] += 1;
            st[c] += (c > 0) as usize;
        }
    }
    for (k, v) in &shiwu_stat {
        println!("实物 {k}：入表 {} 条，落互异之月且只合一家 {}，落相同之月 {}，推步无此月 {}", v[0], v[1], v[2], v[3]);
    }
    // 推步与张表互异之月：逐政权三分类。互异之月 = 推步之月（朔日与月名）不见于张表者；有纪日只合一家者按证据定：
    // ①诸条皆合推步（史载证张表不能代表本政权历日），②皆合张表（史载证推步误），两向皆有为冲突，归③；无纪日可判亦③。
    let keys: std::collections::BTreeSet<String> = ev.keys().map(|k| k.0.clone()).chain(stats.keys().map(|k| k.0.clone())).collect();
    md.extend([
        String::new(),
        "## 相同之月与互异之月".to_string(),
        String::new(),
        "「互异之月」= 本政权推步之月（朔日、月名）不见于张表者（逐年比，史载修正后）。月内诸证只合一家者：皆合推步为**①证张表误**（张表不能代表本政权历日），皆合张表为**②证推步误**，两向皆有为证据冲突，归**③无据待核**；无证可判者亦归③。证据：实物历日（A，明郑大统历）、当时文字（B，走马楼吴简、辽代石刻）一条即可判，有 A 级者只看 A 级；史书明书朔（C）一条即可判；纪日（D、E）须**异源两条**——同一部书（含同源诸书：旧、新唐书，旧、新五代史，《北史》与魏、北齐、周、隋书，《南史》与南朝四书，《建康实录》吴部分与《三国志·吴书》；同书本纪与天文志）同月诸事只算一源（2026-11-27 起），只一源者归③（「纪日一见」或「史书只一源 N 条」，注倾向）。证据等级：C 当朝或近时所修史书明书朔，D 当朝史书纪日（含天文志；《新唐书》多据实录，与《旧唐书》同级），E 后人追修史书（《后汉书》《晋书》，《南史》《北史》《新五代史》前史无同条者；南明诸书——《小腆纪年》《海东逸史》《明季南略》等清人私家追记与《弘光实录钞》《行在阳秋》——及《建康实录》吴部分，2026-11-27 起）。实物、当时文字、天文志所据之条见逐月表「注」列。「①中靠本纪修正」= 推步本按术不合、经史载例外表「本纪」类修正才合之月，证据与修正同源，非独立。合张表不算证实张表。".to_string(),
        String::new(),
        "| 推步之键 | 相同之月 | 相同：有证 | 同误 | 无证 | 互异之月 | ①证张表误 | ①中靠本纪修正 | ②证推步误 | ③无据待核 | ③中证据冲突 | ③中一见或一源 | ① A/B/C/D/E | ② A/B/C/D/E |".to_string(),
        "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|".to_string(),
    ]);
    let mut tot = [0usize; 7];
    let mut same_tot = [0usize; 3];
    let mut mcsv = String::from("推步之键,年,月,推步月首,异同,分类,证据等级,注\n");
    let mut same_lv: BTreeMap<(usize, char), usize> = BTreeMap::new();
    let mut tot_lv: BTreeMap<(char, char), usize> = BTreeMap::new();
    let mut ev_rows: Vec<String> = Vec::new();
    for k in &keys {
        let mut n_diff = 0usize;
        let mut sc = [0usize; 3];
        let mut diff_months: Vec<(i32, (i64, u8, bool))> = Vec::new();
        let mut diff_cls: BTreeMap<(i32, String), (String, String)> = BTreeMap::new();
        for y in -300..=1700 {
            let Some((_, r)) = regime_year_months(k, y) else { continue };
            let t = zhang.year_months(y).map(|t| t.to_vec()).unwrap_or_default();
            n_diff += r.iter().filter(|m| !t.contains(m)).count();
            diff_months.extend(r.iter().filter(|m| !t.contains(m)).map(|m| (y, *m)));
            for m in r.iter().filter(|m| t.contains(m)) {
                let es = same_ev.get(&(k.clone(), y, format!("{}（{}{}）", gz(m.0), if m.2 { "闰" } else { "" }, m.1))).cloned().unwrap_or_default();
                // 独立条数（同源取条数最多之书）与等级（同源最高）
                let count = |sel: &dyn Fn(&(bool, char, String, bool, String)) -> bool| -> (usize, char, bool) {
                    let mut seen = std::collections::BTreeSet::new();
                    let mut per: BTreeMap<(&str, &str), usize> = BTreeMap::new();
                    let mut lvb = 'E';
                    let mut shuo = false;
                    for e in es.iter().filter(|e| sel(e)) {
                        if seen.insert((e.2.clone(), e.4.clone())) {
                            *per.entry((lineage(&e.2), e.2.as_str())).or_default() += 1;
                        }
                        lvb = lvb.min(e.1);
                        shuo |= e.3;
                    }
                    let mut by: BTreeMap<&str, usize> = BTreeMap::new();
                    for ((l, _), n) in per {
                        let x = by.entry(l).or_default();
                        *x = (*x).max(n);
                    }
                    (by.values().sum(), lvb, shuo)
                };
                let (ns, mut ls, ss) = count(&|e| e.0);
                let (nc, mut lc, cs) = count(&|e| !e.0);
                // 实物、当时文字（A、B）一条即可：有合两家者为有证，只有不合者为同误；史书之条不能推翻（证据由上往下）
                let hs = same_hard.get(&(k.clone(), y, format!("{}（{}{}）", gz(m.0), if m.2 { "闰" } else { "" }, m.1))).cloned().unwrap_or_default();
                let h_sup: Vec<_> = hs.iter().filter(|h| h.0).collect();
                let h_con: Vec<_> = hs.iter().filter(|h| !h.0).collect();
                let kk = if !h_sup.is_empty() {
                    ls = h_sup.iter().map(|h| h.1).min().unwrap();
                    0
                } else if !h_con.is_empty() {
                    lc = h_con.iter().map(|h| h.1).min().unwrap();
                    1
                } else {
                    let sup = ss || ns >= 2;
                    let con = cs || nc >= 2;
                    if con { 1 } else if sup { 0 } else { 2 }
                };
                sc[kk] += 1;
                let lvs = match kk {
                    0 => ls.to_string(),
                    1 => lc.to_string(),
                    _ => "—".to_string(),
                };
                let mut note = (if kk == 2 && ns == 1 { "纪日一见，合两家" } else if kk == 2 && nc == 1 { "纪日一见，不合两家" } else { "" }).to_string();
                if !hs.is_empty() {
                    let hn: Vec<String> = hs.iter().map(|h| format!("{}{}{}", h.2, h.1, if h.0 { "合两家" } else { "不合两家" })).collect();
                    note = [note, hn.join("；")].into_iter().filter(|x| !x.is_empty()).collect::<Vec<_>>().join("；").replace(',', "，");
                }
                mcsv += &format!("{k},{y},{}{},{},相同,{},{lvs},{note}\n", if m.2 { "闰" } else { "" }, m.1, gz(m.0), ["有证", "同误", "无证"][kk]);
                if kk < 2 {
                    *same_lv.entry((kk, if kk == 0 { ls } else { lc })).or_default() += 1;
                }
            }
        }
        let (mut c1, mut c1f, mut c2, mut cx) = (0, 0, 0, 0);
        let mut lv: BTreeMap<(char, char), usize> = BTreeMap::new();
        let mut c1y = 0usize;
        for ((kk, y, g), es) in ev.iter().filter(|e| &e.0 .0 == k) {
            let (cls, best) = classify_diff(es);
            if cls.starts_with('①') {
                c1 += 1;
                c1f += es.iter().all(|e| e.benji_fixed) as usize;
                *lv.entry(('①', best)).or_default() += 1;
            } else if cls.starts_with('②') {
                c2 += 1;
                *lv.entry(('②', best)).or_default() += 1;
            } else if cls.contains("冲突") {
                cx += 1;
            } else if cls.contains("一见") || cls.contains("一源") {
                c1y += 1;
            }
            let sr = es.iter().filter(|e| e.sup_r).count();
            let lvs = if cls.starts_with('③') { "—".to_string() } else { best.to_string() };
            ev_rows.push(format!("| {kk} | {y} | {g} | {sr} | {} | {cls} | {lvs} |", es.len() - sr));
            diff_cls.insert((*y, g.clone()), (cls.clone(), lvs));
        }
        let c3 = n_diff.saturating_sub(c1 + c2);
        for (y, m) in &diff_months {
            let lab = format!("{}（{}{}）", gz(m.0), if m.2 { "闰" } else { "" }, m.1);
            let (c, l) = diff_cls.get(&(*y, lab.clone())).cloned().unwrap_or(("③无据待核".into(), "—".into()));
            // 注：实物、当时文字、天文志之条（史书本纪之条见 regime_benji_check.csv）
            let note: Vec<String> = ev
                .get(&(k.clone(), *y, lab))
                .map(|es| es.iter().filter(|e| matches!(e.lv, 'A' | 'B') || e.book.ends_with("_zhi")).map(|e| format!("{}{}合{}", e.what, e.lv, if e.sup_r { "推步" } else { "张表" })).collect())
                .unwrap_or_default();
            mcsv += &format!("{k},{y},{}{},{},互异,{c},{l},{}\n", if m.2 { "闰" } else { "" }, m.1, gz(m.0), note.join("；").replace(',', "，"));
        }
        let f = |c: char| ['A', 'B', 'C', 'D', 'E'].iter().map(|l| lv.get(&(c, *l)).copied().unwrap_or(0).to_string()).collect::<Vec<_>>().join("/");
        md.push(format!("| {k} | {} | {} | {} | {} | {n_diff} | {c1} | {c1f} | {c2} | {c3} | {cx} | {c1y} | {} | {} |", sc[0] + sc[1] + sc[2], sc[0], sc[1], sc[2], f('①'), f('②')));
        for i in 0..3 {
            same_tot[i] += sc[i];
        }
        for (i, v) in [n_diff, c1, c1f, c2, c3, cx, c1y].iter().enumerate() {
            tot[i] += v;
        }
        for (k2, v) in lv {
            *tot_lv.entry(k2).or_default() += v;
        }
        println!("{k}: 互异之月 {n_diff}，①{c1}（靠本纪修正 {c1f}）②{c2}③{c3}（冲突 {cx}）");
    }
    let ft = |c: char| ['A', 'B', 'C', 'D', 'E'].iter().map(|l| tot_lv.get(&(c, *l)).copied().unwrap_or(0).to_string()).collect::<Vec<_>>().join("/");
    md.push(format!("| **合计** | {} | **{}** | **{}** | **{}** | **{}** | **{}** | {} | **{}** | **{}** | {} | {} | {} | {} |", same_tot.iter().sum::<usize>(), same_tot[0], same_tot[1], same_tot[2], tot[0], tot[1], tot[2], tot[3], tot[4], tot[5], tot[6], ft('①'), ft('②')));
    md.extend([
        String::new(),
        "相同之月（推步与张表朔日、月名皆同）：**有证**——实物、当时文字（A、B）一条合两家，或书朔一条合两家，或两条以上独立纪日（落在朔日、晦日或书日序者）合两家；**同误**——实物、当时文字一条与两家皆不合（判实物抄误或待核，所据见「注」列），或书朔一条、独立纪日两条以上与两家皆不合（考证表处置「不计」之条已先剔除，不入此栏；余皆未判史书误）；实物、当时文字之条压过史书之条；**无证**——只是两家相同，不算证实（纪日只一见者亦归此）。逐月见 `data/calendar/regime_benji_months.csv`「分类」列。按证据等级：".to_string(),
        String::new(),
        "| 相同之月 | A | B | C | D | E | 计 |".to_string(),
        "|---|---|---|---|---|---|---|".to_string(),
    ]);
    for (kk, nm) in [(0usize, "有证"), (1, "同误")] {
        let c: Vec<usize> = ['A', 'B', 'C', 'D', 'E'].iter().map(|l| same_lv.get(&(kk, *l)).copied().unwrap_or(0)).collect();
        md.push(format!("| {nm} | {} | {} | {} | {} | {} | {} |", c[0], c[1], c[2], c[3], c[4], c.iter().sum::<usize>()));
    }
    md.push(format!("| 无证 | — | — | — | — | — | {} |", same_tot[2]));
    md.extend([
        String::new(),
        "### 有证据可判之月".to_string(),
        String::new(),
        "| 推步之键 | 年 | 推步月首（月） | 合推步条 | 合张表条 | 分类 | 等级 |".to_string(),
        "|---|---|---|---|---|---|---|".to_string(),
    ]);
    md.extend(ev_rows);
    md.extend([
        String::new(),
        "## 推步不合或两家不同之条".to_string(),
        String::new(),
        "| 政权 | 书 | 推步之历 | 年 | 月 | 纪日 | 推步月首 | 合推步 | 张表月首 | 合张表 | 中原月首 | 合中原 | 原文 |".to_string(),
        "|---|---|---|---|---|---|---|---|---|---|---|---|---|".to_string(),
    ]);
    md.extend(diff);
    md.push(String::new());
    std::fs::write(root.join("data/calendar/regime_benji_months.csv"), &mcsv).unwrap();
    std::fs::write(root.join("doc/calendar/regime-benji-checks.md"), md.join("\n")).unwrap();
}
