//! 古籍命例逐条检验（结果表见 doc/folk/ziping.md「命例检验」）。
//!
//! 历日取中原一线（`calendar::lifa`：实物历日、按术推步、张培瑜表），农历日换成儒略日数后按东八区钟点排盘
//! （古人用当地太阳时，开封、杭州、北京等地与东八区相差不过半小时，所举时辰之中点不致跨时辰）。

use super::pan::{pai_pan, BirthInput};
use super::{gz_name, SiZhu, ZipingOptions};
use crate::astronomy::ephemeris::{load_earth_vsop87_from_repo, Vsop87};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::calendar::gregorian::{civil_from_jdn, civil_to_jdn};
use crate::calendar::historical::ShuoriTable;
use crate::calendar::lifa::{zhongyuan_months_in_jdn_range, ZhongyuanPreference};
use crate::math::real::{real, Real, RealOps};

fn zhang() -> Option<ShuoriTable> {
    let text = std::fs::read_to_string(crate::repo::repo_root().join("vendor/calendar/chinese_calendar/实历朔日表.csv")).ok()?;
    ShuoriTable::from_csv(&text).ok()
}

/// 中原一线农历（年, 月, 闰, 日）→ 儒略日数
fn lunar_jdn(t: Option<&ShuoriTable>, ly: i32, m: u8, leap: bool, d: i64) -> Option<i64> {
    let from = civil_to_jdn(ly, 1, 1) - 70;
    let to = civil_to_jdn(ly + 1, 3, 1);
    zhongyuan_months_in_jdn_range(t, from, to, ZhongyuanPreference::default())
        .into_iter()
        .find(|x| x.1 == ly && x.2 == m && x.3 == leap)
        .map(|x| x.0 + d - 1)
}

/// 儒略日数 → 中原一线农历（年, 月, 闰, 日）
fn jdn_lunar(t: Option<&ShuoriTable>, jdn: i64) -> Option<(i32, u8, bool, i64)> {
    zhongyuan_months_in_jdn_range(t, jdn - 70, jdn, ZhongyuanPreference::default())
        .into_iter()
        .filter(|x| x.0 <= jdn)
        .last()
        .map(|x| (x.1, x.2, x.3, jdn - x.0 + 1))
}

/// 儒略日数某日东八区钟点 `hour` 时之 JD(TT)
fn tt_at(jdn: i64, hour: f64) -> Real {
    let ut = real(jdn as f64 - 0.5 + (hour - 8.0) / 24.0);
    TimePoint::new(TimeScale::UTC, ut).to_scale(TimeScale::TT).jd
}

fn pillars(v: &Vsop87, jdn: i64, hour: f64) -> SiZhu {
    pai_pan(v, &BirthInput { jd_tt: tt_at(jdn, hour), clock_utc_offset_hours: real(8.0), male: true }, 1, &ZipingOptions::default()).sizhu
}

/// 公历年 `y` 前后（立春至次年立春）内年、月、日三柱合者之儒略日数
fn find_days(v: &Vsop87, y: i32, sz: &SiZhu) -> Vec<i64> {
    (civil_to_jdn(y, 1, 20)..civil_to_jdn(y + 1, 2, 20))
        .filter(|&j| {
            let p = pillars(v, j, 12.0);
            p.year == sz.year && p.month == sz.month && p.day == sz.day
        })
        .collect()
}

fn civil(j: i64) -> String {
    let (y, m, d) = civil_from_jdn(j);
    format!("{y}-{m:02}-{d:02}")
}

fn lunar_str(t: Option<&ShuoriTable>, j: i64) -> String {
    match jdn_lunar(t, j) {
        Some((y, m, l, d)) => format!("{y}年{}{m}月{d}日", if l { "閏" } else { "" }),
        None => "?".into(),
    }
}

/// 史载生日之命例：(人, 书中八字, 农历年, 月, 日, 钟点, 史载出处)
const DATED: &[(&str, &str, i32, u8, i64, f64)] = &[
    ("蘇軾", "丙子辛丑癸亥乙卯", 1036, 12, 19, 6.0),
    ("朱熹", "庚戌丙戌甲寅庚午", 1130, 9, 15, 12.0),
    ("明太祖", "戊辰壬戌丁丑丁未", 1328, 9, 18, 14.0),
    ("王守仁", "壬辰辛亥癸亥癸亥", 1472, 9, 30, 22.0),
];

#[test]
fn dated_cases() {
    let v = load_earth_vsop87_from_repo().unwrap();
    let t = zhang();
    for &(who, bz, ly, m, d, h) in DATED {
        let j = lunar_jdn(t.as_ref(), ly, m, false, d).expect(who);
        let p = pillars(&v, j, h);
        println!("{who}\t{}\t{}\t{}", civil(j), p.name(), SiZhu::parse(bz).unwrap().name());
        assert_eq!(p, SiZhu::parse(bz).unwrap(), "{who}");
    }
}

