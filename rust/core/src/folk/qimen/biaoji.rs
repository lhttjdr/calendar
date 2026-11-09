//! 奇门逐宫标记（时家布盘之后，供盘面逐宫标注）：旬空（孤虚）、驿马、截路空亡、暗干、门宫生克（门迫）、六仪击刑、
//! 干墓与星门墓、十干克应（《奇門寶鑑》「混合百神」），并把 [`ge_ju`] 中系于宫的格局按宫归并。
//!
//! 各条出处、异说与逐例检验见 `doc/folk/qimen.md`「逐宫标记」。原典不载、取今人之法者（暗干），注「原典未见」。

use super::geju::{gan_on_pan, ge_ju, gong_wx, ke, men_wx, zhi_gong, GeJuContext};
use super::pan::{QimenPan, XING, YI_QI};

const GAN: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
/// 地支名
pub const ZHI: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];

fn wrap9(p: i32) -> u8 {
    (p - 1).rem_euclid(9) as u8 + 1
}

fn gan_idx(g: &str) -> usize {
    GAN.iter().position(|x| *x == g).expect("干")
}

// —— 旬空、孤虚 ——

/// 某干支所在之旬的空亡二支（孤）：甲子旬戌亥、甲戌旬申酉、甲申旬午未、甲午旬辰巳、甲辰旬寅卯、甲寅旬子丑。
/// 《集成·藝術典》卷七百七「孤虛法」立成表；卷七百三「甲子旬戌亥為孤，辰巳為虛，是以空亡為孤也」。
pub fn xun_kong(gz: u8) -> [u8; 2] {
    let a = (10 + 12 - 2 * ((gz % 60) / 10) as i32).rem_euclid(12) as u8;
    [a, (a + 1) % 12]
}

/// 孤之对冲为虚（《集成》卷七百八凡例「空亡為孤，對衝為虛」）
pub fn gu_xu(gz: u8) -> [u8; 2] {
    let k = xun_kong(gz);
    [(k[0] + 6) % 12, (k[1] + 6) % 12]
}

/// 旬空落宫之法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KongRule {
    /// 空亡二支所属之宫（子坎、丑寅艮、卯震、辰巳巽、午离、未申坤、酉兑、戌亥乾）。缺省：《集成》孤虚表与寶鑑「背孤击虚」皆以支论方
    ZhiGong,
    /// 空亡阳支所当之甲（甲子旬空戌，即甲戌）所遁之仪在地盘之宫：《元靈經》「如甲子旬空戌亥，地盘得甲戌己」
    YiGong,
    /// 二者并取：《元靈經》「地盘得甲戌己或六宫也」
    Both,
}

/// 空亡阳支所当之甲旬（0 甲子 … 5 甲寅）
fn kong_xun(k0: u8) -> usize {
    ((12 - k0 as usize) % 12) / 2
}

/// 旬空所落之宫（升序，不重）
pub fn kong_gong(pan: &QimenPan, gz: u8, rule: KongRule) -> Vec<u8> {
    let k = xun_kong(gz);
    let mut v = Vec::new();
    if rule != KongRule::YiGong {
        v.extend(k.iter().map(|&z| zhi_gong(z)));
    }
    if rule != KongRule::ZhiGong {
        let yi = YI_QI[kong_xun(k[0])];
        for p in 1..=9u8 {
            // 地盘本宫之干；转盘寄宫另带中五之干，亦算
            if pan.at(p).di_pan.contains(&yi) {
                v.push(p);
            }
        }
    }
    v.sort();
    v.dedup();
    v
}

// —— 驿马、截路空亡 ——

/// 驿马：三合局首（长生）之冲，申子辰马在寅、寅午戌在申、巳酉丑在亥、亥卯未在巳。
/// 《協紀辨方書》卷六義例四「驛馬」引曹震圭「假令寅午戌火也病於申……故以申為馬」、儲泳《祛疑說》先天三合数
pub fn yi_ma(zhi: u8) -> u8 {
    [2, 11, 8, 5][(zhi % 4) as usize]
}

/// 截路空亡二时支（以日干）：《元靈經》卷一「截路空亡：此时忌出行。甲己申酉空，乙庚午未中。丙辛辰巳上，丁壬寅卯同。
/// 惟有戊癸日，子丑永无踪」
pub fn jie_lu_kong(day_gan: u8) -> [u8; 2] {
    let a = (8 + 12 - 2 * (day_gan % 5) as i32).rem_euclid(12) as u8;
    [a, a + 1]
}

// —— 暗干 ——

/// 暗干（隐干）起法。《遁甲演義》《奇門寶鑑》《統宗》《元靈經》《集成》奇门诸卷皆无「暗干」「隱干」之名与法（检索见文档），
/// 为今人之法，原典未见，各法皆未核。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnGanRule {
    /// 值使门所临之宫起时干，按六仪三奇之序（戊己庚辛壬癸丁丙乙）依洛书数阳顺阴逆飞布九宫（含中五）；
    /// 值使所临之宫地盘干即时干（如六甲时伏吟）则改从中五起。今人通行（缺省，原典未见）
    ZhiShiZhongWu,
    /// 同上，但不论值使宫地盘干，一律自值使所临之宫起
    ZhiShi,
    /// 不布暗干
    None,
}

