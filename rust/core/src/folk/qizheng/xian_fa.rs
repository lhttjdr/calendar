//! 七政四余诸家限法（《星學大成》卷七《諸家限例》）：洞微飛限、竹羅三限、急脚限、黄泉限，及倒限之候、四煞宫。
//!
//! - 洞微飛限：卷七《洞微飛限詳説》逐岁一表（一至一百岁），照录为 [`DONG_WEI_FEI_XIAN`]。
//! - 竹羅三限：卷七《説竹羅三限》《起限》《三限用七政行年例》：命宫三方主星三颗，依晝夜定次第，为初、中、末三限；
//!   各限年数依曜（水二十四、木三十、土金二十六、火二十八，日月取火水之数），自该曜所躔之宫起，先住「起數」年
//!   （金四、土五、木三、水一、火二），此后每十年隔一宫逆行（「隔節數三宫逆行」），不足十年之零年逐年顺行一宫。
//!
//! 只算各岁所行之宫，不作吉凶。

use super::PALACES;

/// 洞微飛限逐岁所行之宫（十二宫名首字；「遊行」即遷移）。卷七《洞微飛限詳説》原文：
/// 「一二命宫，三四妻妾，五男女，六遊行，七八命宫，九十妻妾，十一男女，十二遊行，十三四命宫，十五妻妾共十五年；
/// 十六七相貌，十八九奴僕，二十田宅，二十一疾厄，二十二三相貌，二十四五奴僕共十年；二十六七福徳，二十八九男女，三十妻妾，
/// 三十一兄弟，三十二三福徳，三十四五男女，三十六妻妾共十一年；三十七八官祿，三十九四十田宅，四十一財帛，四十二奴僕，
/// 四十三四官祿，四十五六田宅，四十七財帛，四十八奴僕，四十九五十官祿，五十一田宅共十五年；五十二三遊行，五十四五兄弟，
/// 五十六男女，五十七命宫，五十八九遊行共八年；六十六十一疾厄，六十二三財帛，六十四田宅，六十五相貌，六十六疾厄共七年；
/// 六十七八妻妾，六十九七十命宫，七十一福徳，七十二兄弟，七十三四妻妾，七十五六命宫，七十七福徳共十一年；七十八九奴僕，
/// 八十八十一相貌，八十二財帛半年共四年半；八十三四男女，八十五六福徳共四年；八十七八田宅，八十九九十官祿，九十一疾厄半年共四年半；
/// 九十二三兄弟，九十四五遊行，九十六福徳共五年；九十七八財帛，九十九一百疾厄共四年」。
/// 「半年」二处（八十二、九十一）按整岁记
pub const DONG_WEI_FEI_XIAN: &str = concat!(
    "命命妻妻男遷命命妻妻男遷命命妻", // 1—15 命宫
    "相相奴奴田疾相相奴奴",           // 16—25 相貌
    "福福男男妻兄福福男男妻",         // 26—36 福德
    "官官田田財奴官官田田財奴官官田", // 37—51 官祿
    "遷遷兄兄男命遷遷",               // 52—59 遷移
    "疾疾財財田相疾",                 // 60—66 疾厄
    "妻妻命命福兄妻妻命命福",         // 67—77 妻妾
    "奴奴相相財",                     // 78—82 奴僕
    "男男福福",                       // 83—86 男女
    "田田官官疾",                     // 87—91 田宅
    "兄兄遷遷福",                     // 92—96 兄弟
    "財財疾疾",                       // 97—100 財帛
);

/// 洞微飛限 `sui` 岁（1—100）所行之宫（地支序）；超出则 None
pub fn dong_wei_fei_xian(ming: u8, sui: u32) -> Option<u8> {
    let c = DONG_WEI_FEI_XIAN.chars().nth((sui as usize).checked_sub(1)?)?;
    let p = PALACES.iter().position(|n| n.starts_with(c))?;
    Some(((ming as i32 - p as i32).rem_euclid(12)) as u8)
}

