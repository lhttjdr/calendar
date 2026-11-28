//! 秦至汉初实用历日推步（第 4 级，按术推算）：四分术平朔（与颛顼历同一历元、同一朔策 27759/940 日、同一置闰），
//! 合朔时刻加一常数改正 δ（以 1/940 日计），δ **逐段由出土历谱、质日定出**，不取今人复原历的参数。
//!
//! 为什么不直接按颛顼历推：张培瑜等《中国古代历法》（2013）第三章第九、十节（书页 240—249）已指出，出土秦汉历日的朔
//! 普遍比颛顼历推步晚，汉初大 430—500 分、秦大 639—649 分，节气则与颛顼历同（[super::zhuanxu] 模块说明）。颛顼历推步
//! 合《汉书》本纪书朔、晦 18 条中 7 条，合实物 40% 左右，不能代表当时实用之历。
//!
//! **δ 怎么定**（`data/calendar/qinhan_shiwu_months.csv` 逐月转录，`examples/qinhan_shiwu_check.rs` 逐月求可行区间，
//! 结果见 doc/calendar/qinhan-shiwu-checks.md）：某月实物朔日为 D、颛顼平朔距历元 t（1/940 日），则 δ 须在
//! [(D − 历元)·940 − t, 同 + 940)；同一段内各月区间之交即该段 δ 的可行区间，取其中点。各段（[SEGMENTS]）：
//! - 秦（秦王政至汉高祖四年）一段 [639, 650)（取 644）：官文书（中央、洞庭郡、南郡、南阳郡）一个相位全合（2026-11-26，
//!   `scripts/calendar_data/qin_phase_check.py`，doc/calendar/qin-phase-checks.md）——张家山《奏谳书》秦王政二年十月癸酉朔、
//!   六年八月丙子朔（≥ 553），里耶秦简廿五至卅五年纪年文书（《里耶秦簡（壹）索引稿》曆日索引 435 条：卅一年四月癸未朔 8-736 等定下界 638，
//!   卅四年十一月丁卯朔 8-1220+8-886 定上界 649），岳麓秦简《质日》三十四、三十五年，周家台秦简《历谱》三十六、三十七年、二世元年（二世元年十月乙亥要求 ≥ 639，
//!   官文书本身之交为 [638, 650)），
//!   与张培瑜书页 249 引其 2007 年文「秦历朔小余比颛顼历大 639～649」同。旧作两段（卅年以前 [429, 533)）的上界只来自岳麓《质日》
//!   二十七年八月癸酉，而同年里耶 8-133「廿七年八月甲戌朔壬辰」要求 ≥ 533，8-63「廿六年三月……辛亥」要求 ≥ 556——两段之界不是里耶所见，
//!   是岳麓《质日》（购藏，出土地不明）与官文书之别，故合为一段；岳麓二十七年六、八月与周家台三十四年三、五、七月作已知歧异；
//! - 汉高祖五年（前 202）至文帝后元前一年（前 164）：张家山汉简《历谱》高祖五年四月至吕后二年 17 年 → [491, 501)；
//! - 文帝后元元年（前 163）至太初元年四月：银雀山《元光元年历谱》13 月（转引张培瑜表 3-20） → [430, 488)。
//!
//! **段界**：秦与汉初之界没有实物：二世元年（前 209）以后至高祖五年四月以前无历日，
//! 取张家山《历谱》起始之高祖五年（前 202）——沿用前段至新段首见之年，与下一段同一原则。汉初两段的界年也没有实物：吕后二年（前 186）至元光元年（前 134）之间改过一次，
//! 取文帝后元元年，因张培瑜（书页 242）据史载日月食与后九月推定置闰标准在文帝后元前后改动（[super::zhuanxu::YUSHUI_RULE_FROM]），
//! 历法改动当在同时——这是推断，不是记载。段界与区间宽度会影响的月份，逐月在检验文档里标「未定」。
//!
//! **史载例外**：汉高祖九年七月按术丙申朔，荆州胡家草场汉简《岁纪》「九年，七月，以丙申朔，朔日食，更以丁酉」，
//! 张家山《历谱》亦书「七月丁酉」——朔日见食而改。
//!
//! 第 4 层 [super::lifa] 主序列「颛顼历（秦汉初）」「太初历（太初元年，五月改历）」两段按术推步即取本模块
//! （[super::push_calendar::reckon_lifa_year]）；纯颛顼历推步仍在 [super::zhuanxu]，供对照与战国秦国。

