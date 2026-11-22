//! 分政权推步以各国史书逐条检验：`data/calendar/本纪纪日/*.csv`（各史本纪纪日，带政权列）中属分政权之条，
//! 以该政权推步（`push_calendar::regime_year_months`）与中原一线今人历表（张表，作对照：看本国史书支持的是本国之历还是中原之历）
//! 两家检验：纪日干支是否落在所记之月（书「朔」者须为朔日，书「晦」者须为晦日）。
//! 输出 data/calendar/regime_benji_check.csv（逐条）、doc/calendar/regime-benji-checks.md（汇总与两家不同之条）。
//! 用法（仓库根）：cargo run --release --manifest-path rust/Cargo.toml -p lunar-core --no-default-features --features real-f64 --example regime_benji_eval
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::historical::ShuoriTable;
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
    /// 合推步之条中，纯推步不合、靠史载修正才合，且当年修正有「本纪」类者
    benji_fix: usize,
    /// 同上，当年修正只有「他书」类者
    other_fix: usize,
}

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    let zhang = ShuoriTable::from_csv(&read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let mut files: Vec<_> = std::fs::read_dir(root.join("data/calendar/本纪纪日"))
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "csv") && !p.to_string_lossy().contains("閏月"))
        .collect();
    files.sort();
    let mut stats: BTreeMap<(String, String, String), Stat> = BTreeMap::new();
    let mut per = String::from("书,政权,推步之历,年,月,干支,位置,推步月首,推步合,张表月首,张表合,原文\n");
    let mut diff: Vec<String> = Vec::new();
    for path in files {
        let book = path.file_stem().unwrap().to_string_lossy().to_string();
        for r in csv_rows(&std::fs::read_to_string(&path).unwrap()) {
            let Some(key) = regime_key(&r["regime"]) else { continue };
            let (Ok(y), Ok(m)) = (r["year"].parse::<i32>(), r["month"].parse::<u8>()) else { continue };
            let leap = r["leap"] == "1";
            let Some(gi) = GAN_ZHI_60.iter().position(|g| *g == r["ganzhi"]) else { continue };
            let reg = |yy: i32| regime_year_months(key, yy).map(|x| x.1);
            let Some(li) = regime_year_months(key, y).map(|x| x.0) else { continue };
            let tab = |yy: i32| zhang.year_months(yy).map(|t| t.to_vec());
            let pos = r.get("position").cloned().unwrap_or_default();
            let judge = |sp: Option<(i64, i64)>| -> (String, bool) {
                let Some((a, b)) = sp else { return ("—".into(), false) };
                let d = a + (gi as i64 - (a + 49)).rem_euclid(60);
                let ok = d < b && match pos.as_str() { "朔" => d == a, "晦" => d == b - 1, _ => true };
                (gz(a).to_string(), ok)
            };
            let sp = span(&reg, y, m, leap);
            let raw = |yy: i32| regime_year_months_raw(key, yy).map(|x| x.1);
            let (gr, okr) = judge(sp);
            // 纯推步（不应用史载修正）下不合、修正后才合
            let fixed = okr && !judge(span(&raw, y, m, leap)).1;
            let (gt, okt) = judge(span(&tab, y, m, leap));
            let text = r.get("text").cloned().unwrap_or_default().replace(',', "，");
            let mn = format!("{}{}", if leap { "闰" } else { "" }, m);
            per += &format!("{book},{},{li},{y},{mn},{},{pos},{gr},{},{gt},{},{text}\n", r["regime"], r["ganzhi"], okr as u8, okt as u8);
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
            if pos == "朔" {
                s.shuo += 1;
                s.shuo_r += okr as usize;
                s.shuo_t += okt as usize;
            }
            if okr != okt || !okr {
                diff.push(format!(
                    "| {} | {book} | {li} | {y} | {mn} | {}{pos} | {gr} | {} | {gt} | {} | {} |",
                    r["regime"], r["ganzhi"], if okr { "✓" } else { "✗" }, if okt { "✓" } else { "✗" }, text.chars().take(40).collect::<String>()
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
        "纪日取自各史本纪（`data/calendar/本纪纪日/`，按政权列归属）。「推步」= 该政权所行之历按术推步（分政权推步）；「张表」= 中原一线的今人历表，作对照——本国史书若合本国推步而不合张表，即本国另行一历之证。纪日合 = 干支落在所记之月（书朔者须为朔日、书晦者须为晦日）。".to_string(),
        String::new(),
        "## 汇总".to_string(),
        String::new(),
        "「合推步」含史载修正（例外表历名「政权：历名」）；其中纯推步不合、靠修正才合者另列：「从本纪修正」= 当年修正有只据本纪之条，「他书修正」= 当年修正皆有本纪以外的记录；合推步减去二者即纯按术推步之合数。".to_string(),
        String::new(),
        "| 推步之键 | 史书政权 | 书 | 纪日 | 合推步 | 合张表 | 只合推步 | 只合张表 | 书朔 | 朔合推步 | 朔合张表 | 从本纪修正 | 他书修正 |".to_string(),
        "|---|---|---|---|---|---|---|---|---|---|---|---|---|".to_string(),
    ];
    for ((k, rg, b), s) in &stats {
        md.push(format!(
            "| {k} | {rg} | {b} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            s.n, s.r, s.t, s.only_r, s.only_t, s.shuo, s.shuo_r, s.shuo_t, s.benji_fix, s.other_fix
        ));
        println!(
            "{k} {rg} {b}: {} 推步 {} 张表 {} 只推 {} 只张 {} | 朔 {} {} {} | 修正 本纪 {} 他书 {}",
            s.n, s.r, s.t, s.only_r, s.only_t, s.shuo, s.shuo_r, s.shuo_t, s.benji_fix, s.other_fix
        );
    }
    md.extend([
        String::new(),
        "## 推步不合或两家不同之条".to_string(),
        String::new(),
        "| 政权 | 书 | 推步之历 | 年 | 月 | 纪日 | 推步月首 | 合推步 | 张表月首 | 合张表 | 原文 |".to_string(),
        "|---|---|---|---|---|---|---|---|---|---|---|".to_string(),
    ]);
    md.extend(diff);
    md.push(String::new());
    std::fs::write(root.join("doc/calendar/regime-benji-checks.md"), md.join("\n")).unwrap();
}
