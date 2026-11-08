//! 奇门格局（时家布盘之后）：依明程道生《遁甲演義》卷二逐条起例。
//!
//! 「天上（上盤）某加地下（下盤）某」即某宫天盘干为某、地盘干为某；「甲」不入盘，遁于本旬之仪（甲子戊、甲戌己、甲申庚、
//! 甲午辛、甲辰壬、甲寅癸），故「时干」「日干」「年干」为甲时取其旬之仪（演义卷二伏干格例「天柱為六庚下臨九宮見今甲申」、
//! 飞干格例「日干六甲下臨六庚」即以甲子之戊当甲）。转盘寄宫之格，地盘、天盘干皆连带中五之干。
//! 原文不明或自相出入处，注「未核」并说明所取。各条出处与逐例检验见 doc/folk/qimen.md「格局」。

use super::pan::{QimenPan, YI_QI};

const GAN: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];

/// 一个格局
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeJu {
    pub name: &'static str,
    /// 所在宫（洛书 1—9）；不系于宫者（五不遇时、时干入墓等）为 None
    pub gong: Option<u8>,
    /// 吉格 true、凶格 false
    pub ji: bool,
    /// 依据（演义卷二歌诀或所引之家）
    pub basis: &'static str,
    /// 说明（未核、另说等）
    pub note: Option<&'static str>,
}

/// 判格所需的盘外之干：日干支（必有）、年干支、月朔干（演义「月格」用月朔之干，例见卷二「論月格」）
#[derive(Clone, Copy, Debug, Default)]
pub struct GeJuContext {
    pub day_gz: u8,
    pub year_gz: Option<u8>,
    /// 本月朔日之干（0 甲 … 9 癸）
    pub month_shuo_gan: Option<u8>,
}

/// 某干支在盘上之干：甲遁于本旬之仪
pub fn gan_on_pan(gz: u8) -> &'static str {
    let gz = gz % 60;
    if gz % 10 == 0 {
        YI_QI[(gz / 10) as usize]
    } else {
        GAN[(gz % 10) as usize]
    }
}

/// 洛书宫之五行：一水、二土、三木、四木、六金、七金、八土、九火（中五土）
fn gong_wx(p: u8) -> u8 {
    // 0 木 1 火 2 土 3 金 4 水
    [0, 4, 2, 0, 0, 2, 3, 3, 2, 1][p as usize]
}
fn men_wx(m: &str) -> Option<u8> {
    Some(match m {
        "休門" => 4,
        "生門" | "死門" => 2,
        "傷門" | "杜門" => 0,
        "景門" => 1,
        "驚門" | "開門" => 3,
        _ => return None,
    })
}
fn ke(a: u8, b: u8) -> bool {
    // 木克土、土克水、水克火、火克金、金克木
    matches!((a, b), (0, 2) | (2, 4) | (4, 1) | (1, 3) | (3, 0))
}

const JI_MEN: [&str; 3] = ["休門", "生門", "開門"];
const SAN_QI: [&str; 3] = ["乙", "丙", "丁"];

/// 九星本宫（伏吟、反吟用）
fn xing_home(x: &str) -> Option<u8> {
    super::pan::XING.iter().position(|s| *s == x).map(|p| p as u8).filter(|&p| p > 0)
}

