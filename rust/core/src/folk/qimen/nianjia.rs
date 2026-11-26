//! 年家奇门：三元年遁，皆阴遁——上元六十年阴遁一局、中元阴遁四局、下元阴遁七局。以年干支为「时」布盘：
//! 旬首之星为直符、之门为直使，直符加年干所临之地盘宫，直使自旬首宫随年支逆飞（与时家阴遁同法）。
//!
//! 依据：
//! - 《遁甲演義》卷一「年家竒門」：「上元一宫起甲子，中元四宫起甲子，下元七宫起甲子，俱逆飛六儀……隨三元内各年支干
//!   審在某甲旬内，則以其甲頭為直符，符上之門為直使，將甲頭直符之星逐年干轉，直使隨年支逆飛」；「自嘉靖四十三年甲子
//!   起杜門，十年一移，萬厯甲午年移休」；「三元年遁上元六十年以隂遁一局加之，中元六十年以隂遁四局加之，下元六十年以
//!   隂遁七局加之」。
//! - 《奇門遁甲統宗》卷二「附年奇门起例」（简体录文，仅作旁证）：上元甲子六十年阴遁一局……；乙丑年「乙在坤二，丑在离九，
//!   即以天盘休门加离九，天蓬加坤二，又以八神之值符加于天蓬逆转」，「共一百八十年为一周」。
//!
//! 三元起年：两书皆不明言某年为上元。演义「嘉靖四十三年甲子起杜門」：杜门本位巽四，甲子旬首居四宫而直使为杜门，即阴遁四局，
//! 故嘉靖四十三年（1564）甲子为中元；上推六十年 1504 为上元，一百八十年一周，则 1684、1864、2044 亦为上元——与协纪卷八
//! 「康熙二十三年甲子為上元」（年紫白，见 [`crate::folk::taiyi::SHANG_YUAN_JIAZI`]）同。演义又言「萬厯甲午年移休」：阴四局
//! 甲午旬首仪辛居坎一，直使休门，相合。
//! 年以何为界（立春、正月朔、冬至）两书未言，调用方定（本项目他处年神、年九星以立春）。
//!
//! 另有「遁年奇法」「遁月奇法」顺飞（統宗、元靈經），只定六仪三奇之宫，见 [`dun_nian_qi`]、[`dun_yue_qi`]；月家两说
//! （[`YueJiaRule`]）与月家布盘 [`yue_jia_pan`]、演义月家休门 [`yue_jia_xiu_men`]；日家：統宗「附日奇门起例」之局与九星顺飞
//! （[`ri_jia_ju_tong_zong`]、[`ri_jia_xing_tong_zong`]）、「遁日奇法」布盘 [`ri_jia_pan`]。见 doc/folk/qimen.md 第六节。

use super::pan::{pai_pan, PanOptions, QimenPan};
use super::{ShiJu, Yuan};

/// 上元甲子之年（演义「嘉靖四十三年甲子」为中元推得；与协纪卷八三元同周期）
pub const NIAN_JIA_SHANG_YUAN: i32 = 1504;

/// 某年（干支年所在之公历年，天文纪年）之元与局：上元阴一、中元阴四、下元阴七
pub fn nian_jia_ju(year: i32) -> (Yuan, u8) {
    match (year - NIAN_JIA_SHANG_YUAN).rem_euclid(180) / 60 {
        0 => (Yuan::Shang, 1),
        1 => (Yuan::Zhong, 4),
        _ => (Yuan::Xia, 7),
    }
}

/// 年家奇门布盘：`year` 公历年（取其干支为「时」）。`term` 无节气之义，记冬至（18）以示阴遁出于三元年遁之定例。
pub fn nian_jia_pan(year: i32, opts: PanOptions) -> QimenPan {
    let (yuan, ju) = nian_jia_ju(year);
    let year_gz = (year - 4).rem_euclid(60) as u8;
    let sj = ShiJu { yang: false, ju, term: 18, yuan, run: false, chao: 0 };
    pai_pan(&sj, year_gz, opts)
}

