//! 玄空飞星（风水理气）：宅运盘、山盘、向盘，下卦与替卦，兼向空亡，格局；零神正神、城门、兼向界线、逐宫标记；
//! 流年、流月紫白飞星。
//!
//! 依据与口径（详见 doc/folk/xuankong.md）：
//! - 挨星诀（替卦所据）：《青囊奧語》「坤壬乙，巨門從頭出；艮丙辛，位位是破星；巽辰亥，儘是武曲位；甲癸申，貪狼一路行」
//!   （据维基文库录文，`vendor/folk/wikisource/青囊奧語/`）。扩为二十四山之替星表（子→貪狼、寅午庚丁→右弼等）
//!   与运盘、山向盘入中飞布之法，出沈竹礽《沈氏玄空學》（民国排印；原书未见，出处未核）。
//! - 零神、正神：经文《天玉經》「陰陽二字看零正，坐向須知病；若遇正神正位裝，撥水入零堂」「正神百步始成龍，水短便遭凶；
//!   零神不問長何短，吉凶不同斷」（四库本影印第 22—24 页）。四库注以十二支为正神、八干四维为零神（双山三合之读，
//!   [`si_ku_zheng_shen`]）；玄空家以当运之宫为正神、对宫为零神（沈氏；出处未核），二读并存。
//! - 三元九运：协纪卷八「康熙二十三年甲子為上元」（[`super::taiyi::san_yuan_jiu_yun`]）。
//! - 年、月紫白：协纪卷八（[`super::jiuxing`]）；入中顺飞。日、时紫白据《奇門遁甲統宗》「日冬至後用陽局一七四，夏至後用陰局九三六」
//!   与协纪卷三十四「時家亦有三竒紫白，倣月例推」（[`ri_xing`]、[`shi_xing`]）。
//! - 罗经三针（正针、中针、缝针）：协纪卷二、卷三十三（[`Zhen`]）。玄空大卦（六十四卦）另为一系统（[`gua_yun`] 等），不入飞星。
//!
//! 九宫以洛书数 1—9 为下标（`[u8; 9]` 的下标 k−1 对应洛书 k 宫：一坎北、二坤西南、三震东、四巽东南、五中、六乾西北、
//! 七兑西、八艮东北、九离南）。飞布次序：中五 → 六 → 七 → 八 → 九 → 一 → 二 → 三 → 四（洛书数序）。
//! 度数一律用 [`Real`]（方位角，真北起顺时针）。

use super::jiuxing;
use super::taiyi::{san_yuan_jiu_yun, JiuYun, SHANG_YUAN_JIAZI};
use crate::math::real::{real, real_const, Real, RealOps};

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

/// 零神、正神之定法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LingZhengRule {
    /// 当运之数的洛书本宫为正神，其对宫（合十之宫）为零神（沈氏通行；出处未核）。缺省
    YunGong,
    /// 上元（一至四运）以一二三四宫为正神、六七八九宫为零神，下元（六至九运）反之（大玄空上下元零正之说；今人转述，未核）
    ShangXiaYuan,
}

/// 五运无本宫（中五）时零正之寄
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WuJi {
    /// 前十年寄坤二、后十年寄艮八（通行；出处未核）。缺省
    KunGen,
    /// 前十年寄巽四、后十年寄乾六（与 [`JiuYun::WuJiSiLiu`] 同意；另说，未核）
    SiLiu,
    /// 不寄：正神在中五，无零神（另说，未核）
    Zhong,
}

/// 七星打劫真假之判
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DaJieRule {
    /// 离宫打劫（离震乾）为真，坎宫打劫（坎巽兑）为假（沈氏之说；出处未核）。缺省
    LiZhenKanJia,
    /// 离宫、坎宫打劫皆真（另说；未核）
    JieZhen,
}

/// 伏吟、反吟以何盘为准、全盘或逐宫
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FuFanRule {
    /// 全盘与元旦盘（洛书本位，五入中顺飞）同为伏吟、合十为反吟（通行；未核）。缺省
    YuanDanQuan,
    /// 逐宫：该宫山（向）星与洛书本宫数同为伏吟、合十为反吟（局部伏反吟之说；未核）
    YuanDanGong,
    /// 全盘与运盘同为伏吟、合十为反吟（他说；未核）
    YunPanQuan,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct XkOptions {
    pub wu: WuRule,
    pub ti_shun: TiShunRule,
    /// 兼向超过此度数（离本山中线）用替卦；缺省 3°（通行；出处未核）。另说 4.5°（一山五分金、中三分金为正，未核）
    pub ti_threshold_deg: Real,
    /// 距同宫两山之界此度数以内为小空亡；缺省 1.5°（出处未核）
    pub kong_wang_deg: Real,
    /// 距两宫（两卦）之界此度数以内为大空亡；缺省 1.5°（与小空亡同；出处未核）
    pub da_kong_wang_deg: Real,
    pub ling_zheng: LingZhengRule,
    pub wu_ji: WuJi,
    pub da_jie: DaJieRule,
    pub fu_fan: FuFanRule,
}

pub const DEFAULT_OPTIONS: XkOptions = XkOptions {
    wu: WuRule::BenShan,
    ti_shun: TiShunRule::YuanXing,
    ti_threshold_deg: real_const(3.0),
    kong_wang_deg: real_const(1.5),
    da_kong_wang_deg: real_const(1.5),
    ling_zheng: LingZhengRule::YunGong,
    wu_ji: WuJi::KunGen,
    da_jie: DaJieRule::LiZhenKanJia,
    fu_fan: FuFanRule::YuanDanQuan,
};

/// 每山 15°、半山 7.5°
fn shan_w() -> Real {
    real_const(15.0)
}
fn half_w() -> Real {
    real_const(7.5)
}

/// 坐向度数（坐山中线的方位角，度；向 = 坐 + 180°）所落之山与偏离本山中线的度数（−7.5—7.5，正为顺时针）
pub fn shan_of_azimuth(az_deg: Real) -> (usize, Real) {
    let full = real_const(360.0);
    let a = az_deg - (az_deg / full).floor() * full;
    let i = ((a + half_w()) / shan_w()).to_i32_floor().rem_euclid(24) as usize;
    let mut off = a - shan_w() * real(i as i32);
    if off > real_const(180.0) {
        off = off - full;
    }
    (i, off)
}

/// 某山中线的方位角（子 0°，顺时针每山 15°）
pub fn shan_center(shan: usize) -> Real {
    shan_w() * real((shan % 24) as i32)
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
    /// 以方位角排盘时，坐山偏离本山中线之度数（正为顺时针）；按山排盘时为 None
    pub pian_deg: Option<Real>,
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
        pian_deg: None,
        ge_ju: vec![],
    };
    p.ge_ju = ge_ju(&p, o);
    p
}

/// 按坐山方位角（度，真北起顺时针）排宅盘：定山、兼向替卦与否、空亡
pub fn zhai_pan_azimuth(yun: u8, zuo_az_deg: Real, o: &XkOptions) -> ZhaiPan {
    let (zuo, off) = shan_of_azimuth(zuo_az_deg);
    let ti = off.abs() > o.ti_threshold_deg;
    let mut p = zhai_pan(yun, zuo, ti, o);
    p.pian_deg = Some(off);
    // 偏向的一侧之邻山：与本山同宫则小空亡，异宫则大空亡，各按其度数
    let nb = if off > real_const(0.0) { (zuo + 1) % 24 } else { (zuo + 23) % 24 };
    let da = gong_of(nb).0 != gong_of(zuo).0;
    let w = if da { o.da_kong_wang_deg } else { o.kong_wang_deg };
    if half_w() - off.abs() <= w {
        p.kong_wang = Some(if da { KongWang::Da } else { KongWang::Xiao });
    }
    p
}

