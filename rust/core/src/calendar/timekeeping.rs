//! 按时代把一日内的时刻写成当时的计时法（独立基础层，只做命名换算）。
//!
//! 输入是民用钟面时分秒，古代本用地方视太阳时，这里不作换算，所以只是表示法，不是当时当地的实测时刻。
//!
//! | 农历年 | 写法 | 依据 |
//! |---|---|---|
//! | 前 221 以前 | 不写 | 尚无能对应钟点的统一时制 |
//! | 前 221—506 | 十二时称（夜半、鸡鸣……） | 睡虎地秦简《日书》已有完整十二时；学界对起点有异说 |
//! | 507—543 | 辰 + 刻，日 96 刻 | 《隋书·天文志》：天监六年「以昼夜为九十六刻，一辰有全刻八焉」 |
//! | 544—559 | 辰 + 刻，日 108 刻 | 同上：大同十年「又改用一百八刻」 |
//! | 560—959 | 辰 + 刻，日 100 刻 | 同上：陈文帝天嘉中「依古百刻为法」；天嘉年内具体年份不详，取元年 |
//! | 960—1644 | 时初/时正 + 刻，日 100 刻 | 辰分初、正，通行说起于北宋，取宋开国之年 |
//! | 1645—1911 | 时初/时正 + 刻分秒，日 96 刻 | 时宪历；每刻 15 分、每分 60 秒（《御制数理精蕴》） |
//! | 1912— | 24 小时制 | 民国起 |
//!
//! 百刻制每刻的「分」历代不一，这里不写到分，避免虚假的精度。

const ZHI: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];
/// 与地支同序：子对夜半，亥对人定。
const SHICHENG: [&str; 12] = [
    "夜半", "鸡鸣", "平旦", "日出", "食时", "隅中", "日中", "日昳", "晡时", "日入", "黄昏", "人定",
];
const KE: [&str; 9] = ["初", "一", "二", "三", "四", "五", "六", "七", "八"];
const DAY_SECONDS: u32 = 86400;
const CHEN_SECONDS: u32 = DAY_SECONDS / 12;

/// 自子初（前一日 23:00）起算的秒数。
fn seconds_from_zichu(hour: u32, minute: u32, second: u32) -> u32 {
    (hour * 3600 + minute * 60 + second + 3600) % DAY_SECONDS
}

/// 第几刻：`seconds` 为自本段起算的秒数，日分 `ke_per_day` 刻。
fn ke_index(seconds: u32, ke_per_day: u32) -> usize {
    (seconds * ke_per_day / DAY_SECONDS) as usize
}

/// 整辰计刻：丑时三刻。
fn chen_ke(from_zichu: u32, ke_per_day: u32) -> String {
    let within = from_zichu % CHEN_SECONDS;
    let zhi = ZHI[(from_zichu / CHEN_SECONDS) as usize];
    format!("{zhi}时{}刻", KE[ke_index(within, ke_per_day)])
}

/// 辰分初、正：丑正初刻。
fn chen_half_ke(from_zichu: u32, ke_per_day: u32) -> String {
    let half = CHEN_SECONDS / 2;
    let within = from_zichu % CHEN_SECONDS;
    let zhi = ZHI[(from_zichu / CHEN_SECONDS) as usize];
    let label = if within < half { "初" } else { "正" };
    format!("{zhi}{label}{}刻", KE[ke_index(within % half, ke_per_day)])
}

fn small_cn(n: u32) -> String {
    const D: [&str; 10] = ["零", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
    match n {
        0..=9 => D[n as usize].to_string(),
        10 => "十".to_string(),
        11..=19 => format!("十{}", D[(n % 10) as usize]),
        _ if n % 10 == 0 => format!("{}十", D[(n / 10) as usize]),
        _ => format!("{}十{}", D[(n / 10) as usize], D[(n % 10) as usize]),
    }
}

/// 时宪历：丑正初刻零一分五十四秒。
fn shixian(from_zichu: u32) -> String {
    let head = chen_half_ke(from_zichu, 96);
    let within_ke = from_zichu % 900;
    let (fen, miao) = (within_ke / 60, within_ke % 60);
    match (fen, miao) {
        (0, 0) => head,
        (_, 0) => format!("{head}零{}分", small_cn(fen)),
        _ => format!("{head}零{}分{}秒", small_cn(fen), small_cn(miao)),
    }
}

/// 该农历年通行的写法；前 221 年以前返回空串。
pub fn format_time_for_lunar_year(lunar_year: i32, hour: u32, minute: u32, second: u32) -> String {
    let from_zichu = seconds_from_zichu(hour, minute, second);
    match lunar_year {
        i32::MIN..=-222 => String::new(),
        -221..=506 => SHICHENG[(from_zichu / CHEN_SECONDS) as usize].to_string(),
        507..=543 => chen_ke(from_zichu, 96),
        544..=559 => chen_ke(from_zichu, 108),
        560..=959 => chen_ke(from_zichu, 100),
        960..=1644 => chen_half_ke(from_zichu, 100),
        1645..=1911 => shixian(from_zichu),
        _ => format!("{hour:02}:{minute:02}:{second:02}"),
    }
}

#[cfg(test)]
mod tests {
    use super::format_time_for_lunar_year as f;

    #[test]
    fn each_era_uses_its_own_notation() {
        assert_eq!(f(-300, 2, 1, 54), "");
        assert_eq!(f(100, 2, 1, 54), "鸡鸣");
        assert_eq!(f(100, 23, 30, 0), "夜半");
        assert_eq!(f(520, 2, 1, 54), "丑时四刻");
        assert_eq!(f(700, 12, 0, 0), "午时四刻");
        assert_eq!(f(1500, 2, 1, 54), "丑正初刻");
        assert_eq!(f(1800, 2, 1, 54), "丑正初刻零一分五十四秒");
        assert_eq!(f(1800, 1, 0, 0), "丑初初刻");
        assert_eq!(f(2026, 2, 1, 54), "02:01:54");
    }

    #[test]
    fn ke_stays_in_range_at_end_of_each_chen() {
        for y in [520, 550, 700, 1500, 1800] {
            for h in 0..24 {
                assert!(!f(y, h, 59, 59).is_empty());
            }
        }
        assert_eq!(f(550, 0, 59, 59), "子时八刻");
        assert_eq!(f(1500, 0, 59, 59), "子正四刻");
    }
}
