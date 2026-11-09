//! 太乙格局：掩、击、迫、囚、关、格、对、提挟、四郭固、四郭杜，及主客将发不发。
//!
//! 依《太乙金鏡式經》卷三「推掩法」至「推提挾法」、卷四「推五將發不發」，集成卷六八七淘金歌「關囚」「格」「對」「掩」「擊」
//! 「迫」「提挾」「門發將具」，太乙秘書「总论」。方位：十六神顺行（子→丑→艮→寅…）为「前」，前曰外、后曰内
//! （淘金歌「乃始擊在太乙先後一宮也。在前為外擊，在後為內擊」；延光二年太乙四宫、始击临寅「在太乙後一位……內擊」，
//! 义熙十四年太乙八宫、始击艮「在前一宮，外擊」）。
//!
//! - 天目（文昌、主目、下目）：与太乙同位为囚；前后一辰为外/内辰迫，前后一宫为外/内宫迫；冲太乙为对（金鏡「下目文昌將
//!   與太乙衝而相當者為對」）。
//! - 始击（客目、上目）：同位为掩；前后一辰、一宫为辰击、宫击（金鏡「王希明曰上目無迫」）；冲太乙为格。
//! - 主客大将参将：与太乙同宫为囚；前后一宫为宫迫；冲太乙者客将为格、主将为对（淘金歌「客目與客大小將沖太乙之宮為格，
//!   主目與主大小將沖太乙之宮為對」；金鏡「客目大小將與太乙對宫為格」）；得五为杜。
//! - 关：主客大小将同宫（金鏡「主客大小將同宫數齊皆為闗」）；客将入天目宫为客关主、主将入始击宫为主关客；二目同宫为二目关
//!   （太乙秘書总论、文昌条）。
//! - 掩将：始击临主大将或主参将之宫（太乙秘書阳局第一局「始击将大武掩主大将」）。
//! - 提挟：某位前后两邻（正宫者邻宫、间神者邻辰）俱为敌方之目、将或太乙所据，且至少一为敌方（金鏡引李淳風「客主兩将或一将
//!   而共太乙挟客主目或大小將扵正宫者為提挟」）；挟太乙者为己方二位。
//! - 四郭固（金鏡）：文昌囚太乙宫，而主客将有关、或客目临太乙宫（掩）。四郭杜（金鏡）：客参将与文昌同宫，且主大将与客大将同宫。
//! - 将发（淘金歌「門發將具」）：文昌与主大小将无囚迫挟、且不杜者主将发；始击与客大小将无掩击格挟、且不杜者客将发。

use super::{gong_pos, pos_gong, TaiyiJu};

/// 方：前为外、后为内
fn fang(d: usize) -> Option<&'static str> {
    match d {
        1 | 2 => Some("外"),
        14 | 15 => Some("內"),
        _ => None,
    }
}

/// 目（十六神位）对太乙之状：`ke` 为客目（始击）
pub fn mu_zhuang(p: usize, gong: u8, ke: bool) -> String {
    let tp = gong_pos(gong);
    let d = (p + 16 - tp) % 16;
    let (tong, po, chong) = if ke { ("掩", "擊", "格") } else { ("囚", "迫", "對") };
    match d {
        0 => tong.into(),
        8 => chong.into(),
        1 | 15 => format!("{}辰{}", fang(d).unwrap(), po),
        2 | 14 => format!("{}宮{}", fang(d).unwrap(), po),
        _ => String::new(),
    }
}

/// 将（宫）对太乙之状：`ke` 为客将
pub fn jiang_zhuang(g: Option<u8>, gong: u8, ke: bool) -> String {
    let Some(g) = g else { return "杜".into() };
    let d = (gong_pos(g) + 16 - gong_pos(gong)) % 16;
    match d {
        0 => "囚".into(),
        8 => (if ke { "格" } else { "對" }).into(),
        2 | 14 => format!("{}宮迫", fang(d).unwrap()),
        _ => String::new(),
    }
}

/// 一局之格局
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeJu {
    /// 天目、始击、主大将、主参将、客大将、客参将各自对太乙之状（空串为无）
    pub tian_mu: String,
    pub shi_ji: String,
    pub zhu_da: String,
    pub zhu_can: String,
    pub ke_da: String,
    pub ke_can: String,
    /// 诸位相互之格：关、掩将、提挟、四郭固、四郭杜
    pub xiang: Vec<String>,
    /// 主将发、客将发
    pub zhu_fa: bool,
    pub ke_fa: bool,
}

