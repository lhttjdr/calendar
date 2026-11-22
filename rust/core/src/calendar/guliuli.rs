//! 战国古六历与春秋鲁历推步（第 1 层），供推步政权表（[super::push_calendar::regime_calendars]）按国别使用。
//!
//! ## 汉传古六历
//!
//! 黄帝、颛顼、夏、殷、周、鲁六历都是四分术（朔策 27759/940 日，岁实 1461/4 日），只差上元与历元气朔
//! （张培瑜等《中国古代历法》2013，第三章第六、七节，书页 206—236）。合朔历元照 [super::ancient_restore] 所录
//! （ytliu0 `guliuli_calendar_parameters`，已以《开元占经》积年核过，calendar-compendium §十二），本模块改用整数运算，
//! 并以同书表 3-17、3-18（魏安釐王二十五年六历推步，书页 234—236）六历各 19 个月朔日干支与小余逐格核对。
//!
//! - **建正**：同书第七节「五历皆建子，以含冬至之月为正月」，算例中殷、周、黄帝、鲁四历以子月为正月，夏历行寅正
//!   （「六历中……夏历行寅正」，书页 234），颛顼历十月岁首另见 [super::zhuanxu]。
//! - **置闰**：子正四历岁始闰余（冬至月龄）满 12/19 月之岁有闰，即冬至总在正月；夏历以正月中气雨水定闰，正月须含雨水，
//!   同书算例「冬至后第二月为闰月，即闰十二月。以为无正月中气雨水之月」（书页 234）。闰月位置有两说：
//!   同书书页 216 以为先秦「很可能仍采用年终闰」，「杜预、祖冲之采用的无中置闰法……可能均非战国六历之术数」；
//!   云梦秦简「（魏安釐王）廿五年闰再十二月」、秦简后九月皆年终。缺省 [Runfa::Nianzhong] 年终闰（闰十二月），
//!   另备 [Runfa::Wuzhongqi] 年中无中气之月为闰，即同书表 3-18 的排法（周历闰五月、黄帝闰三月等）。「含中气」照表 3-18
//!   反推：中气不早于本月合朔时刻，且不在下月朔日（与下月合朔同日者归下月）；纯按时刻或纯按日判，四历闰位都比表晚一月。
//!
//! 月序为**当时月名**（子正历正月即子月），不是中原一线的建寅编号；年为该历正月所在之年（子正历正月始于上一公历年十二月）。
//!
//! ## 春秋鲁历
//!
//! 张培瑜《中国古代历法》表 3-7（第三章第五节，书页 193—200）据《春秋》历日排定的鲁国历：固定闰年表（前 722—前 480，243 年），
//! 朔策 30328/1027 日，闰在岁末。参数照上游 ytliu0 `chunqiu_cmonth`（与 `scripts/calendar_data/chunqiu_check.py` 同），
//! 与表 3-7 正朔逐年核过（`data/calendar/chunqiu_zhang_3_7.csv`，243 年只三年不同，上游已判书误）。这是今人拟合，不是术文推步。

/// 时刻单位：1/22560 日。
const U: i64 = 22560;
const LUNAR: i64 = 27759 * (U / 940);
const SOLAR: i64 = 1461 * (U / 4);
const ZHONGQI: i64 = SOLAR / 12;

/// 汉传古六历（颛顼历另见 [super::zhuanxu]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Liu {
    Huangdi,
    Zhou,
    Yin,
    Lu,
    /// 夏历（历元十一月甲子朔旦冬至；同书「依夏历（历元冬至）」，魏所行）
    Xia,
    /// 夏历另一说（历元人正甲子朔旦雨水，同书表 3-11「夏历（历元人正甲子雨水合朔）」）
    XiaYushui,
}

/// 闰月位置。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Runfa {
    /// 年终置闰（闰十二月）
    #[default]
    Nianzhong,
    /// 年中无中气之月为闰（杜预、祖冲之法；同书表 3-18 的排法）
    Wuzhongqi,
}

