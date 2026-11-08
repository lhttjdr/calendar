//! 《周易》筮法：大衍揲蓍成爻、取辞之法（朱熹《易學啟蒙·考變占》），与卦爻辞原文。只排卦、给出应读之辞，不断吉凶。
//!
//! - 揲蓍：《周易·繫辭上》「大衍之數五十，其用四十有九，分而為二以象兩，掛一以象三，揲之以四以象四時，歸奇於扐以象閏，
//!   五歲再閏，故再扐而後掛」；三变成一爻，十八变成卦。
//! - 取辞：《易學啟蒙》卷四「考變占第四」（据元胡方平《易學啟蒙通釋》四库本所录原文，vendor/folk/wikisource/易學啟蒙通釋/卷下）。
//! - 卦爻辞：维基文库《周易》经文，见 [`text`]（scripts/folk/gen_zhouyi_text.py 生成）。
//! 本模块不取系统随机数：揲蓍由调用方给出每变左手所分之数（或所给随机整数），结果可复现。

#[path = "zhouyi_text.rs"]
pub mod text;

/// 卦（六位，初爻最低位，阳为 1）之经文行
fn row(g: u8) -> &'static (u8, &'static str, u8, &'static str, [&'static str; 6], Option<&'static str>) {
    text::GUA.iter().find(|r| r.2 == g & 63).unwrap()
}

/// 卦名
pub fn name(g: u8) -> &'static str {
    row(g).1
}

/// 大衍一变：所用之策 `total`（49、44 或 40……），左手所分 `left`（1 ≤ left ≤ total − 2，右手须余至少二策以掛一）。
/// 掛一于右，左右各揲之以四，余（奇）一至四（余零作四），归奇于扐。返回本变所去之策（掛一 + 左奇 + 右奇）。
pub fn dayan_change(total: u32, left: u32) -> Option<u32> {
    if left < 1 || left + 2 > total {
        return None;
    }
    let right = total - left - 1; // 掛一
    let r = |n: u32| if n % 4 == 0 { 4 } else { n % 4 };
    Some(1 + r(left) + r(right))
}

/// 三变成一爻：每变左手所分之数。返回爻数 6（老阴）、7（少阳）、8（少阴）、9（老阳），即余策 ÷ 4。
pub fn dayan_yao(lefts: [u32; 3]) -> Option<u8> {
    let mut total = 49;
    for l in lefts {
        total -= dayan_change(total, l)?;
    }
    Some((total / 4) as u8)
}

/// 由调用方所给随机整数定每变之分（left = 1 + r mod (total − 2)），三变成一爻。模块本身不取随机。
pub fn dayan_yao_from_random(r: [u32; 3]) -> u8 {
    let mut total = 49;
    for x in r {
        let left = 1 + x % (total - 2);
        total -= dayan_change(total, left).unwrap();
    }
    (total / 4) as u8
}

/// 大衍成爻之概率（以十六为分母）：设每变左手之余一至四等可能（通常之设），得 6:1、7:5、8:7、9:3（/16）。
/// 掷三钱（每钱阴阳各半）则为 6:2、7:6、8:6、9:2（/16，即 1/8、3/8、3/8、1/8）。
pub fn dayan_probabilities() -> [(u8, u32); 4] {
    // 第一变：左余 1—4 等可能，左右之余和须 ≡ 0 (mod 4)：左余四则去九，余则去五；第二、三变去四或八各半
    let mut cnt = [0u32; 4];
    for a in 1..=4u32 {
        let t1 = if a == 4 { 9 } else { 5 };
        for b in 1..=4u32 {
            let t2 = if b >= 3 { 8 } else { 4 };
            for c in 1..=4u32 {
                let t3 = if c >= 3 { 8 } else { 4 };
                let v = (49 - t1 - t2 - t3) / 4;
                cnt[(v - 6) as usize] += 1;
            }
        }
    }
    // 4×4×4 = 64 种等可能，约为十六分
    [(6, cnt[0] / 4), (7, cnt[1] / 4), (8, cnt[2] / 4), (9, cnt[3] / 4)]
}

/// 掷钱成爻之概率（以十六为分母）
pub fn coin_probabilities() -> [(u8, u32); 4] {
    [(6, 2), (7, 6), (8, 6), (9, 2)]
}