/// 局中诸位
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Wei {
    TaiYi,
    ZhuMu,
    ZhuDa,
    ZhuCan,
    KeMu,
    KeDa,
    KeCan,
}

impl Wei {
    fn name(self) -> &'static str {
        match self {
            Wei::TaiYi => "太乙",
            Wei::ZhuMu => "主目",
            Wei::ZhuDa => "主大將",
            Wei::ZhuCan => "主參將",
            Wei::KeMu => "客目",
            Wei::KeDa => "客大將",
            Wei::KeCan => "客參將",
        }
    }
    /// 0 太乙、1 主、2 客
    fn side(self) -> u8 {
        match self {
            Wei::TaiYi => 0,
            Wei::ZhuMu | Wei::ZhuDa | Wei::ZhuCan => 1,
            _ => 2,
        }
    }
}

/// 判一局格局
pub fn ge_ju(j: &TaiyiJu) -> GeJu {
    let tp = gong_pos(j.gong);
    let mut xiang = Vec::new();
    // 诸位之十六神位
    let mut at: Vec<(Wei, usize)> = vec![(Wei::TaiYi, tp), (Wei::ZhuMu, j.tian_mu), (Wei::KeMu, j.shi_ji)];
    for (w, g) in [(Wei::ZhuDa, j.zhu_da), (Wei::ZhuCan, j.zhu_can), (Wei::KeDa, j.ke_da), (Wei::KeCan, j.ke_can)] {
        if let Some(g) = g {
            at.push((w, gong_pos(g)));
        }
    }
    let pos = |w: Wei| at.iter().find(|x| x.0 == w).map(|x| x.1);
    // 关：主客将同宫
    for a in [Wei::ZhuDa, Wei::ZhuCan] {
        for b in [Wei::KeDa, Wei::KeCan] {
            if let (Some(p), Some(q)) = (pos(a), pos(b)) {
                if p == q {
                    xiang.push(format!("{}{}關", a.name(), b.name()));
                }
            }
        }
    }
    // 客将入文昌宫为客关主；主将入始击宫为主关客
    for b in [Wei::KeDa, Wei::KeCan] {
        if pos(b) == Some(j.tian_mu) {
            xiang.push(format!("{}關主目", b.name()));
        }
    }
    for a in [Wei::ZhuDa, Wei::ZhuCan] {
        if pos(a) == Some(j.shi_ji) {
            xiang.push(format!("客目掩{}", a.name()));
        }
    }
    if j.tian_mu == j.shi_ji {
        xiang.push("二目關".into());
    }
    // 提挟
    let mut xie = Vec::new();
    for &(x, p) in &at {
        let step = if pos_gong(p).is_some() { 2 } else { 1 };
        let lo: Vec<Wei> = at.iter().filter(|y| y.1 == (p + 16 - step) % 16 && y.0 != x).map(|y| y.0).collect();
        let hi: Vec<Wei> = at.iter().filter(|y| y.1 == (p + step) % 16 && y.0 != x).map(|y| y.0).collect();
        let try_side = |s: u8| -> bool {
            let ok = |v: &Vec<Wei>| v.iter().any(|w| w.side() == s || (w.side() == 0 && x.side() != 0));
            let any_s = lo.iter().chain(hi.iter()).any(|w| w.side() == s);
            ok(&lo) && ok(&hi) && any_s
        };
        let enemy: &[u8] = match x.side() {
            0 => &[1, 2],
            1 => &[2],
            _ => &[1],
        };
        for &s in enemy {
            if try_side(s) {
                let who = if s == 1 { "主" } else { "客" };
                xie.push((x, format!("{}挾{}", who, x.name())));
            }
        }
    }
    let xie_of = |w: Wei| xie.iter().any(|(x, _)| *x == w);
    xiang.extend(xie.iter().map(|(_, s)| s.clone()));

    let tian_mu = mu_zhuang(j.tian_mu, j.gong, false);
    let shi_ji = mu_zhuang(j.shi_ji, j.gong, true);
    let zhu_da = jiang_zhuang(j.zhu_da, j.gong, false);
    let zhu_can = jiang_zhuang(j.zhu_can, j.gong, false);
    let ke_da = jiang_zhuang(j.ke_da, j.gong, true);
    let ke_can = jiang_zhuang(j.ke_can, j.gong, true);

    // 四郭固、四郭杜（金鏡卷三）
    let jiang_guan = xiang.iter().any(|s| s.ends_with("將關"));
    if tian_mu == "囚" && (jiang_guan || shi_ji == "掩") {
        xiang.push("四郭固".into());
    }
    if pos(Wei::KeCan).is_some() && pos(Wei::KeCan) == Some(j.tian_mu) && j.zhu_da.is_some() && j.zhu_da == j.ke_da {
        xiang.push("四郭杜".into());
    }

    let bad_zhu = |s: &str| s == "杜" || s.contains('囚') || s.contains('迫');
    let bad_ke = |s: &str| s == "杜" || s.contains('掩') || s.contains('擊') || s.contains('格') || s.contains('囚') || s.contains('迫');
    let zhu_fa = !bad_zhu(&tian_mu)
        && !bad_zhu(&zhu_da)
        && !bad_zhu(&zhu_can)
        && !xie_of(Wei::ZhuMu)
        && !xie_of(Wei::ZhuDa)
        && !xie_of(Wei::ZhuCan);
    let ke_fa = !bad_ke(&shi_ji)
        && !bad_ke(&ke_da)
        && !bad_ke(&ke_can)
        && !xie_of(Wei::KeMu)
        && !xie_of(Wei::KeDa)
        && !xie_of(Wei::KeCan);
    GeJu { tian_mu, shi_ji, zhu_da, zhu_can, ke_da, ke_can, xiang, zhu_fa, ke_fa }
}