/// 一山之内的一段（方位角，自小而大；可为负或逾 360°，取模即可）
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JieDuan {
    pub from: Real,
    pub to: Real,
    pub kind: DuanKind,
    /// 兼向之山（替卦、空亡段所偏向的邻山）；下卦段为 None
    pub jian: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DuanKind {
    /// 正向（不逾替卦限度）
    XiaGua,
    /// 兼向逾限，用替卦
    TiGua,
    /// 近同宫两山之界（亦属替卦）
    XiaoKong,
    /// 近两宫之界（出卦；亦属替卦）
    DaKong,
}

impl DuanKind {
    pub fn name(self) -> &'static str {
        match self {
            DuanKind::XiaGua => "下卦",
            DuanKind::TiGua => "替卦",
            DuanKind::XiaoKong => "小空亡",
            DuanKind::DaKong => "大空亡",
        }
    }
}

/// 一山的兼向界线：供盘上画线
#[derive(Clone, Debug, PartialEq)]
pub struct ShanJie {
    pub shan: usize,
    /// 本山中线
    pub center: Real,
    /// 出线之界：本山两侧山界（逾之即入邻山，坐山改变）
    pub shan_jie: [Real; 2],
    /// 出卦之界：本山所在宫（卦）45° 的两界（逾之即入他卦）
    pub gua_jie: [Real; 2],
    /// 自左（逆时针侧）至右五段：空亡、替卦、下卦、替卦、空亡（度数为零之段略去）
    pub duan: Vec<JieDuan>,
}