/// 暗干：下标 1—9 为洛书宫（0 空）；`None` 法全空。时干为甲取本旬之仪（同 [`gan_on_pan`]）。
/// 值使门所临之宫取盘上该门实际所在之宫（转盘直使数到中五时即寄宫）。
pub fn an_gan(pan: &QimenPan, rule: AnGanRule) -> [&'static str; 10] {
    let mut out = [""; 10];
    if rule == AnGanRule::None {
        return out;
    }
    let shi = gan_on_pan(pan.hour_gz);
    let mut start = (1..=9u8).find(|&p| pan.at(p).men == Some(pan.zhi_shi)).unwrap_or(pan.zhi_shi_gong);
    if rule == AnGanRule::ZhiShiZhongWu && pan.at(start).di_pan.first() == Some(&shi) {
        start = 5;
    }
    let k = YI_QI.iter().position(|x| *x == shi).expect("时干");
    for i in 0..9i32 {
        let p = wrap9(start as i32 + if pan.ju.yang { i } else { -i });
        out[p as usize] = YI_QI[(k + i as usize) % 9];
    }
    out
}

// —— 门宫生克 ——

/// 门与宫五行之关系：《奇門寶鑑》「注释烟波钓叟歌」：「宫制其门曰：门迫。门制其宫曰：宫迫。门生宫为和，宫生门为义」。
/// 寶鑑此处「门迫」「宫迫」二名与演义相反（演义卷二「宫制其門不為迫，門制其宫是迫雄」以门克宫为迫），寶鑑同条所举
/// 「开门临三四宫……此为吉门被迫」亦是门克宫，故本项目统以门克宫为「迫」，宫克门为「制」
pub fn men_gong(men: &str, p: u8) -> Option<&'static str> {
    let m = men_wx(men)?;
    if p == 5 {
        return None;
    }
    let g = gong_wx(p);
    let sheng = |a: u8, b: u8| (a + 1) % 5 == b; // 木生火、火生土、土生金、金生水、水生木
    Some(if m == g {
        "比"
    } else if ke(m, g) {
        "迫"
    } else if ke(g, m) {
        "制"
    } else if sheng(m, g) {
        "和"
    } else {
        debug_assert!(sheng(g, m));
        "義"
    })
}

// —— 击刑 ——

/// 六仪击刑之法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JiXingRule {
    /// 只论直符（旬首之仪）：演义卷二引王璋「甲子直符加三宫……甲寅直符臨四宫」。缺省
    ZhiFu,
    /// 天盘六仪任一临其刑宫：《奇門寶鑑》「释六仪击刑」「甲子戊仪加三宫，甲戌己仪加二宫，甲申庚仪加八宫，
    /// 甲午辛仪加九宫，甲辰壬仪、甲寅癸仪加四宫，为击刑」
    LiuYi,
    /// 同上，兼取寶鑑「一本甲戌加八宫，甲申加四宫，甲寅加二宫亦是」
    LiuYiYiBen,
}

/// 六仪所刑之宫（戊三、己二、庚八、辛九、壬四、癸四）；一本另加己八、庚四、癸二
fn xing_gong(yi: &str, yi_ben: bool) -> &'static [u8] {
    match (yi, yi_ben) {
        ("戊", _) => &[3],
        ("己", false) => &[2],
        ("己", true) => &[2, 8],
        ("庚", false) => &[8],
        ("庚", true) => &[8, 4],
        ("辛", _) => &[9],
        ("壬", _) => &[4],
        ("癸", false) => &[4],
        ("癸", true) => &[4, 2],
        _ => &[],
    }
}

/// p 宫击刑之仪（天盘）
pub fn ji_xing(pan: &QimenPan, p: u8, rule: JiXingRule) -> Vec<&'static str> {
    pan.at(p)
        .tian_pan
        .iter()
        .copied()
        .filter(|y| rule != JiXingRule::ZhiFu || *y == pan.xun_yi)
        .filter(|y| xing_gong(y, rule == JiXingRule::LiuYiYiBen).contains(&p))
        .collect()
}

// —— 入墓 ——

/// 十干入墓之法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GanMuRule {
    /// 十干长生阳顺阴逆之墓：甲未、乙戌、丙戊戌、丁己丑、庚丑、辛壬辰、癸未。
    /// 《統宗》「入墓丁艮乙丙乾」注「乙阴木长生在午墓于戌，丙火长生在寅墓于戌……丁阴火长生在酉墓于丑」；
    /// 演义卷二时干入墓「戊戌壬辰兼丙戌，癸未丁丑」及葛洪「己丑」六时，恰为六十甲子中干墓于本支者之全部（见测试）。缺省
    YangShunYinNi,
    /// 五行之墓（阳干阴干同）：甲乙墓未、丙丁墓戌、庚辛墓丑、壬癸墓辰，戊己随 [`TuMu`]。
    /// 《奇門寶鑑》「释奇墓」「乙木墓于未，丙丁火墓于戌，故乙奇墓二、丙丁墓六」
    WuXing,
}

/// 土之墓（五行墓法与星门墓用）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TuMu {
    /// 土随火墓戌（乾六）：演义卷二时干入墓「戊戌」。缺省
    Xu,
    /// 土随水墓辰（巽四）：《奇門寶鑑》「释奇墓」时干入墓改「戊辰」（「旧本有戊[戌]……今改正之」）
    Chen,
}

/// 五行（0 木 1 火 2 土 3 金 4 水）之墓支
fn wx_mu(wx: u8, tu: TuMu) -> u8 {
    match wx {
        0 => 7,
        1 => 10,
        2 => match tu {
            TuMu::Xu => 10,
            TuMu::Chen => 4,
        },
        3 => 1,
        _ => 4,
    }
}