/// 竹羅三方主之次第异说（只寅午戌一组有异）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ZhuLuoRule {
    /// 《起限》歌「寅午戌兮日木土，夜生木日土為强」（缺省；与卷二論三方主星「日木土向陽」合）
    #[default]
    Ge,
    /// 《洞微飛限詳説》后竹羅三限表「寅午戌宫晝生日水土，夜生水日土」
    Biao,
}

/// 竹羅三方主（初、中、末三限之曜，十一曜下标）：依命宫三合与晝夜。卷七：
/// 「寅午戌宫晝生日水土夜生水日土，亥卯未宫晝生金火月夜生火金月，申子辰宫晝生土水木夜生水土木，己酉丑宫晝生金月火夜生月金火」
/// （表）；《起限》歌「寅午戌兮日木土，夜生木日土為强；亥卯未兮金火月，夜生火金月最光；申子辰兮土水木，夜生水土木為良；
/// 己酉丑兮金月火，夜生月金火相當」。「右三方主星取日出没時分晝夜」
pub fn zhu_luo_zhu(ming: u8, zhou: bool, rule: ZhuLuoRule) -> [usize; 3] {
    // 晝生之序；夜生对调前二
    let d: [usize; 3] = match ming % 4 {
        2 => match rule {
            ZhuLuoRule::Ge => [0, 5, 6],
            ZhuLuoRule::Biao => [0, 2, 6],
        },
        3 => [3, 4, 1],
        0 => [6, 2, 5],
        _ => [3, 1, 4],
    };
    if zhou { d } else { [d[1], d[0], d[2]] }
}

/// 竹羅限年（《三限用七政行年例》「水星限二十四年，木星限三十年，土星限二十六年，金星限二十六年，火星限二十八年，日月取水火之數」）
pub fn zhu_luo_years(yao: usize) -> u32 {
    match yao {
        2 | 1 => 24,
        5 => 30,
        6 | 3 => 26,
        _ => 28, // 火、日
    }
}

/// 起數（《起限》「金星四嵗土星五木三水一火二數」；日月依「太陽元是火之精，太陰便是水之副，二法各取水火行」取火、水之数）
pub fn zhu_luo_qi(yao: usize) -> u32 {
    match yao {
        3 => 4,
        6 => 5,
        5 => 3,
        2 | 1 => 1,
        _ => 2,
    }
}

/// 竹羅一限之内逐节逆移之宫数，两说（《星學大成》卷七）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ZhuLuoXing {
    /// 逆移起數之宫：卷七《起竹羅三限》例「太陽從火論火數二逆數亦二……木三數……土……十年逆數五」及其后
    /// 「太陽火星限二起逆數亦二……木限三起逆數三……金限四起逆數亦四，土限五起逆數五」（缺省：一例三限并四条明文）
    #[default]
    QiShu,
    /// 每节逆移二宫：卷七《説竹羅三限》歌「開端隔節數三宫逆行」與土例「便從巳上起五嵗，大數十五在卯，二十五丑」
    GeJie,
}

/// 一限每节（十年）逆移之宫数
pub fn zhu_luo_step(yao: usize, xing: ZhuLuoXing) -> u32 {
    match xing {
        ZhuLuoXing::QiShu => zhu_luo_qi(yao),
        ZhuLuoXing::GeJie => 2,
    }
}

/// 一限之内第 `k` 年（1 起，`k = 1` 即该限起岁）所在之宫：起宫管十年，此后每满十年逆移 `step` 宫（「開端隔節數三宫逆行」，
/// 两节三宫）；限年之零数（限年除十之余，木三十年无零）自第三宫起逐年顺行一宫（「零年順轉逐宫移」）。
/// 卷七《起竹羅三限》例「從午上起二嵗管十年……十二在辰，二十二在寅，零年二十三囘卯順數……二十九在酉」、
/// 「三十三交木限在亥管十年……四十三在申，五十三在己……木無零數」、「六十五交土限在丑……七十五在申……八十五在卯，
/// 零年順數八十六在辰……九十在申」；《説竹羅三限》土例「巳上起五嵗，大數十五在卯，二十五丑，二十六却囘寅」亦合（逆二宫）。
/// 「若見逺時數二節」即每限只数两节（三宫），其余为零年（本项目读法）
pub fn zhu_luo_gong_in(start: u8, step: u32, len: u32, k: u32) -> u8 {
    let s = start as i32;
    let full = len / 10;
    let z = if k <= full * 10 {
        s - (step * ((k.max(1) - 1) / 10)) as i32
    } else {
        s - (step * 2.min(full)) as i32 + (k - full * 10 - 1) as i32
    };
    z.rem_euclid(12) as u8
}

