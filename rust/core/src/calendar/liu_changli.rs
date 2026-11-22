//! 观象历（807—821）行用期的历日：北宋刘羲叟《长历》月朔，经司马光《资治通鉴目录》上栏保存（第 2 层：古人长历）。
//!
//! 观象历术一字不存（《新唐书·历志》「無蔀章之數」），本项目推不出它的历日；同系之正元、宣明代推，新旧唐书本纪明书朔
//! 只合 104、106/119。刘羲叟（北宋，与修《新唐书》律历、天文、五行志）《长历》原书佚，月朔只存于《通鉴目录》，本项目据
//! 维基共享资源 SSID-12318604（卷二十二至二十四）看图转录、逐月展开（`data/calendar/tongjian_mulu_807_821.csv`，
//! `scripts/calendar_data/tongjian_mulu_guanxiang.py`）。它仍是推排的结果，不是当时历日，但出处、年代、作者都清楚，
//! 本纪合 110/119，比代推多六条，比张表（即此表加清汪曰桢据本纪、碑记所改四处）的依据清楚。
//!
//! 取用时再按史载例外表（`data/calendar/dingshuo_exceptions.csv`，历名 [LIU_CHANGLI_NAME]）修正：本纪明书朔与目录相差
//! 一日、改后前后月长仍为 29、30 日者从本纪（类别「本纪」，汪曰桢已改三处亦在其中），813 年六月据韦瓘《大农陂记》
//! 改壬午（类别「他书」）。不取：811 年正月（本纪丙寅，差三十日，月名或干支讹）、811 年十二月（本纪癸亥，与下月本纪
//! 辛酉朔只隔 28 日，必有一讹）、817 年五月（本纪庚寅，目录五月庚申、闰五月庚寅，属闰位之差，本纪同月又书「五月辛酉」，
//! 自相出入）。见 doc/calendar/calendar-compendium.md「观象：张表的来历与《通鉴目录》对照」。

use std::sync::OnceLock;

/// 古人长历名（[super::lifa::RiliSource::AncientTable]），亦为史载例外表中的历名。
pub const LIU_CHANGLI_NAME: &str = "刘羲叟《长历》（《资治通鉴目录》）";

/// 覆盖的农历年（观象历行用期）。
pub const LIU_CHANGLI_YEARS: (i32, i32) = (807, 821);

/// 元和二年（807）正月己丑朔之 JDN（与正元推步、张表同日）。
const FIRST_JDN: i64 = 2_015_856;

/// 全部月（农历年, (朔日 JDN, 月, 闰)），已按史载例外修正。
fn all_months() -> &'static [(i32, (i64, u8, bool))] {
    static T: OnceLock<Vec<(i32, (i64, u8, bool))>> = OnceLock::new();
    T.get_or_init(|| {
        let csv = include_str!("../../../../data/calendar/tongjian_mulu_807_821.csv");
        let gz = |s: &str| super::gan_zhi::GAN_ZHI_60.iter().position(|g| *g == s).unwrap_or_else(|| panic!("目录朔「{s}」")) as i64;
        let mut out: Vec<(i32, (i64, u8, bool))> = Vec::new();
        for line in csv.lines().filter(|l| !l.starts_with('#') && !l.starts_with("年,") && !l.trim().is_empty()) {
            let c: Vec<&str> = line.split(',').collect();
            let (y, m, leap, g): (i32, u8, bool, i64) = (c[0].parse().unwrap(), c[1].parse().unwrap(), c[2] == "闰", gz(c[5]));
            let jdn = match out.last() {
                None => FIRST_JDN,
                Some((_, (prev, _, _))) => prev + 29 + (g - (prev + 29 + 49)).rem_euclid(60),
            };
            assert_eq!((jdn + 49).rem_euclid(60), g, "{y} 年 {m} 月");
            out.push((y, (jdn, m, leap)));
        }
        assert_eq!(out[0].1, (FIRST_JDN, 1, false));
        for w in out.windows(2) {
            assert!((29..=30).contains(&(w[1].1 .0 - w[0].1 .0)), "目录月长 {:?}", w);
        }
        let mut months: Vec<(i64, u8, bool)> = out.iter().map(|e| e.1).collect();
        super::push_calendar::apply_shizai_liwai(LIU_CHANGLI_NAME, &mut months);
        out.iter().zip(months).map(|((y, _), m)| (*y, m)).collect()
    })
}

/// 观象历行用期某农历年各月（朔日 JDN, 月, 闰否）；不在 807—821 年返回 None。
pub fn year_months(lunar_year: i32) -> Option<Vec<(i64, u8, bool)>> {
    (LIU_CHANGLI_YEARS.0..=LIU_CHANGLI_YEARS.1)
        .contains(&lunar_year)
        .then(|| all_months().iter().filter(|e| e.0 == lunar_year).map(|e| e.1).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 目录 185 月全展开；史载修正七处落实（812 年四月戊子、813 年六月壬午、814 年十二月甲辰、815 年六月辛丑、
    /// 821 年正月己亥、四月丙寅、十二月甲子）
    #[test]
    fn mulu_with_corrections() {
        assert_eq!(all_months().len(), 185);
        let gz = |y: i32, m: u8| {
            let j = year_months(y).unwrap().into_iter().find(|e| e.1 == m && !e.2).unwrap().0;
            super::super::gan_zhi::GAN_ZHI_60[(j + 49).rem_euclid(60) as usize]
        };
        assert_eq!(
            [gz(812, 4), gz(813, 6), gz(814, 12), gz(815, 6), gz(821, 1), gz(821, 4), gz(821, 12)],
            ["戊子", "壬午", "甲辰", "辛丑", "己亥", "丙寅", "甲子"]
        );
        assert_eq!(gz(811, 12), "壬戌", "811 年十二月本纪癸亥不取");
        assert_eq!(year_months(817).unwrap().iter().filter(|e| e.2).map(|e| e.1).collect::<Vec<_>>(), vec![5], "817 年闰五月");
        assert!(year_months(806).is_none() && year_months(822).is_none());
    }
}