#[cfg(test)]
mod tests {
    use super::super::{ju, sui_ji, Dun, JiNian};
    use super::*;

    fn g(y: i32) -> GeJu {
        ge_ju(&ju(sui_ji(y, JiNian::TaoJinGe), Dun::Yang))
    }

    /// 淘金歌（集成卷六八七）所举格局诸例
    #[test]
    fn taojinge_ge_ju_examples() {
        // 建安五年：「主大將在七宮，乃關而又囚」，「此乃主客同關」
        let a = g(200);
        assert_eq!(a.zhu_da, "囚");
        assert!(a.xiang.contains(&"主大將客大將關".to_string()), "{:?}", a.xiang);
        // 建安二十四年：「客大將在七宮，與天目同宮，此乃客關主目也」
        assert!(g(219).xiang.contains(&"客大將關主目".to_string()));
        // 承圣三年：「高叢為天目，主筭得單四主將在四宮……同宮為囚」
        assert_eq!(g(554).zhu_da, "囚");
        // 大宝二年：「始擊格太乙」
        assert_eq!(g(551).shi_ji, "格");
        // 永兴元年：「天目臨大炅，與太乙宮對」
        assert_eq!(g(304).tian_mu, "對");
        // 大业七年：「始擊臨地主，格掩太乙」
        assert_eq!(g(611).shi_ji, "掩");
        // 延光二年：「始擊臨寅在太乙後一位，間神之宮，內擊」；义熙十四年：「始擊艮三宮在前一宮，外擊」
        assert_eq!(g(123).shi_ji, "內辰擊");
        assert_eq!(g(418).shi_ji, "外宮擊");
        // 咸安元年：「陽德為天目，外辰迫，主筭三十三，主大將三宮，外宮迫」
        let a = g(371);
        assert_eq!((a.tian_mu.as_str(), a.zhu_da.as_str()), ("外辰迫", "外宮迫"));
        // 汉成帝阳朔元年丁酉（前 24）：「第三戊子元第十局，太乙在四宮，天目在呂申，主筭單一，內辰迫」
        let j = ju(sui_ji(-23, JiNian::TaoJinGe), Dun::Yang);
        assert_eq!((j.yuan, j.ju, j.gong, j.zhu_suan), (2, 10, 4, 1));
        assert_eq!(ge_ju(&j).tian_mu, "內辰迫");
        // 晋安帝隆安元年丁酉（397）：第三戊子元七十局，太乙九宫，天目天道，主算三十、主大将三宫，计神大神，始击高叢，
        // 客算单四、客大将四宫，「太乙與主將挾客於四宮」
        let j = ju(sui_ji(397, JiNian::TaoJinGe), Dun::Yang);
        assert_eq!((j.yuan, j.ju, j.gong, j.zhu_suan, j.zhu_da, j.ke_suan, j.ke_da), (2, 70, 9, 30, Some(3), 4, Some(4)));
        assert!(ge_ju(&j).xiang.contains(&"主挾客大將".to_string()), "{:?}", ge_ju(&j).xiang);
        // 建安三年戊寅（198）：一元十五局，太乙六宫，天目大威，始击大武，客算单七、客大将七宫、客参将一宫，「乃客挾太乙」
        let j = ju(sui_ji(198, JiNian::TaoJinGe), Dun::Yang);
        assert_eq!((j.ju, j.gong, j.ke_suan, j.ke_da, j.ke_can), (15, 6, 7, Some(7), Some(1)));
        assert!(ge_ju(&j).xiang.contains(&"客挾太乙".to_string()));
        // 汉灵帝中平二年乙丑（185）：一元二局，太乙一宫，太簇为天目，主算单六、主大将六宫、主参将八宫（「此提挾也」）；
        // 客目阴主、客算单一、客大将一宫、客参将三宫（「此掩擊也」）
        let j = ju(sui_ji(185, JiNian::TaoJinGe), Dun::Yang);
        assert_eq!((j.ju, j.gong, j.zhu_suan, j.zhu_da, j.zhu_can, j.ke_suan, j.ke_da, j.ke_can), (2, 1, 6, Some(6), Some(8), 1, Some(1), Some(3)));
        let a = ge_ju(&j);
        assert!(a.xiang.contains(&"主挾客大將".to_string()), "{:?}", a.xiang);
        assert!(a.xiang.contains(&"主挾太乙".to_string()), "{:?}", a.xiang);
        assert_eq!(a.shi_ji, "內辰擊");
    }