/// 某山的兼向、替卦、空亡区间与出线、出卦之界（按 [`XkOptions`] 的度数）
pub fn shan_jie(shan: usize, o: &XkOptions) -> ShanJie {
    let shan = shan % 24;
    let c = shan_center(shan);
    let (lo, hi) = (c - half_w(), c + half_w());
    let (l, r) = ((shan + 23) % 24, (shan + 1) % 24);
    let g = gong_of(shan).0;
    let kw = |nb: usize| if gong_of(nb).0 == g { (o.kong_wang_deg, DuanKind::XiaoKong) } else { (o.da_kong_wang_deg, DuanKind::DaKong) };
    let (wl, kl) = kw(l);
    let (wr, kr) = kw(r);
    let th = o.ti_threshold_deg;
    let cand = [
        JieDuan { from: lo, to: lo + wl, kind: kl, jian: Some(l) },
        JieDuan { from: lo + wl, to: c - th, kind: DuanKind::TiGua, jian: Some(l) },
        JieDuan { from: c - th, to: c + th, kind: DuanKind::XiaGua, jian: None },
        JieDuan { from: c + th, to: hi - wr, kind: DuanKind::TiGua, jian: Some(r) },
        JieDuan { from: hi - wr, to: hi, kind: kr, jian: Some(r) },
    ];
    // 限度相交时以空亡优先、下卦次之（替卦段被挤为零则略去）
    let mut duan: Vec<JieDuan> = Vec::new();
    for d in cand {
        let mut d = d;
        if d.from < lo {
            d.from = lo;
        }
        if d.to > hi {
            d.to = hi;
        }
        if d.kind == DuanKind::XiaGua {
            d.from = if d.from < lo + wl { lo + wl } else { d.from };
            d.to = if d.to > hi - wr { hi - wr } else { d.to };
        }
        if d.to > d.from {
            duan.push(d);
        }
    }
    // 本宫在 RING8 中的次序：坎宫自壬之左界 337.5° 起，每宫 45°
    let k = RING8.iter().position(|&x| x == g).unwrap();
    let mut g_lo = real_const(-22.5) + real_const(45.0) * real(k as i32);
    // 使宫界包住本山中线
    if c - g_lo >= real_const(360.0) {
        g_lo = g_lo + real_const(360.0);
    }
    while c < g_lo {
        g_lo = g_lo - real_const(360.0);
    }
    ShanJie { shan, center: c, shan_jie: [lo, hi], gua_jie: [g_lo, g_lo + real_const(45.0)], duan }
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

/// 某年是否在其运的前十年（只用于五运零正、城门之寄）
pub fn qian_shi_nian(year: i32) -> bool {
    (year - SHANG_YUAN_JIAZI).rem_euclid(20) < 10
}

/// 零神、正神所在之宫（洛书数）：返回（正神诸宫，零神诸宫）。`qian` 为是否在本运前十年（五运寄宫用）。
pub fn ling_zheng(yun: u8, qian: bool, o: &XkOptions) -> (Vec<u8>, Vec<u8>) {
    // 五运按寄法折为他运（Zhong 时为 None）
    let eff = if yun == 5 {
        match o.wu_ji {
            WuJi::KunGen => Some(if qian { 2 } else { 8 }),
            WuJi::SiLiu => Some(if qian { 4 } else { 6 }),
            WuJi::Zhong => None,
        }
    } else {
        Some(yun)
    };
    let Some(y) = eff else { return (vec![5], vec![]) };
    match o.ling_zheng {
        LingZhengRule::YunGong => (vec![y], vec![10 - y]),
        LingZhengRule::ShangXiaYuan => {
            let shang = vec![1, 2, 3, 4];
            let xia = vec![6, 7, 8, 9];
            // 寄坤（二）、寄巽（四）属上元之气；寄艮（八）、寄乾（六）属下元
            if y < 5 { (shang, xia) } else { (xia, shang) }
        }
    }
}

/// 《天玉經》四库注之读：「十二支正神屬陽，八干四維零神屬陰」（四库本影印第 23 页），即二十四山中十二支为正神
pub fn si_ku_zheng_shen(shan: usize) -> bool {
    matches!(SHAN24[shan % 24], "子" | "丑" | "寅" | "卯" | "辰" | "巳" | "午" | "未" | "申" | "酉" | "戌" | "亥")
}

/// 七星打劫
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DaJie {
    /// 劫取之三宫（向宫与之同为一四七或三六九者）
    pub gong: [u8; 3],
    /// 离宫打劫（离震乾）为 true，坎宫打劫（坎巽兑）为 false
    pub li: bool,
    /// 依 [`DaJieRule`] 判真假
    pub zhen: bool,
}

/// 七星打劫：当运山星、向星同到向宫（双星到向），且向宫属离震乾（三六九）或坎巽兑（一四七）一组，
/// 则此三宫为打劫之宫；向宫在坤艮（二五八组含中五）不能打劫（沈氏；出处未核）。
/// 三宫之向星必为父母三般（同余于三）——飞星等差，三宫洛书数同余于三，故恒成立，测试验之。
pub fn da_jie(p: &ZhaiPan, o: &XkOptions) -> Option<DaJie> {
    let (gx, _) = gong_of(p.xiang);
    let i = gx as usize - 1;
    if p.xiang_pan[i] != p.yun || p.shan_pan[i] != p.yun {
        return None;
    }
    let (gong, li) = match gx % 3 {
        0 => ([9, 3, 6], true),
        1 => ([1, 4, 7], false),
        _ => return None,
    };
    let zhen = match o.da_jie {
        DaJieRule::LiZhenKanJia => li,
        DaJieRule::JieZhen => true,
    };
    Some(DaJie { gong, li, zhen })
}

/// 入囚：宅之山盘、向盘中宫之星即其入囚之运（运行至该运，当旺之星困于中宫）。返回（山星入囚之运，向星入囚之运）；
/// 五黄入中者不论（五无本宫，诸说不一；未核）。向首见水不囚等形势之论不在此（沈氏；出处未核）。
pub fn ru_qiu(p: &ZhaiPan) -> (Option<u8>, Option<u8>) {
    let f = |m: u8| if m == 5 { None } else { Some(m) };
    (f(p.shan_in), f(p.xiang_in))
}

/// 格局（全盘）
pub fn ge_ju(p: &ZhaiPan, o: &XkOptions) -> Vec<GeJu> {
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
    if (0..9).all(|k| p.shan_pan[k] + p.xiang_pan[k] == 10) {
        add("山向合十", "各宫山星与向星合十");
    }
    if lian_zhu(p, 0..9) {
        add("連珠三般卦", "各宫运、山、向三星相连（如一二三、八九一）");
    }
    if fu_mu(p, 0..9) {
        add("父母三般卦", "各宫运、山、向三星为一四七、二五八、三六九之一");
    }
    if let Some(d) = da_jie(p, o) {
        add(
            match (d.li, d.zhen) {
                (true, true) => "離宮打劫（真）",
                (true, false) => "離宮打劫（假）",
                (false, true) => "坎宮打劫（真）",
                (false, false) => "坎宮打劫（假）",
            },
            if d.li { "七星打劫：双星到向，劫离震乾三宫" } else { "七星打劫：双星到向，劫坎巽兑三宫" },
        );
    }
    if p.xiang_in == w && w != 5 {
        add("向星入囚", "当运向星入中宫");
    }
    if p.shan_in == w && w != 5 {
        add("山星入囚", "当运山星入中宫");
    }
    v
}

fn sorted3(p: &ZhaiPan, k: usize) -> [u8; 3] {
    let mut a = [p.yun_pan[k], p.shan_pan[k], p.xiang_pan[k]];
    a.sort();
    a
}

/// 连珠：诸宫运、山、向三星为相连三数（含九一相接，如八九一）
fn lian_zhu(p: &ZhaiPan, ks: impl IntoIterator<Item = usize>) -> bool {
    ks.into_iter().all(|k| {
        let t = sorted3(p, k);
        (1..=9u8).any(|st| {
            let mut c = [st, wrap9(st as i32 + 1), wrap9(st as i32 + 2)];
            c.sort();
            c == t
        })
    })
}

/// 父母三般：诸宫运、山、向三星互异且同余于三（一四七、二五八、三六九）
fn fu_mu(p: &ZhaiPan, ks: impl IntoIterator<Item = usize>) -> bool {
    ks.into_iter().all(|k| {
        let a = sorted3(p, k);
        a[0] % 3 == a[1] % 3 && a[1] % 3 == a[2] % 3 && a[0] != a[1] && a[1] != a[2]
    })
}

/// 逐宫标记：九宫（下标 = 宫数 − 1）各列其所当之标记，供盘上标注。`qian` 为是否在本运前十年（五运零正用）。
///
/// 标记：正神、零神；正城門、副城門（注可用否）；旺山、旺向、上山、下水（坐向宫之当运星）；雙星到向（坐）；
/// 合十（全盘者九宫皆标，另标坐、向宫局部合十）；伏吟、反吟（依 [`FuFanRule`]）；三般卦（全盘者九宫皆标）；
/// 七星打劫（三宫）；入囚（中宫）。
pub fn gong_biao(p: &ZhaiPan, qian: bool, o: &XkOptions) -> [Vec<GeJu>; 9] {
    let mut m: [Vec<GeJu>; 9] = Default::default();
    let mut put = |g: u8, name: &'static str, note: &'static str| m[g as usize - 1].push(GeJu { name, note });
    let (zs, ls) = ling_zheng(p.yun, qian, o);
    for g in zs {
        put(g, "正神", "当运之宫，宜山（实）");
    }
    for g in ls {
        put(g, "零神", "失运之宫，宜水（虚）");
    }
    for (g, zheng, ok) in cheng_men(p, o) {
        put(
            g,
            if zheng { "正城門" } else { "副城門" },
            if ok { "可用：城门宫运星入中顺飞" } else { "不可用：城门宫运星入中逆飞" },
        );
    }
    let (gz, _) = gong_of(p.zuo);
    let (gx, _) = gong_of(p.xiang);
    let w = p.yun;
    let si = |g: u8| p.shan_pan[g as usize - 1];
    let xi = |g: u8| p.xiang_pan[g as usize - 1];
    if si(gz) == w {
        put(gz, "旺山", "当运山星到坐");
    }
    if xi(gz) == w {
        put(gz, "上山", "当运向星到坐");
    }
    if xi(gx) == w {
        put(gx, "旺向", "当运向星到向");
    }
    if si(gx) == w {
        put(gx, "下水", "当运山星到向");
    }
    if si(gx) == w && xi(gx) == w {
        put(gx, "雙星到向", "当运山星、向星同到向首");
    }
    if si(gz) == w && xi(gz) == w {
        put(gz, "雙星到坐", "当运山星、向星同到坐山");
    }
    // 合十：全盘者九宫皆标；否则坐、向宫局部合十另标
    let he = |a: &[u8; 9], b: &[u8; 9], k: usize| a[k] + b[k] == 10;
    let kinds: [(&[u8; 9], &[u8; 9], &'static str, &'static str, &'static str); 3] = [
        (&p.shan_pan, &p.yun_pan, "山運合十", "全盘山星与运星合十", "此宫山星与运星合十（局部）"),
        (&p.xiang_pan, &p.yun_pan, "向運合十", "全盘向星与运星合十", "此宫向星与运星合十（局部）"),
        (&p.shan_pan, &p.xiang_pan, "山向合十", "全盘山星与向星合十", "此宫山星与向星合十（局部）"),
    ];
    for (a, b, name, quan, jubu) in kinds {
        if (0..9).all(|k| he(a, b, k)) {
            for g in 1..=9 {
                put(g, name, quan);
            }
        } else {
            for g in [gz, gx] {
                if he(a, b, g as usize - 1) {
                    put(g, name, jubu);
                }
            }
        }
    }
    // 伏吟、反吟
    let luoshu = fly(5, true);
    match o.fu_fan {
        FuFanRule::YuanDanQuan | FuFanRule::YunPanQuan => {
            let r = if o.fu_fan == FuFanRule::YuanDanQuan { luoshu } else { p.yun_pan };
            for (pan, fu, fan) in [(&p.shan_pan, "山星伏吟", "山星反吟"), (&p.xiang_pan, "向星伏吟", "向星反吟")] {
                if *pan == r {
                    for g in 1..=9 {
                        put(g, fu, "全盘与所比之盘同");
                    }
                } else if (0..9).all(|k| pan[k] + r[k] == 10) {
                    for g in 1..=9 {
                        put(g, fan, "全盘与所比之盘合十");
                    }
                }
            }
        }
        FuFanRule::YuanDanGong => {
            for g in 1..=9u8 {
                if g == 5 {
                    continue;
                }
                let k = g as usize - 1;
                for (pan, fu, fan) in [(&p.shan_pan, "山星伏吟", "山星反吟"), (&p.xiang_pan, "向星伏吟", "向星反吟")] {
                    if pan[k] == luoshu[k] {
                        put(g, fu, "此宫星与洛书本宫数同（局部）");
                    } else if pan[k] + luoshu[k] == 10 {
                        put(g, fan, "此宫星与洛书本宫数合十（局部）");
                    }
                }
            }
        }
    }
    if lian_zhu(p, 0..9) {
        for g in 1..=9 {
            put(g, "連珠三般卦", "全盘运、山、向三星相连");
        }
    }
    if fu_mu(p, 0..9) {
        for g in 1..=9 {
            put(g, "父母三般卦", "全盘运、山、向三星为一四七、二五八、三六九");
        }
    }
    if let Some(d) = da_jie(p, o) {
        for g in d.gong {
            put(
                g,
                if d.zhen { "七星打劫（真）" } else { "七星打劫（假）" },
                if d.li { "離宮打劫：離震乾三宫" } else { "坎宮打劫：坎巽兌三宫" },
            );
        }
    }
    if p.xiang_in == w && w != 5 {
        put(5, "向星入囚", "当运向星入中宫");
    }
    if p.shan_in == w && w != 5 {
        put(5, "山星入囚", "当运山星入中宫");
    }
    m
}

/// 一年十二月之流月紫白盘（寅月起至丑月），`ganzhi_year` 为干支年（立春分）
pub fn month_pans(ganzhi_year: i32) -> [[u8; 9]; 12] {
    let zhi = (ganzhi_year - 4).rem_euclid(12) as u8;
    let mut r = [[0u8; 9]; 12];
    for (m, x) in r.iter_mut().enumerate() {
        *x = month_pan(zhi, m as u8);
    }
    r
}

// ───────────────────────────── 罗经三针 ─────────────────────────────
//
// 《欽定協紀辨方書》卷二「縫針三合五行」按：「地理家有三針：一曰正針，乃二十四山之正位，定向用之；一曰中針，其子位在
// 正針壬子二位之中，格龍用之；一曰縫針，其子位在子癸二位之縫，消砂納水用之。」同卷图说「内層所列為縫針，其子位在正針
// 子癸二位之間，比正針後半位」。卷三十三「羅經」：「通書以壬子之中為縫針，今謂之中針……地理家格龍用之；若定方向則用
// 正針，消砂納水則用縫針」。名目古今不一（宋《因話錄》「子午丙壬間縫針」、《靈城精義》「壬子丙午為天盤」皆指今之中针
// 位置），见 doc/folk/xuankong.md「九、罗经三针」。

/// 罗经三针
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zhen {
    /// 正针（地盘）：二十四山正位，子中线 0°
    Zheng,
    /// 中针（今称人盘）：子位在正针壬子之间，各山中线较正针逆时针偏半山（−7.5°）
    Zhong,
    /// 缝针（今称天盘）：子位在正针子癸之缝，各山中线较正针顺时针偏半山（+7.5°，「比正針後半位」）
    Feng,
}

impl Zhen {
    pub const ALL: [Zhen; 3] = [Zhen::Zheng, Zhen::Zhong, Zhen::Feng];
    pub fn name(self) -> &'static str {
        match self {
            Zhen::Zheng => "正針",
            Zhen::Zhong => "中針",
            Zhen::Feng => "縫針",
        }
    }
    /// 今人盘名（地盘、人盘、天盘；协纪不用此名，《靈城精義》之「天盤」在壬子之间，与今名相反，见文档）
    pub fn pan_name(self) -> &'static str {
        match self {
            Zhen::Zheng => "地盤",
            Zhen::Zhong => "人盤",
            Zhen::Feng => "天盤",
        }
    }
    /// 各山中线相对正针之偏（度，顺时针为正）
    pub fn offset_deg(self) -> Real {
        match self {
            Zhen::Zheng => real_const(0.0),
            Zhen::Zhong => -half_w(),
            Zhen::Feng => half_w(),
        }
    }
    /// 协纪所载用法
    pub fn yong_xie_ji(self) -> &'static str {
        match self {
            Zhen::Zheng => "定向",
            Zhen::Zhong => "格龍",
            Zhen::Feng => "消砂納水",
        }
    }
}

