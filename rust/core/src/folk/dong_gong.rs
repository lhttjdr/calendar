//! 宜忌另一说：《董公選擇日要覽》（嘉庆二十二年蒋云抄传本，维基文库录文）。
//!
//! 董公以月为纲、以建除分条（每月十二条，一条管该支五日），条内泛论该支，再分举日干支。与协纪月表
//!（[super::yiji]）相合约五成五，神煞体系与吉凶相抵之法都不同，故作并列的另一说，不作协纪的修正，
//! 也不与协纪互相填补：董公无宜忌之文的日子只报「董公无宜忌之文」。对照与体例见 `doc/folk/tongshu_yiji.md`。
//!
//! 数据（皆 include_str! 编入）：
//! - `data/folk/dong_gong.csv`：原文表，144 条（月 × 建除），含删去应验语后的宜忌之文、原文全文、条目校记；
//! - `data/folk/dong_gong_days.csv`：逐日判定表（依据层，人工逐条校订），一行一日一事：事目用董公自己的话，
//!   「諸事」为泛言（「百事不宜」「次吉」等不举事项者）；有等第、所据原句、条件（如「水土生人」「小小營為」）、校记；
//! - `data/folk/dong_gong_events.csv`：董公事目 → 协纪事项（一对多），只供对照，不参与判定。
//!
//! 月建按节（与协纪同），本模块只收月建支与日干支。董公所举神煞（煞集中宮、紅沙、黃羅紫檀、到州星等，
//! 多为协纪月表所不立）只作董公说的缘由列出，不并入协纪神煞表（[super::shensha]）。

use once_cell::sync::Lazy;

const ENTRY_CSV: &str = include_str!("../../../../data/folk/dong_gong.csv");
const DAYS_CSV: &str = include_str!("../../../../data/folk/dong_gong_days.csv");
const EVENTS_CSV: &str = include_str!("../../../../data/folk/dong_gong_events.csv");
const GAN: &str = "甲乙丙丁戊己庚辛壬癸";
const ZHI: &str = "子丑寅卯辰巳午未申酉戌亥";

/// 无判定之日的说明（不回退协纪）
pub const NO_TEXT: &str = "董公无宜忌之文";

/// 董公所举神煞（含协纪亦立者）；长名在前，匹配后从文中抹去以免短名重复计入。
/// 煞集、煞入、殺集、六煞入中宮同指一事，统称「煞集中宮」。
const GODS: &[(&str, &str)] = &[
    ("煞集中宮", "煞集中宮"),
    ("殺集中宮", "煞集中宮"),
    ("六煞入中宮", "煞集中宮"),
    ("煞入中宮", "煞集中宮"),
    ("煞神聚入中宮", "煞集中宮"),
    ("白虎入中宮", "白虎入中宮"),
    ("小紅沙", "小紅沙"),
    ("紅沙", "紅沙"),
    ("黃沙", "黃沙"),
    ("沙黃", "黃沙"),
    ("黃羅紫檀", "黃羅紫檀"),
    ("黃羅、紫檀", "黃羅紫檀"),
    ("天皇地皇", "天皇地皇"),
    ("天皇星地皇星", "天皇地皇"),
    ("金銀庫樓", "金銀庫樓"),
    ("玉堂聚寶", "玉堂聚寶"),
    ("到州星", "到州星"),
    ("朱雀勾絞", "朱雀勾絞"),
    ("勾絞", "勾絞"),
    ("天地轉煞", "天地轉煞"),
    ("天地轉殺", "天地轉煞"),
    ("轉煞", "天地轉煞"),
    ("天轉地軸", "天轉地軸"),
    ("正四廢", "正四廢"),
    ("天月二德", "天月二德"),
    ("天德", "天德"),
    ("月德", "月德"),
    ("天赦", "天赦"),
    ("天喜", "天喜"),
    ("天富", "天富"),
    ("天成", "天成"),
    ("天賊", "天賊"),
    ("天羅", "天羅"),
    ("地網", "地網"),
    ("往亡", "往亡"),
    ("天空", "天空"),
    ("月厭", "月厭"),
    ("福星", "福星"),
    ("福生", "福生"),
    ("火星", "火星"),
    ("文昌", "文昌"),
    ("九土鬼", "九土鬼"),
    ("土鬼", "土鬼"),
    ("猖鬼敗亡", "猖鬼敗亡"),
    ("十惡", "十惡"),
    ("重喪", "重喪"),
    ("六甲窮日", "六甲窮日"),
    ("五行無氣", "五行無氣"),
    ("陰府決遣", "陰府決遣"),
    ("螣蛇", "螣蛇"),
    ("黑煞", "黑煞"),
    ("受死", "受死"),
    ("偷修", "玄女偷修"),
    ("土瘟", "土瘟"),
    ("天瘟", "天瘟"),
    ("官符", "官符"),
];