/// 竹羅一限
#[derive(Clone, Debug, PartialEq)]
pub struct ZhuLuoXian {
    /// 限星（十一曜下标）
    pub yao: usize,
    /// 起止之岁（含；不含限前停住之年）
    pub from: u32,
    pub to: u32,
    /// 起宫（限星所躔之宫）
    pub start: u8,
}

/// 竹羅三限：三限各自限星所躔之宫起（`gongs`，与 `stars` 同序），返回三限与逐岁之宫（下标 0 为一岁）。
/// 起岁：初限起于起數之岁（「從午上起二嵗」，此前之岁住起宫）；中、末限于前限满后先停住起數之年再交（例「三十及三十一三十二
/// 停住，三十三交木限」，木起數三）。例中土限只停六十三、六十四两年即于六十五交，与「土五」不合，本项目一律停起數之年（未核）；
/// 停住之年记在前限末宫
pub fn zhu_luo(stars: [usize; 3], gongs: [u8; 3], xing: ZhuLuoXing) -> (Vec<ZhuLuoXian>, Vec<u8>) {
    let mut xian = Vec::new();
    let mut per: Vec<u8> = Vec::new();
    for (i, (y, g)) in stars.iter().zip(gongs).enumerate() {
        let len = zhu_luo_years(*y);
        let qi = zhu_luo_qi(*y);
        let step = zhu_luo_step(*y, xing);
        let wait = if i == 0 { qi - 1 } else { qi };
        let hold = per.last().copied().unwrap_or(g);
        per.extend(core::iter::repeat_n(hold, wait as usize));
        let from = per.len() as u32 + 1;
        for k in 1..=len {
            per.push(zhu_luo_gong_in(g, step, len, k));
        }
        xian.push(ZhuLuoXian { yao: *y, from, to: from + len - 1, start: g });
    }
    (xian, per)
}

/// 急脚限起宫（命宫 → 一岁之宫）：卷七《急脚限例》「子起亥，丑起寅，寅起己，卯起申，辰起卯，己起午，午起酉，未起子，
/// 申起戌，酉起未，戌起丑，亥起辰」（歌「寳瓶在亥丑居寅，人馬來蛇，蝎奔申，天秤在卯雙女午，獅子金牛，蟹寳瓶，陰陽戌地，
/// 金牛未，白羊居丑，亥居辰」同）
pub const JI_JIAO_QI: [u8; 12] = [11, 2, 5, 8, 3, 6, 9, 0, 10, 7, 1, 4];

/// 急脚限 `sui` 岁之宫：注「假如子上安命即亥上起一嵗，酉上一十一，未上二十一，零年順數，節節而去，即不隔宫，如算小兒體例一般」
/// ——每十年逆二宫，十年之内逐岁顺行一宫；「不論男女皆順數」
pub fn ji_jiao_xian(ming: u8, sui: u32) -> u8 {
    let s = JI_JIAO_QI[(ming % 12) as usize] as i32;
    let n = sui.max(1) - 1;
    (s - 2 * (n / 10) as i32 + (n % 10) as i32).rem_euclid(12) as u8
}