/// 《三命通會》所记史载日期与星家八字不合之例：吕洞宾（道书天宝十四年四月十四日巳时）、寇准（宋史七月十四日生）
#[test]
fn san_ming_kao_zheng() {
    let v = load_earth_vsop87_from_repo().unwrap();
    let t = zhang();
    let lv = lunar_jdn(t.as_ref(), 755, 4, false, 14).unwrap();
    let p = pillars(&v, lv, 10.0);
    println!("呂洞賓 755-4-14 巳\t{}\t{}", civil(lv), p.name());
    assert_eq!(gz_name(p.year), "乙未");
    // 寇准生年，宋史本传不载；以太平兴国五年进士、年十九推为建隆二年（961，辛酉）。七月为申月
    let kz = lunar_jdn(t.as_ref(), 961, 7, false, 14).unwrap();
    let p = pillars(&v, kz, 12.0);
    println!("寇準 961-7-14\t{}\t{}", civil(kz), p.name());
    assert_eq!(gz_name(p.month), "丙申");
}

/// 《三命通會》卷十所记有年月日之例
#[test]
fn san_ming_juan10() {
    let v = load_earth_vsop87_from_repo().unwrap();
    let t = zhang();
    // 成化丙戌暮秋朔後夜半，丙戌戊戌庚午丁亥
    let sz = SiZhu::parse("丙戌戊戌庚午丁亥").unwrap();
    let ds = find_days(&v, 1466, &sz);
    for &j in &ds {
        println!("成化例\t{}\t{}", civil(j), lunar_str(t.as_ref(), j));
    }
    assert_eq!(ds.len(), 1);
    assert_eq!(jdn_lunar(t.as_ref(), ds[0]).map(|x| x.1), Some(9));
    // 元文式五：大德丁未正月十七日壬午申時
    let j = lunar_jdn(t.as_ref(), 1307, 1, false, 17).unwrap();
    let p = pillars(&v, j, 16.0);
    println!("文式五\t{}\t{}", civil(j), p.name());
    assert_eq!(gz_name(p.day), "壬午");
    // 至正辛卯仲春，四柱皆辛卯
    let sz = SiZhu::parse("辛卯辛卯辛卯辛卯").unwrap();
    let ds = find_days(&v, 1351, &sz);
    for &j in &ds {
        println!("四辛卯\t{}\t{}", civil(j), lunar_str(t.as_ref(), j));
    }
}

/// 《三命通會》卷二《論大運》例：「甲子陽男十二月二十四日巳時生，是月也二十九日申時立春」——检元明各甲子年立春是否在十二月二十九日。
/// 结果：皆不在（例为假设之数，不是实有之日）；折算本身见 `pan::tests::qi_yun_rules`。
#[test]
fn san_ming_da_yun_example() {
    use crate::astronomy::aspects::solar_term_jds_for_year_cached;
    let v = load_earth_vsop87_from_repo().unwrap();
    let t = zhang();
    for ly in [1324, 1384, 1444, 1504, 1564] {
        let lc = solar_term_jds_for_year_cached(&v, ly + 1)[21];
        let clock = TimePoint::new(TimeScale::TT, lc).to_scale(TimeScale::UTC).jd + real(8.0 / 24.0);
        let j = (clock + real(0.5)).floor().to_i64_floor();
        let l = jdn_lunar(t.as_ref(), j);
        println!("{ly}\t次年立春 {}\t{}", civil(j), lunar_str(t.as_ref(), j));
        assert!(l.map(|x| (x.1, x.3)) != Some((12, 29)), "{ly}");
    }
}

/// 《三命通會》卷十一：「辛丑庚寅乙巳甲申……至乙酉運第五年丙申歲八月十九日死」——逐个辛丑年检起运与乙酉运之岁
#[test]
fn san_ming_yi_you_yun() {
    let v = load_earth_vsop87_from_repo().unwrap();
    let sz = SiZhu::parse("辛丑庚寅乙巳甲申").unwrap();
    let t = zhang();
    // 宋元明诸辛丑年（卷十一《明通賦》注引珞琭賦、徐子平解，或为宋人之命；萬曆六年刊本同文）
    for y in [941, 1001, 1061, 1121, 1181, 1241, 1301, 1361, 1421, 1481, 1541, 1601] {
        for j in find_days(&v, y, &sz) {
            let b = BirthInput { jd_tt: tt_at(j, 16.0), clock_utc_offset_hours: real(8.0), male: true };
            let pan = pai_pan(&v, &b, 8, &ZipingOptions::default());
            let yun: Vec<String> = pan.da_yun.iter().map(|d| format!("{}{:.1}", gz_name(d.gz), d.age.as_f64())).collect();
            // 乙酉运（逆行第五步）交运之日与丙申年八月十九日
            let yy = pan.da_yun[4];
            let jy = (yy.start_jd_tt + real(0.5 + 8.0 / 24.0)).floor().to_i64_floor();
            let death = lunar_jdn(t.as_ref(), y + 55, 8, false, 19);
            let nian = death.map(|d| (d - jy) as f64 / 365.2422 + 1.0);
            println!(
                "{y}\t{}（{}）\t起运{:.2}岁\t{}\t乙酉交运 {}（{}）\t丙申八月十九 {:?} 为运内第 {:.2} 年",
                civil(j),
                lunar_str(t.as_ref(), j),
                pan.qi_yun.age.as_f64(),
                yun.join(" "),
                civil(jy),
                lunar_str(t.as_ref(), jy),
                death.map(civil),
                nian.unwrap_or(f64::NAN)
            );
            // 1301、1541 两造：丙申八月十九日在乙酉运第十年，即「前後五年」之后五年（支运）第五年
            if y == 1301 || y == 1541 {
                assert!(nian.is_some_and(|n| (10.0..11.0).contains(&n)), "{y}");
            }
        }
    }
}

