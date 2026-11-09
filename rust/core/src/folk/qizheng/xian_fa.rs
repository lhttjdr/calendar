//! 七政四余诸家限法（《星學大成》卷七《諸家限例》）：洞微飛限、竹羅三限。
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

/// 一限之内第 `k` 年（1 起）所在之宫：起于 `start`，先住起數 `qi` 年，其后每十年逆行二宫（「隔節數三宫逆行」），
/// 余下不足十年者逐年顺行一宫（「零年順轉逐宫移」）。卷七例「土為初限在巳上，便從巳上起五嵗，大數十五在卯，二十五丑，
/// 二十六却囘寅」
pub fn zhu_luo_gong_in(start: u8, qi: u32, len: u32, k: u32) -> u8 {
    let s = start as i32;
    if k <= qi {
        return start;
    }
    let decades = (len.saturating_sub(qi)) / 10; // 整十之数
    let after = k - qi;
    let z = if after <= decades * 10 {
        s - 2 * ((after - 1) / 10 + 1) as i32
    } else {
        s - 2 * decades as i32 + (after - decades * 10) as i32
    };
    z.rem_euclid(12) as u8
}

/// 竹羅一限
#[derive(Clone, Debug, PartialEq)]
pub struct ZhuLuoXian {
    /// 限星（十一曜下标）
    pub yao: usize,
    /// 起止之岁（含）
    pub from: u32,
    pub to: u32,
    /// 起宫（限星所躔之宫）
    pub start: u8,
}

/// 竹羅三限：三限各自限星所躔之宫起（`gongs`，与 `stars` 同序），返回三限与逐岁之宫（下标 0 为一岁）
pub fn zhu_luo(stars: [usize; 3], gongs: [u8; 3]) -> (Vec<ZhuLuoXian>, Vec<u8>) {
    let mut xian = Vec::new();
    let mut per = Vec::new();
    let mut age = 0u32;
    for (y, g) in stars.iter().zip(gongs) {
        let len = zhu_luo_years(*y);
        let qi = zhu_luo_qi(*y);
        for k in 1..=len {
            per.push(zhu_luo_gong_in(g, qi, len, k));
        }
        xian.push(ZhuLuoXian { yao: *y, from: age + 1, to: age + len, start: g });
        age += len;
    }
    (xian, per)
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

    /// 竹羅：卷七例「土為初限在巳上，便從巳上起五嵗，大數十五在卯，二十五丑，二十六却囘寅」
    #[test]
    fn zhu_luo_example() {
        let g = |k| zhu_luo_gong_in(z('巳'), 5, 26, k);
        assert!((1..=5).all(|k| g(k) == z('巳')));
        assert_eq!(g(15), z('卯'));
        assert_eq!(g(25), z('丑'));
        assert_eq!(g(26), z('寅'));
        // 三方主：丑命夜生（卷七「己酉丑兮金月火，又云夜生中即反其初……可作夜生人起限月金火為例」）
        assert_eq!(zhu_luo_zhu(z('丑'), false, ZhuLuoRule::Ge), [1, 3, 4]);
        // 表与歌：唯寅午戌一组有异
        for m in 0..12u8 {
            for d in [true, false] {
                let (a, b) = (zhu_luo_zhu(m, d, ZhuLuoRule::Ge), zhu_luo_zhu(m, d, ZhuLuoRule::Biao));
                assert_eq!(a == b, m % 4 != 2);
            }
        }
        // 月金火三限：月 1—24、金 25—50、火 51—78
        let (x, per) = zhu_luo([1, 3, 4], [0, 0, 0]);
        assert_eq!((x[2].from, x[2].to, per.len()), (51, 78, 78));
    }
}