/// 十干墓支
pub fn gan_mu_zhi(gan: &str, rule: GanMuRule, tu: TuMu) -> u8 {
    let i = gan_idx(gan);
    match rule {
        GanMuRule::YangShunYinNi => [7, 10, 10, 1, 10, 1, 1, 4, 4, 7][i],
        GanMuRule::WuXing => wx_mu([0, 0, 1, 1, 2, 2, 3, 3, 4, 4][i], tu),
    }
}

/// 九星五行：蓬水、芮土、冲木、辅木、禽土、心金、柱金、任土、英火
fn xing_wx(x: &str) -> u8 {
    let p = XING.iter().position(|s| *s == x).expect("星");
    [0, 4, 2, 0, 0, 2, 3, 3, 2, 1][p]
}

/// 选项
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BiaoJiOptions {
    pub kong: KongRule,
    pub an_gan: AnGanRule,
    pub ji_xing: JiXingRule,
    pub gan_mu: GanMuRule,
    pub tu_mu: TuMu,
}

/// 缺省：旬空以支论宫、暗干值使宫起（中五例外）、击刑只论直符、干墓阳顺阴逆、土墓戌
pub const DEFAULT_BIAO_JI: BiaoJiOptions =
    BiaoJiOptions { kong: KongRule::ZhiGong, an_gan: AnGanRule::ZhiShiZhongWu, ji_xing: JiXingRule::ZhiFu, gan_mu: GanMuRule::YangShunYinNi, tu_mu: TuMu::Xu };

/// 十干克应一条（天盘干加地盘干）
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeYing {
    pub tian: &'static str,
    pub di: &'static str,
    /// 寶鑑所立之名（如「青龙返首」）
    pub name: &'static str,
    /// 断语
    pub text: &'static str,
}

/// 一宫之标记
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GongBiaoJi {
    /// 时旬空亡落此宫
    pub kong: bool,
    /// 时支驿马落此宫
    pub ma: bool,
    /// 暗干（不布则空串）
    pub an_gan: &'static str,
    /// 门宫生克：迫、制、和、義、比（无门或中五为 None）
    pub men_gong: Option<&'static str>,
    /// 击刑之仪（天盘）
    pub ji_xing: Vec<&'static str>,
    /// 天盘干入墓者
    pub gan_mu: Vec<&'static str>,
    /// 天盘九星入墓者（五行墓）
    pub xing_mu: Vec<&'static str>,
    /// 八门入墓（五行墓）
    pub men_mu: bool,
    /// 十干克应（天盘各干 × 地盘各干）
    pub ke_ying: Vec<KeYing>,
    /// [`ge_ju`] 中系于本宫之格名，另加地罗遮蔽
    pub ge: Vec<&'static str>,
}

/// 全盘标记
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PanBiaoJi {
    pub options: BiaoJiOptions,
    /// 时旬空亡二支（孤）与其冲（虚）
    pub xun_kong: [u8; 2],
    pub gu_xu: [u8; 2],
    /// 旬空所落之宫
    pub kong_gong: Vec<u8>,
    /// 日旬空亡二支（供参考，不标宫）
    pub ri_xun_kong: [u8; 2],
    /// 时支驿马与其宫
    pub ma_zhi: u8,
    pub ma_gong: u8,
    /// 本时是否截路空亡（以日干）
    pub jie_lu_kong: bool,
    /// 下标 0—8 对应洛书 1—9 宫
    pub gong: [GongBiaoJi; 9],
}

impl PanBiaoJi {
    pub fn at(&self, p: u8) -> &GongBiaoJi {
        &self.gong[p as usize - 1]
    }
}

/// 十干克应：天盘 t 加地盘 d（甲作戊：寶鑑「留甲十干寄戊同安」）
pub fn ke_ying(t: &'static str, d: &'static str) -> KeYing {
    const ORD: &str = "甲乙丙丁己庚辛壬癸";
    let ix = |g: &str| {
        let g = if g == "戊" { "甲" } else { g };
        ORD.chars().position(|c| c.to_string() == g).expect("干")
    };
    let (name, text) = KE_YING[ix(t) * 9 + ix(d)];
    KeYing { tian: t, di: d, name, text }
}

