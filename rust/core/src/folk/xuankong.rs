//! 玄空飞星（风水理气）：宅运盘、山盘、向盘，下卦与替卦，兼向空亡，格局；流年、流月紫白飞星。
//!
//! 依据与口径（详见 doc/folk/xuankong.md）：
//! - 挨星诀（替卦所据）：《青囊奧語》「坤壬乙，巨門從頭出；艮丙辛，位位是破星；巽辰亥，儘是武曲位；甲癸申，貪狼一路行」
//!   （据维基文库录文，`vendor/folk/wikisource/青囊奧語/`）。扩为二十四山之替星表（子→貪狼、寅午庚丁→右弼等）
//!   与运盘、山向盘入中飞布之法，出沈竹礽《沈氏玄空學》（民国排印；原书未见，出处未核）。
//! - 三元九运：协纪卷八「康熙二十三年甲子為上元」（[`super::taiyi::san_yuan_jiu_yun`]）。
//! - 年、月紫白：协纪卷八（[`super::jiuxing`]）；入中顺飞。日、时紫白协纪未收，不提供。
//!
//! 九宫以洛书数 1—9 为下标（`[u8; 9]` 的下标 k−1 对应洛书 k 宫：一坎北、二坤西南、三震东、四巽东南、五中、六乾西北、
//! 七兑西、八艮东北、九离南）。飞布次序：中五 → 六 → 七 → 八 → 九 → 一 → 二 → 三 → 四（洛书数序）。

use super::jiuxing;
use super::taiyi::{san_yuan_jiu_yun, JiuYun};

/// 二十四山，自子起顺行（方位角：子 0°，每山 15°）
pub const SHAN24: [&str; 24] = [
    "子", "癸", "丑", "艮", "寅", "甲", "卯", "乙", "辰", "巽", "巳", "丙", "午", "丁", "未", "坤", "申", "庚", "酉", "辛", "戌", "乾",
    "亥", "壬",
];

/// 洛书宫名（下标 = 宫数 − 1）
pub const GONG: [&str; 9] = ["坎", "坤", "震", "巽", "中", "乾", "兌", "艮", "離"];

/// 九星名（下标 = 星数 − 1）
pub const XING: [&str; 9] = ["貪狼", "巨門", "祿存", "文曲", "廉貞", "武曲", "破軍", "左輔", "右弼"];

/// 三元龙：每宫三山之第一、二、三位（地元、天元、人元）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YuanLong {
    Di,
    Tian,
    Ren,
}

fn wrap9(n: i32) -> u8 {
    let r = n.rem_euclid(9);
    if r == 0 { 9 } else { r as u8 }
}

/// 八宫（洛书数）自北起顺时针：坎一、艮八、震三、巽四、离九、坤二、兑七、乾六
const RING8: [u8; 8] = [1, 8, 3, 4, 9, 2, 7, 6];

/// 某山（[`SHAN24`] 下标）所在之宫（洛书数）与三元龙。壬子癸属坎，丑艮寅属艮……戌乾亥属乾。
pub fn gong_of(shan: usize) -> (u8, YuanLong) {
    // 壬（23）为坎之地元：自壬起每三山一宫
    let k = (shan + 1) % 24;
    let g = RING8[k / 3];
    let y = [YuanLong::Di, YuanLong::Tian, YuanLong::Ren][k % 3];
    (g, y)
}

/// 某宫某元之山（[`SHAN24`] 下标）；中五无山
pub fn shan_at(gong: u8, yuan: YuanLong) -> Option<usize> {
    let i = RING8.iter().position(|&g| g == gong)?;
    let j = match yuan {
        YuanLong::Di => 0,
        YuanLong::Tian => 1,
        YuanLong::Ren => 2,
    };
    Some((i * 3 + j + 23) % 24)
}

/// 二十四山阴阳（阳顺飞、阴逆飞）：四正宫（坎震兑离）地元阳、天元人元阴；四隅宫（坤巽乾艮）地元阴、天元人元阳。
/// 即甲庚丙壬、乾坤艮巽、寅申巳亥为阳，子午卯酉、乙辛丁癸、辰戌丑未为阴（沈氏；出处未核）。
pub fn is_yang(shan: usize) -> bool {
    let (g, y) = gong_of(shan);
    let zheng = matches!(g, 1 | 3 | 7 | 9);
    match y {
        YuanLong::Di => zheng,
        _ => !zheng,
    }
}

