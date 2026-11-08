//! 按农历年列出逐日宜忌：月表（民用三十七事）与加铺注后的结果，供与时宪书逐日对照。
//! 用法（仓库根）：cargo run -p lunar-core --no-default-features --features real-f64 --example puzhu_year -- 1765 [exclusive] [csv]
//! csv：输出 月,日,干支,建除,月表宜,宜,忌,铺注（民用三十七事，空格分隔；諸事不宜作「諸事不宜」，無宜作「—」），供 scripts/folk/shixianshu_check.py 比对。
//! 参数为公历年：列出该年正月初一至次年正月初一前一日（农历以实历朔日表为先）。
use lunar_core::astronomy::ephemeris::{load_all, load_earth_vsop87_from_repo, Elpmpp02Correction};
use lunar_core::calendar::gregorian::Gregorian;
use lunar_core::calendar::historical::HistoricalCalendar;
use lunar_core::folk::puzhu::{apply, minyong, puzhu_days, wulu, QiWangWangCount, WuLu};
use lunar_core::folk::yiji::yuebiao_day;
use lunar_core::math::real::RealOps;

const GZ: [&str; 60] = [
    "甲子", "乙丑", "丙寅", "丁卯", "戊辰", "己巳", "庚午", "辛未", "壬申", "癸酉", "甲戌", "乙亥", "丙子", "丁丑", "戊寅",
    "己卯", "庚辰", "辛巳", "壬午", "癸未", "甲申", "乙酉", "丙戌", "丁亥", "戊子", "己丑", "庚寅", "辛卯", "壬辰", "癸巳",
    "甲午", "乙未", "丙申", "丁酉", "戊戌", "己亥", "庚子", "辛丑", "壬寅", "癸卯", "甲辰", "乙巳", "丙午", "丁未", "戊申",
    "己酉", "庚戌", "辛亥", "壬子", "癸丑", "甲寅", "乙卯", "丙辰", "丁巳", "戊午", "己未", "庚申", "辛酉", "壬戌", "癸亥",
];
const MON: [&str; 12] = ["正", "二", "三", "四", "五", "六", "七", "八", "九", "十", "十一", "十二"];
const DAY: [&str; 30] = [
    "初一", "初二", "初三", "初四", "初五", "初六", "初七", "初八", "初九", "初十", "十一", "十二", "十三", "十四", "十五",
    "十六", "十七", "十八", "十九", "二十", "廿一", "廿二", "廿三", "廿四", "廿五", "廿六", "廿七", "廿八", "廿九", "三十",
];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let y: i32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1765);
    let count = if args.iter().any(|a| a == "exclusive") { QiWangWangCount::Exclusive } else { QiWangWangCount::Inclusive };
    let vsop = load_earth_vsop87_from_repo().unwrap();
    let elp = load_all(&lunar_core::repo::default_loader(), lunar_core::repo::paths::ELPMPP02, Elpmpp02Correction::DEFAULT).unwrap();
    let hist = HistoricalCalendar::from_csv(&std::fs::read_to_string("vendor/calendar/chinese_calendar/实历朔日表.csv").unwrap()).unwrap();
    let j0 = (Gregorian::to_julian_day(y, 1, 1).as_f64() + 0.5).floor() as i64;
    let j1 = (Gregorian::to_julian_day(y + 1, 3, 1).as_f64() + 0.5).floor() as i64;
    let days = puzhu_days(&vsop, &elp, Some(&hist), j0, j1, count);
    let csv = args.iter().any(|a| a == "csv");
    if csv {
        println!("月,日,干支,月建,建除,吉神,凶神,月表宜,宜,忌,铺注");
    }
    let join = |v: &[&str]| if v.is_empty() { "—".to_string() } else { v.join(" ") };
    for d in days.iter().filter(|d| d.lunar.year == y) {
        let base = yuebiao_day(d.month_ord, d.input.day_gz);
        let after = apply(base, &d.input, d.shuo_gz);
        let (yi0, _) = minyong(base);
        let (yi1, ji1) = minyong(&after);
        let names: Vec<&String> = after.xiong_shen.iter().skip(base.xiong_shen.len()).collect();
        let wl = match wulu(d.input.day_gz, d.year_gz, d.month_gz, d.sun_palace_zhi, base.ji_shen.iter().any(|s| s == "天德" || s == "月德")) {
            Some(WuLu::Yes) => " 無祿".to_string(),
            Some(WuLu::Exempt(r)) => format!(" 無祿（{r}）"),
            None => String::new(),
        };
        let (gy, gm, gd) = Gregorian::from_julian_day(lunar_core::math::real::real(d.jdn as f64));
        let fmt = |v: &Option<Vec<&str>>| match v {
            None => "諸事不宜".to_string(),
            Some(v) => join(v),
        };
        let changed = if yi0 != yi1 { "*" } else { " " };
        if csv {
            println!(
                "{}{},{},{},{},{},{},{},{},{},{},{}",
                if d.lunar.is_leap_month { "閏" } else { "" },
                MON[d.lunar.month as usize - 1],
                DAY[d.lunar.day as usize - 1],
                GZ[d.input.day_gz as usize],
                d.month_ord + 1,
                base.jian_chu,
                base.ji_shen.join(" "),
                base.xiong_shen.join(" "),
                fmt(&yi0),
                fmt(&yi1),
                ji1.join(" "),
                names.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" ")
            );
            continue;
        }
        println!(
            "{}{}{}月{} {}{} {gy}-{gm:02}-{gd:02} | 月表宜 {} | 宜 {} | 忌 {} | {}{}",
            changed,
            if d.lunar.is_leap_month { "閏" } else { "" },
            MON[d.lunar.month as usize - 1],
            DAY[d.lunar.day as usize - 1],
            GZ[d.input.day_gz as usize],
            base.jian_chu,
            fmt(&yi0),
            fmt(&yi1),
            join(&ji1),
            names.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" "),
            wl
        );
    }
}