/// 逐宫标记
pub fn biao_ji(pan: &QimenPan, ctx: &GeJuContext, opts: BiaoJiOptions) -> PanBiaoJi {
    let h = pan.hour_gz % 60;
    let shi_zhi = h % 12;
    let kg = kong_gong(pan, h, opts.kong);
    let ma_zhi = yi_ma(shi_zhi);
    let ma_gong = zhi_gong(ma_zhi);
    let jl = jie_lu_kong(ctx.day_gz % 10);
    let ag = an_gan(pan, opts.an_gan);
    let gj = ge_ju(pan, ctx);
    let shi_gan = gan_on_pan(h);
    let mut gong: [GongBiaoJi; 9] = Default::default();
    for p in 1..=9u8 {
        let c = pan.at(p);
        let mu_p = |z: u8| p != 5 && zhi_gong(z) == p;
        let mut ge: Vec<&'static str> = Vec::new();
        for g in gj.iter().filter(|g| g.gong == Some(p)) {
            if !ge.contains(&g.name) {
                ge.push(g.name);
            }
        }
        // 地罗遮蔽：天盘壬加地盘时干（《集成》卷七百八凡例「地網遮　六壬臨時干」；《統宗》格局表「地罗遮蔽　六壬临时干」）
        if c.tian_pan.contains(&"壬") && c.di_pan.contains(&shi_gan) {
            ge.push("地羅遮蔽");
        }
        let mut ky = Vec::new();
        for &t in &c.tian_pan {
            for &d in &c.di_pan {
                ky.push(ke_ying(t, d));
            }
        }
        gong[p as usize - 1] = GongBiaoJi {
            kong: kg.contains(&p),
            ma: ma_gong == p,
            an_gan: ag[p as usize],
            men_gong: c.men.and_then(|m| men_gong(m, p)),
            ji_xing: ji_xing(pan, p, opts.ji_xing),
            gan_mu: c.tian_pan.iter().copied().filter(|g| mu_p(gan_mu_zhi(g, opts.gan_mu, opts.tu_mu))).collect(),
            xing_mu: c.xing.iter().copied().filter(|x| mu_p(wx_mu(xing_wx(x), opts.tu_mu))).collect(),
            men_mu: c.men.and_then(men_wx).is_some_and(|w| mu_p(wx_mu(w, opts.tu_mu))),
            ke_ying: ky,
            ge,
        };
    }
    PanBiaoJi {
        options: opts,
        xun_kong: xun_kong(h),
        gu_xu: gu_xu(h),
        kong_gong: kg,
        ri_xun_kong: xun_kong(ctx.day_gz),
        ma_zhi,
        ma_gong,
        jie_lu_kong: jl.contains(&shi_zhi),
        gong,
    }
}