/// 八节卦宫：节气 `term`（[`super::JIE_QI`] 序）所属之卦之宫数——坎（冬至小寒大寒）一、艮（立春雨水驚蟄）八、震三、巽四、离九、坤二、兑七、乾六
/// （演义卷三「二十四氣直於八卦」）
pub fn ba_jie_gong(term: usize) -> u8 {
    let q = (term % 24 + 24 - 18) % 24;
    [1, 8, 3, 4, 9, 2, 7, 6][q / 3]
}

/// 洛书数序飞宫（入中五）：自 `start` 宫起，顺飞（阳）或逆飞（阴）`n` 步所到之宫
pub fn fei_gong(start: u8, yang: bool, n: i32) -> u8 {
    let p = start as i32 + if yang { n } else { -n };
    ((p - 1).rem_euclid(9) + 1) as u8
}

/// 「遁年奇法」（顺飞）：《奇門遁甲統宗》卷二「又一本遁年奇法」：「假如庚子年立春节出师，阳气用事，艮上起甲子顺行……以甲子起艮，以甲戌起离，
/// 以甲申起坎，以甲午起坤，则是奇头在坤也。阳遁逆奇，丁奇在中，丙奇在乾，乙奇在兑……若在夏至后出师，用阴遁逆行，余仿此」；
/// 《奇門遁甲元靈經》卷一歌「甲子旬头起艮，甲戌飞入离宫，猿猴来入水晶宫，甲午坤中不动」同。
/// 起宫：統宗同卷「遁日奇法」云「年月奇以节气起宫，而日奇以局数起宫」，立春节起艮即立春之卦宫，故以所用节气之八节卦宫起甲子戊
/// （[`ba_jie_gong`]），冬至后阳遁顺飞、夏至后阴遁逆飞（入中五）。返回各宫之干（1—9）与本年旬首（「奇头」）所在宫。
/// 阴遁起宫之「余仿此」即读作同取卦宫（夏至离九……），原文无阴遁之例，未核
pub fn dun_nian_qi(year_gz: u8, term: usize) -> ([&'static str; 10], u8) {
    let d = super::pan::di_pan(super::is_yang(term), ba_jie_gong(term));
    let yi = super::pan::YI_QI[((year_gz % 60) / 10) as usize];
    let p = (1..=9u8).find(|&p| d[p as usize] == yi).unwrap();
    (d, p)
}

/// 「遁月奇法」（顺飞）：統宗卷二「又一本遁月奇法：假如己亥年冬至节，阳遁坎上起甲子，顺飞，其己亥年奇乃在离，甲己之年正月起丙寅，
/// 即以丙寅入离宫，顺飞。丁卯在坎，戊辰在坤，己巳在震，庚午在巽，辛未在中，壬申在乾，癸酉在兑，甲戌在艮，乙亥在离，丙子在坎，
/// 则离坎坤奇到方为吉」。即自节气卦宫起甲子飞至本年干支得年宫，正月（五虎遁）入年宫，逐月顺（阴遁逆）飞。
/// 返回（年宫，十二月各月之宫（正月起），十二月之干支序，乙丙丁三奇之月所到之宫）。夏至后逆飞依「阳顺阴逆」推，原文无阴遁之例，未核
pub fn dun_yue_qi(year_gz: u8, term: usize) -> (u8, [u8; 12], [u8; 12], Vec<u8>) {
    let yang = super::is_yang(term);
    let nian = fei_gong(ba_jie_gong(term), yang, (year_gz % 60) as i32);
    // 五虎遁：甲己之年丙寅……
    let first = ((year_gz % 5) * 2 + 2) % 10;
    let mut gong = [0u8; 12];
    let mut gz = [0u8; 12];
    let mut qi: Vec<u8> = Vec::new();
    for m in 0..12u8 {
        let g = (first + m) % 10;
        let z = (2 + m) % 12;
        gz[m as usize] = (0..60u8).find(|k| k % 10 == g && k % 12 == z).unwrap();
        gong[m as usize] = fei_gong(nian, yang, m as i32);
        if (1..=3).contains(&g) && !qi.contains(&gong[m as usize]) {
            qi.push(gong[m as usize]);
        }
    }
    (nian, gong, gz, qi)
}

/// 日家（統宗卷二「附日奇门起例」）之局：「冬至后用阳遁，夏至后用阴遁，子午日用二局……丑未日用六局，寅申日用一局，卯酉日用八局，
/// 辰戌日用四局，巳亥日用九局」
pub fn ri_jia_ju_tong_zong(day_gz: u8) -> u8 {
    [2, 6, 1, 8, 4, 9][((day_gz % 12) % 6) as usize]
}

/// 九星（洛书本位序）：蓬一 芮二 冲三 辅四 禽五 心六 柱七 任八 英九
pub const JIU_XING_LUO_SHU: [&str; 9] = ["天蓬", "天芮", "天冲", "天輔", "天禽", "天心", "天柱", "天任", "天英"];

/// 日家九星顺飞（統宗同条）：「其起九星八门之法不必复分阴遁阳遁。凡甲子日天蓬起艮八宫，芮在离九，顺飞九宫。乙丑日天蓬起离九，
/// 天芮在坎一。一日顺移」。返回各星（[`JIU_XING_LUO_SHU`] 序）所在之宫。甲子日起，逐日顺移一宫（入中五）；六十日后甲子复起艮：
/// 統宗原文未言，《奇門遁甲秘笈大全》卷二十「日家九星」「甲子日以太乙加艮宫顺行，甲戌加离，甲申加坎，甲午加坤，甲辰加震，
/// 甲寅日以太乙加巽宫」、歌「甲子回头把艮」，逐旬定宫、甲子必起艮（彼书九星名太乙攝提軒轅招搖天符青龍咸池太陰天乙，起宫同）
pub fn ri_jia_xing_tong_zong(day_gz: u8) -> [u8; 9] {
    let peng = fei_gong(8, true, (day_gz % 60) as i32);
    core::array::from_fn(|k| fei_gong(peng, true, k as i32))
}

/// 日家八门之序（門本位洛书数序：休一 死二 傷三 杜四 開六 驚七 生八 景九）
pub const RI_JIA_MEN: [&str; 8] = ["休門", "死門", "傷門", "杜門", "開門", "驚門", "生門", "景門"];
/// 日家八门移宫之序（洛书数序去中五）：坎坤震巽乾兌艮離
const RI_JIA_GONG: [u8; 8] = [1, 2, 3, 4, 6, 7, 8, 9];

fn ri_jia_men_from(start: usize) -> [&'static str; 10] {
    let mut out = [""; 10];
    for (i, m) in RI_JIA_MEN.iter().enumerate() {
        out[RI_JIA_GONG[(start + i) % 8] as usize] = m;
    }
    out
}

/// 日家八门全布（《奇門遁甲秘笈大全》卷二十「日家奇門歌訣」「八門三奇出行定局」）：「其法一卦管三日，如甲子乙丑丙寅三日於坎宫起休門，
/// 丁卯戊辰己巳三日於坤宫起休門，依九宫次序三日一换局，去中五不用」；定局逐三日列八门，甲子乙丑丙寅（戊子己丑庚寅、壬子癸丑甲寅同）
/// 「休坎 死坤 傷震 杜巽 開乾 驚兌 生艮 景離」，丁卯组「休坤 死震 傷巽 杜乾 開兌 驚艮 生離 景坎」……即八门依本位数序整体沿
/// 坎坤震巽乾兌艮離平移。休门与 [`super::ri_jia_xiu_men`]（演义）同。返回各宫（下标 1—9）之门，中五为空
pub fn ri_jia_ba_men(day_gz: u8) -> [&'static str; 10] {
    ri_jia_men_from(((day_gz % 60) / 3 % 8) as usize)
}

/// 「大八門逐日出行訣」（秘笈大全卷二十）：按节气分六段，甲子组休门所起之宫各异，余同 [`ri_jia_ba_men`] 三日一移：
/// 冬至小寒大寒立春「甲戊壬子居坎」、雨水驚蟄春分清明「居兌」、穀雨立夏小滿芒種「居巽」、夏至小暑大暑立秋「離位」、
/// 處暑白露秋分寒露「居震」、霜降立冬小雪大雪「乾位」（两两相冲）。`term` 为节气序（春分 = 0，冬至 = 18）
pub fn da_ba_men(day_gz: u8, term: usize) -> [&'static str; 10] {
    let q = (term % 24 + 24 - 18) % 24;
    let qi = [0usize, 5, 3, 7, 2, 4][q / 4];
    ri_jia_men_from(qi + ((day_gz % 60) / 3 % 8) as usize)
}

/// 秘笈大全卷二十「起月例」月奇（**未核**）：「假如己未年正月丙寅……立春節後艮宫起甲子，至離宫是己未太歲住處，就於離上正月五虎遁起丙寅，
/// 便是奇至離。月建丙寅，就於離上起庚寅，順飛至中宫乾兌，是為奇」。年宫、月宫同 [`dun_yue_qi`]；「起庚寅」读作以月干为岁干再遁五虎
/// （丙辛之岁起庚寅），自月宫顺飞，乙丙丁所到为奇。书只此一例，阴遁未言。返回乙、丙、丁三奇之宫
pub fn yue_jian_qi(month_gong: u8, month_gz: u8, yang: bool) -> [u8; 3] {
    let first = ((month_gz % 5) * 2 + 2) % 10; // 五虎遁寅月之干
    core::array::from_fn(|k| fei_gong(month_gong, yang, (k as i32 + 1 - first as i32).rem_euclid(10)))
}

/// 月家之元局两说
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YueJiaRule {
    /// 《遁甲演義》卷一「月家竒門」：甲己（年符头）遇四孟上元一宫、四仲中元七宫、四季下元四宫，阴遁，每元五年。缺省
    YanYi,
    /// 《奇門遁甲統宗》卷二「附月奇门起例」：甲子至戊辰、己卯至癸未、甲午至戊戌、己酉至癸丑为上元，阴遁一局；己巳至癸酉、甲申至戊子、
    /// 己亥至癸卯、甲寅至戊午为中元，阴遁四局；甲戌至戊寅、己丑至癸巳、甲辰至戊申、己未至癸亥为下元，阴遁七局（即年符头仲上、孟中、季下）
    TongZong,
}

/// 月家之元局
pub fn yue_jia_ju_with(year_gz: u8, rule: YueJiaRule) -> (super::Yuan, u8) {
    match rule {
        YueJiaRule::YanYi => super::yue_jia_ju(year_gz),
        YueJiaRule::TongZong => {
            let fu = year_gz % 60 - year_gz % 60 % 5;
            match fu % 3 {
                0 => (super::Yuan::Shang, 1),
                2 => (super::Yuan::Zhong, 4),
                _ => (super::Yuan::Xia, 7),
            }
        }
    }
}

/// 月家布盘：以月建干支为「时」，阴遁本元之局（演义「就其年遁月建係某甲頭直符逐地下月干轉符上之門為直使隨月支飛泊」；
/// 統宗「看月建之支临于地盘何宫，即以天盘值使加之；看月建之干临于地盘何宫，即以天盘值符加之，与年奇门同」）
pub fn yue_jia_pan(year_gz: u8, month_gz: u8, rule: YueJiaRule, opts: PanOptions) -> QimenPan {
    let (yuan, ju) = yue_jia_ju_with(year_gz, rule);
    let sj = ShiJu { yang: false, ju, term: 18, yuan, run: false, chao: 0 };
    pai_pan(&sj, month_gz % 60, opts)
}

/// 演义月家休门：「自嘉靖四十二年十一月甲子起休門在坎三箇月一移至萬厯四十一年十一月甲子起休門在坎」。三月一移，移宫之序同日家
/// （坎坤震巽乾兑艮离，不入中五），甲子之月起坎。六十月后甲子月是否复起坎原文未言（嘉靖四十二年至万历四十一年六百月，两读皆合），
/// 此处以月干支序推（甲子月复起坎，同日家「又復自坎宫起」），未核
pub fn yue_jia_xiu_men(month_gz: u8) -> &'static str {
    super::ri_jia_xiu_men(month_gz)
}

