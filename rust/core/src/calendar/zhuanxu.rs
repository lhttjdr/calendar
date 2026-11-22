//! 颛顼历推步（秦至汉初，前 221—太初元年四月）：四分术平朔，十月为岁首，闰在年终（后九月）。
//!
//! - **常数**：四分术，日法 940，朔策 27759/940 日，岁实 1461/4 日（与古六历同，见 [super::ancient_restore]）。
//! - **历元**：前 366 年（天文纪年 −365）正月甲寅日夜半朔旦立春，为一蔀之首。东汉蔡邕、刘洪所称「人正正月己巳朔旦立春」
//!   （《开元占经》卷一百五积年 2761019，上元乙卯）是同一部历的另一蔀首：自前 366 年下推五蔀（380 年）至公元 15 年立春，
//!   日名正为己巳（测试 `epoch_is_the_han_transmitted_zhuanxu`）。
//! - **置闰**：术文不传。照张培瑜等《中国古代历法》（2013）第三章第九节（书页 242）：汉初「以固定冬至在十一月作为设闰的
//!   标准，但在文帝后元前后，做过变动。把以冬至总在十一月改成以雨水不出正月当作置闰的依据」；本项目取文帝后元元年（前 163，
//!   天文纪年 −162，李忠林复原亦于此年改章首）为界，−170 至 −160 间任取一年为界结果都相同。闰月一律放在岁末九月之后，称后九月。
//!   以平朔、平气时刻比较：建亥年 Y 的十月朔，是含该岁冬至（或雨水）之月往前推一（或三）月；Y 年十月至 Y+1 年十月之间
//!   若有十三月，第十三月为后九月。
//! - **核验**：同书表 3-19 至 3-22 所列颛顼历推步朔日（元光元年 13 月、惠帝三年 12 月、文帝二条、秦王政五条）干支与小余、
//!   元光元年四气小余逐格相合（测试 `matches_zhang_peiyu_tables`）。
//!
//! 同书的结论：汉初实行之历「不是颛顼历，也不是殷历」，节气与颛顼历相同，合朔小余比颛顼历大 430—487 分（近半日，书页 246）；
//! 秦王政时所记朔日合颛顼历（表 3-22），而据新出秦简，秦历朔小余又比汉传颛顼历大 639—649 分（书页 249）。所以本术是
//! 「史称颛顼历」的按术推步，不等于当时实用历日：照此推得的置闰与李忠林（2012）据出土历简的复原（[super::qinhan_restore]）
//! 在前 220—前 104 年逐年全同，朔日则复原历一半以上的月晚一日。见 doc/calendar/qin-han-calendar-evaluation.md「颛顼历推步」。
//! 本模块按术推步（第 4 级），由第 4 层 [super::lifa] 按本纪检验与今人复原比较取舍。

/// 时刻单位：1/22560 日（940 与 96 的最小公倍数，朔策、岁实、气策都是整数）。
const U: i64 = 22560;
/// 朔策 27759/940 日
const LUNAR: i64 = 27759 * (U / 940);
/// 岁实 1461/4 日
const SOLAR: i64 = 1461 * (U / 4);
/// 气策 = 岁实/24
const QI: i64 = SOLAR / 24;
/// 历元（前 366 年正月甲寅朔旦立春）所在之日的儒略日数；历元时刻为此日夜半。
pub const EPOCH_JDN: i64 = 1_587_781;
/// 历元之岁（天文纪年）。
pub const EPOCH_YEAR: i32 = -365;
/// 改用「雨水在正月」置闰标准的建亥年（文帝后元元年，前 163）。
pub const YUSHUI_RULE_FROM: i32 = -162;

/// 第 i 个平朔（自历元起）所在之日的儒略日数。
fn shuo_jdn(i: i64) -> i64 {
    EPOCH_JDN + (i * LUNAR).div_euclid(U)
}