/// 《淵海》卷一、《三命》卷二交运例：「甲子年正月初一日子時正一刻生……其年內小六個月即進六日，在初七日……
/// 却為本年有閏月（三命作閏四月）……當退還本年十二月初七日」——检各甲子年正月初一加三百六十日（十二月各作三十日）落在何日，
/// 及所经十二月中小月之数、有无闰月
#[test]
fn jiao_yun_li_yue_example() {
    let t = zhang();
    for ly in [1204, 1264, 1324, 1384, 1444, 1504, 1564, 1624, 1984] {
        let Some(j0) = lunar_jdn(t.as_ref(), ly, 1, false, 1) else { continue };
        let j12 = j0 + 360;
        // 十二个朔望月后之初一与小月数
        let months: Vec<i64> = zhongyuan_months_in_jdn_range(t.as_ref(), j0, j0 + 400, ZhongyuanPreference::default())
            .into_iter()
            .map(|x| x.0)
            .filter(|&m| m >= j0)
            .collect();
        let small = months.windows(2).take(12).filter(|w| w[1] - w[0] == 29).count();
        let leap = zhongyuan_months_in_jdn_range(t.as_ref(), j0, months[12] - 1, ZhongyuanPreference::default())
            .into_iter()
            .filter(|x| x.0 >= j0 && x.3)
            .map(|x| x.2)
            .next();
        println!("{ly}\t正月初一 {}\t+360 日 {}（{}）\t十二月中小月 {small}\t闰 {:?}", civil(j0), civil(j12), lunar_str(t.as_ref(), j12), leap);
        // 《三命》「本年有閏四月……當退還本年十二月初七日」：弘治十七年甲子闰四月
        if ly == 1504 {
            assert_eq!((leap, jdn_lunar(t.as_ref(), j12).map(|x| (x.0, x.1, x.3))), (Some(4), Some((1504, 12, 7))));
        }
    }
}

/// 《三命通會》所举名人命造：通行生年内合年、月、日三柱之日（生年为通行之说，未逐一核原书）
const KNOWN_YEAR: &[(&str, &str, i32)] = &[
    ("韓魏公（韓琦）", "戊申庚申庚辰庚辰", 1008),
    ("呂吉甫（呂惠卿）", "壬申己酉丁巳庚子", 1032),
    ("蔡京", "丁亥壬寅壬辰辛巳", 1047),
    ("秦檜", "庚午己丑乙卯壬午", 1090),
    ("李東陽", "丁卯丁未癸亥己未", 1447),
    ("楊一清", "甲戌丙子壬午丁未", 1454),
    ("嚴嵩", "庚子己卯癸卯辛酉", 1480),
    ("楊博", "己巳庚午乙卯庚辰", 1509),
    ("陳以勤", "辛未戊戌丁卯辛亥", 1511),
    ("高拱", "壬申癸丑癸丑甲寅", 1512),
    ("胡宗憲", "壬申辛亥丁酉壬寅", 1512),
    ("王崇古", "乙亥辛巳戊申庚申", 1515),
    ("譚綸", "庚辰甲申丁未丙午", 1520),
    ("方逢時", "壬午乙巳甲辰己巳", 1522),
    ("劉應節", "癸未乙卯丙戌戊子", 1523),
    ("張居正", "乙酉辛巳辛酉辛卯", 1525),
    ("申時行", "乙未乙酉甲辰乙亥", 1535),
];

#[test]
fn known_year_cases() {
    let v = load_earth_vsop87_from_repo().unwrap();
    let t = zhang();
    for &(who, bz, y) in KNOWN_YEAR {
        let sz = SiZhu::parse(bz).unwrap();
        let ds = find_days(&v, y, &sz);
        let s: Vec<String> = ds.iter().map(|&j| format!("{}（{}）", civil(j), lunar_str(t.as_ref(), j))).collect();
        println!("{who}\t{bz}\t{y}\t{}", s.join("；"));
    }
}
