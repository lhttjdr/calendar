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
//! 另有「遁年奇法」顺飞（統宗、元靈經），只定六仪三奇之宫，见 [`dun_nian_qi`]。

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

/// 「遁年奇法」（顺飞）：《奇門遁甲統宗》卷二「又一本遁年奇法」：「艮上起甲子顺行……以甲子起艮，以甲戌起离，以甲申起坎，
/// 以甲午起坤」，庚子年「奇头在坤」，「阳遁逆奇，丁奇在中，丙奇在乾，乙奇在兑」；《奇門遁甲元靈經》卷一歌「甲子旬头起艮，
/// 甲戌飞入离宫，猿猴来入水晶宫，甲午坤中不动」同。即六仪三奇自艮八起甲子戊，按洛书数序顺飞（可入中五）。
/// 返回各宫之干（1—9）与本年旬首（「奇头」）所在宫。`yang`：立春（冬至）后阳遁顺飞；夏至后「用阴遁逆行，余仿此」，
/// 起宫与逆飞之式原文未详，取艮八起甲子逆飞，未核。
pub fn dun_nian_qi(year_gz: u8, yang: bool) -> ([&'static str; 10], u8) {
    let d = super::pan::di_pan(yang, 8);
    let yi = super::pan::YI_QI[((year_gz % 60) / 10) as usize];
    let p = (1..=9u8).find(|&p| d[p as usize] == yi).unwrap();
    (d, p)
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
        let (d, tou) = dun_nian_qi(36, true); // 庚子
        assert_eq!(tou, 2);
        assert_eq!((d[5], d[6], d[7]), ("丁", "丙", "乙"));
        assert_eq!((d[8], d[9], d[1], d[2]), ("戊", "己", "庚", "辛"));
    }
}