/// 《奇門寶鑑》「混合百神」十干克应：行为天盘、列为地盘，序甲（即戊）乙丙丁己庚辛壬癸。
/// 录文（简体，未注底本）照录；「六辛加卒」为「六辛加辛」之讹，已正。
#[rustfmt::skip]
const KE_YING: [(&str, &str); 81] = [
    ("青龙出地", "喜信必来，门合则美，门塞星凶，空有财至。"), // 甲加甲
    ("青龙入云", "三奇门交，贵子生成，星干不利，虚得其名。"), // 甲加乙
    ("青龙返首", "凡事亨通，兼得长久，门仪不合，亦难得就。"), // 甲加丙
    ("青龙耀明", "三门合吉，宜谒贵人，改官迁职，大有英名，符事役凶，立待词刑。 （二句有误）"), // 甲加丁
    ("青龙合灵", "吉星主财，吉门事成，星门不合，徒费精神。"), // 甲加己
    ("青龙符格", "起咎成凶，在于不测，星吉门顺，亦宜静默。"), // 甲加庚
    ("青龙失惊", "门中一合，万事从心，凶星上立，财利亡倾。"), // 甲加辛
    ("青龙网罗", "阴人用之，灾祸弥多，阳人用之，诡谲不和。"), // 甲加壬
    ("青龙华盖", "门合吉星，永无灾害，若伤死门，奸阴阳会。"), // 甲加癸
    ("阴中返阳", "凶星财破，人口损伤，阴人才合，阳人慌张。"), // 乙加甲
    ("日奇伏刑", "贵人问之，主失其名，门合再叙，门逆丧停。"), // 乙加乙
    ("奇仪顺格", "吉星临之，授官迁职，夫占其妻，必有离隔。"), // 乙加丙
    ("朱雀入墓", "文书入官，架阁留住，星门吉泰，文词得路。"), // 乙加丁
    ("日奇入雾", "土木混同，雨有交互，求事得微，门乘必悞。"), // 乙加己
    ("日奇自刑", "鬬起争财，必入讼庭，星干不吉，夫妻外情。"), // 乙加庚
    ("青龙逃走", "失财逃亡，二事俱丑，强立强为，定无长久。"), // 乙加辛
    ("青龙得云", "阳人主失，病是阴人。"), // 乙加壬
    ("日入天网", "阴人望信，立见灾恙，官事失财，凡事虚妄。"), // 乙加癸
    ("飞鸟跌穴", "贵面天颜，荣迁越职，所有图谋，一切通彻。"), // 丙加甲
    ("月奇浮云", "贵人印信，即可敷陈，公私利亨，百事称心。"), // 丙加乙
    ("月奇勃格", "两重文书，皆遭障格，门逆财亡，门顺虚迫。"), // 丙加丙
    ("奇入朱雀", "文书亨通，贵占权握，常人得之，衣禄退剥。"), // 丙加丁
    ("火孛入刑", "文书不来，狱中有人，星吉门顺，空禁虚名，星乘门逆，徒杖而刑。"), // 丙加己
    ("荧入太白", "家事熬煎，又逢盗贼，门合星吉，贼在人获，星门相遗，祸生因客。"), // 丙加庚
    ("月精合佑", "久病之人，药师来救，文状入官，亦能成就。"), // 丙加辛
    ("孛乱来临", "文讼公庭，因淫妇人，庶人流离，贵人失名。"), // 丙加壬
    ("华盖孛师", "阴人用之，灾祸相随，必因词讼，后必无亏。"), // 丙加癸
    ("青龙得光", "贵人迁职，常人得良，星符合处，喜美非常。"), // 丁加甲
    ("格为人遁", "得喜非常，贵人荐论，改禄受权，实为广运。"), // 丁加乙
    ("加中复奇", "多生口舌，事生跷蹊，贵招官禄，常人刑之。"), // 丁加丙
    ("奇入太阴", "望文书至，远大名深，两重文意，凡百遂心，若信近用，名为伏吟。"), // 丁加丁
    ("火入勾神", "文状词凶，奸因妇人，私中有私，往则刑名。"), // 丁加己
    ("织女寻牛", "不有私情，定遇冤雠，阴人无理，刑禁官囚。"), // 丁加庚
    ("朱雀入狱", "官人刑囚，亦遭剥落，常人枷锁，百日放却。"), // 丁加辛
    ("五人相和", "丁壬化木，财利得多，贵人赐禄，文状平和。"), // 丁加壬
    ("朱雀沉江", "女人官府，定争成双，公私不协。两有所伤。"), // 丁加癸
    ("伏格青龙", "门合吉星，财利是隆，门逆凶星，所干成空。"), // 己加甲
    ("墓入不明", "星门合吉，且喜平平，不合而逆，凡干不成。"), // 己加乙
    ("格名孛师", "阳人宣赐，爵禄加之，阴人大忌，奸乱乘违。"), // 己加丙
    ("奇入墓名", "文书当诉，门合吉神，先论得理，后对遭惩。"), // 己加丁
    ("地户逢鬼", "阴人望信，信者难委，门符合处，远近来至。"), // 己加己
    ("刑格之名", "阴人发用，必得阳人，阳宜静默，阴主私情。"), // 己加庚
    ("魂神入墓", "家有阴人，鬼妖惊户，符门合处，小口灾遇。"), // 己加辛
    ("刑网高张", "阴人奸恶，阳人遭伤，门迫星凶，两人俱亡。"), // 己加壬
    ("地刑元武", "阴人沉吟，灾病不语，门合星扶，遂成疾苦。"), // 己加癸
    ("刑青龙格", "财利多荣，星门法则，不合则凶，合则免厄。"), // 庚加甲
    ("日合六格", "百事安然，尤当缄默，星门乘凶，官事刑迫。"), // 庚加乙
    ("太白入荧", "失物被盗，难获难寻，门合星吉，赃物知情，符门不吉，官事来临。"), // 庚加丙
    ("名曰亭亭", "文状争论，私匿之情，符门逆背，词讼难成。"), // 庚加丁
    ("刑格凶否", "狱囚之人，难有伸理。"), // 庚加己
    ("太白之名", "官事并发，狱禁平人，凶期百日，却有舒情。"), // 庚加庚
    ("干格白虎", "道路伤亡，必失伴侣，有事难休，定执客主。"), // 庚加辛
    ("蛇格之名", "阴阳占信，迷路无音，伤门主灾，死门丧停。"), // 庚加壬
    ("大刑之格", "占路远求，阴人疾厄，鬼贼相扶，遂生离格。"), // 庚加癸
    ("龙困遭伤", "系官争财，阴人不妨，阳人有灾，门顺吉昌。"), // 辛加甲
    ("白虎猖狂", "失财破家，人口死亡，远行失信，家事彷徨。"), // 辛加乙
    ("干合荧惑", "文状虚词，竞争财物，星门不合，暗昧屈厄。"), // 辛加丙
    ("狱神入奇", "远行经商，利倍得迟，星门不吉，夫妇分离。"), // 辛加丁
    ("刑狱之格", "奴婢欺主，先自刑克，门吉星强，虚成累及。"), // 辛加己
    ("白虎伤格", "两女争男，皆因酒色，星合门凶，丑声难塞。"), // 辛加庚
    ("狱入自刑", "求财喜合，利用阴人，阳人用之，灾害时临。"), // 辛加辛
    ("蛇入狱刑", "两男争女，讼逼难停，门符合吉，首者遭黥。"), // 辛加壬
    ("直格华盖", "阴人用之，无灾无害，吉门吉星，阳人得财，兼有酒食，喜庆咸来。"), // 辛加癸
    ("蛇化为龙", "阴人用事，喜庆重重，阳人求事，有始无终。"), // 壬加甲
    ("格名小蛇", "阴人伏灾，阳人伤嗟，孕生贵子，禄马光华。"), // 壬加乙
    ("蛇入冶炉", "词讼争端，空事无图，若遇刑禁，出则招徒。"), // 壬加丙
    ("干合蛇刑", "文书财喜，大宜阴人，贵人官禄，常人平平。"), // 壬加丁
    ("蛇凶入狱", "大祸将成，夫妻不睦，罪若有刑，金银沾足。"), // 壬加己
    ("太白骑蛇", "刑狱公明，好分正邪，如逢伤死，刑戮无差。"), // 壬加庚
    ("螣蛇格干", "符门虽吉，亦不可安，所有运谋，内生欺瞒。"), // 壬加辛
    ("罗网自缠", "阴人利用，阳人莫前，三吉不入，万事无缘。"), // 壬加壬
    ("螣蛇飞空", "家不和睦，室女私通，星门俱吉，信息隆隆。"), // 壬加癸
    ("罗网青龙", "财喜姻亲，必遇吉人，星门不合，阳人讼刑。"), // 癸加甲
    ("华盖逢星", "贵人宠禄，权位如轻，常人怪异，口舌来临。"), // 癸加乙
    ("盖遇孛师", "贵人受官，小人得依，若占文状，定得门眉。"), // 癸加丙
    ("螣蛇夭矫", "阴人文书，为凶之兆，损财招刑，不宜叠调。"), // 癸加丁
    ("华盖地户", "阴人问夫，求至居住，门顺入室，门逆夫殆。"), // 癸加己
    ("大格飞名", "只宜上官，握柄雄声，公事得迟，钱则有争。"), // 癸加庚
    ("狱入天牢", "军吏遭系，罪恐难逃，门吉星吉，虚禁无劳。"), // 癸加辛
    ("复见螣蛇", "阴人绝子，又复离家，理顺后嫁，未保年华。"), // 癸加壬
    ("天网高张", "行旅失约，游去四方，合处相得，逆处相伤。"), // 癸加癸
];