/// 所读之辞
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ci {
    /// 卦辞（彖辞，卦下之辞）
    GuaCi,
    /// 某爻辞（1—6，初至上）
    Yao(u8),
    /// 二用（乾用九、坤用六）
    Yong,
}

/// 辞之地位
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// 主
    Zhu,
    /// 并占（次）
    Bing,
    /// 贞（本卦／内卦）
    Zhen,
    /// 悔（之卦／外卦）
    Hui,
}

/// 一条应读之辞
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reading {
    pub gua: u8,
    pub ci: Ci,
    pub role: Role,
    pub text: &'static str,
}

/// 三爻变之主：《啟蒙》「前十卦主貞，後十卦主悔」，其次序依三十二图（原图未见）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SanBianZhu {
    /// 以《啟蒙》所举首末四卦（乾三爻变「自否至恒為前十卦，自益至泰為後十卦」）推定：前十卦即初爻在变爻之中者，主本卦（贞）；余主之卦（悔）。
    /// 由推求而得，未见原图，未核（缺省）
    #[default]
    ChuYaoZaiBian,
    /// 不分主次，本卦（贞）、之卦（悔）彖辞并列
    Both,
}

/// 取辞选项
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ZhanOptions {
    pub san_bian: SanBianZhu,
}

/// 取辞结果
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZhanCi {
    pub ben: u8,
    pub zhi: u8,
    /// 变爻（1—6）
    pub bian: Vec<u8>,
    /// 所依《啟蒙》原文
    pub rule: &'static str,
    pub readings: Vec<Reading>,
}

fn gua_ci(g: u8, role: Role) -> Reading {
    Reading { gua: g, ci: Ci::GuaCi, role, text: row(g).3 }
}
fn yao_ci(g: u8, k: u8, role: Role) -> Reading {
    Reading { gua: g, ci: Ci::Yao(k), role, text: row(g).4[k as usize - 1] }
}

