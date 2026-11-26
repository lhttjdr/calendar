//! 斯0612《宋太平兴国三年应天具注历日》历序「六十相屬宫宿法」所列 919—978 年历年閏月，与张表、中原一线（缺省逐历取法）閏位逐年比较。
//! 历序是宋司天台官本历书的抄本，所列閏月是当时人记前六十年之閏（第 1 级旁证）。
//! 输入 data/calendar/s0612_runyue.csv，输出 doc/calendar/s0612-runyue-checks.md。
//! 取用閏月：原卷列（图版所见）有字者优先，空则用邓氏录文；原卷与录文不同之岁另列「录文异读」。
//! 用法（仓库根）：cargo run --release -p lunar-core --no-default-features --features real-f64 --example s0612_runyue_check
use lunar_core::calendar::historical::ShuoriTable;
use lunar_core::calendar::lifa::{zhongyuan_year_months, ZhongyuanPreference};

const NUM: [&str; 13] = ["", "正", "二", "三", "四", "五", "六", "七", "八", "九", "十", "十一", "十二"];

fn leap_of(months: &[(i64, u8, bool)]) -> Option<u8> {
    months.iter().find(|m| m.2).map(|m| m.1)
}

fn show(l: Option<u8>) -> String {
    l.map(|m| format!("閏{}", NUM[m as usize])).unwrap_or_else(|| "无".into())
}

fn main() {
    let read = |p: &str| std::fs::read_to_string(p).unwrap_or_else(|e| panic!("读 {p}: {e}"));
    let zhang = ShuoriTable::from_csv(&read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let src = read("data/calendar/s0612_runyue.csv");
    let mut md = vec![
        "# 斯0612 历序閏月与张表、中原一线（自动生成）".to_string(),
        String::new(),
        "> 由 `cargo run --example s0612_runyue_check` 生成，勿手改。底本与结论见 [dunhuang-calendar.md](dunhuang-calendar.md)「附录一未列之件」。".into(),
        String::new(),
        "录文：邓文宽《敦煌天文历法文献辑校》下编第三三件（书页 523—527），岁 1 = 978 年逆数至岁 60 = 919 年。原卷：《英藏敦煌文献》第 2 卷书页 73 图版，只录读图时逐行确认之岁。「历序閏」取原卷，原卷未录者取录文（「依据」列注明）。".into(),
        String::new(),
        "| 岁 | 公历年 | 干支 | 年号 | 历序閏 | 依据 | 张表閏 | 合张表 | 中原一线閏 | 来源 | 合中原一线 |".into(),
        "|---|---|---|---|---|---|---|---|---|---|---|".into(),
    ];
    let (mut n, mut ok_z, mut ok_c, mut n_yuan) = (0, 0, 0, 0);
    let mut diffs = Vec::new();
    let mut variants = Vec::new();
    for line in src.lines().filter(|l| !l.starts_with('#')).skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        let y: i32 = f[3].parse().unwrap();
        let lu: Option<u8> = f[4].parse().ok();
        let yuan_raw = f.get(6).copied().unwrap_or("").trim();
        let (book, basis) = if yuan_raw.is_empty() {
            (lu, "录文")
        } else {
            n_yuan += 1;
            (if yuan_raw == "无" { None } else { Some(yuan_raw.parse::<u8>().unwrap_or_else(|_| panic!("{y} 原卷列：{yuan_raw}"))) }, "原卷")
        };
        if basis == "原卷" && book != lu {
            variants.push(format!("{y}（{}）原卷{}、录文{}", f[2], show(book), show(lu)));
        }
        let z = zhang.year_months(y).and_then(|m| leap_of(m));
        let (c, label) = zhongyuan_year_months(Some(&zhang), y, ZhongyuanPreference::Evidence)
            .map(|(m, s)| (leap_of(&m), s.label()))
            .unwrap_or((None, "现算".into()));
        n += 1;
        ok_z += (book == z) as i32;
        ok_c += (book == c) as i32;
        if book != z || book != c {
            diffs.push(format!("{y}（{}）历序{}、张表{}、中原一线{}", f[2], show(book), show(z), show(c)));
        }
        let mk = |b: bool| if b { "✓" } else { "✗" };
        md.push(format!("| {} | {y} | {} | {} | {} | {basis} | {} | {} | {} | {label} | {} |", f[0], f[1], f[2], show(book), show(z), mk(book == z), show(c), mk(book == c)));
    }
    md.push(String::new());
    md.push(format!("共 {n} 年（原卷 {n_yuan} 年、录文 {} 年）：合张表 {ok_z}，合中原一线 {ok_c}。", n - n_yuan));
    if !variants.is_empty() {
        md.push(String::new());
        md.push(format!("录文异读：{}。", variants.join("；")));
    }
    if !diffs.is_empty() {
        md.push(String::new());
        md.push(format!("不合：{}。", diffs.join("；")));
    }
    std::fs::write("doc/calendar/s0612-runyue-checks.md", md.join("\n") + "\n").unwrap();
    println!("{n} 年（原卷 {n_yuan}）：合张表 {ok_z}，合中原一线 {ok_c}；{diffs:?}；录文异读 {variants:?}");
}