/// 一日一事之判定（`dong_gong_days.csv` 一行）
#[derive(Clone, Debug)]
pub struct DongGongRule {
    /// 董公事目（起造、婚姻、安葬……）；「諸事」为泛言
    pub event: &'static str,
    /// 宜为 true，忌为 false
    pub yi: bool,
    /// 等第（大吉、上吉、次吉、又次吉、更凶、大凶……）；原文无则空
    pub grade: &'static str,
    /// 所据原句
    pub basis: &'static str,
    /// 条件（「水土生人」「小小營為」等）；无则空
    pub condition: &'static str,
    /// 校记
    pub note: &'static str,
}

/// 某月某日董公之说
#[derive(Clone, Debug)]
pub struct DongGongDay {
    /// 建除（建、除、滿……）
    pub jian_chu: &'static str,
    /// 本条宜忌之文（删去应验语）
    pub text: &'static str,
    /// 本条原文全文
    pub full_text: &'static str,
    /// 条目校记
    pub entry_note: &'static str,
    /// 判定；空表示「董公无宜忌之文」
    pub rules: Vec<DongGongRule>,
    /// 董公为此日所举神煞（自判定所据原句，无判定则自原文中举此干支之句），去重
    pub gods: Vec<&'static str>,
}

impl DongGongDay {
    /// 无宜忌之文
    pub fn no_text(&self) -> bool {
        self.rules.is_empty()
    }
}

struct Entry {
    jian_chu: &'static str,
    text: &'static str,
    full: &'static str,
    note: &'static str,
}

fn fields(line: &'static str) -> Vec<&'static str> {
    // 表内不含半角引号与半角逗号（原文标点为全角），按逗号直接切分
    assert!(!line.contains('"'), "{line}");
    line.split(',').collect()
}

fn gz_index(name: &str) -> u8 {
    let g = GAN.chars().position(|c| name.starts_with(c)).expect("干");
    let z = ZHI.chars().position(|c| name.ends_with(c)).expect("支");
    (0..60u8).find(|k| *k as usize % 10 == g && *k as usize % 12 == z).expect("干支")
}

fn gz_name(k: u8) -> String {
    let g = GAN.chars().nth(k as usize % 10).unwrap();
    let z = ZHI.chars().nth(k as usize % 12).unwrap();
    format!("{g}{z}")
}

/// 144 条：下标 = 月序（寅月 = 0）× 12 + 日支
static ENTRIES: Lazy<Vec<Entry>> = Lazy::new(|| {
    let mut v: Vec<Option<Entry>> = (0..144).map(|_| None).collect();
    let mut lines = ENTRY_CSV.lines();
    let head = fields(lines.next().unwrap());
    assert_eq!(head, ["月", "月建", "建除", "日支", "宜忌之文", "原文", "校"]);
    for line in lines.filter(|l| !l.trim().is_empty()) {
        let f = fields(line);
        let m: usize = f[0].parse().expect("月");
        let z = ZHI.chars().position(|c| f[3].starts_with(c)).expect("日支");
        v[(m - 1) * 12 + z] = Some(Entry { jian_chu: f[2], text: f[4], full: f[5], note: f[6] });
    }
    v.into_iter().map(|e| e.expect("董公缺条")).collect()
});

/// 720 日：下标 = 月序 × 60 + 干支序
static DAYS: Lazy<Vec<Vec<DongGongRule>>> = Lazy::new(|| {
    let mut v: Vec<Vec<DongGongRule>> = vec![Vec::new(); 720];
    let mut lines = DAYS_CSV.lines();
    let head = fields(lines.next().unwrap());
    assert_eq!(head, ["月", "干支", "建除", "事", "宜忌", "等第", "泛言", "依据", "条件", "校"]);
    for line in lines.filter(|l| !l.trim().is_empty()) {
        let f = fields(line);
        let m: usize = f[0].parse().expect("月");
        let k = gz_index(f[1]) as usize;
        let yi = match f[4] {
            "宜" => true,
            "忌" => false,
            x => panic!("宜忌：{x}"),
        };
        v[(m - 1) * 60 + k].push(DongGongRule { event: f[3], yi, grade: f[5], basis: f[7], condition: f[8], note: f[9] });
    }
    v
});

/// 董公事目 → 协纪事项
static EVENTS: Lazy<Vec<(&'static str, Vec<&'static str>)>> = Lazy::new(|| {
    EVENTS_CSV
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let f = fields(l);
            (f[0], f[2].split_whitespace().collect())
        })
        .collect()
});