    /// 太乙秘書「定主客胜负阳局、阴局七十二局」逐局比对（太乙、天目、始击、计神、主客算、四将之宫与状）
    #[test]
    fn mishu_144_ju() {
        let rows = mishu_check();
        let total: usize = rows.iter().map(|r| r.n).sum();
        let bad: Vec<_> = rows.iter().filter(|r| !r.diff.is_empty()).collect();
        let nbad: usize = bad.iter().map(|r| r.diff.len()).sum();
        // 逐项不合者列于 doc；此处只防回退
        assert!(total > 1500, "{total}");
        assert!(nbad <= MISHU_KNOWN_BAD, "{nbad} 项不合：{:#?}", bad);
    }

    /// 秘書 144 局比对中已知不合的项数（逐项见 doc/folk/taiyi.md）
    const MISHU_KNOWN_BAD: usize = 55;

    #[derive(Debug)]
    pub(crate) struct MiShuRow {
        pub dun: Dun,
        pub ju: u16,
        /// 比对项数
        pub n: usize,
        /// 不合项：（项，书，推）
        pub diff: Vec<(String, String, String)>,
    }

    fn jian(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '阳' => '陽',
                '阴' => '陰',
                '德' => '德',
                '义' => '義',
                '丛' => '叢',
                '吕' => '呂',
                '炅' | '昊' => '炅',
                '簇' => '簇',
                c => c,
            })
            .collect::<String>()
            .replace("大簇", "太簇")
            .replace("太炅", "大炅")
            .replace("大阳", "太陽")
            .replace("大陽", "太陽")
    }

    /// 解析秘書 144 局并与推算逐项比对
    pub(crate) fn mishu_check() -> Vec<MiShuRow> {
        use super::super::{POS16, SHEN16};
        let path = crate::repo::repo_root().join("vendor/folk/wikisource/太乙秘書/全文.wikitext");
        let text = std::fs::read_to_string(path).expect("太乙秘書");
        let yang = text.find("○ 定主客胜负阳局七十二局").unwrap();
        let yin = text.find("○ 定主客胜负阴局七十二局").unwrap();
        let end = text[yin..].find("○ 太乙总图").map(|e| e + yin).unwrap();
        let num = |s: &str| -> Option<u16> {
            let d = |c: char| "一二三四五六七八九".chars().position(|x| x == c).map(|p| p as u16 + 1);
            let cs: Vec<char> = s.chars().collect();
            match cs.as_slice() {
                [] => None,
                ['單' | '单', rest @ ..] => d(*rest.first()?),
                [a] => d(*a).or(if *a == '十' { Some(10) } else { None }),
                ['十', b] => Some(10 + d(*b)?),
                [a, '十'] => Some(d(*a)? * 10),
                [a, '十', b] => Some(d(*a)? * 10 + d(*b)?),
                _ => None,
            }
        };
        let names: Vec<String> = SHEN16.iter().map(|s| s.to_string()).collect();
        let find_name = |s: &str| -> Option<usize> {
            let s = jian(s);
            names.iter().position(|n| s.starts_with(n.as_str())).or_else(|| POS16.iter().position(|z| s.starts_with(z)))
        };
        let mut out = Vec::new();
        for (dun, seg) in [(Dun::Yang, &text[yang..yin]), (Dun::Yin, &text[yin..end])] {
            for blk in seg.split("● 第").skip(1) {
                let jn: String = blk.chars().take_while(|c| *c != '局').collect();
                let Some(r) = num(&jn) else { continue };
                let j = ju(r, dun);
                let gj = ge_ju(&j);
                let mut row = MiShuRow { dun, ju: r, n: 0, diff: Vec::new() };
                let mut cmp = |k: &str, book: String, got: String| {
                    row.n += 1;
                    if book != got {
                        row.diff.push((k.to_string(), book, got));
                    }
                };
                let body = jian(blk);
                let after = |key: &str| body.find(key).map(|i| &body[i + key.len()..]);
                if let Some(s) = after("太乙在") {
                    let shu = ["", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
                    cmp("太乙", s.chars().take(1).collect(), shu[j.gong as usize].into());
                }
                if let Some(s) = after("天目") {
                    let s = s.trim_start_matches("文昌將臨").trim_start_matches("文昌将临");
                    if let Some(p) = find_name(s) {
                        cmp("天目", SHEN16[p].into(), SHEN16[j.tian_mu].into());
                    }
                }
                if let Some(s) = after("始击将") {
                    let s = s.trim_start_matches('临').trim_start_matches('臨');
                    if let Some(p) = find_name(s) {
                        cmp("始击", SHEN16[p].into(), SHEN16[j.shi_ji].into());
                    }
                }
                if let Some(s) = after("计神") {
                    if let Some(p) = find_name(s) {
                        cmp("计神", POS16[p].into(), POS16[j.ji_shen].into());
                    }
                }
                for (key, v) in [("主算", j.zhu_suan), ("客算", j.ke_suan)] {
                    if let Some(s) = after(key) {
                        let t: String = s.chars().take_while(|c| "單单一二三四五六七八九十".contains(*c)).collect();
                        if let Some(b) = num(&t) {
                            cmp(key, b.to_string(), v.to_string());
                        }
                    }
                }
                for (key, g, st) in [
                    ("主大将", j.zhu_da, &gj.zhu_da),
                    ("主参将", j.zhu_can, &gj.zhu_can),
                    ("客大将", j.ke_da, &gj.ke_da),
                    ("客参将", j.ke_can, &gj.ke_can),
                ] {
                    let Some(s) = after(key) else { continue };
                    let gc: String = s.chars().take(1).collect();
                    let Some(bg) = num(&gc) else { continue };
                    if !s.chars().nth(1).is_some_and(|c| c == '宫') {
                        continue;
                    }
                    cmp(&format!("{key}宫"), bg.to_string(), g.map(|v| v.to_string()).unwrap_or("杜".into()));
                    // 状：取「X宫」后至下一将名或「计神」前之文字
                    let rest: String = s.chars().skip(2).collect();
                    let stop = ["主大将", "主参将", "客大将", "客参将", "始击", "客算", "计神", "此局"]
                        .iter()
                        .filter_map(|k| rest.find(k))
                        .min()
                        .unwrap_or(rest.len());
                    let tag = &rest[..stop];
                    // 「与天目囚」「与主目囚」之类言将与目同宫，非对太乙之状，不比
                    if tag.contains('与') || tag.contains('與') {
                        continue;
                    }
                    let book = if tag.contains('囚') {
                        "囚"
                    } else if tag.contains('迫') {
                        "迫"
                    } else if tag.contains('格') || tag.contains('对') || tag.contains('對') {
                        "格對"
                    } else if tag.contains('发') {
                        "发"
                    } else {
                        continue;
                    };
                    let got = if st.contains('囚') {
                        "囚"
                    } else if st.contains('迫') {
                        "迫"
                    } else if st == "格" || st == "對" {
                        "格對"
                    } else {
                        "发"
                    };
                    cmp(&format!("{key}状"), book.into(), got.into());
                }
                out.push(row);
            }
        }
        out
    }

    /// 打印秘書比对不合项（供 doc 表；`cargo test ... -- --ignored --nocapture mishu_dump`）
    #[test]
    #[ignore]
    fn mishu_dump() {
        let rows = mishu_check();
        let cell = |d: Dun, r: u16| match rows.iter().find(|x| x.dun == d && x.ju == r) {
            None => "缺".to_string(),
            Some(x) if x.diff.is_empty() => format!("{} 项合", x.n),
            Some(x) => format!(
                "{} 项，不合：{}",
                x.n,
                x.diff.iter().map(|(k, b, g)| format!("{k}書{b}推{g}")).collect::<Vec<_>>().join("；")
            ),
        };
        for r in 1..=72u16 {
            println!("| {} | {} | {} |", r, cell(Dun::Yang, r), cell(Dun::Yin, r));
        }
        println!("total {} bad {}", rows.iter().map(|r| r.n).sum::<usize>(), rows.iter().map(|r| r.diff.len()).sum::<usize>());
    }
}