impl Liu {
    pub const ALL: [Liu; 6] = [Liu::Huangdi, Liu::Zhou, Liu::Yin, Liu::Lu, Liu::Xia, Liu::XiaYushui];

    pub fn name(self) -> &'static str {
        match self {
            Liu::Huangdi => "黄帝历",
            Liu::Zhou => "周历",
            Liu::Yin => "殷历",
            Liu::Lu => "鲁历",
            Liu::Xia => "夏历",
            Liu::XiaYushui => "夏历（雨水历元）",
        }
    }

    /// (合朔历元之日 JDN（历元时刻为此日夜半）, 合朔历元时刻相对此夜半, 同一历元附近之冬至相对此夜半)，单位 U。
    fn epoch(self) -> (i64, i64, i64) {
        match self {
            Liu::Huangdi => (1_783_511, 0, 0),
            Liu::Zhou => (1_683_431, 0, 0),
            Liu::Yin => (1_704_251, 0, 0),
            // 鲁：冬至历元甲子夜半，合朔早 1/19 月（「以闰余一之岁为蔀首」）
            Liu::Lu => (1_545_731, -LUNAR / 19, 0),
            Liu::Xia => (1_883_591, 0, 0),
            // 雨水与合朔齐同于甲子夜半，冬至在其前 1/6 岁
            Liu::XiaYushui => (1_883_651, 0, -SOLAR / 6),
        }
    }

    /// 定正月的中气距冬至的中气数：子正历正月含冬至（0）；夏历正月含雨水（2，书页 234「以為無正月中氣雨水之月」）。
    fn zheng_zhongqi(self) -> i64 {
        match self {
            Liu::Xia | Liu::XiaYushui => 2,
            _ => 0,
        }
    }
}

struct Ctx {
    day0: i64,
    shuo0: i64,
    dz0: i64,
    /// dz0 那次冬至之后正月所在之年（以冬至后 30 日的公历年计）
    year0: i32,
}

fn ctx(li: Liu) -> Ctx {
    let (day0, shuo0, dz0) = li.epoch();
    let (y, _, _) = super::gregorian::civil_from_jdn(day0 + dz0.div_euclid(U) + 30);
    Ctx { day0, shuo0, dz0, year0: y }
}

impl Ctx {
    /// y 年之前那次冬至（上一公历年十二月前后）的时刻。
    fn dongzhi(&self, y: i32) -> i64 {
        self.dz0 + (y - self.year0) as i64 * SOLAR
    }
    /// 含时刻 t 之月的平朔序号。
    fn month_of(&self, t: i64) -> i64 {
        (t - self.shuo0).div_euclid(LUNAR)
    }
    fn shuo(&self, i: i64) -> i64 {
        self.shuo0 + i * LUNAR
    }
    fn jdn(&self, t: i64) -> i64 {
        self.day0 + t.div_euclid(U)
    }
}

/// 古六历 y 年（该历正月所在之年）各月：(朔日 JDN, 当时月序, 闰否)。
pub fn year_months(li: Liu, lunar_year: i32, runfa: Runfa) -> Vec<(i64, u8, bool)> {
    let c = ctx(li);
    let z = |y: i32| c.month_of(c.dongzhi(y) + li.zheng_zhongqi() * ZHONGQI);
    let (a, b) = (z(lunar_year), z(lunar_year + 1));
    let n = (b - a) as usize;
    let leap_at = if n == 13 {
        match runfa {
            Runfa::Nianzhong => Some(12),
            Runfa::Wuzhongqi => {
                // 年中第一个无中气之月为闰；中气 = 冬至 + k·岁实/12
                let dz = c.dongzhi(lunar_year);
                // 某月含中气：中气不早于本月合朔时刻，且不在下月朔日（与下月合朔同日者归下月）
                (1..13).find(|&j| {
                    let (s, e) = (c.shuo(a + j as i64), c.shuo(a + j as i64 + 1));
                    let k0 = (s - dz + ZHONGQI - 1).div_euclid(ZHONGQI);
                    let t = dz + k0 * ZHONGQI;
                    c.jdn(t) >= c.jdn(e)
                })
                .or(Some(12))
            }
        }
    } else {
        None
    };
    let mut out = Vec::with_capacity(n);
    let mut m = 0u8;
    for j in 0..n {
        let d = c.jdn(c.shuo(a + j as i64));
        if Some(j) == leap_at {
            out.push((d, m, true));
        } else {
            m += 1;
            out.push((d, m, false));
        }
    }
    out
}

