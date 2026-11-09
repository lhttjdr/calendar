//! 宜日另一说：《增補鰲頭通書大全》（乾隆丙午重梓本，上海廣益書局石印）各门逐月吉日表。
//!
//! 鳌头以事为纲，每门列「逐月某事吉日」（正选与「外」日），表尾注所避诸神。本模块只做查表：某月某日见于哪几门的吉日表，
//! 不推神煞、不判忌——鳌头不出逐日宜忌总表，表中未收之日只说「鳌头不收」，不等于忌，也不回退协纪。
//! 与协纪月表（[super::yiji]，缺省）并列，不取舍；逐日对照见 `doc/folk/aotou_men_checks.md`、`doc/folk/tongshu_yiji.md` 第十节。
//! 神煞起例的鳌头一说见 [super::shensha::ShenshaOptions::aotou]。
//!
//! 数据（书影逐页转录，include_str! 编入；「页」为 `doc/references/folk/通書/鰲頭通書大全/第N冊.pdf` 的 PDF 页序）：
//! - 卷二：`aotou_chuxing.csv`（出行）、`aotou_xingchuan.csv`（行船）、`aotou_kaizhang.csv`（開張店肆）、`aotou_liqi.csv`（立契）、
//!   `aotou_ruxue.csv`（入學）、`aotou_shangguan.csv`（上官赴任）；
//! - 卷三：`aotou_jiaqu_days.csv`（求婚、下定結婚、嫁娶、嫁娶備急）、`aotou_dongtu.csv`（平基動土）、`aotou_qigong.csv`（起工）、
//!   `aotou_shuzhu.csv`（豎柱上梁）、`aotou_gaiwu.csv`（蓋屋）、`aotou_ruzhai.csv`（入宅歸火）；
//! - 卷四：`aotou_zhancao.csv`（斬草）、`aotou_anzang_zhuyue.csv`（安葬）。
//!
//! 月为月建（按节，寅月为正月），与协纪月表同。

use once_cell::sync::Lazy;

const GAN: &str = "甲乙丙丁戊己庚辛壬癸";
const ZHI: &str = "子丑寅卯辰巳午未申酉戌亥";

/// (简称, 协纪对应事项, CSV)
const TABLES: &[(&str, &str, &str)] = &[
    ("出行", "出行", include_str!("../../../../data/folk/aotou_chuxing.csv")),
    ("行船", "乘船渡水", include_str!("../../../../data/folk/aotou_xingchuan.csv")),
    ("開張", "開市", include_str!("../../../../data/folk/aotou_kaizhang.csv")),
    ("立契", "立券交易", include_str!("../../../../data/folk/aotou_liqi.csv")),
    ("入學", "入學", include_str!("../../../../data/folk/aotou_ruxue.csv")),
    ("上官", "上官赴任", include_str!("../../../../data/folk/aotou_shangguan.csv")),
    ("婚姻", "", include_str!("../../../../data/folk/aotou_jiaqu_days.csv")),
    ("動土", "修造動土", include_str!("../../../../data/folk/aotou_dongtu.csv")),
    ("起工", "修造動土", include_str!("../../../../data/folk/aotou_qigong.csv")),
    ("豎柱上梁", "豎柱上梁", include_str!("../../../../data/folk/aotou_shuzhu.csv")),
    ("蓋屋", "修造動土", include_str!("../../../../data/folk/aotou_gaiwu.csv")),
    ("入宅", "移徙", include_str!("../../../../data/folk/aotou_ruzhai.csv")),
    ("斬草", "破土", include_str!("../../../../data/folk/aotou_zhancao.csv")),
    ("安葬", "安葬", include_str!("../../../../data/folk/aotou_anzang_zhuyue.csv")),
];

/// 婚姻四表各自对应的协纪事项
fn jiaqu_event(table: &str) -> &'static str {
    match table {
        "逐月求婚吉日" => "納采問名",
        "逐月下定結婚吉日" => "結婚姻",
        _ => "嫁娶",
    }
}

/// 某日见于鳌头某表
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AoTouJiRi {
    /// 事（简称；婚姻四表取表名去「逐月」「吉日」）
    pub shi: String,
    /// 原书表名
    pub table: String,
    /// 协纪对应事项（只供对照）
    pub xieji_event: &'static str,
    /// 正（正选）／外（「外」日）
    pub deng: String,
    /// 原书夹注（命宜、不將等）
    pub note: String,
    /// 出处：卷、册、页
    pub source: String,
}

struct Row {
    month: u8,
    gz: u8,
    item: AoTouJiRi,
}

