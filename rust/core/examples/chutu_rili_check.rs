//! 吐鲁番、黑城出土唐宋历日残件所示月朔（data/calendar/chutu_rili_shuo.csv，录自邓文宽《敦煌吐鲁番天文历法研究》2002）
//! 与张表、中原一线（缺省逐历取法、一律推步）逐月比较，输出 doc/calendar/chutu-rili-checks.md。
//! 这些残件的年代是邓氏以今人表（张表节气日、陈垣朔闰表）选定的，凡定年用到的月朔与今人表相合是选年的结果（表中标「定年」），
//! 不作独立证据；有意义的是定年条件以外之月，及推步与张表不同之月。
//! 用法（仓库根）：cargo run --release -p lunar-core --no-default-features --features real-f64 --example chutu_rili_check
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::historical::ShuoriTable;
use lunar_core::calendar::lifa::{zhongyuan_year_months, ZhongyuanPreference};

const NUM: [&str; 13] = ["", "正", "二", "三", "四", "五", "六", "七", "八", "九", "十", "十一", "十二"];

fn gz(jdn: i64) -> &'static str {
    GAN_ZHI_60[(jdn + 49).rem_euclid(60) as usize]
}

fn main() {
    let read = |p: &str| std::fs::read_to_string(p).unwrap_or_else(|e| panic!("读 {p}: {e}"));
    let zhang = ShuoriTable::from_csv(&read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let src = read("data/calendar/chutu_rili_shuo.csv");
    let mut md = vec![
        "# 出土唐宋历日残件月朔与张表、中原一线（自动生成）".to_string(),
        String::new(),
        "> 由 `cargo run --example chutu_rili_check` 生成，勿手改。录文 `data/calendar/chutu_rili_shuo.csv`，说明见 [historical-calendar.md](historical-calendar.md)「出土唐宋历日残件」。".into(),
        String::new(),
        "「定年」= 该件年代由此月朔或节气与今人表相合而选定，相合不作独立证据；「推得」= 残件无该月干支，由注历规则推出。".into(),
        String::new(),
        "| 件 | 年 | 月 | 残件朔 | 依据 | 张表 | 中原一线（缺省） | 来源 | 一律推步 | 定年 | 原文 | 书页 |".into(),
        "|---|---|---|---|---|---|---|---|---|---|---|---|".into(),
    ];
    let (mut n_ind, mut ok_z, mut ok_c, mut ok_r) = (0, 0, 0, 0);
    for line in src.lines().filter(|l| !l.starts_with('#')).skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        let (y, m, leap): (i32, u8, bool) = (f[1].parse().unwrap(), f[2].parse().unwrap(), f[3] == "1");
        let find = |ms: &[(i64, u8, bool)]| ms.iter().find(|e| e.1 == m && e.2 == leap).map(|e| gz(e.0)).unwrap_or("（无）");
        let z = zhang.year_months(y).map(|ms| find(ms)).unwrap_or("（无）");
        let (c, src_label) = zhongyuan_year_months(Some(&zhang), y, ZhongyuanPreference::Evidence)
            .map(|(ms, s)| (find(&ms), s.label()))
            .unwrap_or(("（无）", "".into()));
        let r = zhongyuan_year_months(Some(&zhang), y, ZhongyuanPreference::Reckoning).map(|(ms, _)| find(&ms)).unwrap_or("（无）");
        let book = f[4];
        let dingnian = f[7].contains("定年") && (f[7].contains(&format!("{}月", NUM[m as usize])) || f[7].contains(&format!("{}、", NUM[m as usize])));
        let mk = |x: &str| if x == book { format!("{x} ✓") } else { format!("{x} ✗") };
        if f[5] == "原印" && !dingnian {
            n_ind += 1;
            ok_z += (z == book) as i32;
            ok_c += (c == book) as i32;
            ok_r += (r == book) as i32;
        }
        md.push(format!(
            "| {} | {y} | {}{}月 | {book} | {} | {} | {} | {src_label} | {} | {} | {} | {} |",
            f[0], if leap { "閏" } else { "" }, NUM[m as usize], f[5], mk(z), mk(c), mk(r), if dingnian { "定年" } else { "" }, f[6], f[8]
        ));
    }
    md.push(String::new());
    md.push(format!("原印且非定年所用之月 {n_ind} 个：合张表 {ok_z}，合中原一线（缺省）{ok_c}，合一律推步 {ok_r}。"));
    std::fs::write("doc/calendar/chutu-rili-checks.md", md.join("\n") + "\n").unwrap();
    println!("{}", md.join("\n"));
}