/// 古六历第 i 月朔的 (日干支序, 小余（940 分）)——供与《中国古代历法》表 3-18 核对。
pub fn shuo_ganzhi_xiaoyu(li: Liu, jdn_near: i64) -> (usize, i64) {
    let c = ctx(li);
    let t = (jdn_near - c.day0) * U;
    let i = (t - c.shuo0 + LUNAR / 2).div_euclid(LUNAR);
    let s = c.shuo(i);
    let d = c.jdn(s);
    (((d + 49).rem_euclid(60)) as usize, (s.rem_euclid(U)) * 940 / U)
}

// ---------------- 春秋鲁历 ----------------

/// 春秋鲁历（据经文排定）的历名。
pub const CHUNQIU_JINGWEN_NAME: &str = "春秋鲁历（据《春秋》经文排定）";

/// 春秋鲁历由《春秋》经文历日直接排定之月表（`data/calendar/chunqiu_luli_months.csv`，`scripts/calendar_data/chunqiu_luli_rebuild.py`
/// 生成）：平朔（朔策 30328/1027），逐年正月与岁末闰按经文纪日（书朔须朔、日食须在有食之合朔）与置闰节奏先验求最大后验，
/// 每年注「经文定 / 无经文按建推定 / 未定」（后验见 `chunqiu_luli_rebuild.csv`）。分政权「鲁」缺省取此；张培瑜表 3-7
/// （[chunqiu_year_months]）是同类拟合而不注各年依据，只作对照。y 超出范围返回空。
pub fn chunqiu_jingwen_year_months(lunar_year: i32) -> Vec<(i64, u8, bool)> {
    use std::collections::BTreeMap;
    use std::sync::OnceLock;
    static T: OnceLock<BTreeMap<i32, Vec<(i64, u8, bool)>>> = OnceLock::new();
    T.get_or_init(|| {
        let mut m: BTreeMap<i32, Vec<(i64, u8, bool)>> = BTreeMap::new();
        for l in include_str!("../../../../data/calendar/chunqiu_luli_months.csv").lines().filter(|l| !l.starts_with('#') && !l.starts_with("年,")) {
            let c: Vec<&str> = l.split(',').collect();
            m.entry(c[0].parse().unwrap()).or_default().push((c[3].parse().unwrap(), c[1].parse().unwrap(), c[2] == "1"));
        }
        m
    })
    .get(&lunar_year)
    .cloned()
    .unwrap_or_default()
}

/// 春秋历闰年表（前 722—前 480 年，1 为闰）：上游 ytliu0 `chunqiu_cmonth`，出自张培瑜表 3-7。
const CQ_LEAP: &str = "010010101001001001001010001001010001000011011001000100101001001100000101010100010100010100001001101001001001001001000101100100101000100100100101100010100100100101000100110010001100010100101001000101010100010011000010101000101010010010100100100";
/// 春秋历首年（天文纪年，隐公元年前 722）。
pub const CHUNQIU_FIRST_YEAR: i32 = -721;

/// 春秋历覆盖的末年（天文纪年）。
pub fn chunqiu_last_year() -> i32 {
    CHUNQIU_FIRST_YEAR + CQ_LEAP.len() as i32 - 1
}