/// 日家布盘（統宗卷二「遁日奇法」：「日奇以局数起宫，法从本节上起甲子，阳顺阴逆数去……假如中元冬至后辛未日，阳遁七局，中元旬头甲子为值使，
/// 就于兑七宫起甲子顺飞。乙丑艮，丙寅离，丁卯坎，戊辰坤，己巳震，庚午巽，辛未中。则本日在中宫，兑宫惊门值事，即以惊门加中五寄坤，
/// 便知开在兑，休在乾，生在坎也」；《元靈經》卷一日奇门「其法亦以本节三元定局作地盘，再以本甲旬头加本日干上论数……
/// 阴遁奇直皆逆，星门皆顺」）：以日干支为「时」，局取本日（子时）之时家局，与时家布盘同法。`sj` 为本日子时之 [`ShiJu`]
pub fn ri_jia_pan(sj: &ShiJu, day_gz: u8, opts: PanOptions) -> QimenPan {
    pai_pan(sj, day_gz % 60, opts)
}

/// 統宗「又法日奇门」夏至第二甲子之局
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum XiaZhiZhongJu {
    /// 原文「第二甲子六十日用阴遁二局」。缺省
    #[default]
    Er,
    /// 作三局：冬至三甲子一、七、四即时家冬至上中下元之局，夏至依此当九、三、六（时家夏至三元），疑「二」为「三」之讹（**未核**）
    San,
}