fn gods_in(texts: &[&str]) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for t in texts {
        let mut s = t.to_string();
        for (pat, name) in GODS {
            if s.contains(pat) {
                s = s.replace(pat, "\u{3000}");
                if !out.contains(name) {
                    out.push(name);
                }
            }
        }
    }
    out
}

/// 某月某日董公之宜忌与缘由。`month_zhi`：月建支（子 = 0，寅月 = 2；月以节为界）；`day_gz`：日干支序（甲子 = 0）。
/// 董公无宜忌之文时 `rules` 为空（见 [NO_TEXT]），不回退协纪。
pub fn dong_gong_day(month_zhi: u8, day_gz: u8) -> DongGongDay {
    let mo = (month_zhi as usize + 10) % 12;
    let k = day_gz % 60;
    let e = &ENTRIES[mo * 12 + k as usize % 12];
    let rules = DAYS[mo * 60 + k as usize].clone();
    let gods = if rules.is_empty() {
        // 原文中举此干支之句；条内无之，则取条首（泛论该支）
        let name = gz_name(k);
        let sents: Vec<&str> = e.full.split(['。', '！']).filter(|s| s.contains(name.as_str())).collect();
        if sents.is_empty() {
            gods_in(&[e.full.split(['。', '！']).next().unwrap_or("")])
        } else {
            gods_in(&sents)
        }
    } else {
        gods_in(&rules.iter().map(|r| r.basis).collect::<Vec<_>>())
    };
    DongGongDay { jian_chu: e.jian_chu, text: e.text, full_text: e.full, entry_note: e.note, rules, gods }
}

/// 董公事目对应的协纪事项（「諸事」返回 `["*"]`；协纪无对应者为空）
pub fn xieji_events(dong_event: &str) -> &'static [&'static str] {
    EVENTS.iter().find(|(e, _)| *e == dong_event).map(|(_, v)| v.as_slice()).unwrap_or(&[])
}

#[cfg(test)]
mod tests {
    use super::*;

    const YIN: u8 = 2; // 寅月 = 正月

    fn day(month: u8, gz: &str) -> DongGongDay {
        // month：正月 = 1
        dong_gong_day((month + 1) % 12, gz_index(gz))
    }

    fn has(d: &DongGongDay, ev: &str, yi: bool) -> bool {
        d.rules.iter().any(|r| r.event == ev && r.yi == yi)
    }

    #[test]
    fn tables_complete() {
        assert_eq!(ENTRIES.len(), 144);
        let judged = DAYS.iter().filter(|d| !d.is_empty()).count();
        assert!((600..720).contains(&judged), "{judged}");
        // 事目都在映射表中
        for d in DAYS.iter() {
            for r in d {
                assert!(EVENTS.iter().any(|(e, _)| *e == r.event), "{}", r.event);
            }
        }
        assert_eq!(xieji_events("婚姻"), ["結婚姻", "納采問名", "嫁娶"]);
        assert_eq!(xieji_events("諸事"), ["*"]);
        assert!(xieji_events("生基").is_empty());
    }