/// 某针某山之中线方位角（度）
pub fn zhen_shan_center(shan: usize, zhen: Zhen) -> Real {
    shan_center(shan) + zhen.offset_deg()
}

/// 方位角在某针上所落之山，与偏离该针本山中线之度数（−7.5—7.5）
pub fn shan_of_azimuth_zhen(az_deg: Real, zhen: Zhen) -> (usize, Real) {
    shan_of_azimuth(az_deg - zhen.offset_deg())
}

/// 三盘二十四山方位表：`[山][针]` = (lo, 中线, hi)，度数可负（中针子山 −15—0）
pub fn san_zhen_table() -> [[[Real; 3]; 3]; 24] {
    let mut t = [[[real_const(0.0); 3]; 3]; 24];
    for (s, row) in t.iter_mut().enumerate() {
        for (k, z) in Zhen::ALL.iter().enumerate() {
            let c = zhen_shan_center(s, *z);
            row[k] = [c - half_w(), c, c + half_w()];
        }
    }
    t
}

// ───────────────────────────── 玄空大卦（六十四卦） ─────────────────────────────
//
// 与九星飞布不同之系统。原文可结构化者：
// - 《天玉經》「江東一卦從來吉，八神四個一；江西一卦排龍位，八神四個二；南北八神共一卦，端的應無差」；四库本卷一注与
//   吳公發揮读为「寅甲卯乙辰巽巳丙共為江東卦，申庚酉辛戌乾亥壬共為江西卦，午丁未坤子癸丑艮共為一卦」（[`jiang_san_gua`]）。
// - 《天玉經外編》「四經五行」：乾丙乙子寅辰金（一龍）、坤壬辛午申戌木（二龍）、艮庚丁卯巳丑水（三龍）、巽甲癸亥酉未火（四龍）
//   （[`si_jing_long`]）。
// - 蒋大鸿《地理辨正》青囊序注批此二读皆「非」，而所倡「玄空大卦」排法不著于书（「有訣無書」）。
// - 先天六十四卦圆图之序：《易學啟蒙》（胡方平通釋卷上）「內卦為乾，自乾至泰八卦……兊八卦自履至臨，離八卦自同人至明夷，
//   震八卦自无妄至復，艮八卦自遯至謙，坎八卦自訟至師，㢲八卦自姤至升」，乾南坤北。
// 今人（沈氏、谈养吾一系）之卦运、卦数及卦在二十四山之位，非蒋氏原书所载，出处未核（[`gua_yun`]、[`gua_shu`]、[`da_gua_at`]）。

/// 先天八卦（三位，初爻最低位，阳为 1）之洛书数：乾九、兑四、离三、震八、巽二、坎七、艮六、坤一（先天配洛书；今人，未核）
pub fn xian_tian_shu(tri: u8) -> u8 {
    match tri & 7 {
        0b111 => 9, // 乾
        0b011 => 4, // 兌（初、二阳）
        0b101 => 3, // 離
        0b001 => 8, // 震
        0b110 => 2, // 巽
        0b010 => 7, // 坎
        0b100 => 6, // 艮
        _ => 1,     // 坤
    }
}

/// 六十四卦（六位，初爻最低位，与 [`super::zhouyi::name`] 同）之卦数：内卦先天洛书数（今人，未核）
pub fn gua_shu(gua: u8) -> u8 {
    xian_tian_shu(gua & 7)
}

/// 卦运（1—4、6—9）：以内外卦逐爻比，不同之爻为阳、相同为阴，得一三画卦，取其先天洛书数。即八纯卦一运，
/// 内外全反（泰否、损咸、既济未济、益恒）九运；只初爻异者（姤困旅豫小畜節賁復）八运（今人，未核）。
pub fn gua_yun(gua: u8) -> u8 {
    xian_tian_shu((gua & 7) ^ ((gua >> 3) & 7))
}

/// 先天八卦次序（乾兑离震巽坎艮坤）之三位
const XT_ORDER: [u8; 8] = [0b111, 0b011, 0b101, 0b001, 0b110, 0b010, 0b100, 0b000];