#[cfg(test)]
mod tests {
    use super::super::pan::{pai_pan, PanMethod, PanOptions, DEFAULT_PAN, MEN};
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
    fn zs(v: &[u8]) -> String {
        v.iter().map(|&z| ZHI[z as usize]).collect()
    }
    fn sj(yang: bool, ju: u8) -> ShiJu {
        ShiJu { yang, ju, term: if yang { 18 } else { 6 }, yuan: Yuan::Shang, run: false, chao: 0 }
    }
    fn pan(yang: bool, ju: u8, h: &str) -> QimenPan {
        pai_pan(&sj(yang, ju), gz(h), DEFAULT_PAN)
    }
    fn all_pans(o: PanOptions) -> Vec<QimenPan> {
        let mut v = Vec::new();
        for yang in [true, false] {
            for ju in 1..=9u8 {
                for h in 0..60u8 {
                    v.push(pai_pan(&sj(yang, ju), h, o));
                }
            }
        }
        v
    }

    /// 《集成》卷七百七「孤虛法」立成表：六旬之孤与虚
    #[test]
    fn gu_xu_table() {
        let t = [("甲子", "戌亥", "辰巳"), ("甲戌", "申酉", "寅卯"), ("甲申", "午未", "子丑"), ("甲午", "辰巳", "戌亥"), ("甲辰", "寅卯", "申酉"), ("甲寅", "子丑", "午未")];
        for (x, gu, xu) in t {
            for k in 0..10 {
                let h = gz(x) + k;
                assert_eq!(zs(&xun_kong(h)), gu, "{x}旬孤");
                assert_eq!(zs(&gu_xu(h)), xu, "{x}旬虚");
            }
        }
    }

    /// 《元靈經》「如甲子旬空戌亥，地盘得甲戌己或六宫也」：阳一局地盘己在坤二
    #[test]
    fn kong_gong_yuanling() {
        let p = pan(true, 1, "乙丑");
        assert_eq!(kong_gong(&p, p.hour_gz, KongRule::ZhiGong), vec![6]);
        assert_eq!(kong_gong(&p, p.hour_gz, KongRule::YiGong), vec![2]);
        assert_eq!(kong_gong(&p, p.hour_gz, KongRule::Both), vec![2, 6]);
        // 甲寅旬空子丑：坎一、艮八
        let p = pan(true, 1, "乙卯");
        assert_eq!(kong_gong(&p, p.hour_gz, KongRule::ZhiGong), vec![1, 8]);
        // 甲午旬空辰巳同在巽四
        let p = pan(true, 1, "乙未");
        assert_eq!(kong_gong(&p, p.hour_gz, KongRule::ZhiGong), vec![4]);
        // 甲午旬空辰（甲辰壬）：阳一局壬在中五，转盘寄坤，二宫亦带壬
        assert_eq!(kong_gong(&p, p.hour_gz, KongRule::YiGong), vec![2, 5]);
    }

    /// 《協紀辨方書》卷六「驛馬」：寅午戌馬申、申子辰馬寅、巳酉丑馬亥、亥卯未馬巳
    #[test]
    fn yi_ma_xieji() {
        let want = "寅亥申巳寅亥申巳寅亥申巳"; // 子丑寅卯……亥
        for z in 0..12u8 {
            assert_eq!(ZHI[yi_ma(z) as usize], want.chars().nth(z as usize).unwrap().to_string(), "{}", ZHI[z as usize]);
        }
        assert_eq!((zhi_gong(yi_ma(0)), zhi_gong(yi_ma(2)), zhi_gong(yi_ma(1)), zhi_gong(yi_ma(3))), (8, 2, 6, 4));
    }

    /// 《元靈經》卷一截路空亡五句
    #[test]
    fn jie_lu_kong_yuanling() {
        let t = [("甲", "申酉"), ("己", "申酉"), ("乙", "午未"), ("庚", "午未"), ("丙", "辰巳"), ("辛", "辰巳"), ("丁", "寅卯"), ("壬", "寅卯"), ("戊", "子丑"), ("癸", "子丑")];
        for (g, w) in t {
            assert_eq!(zs(&jie_lu_kong(gan_idx(g) as u8)), w, "{g}");
        }
    }

    /// 《奇門寶鑑》「释迫」：「开门临三、四宫，休门临九宫，生门临一宫，景门临六、七宫，伤、杜二门临二、八宫，死门临一宫，
    /// 惊门临三、四宫，凡此为迫」——八门×八宫逐一比对，迫者恰为此十三项
    #[test]
    fn men_po_baojian() {
        let want = [("開門", 3), ("開門", 4), ("休門", 9), ("生門", 1), ("景門", 6), ("景門", 7), ("傷門", 2), ("傷門", 8), ("杜門", 2), ("杜門", 8), ("死門", 1), ("驚門", 3), ("驚門", 4)];
        let mut got = Vec::new();
        for m in MEN.iter().filter(|m| !m.is_empty()) {
            for p in [1u8, 2, 3, 4, 6, 7, 8, 9] {
                if men_gong(m, p) == Some("迫") {
                    got.push((*m, p));
                }
            }
        }
        got.sort();
        let mut w = want.to_vec();
        w.sort();
        assert_eq!(got, w);
        // 门生宫为和、宫生门为义
        assert_eq!(men_gong("休門", 3), Some("和"));
        assert_eq!(men_gong("休門", 6), Some("義"));
        assert_eq!(men_gong("休門", 1), Some("比"));
        assert_eq!(men_gong("休門", 2), Some("制"));
        assert_eq!(men_gong("休門", 5), None);
    }