use super::zhuanxu::{jianhai_months_to_xiazheng, shiyue_index, EPOCH_JDN, LUNAR, U};

/// 一段：自建亥年 `from`（含）起行用，δ 可行区间 `[lo, hi)`（1/940 日），定参数所据之实物。
#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub from: i32,
    pub lo: i64,
    pub hi: i64,
    pub basis: &'static str,
}

impl Segment {
    /// 本段所取 δ：可行区间的中点。
    pub fn delta(&self) -> i64 {
        (self.lo + self.hi - 1) / 2
    }
}

/// 各段，按 `from` 递增；第一段向前延伸至秦统一（本模块覆盖 [YEARS]），统一前秦国推步（[super::push_calendar] 「秦」秦王政以后）亦取第一段之 δ。
pub const SEGMENTS: &[Segment] = &[
    Segment { from: -230, lo: 639, hi: 650, basis: "里耶秦简廿五至卅五年纪年文书（《里耶秦簡（壹）索引稿》曆日索引）；张家山《奏谳书》秦王政二年、六年；岳麓秦简《质日》三十四、三十五年；周家台秦简《历谱》三十六、三十七年、二世元年" },
    Segment { from: -201, lo: 491, hi: 501, basis: "张家山汉简《历谱》高祖五年至吕后二年" },
    Segment { from: -162, lo: 430, hi: 488, basis: "银雀山汉简《元光元年历谱》（转引）" },
];

/// 覆盖的夏正年（与 [super::qinhan_restore::QINHAN_YEARS] 同起，至太初元年；太初元年只用其正月至四月）。
pub const YEARS: (i32, i32) = (-220, -103);

/// 史载例外：(建亥年, 月, 闰, 朔日 JDN, 依据)。
pub const EXCEPTIONS: &[(i32, u8, bool, i64, &str)] = &[(
    -197,
    7,
    false,
    1_649_324,
    "胡家草场汉简《岁纪》「九年，七月，以丙申朔，朔日食，更以丁酉」；张家山汉简《历谱》高祖九年「七月丁酉」",
)];

/// 建亥年 n 所属之段。
pub fn segment(n: i32) -> &'static Segment {
    SEGMENTS
        .iter()
        .rev()
        .find(|s| n >= s.from)
        .unwrap_or(&SEGMENTS[0])
}

/// 第 i 个平朔（自颛顼历元起）加改正 δ 后所在之日。
fn shuo_day(i: i64, delta: i64) -> i64 {
    EPOCH_JDN + (i * LUNAR + delta * (U / 940)).div_euclid(U)
}