/// 建亥年 Y（以其正月所在的夏正年记，即纪年年）十月朔的平朔序号。
fn shiyue_index(y: i32) -> i64 {
    let since = (y - EPOCH_YEAR) as i64 * SOLAR;
    // (锚点气距立春的气策数, 锚点所在之月距十月的月数)：冬至在十一月 / 雨水在正月
    let (qi, months_after_shiyue) = if y >= YUSHUI_RULE_FROM { (1, 3) } else { (-3, 1) };
    (since + qi * QI).div_euclid(LUNAR) - months_after_shiyue
}

/// 建亥年 Y 各月：(朔日 JDN, 月, 是否后九月)，自十月起至九月（或后九月）止。
pub fn jianhai_year_months(y: i32) -> Vec<(i64, u8, bool)> {
    const LABELS: [u8; 12] = [10, 11, 12, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let (a, b) = (shiyue_index(y), shiyue_index(y + 1));
    (a..b)
        .enumerate()
        .map(|(j, i)| if j < 12 { (shuo_jdn(i), LABELS[j], false) } else { (shuo_jdn(i), 9, true) })
        .collect()
}

/// 夏正年 y 各月 (朔日 JDN, 月, 闰否)：建亥年 y 的正月至九月（后九月记作闰九月）与建亥年 y+1 的十月至十二月，
/// 与张表编月一致（中原一线历日一律建寅编月）。
pub fn year_months(lunar_year: i32) -> Vec<(i64, u8, bool)> {
    jianhai_months_to_xiazheng(lunar_year, jianhai_year_months)
}

/// 建亥年序列换为夏正年 y 的各月（供本模块与 [super::qinhan_restore] 共用）。
pub(crate) fn jianhai_months_to_xiazheng(y: i32, f: impl Fn(i32) -> Vec<(i64, u8, bool)>) -> Vec<(i64, u8, bool)> {
    f(y).into_iter()
        .filter(|&(_, m, _)| m <= 9)
        .chain(f(y + 1).into_iter().filter(|&(_, m, leap)| m >= 10 && !leap))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gz(j: i64) -> usize {
        (j + 49).rem_euclid(60) as usize
    }

    /// 历元甲寅；下推五蔀（每蔀 27759 日）为公元 15 年己巳朔旦立春，即汉传颛顼历「己巳朔旦立春」之元。
    /// 前 366 至上元（《开元占经》积年 2761019 算外至开元二年）恰为整蔀。
    #[test]
    fn epoch_is_the_han_transmitted_zhuanxu() {
        assert_eq!(gz(EPOCH_JDN), 50, "甲寅");
        assert_eq!(gz(EPOCH_JDN + 5 * 27759), 5, "己巳");
        let shangyuan = 714 - 2_761_019;
        assert_eq!((EPOCH_YEAR - shangyuan).rem_euclid(76), 0, "前 366 年为蔀首");
        assert_eq!(SOLAR * 19, LUNAR * 235, "十九年七闰");
        // 历元时刻：正月朔与立春同在历元
        assert_eq!(shuo_jdn(0), EPOCH_JDN);
    }

    /// 建亥年 y 某月朔的 (日名, 小余（940 分）)。
    fn shuo_of(y: i32, m: u8, leap: bool) -> (&'static str, i64) {
        let j = jianhai_year_months(y).iter().position(|e| e.1 == m && e.2 == leap).expect("月") as i64;
        let i = shiyue_index(y) + j;
        (crate::calendar::gan_zhi::GAN_ZHI_60[gz(shuo_jdn(i))], (i * 27759).rem_euclid(940))
    }

    /// 张培瑜等《中国古代历法》（2013）第三章第九至十节所列颛顼历推步朔日与小余，逐格相合：
    /// 表 3-20 元光元年（书页 243）、表 3-21 惠帝三年（书页 247）、表 3-19 文帝十二年二月与十六年后九月（书页 240）、
    /// 表 3-22 秦王政二年至廿年（书页 248）。闰月与月序也随之核对（元光元年有后九月）。
    #[test]
    fn matches_zhang_peiyu_tables() {
        let yuanguang = [
            (10, "己丑", 394), (11, "戊午", 893), (12, "戊子", 452), (1, "戊午", 11), (2, "丁亥", 510), (3, "丁巳", 69),
            (4, "丙戌", 568), (5, "丙辰", 127), (6, "乙酉", 626), (7, "乙卯", 185), (8, "甲申", 684), (9, "甲寅", 243),
        ];
        for (m, g, x) in yuanguang {
            assert_eq!(shuo_of(-133, m, false), (g, x), "元光元年 {m} 月");
        }
        assert_eq!(shuo_of(-133, 9, true), ("癸未", 742), "元光元年后九月");
        let huidi = [
            (10, "乙未", 751), (11, "乙丑", 310), (12, "甲午", 809), (1, "甲子", 368), (2, "癸巳", 867), (3, "癸亥", 426),
            (4, "壬辰", 925), (5, "壬戌", 484), (6, "壬辰", 43), (7, "辛酉", 542), (8, "辛卯", 101), (9, "庚申", 600),
        ];
        assert_eq!(jianhai_year_months(-191).len(), 12, "惠帝三年无闰");
        for (m, g, x) in huidi {
            assert_eq!(shuo_of(-191, m, false), (g, x), "惠帝三年 {m} 月");
        }
        assert_eq!(shuo_of(-167, 2, false).0, "甲辰", "文帝十二年二月");
        assert_eq!(shuo_of(-163, 9, true).0, "丁未", "文帝十六年后九月");
        for (y, m, g, x) in [(-244, 10, "癸酉", 86), (-240, 8, "乙亥", 387), (-238, 7, "甲子", 143), (-226, 4, "丙戌", 118), (-226, 7, "甲寅", 675)] {
            assert_eq!(shuo_of(y, m, false), (g, x), "秦王政 {y} 年 {m} 月");
        }
        // 表 3-20 元光元年四气：冬至丙戌小余 11、立春壬申 0、夏至戊子 31、立秋甲戌 20（小余以 32 分为日）
        for (k, g, x) in [(-3, "丙戌", 11), (0, "壬申", 0), (9, "戊子", 31), (12, "甲戌", 20)] {
            let t = (-133 - EPOCH_YEAR) as i64 * SOLAR + k * QI;
            let d = EPOCH_JDN + t.div_euclid(U);
            assert_eq!((crate::calendar::gan_zhi::GAN_ZHI_60[gz(d)], t.rem_euclid(U) * 32 / U), (g, x), "元光元年气 {k}");
        }
        // 书页 239：始皇三十三年（前 214）入颛顼历壬申蔀首；书页 246：秦王政八年（前 239）入癸巳蔀 52 年
        assert_eq!(crate::calendar::gan_zhi::GAN_ZHI_60[gz(EPOCH_JDN + 2 * 27759)], "壬申");
        assert_eq!((-213 - EPOCH_YEAR).rem_euclid(76), 0);
        assert_eq!(crate::calendar::gan_zhi::GAN_ZHI_60[gz(EPOCH_JDN + 27759)], "癸巳");
        assert_eq!((-238 - (EPOCH_YEAR + 76)) + 1, 52);
    }

    /// 一年 12 或 13 月，月 29 或 30 日；十九年七闰；闰都是后九月。
    #[test]
    fn year_shapes() {
        let mut leaps = 0;
        for y in -220..=-104 {
            let m = jianhai_year_months(y);
            assert!(m.len() == 12 || m.len() == 13);
            assert_eq!(m[0].1, 10);
            leaps += (m.len() == 13) as i32;
            let next = jianhai_year_months(y + 1)[0].0;
            for (k, w) in m.iter().enumerate() {
                let end = m.get(k + 1).map_or(next, |x| x.0);
                assert!(end - w.0 == 29 || end - w.0 == 30, "{y} 年 {k}");
            }
            if m.len() == 13 {
                assert_eq!((m[12].1, m[12].2), (9, true));
            }
        }
        // 117 年：43 闰左右（7/19）
        assert!((42..=45).contains(&leaps), "{leaps}");
        // 夏正年：元封五年（−105）正月至十二月
        let y = year_months(-105);
        assert_eq!(y.iter().map(|m| m.1).collect::<Vec<_>>(), vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        assert!(year_months(-104).iter().any(|m| m.1 == 9 && m.2), "元封六年后九月");
    }
}
