//! 西夏历日（1120—1207）：彭向前《俄藏西夏历日文献整理研究》（社会科学文献出版社 2018）下篇二「西夏历与宋历朔闰对照表」
//! 转录（`data/calendar/xixia_song_shuorun.csv`，`scripts/calendar_data/xixia_songli_table.py`）。
//!
//! 主要依据俄藏黑水城 Инв.No.8085 夏汉合璧历书（1120—1207，88 年，原件实物；书中逐月释读朔日、月大小、闰月），
//! 另有 Инв.No.5282、英藏 Or.12380、中国藏 G21·028 等残历与纪年文书。书的结论：「夏历的朔日与宋历或同，或前后相差一日」，
//! 「闰月与宋历或同，或前后相差一月」，「这表明西夏历是承袭宋历而来的」（书页 469）；宋颁历于西夏见《宋史·夏国传》
//! （赵德明时「頒儀天具注曆」；绍兴元年「詔以夏本敵國，毋復班曆日」）。
//!
//! 1088 个月中：西夏朔日与宋历同 940、早一日 12、晚一日 10，原缺或原误待考 126（这些月以宋历朔日代，[XixiaMonth::basis] 标明）；
//! 闰月位置不同者 5 年。月序为西夏历月序。
//! 本模块是第 2 层（历日实物经今人整理），经第 4 层推步政权表 [super::push_calendar::regime_calendars] 供换算。

/// 某月历日的来源。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XixiaBasis {
    /// 书中有西夏朔日（Инв.No.8085 等原件释读）
    Original,
    /// 西夏朔日原缺或原误待考，以同月宋历朔日代（月序仍从西夏历）
    SongFallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct XixiaMonth {
    pub jdn: i64,
    pub year: i32,
    pub month: u8,
    pub leap: bool,
    pub basis: XixiaBasis,
}

/// 全部月份（按时间先后）。
pub fn months() -> &'static [XixiaMonth] {
    use std::sync::OnceLock;
    static M: OnceLock<Vec<XixiaMonth>> = OnceLock::new();
    M.get_or_init(|| parse(include_str!("../../../../data/calendar/xixia_song_shuorun.csv")))
}

fn parse(csv: &str) -> Vec<XixiaMonth> {
    let mut lines = csv.lines().filter(|l| !l.starts_with('#'));
    let head: Vec<&str> = lines.next().expect("表头").split(',').collect();
    let col = |n: &str| head.iter().position(|h| *h == n).unwrap_or_else(|| panic!("缺列 {n}"));
    let (cy, csm, cxm, csj, cxj) = (col("公元年"), col("宋月"), col("夏月"), col("宋朔JDN"), col("夏朔JDN"));
    let mut out: Vec<XixiaMonth> = lines
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            let lab = if f[cxm].is_empty() { f[csm] } else { f[cxm] };
            let (leap, m) = match lab.strip_prefix('闰') {
                Some(x) => (true, x),
                None => (false, lab),
            };
            let (jdn, basis) = match f[cxj].parse::<i64>() {
                Ok(j) => (j, XixiaBasis::Original),
                Err(_) => (f[csj].parse().expect("宋朔JDN"), XixiaBasis::SongFallback),
            };
            XixiaMonth { jdn, year: f[cy].parse().expect("公元年"), month: m.parse().expect("月"), leap, basis }
        })
        .collect();
    out.sort_by_key(|m| m.jdn);
    out
}

/// 西夏某年各月 (朔日 JDN, 月, 闰否)；年不在 1120—1207 返回空。
pub fn year_months(year: i32) -> Vec<(i64, u8, bool)> {
    months().iter().filter(|m| m.year == year).map(|m| (m.jdn, m.month, m.leap)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::gan_zhi::GAN_ZHI_60;

    fn gz(j: i64) -> &'static str {
        GAN_ZHI_60[(j + 49).rem_euclid(60) as usize]
    }

    /// 以宋历代缺月、与相邻原件月相接而成 28 或 31 日之处
    const SEAMS: usize = 1; // 1193 年九月（宋甲子代）至十月（夏乙未，晚宋一日）31 日

    #[test]
    fn table_shape() {
        let m = months();
        assert_eq!(m.len(), 1088, "1120—1207 共 1088 月");
        assert_eq!(m.iter().filter(|x| x.basis == XixiaBasis::SongFallback).count(), 126);
        // 两端都是原件朔日者必 29 或 30 日；一端以宋历代者在接缝处可成 28 或 31 日
        let mut seams = 0;
        for w in m.windows(2) {
            let d = w[1].jdn - w[0].jdn;
            if w[0].basis == XixiaBasis::Original && w[1].basis == XixiaBasis::Original {
                assert!((29..=30).contains(&d), "{:?}", w);
            } else {
                assert!((28..=31).contains(&d), "{:?}", w);
                seams += !(29..=30).contains(&d) as usize;
            }
        }
        assert_eq!(seams, SEAMS);
        for y in 1120..=1207 {
            let n = year_months(y).len();
            assert!(n == 12 || n == 13, "{y}");
        }
    }

    /// 书页 64、512：天庆十二年（1205）二月夏戊子（宋己丑）、五月夏戊午（宋丁巳），夏闰九月而宋闰八月
    #[test]
    fn tianqing_12() {
        let y = year_months(1205);
        let f = |m: u8, l: bool| gz(y.iter().find(|e| e.1 == m && e.2 == l).unwrap().0);
        assert_eq!((f(2, false), f(5, false), f(9, true)), ("戊子", "戊午", "甲申"));
        assert!(!y.iter().any(|e| e.1 == 8 && e.2));
    }
}
