//! 麴氏高昌历推步（第 1 层：纯算术，平朔平气、无中气置闰）：北魏正光历常数 + 两段历元平移 δ。
//!
//! **性质**：「按术推步 + 据实物定参」。正光历的朔策、岁实、闰法出自术文（[super::push_calendar::zhengguang_params]）；
//! 历元平移 δ（平朔时刻一律加 δ 日，冬至不动）**不是术文**，是由吐鲁番出土砖志、衣物疏所记朔日、闰月与「某月卅日」
//! 反推的参数（`scripts/calendar_data/tulufan_jiri_check.py`，doc/calendar/tulufan-jiri-checks.md「改历元之年」「高昌历推步」）。
//! 史书不载高昌所用历术；高昌不行中原历、亦不行北周隋历，闰法合正光不合玄始，由砖志比对得出。
//!
//! **两段**（[SEGMENTS]）：以「满足条数最多」求 δ（184 条约束：朔 162 + 630 残历诸月、卅日、闰 7）——
//! - 前段（至 616 年）：最优区间 [−0.14497, −0.14307)（满足 121/126；两端由 565 年四月、573 年三月二方墓表之朔定），取中 −0.144；
//! - 后段（617 年起）：最优区间 [+0.01548, +0.03556)（满足 55/58；两端由 638 年十一月丙午朔二方、637 年七月卅日定），取中 +0.026；
//! - 切点：前后两段合计满足最多（176/184）者为后段始于 617 年。逐条看 616 年閏五月须前段、617 年十一月戊申朔
//!   （义和四年，《吐鲁番出土砖志集注》150 号）须后段，故改历元在 616 年閏五月之后、617 年十一月之前；后端只靠 617 年一方，
//!   若除去则可晚至 621 年。取 617 年为界是「满足最多」之解，不是记载。
//!
//! **年限**：523（北魏正光历行用之年，高昌得之不能更早；是推定，不是记载）至 640（延寿十七年，是年八月唐平高昌置西州，
//! 此后行唐历，见主序列）。有朔之砖志最早 537 年，523—536 年为外推、未经检验。

use super::push_calendar::{year_months as ping_year_months, zhengguang_params, LiParams};

/// 一段：自农历年 `from`（含）起行用，历元平移 `delta`（日），满足最多之 δ 区间 `[lo, hi)`。
#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub from: i32,
    pub delta: f64,
    pub lo: f64,
    pub hi: f64,
    pub name: &'static str,
}

/// 各段，按 `from` 递增。
pub const SEGMENTS: &[Segment] = &[
    Segment { from: 523, delta: -0.144, lo: -0.14497, hi: -0.14307, name: "高昌历（正光常数，历元 −0.144 日）" },
    Segment { from: 617, delta: 0.026, lo: 0.01548, hi: 0.03556, name: "高昌历（正光常数，历元 +0.026 日）" },
];

/// 覆盖的农历年（含两端）。
pub const YEARS: (i32, i32) = (523, 640);

/// 农历年 n 所属之段（n 早于首段者取首段）。
pub fn segment(n: i32) -> &'static Segment {
    SEGMENTS.iter().rev().find(|s| n >= s.from).unwrap_or(&SEGMENTS[0])
}

/// 正光常数下历元平移 δ 日之参数（只动合朔历元，冬至历元不动）。
pub fn params_with(delta: f64) -> LiParams {
    let p = zhengguang_params();
    LiParams { jd_epoch: p.jd_epoch + delta, ..p }
}

