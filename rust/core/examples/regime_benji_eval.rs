//! 分政权推步以各国史书逐条检验：`data/calendar/本纪纪日/*.csv`（各史本纪纪日，带政权列）中属分政权之条，
//! 以该政权推步（`push_calendar::regime_year_months`）与中原一线今人历表（张表，作对照：看本国史书支持的是本国之历还是中原之历）
//! 两家检验：纪日干支是否落在所记之月（书「朔」者须为朔日，书「晦」者须为晦日）。
//! 输出 data/calendar/regime_benji_check.csv（逐条）、doc/calendar/regime-benji-checks.md（汇总与两家不同之条）。
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example regime_benji_eval
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::historical::ShuoriTable;
use lunar_core::calendar::lifa::{zhongyuan_year_months, ZhongyuanPreference};
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
    /// 合中原一线（本项目缺省取用之中原历日：实物、推步，否则张表）
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
    for path in files {
        let book = path.file_stem().unwrap().to_string_lossy().to_string();
        // 上一条所记之年、月（不闰）：本纪「閏月」不书月数（月列只记「闰」），只保证闰月在所记最后一月之后
        let mut last: Option<(i32, u8)> = None;
        for r in csv_rows(&std::fs::read_to_string(&path).unwrap()) {
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
            let zy = |yy: i32| zhongyuan_year_months(Some(&zhang), yy, ZhongyuanPreference::Evidence).map(|x| x.0).or_else(|| tab(yy));
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
            let m_zy = if m == 0 { leap_of(&zy) } else { m_reg };
            let (gzy, okz) = judge(span(&zy, y, m_zy, leap));
            let text = r.get("text").cloned().unwrap_or_default().replace(',', "，");
            let mn0 = if m == 0 { format!("闰（推步闰{m_reg}，张表闰{m_tab}）") } else { format!("{}{}", if leap { "闰" } else { "" }, m) };
            let mn = if fix.is_some() { format!("{}→{mn0}（考证改月）", r["month"]) } else { mn0 };
            per += &format!("{book},{},{li},{y},{mn},{},{pos},{gr},{},{gt},{},{text},{gzy},{}\n", r["regime"], r["ganzhi"], okr as u8, okt as u8, okz as u8);
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
    std::fs::write(root.join("data/calendar/regime_benji_check.csv"), per).unwrap();
    let mut md = vec![
        "# 分政权推步之史书逐条检验（自动生成）".to_string(),
        String::new(),
        "> 由 `rust/core/examples/regime_benji_eval.rs` 生成，勿手改；逐条见 `data/calendar/regime_benji_check.csv`。方法见 [historical-calendar.md](historical-calendar.md)「分政权推步的史书检验」。".to_string(),
        String::new(),
        "纪日取自各史本纪（`data/calendar/本纪纪日/`，按政权列归属）。「推步」= 该政权所行之历按术推步（分政权推步）；「张表」= 中原一线的今人历表，作对照——本国史书若合本国推步而不合张表，即本国另行一历之证。纪日合 = 干支落在所记之月（书朔者须为朔日、书晦者须为晦日，书日序者干支须为该日）。「中原一线」= 本项目中原历日缺省取用之来源（实物、推步、古人长历；南明期间即清新法推步与时宪书实物），也作对照。".to_string(),
        String::new(),
        "## 汇总".to_string(),
        String::new(),
        "「合推步」含史载修正（例外表历名「政权：历名」）；其中纯推步不合、靠修正才合者另列：「从本纪修正」= 当年修正有只据本纪之条，「他书修正」= 当年修正皆有本纪以外的记录；合推步减去二者即纯按术推步之合数。「考讹不计」= 考证表处置为「不计」之条。".to_string(),
        String::new(),
        "| 推步之键 | 史书政权 | 书 | 纪日 | 合推步 | 合张表 | 只合推步 | 只合张表 | 书朔 | 朔合推步 | 朔合张表 | 考讹不计 | 从本纪修正 | 他书修正 | 合中原一线 | 只合推步（对中原） | 只合中原 |".to_string(),
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
    md.extend([
        String::new(),
        "## 推步不合或两家不同之条".to_string(),
        String::new(),
        "| 政权 | 书 | 推步之历 | 年 | 月 | 纪日 | 推步月首 | 合推步 | 张表月首 | 合张表 | 中原月首 | 合中原 | 原文 |".to_string(),
        "|---|---|---|---|---|---|---|---|---|---|---|---|---|".to_string(),
    ]);
    md.extend(diff);
    md.push(String::new());
    std::fs::write(root.join("doc/calendar/regime-benji-checks.md"), md.join("\n")).unwrap();
}