/// 黄泉限 `sui` 岁之宫：卷七《論黄泉急脚限》「黄泉急脚衆皆知，兩様推尋識者稀，急則順行還逆轉」，注「世人皆以黄泉急脚為一限，
/// 不知分為二，蓋順行則為急脚，逆行則為黄泉」；卷二十六注「一順一逆而互推之」。本项目读作与急脚同起宫、方向全反：
/// 每十年顺二宫，十年之内逐岁逆行（未核：书无黄泉起例与算例）
pub fn huang_quan_xian(ming: u8, sui: u32) -> u8 {
    let s = JI_JIAO_QI[(ming % 12) as usize] as i32;
    let n = sui.max(1) - 1;
    (s + 2 * (n / 10) as i32 - (n % 10) as i32).rem_euclid(12) as u8
}

/// 黄泉路、二八門：卷七急脚歌「子午即是黄泉路，卯酉呼為二八門，限到煞宫人便死」
pub fn huang_quan_lu(gong: u8) -> bool {
    gong % 3 == 0
}

/// 生年納音四煞宫（沐浴、死、墓、絶之支）：卷七《三限用七政行年例》「四煞乃本生年納音沐浴死墓絶宫是也」；卷三《論沐死墓絶四煞宫度歌》。
/// `xing` 为納音五行（木0 火1 土2 金3 水4）；長生：木亥、火寅、金巳、水土申（土随水），沐浴长生后一、死后七、墓后八、絶后九（顺支，長生沐浴冠帶臨官帝旺衰病死墓絶）
pub fn si_sha_gong(xing: usize) -> [u8; 4] {
    let cs = [11u8, 2, 8, 5, 8][xing % 5];
    [1u8, 7, 8, 9].map(|k| (cs + k) % 12)
}

/// 洞微飛限三合二宫先后之说
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DongWeiRule {
    /// 照卷七《洞微飛限詳説》逐岁原表（缺省）
    #[default]
    Biao,
    /// 依《洞微飛限歌》「陽宫逆取順行去，陰宫順取逆行同」以大限所在地支之阴阳定先后：陽支（子寅辰午申戌）先取逆支四位
    /// （逆布第五宫），陰支先取顺支四位（本项目读法；原表所据之阴阳与此不尽合，见 doc/folk/qizheng.md §4.10）
    YinYangGong,
}

/// 洞微百六大限之宫序与年数（命十五、貌十、福十一、官十五、遷八、疾七、妻十一、奴四半、男四、田四半、兄五、財四；半年并入整岁，
/// 与 [`DONG_WEI_FEI_XIAN`] 同）：（十二宫下标，年数）
pub const DONG_WEI_BLOCKS: [(usize, u32); 12] = [(0, 15), (11, 10), (10, 11), (9, 15), (8, 8), (7, 7), (6, 11), (5, 5), (4, 4), (3, 5), (2, 5), (1, 4)];

/// 按《洞微飛限歌》「一二本限裏，三四對照衝，三合五六載……三合各一嵗，本宫兩嵗」生成一至一百岁之宫；`first_plus4(宫, 支)`
/// 定三合二宫中是否先取逆布第五宫（+4）
pub fn dong_wei_by_rule(ming: u8, rule: DongWeiRule) -> Vec<u8> {
    let mut v = Vec::with_capacity(100);
    for (b, n) in DONG_WEI_BLOCKS {
        let zhi = ((ming as i32 - b as i32).rem_euclid(12)) as u8;
        let plus4_first = match rule {
            DongWeiRule::Biao => DONG_WEI_BIAO_PLUS4[b],
            DongWeiRule::YinYangGong => zhi % 2 == 0,
        };
        for k in 0..n {
            let p = match k % 6 {
                0 | 1 => b,
                2 | 3 => (b + 6) % 12,
                4 => if plus4_first { (b + 4) % 12 } else { (b + 8) % 12 },
                _ => if plus4_first { (b + 8) % 12 } else { (b + 4) % 12 },
            };
            v.push(((ming as i32 - p as i32).rem_euclid(12)) as u8);
        }
    }
    v
}

/// 原表各大限（十二宫下标）三合先取逆布第五宫（+4）否：命、相貌、官祿、妻妾、田宅为是，福德、遷移、疾厄、奴僕、兄弟为否；
/// 男女、財帛四年无三合，任填
const DONG_WEI_BIAO_PLUS4: [bool; 12] = [true, true, false, true, true, false, true, false, false, true, false, true];