/// 統宗「又法日奇门」一日之局
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RiJiaYouFa {
    pub yang: bool,
    pub ju: u8,
    /// 本至之后第几甲子（0—2）；闰为 2
    pub di: u8,
    /// 是否闰（叠用第三甲子）
    pub run: bool,
    /// 本至之首甲子（符头）日
    pub fu_tou: i64,
    /// 至日 − 符头日：正为超神、负为接气
    pub chao: i64,
}

/// 統宗卷二「又法日奇门起例」：「冬至甲子第一六十日用阳遁一局，第二甲子六十日用阳遁七局，第三甲子六十日用阳遁四局。夏至第一甲子
/// 六十日用阴遁九局，第二甲子六十日用阴遁二局，第三甲子六十日用阴遁六局，皆以其日旬首所值之星为值符，所值之门为值使，法亦同于年奇。
/// 其超接置闰，则如时奇，凡甲子符头超过三十日有奇，即宜置闰」。
///
/// 读法：每至以一甲子日为符头，三甲子凡一百八十日；半岁约一百八十二日有半，符头逐至渐早（超神），超过三十日则置闰——
/// 叠用前至第三甲子之局六十日，下至之符头遂在至后（接气，不及三十日）。如此每至之符头即至日前三十日至后三十日之内（不含前三十日之外）
/// 之甲子日，与起算之年无涉。闰叠第三甲子为本项目之读（时奇置闰叠芒种、大雪，此处以甲子为单位），**未核**。
/// `zhi` 为冬至、夏至交节民用日（升序，[`super::QiTable::zhi_days`]），须覆盖 `jdn` 前一至；`jdn` 早于表首则 None。
pub fn ri_jia_you_fa(jdn: i64, zhi: &[(i64, bool)], fa: XiaZhiZhongJu) -> Option<RiJiaYouFa> {
    // 至日 z 之符头：z − s ∈ (−30, 30] 之甲子日 s
    let fu_tou = |z: i64| {
        let s = z - super::day_gz(z) as i64; // 至日以前（含）最近之甲子
        if z - s > 30 {
            s + 60
        } else {
            s
        }
    };
    let (z, dong) = zhi.iter().filter(|(z, _)| fu_tou(*z) <= jdn).last().copied()?;
    let s = fu_tou(z);
    let k = (jdn - s) / 60;
    let (di, run) = if k >= 3 { (2u8, true) } else { (k as u8, false) };
    let ju = if dong {
        [1, 7, 4][di as usize]
    } else {
        [9, if fa == XiaZhiZhongJu::San { 3 } else { 2 }, 6][di as usize]
    };
    Some(RiJiaYouFa { yang: dong, ju, di, run, fu_tou: s, chao: z - s })
}