/// 替星（起星）表：二十四山各所替之星数。《青囊奧語》四句定十二山（坤壬乙→二、艮丙辛→七、巽辰亥→六、甲癸申→一），
/// 余十二山（子→一，卯未→二，乾戌巳→六，酉丑→七，寅午庚丁→九）依沈氏所扩（出处未核）。
pub fn ti_xing(shan: usize) -> u8 {
    match SHAN24[shan] {
        "子" | "癸" | "甲" | "申" => 1,
        "壬" | "卯" | "乙" | "未" | "坤" => 2,
        "乾" | "亥" | "辰" | "巽" | "巳" | "戌" => 6,
        "艮" | "丙" | "辛" | "酉" | "丑" => 7,
        "寅" | "午" | "庚" | "丁" => 9,
        _ => unreachable!(),
    }
}

/// 洛书顺飞（`shun` 为 true）或逆飞：中宫为 `center`，返回九宫之星（下标 = 宫数 − 1）
pub fn fly(center: u8, shun: bool) -> [u8; 9] {
    let mut p = [0u8; 9];
    for k in 1..=9i32 {
        let d = k - 5;
        p[(k - 1) as usize] = wrap9(center as i32 + if shun { d } else { -d });
    }
    p
}

/// 五黄入中时山盘、向盘的顺逆依何而定
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WuRule {
    /// 依坐山（山盘）、向首（向盘）本山之阴阳（沈氏通行；出处未核）。缺省
    BenShan,
    /// 依运星原宫同元之山之阴阳（另说；出处未核）
    YunGong,
}

