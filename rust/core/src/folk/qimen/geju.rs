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
pub(super) fn gong_wx(p: u8) -> u8 {
    // 0 木 1 火 2 土 3 金 4 水
    [0, 4, 2, 0, 0, 2, 3, 3, 2, 1][p as usize]
}
pub(super) fn men_wx(m: &str) -> Option<u8> {
    Some(match m {
        "休門" => 4,
        "生門" | "死門" => 2,
        "傷門" | "杜門" => 0,
        "景門" => 1,
        "驚門" | "開門" => 3,
        _ => return None,
    })
}
pub(super) fn ke(a: u8, b: u8) -> bool {
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
        // —— 三奇受制：乙临乾六合惊门（木入金乡）、丙临坎一合休门（火入水乡）（卷二引王璋）
        if (t("乙") && p == 6 && men == "驚門") || (t("丙") && p == 1 && men == "休門") {
            add(
                "三奇受制",
                g,
                false,
                "卷二引王璋「乙竒臨乾驚門庚辛囚死乃木入金鄉也丙竒臨坎休門壬癸囚死乃火入水鄉也此謂三竒受制」",
                Some("「庚辛囚死」「壬癸囚死」作受制之由（金克木、水克火）解，未另加地盘干或旺相休囚之条件（未核）；丁奇原文未举"),
            );
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

/// 依月将加时而定的方位格（演义卷二「天三門」「地四戸」「地私門」「天馬」「亭亭白奸」诸条）所需的盘外之数
#[derive(Clone, Copy, Debug)]
pub struct GeJuExtra {
    /// 月将（地支序，子 0 … 亥 11），即太阳所过之次，见 `liuren::yue_jiang`（演义「正月雨水後太陽某日過亥乃登明將」，按中气换将）
    pub yue_jiang: u8,
}

/// 一个方位格：系于地盘十二支（方），附所属洛书宫（子坎一、丑寅艮八、卯震三、辰巳巽四、午离九、未申坤二、酉兑七、戌亥乾六）
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FangGe {
    pub name: &'static str,
    /// 地盘支（方位）
    pub zhi: u8,
    pub gong: u8,
    pub ji: bool,
    pub basis: &'static str,
    pub note: Option<&'static str>,
}

/// 地支所属洛书宫
pub fn zhi_gong(z: u8) -> u8 {
    [1, 8, 8, 3, 4, 4, 9, 2, 2, 7, 6, 6][(z % 12) as usize]
}

/// 演义「地私門」旦暮之分：卷二「旦貴用隂貴……乙巽丙丁坤庚六時」即辰巳午未申酉六时为旦，余为暮
pub fn is_dan(hour_zhi: u8) -> bool {
    (4..=9).contains(&(hour_zhi % 12))
}

/// 地私门三将（六合、太阴、太常）所临地盘之支：月将加时，起本日贵人布十二将。
/// 演义阳贵诗「庚戊見牛甲在羊……六辛逢虎」即六壬俗例夜贵，阴贵诗「甲貴隂牛庚戊羊……辛遇馬」即俗例昼贵，
/// 故 `dan`（旦，用阴贵）对应 `GuiRule::SuLi` 的昼贵。返回 (六合, 太陰, 太常)。
pub fn si_men(yue_jiang: u8, hour_zhi: u8, day_gan: u8, dan: bool) -> (u8, u8, u8) {
    use crate::folk::liuren::{tian_jiang, GuiRule, TianPan};
    let tp = TianPan { jiang: yue_jiang % 12, shi: hour_zhi % 12 };
    let tj = tian_jiang(&tp, day_gan % 10, dan, GuiRule::SuLi);
    let at = |n: &str| tp.below((0..12u8).find(|&z| tj.name_of(z) == n).unwrap());
    (at("六合"), at("太陰"), at("太常"))
}

/// 月将加时诸方位格：天三门、地四户、地私门、天马、亭亭、贵人登天门。白奸原文自相出入，未取（见 doc/folk/qimen.md）。
pub fn fang_ge(pan: &QimenPan, ctx: &GeJuContext, x: &GeJuExtra) -> Vec<FangGe> {
    use crate::folk::liuren::{gui_ren, GuiRule, TianPan};
    let mut out = Vec::new();
    let mut add = |name, zhi: u8, ji, basis, note| out.push(FangGe { name, zhi, gong: zhi_gong(zhi), ji, basis, note });
    let shi = pan.hour_gz % 12;
    let tp = TianPan { jiang: x.yue_jiang % 12, shi };
    // 天三门：太冲卯、小吉未、从魁酉所临（卷二「天三門」引煙波叟）
    for s in [3u8, 7, 9] {
        add("天門", tp.below(s), true, "卷二「天三門」：「以月將加正時順數太冲小吉從魁所臨之方為天門」；例正月雨水後午時登明加午，戌寅辰三方為天門", None);
    }
    // 地四户：时支上加月建，建除顺数，除、定、危、开所临（卷二「地四戸」）
    for k in [1u8, 4, 7, 10] {
        add(
            "地戸",
            (shi + k) % 12,
            true,
            "卷二「地四戸」：「用時支上加月建」，以建除十二神順數，除定危開所臨為地戸；例九月巳時除午定酉危子開夘",
            Some("建加于时支，四户之位只随时支而定，月建不影响方位"),
        );
    }
    // 地私门：六合、太阴、太常所临（卷二「地私門」），旦用阴贵、暮用阳贵
    let (lh, ty, tc) = si_men(x.yue_jiang, shi, ctx.day_gz % 10, is_dan(shi));
    for z in [lh, ty, tc] {
        add("地私門", z, true, "卷二「地私門」：「以月將加用時……起本日貴人……六合太隂太常所臨之方為地私門」", Some("旦（辰至酉六时）用阴贵、暮用阳贵；辰时两例阴阳贵并举，取旦"));
    }
    // 天马：太冲所临（卷二「天馬」）
    add("天馬", tp.below(3), true, "卷二「天馬」：「以月將加時順數至太冲所臨之方」；例正月子時亥加子，夘在辰", None);
    // 亭亭：神后子所临（卷二「亭亭白奸」引王璋）
    add("亭亭", tp.below(0), true, "卷二「亭亭白奸」：「以天月將加所用正時神后所臨之宫即亭亭居其上」；例正月午時子到未", Some("宜背之（坐其上），非吉凶断语"));
    // 贵人登天门：本日贵人（旦阴暮阳）临地盘亥
    let gui = gui_ren(ctx.day_gz % 10, is_dan(shi), GuiRule::SuLi);
    if tp.below(gui) == 11 {
        add("貴人登天門", 11, true, "卷二「天三門」：「如得本日貴人到乾亥就是貴人登天門」", Some("贵人旦暮依地私门条取"));
    }
    out
}

/// 三胜五不击（卷二「亭亭白奸」条后引湯謂）：三胜为天乙（天上直符）宫、九天宫、生门宫；五不击为天乙、九天、生门、九地、直使之宫。
/// 每宫一条，`name` 为「三勝」或「五不擊」，`note` 标所据何神；中五依所选寄宫。不需月将。
pub fn san_sheng_wu_bu_ji(pan: &QimenPan) -> Vec<GeJu> {
    let ji = pan.options.ji_gong.gong(pan.ju.yang);
    let eff = |p: u8| if p == 5 { ji } else { p };
    let find = |f: &dyn Fn(&super::pan::GongCell) -> bool| (1..=9u8).find(|&p| f(pan.at(p))).map(eff);
    let tian_yi = Some(eff(pan.zhi_fu_gong));
    let jiu_tian = find(&|c| c.shen == Some("九天"));
    let sheng = find(&|c| c.men == Some("生門"));
    let jiu_di = find(&|c| c.shen == Some("九地"));
    let zhi_shi = Some(eff(pan.zhi_shi_gong));
    const B3: &str = "卷二「亭亭白奸」條後：天乙宫第一勝、九天第二勝、生門第三勝";
    const B5: &str = "卷二引湯謂「五不可擊第一不擊天乙宫第二不擊九天宫第三不擊生門宫第四不擊九地宫第五不擊直使宫」";
    let mut out = Vec::new();
    for (g, n) in [(tian_yi, "天乙"), (jiu_tian, "九天"), (sheng, "生門")] {
        out.push(GeJu { name: "三勝", gong: g, ji: true, basis: B3, note: Some(n) });
    }
    for (g, n) in [(tian_yi, "天乙"), (jiu_tian, "九天"), (sheng, "生門"), (jiu_di, "九地"), (zhi_shi, "直使")] {
        out.push(GeJu { name: "五不擊", gong: g, ji: true, basis: B5, note: Some(n) });
    }
    out
}

/// 寄宫避五（转盘）：直符所加（时干在中五）或直使所加之宫为五、借寄宫布之者，演义注「避五」。
/// 演义卷三「竒遁佈局法」：「此寄宫終非正位，故遇直符直使在五則皆註避五於本時之下，恐人誤用之也」「凡遇寄宫終非正位，
/// 雖得竒亦不宜用事」；直使在五另据湯謂「凡直使在五宮之時，利客不利主」（《集成》卷七百五「釋陰陽二遁」）。
/// `gong` 为实际所寄之宫（随所选寄宫之说）。飞盘不寄（入中五即中五），此说不及，返回空。
pub fn bi_wu(pan: &QimenPan) -> Vec<GeJu> {
    if pan.options.method != super::pan::PanMethod::ZhuanPan {
        return Vec::new();
    }
    const B: &str = "演义卷三「竒遁佈局法」：「此寄宫終非正位，故遇直符直使在五則皆註避五於本時之下」「凡遇寄宫終非正位，雖得竒亦不宜用事」";
    let ji = pan.options.ji_gong.gong(pan.ju.yang);
    let mut out = Vec::new();
    if pan.zhi_fu_gong == 5 {
        out.push(GeJu { name: "避五", gong: Some(ji), ji: false, basis: B, note: Some("直符在五（时干在中五，直符加寄宫）") });
    }
    if pan.zhi_shi_gong == 5 {
        out.push(GeJu {
            name: "避五",
            gong: Some(ji),
            ji: false,
            basis: B,
            note: Some("直使在五（加寄宫）；湯謂「凡直使在五宮之時，利客不利主」（《集成》卷七百五「釋陰陽二遁」）"),
        });
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

    fn pan(yang: bool, ju: u8, hour: &str) -> QimenPan {
        let sj = ShiJu { yang, ju, term: if yang { 18 } else { 6 }, yuan: Yuan::Shang, run: false, chao: 0 };
        pai_pan(&sj, gz(hour), DEFAULT_PAN)
    }
    const ZHI: &str = "子丑寅卯辰巳午未申酉戌亥";
    fn zs(v: &[FangGe], name: &str) -> String {
        let mut z: Vec<u8> = v.iter().filter(|g| g.name == name).map(|g| g.zhi).collect();
        z.sort();
        z.iter().map(|&i| ZHI.chars().nth(i as usize).unwrap()).collect()
    }
    fn fg(jiang: u8, day: &str, hour: &str) -> Vec<FangGe> {
        fang_ge(&pan(true, 1, hour), &GeJuContext { day_gz: gz(day), ..Default::default() }, &GeJuExtra { yue_jiang: jiang })
    }

    /// 卷二月将加时诸方位格原例（正月雨水后登明亥将）
    #[test]
    fn fang_wei_ge() {
        // 天三门：「正月雨水後……午時……太冲在戌、小吉在寅、從魁在辰……戌寅辰三方為天門」
        assert_eq!(zs(&fg(11, "甲子", "庚午"), "天門"), "寅辰戌");
        // 地四户：「九月……巳時……除在午定在酉危在子開在夘」
        assert_eq!(zs(&fg(3, "甲子", "己巳"), "地戸"), "子卯午酉");
        // 地私门：甲日卯时暮贵（阳贵未加亥顺行）六合寅、太常未、太阴酉
        assert_eq!(zs(&fg(11, "甲子", "丁卯"), "地私門"), "寅未酉");
        // 甲日午时旦贵（阴贵丑在申逆行）太阴戌、太常子、六合巳
        assert_eq!(zs(&fg(11, "甲子", "庚午"), "地私門"), "子巳戌");
        // 甲日子时阳贵在申逆行：同上
        assert_eq!(zs(&fg(11, "甲子", "甲子"), "地私門"), "子巳戌");
        // 辰时两例：阳贵六合卯太常申太阴戌；阴贵六合卯太常戌太阴申
        assert_eq!(si_men(11, 4, 0, false), (3, 10, 8));
        assert_eq!(si_men(11, 4, 0, true), (3, 8, 10));
        // 天马：「正月子時……亥加子……夘在辰」
        assert_eq!(zs(&fg(11, "甲子", "甲子"), "天馬"), "辰");
        // 天马表：正月子至亥时辰巳……寅夘，余月依次
        use crate::folk::liuren::TianPan;
        for m in 0..12u8 {
            for h in 0..12u8 {
                let tp = TianPan { jiang: (11 + 12 - m) % 12, shi: h };
                assert_eq!(tp.below(3), (4 + h + m) % 12);
            }
        }
        // 亭亭：「正月……午時……子到未宫即是亭亭在未上」；「正月夘時……數至辰上得子亭亭在辰」
        assert_eq!(zs(&fg(11, "甲子", "庚午"), "亭亭"), "未");
        assert_eq!(zs(&fg(11, "甲子", "丁卯"), "亭亭"), "辰");
        // 贵人登天门：甲日卯时暮贵未加亥
        assert!(fg(11, "甲子", "丁卯").iter().any(|g| g.name == "貴人登天門"));
        assert!(!fg(11, "甲子", "庚午").iter().any(|g| g.name == "貴人登天門"));
    }

    fn ssw(yang: bool, ju: u8, hour: &str) -> Vec<(&'static str, &'static str, Option<u8>)> {
        san_sheng_wu_bu_ji(&pan(yang, ju, hour)).into_iter().map(|g| (g.name, g.note.unwrap(), g.gong)).collect()
    }
    fn at(v: &[(&str, &str, Option<u8>)], name: &str, who: &str) -> Option<u8> {
        v.iter().find(|x| x.0 == name && x.1 == who).unwrap().2
    }

    /// 三胜五不击原例四则
    #[test]
    fn san_sheng_wu_bu_ji_li() {
        // 「大寒上元陽三局甲巳日丁夘時天上直符乗六丁臨九宫……九天四宫……生門……臨七宫」
        let v = ssw(true, 3, "丁卯");
        assert_eq!((at(&v, "三勝", "天乙"), at(&v, "三勝", "九天"), at(&v, "三勝", "生門")), (Some(9), Some(4), Some(7)));
        // 「隂八局甲巳之日戊辰時地下直符在八宫……九天在三宫……生門臨四宫」
        let v = ssw(false, 8, "戊辰");
        assert_eq!((at(&v, "三勝", "天乙"), at(&v, "三勝", "九天"), at(&v, "三勝", "生門")), (Some(8), Some(3), Some(4)));
        // 「陽八局丙辛日辛夘時天乙在坤二宫……生門在三宫……九地在四宫……直使在八宫」
        let v = ssw(true, 8, "辛卯");
        assert_eq!(
            (at(&v, "五不擊", "天乙"), at(&v, "五不擊", "生門"), at(&v, "五不擊", "九地"), at(&v, "五不擊", "直使")),
            (Some(2), Some(3), Some(4), Some(8))
        );
        // 「隂七局甲巳之日丙寅時天上直符臨九宫……九天在二宫……生門在一宫……九地在七宫……直使在五宫寄坤二宫」
        let v = ssw(false, 7, "丙寅");
        assert_eq!(
            (at(&v, "五不擊", "天乙"), at(&v, "五不擊", "九天"), at(&v, "五不擊", "生門"), at(&v, "五不擊", "九地"), at(&v, "五不擊", "直使")),
            (Some(9), Some(2), Some(1), Some(7), Some(2))
        );
    }

    /// 寄宫避五：演义卷二原例中直符或直使在五者（人遁、月格、三胜五不击诸例）必报，并标所寄之宫；飞盘不报
    #[test]
    fn bi_wu_li() {
        // 人遁「陽遁七局乙庚日丙子時天任為直符加時干六丙於五宫」：直符在五寄坤
        let v = bi_wu(&pan(true, 7, "丙子"));
        assert_eq!(v.iter().map(|g| (g.gong, g.note.unwrap().starts_with("直符"))).collect::<Vec<_>>(), vec![(Some(2), true)]);
        // 三胜五不击「隂七局甲巳之日丙寅時……直使在五宫寄坤二宫」：直使在五
        let v = bi_wu(&pan(false, 7, "丙寅"));
        assert_eq!(v.iter().map(|g| (g.gong, g.note.unwrap().starts_with("直使"))).collect::<Vec<_>>(), vec![(Some(2), true)]);
        // 月格「立春上元陽遁八局……丁邜時……以直符天任加六丁於五宫」；阳坤阴艮之说阴遁寄艮
        assert_eq!(bi_wu(&pan(true, 8, "丁卯")).len(), 1);
        use super::super::pan::{JiGong, PanMethod, PanOptions};
        let sj = ShiJu { yang: false, ju: 7, term: 6, yuan: Yuan::Shang, run: false, chao: 0 };
        let p = pai_pan(&sj, gz("丙寅"), PanOptions { ji_gong: JiGong::KunGen, ..DEFAULT_PAN });
        assert_eq!(bi_wu(&p)[0].gong, Some(8));
        // 直符直使皆不在五则不报（大寒上元陽三局丁夘時，直符臨九宫）
        assert!(bi_wu(&pan(true, 3, "丁卯")).is_empty());
        // 飞盘不寄，不报
        let p = pai_pan(&sj, gz("丙寅"), PanOptions { method: PanMethod::FeiPan, ..DEFAULT_PAN });
        assert!(bi_wu(&p).is_empty());
    }

    /// 三奇受制（原文无盘例）：遍历阴阳十八局六十时，凡天盘乙临六宫合惊门、丙临一宫合休门者必报，反之不报
    #[test]
    fn san_qi_shou_zhi() {
        let mut n = 0;
        for yang in [true, false] {
            for ju in 1..=9u8 {
                for h in 0..60u8 {
                    let p = pai_pan(&ShiJu { yang, ju, term: 0, yuan: Yuan::Shang, run: false, chao: 0 }, h, DEFAULT_PAN);
                    let v = ge_ju(&p, &GeJuContext { day_gz: h, ..Default::default() });
                    for g in 1..=9u8 {
                        let c = p.at(g);
                        let want = (c.tian_pan.contains(&"乙") && g == 6 && c.men == Some("驚門"))
                            || (c.tian_pan.contains(&"丙") && g == 1 && c.men == Some("休門"));
                        assert_eq!(has(&v, "三奇受制", Some(g)), want);
                        n += want as u32;
                    }
                }
            }
        }
        assert!(n > 0);
    }
}
