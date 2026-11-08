//! 年神方位：《協紀辨方書》卷三「義例一」诸年神，按年干支定其所在方位（二十四山：十二支、八干、四维）。
//!
//! 起例逐条依卷三所引旧文与协纪按语（见各神注）；以乾隆三十年乙酉时宪书「年神方位之圖」核过（见 doc/folk/nianshen.md）。
//! 方位以年计，不论立春前后：调用方按立春定年干支。

/// 十二支
const ZHI: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];
/// 十干
const GAN: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];

/// 一位年神及其所在方位（可多处，如金神）
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NianShen {
    pub name: &'static str,
    /// 吉神（岁德、岁德合、岁干合、岁枝德、奏书、博士）为 true
    pub ji: bool,
    /// 方位：支（子……亥）、干（甲乙丙丁庚辛壬癸；岁德、岁德合可为戊己，即中宫）、维（乾坤艮巽）
    pub positions: Vec<&'static str>,
}

fn zhi(k: i32) -> &'static str {
    ZHI[k.rem_euclid(12) as usize]
}

/// 太岁所在之方：0 东（寅卯辰）、1 南（巳午未）、2 西（申酉戌）、3 北（亥子丑）
fn fang(z: i32) -> usize {
    (((z - 2).rem_euclid(12)) / 3) as usize
}

/// 三合局：0 水（申子辰）、1 金（巳酉丑）、2 火（寅午戌）、3 木（亥卯未）
fn san_he(z: i32) -> usize {
    (z.rem_euclid(4)) as usize
}

fn shen(name: &'static str, ji: bool, positions: Vec<&'static str>) -> NianShen {
    NianShen { name, ji, positions }
}