/// 替卦入中之星的顺逆依何而定
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TiShunRule {
    /// 仍依原入中之星原宫同元之山的阴阳，只换入中之数（沈氏通行；出处未核）。缺省
    YuanXing,
    /// 依替星原宫同元之山的阴阳（另说；出处未核）
    TiXing,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct XkOptions {
    pub wu: WuRule,
    pub ti_shun: TiShunRule,
    /// 兼向超过此度数（离本山中线）用替卦；缺省 3°（通行；出处未核）
    pub ti_threshold_deg: f64,
    /// 距山界此度数以内为空亡（同宫两山之间小空亡，两宫之间大空亡）；缺省 1.5°（出处未核）
    pub kong_wang_deg: f64,
}

pub const DEFAULT_OPTIONS: XkOptions =
    XkOptions { wu: WuRule::BenShan, ti_shun: TiShunRule::YuanXing, ti_threshold_deg: 3.0, kong_wang_deg: 1.5 };

/// 坐向度数（坐山中线的方位角，度；向 = 坐 + 180°）所落之山与偏离本山中线的度数（−7.5—7.5，正为顺时针）
pub fn shan_of_azimuth(az_deg: f64) -> (usize, f64) {
    let a = az_deg.rem_euclid(360.0);
    let i = ((a + 7.5) / 15.0).floor() as usize % 24;
    let off = a - 15.0 * i as f64;
    let off = if off > 180.0 { off - 360.0 } else { off };
    (i, off)
}

/// 空亡
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KongWang {
    /// 同宫两山之间
    Xiao,
    /// 两宫（两卦）之间
    Da,
}

/// 一格局
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeJu {
    pub name: &'static str,
    pub note: &'static str,
}

/// 宅盘
#[derive(Clone, Debug, PartialEq)]
pub struct ZhaiPan {
    pub yun: u8,
    /// 坐山、向首（[`SHAN24`] 下标）
    pub zuo: usize,
    pub xiang: usize,
    /// 是否用替卦
    pub ti: bool,
    pub yun_pan: [u8; 9],
    pub shan_pan: [u8; 9],
    pub xiang_pan: [u8; 9],
    /// 山盘、向盘入中之星与顺逆
    pub shan_in: u8,
    pub xiang_in: u8,
    pub shan_shun: bool,
    pub xiang_shun: bool,
    pub kong_wang: Option<KongWang>,
    pub ge_ju: Vec<GeJu>,
}

/// 入中之星 m 的顺逆：以 m 之原宫与 `ref_shan` 同元之山的阴阳定；m 为五时按 [`WuRule`]
fn shun_of(m: u8, ref_shan: usize, yun: u8, o: &XkOptions) -> bool {
    let (_, y) = gong_of(ref_shan);
    if m == 5 {
        return match o.wu {
            WuRule::BenShan => is_yang(ref_shan),
            WuRule::YunGong => shan_at(if yun == 5 { gong_of(ref_shan).0 } else { yun }, y).map(is_yang).unwrap_or(true),
        };
    }
    is_yang(shan_at(m, y).unwrap())
}

/// 入中之星（山盘或向盘）：取运盘 `ref_shan` 所在宫之星 m；替卦则换为 m 原宫同元之山的替星
fn enter(yun_pan: &[u8; 9], ref_shan: usize, ti: bool, yun: u8, o: &XkOptions) -> (u8, bool) {
    let (g, y) = gong_of(ref_shan);
    let m = yun_pan[g as usize - 1];
    let shun_yuan = shun_of(m, ref_shan, yun, o);
    if !ti || m == 5 {
        return (m, shun_yuan);
    }
    let src = shan_at(m, y).unwrap();
    let t = ti_xing(src);
    let shun = match o.ti_shun {
        TiShunRule::YuanXing => shun_yuan,
        TiShunRule::TiXing => shun_of(t, ref_shan, yun, o),
    };
    (t, shun)
}

/// 排宅盘。`yun` 1—9；`zuo` 坐山；`ti` 是否用替卦（兼向超过限度时由 [`zhai_pan_azimuth`] 定）。
pub fn zhai_pan(yun: u8, zuo: usize, ti: bool, o: &XkOptions) -> ZhaiPan {
    let xiang = (zuo + 12) % 24;
    let yun_pan = fly(yun, true);
    let (shan_in, shan_shun) = enter(&yun_pan, zuo, ti, yun, o);
    let (xiang_in, xiang_shun) = enter(&yun_pan, xiang, ti, yun, o);
    let shan_pan = fly(shan_in, shan_shun);
    let xiang_pan = fly(xiang_in, xiang_shun);
    let mut p = ZhaiPan {
        yun,
        zuo,
        xiang,
        ti,
        yun_pan,
        shan_pan,
        xiang_pan,
        shan_in,
        xiang_in,
        shan_shun,
        xiang_shun,
        kong_wang: None,
        ge_ju: vec![],
    };
    p.ge_ju = ge_ju(&p);
    p
}

/// 按坐山方位角（度，真北起顺时针）排宅盘：定山、兼向替卦与否、空亡
pub fn zhai_pan_azimuth(yun: u8, zuo_az_deg: f64, o: &XkOptions) -> ZhaiPan {
    let (zuo, off) = shan_of_azimuth(zuo_az_deg);
    let ti = off.abs() > o.ti_threshold_deg;
    let mut p = zhai_pan(yun, zuo, ti, o);
    if 7.5 - off.abs() <= o.kong_wang_deg {
        // 偏向的一侧之邻山：若与本山不同宫则大空亡
        let nb = if off > 0.0 { (zuo + 1) % 24 } else { (zuo + 23) % 24 };
        p.kong_wang = Some(if gong_of(nb).0 == gong_of(zuo).0 { KongWang::Xiao } else { KongWang::Da });
    }
    p
}

/// 某年（干支年，立春分）所属之运
pub fn yun_of_year(year: i32, scheme: JiuYun) -> u8 {
    san_yuan_jiu_yun(year, scheme).1
}

/// 流年紫白飞星盘（年星入中顺飞）
pub fn year_pan(ganzhi_year: i32) -> [u8; 9] {
    fly(jiuxing::year_star(ganzhi_year), true)
}

/// 流月紫白飞星盘（月星入中顺飞）；`year_zhi` 年支序，`month_ord` 节气月序（寅月 0）
pub fn month_pan(year_zhi: u8, month_ord: u8) -> [u8; 9] {
    fly(jiuxing::month_star(year_zhi, month_ord), true)
}

/// 河图生成之配（一六、二七、三八、四九）
fn he_tu(a: u8, b: u8) -> bool {
    a != 5 && b != 5 && (a as i32 - b as i32).abs() == 5
}

/// 城门：向宫左右两宫中与向宫成河图之配者为正城门，另一为副城门。可用与否：城门宫之运星入中，
/// 以其原宫与向首同元之山阴阳定，阳则顺飞可用（通行说；出处未核）。返回 (宫, 正否, 可用否)。
pub fn cheng_men(p: &ZhaiPan, o: &XkOptions) -> Vec<(u8, bool, bool)> {
    let (gx, _) = gong_of(p.xiang);
    let i = RING8.iter().position(|&g| g == gx).unwrap();
    [RING8[(i + 7) % 8], RING8[(i + 1) % 8]]
        .into_iter()
        .map(|c| {
            let s = p.yun_pan[c as usize - 1];
            let ok = shun_of(s, p.xiang, p.yun, o);
            (c, he_tu(c, gx), ok)
        })
        .collect()
}

/// 格局
pub fn ge_ju(p: &ZhaiPan) -> Vec<GeJu> {
    let mut v = Vec::new();
    let (gz, _) = gong_of(p.zuo);
    let (gx, _) = gong_of(p.xiang);
    let s = |g: u8| p.shan_pan[g as usize - 1];
    let x = |g: u8| p.xiang_pan[g as usize - 1];
    let w = p.yun;
    let mut add = |name, note| v.push(GeJu { name, note });
    match (s(gz) == w, x(gx) == w, x(gz) == w, s(gx) == w) {
        (true, true, _, _) => add("旺山旺向", "当运山星到坐、向星到向"),
        (_, _, true, true) => add("上山下水", "当运向星到坐、山星到向"),
        (false, true, false, true) => add("雙星到向", "当运山星、向星同到向首"),
        (true, false, true, false) => add("雙星到坐", "当运山星、向星同到坐山"),
        _ => {}
    }
    let luoshu = fly(5, true);
    let fan = fly(5, false);
    if p.shan_pan == luoshu {
        add("山星伏吟", "山盘与元旦盘（洛书本位）同");
    }
    if p.xiang_pan == luoshu {
        add("向星伏吟", "向盘与元旦盘同");
    }
    if p.shan_pan == fan {
        add("山星反吟", "山盘与元旦盘合十（五入中逆飞）");
    }
    if p.xiang_pan == fan {
        add("向星反吟", "向盘与元旦盘合十");
    }
    if (0..9).all(|k| p.shan_pan[k] + p.yun_pan[k] == 10) {
        add("山星合十", "各宫山星与运星合十");
    }
    if (0..9).all(|k| p.xiang_pan[k] + p.yun_pan[k] == 10) {
        add("向星合十", "各宫向星与运星合十");
    }
    let sorted = |k: usize| {
        let mut a = [p.yun_pan[k], p.shan_pan[k], p.xiang_pan[k]];
        a.sort();
        a
    };
    // 连珠：三星为相连三数（含九一相接，如八九一）
    let lian = (0..9).all(|k| {
        let t = sorted(k);
        (1..=9u8).any(|st| {
            let mut c = [st, wrap9(st as i32 + 1), wrap9(st as i32 + 2)];
            c.sort();
            c == t
        })
    });
    if lian {
        add("連珠三般卦", "各宫运、山、向三星相连（如一二三、八九一）");
    }
    if (0..9).all(|k| {
        let a = sorted(k);
        a[0] % 3 == a[1] % 3 && a[1] % 3 == a[2] % 3 && a[0] != a[1] && a[1] != a[2]
    }) {
        add("父母三般卦", "各宫运、山、向三星为一四七、二五八、三六九之一");
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idx(s: &str) -> usize {
        SHAN24.iter().position(|x| *x == s).unwrap()
    }

    #[test]
    fn mountains_and_yinyang() {
        assert_eq!(gong_of(idx("壬")), (1, YuanLong::Di));
        assert_eq!(gong_of(idx("子")), (1, YuanLong::Tian));
        assert_eq!(gong_of(idx("癸")), (1, YuanLong::Ren));
        assert_eq!(gong_of(idx("丑")), (8, YuanLong::Di));
        assert_eq!(gong_of(idx("乾")), (6, YuanLong::Tian));
        for (s, y) in [("甲", true), ("庚", true), ("丙", true), ("壬", true), ("乾", true), ("坤", true), ("艮", true), ("巽", true),
            ("寅", true), ("申", true), ("巳", true), ("亥", true), ("子", false), ("午", false), ("卯", false), ("酉", false),
            ("乙", false), ("辛", false), ("丁", false), ("癸", false), ("辰", false), ("戌", false), ("丑", false), ("未", false)]
        {
            assert_eq!(is_yang(idx(s)), y, "{s}");
        }
        // 对宫同元之山阴阳相同（坐向两盘同法）
        for i in 0..24 {
            assert_eq!(is_yang(i), is_yang((i + 12) % 24));
        }
        for i in 0..24 {
            let (g, y) = gong_of(i);
            assert_eq!(shan_at(g, y), Some(i));
        }
    }

    #[test]
    fn ti_xing_ao_yu() {
        // 《青囊奧語》四句十二山
        for (ss, n) in [(["坤", "壬", "乙"], 2), (["艮", "丙", "辛"], 7), (["巽", "辰", "亥"], 6), (["甲", "癸", "申"], 1)] {
            for s in ss {
                assert_eq!(ti_xing(idx(s)), n, "{s}");
            }
        }
    }

    #[test]
    fn azimuth() {
        assert_eq!(shan_of_azimuth(0.0), (idx("子"), 0.0));
        assert_eq!(shan_of_azimuth(359.0).0, idx("子"));
        assert_eq!(shan_of_azimuth(180.0).0, idx("午"));
        assert_eq!(shan_of_azimuth(45.0).0, idx("艮"));
        let p = zhai_pan_azimuth(8, 7.0, &DEFAULT_OPTIONS);
        assert!(p.ti);
        assert_eq!(p.kong_wang, Some(KongWang::Xiao)); // 子癸之间
        let p = zhai_pan_azimuth(8, 22.0, &DEFAULT_OPTIONS);
        assert_eq!(p.kong_wang, Some(KongWang::Da)); // 癸丑之间（坎艮两宫）
        assert!(!zhai_pan_azimuth(8, 2.0, &DEFAULT_OPTIONS).ti);
    }

    /// 飞星定理：坐向对宫洛书数和十，故（下卦、五不入中时）旺山 ⟺ 山盘逆飞，旺向 ⟺ 向盘逆飞；
    /// 山顺向逆 ⟺ 双星到向，山逆向顺 ⟺ 双星到坐，山向俱顺 ⟺ 上山下水
    #[test]
    fn theorem_shun_ni() {
        for yun in 1..=9u8 {
            for zuo in 0..24 {
                let p = zhai_pan(yun, zuo, false, &DEFAULT_OPTIONS);
                assert_eq!(p.yun_pan[4], yun);
                let mut seen = p.shan_pan;
                seen.sort();
                assert_eq!(seen, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
                let name = p.ge_ju.first().map(|g| g.name).unwrap_or("");
                let expect = match (p.shan_shun, p.xiang_shun) {
                    (false, false) => "旺山旺向",
                    (true, true) => "上山下水",
                    (true, false) => "雙星到向",
                    (false, true) => "雙星到坐",
                };
                assert_eq!(name, expect, "{yun} 运 {}", SHAN24[zuo]);
            }
        }
    }

    /// 通行之例（未核原书）：八运子山午向下卦，山星四入中顺飞、向星三入中逆飞，双星到向
    #[test]
    fn ba_yun_zi_wu() {
        let p = zhai_pan(8, idx("子"), false, &DEFAULT_OPTIONS);
        assert_eq!((p.shan_in, p.shan_shun, p.xiang_in, p.xiang_shun), (4, true, 3, false));
        assert_eq!(p.shan_pan[8], 8);
        assert_eq!(p.xiang_pan[8], 8);
        assert_eq!(p.ge_ju[0].name, "雙星到向");
        // 正城门：离向左右巽、坤，巽四与离九为河图四九之配
        let cm = cheng_men(&p, &DEFAULT_OPTIONS);
        assert!(cm.iter().any(|&(g, zheng, _)| g == 4 && zheng));
        assert!(cm.iter().any(|&(g, zheng, _)| g == 2 && !zheng));
    }

    #[test]
    fn ti_gua_changes_entering_star() {
        // 八运壬山丙向：运盘坎宫四、离宫三；壬（地元）→ 巽宫地元辰替六；丙（地元）→ 震宫地元甲替一
        let p = zhai_pan(8, idx("壬"), true, &DEFAULT_OPTIONS);
        assert_eq!((p.shan_in, p.xiang_in), (6, 1));
        let q = zhai_pan(8, idx("壬"), false, &DEFAULT_OPTIONS);
        assert_eq!((q.shan_in, q.xiang_in), (4, 3));
    }

    #[test]
    fn liu_nian() {
        // 2026 丙午年：年星一白（协纪：1684 上元一白起，逐年逆转）→ 2024 三碧，2026 一白
        assert_eq!(year_pan(2026)[4], jiuxing::year_star(2026));
        assert_eq!(fly(1, true)[5], 2); // 一白入中，乾六宫二黑
    }
}