/// 先天六十四卦圆图，自北（坤、复之交）起顺时针（方位角增）之卦序。东半自复至乾：内卦震、离、兑、乾，各宫外卦坤艮坎巽震离兑乾；
/// 西半自姤至坤：内卦巽、坎、艮、坤，各宫外卦乾兑离震巽坎艮坤（《易學啟蒙》）。
pub fn yuan_tu() -> [u8; 64] {
    let mut r = [0u8; 64];
    let nei = [XT_ORDER[3], XT_ORDER[2], XT_ORDER[1], XT_ORDER[0], XT_ORDER[4], XT_ORDER[5], XT_ORDER[6], XT_ORDER[7]];
    for (o, n) in nei.iter().enumerate() {
        for j in 0..8 {
            let wai = if o < 4 { XT_ORDER[7 - j] } else { XT_ORDER[j] };
            r[o * 8 + j] = (wai << 3) | n;
        }
    }
    r
}

/// 每卦所占度数 360/64
pub fn da_gua_w() -> Real {
    real_const(5.625)
}

/// 方位角所落之卦（六位）与在卦内之度数（0—5.625）。坤复之交置于正子（0°），今人之盘，未核
pub fn da_gua_at(az_deg: Real) -> (u8, Real) {
    let full = real_const(360.0);
    let a = az_deg - (az_deg / full).floor() * full;
    let k = (a / da_gua_w()).to_i32_floor().rem_euclid(64) as usize;
    (yuan_tu()[k], a - da_gua_w() * real(k as i32))
}

/// 与方位区间 [lo, hi) 相交之卦：(卦, 卦起, 卦止)，度数与区间同一圈（可负）
pub fn da_gua_in(lo: Real, hi: Real) -> Vec<(u8, Real, Real)> {
    let w = da_gua_w();
    let t = yuan_tu();
    let mut v = Vec::new();
    let mut k = (lo / w).to_i32_floor();
    while w * real(k) < hi {
        v.push((t[k.rem_euclid(64) as usize], w * real(k), w * real(k + 1)));
        k += 1;
    }
    v
}

/// 《天玉經》江东、江西、南北三卦（四库本卷一注、吳公發揮之读）：0 江東（寅甲卯乙辰巽巳丙）、1 江西（申庚酉辛戌乾亥壬）、
/// 2 南北（午丁未坤子癸丑艮）。蒋氏批为「非」
pub fn jiang_san_gua(shan: usize) -> u8 {
    match shan % 24 {
        4..=11 => 0,
        16..=23 => 1,
        _ => 2,
    }
}

/// 《天玉經外編》四經五行：返回 (龍序 1—4, 五行)。一龍金乾丙乙子寅辰，二龍木坤壬辛午申戌，三龍水艮庚丁卯巳丑，
/// 四龍火巽甲癸亥酉未。蒋氏批为「非」
pub fn si_jing_long(shan: usize) -> (u8, &'static str) {
    match SHAN24[shan % 24] {
        "乾" | "丙" | "乙" | "子" | "寅" | "辰" => (1, "金"),
        "坤" | "壬" | "辛" | "午" | "申" | "戌" => (2, "木"),
        "艮" | "庚" | "丁" | "卯" | "巳" | "丑" => (3, "水"),
        _ => (4, "火"),
    }
}

// ───────────────────────────── 日、时紫白 ─────────────────────────────
//
// 《奇門遁甲統宗》（维基文库简体录文，未注底本）末卷「陰陽二遁分三元」注：「年、月、日、時俱分三元，年、月俱用一四七。
// 陰局則日冬至後用陽局一七四，夏至後用陰局九三六，陽順行陰逆行……即將年月干支所到宮之星，由入中宮順飛九宮」。
// 即冬至后甲子日一白入中，日进一星，六十日后甲子七赤、再六十日四绿；夏至后甲子日九紫，日退一星，次甲子三碧、六白。
// 时：协纪卷三十四「時家亦有三竒紫白，倣月例推」（月例按年支三组定正月之星），推之为子午卯酉日子时、辰戌丑未日子时、
// 寅申巳亥日子时三组；数取时辰连续（六十时一周），阳一四七、阴九六三（数值系连续推得，原文未明列，未核）。

/// 日紫白起例之甲子取法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RiBaiRule {
    /// 二至当日或其后第一个甲子日（「冬至後甲子」字面）。缺省
    HouJiaZi,
    /// 距二至最近之甲子（前后各三十日内；今人另说，未核）
    JinJiaZi,
}

/// 日、时紫白之九宫飞法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RiPanFei {
    /// 入中顺飞（統宗「由入中宮順飛九宮」）。缺省
    ShunFei,
    /// 阴遁之日、时逆飞（今人另说，未核）
    YinNi,
}

/// 日、时紫白选项
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RiShiOptions {
    pub jia_zi: RiBaiRule,
    pub fei: RiPanFei,
}

pub const DEFAULT_RI_SHI: RiShiOptions = RiShiOptions { jia_zi: RiBaiRule::HouJiaZi, fei: RiPanFei::ShunFei };

fn day_gz_of(jdn: i64) -> u8 {
    (jdn + 49).rem_euclid(60) as u8
}

/// 二至（民用日之儒略日数）后起例之甲子日
pub fn er_zhi_jia_zi(zhi_jdn: i64, rule: RiBaiRule) -> i64 {
    let after = zhi_jdn + (60 - day_gz_of(zhi_jdn) as i64) % 60;
    match rule {
        RiBaiRule::HouJiaZi => after,
        RiBaiRule::JinJiaZi => {
            if after - zhi_jdn > 30 { after - 60 } else { after }
        }
    }
}

/// 日紫白：`qi` 为二至表（民用日儒略日数, 是否冬至），升序。返回 (入中之星, 阳遁否, 起例甲子日)
pub fn ri_xing(jdn: i64, qi: &[(i64, bool)], rule: RiBaiRule) -> Option<(u8, bool, i64)> {
    let (s, yang) = qi.iter().map(|&(d, dong)| (er_zhi_jia_zi(d, rule), dong)).filter(|(s, _)| *s <= jdn).max_by_key(|x| x.0)?;
    let n = (jdn - s) as i32;
    Some((if yang { wrap9(1 + n) } else { wrap9(9 - n) }, yang, s))
}

/// 时紫白（入中之星）：`day_gz` 日干支序，`hour_zhi` 时支序（子 0），`yang` 阳遁否（随日紫白）
pub fn shi_xing(day_gz: u8, hour_zhi: u8, yang: bool) -> u8 {
    let g = (day_gz % 12 % 3) as usize;
    let h = (hour_zhi % 12) as i32;
    if yang { wrap9([1, 4, 7][g] + h) } else { wrap9([9, 6, 3][g] - h) }
}

/// 日、时紫白盘（九宫）
pub fn ri_shi_pan(star: u8, yang: bool, o: &RiShiOptions) -> [u8; 9] {
    fly(star, yang || o.fei == RiPanFei::ShunFei)
}