/// 統宗「又法日奇门」布盘：以日干支为「时」，阳遁或阴遁本局，旬首之星为直符、之门为直使（「法亦同于年奇」）
pub fn ri_jia_you_fa_pan(r: &RiJiaYouFa, day_gz: u8, opts: PanOptions) -> QimenPan {
    let yuan = [Yuan::Shang, Yuan::Zhong, Yuan::Xia][r.di as usize];
    let sj = ShiJu { yang: r.yang, ju: r.ju, term: if r.yang { 18 } else { 6 }, yuan, run: r.run, chao: r.chao };
    pai_pan(&sj, day_gz % 60, opts)
}

#[cfg(test)]
mod tests {
    use super::super::pan::DEFAULT_PAN;
    use super::*;

    #[test]
    fn yuan_and_examples() {
        // 嘉靖四十三年（1564）甲子：中元阴四局，直使杜门（「自嘉靖四十三年甲子起杜門」）
        assert_eq!(nian_jia_ju(1564), (Yuan::Zhong, 4));
        let p = nian_jia_pan(1564, DEFAULT_PAN);
        assert_eq!((p.xun_shou, p.zhi_shi), ("甲子", "杜門"));
        // 「十年一移」：甲戌年（1574）直使移
        assert_ne!(nian_jia_pan(1574, DEFAULT_PAN).zhi_shi, "杜門");
        // 「萬厯甲午年移休」：1594 甲午，直使休门
        assert_eq!(nian_jia_pan(1594, DEFAULT_PAN).zhi_shi, "休門");
        // 与协纪卷八三元同周期：1684、1864 上元
        assert_eq!(nian_jia_ju(1684).0, Yuan::Shang);
        assert_eq!(nian_jia_ju(1864).0, Yuan::Shang);
        assert_eq!(nian_jia_ju(2026), (Yuan::Xia, 7));
        // 統宗例：上元乙丑年（阴一局）——「乙在坤二，丑在离九，天盘休门加离九，天蓬加坤二」
        let p = nian_jia_pan(1505, DEFAULT_PAN);
        assert_eq!(nian_jia_ju(1505), (Yuan::Shang, 1));
        assert_eq!((p.zhi_fu, p.zhi_shi, p.zhi_fu_gong, p.zhi_shi_gong), ("天蓬", "休門", 2, 9));
        assert_eq!(p.at(9).men, Some("休門"));
        assert!(p.at(2).xing.contains(&"天蓬"));
        // 「又以八神之值符加于天蓬逆转」：直符神在坤二
        assert_eq!(p.at(2).shen, Some("直符"));
    }