/// 判一盘之格局，次序：吉格（三遁、九遁、跌穴返首、玉女、得使、诈假）在前，凶格在后。
pub fn ge_ju(pan: &QimenPan, ctx: &GeJuContext) -> Vec<GeJu> {
    let mut out = Vec::new();
    let mut add = |name, gong, ji, basis, note| out.push(GeJu { name, gong, ji, basis, note });
    let xun_yi = pan.xun_yi;
    let hour_gan = gan_on_pan(pan.hour_gz);
    let day_gan = gan_on_pan(ctx.day_gz);
    for p in 1..=9u8 {
        let c = pan.at(p);
        let t = |s: &str| c.tian_pan.contains(&s);
        let d = |s: &str| c.di_pan.contains(&s);
        let men = c.men.unwrap_or("");
        let shen = c.shen.unwrap_or("");
        let g = Some(p);
        // —— 三遁（卷二首三歌）
        if t("丙") && d("丁") && men == "生門" {
            add("天遁", g, true, "卷二「生門六丙合六丁，此為天遁自分明」", None);
        }
        if t("乙") && d("己") && men == "開門" {
            add("地遁", g, true, "卷二「開門六乙合六己，地遁如斯而已矣」", None);
        }
        if t("丁") && men == "休門" && shen == "太陰" {
            add("人遁", g, true, "卷二「休門六丁共太隂，欲求人遁無過此」", None);
        }
        // —— 九遁之余六遁（取歌诀；「又曰」诸说未取）
        if t("丙") && men == "生門" && shen == "九天" {
            add("神遁", g, true, "卷二「天上六丙合九天，再合生門神遁然」", None);
        }
        if t("乙") && men == "杜門" && shen == "九地" {
            add("鬼遁", g, true, "卷二「天上六乙合九地，臨於杜門鬼遁」", Some("又曰丁奇或乙奇合休門、開門臨九地，未取"));
        }
        if t("乙") && JI_MEN.contains(&men) && p == 4 {
            add("風遁", g, true, "卷二「天上六乙合三門，下臨巽宫風遁矣」", Some("又曰丙奇開門臨巽等，未取"));
        }
        if t("乙") && d("辛") && JI_MEN.contains(&men) {
            add("雲遁", g, true, "卷二「天上六乙合六辛，臨三吉門雲遁取」", Some("又曰乙奇開門臨坤等，未取"));
        }
        if t("乙") && men == "休門" && p == 1 {
            add("龍遁", g, true, "卷二「日竒合休於坎宫，此為龍遁雲從起」", Some("又曰乙奇休門臨六癸，未取"));
        }
        if t("乙") && d("辛") && men == "休門" && p == 8 {
            add("虎遁", g, true, "卷二「天上六乙合六辛，臨休到艮虎遁門」", Some("又曰乙奇生門臨六辛、辛儀合生臨艮，未取"));
        }
        // —— 飞鸟跌穴、青龙返首：天盘丙加地盘本时旬首之仪；直符（旬首之仪）加地盘丙
        if t("丙") && d(xun_yi) {
            add("飛鳥跌穴", g, true, "卷二「丙加甲兮鳥跌穴」：天盤丙竒加地盤本時甲旬頭", None);
        }
        if t(xun_yi) && d("丙") {
            add("青龍返首", g, true, "卷二「甲加丙兮龍囘首」：直符（旬首之儀）加地盤丙", None);
        }
        // —— 玉女守门：直使之门加地盘丁（卷二例「休門直使加地丁七宫」）
        if p == pan.zhi_shi_gong && d("丁") {
            add("玉女守門", g, true, "卷二「三竒遊六儀，號為玉女守門扉」；例「休門直使加地丁七宫」", Some("以直使所加之宫地盘为丁取；歌「三竒遊六儀」泛指奇加仪，未另立"));
        }
        // —— 三奇得使：乙加甲戌己、甲午辛；丙加甲子戊、甲申庚；丁加甲辰壬、甲寅癸
        for (q, ys) in [("乙", ["己", "辛"]), ("丙", ["戊", "庚"]), ("丁", ["壬", "癸"])] {
            if t(q) && ys.iter().any(|y| d(y)) {
                let flaw = (q == "乙" && d("辛")) || (q == "丙" && d("庚")) || (q == "丁" && d("癸"));
                add(
                    "三奇得使",
                    g,
                    true,
                    "卷二「三竒得使誠堪使……乙馬逢犬丙鼠猴，六丁玉女騎龍虎」",
                    flaw.then_some("乙加辛即青龍逃走、丙加庚即熒入太白、丁加癸即朱雀投江，「尚有微疵」"),
                );
            }
        }
        // —— 三诈、五假（卷二「竒與門兮共太隂」条引古經）
        let qi = SAN_QI.iter().any(|q| t(q));
        if qi && JI_MEN.contains(&men) {
            match shen {
                "太陰" => add("真詐", g, true, "卷二引古經「三門合三竒下臨太隂宫名曰真詐」", None),
                "九地" => add("重詐", g, true, "卷二引古經「三門合三竒下臨九地宫名曰重詐」", None),
                "六合" => add("休詐", g, true, "卷二引古經「三門合三竒下臨六合宫名曰休詐」", None),
                _ => {}
            }
        }
        let dgk = ["丁", "己", "癸"].iter().any(|q| t(q));
        if men == "景門" && qi && shen == "九天" {
            add("天假", g, true, "卷二引古經「景門合乙丙丁臨九天名曰天假」", None);
        }
        if men == "杜門" && dgk && shen == "九地" {
            add("地假", g, true, "卷二引古經「杜門合丁己癸下臨九地名曰地假」", None);
        }
        if men == "驚門" && t("壬") && shen == "九天" {
            add("人假", g, true, "卷二引古經「驚門合六合下臨九天宫名曰人假」", Some("原文「合六合」与「下臨九天」皆为神，一宫不能并值，疑「六壬」之讹；暂取驚門合六壬臨九天，未核"));
        }
        if men == "傷門" && dgk && shen == "九地" {
            add("神假", g, true, "卷二引古經「傷門合丁己癸下臨九地名曰神假」", None);
        }
        if men == "死門" && dgk && shen == "九地" {
            add("鬼假", g, true, "卷二引古經「死門合丁己癸臨九地名曰鬼假」", None);
        }
        // —— 奇仪相加之凶格
        if t("乙") && d("辛") {
            add("青龍逃走", g, false, "卷二「六乙加辛龍逃走」", None);
        }
        if t("乙") && d("庚") {
            add("青龍逃走", g, false, "卷二青龍逃走條引「又曰乙加庚亦是」", Some("另说"));
        }
        if t("辛") && d("乙") {
            add("白虎猖狂", g, false, "卷二「六辛加乙虎猖狂」", None);
        }
        if t("丁") && d("癸") {
            add("朱雀投江", g, false, "卷二「六丁加癸雀入江」", None);
        }
        if t("癸") && d("丁") {
            add("螣蛇夭矯", g, false, "卷二「六癸加丁蛇夭蹻」", None);
        }
        if t("庚") && d("癸") {
            add("大格", g, false, "卷二「庚加癸兮為大格」", None);
        }
        if t("庚") && d("壬") {
            add("小格", g, false, "卷二「加壬之時為小格」", None);
        }
        if t("庚") && d("己") {
            add("刑格", g, false, "卷二「加己為刑最不宜」", None);
        }
        if t("丙") && d("庚") {
            add("熒入白", g, false, "卷二「六丙加庚熒入白……賊須滅」", None);
        }
        if t("庚") && d("丙") {
            add("白入熒", g, false, "卷二「六庚加丙白入熒……賊即來」", None);
        }
        // —— 庚加年月日时干：岁格、月格、日格（即伏干格）、时格；日干加庚：飞干格
        if t("庚") {
            if let Some(y) = ctx.year_gz {
                if d(gan_on_pan(y)) {
                    add("歲格", g, false, "卷二「論歲格」引三元經「六庚加當年太歲之干」", None);
                }
            }
            if let Some(m) = ctx.month_shuo_gan {
                let mg = if m == 0 { "" } else { GAN[(m % 10) as usize] };
                if !mg.is_empty() && d(mg) {
                    add("月格", g, false, "卷二「論月格」引三元經「六庚加月朔」", Some("月朔之干为甲者不入盘，未判"));
                }
            }
            if d(day_gan) {
                add("伏干格", g, false, "卷二「庚加日干為伏干」；即「論日格」之日干格", None);
            }
            if d(hour_gan) {
                add("時格", g, false, "卷二「論時格」引三元經「六庚加本時干者為時格，亦名伏吟格」", None);
            }
            if d(xun_yi) {
                add("天乙伏宫格", g, false, "卷二「庚加直符天乙伏」；例「天輔為六庚下臨直符天芮於二」", Some("以地盘旬首之仪（直符本位）取"));
            }
        }
        if t(day_gan) && d("庚") {
            add("飛干格", g, false, "卷二「日干加庚飛干格」", None);
        }
        if t(xun_yi) && d("庚") {
            add("天乙飛宫格", g, false, "卷二「直符加庚天乙飛」", None);
        }
        // —— 勃格：天盘丙加年月日时之干（卷二「丙為勃兮庚為格」）
        if t("丙") {
            let mut hit = d(day_gan) || d(hour_gan);
            if let Some(y) = ctx.year_gz {
                hit |= d(gan_on_pan(y));
            }
            if let Some(m) = ctx.month_shuo_gan {
                hit |= m != 0 && d(GAN[(m % 10) as usize]);
            }
            if hit {
                add("勃格", g, false, "卷二「丙為勃兮庚為格」：「天上六丙加年月日時之干」", Some("同条又云「丙加地庚為勃」，与熒入白重，未另立"));
            }
        }
        // —— 三奇入墓：乙临坤二、丙临乾六、丁临艮八（卷二引王璋）
        for (q, gg) in [("乙", 2u8), ("丙", 6), ("丁", 8)] {
            if t(q) && p == gg {
                add("三奇入墓", g, false, "卷二「三竒入墓好思推」引王璋「乙竒下臨二宫、丙竒到六宫、丁竒下臨八宫」", None);
            }
        }
        // —— 门迫：门克宫（卷二「宫制其門不為迫，門制其宫是迫雄」）
        if let (Some(mw), true) = (men_wx(men), p != 5) {
            if ke(mw, gong_wx(p)) {
                let jm = JI_MEN.contains(&men) || men == "景門";
                add("門迫", g, false, "卷二「宫制其門不為迫，門制其宫是迫雄」", Some(if jm { "吉門被迫，吉事不成" } else { "凶門被迫，凶事尤甚" }));
            }
        }
        // —— 天网四张：天盘癸所临之宫；一至四宫低、六至九宫高（卷二「天網四張」）
        if t("癸") && (d("癸") || hour_gan == "癸") {
            add(
                "天網四張",
                g,
                false,
                "卷二「天網四張無路走」：「天上六癸所加之下」「凡六癸之時亦是」",
                Some(if p <= 4 { "網低（一至四宫），有路可走" } else { "網高（六至九宫）" }),
            );
        }
    }
    // —— 九星伏吟、反吟（直符在本宫、对冲之宫）
    if let Some(h) = xing_home(pan.zhi_fu) {
        let fg = pan.zhi_fu_gong;
        if fg == h {
            add("伏吟", Some(fg), false, "卷二「就中伏吟為最凶，天蓬加著地天蓬」；「凡六甲之時門符皆是伏吟」", None);
        } else if h != 5 && fg != 5 && fg + h == 10 {
            add("反吟", Some(fg), false, "卷二「天蓬若到天英上，須知即是反吟宫」；「凡星符對沖皆反吟」", None);
        }
    }
    // —— 六仪击刑：直符加所刑之宫（卷二引王璋：甲子三、甲戌二、甲申八、甲午九、甲辰四、甲寅四）
    let xing_gong = [3u8, 2, 8, 9, 4, 4][(pan.hour_gz / 10) as usize];
    if pan.zhi_fu_gong == xing_gong {
        add("六儀擊刑", Some(xing_gong), false, "卷二「六儀擊刑何太凶」引王璋「甲子直符加三宫……甲寅直符臨四宫」", None);
    }
    // —— 五不遇时：时干克日干，阳克阳、阴克阴（卷二引葛洪）
    if (pan.hour_gz % 10 + 10 - ctx.day_gz % 10) % 10 == 6 {
        add("五不遇時", None, false, "卷二「五不遇時龍不精」引葛洪「甲日庚午時……癸日己未時」", None);
    }
    // —— 时干入墓：丙戌、壬辰、丁丑、癸未、戊戌、己丑时（卷二「又有時干入墓宫」）
    if [22u8, 28, 13, 19, 34, 25].contains(&(pan.hour_gz % 60)) {
        add("時干入墓", None, false, "卷二歌「戊戌壬辰兼丙戌，癸未丁丑已同凶」；葛洪「己丑時……亦不可用」", None);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::pan::{pai_pan, DEFAULT_PAN};
    use super::super::{ShiJu, Yuan};
    use super::*;

    const GZ: [&str; 60] = [
        "甲子", "乙丑", "丙寅", "丁卯", "戊辰", "己巳", "庚午", "辛未", "壬申", "癸酉", "甲戌", "乙亥", "丙子", "丁丑", "戊寅",
        "己卯", "庚辰", "辛巳", "壬午", "癸未", "甲申", "乙酉", "丙戌", "丁亥", "戊子", "己丑", "庚寅", "辛卯", "壬辰", "癸巳",
        "甲午", "乙未", "丙申", "丁酉", "戊戌", "己亥", "庚子", "辛丑", "壬寅", "癸卯", "甲辰", "乙巳", "丙午", "丁未", "戊申",
        "己酉", "庚戌", "辛亥", "壬子", "癸丑", "甲寅", "乙卯", "丙辰", "丁巳", "戊午", "己未", "庚申", "辛酉", "壬戌", "癸亥",
    ];
    fn gz(s: &str) -> u8 {
        GZ.iter().position(|x| *x == s).unwrap() as u8
    }
    /// 卷二诸例：（阴阳, 局, 日干支, 时干支）起盘判格
    fn gj(yang: bool, ju: u8, day: &str, hour: &str, year: Option<&str>, shuo: Option<u8>) -> Vec<GeJu> {
        let sj = ShiJu { yang, ju, term: if yang { 18 } else { 6 }, yuan: Yuan::Shang, run: false, chao: 0 };
        let pan = pai_pan(&sj, gz(hour), DEFAULT_PAN);
        ge_ju(&pan, &GeJuContext { day_gz: gz(day), year_gz: year.map(gz), month_shuo_gan: shuo })
    }
    fn has(v: &[GeJu], name: &str, gong: Option<u8>) -> bool {
        v.iter().any(|g| g.name == name && g.gong == gong)
    }
    fn chk(v: &[GeJu], name: &str, gong: Option<u8>) {
        assert!(has(v, name, gong), "缺 {name} {gong:?}；得 {:?}", v.iter().map(|g| (g.name, g.gong)).collect::<Vec<_>>());
    }

    /// 卷二三遁诸例
    #[test]
    fn san_dun() {
        chk(&gj(true, 4, "乙巳", "乙酉", None, None), "天遁", Some(1)); // 「陽遁四局乙庚之日乙酉時……六丙臨六丁於一宫」
        chk(&gj(false, 6, "戊辰", "庚申", None, None), "天遁", Some(9)); // 「隂遁六局戊癸之日庚申時……於九宫」
        chk(&gj(true, 1, "丙寅", "辛卯", None, None), "地遁", Some(2)); // 「陽遁一局丙辛之日辛卯時……日竒臨六己於二宫」
        chk(&gj(false, 9, "甲子", "丙寅", None, None), "地遁", Some(8)); // 「隂遁九局……丙寅時……開門合六己下臨八宫」
        chk(&gj(true, 7, "乙丑", "丙子", None, None), "人遁", Some(6)); // 「陽遁七局乙庚日丙子時……下臨直符前二六宫太隂中」
    }

    /// 跌穴、返首、玉女守门、三奇得使
    #[test]
    fn ji_ge() {
        chk(&gj(true, 9, "甲子", "辛未", None, None), "飛鳥跌穴", Some(9)); // 「陽遁九局甲巳日辛未時……六丙下臨六甲於九宫戊上」
        chk(&gj(true, 3, "甲子", "丁卯", None, None), "飛鳥跌穴", Some(3)); // 「陽三遁……丁卯時……六丙下臨六甲於三宫」
        chk(&gj(true, 1, "甲子", "丙寅", None, None), "青龍返首", Some(8)); // 「陽遁一局甲巳日丙寅時直符甲子戊與開門相合加於八宫地丙之上」
        chk(&gj(true, 1, "甲子", "庚午", None, None), "玉女守門", Some(7)); // 「陽遁一局甲巳日庚午時……休門直使加地丁七宫」
        chk(&gj(true, 9, "甲子", "庚午", None, None), "三奇得使", Some(3)); // 「陽遁九局甲巳日庚午時……六乙日竒下臨三宫甲午」
        chk(&gj(false, 3, "丙寅", "壬辰", None, None), "三奇得使", Some(9)); // 「隂遁三局丙辛日壬辰時……乙竒……下臨九宫甲午辛」
        chk(&gj(true, 9, "乙丑", "戊寅", None, None), "三奇得使", Some(4)); // 「陽遁九局乙庚日戊寅時……丁竒下臨四宫甲辰」
    }

    /// 奇仪相加诸凶格
    #[test]
    fn xiong_ge() {
        chk(&gj(false, 2, "丙寅", "己亥", None, None), "青龍逃走", Some(8)); // 「隂遁二局丙辛日己亥時……六乙下臨六辛於八宫」
        chk(&gj(false, 2, "甲子", "壬申", None, None), "白虎猖狂", Some(3)); // 「隂遁二局甲巳之日壬申時……天盤六辛下臨於三宫」
        chk(&gj(false, 3, "甲子", "壬申", None, None), "朱雀投江", Some(7)); // 「隂遁三局甲巳日壬申時……六丁下臨六癸於七宫」
        chk(&gj(true, 4, "丙寅", "戊子", None, None), "螣蛇夭矯", Some(1)); // 「陽遁四局丙辛日戊子時……六癸下臨六丁於一宫」
        chk(&gj(false, 4, "甲子", "丙寅", None, None), "大格", Some(8)); // 「隂遁四局甲巳日丙寅時……六庚下臨六癸於八宫」
        chk(&gj(true, 3, "甲子", "丙寅", None, None), "刑格", Some(4)); // 「陽遁三局甲巳日丙寅時……六庚下臨六己於四宫」
        chk(&gj(true, 5, "丙寅", "戊戌", None, None), "熒入白", Some(7)); // 「陽遁五局丙辛之日戊戌時……六丙下臨六庚於七宫」
        chk(&gj(false, 6, "甲子", "丙寅", None, None), "熒入白", Some(4)); // 「隂六局甲巳日丙寅時……六丙下臨六庚於四宫」
        chk(&gj(true, 4, "甲子", "壬申", None, None), "白入熒", Some(2)); // 「陽遁四局甲巳之日壬申時……六庚加臨六丙於二宫」
        chk(&gj(false, 3, "乙丑", "戊寅", None, None), "白入熒", Some(2)); // 「隂遁三局乙庚日戊寅時……六庚下臨六丙於二宫」
    }

    /// 岁月日时格、伏干飞干、天乙伏宫飞宫
    #[test]
    fn geng_ge() {
        // 「辛丑……陽遁五局歲干在辛以甲巳之日癸酉時……天柱六庚下臨六辛歲干於八宫」
        chk(&gj(true, 5, "甲子", "癸酉", Some("辛丑"), None), "歲格", Some(8));
        // 「陽遁八局月朔干在……己……丁卯時……天蓬為六庚下臨月朔干六己於九宫」
        chk(&gj(true, 8, "甲子", "丁卯", None, Some(5)), "月格", Some(9));
        // 「隂遁五局日干在甲巳日丙寅時……六庚……下臨六己於四宫此名日干格」：日干己
        chk(&gj(false, 5, "己巳", "丙寅", None, None), "伏干格", Some(4));
        // 「陽遁二局丙辛日己丑時……直符天輔加時干六己於三宫此為時格」
        chk(&gj(true, 2, "丙寅", "己丑", None, None), "時格", Some(3));
        // 卷二例「陽五局甲申日壬申時……天柱為六庚下臨九宫見今甲申」：壬申属甲子旬，直符天禽（寄坤）而非天柱，
        // 依转盘天柱不临九宫，原例自相出入（见 doc/folk/qimen.md）。另取合于条文者：甲申日甲申时，天柱（庚）伏于七宫，
        // 日干甲遁庚，即「庚加日干」
        chk(&gj(true, 5, "甲申", "甲申", None, None), "伏干格", Some(7));
        // 「陽遁五局甲巳日庚午時……日干六甲下臨六庚於七宫此名飛干格」（甲子日）
        chk(&gj(true, 5, "甲子", "庚午", None, None), "飛干格", Some(7));
        // 「陽遁二局甲巳日壬申時……天輔為六庚下臨直符天芮於二」
        chk(&gj(true, 2, "甲子", "壬申", None, None), "天乙伏宫格", Some(2));
        // 「陽遁九局甲巳之日……庚午時……天英為直符下臨二宫見六庚」
        chk(&gj(true, 9, "甲子", "庚午", None, None), "天乙飛宫格", Some(2));
    }

    /// 入墓、击刑、伏吟反吟、五不遇时、天网、门迫
    #[test]
    fn qita() {
        chk(&gj(false, 4, "丙寅", "庚寅", None, None), "三奇入墓", Some(6)); // 「隂遁四局丙辛日庚寅時六丙月竒下臨六宫」
        chk(&gj(true, 1, "甲子", "庚午", None, None), "六儀擊刑", Some(3)); // 「陽遁一局……庚午時以甲子直符加六庚於三宫」
        chk(&gj(true, 1, "甲子", "甲子", None, None), "伏吟", Some(1)); // 「陽遁一局甲巳之日夜半生甲子時……門符皆伏吟」
        chk(&gj(true, 1, "甲子", "乙丑", None, None), "反吟", Some(9)); // 「陽遁一局甲巳日乙丑時……天蓬直符加臨時干在九宫」
        chk(&gj(false, 4, "乙丑", "己卯", None, None), "天網四張", Some(8)); // 「隂遁四局乙庚日己卯時……天盤癸加地盤癸即天網高八尺」
        // 葛洪「甲日庚午時……癸日己未時」
        for (d, h) in [("甲子", "庚午"), ("乙丑", "辛巳"), ("丙寅", "壬辰"), ("丁卯", "癸卯"), ("戊辰", "甲寅"), ("己巳", "乙丑"), ("庚午", "丙子"), ("辛未", "丁酉"), ("壬申", "戊申"), ("癸酉", "己未")] {
            assert!(has(&gj(true, 1, d, h, None, None), "五不遇時", None), "{d}{h}");
        }
        assert!(!has(&gj(true, 1, "甲子", "辛未", None, None), "五不遇時", None));
        // 时干入墓：丙戌、壬辰、丁丑、癸未、戊戌、己丑
        for h in ["丙戌", "壬辰", "丁丑", "癸未", "戊戌", "己丑"] {
            assert!(has(&gj(true, 1, "甲子", h, None, None), "時干入墓", None), "{h}");
        }
        // 门迫：「開門臨三宫、休門臨九宫、生門臨一宫、景門臨七宫為吉門被迫」
        for p in 1..=9u8 {
            let v = gj(true, 1, "甲子", "甲子", None, None); // 伏吟，八门在本宫，无门迫
            assert!(!has(&v, "門迫", Some(p)));
        }
        assert_eq!(gan_on_pan(gz("甲申")), "庚");
        assert_eq!(gan_on_pan(gz("乙丑")), "乙");
    }

    /// 门迫五行：开门临三（金克木）为迫，开门临二（土生金）不迫
    #[test]
    fn men_po_rule() {
        assert!(ke(men_wx("開門").unwrap(), gong_wx(3)));
        assert!(ke(men_wx("休門").unwrap(), gong_wx(9)));
        assert!(ke(men_wx("生門").unwrap(), gong_wx(1)));
        assert!(ke(men_wx("景門").unwrap(), gong_wx(7)));
        assert!(ke(men_wx("傷門").unwrap(), gong_wx(2)) && ke(men_wx("杜門").unwrap(), gong_wx(8)));
        assert!(ke(men_wx("死門").unwrap(), gong_wx(1)) && ke(men_wx("驚門").unwrap(), gong_wx(4)));
        assert!(!ke(men_wx("開門").unwrap(), gong_wx(2)));
    }
}