/// 某年干支（0 甲子 … 59 癸亥）的年神方位，次第依卷三。
pub fn nian_shen(year_gz: u8) -> Vec<NianShen> {
    let g = (year_gz % 10) as usize;
    let z = (year_gz % 12) as i32;
    let f = fang(z);
    let sh = san_he(z);
    // 岁德：阳干自得，阴干取所合之阳干（卷三引《廣聖厯》）
    let sui_de = if g % 2 == 0 { GAN[g] } else { GAN[(g + 5) % 10] };
    // 岁德合：岁德五合之干（《考原》）
    let sui_de_he = if g % 2 == 0 { GAN[(g + 5) % 10] } else { GAN[g] };
    // 岁干合：与岁干相合之干（《厯例》）
    let sui_gan_he = GAN[(g + 5) % 10];
    // 四维：奏书、博士、力士、蚕室（按太岁所在之方；《蓬瀛書》《堪輿經》《明時總要》《廣聖厯》）
    let zou_shu = ["艮", "巽", "坤", "乾"][f];
    let bo_shi = ["坤", "乾", "艮", "巽"][f];
    let li_shi = ["巽", "坤", "乾", "艮"][f];
    let can_shi = ["乾", "艮", "巽", "坤"][f];
    // 大将军：寅卯辰在子、巳午未在卯、申酉戌在午、亥子丑在酉（《考原》）
    let da_jiang_jun = ["子", "卯", "午", "酉"][f];
    // 蚕官：岁方五行之墓；蚕命：岁方五行长生（黎幹、《考原》）
    let can_guan = ["戌", "丑", "辰", "未"][f];
    let can_ming = ["亥", "寅", "巳", "申"][f];
    let tai_yin = zhi(z - 2);
    // 劫煞、灾煞、岁煞（李鼎祚、曹震圭）；伏兵、大祸（黎幹）：按三合局
    let jie_sha = ["巳", "寅", "亥", "申"][sh];
    let zai_sha = ["午", "卯", "子", "酉"][sh];
    let sui_sha = ["未", "辰", "丑", "戌"][sh];
    let (fu_bing, da_huo) = [("丙", "丁"), ("甲", "乙"), ("壬", "癸"), ("庚", "辛")][sh];
    // 黄幡：三合墓辰（《廣聖厯》）；豹尾与之对冲
    let huang_fan_k = [4, 1, 10, 7][sh];
    // 岁刑：子刑卯、丑刑戌、寅刑巳、卯刑子、巳刑申、未刑丑、申刑寅、戌刑未，辰午酉亥自刑（曾門經）
    let sui_xing = ["卯", "戌", "巳", "子", "辰", "申", "午", "丑", "寅", "酉", "未", "亥"][z as usize];
    // 大煞：子年起子逆行四正（《明時總要》）
    let da_sha = ["子", "酉", "午", "卯"][(z % 4) as usize];
    // 飞廉（《廣聖厯》）
    let fei_lian = ["申", "酉", "戌", "巳", "午", "未", "寅", "卯", "辰", "亥", "子", "丑"][z as usize];
    // 金神：年干五虎遁得庚辛及纳音金之支（《堪輿經》）
    let jin_shen: Vec<&str> = match g % 5 {
        0 => vec!["午", "未", "申", "酉"],
        1 => vec!["辰", "巳"],
        2 => vec!["子", "丑", "寅", "卯", "午", "未"],
        3 => vec!["寅", "卯", "戌", "亥"],
        _ => vec!["申", "酉", "子", "丑"],
    };
    // 五鬼：子年在辰逆行（《厯例》）
    let wu_gui = zhi(4 - z);
    // 破败五鬼：岁干所纳之卦的对冲之卦（《厯例》、曹震圭）
    let po_bai = ["巽", "艮", "坤", "卯", "午", "子", "酉", "乾", "巽", "艮"][g];

    let mut v = vec![
        shen("歲德", true, vec![sui_de]),
        shen("歲德合", true, vec![sui_de_he]),
        shen("歲幹合", true, vec![sui_gan_he]),
        shen("歲枝德", true, vec![zhi(z + 5)]),
        shen("太歲", false, vec![zhi(z)]),
        shen("歲破", false, vec![zhi(z + 6)]),
        shen("大耗", false, vec![zhi(z + 6)]),
        shen("大將軍", false, vec![da_jiang_jun]),
        shen("奏書", true, vec![zou_shu]),
        shen("博士", true, vec![bo_shi]),
        shen("力士", false, vec![li_shi]),
        shen("蠶室", false, vec![can_shi]),
        shen("蠶官", false, vec![can_guan]),
        shen("蠶命", false, vec![can_ming]),
        shen("喪門", false, vec![zhi(z + 2)]),
        shen("太陰", false, vec![tai_yin]),
        shen("弔客", false, vec![tai_yin]),
    ];
    // 羣醜：岁在四孟，太阴与大将军会于四仲（《蓬瀛書》）
    if tai_yin == da_jiang_jun {
        v.push(shen("羣醜", false, vec![tai_yin]));
    }
    v.extend([
        shen("官符", false, vec![zhi(z + 4)]),
        shen("畜官", false, vec![zhi(z + 4)]),
        shen("白虎", false, vec![zhi(z + 8)]),
        shen("黄幡", false, vec![ZHI[huang_fan_k]]),
        shen("豹尾", false, vec![zhi(huang_fan_k as i32 + 6)]),
        shen("病符", false, vec![zhi(z - 1)]),
        shen("死符", false, vec![zhi(z + 5)]),
        shen("小耗", false, vec![zhi(z + 5)]),
        shen("劫煞", false, vec![jie_sha]),
        shen("災煞", false, vec![zai_sha]),
        shen("歲煞", false, vec![sui_sha]),
        shen("伏兵", false, vec![fu_bing]),
        shen("大禍", false, vec![da_huo]),
        shen("歲刑", false, vec![sui_xing]),
        shen("大煞", false, vec![da_sha]),
        shen("飛廉", false, vec![fei_lian]),
        shen("金神", false, jin_shen),
        shen("五鬼", false, vec![wu_gui]),
        shen("破敗五鬼", false, vec![po_bai]),
    ]);
    v
}