    /// 統宗「遁年奇法」庚子年例：奇头甲午在坤，丁奇在中、丙奇在乾、乙奇在兑；元靈經甲子艮、甲戌离、甲申坎、甲午坤
    #[test]
    fn shun_fei_nian_qi() {
        let (d, tou) = dun_nian_qi(36, 21); // 庚子，立春节
        assert_eq!(tou, 2);
        assert_eq!((d[5], d[6], d[7]), ("丁", "丙", "乙"));
        assert_eq!((d[8], d[9], d[1], d[2]), ("戊", "己", "庚", "辛"));
        // 立春、雨水、驚蟄同起艮；冬至节起坎
        assert_eq!(dun_nian_qi(36, 23).1, 2);
        assert_eq!(ba_jie_gong(18), 1);
        assert_eq!(ba_jie_gong(6), 9);
    }

    /// 秘笈大全卷二十：日家八门定局、大八門六段、起月例
    #[test]
    fn mi_ji_ri_jia_ba_men() {
        let b = ri_jia_ba_men(0); // 甲子
        assert_eq!([b[1], b[2], b[3], b[4], b[6], b[7], b[8], b[9]], ["休門", "死門", "傷門", "杜門", "開門", "驚門", "生門", "景門"]);
        assert_eq!(ri_jia_ba_men(24), b); // 戊子
        assert_eq!(ri_jia_ba_men(48), b); // 壬子
        let c = ri_jia_ba_men(3); // 丁卯：休坤 死震 傷巽 杜乾 開兌 驚艮 生離 景坎
        assert_eq!([c[2], c[3], c[4], c[6], c[7], c[8], c[9], c[1]], ["休門", "死門", "傷門", "杜門", "開門", "驚門", "生門", "景門"]);
        let w = ri_jia_ba_men(6); // 庚午：休震 死巽 傷乾 杜兌 開艮 驚離 生坎 景坤
        assert_eq!((w[3], w[4], w[6], w[7], w[8], w[9], w[1], w[2]), ("休門", "死門", "傷門", "杜門", "開門", "驚門", "生門", "景門"));
        for d in 0..60u8 {
            let g = ri_jia_ba_men(d).iter().position(|m| *m == "休門").unwrap();
            assert_eq!(["", "坎", "坤", "震", "巽", "", "乾", "兑", "艮", "离"][g], super::super::ri_jia_xiu_men(d));
        }
        // 大八門：各段甲子组休门、丁卯组休门、壬午组休门
        let xiu = |d: u8, t: usize| da_ba_men(d, t).iter().position(|m| *m == "休門").unwrap() as u8;
        let terms = [18, 22, 2, 6, 10, 14]; // 冬至、雨水、穀雨、夏至、處暑、霜降
        assert_eq!(terms.map(|t| xiu(0, t)), [1, 7, 4, 9, 3, 6]);
        assert_eq!(terms.map(|t| xiu(3, t)), [2, 8, 6, 1, 4, 7]); // 丁卯：坤、八宫、乾、北、東南、酉宫
        assert_eq!(terms.map(|t| xiu(18, t)), [8, 4, 2, 7, 1, 3]); // 壬午：艮、巽、坤、兌、坎、震
        assert_eq!(xiu(0, 21), 1); // 立春仍属冬至段
        // 起月例：己未年立春节，年宫离，正月丙寅入离；月建丙寅「就於離上起庚寅，順飛至中宫乾兌，是為奇」
        let (nian, gong, gz, _) = dun_yue_qi(55, 21);
        assert_eq!((nian, gong[0], gz[0]), (9, 9, 2));
        assert_eq!(yue_jian_qi(9, 2, true), [5, 6, 7]);
    }