/// 某民用日前后之二至表（民用日儒略日数, 是否冬至），取东八区民用日（子正为界）
pub fn er_zhi_around(vsop: &crate::astronomy::ephemeris::Vsop87, jdn: i64) -> Vec<(i64, bool)> {
    use crate::astronomy::aspects::solar_term_jds_for_year_cached;
    use crate::astronomy::time::{TimePoint, TimeScale};
    let (y, _, _) = crate::calendar::gregorian::Gregorian::from_julian_day(real(jdn as i32));
    let mut v = Vec::new();
    for yy in (y - 2)..=y {
        let t = solar_term_jds_for_year_cached(vsop, yy);
        for (k, dong) in [(6usize, false), (18usize, true)] {
            let ut = TimePoint::new(TimeScale::TT, t[k]).to_scale(TimeScale::UTC).jd;
            let d = (ut + real_const(8.0) / real_const(24.0) + real_const(0.5)).to_i32_floor() as i64;
            v.push((d, dong));
        }
    }
    v.sort();
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
        let r = |x: f64| real(x);
        assert_eq!(shan_of_azimuth(r(0.0)), (idx("子"), r(0.0)));
        assert_eq!(shan_of_azimuth(r(359.0)).0, idx("子"));
        assert_eq!(shan_of_azimuth(r(180.0)).0, idx("午"));
        assert_eq!(shan_of_azimuth(r(45.0)).0, idx("艮"));
        assert_eq!(shan_of_azimuth(r(-15.0)).0, idx("壬"));
        let p = zhai_pan_azimuth(8, r(7.0), &DEFAULT_OPTIONS);
        assert!(p.ti);
        assert_eq!(p.kong_wang, Some(KongWang::Xiao)); // 子癸之间
        let p = zhai_pan_azimuth(8, r(22.0), &DEFAULT_OPTIONS);
        assert_eq!(p.kong_wang, Some(KongWang::Da)); // 癸丑之间（坎艮两宫）
        assert!(!zhai_pan_azimuth(8, r(2.0), &DEFAULT_OPTIONS).ti);
        // 大空亡另设度数：3° 时 20° 即入大空亡、而 7.0° 仍按小空亡 1.5°
        let o = XkOptions { da_kong_wang_deg: r(3.0), ..DEFAULT_OPTIONS };
        assert_eq!(zhai_pan_azimuth(8, r(20.0), &o).kong_wang, Some(KongWang::Da));
        assert_eq!(zhai_pan_azimuth(8, r(5.5), &o).kong_wang, None);
    }

    /// 兼向界线：每山 15° 依次分空亡、替卦、下卦、替卦、空亡，首尾相接，合为本山
    #[test]
    fn jie_xian() {
        for o in [DEFAULT_OPTIONS, XkOptions { ti_threshold_deg: real(4.5), ..DEFAULT_OPTIONS }] {
            for i in 0..24 {
                let j = shan_jie(i, &o);
                assert_eq!(j.duan.first().unwrap().from, j.shan_jie[0]);
                assert_eq!(j.duan.last().unwrap().to, j.shan_jie[1]);
                for w in j.duan.windows(2) {
                    assert_eq!(w[0].to, w[1].from);
                }
                assert!(j.gua_jie[0] <= j.shan_jie[0] && j.shan_jie[1] <= j.gua_jie[1]);
                // 段内一点按方位角排盘，替卦与空亡与段相符
                for d in &j.duan {
                    let mid = (d.from + d.to) / real(2.0);
                    let p = zhai_pan_azimuth(8, mid, &o);
                    assert_eq!(p.zuo, i);
                    assert_eq!(p.ti, d.kind != DuanKind::XiaGua, "{} {:?}", SHAN24[i], d.kind);
                    let kw = match d.kind {
                        DuanKind::XiaoKong => Some(KongWang::Xiao),
                        DuanKind::DaKong => Some(KongWang::Da),
                        _ => None,
                    };
                    assert_eq!(p.kong_wang, kw, "{} {:?}", SHAN24[i], d.kind);
                }
                // 天元（每宫中山）两侧皆小空亡，地元左、人元右为大空亡
                let (_, y) = gong_of(i);
                let k0 = j.duan.first().unwrap().kind;
                let k1 = j.duan.last().unwrap().kind;
                match y {
                    YuanLong::Tian => assert_eq!((k0, k1), (DuanKind::XiaoKong, DuanKind::XiaoKong)),
                    YuanLong::Di => assert_eq!((k0, k1), (DuanKind::DaKong, DuanKind::XiaoKong)),
                    YuanLong::Ren => assert_eq!((k0, k1), (DuanKind::XiaoKong, DuanKind::DaKong)),
                }
            }
        }
        let j = shan_jie(idx("子"), &DEFAULT_OPTIONS);
        let v: Vec<(f64, f64, &str)> = j.duan.iter().map(|d| (d.from.as_f64(), d.to.as_f64(), d.kind.name())).collect();
        assert_eq!(v, vec![(-7.5, -6.0, "小空亡"), (-6.0, -3.0, "替卦"), (-3.0, 3.0, "下卦"), (3.0, 6.0, "替卦"), (6.0, 7.5, "小空亡")]);
        assert_eq!((j.gua_jie[0].as_f64(), j.gua_jie[1].as_f64()), (-22.5, 22.5));
        let j = shan_jie(idx("壬"), &DEFAULT_OPTIONS);
        assert_eq!((j.gua_jie[0].as_f64(), j.gua_jie[1].as_f64()), (337.5, 382.5));
    }

    /// 零神正神：缺省当运本宫为正神、对宫为零神；五运前十年寄坤、后十年寄艮
    #[test]
    fn ling_zheng_shen() {
        let o = DEFAULT_OPTIONS;
        for y in [1u8, 2, 3, 4, 6, 7, 8, 9] {
            assert_eq!(ling_zheng(y, true, &o), (vec![y], vec![10 - y]));
        }
        assert_eq!(ling_zheng(5, true, &o), (vec![2], vec![8]));
        assert_eq!(ling_zheng(5, false, &o), (vec![8], vec![2]));
        let o2 = XkOptions { wu_ji: WuJi::SiLiu, ..o };
        assert_eq!(ling_zheng(5, true, &o2), (vec![4], vec![6]));
        let o3 = XkOptions { ling_zheng: LingZhengRule::ShangXiaYuan, ..o };
        assert_eq!(ling_zheng(8, true, &o3), (vec![6, 7, 8, 9], vec![1, 2, 3, 4]));
        assert_eq!(ling_zheng(2, true, &o3), (vec![1, 2, 3, 4], vec![6, 7, 8, 9]));
        // 1684 上元甲子：2024—2033 为九运前十年，2034—2043 后十年；五运 1744—1763
        assert!(qian_shi_nian(2024) && qian_shi_nian(2033) && !qian_shi_nian(2034) && qian_shi_nian(1744) && !qian_shi_nian(1754));
        // 四库注：十二支正神、八干四维零神
        assert_eq!((0..24).filter(|&i| si_ku_zheng_shen(i)).count(), 12);
        assert!(si_ku_zheng_shen(idx("子")) && !si_ku_zheng_shen(idx("癸")) && !si_ku_zheng_shen(idx("乾")));
    }

    /// 七星打劫：遍历九运二十四山下卦，打劫恰为双星到向且向宫不在坤艮者；三宫向星恒为父母三般
    #[test]
    fn qi_xing_da_jie() {
        for yun in 1..=9u8 {
            for zuo in 0..24 {
                for ti in [false, true] {
                    let p = zhai_pan(yun, zuo, ti, &DEFAULT_OPTIONS);
                    let (gx, _) = gong_of(p.xiang);
                    let shuang = p.shan_pan[gx as usize - 1] == yun && p.xiang_pan[gx as usize - 1] == yun;
                    let d = da_jie(&p, &DEFAULT_OPTIONS);
                    assert_eq!(d.is_some(), shuang && gx != 2 && gx != 8 && gx != 5);
                    if let Some(d) = d {
                        assert!(d.gong.contains(&gx));
                        let r: Vec<u8> = d.gong.iter().map(|&g| p.xiang_pan[g as usize - 1] % 3).collect();
                        assert!(r.iter().all(|&x| x == r[0]));
                        assert_eq!(d.zhen, d.li);
                    }
                }
            }
        }
        // 八运子山午向下卦：双星到向（离），离宫打劫，劫离震乾（通行之例，未核原书）
        let p = zhai_pan(8, idx("子"), false, &DEFAULT_OPTIONS);
        let d = da_jie(&p, &DEFAULT_OPTIONS).unwrap();
        assert_eq!((d.gong, d.li, d.zhen), ([9, 3, 6], true, true));
        assert_eq!([p.xiang_pan[8], p.xiang_pan[2], p.xiang_pan[5]], [8, 5, 2]);
        // 八运午山子向：双星到坐，无打劫
        assert!(da_jie(&zhai_pan(8, idx("午"), false, &DEFAULT_OPTIONS), &DEFAULT_OPTIONS).is_none());
    }

    /// 入囚：下卦时中宫山、向星取自运盘坐、向宫，必非当运之星，故当运不囚；八运子山午向向星三入中，至三运入囚
    #[test]
    fn ru_qiu_test() {
        for yun in 1..=9u8 {
            for zuo in 0..24 {
                let p = zhai_pan(yun, zuo, false, &DEFAULT_OPTIONS);
                assert!(p.ge_ju.iter().all(|g| !g.name.contains("入囚")));
            }
        }
        let p = zhai_pan(8, idx("子"), false, &DEFAULT_OPTIONS);
        assert_eq!(ru_qiu(&p), (Some(4), Some(3)));
    }

    /// 逐宫标记：八运子山午向下卦
    #[test]
    fn gong_biao_ba_yun_zi() {
        let p = zhai_pan(8, idx("子"), false, &DEFAULT_OPTIONS);
        let b = gong_biao(&p, true, &DEFAULT_OPTIONS);
        let names = |g: usize| b[g - 1].iter().map(|x| x.name).collect::<Vec<_>>();
        assert!(names(8).contains(&"正神"));
        assert!(names(2).contains(&"零神"));
        assert!(names(2).contains(&"副城門"));
        assert!(names(4).contains(&"正城門"));
        for n in ["旺向", "下水", "雙星到向", "七星打劫（真）"] {
            assert!(names(9).contains(&n), "{n}");
        }
        assert!(names(3).contains(&"七星打劫（真）") && names(6).contains(&"七星打劫（真）"));
        // 全盘格局与逐宫标记相应：全盘合十、三般、伏反吟者九宫皆标
        for yun in 1..=9u8 {
            for zuo in 0..24 {
                for ti in [false, true] {
                    let p = zhai_pan(yun, zuo, ti, &DEFAULT_OPTIONS);
                    let b = gong_biao(&p, true, &DEFAULT_OPTIONS);
                    for g in &p.ge_ju {
                        let quan = ["山星合十", "向星合十", "山向合十", "連珠三般卦", "父母三般卦", "山星伏吟", "向星伏吟", "山星反吟", "向星反吟"];
                        if let Some(k) = quan.iter().position(|q| *q == g.name) {
                            let n = ["山運合十", "向運合十", "山向合十", "連珠三般卦", "父母三般卦", "山星伏吟", "向星伏吟", "山星反吟", "向星反吟"][k];
                            assert!(b.iter().all(|v| v.iter().any(|x| x.name == n)), "{yun} {} {}", SHAN24[zuo], g.name);
                        }
                    }
                }
            }
        }
        // 局部伏反吟：逆飞之盘恰有一宫星到本宫（2k ≡ m+5 mod 9）
        let o = XkOptions { fu_fan: FuFanRule::YuanDanGong, ..DEFAULT_OPTIONS };
        let b = gong_biao(&p, true, &o);
        let fu = b.iter().filter(|v| v.iter().any(|x| x.name == "向星伏吟")).count();
        assert!(fu <= 1);
    }

    #[test]
    fn liu_yue() {
        // 2026 丙午年：子午卯酉年正月八白入中，逐月逆退
        let m = month_pans(2026);
        assert_eq!(m[0][4], 8);
        assert_eq!(m[1][4], 7);
        assert_eq!(m[8][4], 9); // 戌月九紫
        assert_eq!(m[11][4], 6); // 丑月六白
        // 2025 乙巳年（寅申巳亥）正月二黑；2024 甲辰年（辰戌丑未）正月五黄
        assert_eq!(month_pans(2025)[0][4], 2);
        assert_eq!(month_pans(2024)[0][4], 5);
    }

    /// 生成文档检验表（cargo test ... -- --ignored --nocapture xuankong::tests::dump）
    #[test]
    #[ignore]
    fn dump() {
        let o = DEFAULT_OPTIONS;
        println!("## 打劫、入囚（下卦）");
        for yun in 1..=9u8 {
            let mut v = vec![];
            for zuo in 0..24 {
                let p = zhai_pan(yun, zuo, false, &o);
                if let Some(d) = da_jie(&p, &o) {
                    v.push(format!("{}{}{}", SHAN24[zuo], SHAN24[p.xiang], if d.li { "離" } else { "坎" }));
                }
            }
            println!("| {yun} | {} |", v.join("、"));
        }
        println!("## 替卦打劫");
        for yun in 1..=9u8 {
            let mut v = vec![];
            for zuo in 0..24 {
                let p = zhai_pan(yun, zuo, true, &o);
                if let Some(d) = da_jie(&p, &o) {
                    v.push(format!("{}{}{}", SHAN24[zuo], SHAN24[p.xiang], if d.li { "離" } else { "坎" }));
                }
            }
            println!("| {yun} | {} |", v.join("、"));
        }
        println!("## 替卦当运入囚");
        for yun in 1..=9u8 {
            let mut v = vec![];
            for zuo in 0..24 {
                let p = zhai_pan(yun, zuo, true, &o);
                for g in &p.ge_ju {
                    if g.name.contains("入囚") {
                        v.push(format!("{}{}{}", SHAN24[zuo], SHAN24[p.xiang], g.name));
                    }
                }
            }
            println!("| {yun} | {} |", v.join("、"));
        }
        println!("## 界线");
        for i in 0..24 {
            let j = shan_jie(i, &o);
            let d: Vec<String> = j.duan.iter().map(|d| format!("{}〔{}，{}〕", d.kind.name(), d.from.as_f64(), d.to.as_f64())).collect();
            println!("| {} | {} | {}—{} | {} |", SHAN24[i], j.center.as_f64(), j.gua_jie[0].as_f64(), j.gua_jie[1].as_f64(), d.join(" "));
        }
        println!("## 全盘格局（下卦、替卦，旺山旺向等四格以外）");
        for ti in [false, true] {
            for yun in 1..=9u8 {
                let mut v = vec![];
                for zuo in 0..24 {
                    let p = zhai_pan(yun, zuo, ti, &o);
                    let g: Vec<&str> =
                        p.ge_ju.iter().map(|g| g.name).filter(|n| !["旺山旺向", "上山下水", "雙星到向", "雙星到坐"].contains(n)).collect();
                    if !g.is_empty() {
                        v.push(format!("{}{}：{}", SHAN24[zuo], SHAN24[p.xiang], g.join("、")));
                    }
                }
                println!("| {} | {yun} | {} |", if ti { "替" } else { "下" }, v.join("；"));
            }
        }
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

    #[test]
    fn san_zhen() {
        let r = |x: Real| x.as_f64();
        // 协纪：中针子位在正针壬子之间，缝针子位在子癸之缝
        assert_eq!(r(zhen_shan_center(idx("子"), Zhen::Zhong)), -7.5);
        assert_eq!(r(zhen_shan_center(idx("子"), Zhen::Feng)), 7.5);
        // 「中針與縫針差一位」：同一方位，中针所落之山恒为缝针所落之次山
        for a in 0..720 {
            let az = real(a) * real_const(0.5) + real_const(0.25);
            let (z, oz) = shan_of_azimuth_zhen(az, Zhen::Zhong);
            let (f, of) = shan_of_azimuth_zhen(az, Zhen::Feng);
            assert_eq!(z, (f + 1) % 24);
            assert!((r(oz) - r(of)).abs() < 1e-9);
        }
        assert_eq!(shan_of_azimuth_zhen(real(3), Zhen::Zhong).0, idx("癸"));
        assert_eq!(shan_of_azimuth_zhen(real(3), Zhen::Feng).0, idx("子"));
        assert_eq!(shan_of_azimuth_zhen(real(-3), Zhen::Zhong).0, idx("子"));
        let t = san_zhen_table();
        assert_eq!(r(t[idx("午")][1][1]), 172.5);
        assert_eq!(r(t[idx("午")][2][1]), 187.5);
    }

    #[test]
    fn da_gua() {
        use crate::folk::zhouyi;
        let t = yuan_tu();
        // 《易學啟蒙》：坤北乾南；东半自复至乾，西半自姤至坤
        assert_eq!(zhouyi::name(t[0]), "復");
        assert_eq!(zhouyi::name(t[31]), "乾");
        assert_eq!(zhouyi::name(t[32]), "姤");
        assert_eq!(zhouyi::name(t[63]), "坤");
        // 各宫首末（啟蒙：乾至泰、履至臨、同人至明夷、无妄至復、姤至升、訟至師、遯至謙、否至坤）
        let ends: Vec<(&str, &str)> = (0..8).map(|o| (zhouyi::name(t[o * 8]), zhouyi::name(t[o * 8 + 7]))).collect();
        for (o, (a, b)) in ends.iter().enumerate() {
            let exp = [("復", "无妄"), ("明夷", "同人"), ("臨", "履"), ("泰", "乾"), ("姤", "升"), ("訟", "師"), ("遯", "謙"), ("否", "坤")][o];
            assert!(a.contains(exp.0.chars().next().unwrap()) && b.contains(exp.1.chars().last().unwrap()), "{o}: {a} {b}");
        }
        let mut seen = [false; 64];
        for g in t {
            assert!(!seen[g as usize]);
            seen[g as usize] = true;
        }
        // 卦运：八纯卦一运，泰否九运；每运八卦，无五运
        assert_eq!((gua_yun(0), gua_yun(63), gua_yun(0b000_111), gua_yun(0b111_000)), (1, 1, 9, 9));
        // 今人转述八运卦：姤、困、旅、豫、小畜、節、賁、復
        let mut ba: Vec<&str> = (0..64u8).filter(|g| gua_yun(*g) == 8).map(zhouyi::name).collect();
        ba.sort();
        let mut exp = vec!["姤", "困", "旅", "豫", "小畜", "節", "賁", "復"];
        exp.sort();
        assert_eq!(ba, exp);
        for y in [1u8, 2, 3, 4, 6, 7, 8, 9] {
            assert_eq!((0..64u8).filter(|g| gua_yun(*g) == y).count(), 8);
        }
        // 卦数：内卦宫，自北顺时针为八、三、四、九、二、七、六、一
        for (o, s) in [8u8, 3, 4, 9, 2, 7, 6, 1].iter().enumerate() {
            assert!((0..8).all(|j| gua_shu(t[o * 8 + j]) == *s));
        }
        assert_eq!(zhouyi::name(da_gua_at(real(1)).0), "復");
        assert_eq!(zhouyi::name(da_gua_at(real(-1)).0), "坤");
        assert_eq!(da_gua_in(real_const(-7.5), real_const(7.5)).len(), 4);
        // 原文两读
        for k in 0..3 {
            assert_eq!((0..24).filter(|s| jiang_san_gua(*s) == k).count(), 8);
        }
        assert_eq!(jiang_san_gua(idx("寅")), 0);
        assert_eq!(jiang_san_gua(idx("丙")), 0);
        assert_eq!(jiang_san_gua(idx("壬")), 1);
        assert_eq!(jiang_san_gua(idx("艮")), 2);
        for k in 1..=4 {
            assert_eq!((0..24).filter(|s| si_jing_long(*s).0 == k).count(), 6);
        }
        // 地理辨正青囊序批：「子寅辰乾丙乙一龍……午申戌坤辛壬二龍……卯巳丑艮庚丁三龍……酉亥未巽癸甲四龍」
        for (ss, n) in [("子寅辰乾丙乙", 1), ("午申戌坤辛壬", 2), ("卯巳丑艮庚丁", 3), ("酉亥未巽癸甲", 4)] {
            for c in ss.chars() {
                assert_eq!(si_jing_long(idx(&c.to_string())).0, n, "{c}");
            }
        }
    }

    #[test]
    fn ri_shi_zi_bai() {
        // 时：六十时一周连续；阳顺阴逆
        for gz in 0..60u8 {
            for h in 0..12u8 {
                let n = gz as i32 * 12 + h as i32;
                assert_eq!(shi_xing(gz, h, true), wrap9(1 + n));
                assert_eq!(shi_xing(gz, h, false), wrap9(9 - n));
            }
        }
        assert_eq!((shi_xing(0, 0, true), shi_xing(1, 0, true), shi_xing(2, 0, true)), (1, 4, 7));
        assert_eq!((shi_xing(0, 0, false), shi_xing(1, 0, false), shi_xing(2, 0, false)), (9, 6, 3));
        // 日：冬至后甲子一白、次甲子七赤、再次四绿；夏至后甲子九紫、次三碧、再次六白
        let d = 2461031; // 2025-12-21 甲子，即冬至
        assert_eq!(day_gz_of(d), 0);
        let qi = [(d, true), (d + 182, false)];
        let st = |j: i64| ri_xing(j, &qi, RiBaiRule::HouJiaZi).unwrap();
        assert_eq!((st(d).0, st(d + 60).0, st(d + 120).0, st(d + 1).0), (1, 7, 4, 2));
        assert_eq!(er_zhi_jia_zi(d + 182, RiBaiRule::HouJiaZi), d + 240);
        assert_eq!(st(d + 240), (9, false, d + 240));
        assert_eq!((st(d + 300).0, st(d + 360).0, st(d + 241).0), (3, 6, 8));
        // 最近甲子：夏至后二日之前甲子
        assert_eq!(er_zhi_jia_zi(d + 182, RiBaiRule::JinJiaZi), d + 180);
        assert!(ri_xing(d - 1, &qi, RiBaiRule::HouJiaZi).is_none());
        assert_eq!(ri_shi_pan(9, false, &DEFAULT_RI_SHI), fly(9, true));
        assert_eq!(ri_shi_pan(9, false, &RiShiOptions { fei: RiPanFei::YinNi, ..DEFAULT_RI_SHI }), fly(9, false));
    }

    #[test]
    fn ri_zi_bai_real_solstice() {
        use crate::astronomy::ephemeris::load_earth_vsop87_from_repo;
        let v = load_earth_vsop87_from_repo().unwrap();
        let j = 2461042; // 2026-01-01
        let qi = er_zhi_around(&v, j);
        assert!(qi.contains(&(2461031, true))); // 2025 冬至 12-21（东八区）
        assert!(qi.contains(&(2461213, false))); // 2026 夏至 6-21
        assert_eq!(ri_xing(j, &qi, RiBaiRule::HouJiaZi), Some((3, true, 2461031)));
        // 2026-08-01：字面「夏至後甲子」在 8-18，尚属阳遁；最近甲子法则已入阴遁（6-19 起）
        let a = 2461254;
        assert!(ri_xing(a, &qi, RiBaiRule::HouJiaZi).unwrap().1);
        assert_eq!(ri_xing(a, &qi, RiBaiRule::JinJiaZi), Some((wrap9(9 - (a - 2461211) as i32), false, 2461211)));
    }
}