/// 二十四山（自子起顺行）各位所临的年神名；无神之位为空。时宪书年神图于空位注「空」。
pub fn by_position(year_gz: u8) -> Vec<(&'static str, Vec<&'static str>)> {
    const SHAN: [&str; 24] = [
        "子", "癸", "丑", "艮", "寅", "甲", "卯", "乙", "辰", "巽", "巳", "丙", "午", "丁", "未", "坤", "申", "庚", "酉", "辛",
        "戌", "乾", "亥", "壬",
    ];
    // 岁德、岁德合时宪书另行印「歲德在某合在某」；岁干合、岁枝德卷三《考原》谓「今厯日歲轉吉神無此條」，
    // 乾隆三十年图寅位（岁枝德所在）亦不见。均不入图
    let skip = ["歲德", "歲德合", "歲幹合", "歲枝德"];
    let all = nian_shen(year_gz);
    SHAN.iter()
        .map(|&p| (p, all.iter().filter(|s| !skip.contains(&s.name) && s.positions.contains(&p)).map(|s| s.name).collect()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(v: &[NianShen], name: &str) -> Vec<&'static str> {
        v.iter().find(|s| s.name == name).map(|s| s.positions.clone()).unwrap_or_default()
    }

    /// 乾隆三十年乙酉时宪书「年神方位之圖」可读之位（见 doc/folk/nianshen.md）
    #[test]
    fn yi_you_1765_vs_shixianshu() {
        let v = nian_shen(21); // 乙酉
        let expect = [
            ("歲德", "庚"), ("歲德合", "乙"),
            ("太歲", "酉"), ("歲刑", "酉"), ("大煞", "酉"), ("病符", "申"), ("奏書", "坤"), ("力士", "乾"),
            ("喪門", "亥"), ("飛廉", "亥"), ("太陰", "未"), ("豹尾", "未"), ("五鬼", "未"),
            ("官符", "丑"), ("畜官", "丑"), ("黄幡", "丑"), ("蠶室", "巽"), ("博士", "艮"), ("破敗五鬼", "艮"),
            ("歲煞", "辰"), ("蠶官", "辰"), ("大禍", "乙"), ("歲破", "卯"), ("大耗", "卯"), ("災煞", "卯"),
            ("伏兵", "甲"), ("劫煞", "寅"), ("死符", "寅"), ("小耗", "寅"), ("蠶命", "巳"),
        ];
        for (n, p) in expect {
            assert_eq!(at(&v, n), vec![p], "{n}");
        }
        assert_eq!(at(&v, "金神"), vec!["辰", "巳"]);
        // 图中注「空」之位
        let bp = by_position(21);
        for p in ["庚", "辛", "戌", "壬"] {
            assert!(bp.iter().find(|(q, _)| *q == p).unwrap().1.is_empty(), "{p} 应空");
        }
    }

    #[test]
    fn relations() {
        let opp = |a: &str, b: &str| {
            let i = ZHI.iter().position(|x| *x == a).unwrap() as i32;
            ZHI[(i + 6).rem_euclid(12) as usize] == b
        };
        for y in 0..60u8 {
            let v = nian_shen(y);
            // 卷三：博士与奏书对冲，蚕室与力士对冲，豹尾与黄幡对冲，灾煞居劫煞前一辰
            let pair = |a: &str| at(&v, a)[0];
            let wei_opp = |a: &str, b: &str| matches!((a, b), ("乾", "巽") | ("巽", "乾") | ("坤", "艮") | ("艮", "坤"));
            assert!(wei_opp(pair("奏書"), pair("博士")));
            assert!(wei_opp(pair("力士"), pair("蠶室")));
            assert!(opp(pair("黄幡"), pair("豹尾")));
            assert!(opp(pair("太歲"), pair("歲破")));
            let j = ZHI.iter().position(|x| *x == pair("劫煞")).unwrap();
            assert_eq!(pair("災煞"), ZHI[(j + 1) % 12]);
            // 卷三黄幡总论：子午卯酉年黄幡即官符、豹尾即吊客；寅申巳亥年黄幡即白虎、豹尾即丧门；辰戌丑未年黄幡即太岁、豹尾即岁破
            let (hf, bw) = match y % 12 % 3 {
                0 => ("官符", "弔客"),
                2 => ("白虎", "喪門"),
                _ => ("太歲", "歲破"),
            };
            assert_eq!(pair("黄幡"), pair(hf), "年 {y}");
            assert_eq!(pair("豹尾"), pair(bw), "年 {y}");
            // 卷三飞廉按语：子丑寅午未申年飞廉即白虎，卯辰巳酉戌亥年即丧门
            let fl = if [0, 1, 2, 6, 7, 8].contains(&(y % 12)) { "白虎" } else { "喪門" };
            assert_eq!(pair("飛廉"), pair(fl), "年 {y}");
            // 卷三五鬼按语：子午年与官符同位、丑未年与丧门、寅申年与太岁、卯酉年与太阴、辰戌年与白虎、巳亥年与岁破
            let wg = ["官符", "喪門", "太歲", "太陰", "白虎", "歲破"][(y % 12 % 6) as usize];
            assert_eq!(pair("五鬼"), pair(wg), "年 {y}");
            // 羣醜只在四孟年
            assert_eq!(v.iter().any(|s| s.name == "羣醜"), [2, 5, 8, 11].contains(&(y % 12)), "年 {y}");
        }
    }
}
