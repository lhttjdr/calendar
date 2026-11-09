//! 敦煌历日换算逐条检验：比较表（data/calendar/敦煌历日_比较表.csv，邓文宽 1996 附录一）敦煌曆一栏每一格，
//! 经第 4 层 `lifa::resolve_dunhuang_month` / `resolve_dunhuang_day`（退回主表）换算，看能否重现原书朔干支与月大小；
//! 并列各岁敦煌纪年（第 3 层 `era_names::dunhuang_year_name`）。
//! 输出 data/calendar/dunhuang_shuori_check.csv、data/calendar/dunhuang_years.csv，
//! 由 scripts/calendar_data/dunhuang_checks_to_md.py 汇入 doc/calendar/dunhuang-calendar.md。
//! 用法（rust/ 目录）：cargo run --release -p lunar-core --example dunhuang_check
use lunar_core::calendar::dunhuang::{parse_comparison_table, DunhuangCalendar, ShuoBasis};
use lunar_core::calendar::era_names::{dunhuang_year_name, EraNames};
use lunar_core::calendar::gan_zhi::GAN_ZHI_60;
use lunar_core::calendar::gregorian::{civil_from_jdn, civil_to_jdn};
use lunar_core::calendar::historical::HistoricalCalendar;
use lunar_core::calendar::lifa::{dunhuang_months, resolve_dunhuang_day, resolve_dunhuang_month, DunhuangFallback};

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    let main = HistoricalCalendar::from_csv(&read("vendor/calendar/chinese_calendar/实历朔日表.csv")).unwrap();
    let dh = DunhuangCalendar::embedded();
    let era = EraNames::from_csv(&read("data/calendar/汇总/era_names_merged.csv")).unwrap();
    let rows = parse_comparison_table(&read("data/calendar/敦煌历日_比较表.csv")).unwrap();
    let gz = |j: i64| GAN_ZHI_60[(j + 49).rem_euclid(60) as usize];
    let label = |m: u8, leap: bool| {
        const N: [&str; 12] = ["正", "二", "三", "四", "五", "六", "七", "八", "九", "十", "十一", "十二"];
        format!("{}{}月", if leap { "閏" } else { "" }, N[m as usize - 1])
    };
    let fb = DunhuangFallback::MainTable;
    let mut out = String::from("文书,原书定名,中原月序,敦煌岁,敦煌月,原书朔,朔依据,原书大小,换算月首,换算朔,朔合,换算月长,大小合,只认有据亦合,书页\n");
    let (mut n, mut ok_shuo, mut ok_size, mut n_size, mut ok_strict) = (0, 0, 0, 0, 0);
    for r in &rows {
        n += 1;
        let s = resolve_dunhuang_month(&dh, Some(&main), r.lunar_year, r.month, r.is_leap, fb);
        let strict = resolve_dunhuang_month(&dh, Some(&main), r.lunar_year, r.month, r.is_leap, DunhuangFallback::EvidenceOnly)
            .is_some_and(|s| gz(s.start_jdn) == r.shuo_ganzhi);
        let (date, shuo, len) = match &s {
            Some(s) => {
                let (y, m, d) = civil_from_jdn(s.start_jdn);
                // 月长：从月首逐日换算到次月初一
                let len = (29..=31).find(|&k| resolve_dunhuang_day(&dh, Some(&main), s.start_jdn + k, fb).is_some_and(|d| d.day == 1));
                (format!("{y}-{m:02}-{d:02}"), gz(s.start_jdn).to_string(), len)
            }
            None => (String::new(), String::new(), None),
        };
        let shuo_ok = shuo == r.shuo_ganzhi;
        let size_ok = match (r.days, len) {
            (Some(a), Some(b)) => {
                n_size += 1;
                Some(a as i64 == b)
            }
            _ => None,
        };
        ok_shuo += shuo_ok as u32;
        ok_size += (size_ok == Some(true)) as u32;
        ok_strict += strict as u32;
        let yes = |b: bool| if b { "合" } else { "不合" };
        out += &format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            r.document,
            r.title,
            r.central_label,
            r.lunar_year,
            label(r.month, r.is_leap),
            r.shuo_ganzhi,
            if r.shuo_basis == ShuoBasis::Original { "原件" } else { "推算" },
            r.days.map_or("?".into(), |d| if d == 30 { "大".to_string() } else { "小".to_string() }),
            date,
            shuo,
            yes(shuo_ok),
            len.map_or(String::new(), |l| l.to_string()),
            size_ok.map_or("", |b| yes(b)),
            yes(strict),
            r.page
        );
    }
    std::fs::write(root.join("data/calendar/dunhuang_shuori_check.csv"), out).unwrap();
    println!("比较表敦煌曆有朔之格 {n}：朔合 {ok_shuo}，大小可验 {n_size} 合 {ok_size}，只认有据之月亦合 {ok_strict}");
    // 退回主表之月中，月首因接有据之月而挪动者
    let all = dunhuang_months(&dh, Some(&main), civil_to_jdn(786, 3, 1), civil_to_jdn(1036, 12, 1), fb);
    let fallback: Vec<_> = all.iter().filter(|s| s.basis == lunar_core::calendar::lifa::DunhuangMonthBasis::MainTable).collect();
    let moved: Vec<String> = fallback
        .iter()
        .filter_map(|s| {
            let (y, m, d) = main.new_moon_date(s.lunar_year, s.month, s.is_leap)?;
            let diff = s.start_jdn - civil_to_jdn(y, m, d);
            (diff != 0).then(|| format!("{}{}({:+})", s.lunar_year, label(s.month, s.is_leap), diff))
        })
        .collect();
    println!("786—1036 年敦煌历 {} 月，其中退回主表 {} 月；月首因接有据之月而挪动 {} 月：{}", all.len(), fallback.len(), moved.len(), moved.join(" "));

    // 各岁：纪年、有据之月、闰月（敦煌 vs 主表）
    let mut years = String::from("岁,干支,时期,通行纪年,出土历日定名,原件之月,推算之月,敦煌闰月,主表闰月\n");
    let mut ys: Vec<i32> = dh.months().iter().map(|m| m.lunar_year).collect();
    ys.dedup();
    for y in ys {
        let name = dunhuang_year_name(y, &era, Some(&dh)).unwrap();
        let ms: Vec<_> = dh.months().iter().filter(|m| m.lunar_year == y).collect();
        let orig = ms.iter().filter(|m| m.shuo_basis == ShuoBasis::Original).count();
        let span = dunhuang_months(&dh, Some(&main), civil_to_jdn(y, 2, 15), civil_to_jdn(y + 1, 1, 15), fb);
        let dh_leap: Vec<String> = span.iter().filter(|s| s.lunar_year == y && s.is_leap).map(|s| label(s.month, true)).collect();
        let main_leap: Vec<String> = main
            .months_in_jdn_range(civil_to_jdn(y, 1, 1), civil_to_jdn(y + 1, 3, 1))
            .into_iter()
            .filter(|&(_, ly, _, leap)| ly == y && leap)
            .map(|(_, _, m, _)| label(m, true))
            .collect();
        years += &format!(
            "{y},{},{},{},{},{orig},{},{},{}\n",
            name.ganzhi,
            name.period,
            name.label,
            name.documents.join("；"),
            ms.len() - orig,
            dh_leap.join("；"),
            main_leap.join("；")
        );
    }
    std::fs::write(root.join("data/calendar/dunhuang_years.csv"), years).unwrap();
}