    /// 六仪击刑：演义例「陽遁一局……庚午時以甲子直符加六庚於三宫」；寶鑑诸仪；直符法与 ge_ju 之「六儀擊刑」一致
    #[test]
    fn ji_xing_rules() {
        let p = pan(true, 1, "庚午");
        assert_eq!(ji_xing(&p, 3, JiXingRule::ZhiFu), vec!["戊"]);
        let (mut n_zf, mut n_ly, mut n_yb) = (0, 0, 0);
        for pn in all_pans(DEFAULT_PAN) {
            let gj = ge_ju(&pn, &GeJuContext { day_gz: pn.hour_gz, ..Default::default() });
            for g in 1..=9u8 {
                let a = ji_xing(&pn, g, JiXingRule::ZhiFu);
                let b = ji_xing(&pn, g, JiXingRule::LiuYi);
                let c = ji_xing(&pn, g, JiXingRule::LiuYiYiBen);
                assert!(a.iter().all(|y| b.contains(y)) && b.iter().all(|y| c.contains(y)));
                // 寶鑑：天盘某仪在 g 宫且 g 为其刑宫
                for y in &pn.at(g).tian_pan {
                    let hit = matches!((*y, g), ("戊", 3) | ("己", 2) | ("庚", 8) | ("辛", 9) | ("壬", 4) | ("癸", 4));
                    assert_eq!(b.contains(y), hit);
                }
                // 直符不在中五时，与格局之「六儀擊刑」同
                if pn.zhi_fu_gong != 5 {
                    let geju_hit = gj.iter().any(|x| x.name == "六儀擊刑" && x.gong == Some(g));
                    assert_eq!(!a.is_empty(), geju_hit, "{:?} {} {g}", pn.ju, GZ[pn.hour_gz as usize]);
                }
                n_zf += a.len();
                n_ly += b.len();
                n_yb += c.len();
            }
        }
        assert!(n_zf > 0 && n_ly > n_zf && n_yb > n_ly);
    }

    /// 干墓：演义卷二时干入墓六时恰为阳顺阴逆法中干墓于本支之全部；寶鑑、統宗、集成、王璋诸说之三奇墓宫
    #[test]
    fn gan_mu_rules() {
        let yy: Vec<&str> = (0..60u8).filter(|&h| gan_mu_zhi(GAN[(h % 10) as usize], GanMuRule::YangShunYinNi, TuMu::Xu) == h % 12).map(|h| GZ[h as usize]).collect();
        let mut want = vec!["丙戌", "壬辰", "丁丑", "癸未", "戊戌", "己丑"];
        want.sort();
        let mut got = yy.clone();
        got.sort();
        assert_eq!(got, want);
        let g = |gan, r| zhi_gong(gan_mu_zhi(gan, r, TuMu::Xu));
        // 寶鑑「乙奇墓二、丙丁墓六」
        assert_eq!((g("乙", GanMuRule::WuXing), g("丙", GanMuRule::WuXing), g("丁", GanMuRule::WuXing)), (2, 6, 6));
        // 統宗「入墓丁艮乙丙乾」
        assert_eq!((g("乙", GanMuRule::YangShunYinNi), g("丙", GanMuRule::YangShunYinNi), g("丁", GanMuRule::YangShunYinNi)), (6, 6, 8));
        // 演义引王璋乙二、丙六、丁八：乙从五行，丙丁从阳顺阴逆；《集成》凡例「乙奇加二宮及六宮，丙奇加六宮」两说兼取
        assert_eq!(g("乙", GanMuRule::WuXing), 2);
        assert_eq!(g("丁", GanMuRule::YangShunYinNi), 8);
        // 土墓：戌（演义戊戌）、辰（寶鑑改戊辰）
        assert_eq!(zhi_gong(gan_mu_zhi("戊", GanMuRule::WuXing, TuMu::Xu)), 6);
        assert_eq!(zhi_gong(gan_mu_zhi("戊", GanMuRule::WuXing, TuMu::Chen)), 4);
    }

    /// 门墓：《元靈經》卷十一占胎孕「如伤杜门到（坤）宫，为入墓」
    #[test]
    fn men_mu_yuanling() {
        let mut n = 0;
        for pn in all_pans(DEFAULT_PAN) {
            let b = biao_ji(&pn, &GeJuContext { day_gz: pn.hour_gz, ..Default::default() }, DEFAULT_BIAO_JI);
            let m2 = pn.at(2).men.unwrap();
            if m2 == "傷門" || m2 == "杜門" {
                assert!(b.at(2).men_mu);
                n += 1;
            }
            for g in [1u8, 3, 4, 6, 7, 8, 9] {
                let m = pn.at(g).men.unwrap();
                if m == "傷門" || m == "杜門" {
                    assert!(!b.at(g).men_mu);
                }
            }
        }
        assert!(n > 0);
    }