fn gz_index(s: &str) -> Option<u8> {
    let mut it = s.chars();
    let (a, b) = (it.next()?, it.next()?);
    let g = GAN.chars().position(|c| c == a)?;
    let z = ZHI.chars().position(|c| c == b)?;
    (0..60u8).find(|&i| (i % 10) as usize == g && (i % 12) as usize == z)
}

static ROWS: Lazy<Vec<Row>> = Lazy::new(|| {
    let mut out: Vec<Row> = Vec::new();
    for (short, ev, csv) in TABLES {
        for l in csv.lines().skip(1) {
            // 表,卷,门,册,页,月,干支,等,注,原文,校（录文无半角逗号）
            let f: Vec<&str> = l.split(',').collect();
            let (Ok(m), Some(gz)) = (f[5].parse::<u8>(), gz_index(f[6])) else { continue };
            if m == 0 {
                continue;
            }
            let (shi, xe) = if *short == "婚姻" {
                (f[0].trim_start_matches("逐月").trim_end_matches("吉日").to_string(), jiaqu_event(f[0]))
            } else {
                (short.to_string(), *ev)
            };
            // 同表同月同日重出者只收一次
            if out.iter().any(|r| r.month == m - 1 && r.gz == gz && r.item.table == f[0]) {
                continue;
            }
            out.push(Row {
                month: m - 1,
                gz,
                item: AoTouJiRi {
                    shi,
                    table: f[0].to_string(),
                    xieji_event: xe,
                    deng: f[7].to_string(),
                    note: f[8].to_string(),
                    source: format!("鳌头{}，第 {} 册第 {} 页", f[1], f[3], f[4]),
                },
            });
        }
    }
    out
});

/// 某月（`month_ord`：寅月 = 0）某日（干支序 `day_gz`）见于鳌头哪些门的逐月吉日表
pub fn jiri_for_day(month_ord: u8, day_gz: u8) -> Vec<AoTouJiRi> {
    let m = month_ord % 12;
    ROWS.iter().filter(|r| r.month == m && r.gz == day_gz).map(|r| r.item.clone()).collect()
}

/// 各表所收日数（去重），供核对
pub fn table_sizes() -> Vec<(String, usize)> {
    let mut v: Vec<(String, usize)> = Vec::new();
    for r in ROWS.iter() {
        match v.iter_mut().find(|(t, _)| *t == r.item.table) {
            Some((_, n)) => *n += 1,
            None => v.push((r.item.table.clone(), 1)),
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 去重日数与对照脚本（scripts/folk/aotou_men_vs_xieji.py、aotou_vs_xieji.py）一致
    #[test]
    fn sizes() {
        let s = table_sizes();
        let get = |t: &str| s.iter().find(|(x, _)| x == t).map(|(_, n)| *n).unwrap_or(0);
        for (t, n) in [
            ("逐月出行吉日", 89),
            ("逐月行船吉日", 60),
            ("逐月開張店肆吉日", 95),
            ("逐月立契吉日", 101),
            ("逐月入學吉日", 110),
            ("逐月上官赴任吉日", 115),
            ("選定逐月平基動土吉日", 96),
            ("選定逐月起工吉日", 84),
            ("逐月豎柱上梁吉日", 115),
            ("逐月蓋屋吉日", 82),
            ("逐月入宅歸火吉日", 48),
            ("逐月斬草吉日", 72),
            ("逐月安葬吉日", 120),
            ("逐月求婚吉日", 105),
            ("逐月下定結婚吉日", 113),
            ("逐月嫁娶吉日", 53),
            ("逐月嫁娶結婚吉日備急用", 64),
        ] {
            assert_eq!(get(t), n, "{t}");
        }
    }

    #[test]
    fn lookup() {
        // 正月（寅月）辛未：出行正选、求婚、下定
        let v = jiri_for_day(0, gz_index("辛未").unwrap());
        let shi: Vec<&str> = v.iter().map(|x| x.shi.as_str()).collect();
        assert!(shi.contains(&"出行") && shi.contains(&"求婚"), "{shi:?}");
        let cx = v.iter().find(|x| x.shi == "出行").unwrap();
        assert_eq!((cx.deng.as_str(), cx.xieji_event), ("正", "出行"));
        assert!(cx.source.contains("第 3 册第 28 页"));
        // 十月安葬：酉日（开日）原书删去不载
        assert!((0..60u8).filter(|d| d % 12 == 9).all(|d| jiri_for_day(9, d).iter().all(|x| x.shi != "安葬")));
    }
}