/// 春秋鲁历 y 年各月：(朔日 JDN, 月序, 闰否)，闰在岁末记闰十二月。y 超出闰年表返回空。
pub fn chunqiu_year_months(lunar_year: i32) -> Vec<(i64, u8, bool)> {
    let o = (lunar_year - CHUNQIU_FIRST_YEAR) as usize;
    let leaps = CQ_LEAP.as_bytes();
    if lunar_year < CHUNQIU_FIRST_YEAR || o >= leaps.len() {
        return Vec::new();
    }
    let acc: i64 = leaps[..o].iter().filter(|&&c| c == b'1').count() as i64;
    let n0 = 12 * o as i64 + acc;
    let len = 12 + (leaps[o] == b'1') as usize;
    // 历元 J0 = 1457727.761054236（上游十进制常数），朔策 30328/1027；朔日 = ⌊J0 + k·朔策 + 0.5⌋，精确有理数运算
    const J0_NUM: i128 = 1_457_727_761_054_236;
    const J0_DEN: i128 = 1_000_000_000;
    (0..len)
        .map(|j| {
            let k = (n0 + j as i64) as i128;
            let num = (J0_NUM * 2 + J0_DEN) * 1027 + 2 * J0_DEN * k * 30328;
            let den = 2 * J0_DEN * 1027;
            let d = num.div_euclid(den) as i64;
            if j < 12 { (d, j as u8 + 1, false) } else { (d, 12, true) }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::gan_zhi::GAN_ZHI_60;

    fn gz(d: i64) -> &'static str {
        GAN_ZHI_60[(d + 49).rem_euclid(60) as usize]
    }

    /// 《中国古代历法》表 3-18（书页 235—236）：魏安釐王二十五年前后六历各月朔干支、小余，自前 253 年 12 月 11 日起 19 个月
    /// （颛顼历见 zhuanxu 测试）；并核表 3-17 天正朔与闰否、表 3-18 闰月位置（无中气排法）。
    #[test]
    fn matches_zhang_table_3_18() {
        let rows: [(Liu, &str); 5] = [
            (Liu::Xia, "壬子849 壬午408 辛亥907 辛巳466 辛亥25 庚辰524 庚戌83 己卯582 己酉141 戊寅640 戊申199 丁丑698 丁未257 丙子756 丙午315 乙亥814 乙巳373 甲戌872 甲辰431"),
            (Liu::Zhou, "癸丑11 壬午510 壬子69 辛巳568 辛亥127 庚辰626 庚戌185 己卯684 己酉243 戊寅742 戊申301 丁丑800 丁未359 丙子858 丙午417 乙亥916 乙巳475 乙亥34 甲辰533"),
            (Liu::Yin, "癸丑716 癸未275 壬子774 壬午333 辛亥832 辛巳391 庚戌890 庚辰449 庚戌8 己卯507 己酉66 戊寅565 戊申124 丁丑623 丁未182 丙子681 丙午240 乙亥739 乙巳298"),
            (Liu::Huangdi, "壬子900 壬午459 壬子18 辛巳517 辛亥76 庚辰575 庚戌134 己卯633 己酉192 戊寅691 戊申250 丁丑749 丁未308 丙子807 丙午366 乙亥865 乙巳424 甲戌923 甲辰482"),
            (Liu::Lu, "癸丑767 癸未326 壬子825 壬午384 辛亥883 辛巳442 辛亥1 庚辰500 庚戌59 己卯558 己酉117 戊寅616 戊申175 丁丑674 丁未233 丙子732 丙午291 乙亥790 乙巳349"),
        ];
        let d0 = crate::calendar::gregorian::civil_to_jdn(-252, 12, 11);
        for (li, s) in rows {
            let c = ctx(li);
            let i0 = c.month_of((d0 - c.day0 + 1) * U - 1);
            let got: Vec<String> = (0..19)
                .map(|j| {
                    let t = c.shuo(i0 + j);
                    format!("{}{}", gz(c.jdn(t)), t.rem_euclid(U) * 940 / U)
                })
                .collect();
            assert_eq!(got.join(" "), s, "{}", li.name());
        }
        // 月序与闰：夏历寅正，前 252 年（天文 −251）年终闰再十二月丙午；周、殷、黄帝、鲁子正，前 251 年（天文 −250）有闰，
        // 年中无中气排法下周、殷闰五月，黄帝闰三月，鲁闰六月（表 3-18）
        let xia = year_months(Liu::Xia, -251, Runfa::Nianzhong);
        assert_eq!(xia.len(), 13);
        assert_eq!((gz(xia[12].0), xia[12].1, xia[12].2), ("丙午", 12, true));
        assert_eq!(xia[12].0, crate::calendar::gregorian::civil_to_jdn(-250, 1, 28), "前 251 年 1 月 28 日");
        assert_eq!(gz(year_months(Liu::Xia, -250, Runfa::Nianzhong)[0].0), "乙亥", "前 251 年正月乙亥");
        for (li, m, g) in [(Liu::Zhou, 5, "乙亥"), (Liu::Yin, 5, "乙亥"), (Liu::Huangdi, 3, "乙亥"), (Liu::Lu, 6, "乙巳")] {
            assert_eq!(year_months(li, -251, Runfa::Nianzhong).len(), 12, "{} 前 252 年不闰", li.name());
            let y = year_months(li, -250, Runfa::Wuzhongqi);
            let l = y.iter().find(|e| e.2).unwrap();
            assert_eq!((l.1, gz(l.0)), (m, g), "{}", li.name());
            assert_eq!(gz(y[0].0), ["丁未", "戊申", "丁未", "戊申"][[Liu::Zhou, Liu::Yin, Liu::Huangdi, Liu::Lu].iter().position(|x| *x == li).unwrap()], "{} 正月", li.name());
            let n = year_months(li, -250, Runfa::Nianzhong);
            assert_eq!((n.len(), n[12].1, n[12].2), (13, 12, true));
        }
    }

    /// 十九年七闰；月 29 或 30 日；前后年相接。
    #[test]
    fn year_shapes() {
        for li in Liu::ALL {
            for r in [Runfa::Nianzhong, Runfa::Wuzhongqi] {
                let mut leaps = 0;
                for y in -480..-480 + 76 {
                    let m = year_months(li, y, r);
                    let next = year_months(li, y + 1, r)[0].0;
                    leaps += (m.len() == 13) as i32;
                    assert_eq!(m.iter().filter(|e| e.2).count(), m.len() - 12);
                    for (k, e) in m.iter().enumerate() {
                        let end = m.get(k + 1).map_or(next, |x| x.0);
                        assert!((29..=30).contains(&(end - e.0)), "{} {y} {k}", li.name());
                    }
                }
                assert_eq!(leaps, 28, "{} 一蔀 28 闰", li.name());
            }
        }
    }

    /// 春秋历正朔与张培瑜表 3-7 逐年（`data/calendar/chunqiu_zhang_3_7.csv` 书干支）：只隐二、僖六两年干支不同（上游判书误）。
    #[test]
    fn chunqiu_matches_table_3_7() {
        assert_eq!(chunqiu_last_year(), -479);
        let Ok(csv) = std::fs::read_to_string(crate::repo::repo_root().join("data/calendar/chunqiu_zhang_3_7.csv")) else { return };
        let mut diff = Vec::new();
        let mut n = 0;
        for line in csv.lines().filter(|l| !l.starts_with('#') && !l.starts_with("年")) {
            let f: Vec<&str> = line.split(',').collect();
            let y: i32 = f[0].parse().unwrap();
            let m = chunqiu_year_months(y);
            if m.is_empty() {
                continue;
            }
            n += 1;
            if gz(m[0].0) != f[1] {
                diff.push(y);
            }
        }
        assert!(n >= 240, "{n}");
        assert_eq!(diff, vec![-720, -653], "隐二、僖六");
    }
}