    /// 暗干（今人之法，原典未见）：自洽检验
    #[test]
    fn an_gan_rules() {
        // 阳一局甲子时伏吟：值使休门在坎，地盘戊即时干（甲作戊），中五法改从中五起
        let p = pan(true, 1, "甲子");
        let a = an_gan(&p, AnGanRule::ZhiShiZhongWu);
        assert_eq!(a[5], "戊");
        let b = an_gan(&p, AnGanRule::ZhiShi);
        for q in 1..=9u8 {
            assert_eq!(b[q as usize], p.at(q).di_pan[0], "伏吟时暗干即地盘 {q}");
        }
        assert!(an_gan(&p, AnGanRule::None).iter().all(|x| x.is_empty()));
        for o in [DEFAULT_PAN, PanOptions { method: PanMethod::FeiPan, ..DEFAULT_PAN }] {
            for pn in all_pans(o) {
                let shi = gan_on_pan(pn.hour_gz);
                for r in [AnGanRule::ZhiShiZhongWu, AnGanRule::ZhiShi] {
                    let a = an_gan(&pn, r);
                    let mut s: Vec<&str> = a[1..].to_vec();
                    s.sort();
                    s.dedup();
                    assert_eq!(s.len(), 9);
                    let men_at = (1..=9u8).find(|&q| pn.at(q).men == Some(pn.zhi_shi)).unwrap();
                    let start = (1..=9u8).find(|&q| a[q as usize] == shi).unwrap();
                    assert!(start == men_at || (r == AnGanRule::ZhiShiZhongWu && start == 5));
                }
            }
        }
    }

    /// 十干克应：与寶鑑「混合百神」录文逐条比对；与演义卷二诸格之名相应者
    #[test]
    fn ke_ying_baojian() {
        assert_eq!(ke_ying("丙", "戊").name, "飞鸟跌穴");
        assert_eq!(ke_ying("戊", "丙").name, "青龙返首");
        assert_eq!(ke_ying("乙", "辛").name, "青龙逃走");
        assert_eq!(ke_ying("辛", "乙").name, "白虎猖狂");
        assert_eq!(ke_ying("癸", "丁").name, "螣蛇夭矫");
        assert_eq!(ke_ying("庚", "丙").name, "太白入荧");
        assert_eq!(ke_ying("丙", "庚").name, "荧入太白");
        assert_eq!(ke_ying("辛", "辛").name, "狱入自刑");
        let path = crate::repo::repo_root().join("vendor/folk/wikisource/奇門寶鑑/全文.wikitext");
        let Ok(t) = std::fs::read_to_string(path) else {
            println!("skipped: 无寶鑑录文");
            return;
        };
        let i = t.find("六甲加甲").unwrap();
        let j = t[i..].find("昔日举要").unwrap() + i;
        let re = regex::Regex::new(r"六([甲乙丙丁己庚辛壬癸])(?:加|交)([甲乙丙丁己庚辛壬癸卒])[；：]\s*([^，\n]+)").unwrap();
        let mut n = 0;
        for c in re.captures_iter(&t[i..j]) {
            let a: &'static str = GAN[gan_idx(&c[1])];
            let b: &'static str = GAN[gan_idx(if &c[2] == "卒" { "辛" } else { &c[2] })];
            let a = if a == "甲" { "戊" } else { a };
            let b = if b == "甲" { "戊" } else { b };
            assert_eq!(ke_ying(a, b).name, c[3].trim(), "{a}加{b}");
            n += 1;
        }
        assert_eq!(n, 81);
    }

    /// 整盘：格局按宫归并、地罗遮蔽；阴阳十八局六十时两种布盘皆可标
    #[test]
    fn biao_ji_pan() {
        // 阳一局甲己日庚午时：三宫六仪击刑，七宫玉女守门（演义卷二例）
        let p = pan(true, 1, "庚午");
        let b = biao_ji(&p, &GeJuContext { day_gz: gz("甲子"), ..Default::default() }, DEFAULT_BIAO_JI);
        assert!(b.at(3).ge.contains(&"六儀擊刑") && b.at(3).ji_xing == vec!["戊"]);
        assert!(b.at(7).ge.contains(&"玉女守門"));
        assert_eq!(zs(&b.xun_kong), "戌亥");
        assert_eq!((ZHI[b.ma_zhi as usize], b.ma_gong), ("申", 2)); // 午时马在申
        assert!(b.at(2).ma && b.at(6).kong);
        // 地罗遮蔽：天盘壬所临之宫地盘为时干
        let mut n = 0;
        for o in [DEFAULT_PAN, PanOptions { method: PanMethod::FeiPan, ..DEFAULT_PAN }] {
            for pn in all_pans(o) {
                let b = biao_ji(&pn, &GeJuContext { day_gz: pn.hour_gz, ..Default::default() }, DEFAULT_BIAO_JI);
                let shi = gan_on_pan(pn.hour_gz);
                for g in 1..=9u8 {
                    let c = pn.at(g);
                    assert_eq!(b.at(g).ge.contains(&"地羅遮蔽"), c.tian_pan.contains(&"壬") && c.di_pan.contains(&shi));
                    assert_eq!(b.at(g).ke_ying.len(), c.tian_pan.len() * c.di_pan.len());
                    n += b.at(g).ge.contains(&"地羅遮蔽") as u32;
                }
            }
        }
        assert!(n > 0);
    }
}