/// 某农历年各月 (朔日 JDN, 月序, 闰否)，按所属段之 δ 推步；不在 [YEARS] 内返回 None。
pub fn year_months(n: i32) -> Option<(&'static str, Vec<(i64, u8, bool)>)> {
    if !(YEARS.0..=YEARS.1).contains(&n) {
        return None;
    }
    let s = segment(n);
    Some((s.name, ping_year_months(&params_with(s.delta), n)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::gregorian::civil_to_jdn;

    const GZ_GAN: [char; 10] = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'];
    const GZ_ZHI: [char; 12] = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];

    fn gz(jdn: i64) -> String {
        let i = (jdn + 49).rem_euclid(60) as usize;
        format!("{}{}", GZ_GAN[i % 10], GZ_ZHI[i % 12])
    }

    fn shuo(y: i32, m: u8, leap: bool) -> i64 {
        year_months(y).unwrap().1.into_iter().find(|e| e.1 == m && e.2 == leap).unwrap_or_else(|| panic!("{y} 年无此月")).0
    }

    fn len(y: i32, m: u8, leap: bool) -> i64 {
        let all: Vec<i64> = (y..=y + 1).flat_map(|yy| year_months(yy).unwrap().1).map(|e| e.0).collect();
        let s = shuo(y, m, leap);
        all.iter().find(|&&j| j > s).unwrap() - s
    }

    /// 砖志所书之朔（集注录文）。
    #[test]
    fn zhuanzhi_shuo() {
        // 义和四年丁丑（617）十一月戊申朔：后段之证（集注 150 号）
        assert_eq!(gz(shuo(617, 11, false)), "戊申");
        // 义和二年乙亥（615）六月朔辛酉，卅日庚寅（唐幼谦妻麴氏墓表）：前段，六月大
        assert_eq!(gz(shuo(615, 6, false)), "辛酉");
        assert_eq!(len(615, 6, false), 30);
        // 延寿七年（630）残历：七月甲子、十月壬辰、十二月壬辰（小）；延寿八年正月辛酉
        assert_eq!(gz(shuo(630, 7, false)), "甲子");
        assert_eq!(gz(shuo(630, 12, false)), "壬辰");
        assert_eq!(len(630, 12, false), 29);
        assert_eq!(gz(shuo(631, 1, false)), "辛酉");
        // 延寿七年七月甲子朔即民用历（儒略历）630 年 8 月 14 日
        assert_eq!(shuo(630, 7, false), civil_to_jdn(630, 8, 14));
    }

    /// 闰位：正光闰法 + 历元平移下 597 閏六、600 閏二、605 閏七、616 閏五、627 閏四、635 閏五皆合；592 閏正月不合（录文据诸家，原砖漫漶）。
    #[test]
    fn leap_months() {
        let leap = |y: i32| year_months(y).unwrap().1.into_iter().find(|e| e.2).map(|e| e.1);
        for (y, m) in [(597, 6), (600, 2), (605, 7), (616, 5), (627, 4), (635, 5)] {
            assert_eq!(leap(y), Some(m), "{y} 年闰位");
        }
        assert_ne!(leap(592), Some(1));
    }

    /// δ 区间两端由哪几方砖志定：前段下端 565 年四月癸丑朔、上端 573 年三月丙寅朔；后段下端 638 年十一月丙午朔、上端 637 年七月卅日（月大）。
    /// 区间内取值不改这几条，刚出区间即失之。
    #[test]
    fn delta_interval_ends() {
        let at = |d: f64, y: i32, m: u8| {
            let ms = ping_year_months(&params_with(d), y);
            let i = ms.iter().position(|e| e.1 == m && !e.2).unwrap();
            let next = ms.get(i + 1).map(|e| e.0).unwrap_or_else(|| ping_year_months(&params_with(d), y + 1)[0].0);
            (gz(ms[i].0), next - ms[i].0)
        };
        let [a, b] = [SEGMENTS[0], SEGMENTS[1]];
        for d in [a.lo, a.delta, a.hi - 1e-5] {
            assert_eq!(at(d, 565, 4).0, "癸丑", "δ = {d}");
            assert_eq!(at(d, 573, 3).0, "丙寅", "δ = {d}");
        }
        assert_ne!(at(a.lo - 1e-5, 565, 4).0, "癸丑");
        assert_ne!(at(a.hi, 573, 3).0, "丙寅");
        for d in [b.lo, b.delta, b.hi - 1e-5] {
            assert_eq!(at(d, 638, 11).0, "丙午", "δ = {d}");
            assert_eq!(at(d, 637, 7).1, 30, "δ = {d}");
        }
        assert_ne!(at(b.lo - 1e-5, 638, 11).0, "丙午");
        assert_eq!(at(b.hi, 637, 7).1, 29);
    }

    /// 段界前后月长皆 29、30 日；年限外不推。
    #[test]
    fn segment_boundary_and_range() {
        let all: Vec<i64> = (YEARS.0..=YEARS.1).flat_map(|y| year_months(y).unwrap().1).map(|e| e.0).collect();
        assert!(all.windows(2).all(|w| (29..=30).contains(&(w[1] - w[0]))));
        assert!(year_months(522).is_none() && year_months(641).is_none());
        assert_eq!(segment(616).delta, -0.144);
        assert_eq!(segment(617).delta, 0.026);
        for s in SEGMENTS {
            assert!(s.lo <= s.delta && s.delta < s.hi);
        }
    }
}