/// 建亥年 n 各月：(朔日 JDN, 月, 是否后九月)，自十月起。置闰同颛顼历推步（[super::zhuanxu::jianhai_year_months]）。
pub fn jianhai_year_months_with(n: i32, delta: i64) -> Vec<(i64, u8, bool)> {
    const LABELS: [u8; 12] = [10, 11, 12, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let (a, b) = (shiyue_index(n), shiyue_index(n + 1));
    (a..b)
        .enumerate()
        .map(|(j, i)| {
            let (m, leap) = if j < 12 {
                (LABELS[j], false)
            } else {
                (9, true)
            };
            let day = EXCEPTIONS
                .iter()
                .find(|e| (e.0, e.1, e.2) == (n, m, leap))
                .map_or_else(|| shuo_day(i, delta), |e| e.3);
            (day, m, leap)
        })
        .collect()
}

/// 建亥年 n 各月（取本段 δ）。
pub fn jianhai_year_months(n: i32) -> Vec<(i64, u8, bool)> {
    jianhai_year_months_with(n, segment(n).delta())
}

/// 夏正年 y 各月 (朔日 JDN, 月, 闰否)，后九月记作闰九月；y 在 [YEARS] 外返回 None。
pub fn year_months(lunar_year: i32) -> Option<Vec<(i64, u8, bool)>> {
    (YEARS.0..=YEARS.1)
        .contains(&lunar_year)
        .then(|| jianhai_months_to_xiazheng(lunar_year, jianhai_year_months))
}

/// 建亥年 n 第 j 月（自十月起）的朔日在本段 δ 可行区间内是否唯一（区间两端推得同日）。
pub fn pinned_within_segment(n: i32, j: usize) -> bool {
    let s = segment(n);
    let a = jianhai_year_months_with(n, s.lo);
    let b = jianhai_year_months_with(n, s.hi - 1);
    a[j].0 == b[j].0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::gan_zhi::GAN_ZHI_60;

    /// 出土历谱、质日各月朔日（`qinhan_shiwu_months.csv`，整理者补字与转引也在内）：除下列已知的实物歧异外逐月相合。
    /// 歧异：周家台秦始皇三十四年三、五、七月（与同年岳麓《质日》差一日，本推步合岳麓）；岳麓《质日》秦始皇二十七年六、八月
    /// （要求 δ ≤ 590、≤ 532；同年里耶 8-133「廿七年八月甲戌朔壬辰」要求 ≥ 533——秦一段 δ 依官文书一相位全合而定，
    /// 本推步合里耶，见 doc/calendar/qin-phase-checks.md）；岳麓三十四年「後九月壬辰小」
    /// 与三十五年「十月小」「十一月辛卯大」相差一日（本推步十月壬戌，合三十五年质日）。
    #[test]
    fn matches_excavated_calendars() {
        let csv = include_str!("../../../../data/calendar/qinhan_shiwu_months.csv");
        let mut lines = csv.lines().filter(|l| !l.starts_with('#'));
        let head: Vec<&str> = lines.next().unwrap().split(',').collect();
        let col = |n: &str| head.iter().position(|c| *c == n).unwrap();
        let (cs, cy, cm, cl, cg) = (col("来源"), col("年"), col("月"), col("闰"), col("朔干支"));
        let known = [
            ("周家台", -212, 3),
            ("周家台", -212, 5),
            ("周家台", -212, 7),
            ("岳麓", -219, 6),
            ("岳麓", -219, 8),
        ];
        let (mut ok, mut bad) = (0, Vec::new());
        for l in lines {
            let f: Vec<&str> = l.split(',').collect();
            let (src, n, m, leap, g) = (
                f[cs],
                f[cy].parse::<i32>().unwrap(),
                f[cm].parse::<u8>().unwrap(),
                f[cl] == "1",
                f[cg],
            );
            let ms = jianhai_year_months(n);
            let d = ms
                .iter()
                .find(|x| (x.1, x.2) == (m, leap))
                .unwrap_or_else(|| panic!("{src} {n} 年无 {m} 月（闰 {leap}）"));
            if GAN_ZHI_60[(d.0 + 49).rem_euclid(60) as usize] == g {
                ok += 1;
            } else {
                bad.push((src.to_string(), n, m));
            }
        }
        let bad_known: Vec<_> = bad
            .iter()
            .filter(|b| known.contains(&(b.0.as_str(), b.1, b.2)))
            .collect();
        assert_eq!(bad.len(), bad_known.len(), "实物不合：{bad:?}");
        assert_eq!(bad.len(), 5);
        assert!(ok > 250, "{ok}");
    }

    /// 各段 δ 落在可行区间内；例外月的朔日干支为丁酉。
    #[test]
    fn segments_and_exception() {
        for s in SEGMENTS {
            assert!(s.lo <= s.delta() && s.delta() < s.hi);
        }
        assert_eq!(
            GAN_ZHI_60[(EXCEPTIONS[0].3 + 49).rem_euclid(60) as usize],
            "丁酉"
        );
        // 无例外时按术为丙申
        let i = shiyue_index(-197) + 9;
        assert_eq!(
            GAN_ZHI_60[(shuo_day(i, segment(-197).delta()) + 49).rem_euclid(60) as usize],
            "丙申"
        );
    }

    /// 月长 29、30 日；置闰与颛顼历推步逐年相同。
    #[test]
    fn year_shapes() {
        for n in -220..=-104 {
            let a = jianhai_year_months(n);
            let b = super::super::zhuanxu::jianhai_year_months(n);
            assert_eq!(
                a.iter().map(|x| (x.1, x.2)).collect::<Vec<_>>(),
                b.iter().map(|x| (x.1, x.2)).collect::<Vec<_>>()
            );
            let next = jianhai_year_months(n + 1)[0].0;
            for (k, w) in a.iter().enumerate() {
                let end = a.get(k + 1).map_or(next, |x| x.0);
                assert!(
                    end - w.0 == 29 || end - w.0 == 30,
                    "{n} 年第 {k} 月 {}",
                    end - w.0
                );
            }
        }
    }
}