    /// 正月危酉：「辛酉正四廢不宜用事，惟丁酉有天德福星蓋照、宜安葬還福愿、出行開張、參官見貴吉。
    /// 只不宜起造及婚姻嫁娶等事……餘酉日均不可用。」
    #[test]
    fn zheng_yue_wei_you() {
        let d = dong_gong_day(YIN, gz_index("丁酉"));
        assert_eq!(d.jian_chu, "危");
        for ev in ["安葬", "還愿", "出行", "開張", "上官", "見貴"] {
            assert!(has(&d, ev, true), "{ev}");
        }
        assert!(has(&d, "起造", false) && has(&d, "婚姻", false));
        assert!(d.gods.contains(&"天德") && d.gods.contains(&"福星"));
        let x = day(1, "辛酉");
        assert!(has(&x, "諸事", false));
        assert!(x.gods.contains(&"正四廢"));
        assert!(has(&day(1, "癸酉"), "諸事", false), "餘酉日均不可用");
    }

    /// 正月開子：「戊子、丙子、庚子三日惟水土生人用之大吉」——条件之宜
    #[test]
    fn zheng_yue_kai_zi_condition() {
        let d = day(1, "丙子");
        let r = d.rules.iter().find(|r| r.event == "諸事").unwrap();
        assert!(r.yi && r.grade == "大吉" && r.condition == "水土生人");
        let d = day(1, "壬子");
        for ev in ["起造", "婚姻", "入宅", "開張"] {
            assert!(has(&d, ev, false), "{ev}");
        }
    }

    /// 二月定未：「是月惟癸未一日……大吉。餘未具不利」；二月平午「只宜作生基，如婚姻、修造等事用之……」
    #[test]
    fn er_yue() {
        let d = day(2, "癸未");
        assert!(d.rules.iter().any(|r| r.event == "諸事" && r.yi && r.grade == "大吉"));
        assert!(has(&day(2, "丁未"), "諸事", false));
        let d = day(2, "丙午");
        assert!(has(&d, "生基", true) && has(&d, "婚姻", false) && has(&d, "修造", false));
        // 二月滿巳「不宜動土，如修造百事俱吉」
        let d = day(2, "乙巳");
        assert!(has(&d, "動土", false) && has(&d, "修造", true));
    }

    /// 五月滿申：「壬申西沉之日、五行無氣不可用」（对照稿只判安葬，校订为诸事）
    #[test]
    fn wu_yue_man_shen() {
        assert!(has(&day(5, "壬申"), "諸事", false));
        let d = day(5, "庚申");
        assert!(has(&d, "安葬", true) && has(&d, "修造", false) && has(&d, "入宅", false));
        let d = day(5, "甲申");
        assert!(has(&d, "動土", false) && d.rules.iter().any(|r| r.event == "婚姻" && r.yi && r.grade == "次吉"));
    }

    /// 七月建申：「戊申天赦」只举神煞，无宜忌之文；不回退协纪
    #[test]
    fn no_text_day() {
        let d = day(7, "戊申");
        assert!(d.no_text());
        assert!(d.gods.contains(&"天赦"), "{:?}", d.gods);
        assert!(d.full_text.starts_with("戊申天赦"));
    }

    /// 正月滿辰：「甲辰雖有氣，卻與戊辰同煞集中宮，百事皆忌」——董公独有神作缘由
    #[test]
    fn dong_only_gods() {
        let d = day(1, "甲辰");
        assert!(has(&d, "諸事", false));
        assert!(d.gods.contains(&"煞集中宮"), "{:?}", d.gods);
        // 正月定午：黃沙、黃羅紫檀诸吉星
        let d = day(1, "甲午");
        assert!(has(&d, "起造", true) && has(&d, "安葬", true));
    }

    /// 十二月：開亥「辛亥陰氣太甚、非陽間所用」；閉子「甲子天赦是進神並戊子宜小可修、為吉。若大用之則凶禍纏繞」
    #[test]
    fn shi_er_yue() {
        assert!(has(&day(12, "辛亥"), "諸事", false));
        assert!(has(&day(12, "乙亥"), "婚姻", true));
        let d = day(12, "甲子");
        assert!(d.rules.iter().any(|r| r.event == "修造" && r.yi && r.condition == "小可修"));
        assert!(has(&d, "諸事", false));
        assert_eq!(day(12, "甲子").jian_chu, "閉");
    }

    /// 月建按支：寅月 = 2、丑月 = 1
    #[test]
    fn month_index() {
        assert_eq!(dong_gong_day(2, gz_index("丙寅")).jian_chu, "建");
        assert_eq!(dong_gong_day(1, gz_index("乙丑")).jian_chu, "建");
        assert_eq!(dong_gong_day(0, gz_index("丙子")).jian_chu, "建");
    }
}