    /// 統宗「遁月奇法」己亥年冬至节例：年在离，正月丙寅入离，丁卯坎、戊辰坤……甲戌艮、乙亥离、丙子坎，「离坎坤奇到」
    #[test]
    fn shun_fei_yue_qi() {
        let (nian, gong, gz, qi) = dun_yue_qi(35, 18); // 己亥，冬至
        assert_eq!(nian, 9);
        assert_eq!(gong[..11], [9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1]);
        assert_eq!(gz[0], 2); // 丙寅
        let mut q = qi.clone();
        q.sort();
        assert_eq!(q, vec![1, 2, 9]);
    }

    /// 統宗「附日奇门起例」：子午日二局……；甲子日天蓬艮八、芮离九，乙丑日天蓬离九、芮坎一；休门甲子乙丑丙寅坎一、丁卯坤二……丙子乾六
    #[test]
    fn ri_jia_tong_zong() {
        assert_eq!([0u8, 1, 2, 3, 4, 5, 6, 7].map(ri_jia_ju_tong_zong), [2, 6, 1, 8, 4, 9, 2, 6]);
        let x = ri_jia_xing_tong_zong(0);
        assert_eq!((x[0], x[1]), (8, 9));
        let x = ri_jia_xing_tong_zong(1);
        assert_eq!((x[0], x[1]), (9, 1));
        let xm = |g: u8| super::super::ri_jia_xiu_men(g);
        assert_eq!([xm(0), xm(2), xm(3), xm(6), xm(9), xm(12)], ["坎", "坎", "坤", "震", "巽", "乾"]);
    }

    /// 統宗「遁日奇法」二例：立春节丙寅日上元阳八局，奇在中乾兑；中元冬至后辛未日阳七局，直使惊门加中五寄坤，开在兑、休在乾、生在坎
    #[test]
    fn ri_jia_dun_ri_qi() {
        let d = super::super::pan::di_pan(true, 8);
        assert_eq!((d[5], d[6], d[7]), ("丁", "丙", "乙"));
        let sj = ShiJu { yang: true, ju: 7, term: 18, yuan: Yuan::Zhong, run: false, chao: 0 };
        let p = ri_jia_pan(&sj, 7, DEFAULT_PAN); // 辛未
        assert_eq!(p.zhi_shi, "驚門");
        assert_eq!(p.zhi_shi_gong, 5);
        assert_eq!((p.at(7).men, p.at(6).men, p.at(1).men), (Some("開門"), Some("休門"), Some("生門")));
        assert_eq!(p.at(2).men, Some("驚門"));
    }