/// 倒限之候（卷七「大算須知倒限年……火羅孛土皆臨限，此地催人下九泉」，注「如諸限皆為惡星相照死無疑也」）：
/// `gongs` 为十一曜之宫，判火、羅、孛、土四曜是否皆「臨」限宫——在限宫本宫、對宫或三合（本项目读「照」为三方四正中之本、對、三合）
pub fn si_e_lin(gongs: &[u8], xian: u8) -> bool {
    [4usize, 7, 9, 6].iter().all(|&k| gongs.get(k).is_some_and(|&g| zhao(g, xian)))
}

/// 曜在 `g` 宫是否照临 `xian` 宫（同宫、對宫、三合）
pub fn zhao(g: u8, xian: u8) -> bool {
    let d = (g as i32 - xian as i32).rem_euclid(12);
    matches!(d, 0 | 4 | 6 | 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn z(c: char) -> u8 {
        super::super::ZHI_CH.chars().position(|x| x == c).unwrap() as u8
    }

    /// 洞微飛限表之结构：各大限（命十五、貌十、福十一、官十五、遷八、疾七、妻十一、奴四半、男四、田四半、兄五、財四）
    /// 皆「一二本限裏，三四對照衝，三合五六載……三合各一嵗，本宫兩嵗」（《洞微飛限歌》）
    #[test]
    fn dong_wei_structure() {
        assert_eq!(DONG_WEI_FEI_XIAN.chars().count(), 100);
        let blocks: [(usize, usize); 12] = [(0, 15), (11, 10), (10, 11), (9, 15), (8, 8), (7, 7), (6, 11), (5, 5), (4, 4), (3, 5), (2, 5), (1, 4)];
        let cs: Vec<usize> = DONG_WEI_FEI_XIAN.chars().map(|c| PALACES.iter().position(|n| n.starts_with(c)).unwrap()).collect();
        let mut i = 0;
        let mut order = Vec::new();
        for (b, n) in blocks {
            for k in 0..n {
                let p = cs[i + k];
                let want = match k % 6 {
                    0 | 1 => vec![b],
                    2 | 3 => vec![(b + 6) % 12],
                    _ => vec![(b + 4) % 12, (b + 8) % 12],
                };
                assert!(want.contains(&p), "大限{} 第{}年 {}", PALACES[b], k + 1, PALACES[p]);
            }
            if n >= 5 {
                // 三合二宫先后：+4（逆布第五）在前为「順」
                order.push((PALACES[b], cs[i + 4] == (b + 4) % 12));
            }
            i += n;
        }
        eprintln!("洞微飛限三合先后（true = 逆布 +4 在前）：{order:?}");
        // 大限次序与年数同《洞微百六限》（相貌十、福德十一……），命宫一岁起
        assert_eq!(dong_wei_fei_xian(z('子'), 1), Some(z('子')));
        assert_eq!(dong_wei_fei_xian(z('子'), 3), Some(z('午'))); // 妻妾：命宫逆布第七
        assert_eq!(dong_wei_fei_xian(z('子'), 6), Some(z('辰'))); // 遊行（遷移）：逆布第九
        assert_eq!(dong_wei_fei_xian(z('子'), 101), None);
    }

    /// 竹羅：卷七《説竹羅三限》土例与《起竹羅三限》日寅三限例
    #[test]
    fn zhu_luo_example() {
        // 《説竹羅三限》：「土為初限在巳上，便從巳上起五嵗，大數十五在卯，二十五丑，二十六却囘寅」（逆二宫；k = 岁 − 4）
        let g = |sui: u32| zhu_luo_gong_in(z('巳'), 2, 26, sui - 4);
        assert!((5..=14).all(|s| g(s) == z('巳')));
        assert_eq!(g(15), z('卯'));
        assert_eq!(g(25), z('丑'));
        assert_eq!(g(26), z('寅'));
        // 《起竹羅三限》：晝生寅命，日木土三方主在午、亥、丑
        let (x, per) = zhu_luo([0, 5, 6], [z('午'), z('亥'), z('丑')], ZhuLuoXing::QiShu);
        let at = |s: usize| per[s - 1];
        // 「從午上起二嵗管十年……十二在辰，二十二在寅，零年二十三囘卯順數……二十九在酉，共二十八年」
        assert_eq!((x[0].from, x[0].to), (2, 29));
        assert!((2..=11).all(|s| at(s) == z('午')));
        assert_eq!((at(12), at(21), at(22), at(23), at(24), at(29)), (z('辰'), z('辰'), z('寅'), z('卯'), z('辰'), z('酉')));
        // 「三十及三十一三十二停住，三十三交木限在亥管十年……四十三在申，五十三在己……木無零數」
        assert_eq!((at(30), at(32)), (z('酉'), z('酉')));
        assert_eq!((x[1].from, x[1].to), (33, 62));
        assert_eq!((at(33), at(42), at(43), at(53), at(62)), (z('亥'), z('亥'), z('申'), z('巳'), z('巳')));
        // 土限：书「六十三六十四停住，六十五交土限在丑……七十五在申……八十五在卯，零年順數八十六在辰……九十在申」；
        // 本项目停五年（起數），于六十八交——限内宫序全合，起岁差三年（记异）
        assert_eq!((x[2].from, x[2].to), (68, 93));
        let tu = |k: u32| zhu_luo_gong_in(z('丑'), 5, 26, k);
        assert_eq!((tu(1), tu(10), tu(11), tu(21), tu(22), tu(26)), (z('丑'), z('丑'), z('申'), z('卯'), z('辰'), z('申')));
        // 歌（逆二宫）于此例木、土二限不合：木第十一年在酉（书申）
        assert_eq!(zhu_luo_gong_in(z('亥'), 2, 30, 11), z('酉'));
        // 三方主：丑命夜生（卷七「己酉丑兮金月火，又云夜生中即反其初……可作夜生人起限月金火為例」）
        assert_eq!(zhu_luo_zhu(z('丑'), false, ZhuLuoRule::Ge), [1, 3, 4]);
        // 表与歌：唯寅午戌一组有异
        for m in 0..12u8 {
            for d in [true, false] {
                let (a, b) = (zhu_luo_zhu(m, d, ZhuLuoRule::Ge), zhu_luo_zhu(m, d, ZhuLuoRule::Biao));
                assert_eq!(a == b, m % 4 != 2);
            }
        }
        // 月金火三限：月 1—24、停四年、金 29—54、停二年、火 57—84
        let (x, per) = zhu_luo([1, 3, 4], [0, 0, 0], ZhuLuoXing::QiShu);
        assert_eq!((x[0].from, x[1].from, x[2].from, x[2].to, per.len()), (1, 29, 57, 84, 84));
    }

    /// 急脚限：表、歌、注例三者互证
    #[test]
    fn ji_jiao_examples() {
        // 歌（以宫次名）：寳瓶（子）亥、磨蝎（丑）寅、人馬（寅）巳、天蝎（卯）申、天秤（辰）卯、雙女（巳）午、獅子（午）金牛（酉）、
        // 巨蟹（未）寳瓶（子）、陰陽（申）戌、金牛（酉）未、白羊（戌）丑、雙魚（亥）辰
        let ge = "亥寅巳申卯午酉子戌未丑辰";
        for (m, c) in ge.chars().enumerate() {
            assert_eq!(JI_JIAO_QI[m], z(c));
            assert_eq!(ji_jiao_xian(m as u8, 1), z(c));
        }
        // 注「子上安命即亥上起一嵗，酉上一十一，未上二十一，零年順數」
        assert_eq!(ji_jiao_xian(z('子'), 1), z('亥'));
        assert_eq!(ji_jiao_xian(z('子'), 2), z('子'));
        assert_eq!(ji_jiao_xian(z('子'), 11), z('酉'));
        assert_eq!(ji_jiao_xian(z('子'), 21), z('未'));
        // 黄泉为其逆：亥起，十一岁丑，二岁戌
        assert_eq!((huang_quan_xian(z('子'), 1), huang_quan_xian(z('子'), 2), huang_quan_xian(z('子'), 11)), (z('亥'), z('戌'), z('丑')));
        assert!(huang_quan_lu(z('子')) && huang_quan_lu(z('酉')) && !huang_quan_lu(z('寅')));
    }

    /// 四煞宫：卷三《論沐死墓絶四煞宫度歌》以宿度记「火生人絶璧三、墓婁一、死昴二、沐房一」等，换为宫（中气分宫、清代黄道宿度约略）：
    /// 火人沐浴卯（房）、死酉（昴）、墓戌（婁）、絶亥（壁）；金人沐浴午（星）、死子（危）、墓丑（牛）、絶寅（尾）
    #[test]
    fn si_sha_examples() {
        assert_eq!(si_sha_gong(1), [z('卯'), z('酉'), z('戌'), z('亥')]);
        assert_eq!(si_sha_gong(3), [z('午'), z('子'), z('丑'), z('寅')]);
        assert_eq!(si_sha_gong(0), [z('子'), z('午'), z('未'), z('申')]); // 木：沐危（子）、死星（午）、墓柳（未）、絶參（申）
        assert_eq!(si_sha_gong(4), [z('酉'), z('卯'), z('辰'), z('巳')]); // 水：沐胃十四（酉）、死氐十六（卯）、墓氐一（辰）、絶翼十九（巳）
        // 土：本项目随水（長生申）；书土列「絶軫一、墓軫十八、死心一、沐畢四」（辰/巳、卯、申），与水不同，未核
        assert_eq!(si_sha_gong(2), si_sha_gong(4));
    }

    /// 倒限之候与照临
    #[test]
    fn dao_xian_lin() {
        assert!(zhao(z('子'), z('子')) && zhao(z('午'), z('子')) && zhao(z('辰'), z('子')) && zhao(z('申'), z('子')));
        assert!(!zhao(z('丑'), z('子')));
        let mut g = [z('寅'); 11];
        g[4] = z('子');
        g[7] = z('午');
        g[9] = z('辰');
        g[6] = z('申');
        assert!(si_e_lin(&g, z('子')));
        g[6] = z('丑');
        assert!(!si_e_lin(&g, z('子')));
    }

    /// 洞微飛限：按歌生成之法，以原表之三合先后可复原全表；依「陽宫逆取」读作地支阴阳则与原表不尽合
    #[test]
    fn dong_wei_rule_vs_table() {
        for m in 0..12u8 {
            let t: Vec<u8> = (1..=100).filter_map(|s| dong_wei_fei_xian(m, s)).collect();
            assert_eq!(dong_wei_by_rule(m, DongWeiRule::Biao), t);
        }
        // 地支阴阳读法：逐命宫计与原表三合先后相合之大限数（共十限可判）
        let mut best = (0, 0u8);
        for m in 0..12u8 {
            let n = DONG_WEI_BLOCKS.iter().filter(|(b, n)| *n >= 5 && (((m as i32 - *b as i32).rem_euclid(12)) % 2 == 0) == DONG_WEI_BIAO_PLUS4[*b]).count();
            if n > best.0 {
                best = (n, m);
            }
        }
        eprintln!("洞微三合：地支阴阳读法最多合 {} / 10（命 {}）", best.0, best.1);
        assert!(best.0 < 10);
        // 以大限起岁之奇偶为阴阳（奇为陽、先逆取）：十限合九，唯相貌（十六岁起）不合
        let mut age = 1u32;
        let mut hit = 0;
        for (b, n) in DONG_WEI_BLOCKS {
            if n >= 5 && (age % 2 == 1) == DONG_WEI_BIAO_PLUS4[b] {
                hit += 1;
            }
            age += n;
        }
        assert_eq!(hit, 9);
    }
}