/// 由六爻之数（初至上，6—9）定本卦、之卦与应读之辞（朱熹《易學啟蒙·考變占》）。
pub fn kao_bian_zhan(values: [u8; 6], o: ZhanOptions) -> ZhanCi {
    let mut ben = 0u8;
    let mut zhi = 0u8;
    let mut bian = Vec::new();
    for (k, &v) in values.iter().enumerate() {
        let yang = v == 7 || v == 9;
        let dong = v == 6 || v == 9;
        if yang {
            ben |= 1 << k;
        }
        if yang != dong {
            zhi |= 1 << k;
        }
        if dong {
            bian.push(k as u8 + 1);
        }
    }
    let bu_bian: Vec<u8> = (1..=6).filter(|k| !bian.contains(k)).collect();
    let (rule, readings): (&'static str, Vec<Reading>) = match bian.len() {
        0 => ("凡卦六爻皆不變則占本卦彖辭而以内卦為貞外卦為悔", vec![gua_ci(ben, Role::Zhu)]),
        1 => ("一爻變則以本卦變爻辭占", vec![yao_ci(ben, bian[0], Role::Zhu)]),
        2 => (
            "二爻變則以本卦二變爻辭占仍以上爻為主",
            vec![yao_ci(ben, bian[1], Role::Zhu), yao_ci(ben, bian[0], Role::Bing)],
        ),
        3 => {
            let (rz, rh) = match o.san_bian {
                SanBianZhu::Both => (Role::Zhen, Role::Hui),
                SanBianZhu::ChuYaoZaiBian => {
                    if bian.contains(&1) { (Role::Zhu, Role::Hui) } else { (Role::Zhen, Role::Zhu) }
                }
            };
            ("三爻變則占本卦及之卦之彖辭即以本卦為貞之卦為悔前十卦主貞後十卦主悔", vec![gua_ci(ben, rz), gua_ci(zhi, rh)])
        }
        4 => (
            "四爻變則以之卦二不變爻占仍以下爻為主",
            vec![yao_ci(zhi, bu_bian[0], Role::Zhu), yao_ci(zhi, bu_bian[1], Role::Bing)],
        ),
        5 => ("五爻變則以之卦不變爻占", vec![yao_ci(zhi, bu_bian[0], Role::Zhu)]),
        _ => {
            if ben == 0b111111 || ben == 0 {
                (
                    "六爻變則乾坤占二用餘卦占之卦彖辭",
                    vec![Reading { gua: ben, ci: Ci::Yong, role: Role::Zhu, text: row(ben).5.unwrap() }],
                )
            } else {
                ("六爻變則乾坤占二用餘卦占之卦彖辭", vec![gua_ci(zhi, Role::Zhu)])
            }
        }
    };
    ZhanCi { ben, zhi, bian, rule, readings }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(n: &str) -> u8 {
        text::GUA.iter().find(|r| r.1 == n).unwrap().2
    }

    /// 由本卦、之卦作六爻之数（变者老、不变者少）
    fn vals(ben: &str, zhi: &str) -> [u8; 6] {
        let (b, z) = (g(ben), g(zhi));
        core::array::from_fn(|k| {
            let yb = b >> k & 1 == 1;
            let yz = z >> k & 1 == 1;
            match (yb, yb != yz) {
                (true, true) => 9,
                (true, false) => 7,
                (false, true) => 6,
                (false, false) => 8,
            }
        })
    }

    #[test]
    fn text_table() {
        assert_eq!(text::GUA.len(), 64);
        let mut bits: Vec<u8> = text::GUA.iter().map(|r| r.2).collect();
        bits.sort();
        assert_eq!(bits, (0..64).collect::<Vec<u8>>());
        assert_eq!(row(g("乾")).5, Some("用九：見羣龍无首，吉。"));
        assert_eq!(row(g("坤")).5, Some("用六：利永貞。"));
    }

    #[test]
    fn dayan() {
        assert_eq!(dayan_probabilities(), [(6, 1), (7, 5), (8, 7), (9, 3)]);
        assert_eq!(coin_probabilities().iter().map(|x| x.1).sum::<u32>(), 16);
        // 第一变所去必为五或九，第二、三变必为四或八
        for left in 1..=47u32 {
            let t = dayan_change(49, left).unwrap();
            assert!(t == 5 || t == 9, "left {left} → {t}");
        }
        for total in [44u32, 40] {
            for left in 1..=total - 2 {
                let t = dayan_change(total, left).unwrap();
                assert!(t == 4 || t == 8);
            }
        }
        // 三变皆多（9、8、8）余 24 策为老阴六；皆少（5、4、4）余 36 为老阳九
        // 左皆分四：第一变左奇四、右（44）奇四去九，余四十；第二变左奇四、右（35）奇三去八；第三变同去八，余二十四策为老阴
        assert_eq!(dayan_yao([4, 4, 4]), Some(6));
        assert!(dayan_yao([0, 1, 1]).is_none());
        // 逐分遍历：各爻数皆可得
        let mut seen = [false; 4];
        for a in 1..=47 {
            for b in [1u32, 2, 3, 4] {
                for c in [1u32, 2, 3, 4] {
                    if let Some(v) = dayan_yao([a, b, c]) {
                        seen[(v - 6) as usize] = true;
                    }
                }
            }
        }
        assert_eq!(seen, [true; 4]);
        assert!((6..=9).contains(&dayan_yao_from_random([123456, 7, 99])));
    }

    #[test]
    fn zuo_guo_examples() {
        // 一爻变以本卦变爻辞占（《啟蒙》引沙隨程氏所举左传诸例）
        let one = |b: &str, z: &str, k: u8, text: &'static str| {
            let r = kao_bian_zhan(vals(b, z), ZhanOptions::default());
            assert_eq!(r.bian, vec![k], "{b}之{z}");
            assert_eq!(r.readings, vec![Reading { gua: g(b), ci: Ci::Yao(k), role: Role::Zhu, text }]);
        };
        // 左传庄公二十二年：陳敬仲遇觀之否，「是謂觀國之光，利用賓于王」
        one("觀", "否", 4, "六四：觀國之光，利用賓于王。");
        one("屯", "比", 1, row(g("屯")).4[0]); // 閔元年 畢萬
        one("乾", "同人", 2, "九二：見龍在田，利見大人。"); // 昭二十九年 蔡墨
        one("大有", "睽", 3, row(g("大有")).4[2]); // 僖二十五年 晉文公
        one("坤", "比", 5, "六五：黃裳，元吉。"); // 昭十二年 南蒯
        one("歸妹", "睽", 6, row(g("歸妹")).4[5]); // 僖十五年 晉獻公
        // 国语晋语四：重耳「貞屯悔豫皆八」——初、四、五三爻变，本卦屯、之卦豫，「皆利建侯」
        let r = kao_bian_zhan(vals("屯", "豫"), ZhanOptions::default());
        assert_eq!(r.bian, vec![1, 4, 5]);
        assert_eq!(r.readings.iter().map(|x| (name(x.gua), x.ci)).collect::<Vec<_>>(), [("屯", Ci::GuaCi), ("豫", Ci::GuaCi)]);
        assert!(r.readings.iter().all(|x| x.text.contains("利建侯")));
        // 左传襄公九年：穆姜遇艮之八，「是謂艮之隨」——五爻变唯二不变，《啟蒙》「法宜以係小子失丈夫為占」
        let r = kao_bian_zhan(vals("艮", "隨"), ZhanOptions::default());
        assert_eq!(r.bian, vec![1, 3, 4, 5, 6]);
        assert_eq!(r.readings.len(), 1);
        assert_eq!((name(r.readings[0].gua), r.readings[0].ci), ("隨", Ci::Yao(2)));
        assert!(r.readings[0].text.contains("係小子，失丈夫"));
        // 蔡墨「乾之坤曰見羣龍无首吉」——六爻变，乾坤占二用
        let r = kao_bian_zhan([9; 6], ZhanOptions::default());
        assert_eq!(r.readings[0].ci, Ci::Yong);
        assert!(r.readings[0].text.contains("見羣龍无首"));
        // 六爻皆不变：占本卦彖辞（《啟蒙》引孔成子筮立衛公子元遇屯「利建侯」）
        let r = kao_bian_zhan([7, 8, 8, 8, 7, 8], ZhanOptions::default());
        assert_eq!((name(r.ben), r.readings[0].ci), ("屯", Ci::GuaCi));
    }

    #[test]
    fn two_and_four() {
        // 二爻变：本卦二变爻辞，以上爻为主
        let r = kao_bian_zhan([9, 9, 7, 7, 7, 7], ZhanOptions::default());
        assert_eq!(r.readings.iter().map(|x| (x.ci, x.role)).collect::<Vec<_>>(), [(Ci::Yao(2), Role::Zhu), (Ci::Yao(1), Role::Bing)]);
        // 四爻变：之卦二不变爻，以下爻为主
        let r = kao_bian_zhan([9, 9, 9, 9, 7, 7], ZhanOptions::default());
        assert_eq!(name(r.zhi), "觀"); // 《啟蒙》「以乾為本卦四爻變自觀至臨」
        assert_eq!(r.readings.iter().map(|x| (x.ci, x.role)).collect::<Vec<_>>(), [(Ci::Yao(5), Role::Zhu), (Ci::Yao(6), Role::Bing)]);
        // 三爻变：乾变初二三得否（含初爻）主贞，变四五上得泰主悔
        let r = kao_bian_zhan([9, 9, 9, 7, 7, 7], ZhanOptions::default());
        assert_eq!((name(r.zhi), r.readings[0].role), ("否", Role::Zhu));
        let r = kao_bian_zhan([7, 7, 7, 9, 9, 9], ZhanOptions::default());
        assert_eq!((name(r.zhi), r.readings[1].role), ("泰", Role::Zhu));
        // 《啟蒙》：乾三爻变「自否至恒為前十卦」——恒（巽下震上，变初五上）含初爻；「自益至泰為後十卦」——益（震下巽上，变二三四）不含
        let r = kao_bian_zhan([9, 7, 7, 7, 9, 9], ZhanOptions::default());
        assert_eq!((name(r.zhi), r.readings[0].role), ("恒", Role::Zhu));
        let r = kao_bian_zhan([7, 9, 9, 9, 7, 7], ZhanOptions::default());
        assert_eq!((name(r.zhi), r.readings[1].role), ("益", Role::Zhu));
    }
}