    /// 月家两说与演义月家休门「嘉靖四十二年十一月甲子起休門在坎……萬厯四十一年十一月甲子起休門在坎」
    #[test]
    fn yue_jia() {
        // 演义：甲子年（符头甲子，仲）中元七局；統宗：上元一局
        assert_eq!(yue_jia_ju_with(0, YueJiaRule::YanYi), (Yuan::Zhong, 7));
        assert_eq!(yue_jia_ju_with(0, YueJiaRule::TongZong), (Yuan::Shang, 1));
        assert_eq!(yue_jia_ju_with(5, YueJiaRule::TongZong), (Yuan::Zhong, 4)); // 己巳
        assert_eq!(yue_jia_ju_with(10, YueJiaRule::TongZong), (Yuan::Xia, 7)); // 甲戌
        assert_eq!(yue_jia_ju_with(15, YueJiaRule::TongZong), (Yuan::Shang, 1)); // 己卯
        assert_eq!(yue_jia_ju_with(59, YueJiaRule::TongZong), (Yuan::Xia, 7)); // 癸亥（己未至癸亥下元）
        assert_eq!(yue_jia_xiu_men(0), "坎");
        // 統宗附月例同年奇：上元甲子年（阴一局）正月丙寅，丙寅在甲子旬，直符天蓬加丙所临、直使休门自坎逆飞至寅
        let p = yue_jia_pan(0, 2, YueJiaRule::TongZong, DEFAULT_PAN);
        assert_eq!((p.zhi_fu, p.zhi_shi), ("天蓬", "休門"));
    }

    /// 統宗「又法日奇门」：符头皆甲子、超接不过三十日，三甲子之局与闰
    #[test]
    fn ri_jia_you_fa_chao_jie() {
        use crate::astronomy::ephemeris::load_earth_vsop87_from_repo;
        let vsop = load_earth_vsop87_from_repo().unwrap();
        let t = super::super::QiTable::new(&vsop, 1990, 2030, super::super::JieQiSource::DingQi);
        let zhi = t.zhi_days();
        let jdn0 = 2_448_000; // 1990-02
        let (mut runs, mut prev): (i32, Option<RiJiaYouFa>) = (0, None);
        for d in jdn0..jdn0 + 365 * 36 {
            let r = ri_jia_you_fa(d, &zhi, XiaZhiZhongJu::Er).unwrap();
            assert_eq!(super::super::day_gz(r.fu_tou), 0);
            assert!(r.chao > -30 && r.chao <= 30, "{d} {}", r.chao);
            assert_eq!(r.ju, if r.yang { [1, 7, 4][r.di as usize] } else { [9, 2, 6][r.di as usize] });
            if r.run && !prev.map(|p| p.run).unwrap_or(false) {
                runs += 1;
                // 闰起于超神逾三十日之前：本至符头超得多，下至转为接气
                assert!(r.chao > 0);
            }
            prev = Some(r);
        }
        // 每至符头早约二日六（182.6 − 180），六十日之差约二十三至（十一年半）一闰，三十六年三闰上下
        assert!((2..=4).contains(&runs), "{runs}");
        // 夏至第二甲子作三局之说
        let d = zhi.iter().find(|z| !z.1 && z.0 > jdn0).unwrap().0;
        let r = ri_jia_you_fa(d + 90, &zhi, XiaZhiZhongJu::San).unwrap();
        if r.di == 1 {
            assert_eq!(r.ju, 3);
        }
        // 布盘：以日干支为时，甲子日直符为旬首甲子（戊）所在之星
        let r = ri_jia_you_fa(t.zhi_days()[10].0, &zhi, XiaZhiZhongJu::Er).unwrap();
        let p = ri_jia_you_fa_pan(&r, 0, DEFAULT_PAN);
        assert_eq!(p.xun_shou, "甲子");
    }
}
