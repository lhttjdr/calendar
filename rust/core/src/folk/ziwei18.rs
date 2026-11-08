//! 紫微斗数十八飞星派：《續道藏》本《紫微斗數》三卷（题陳摶；明萬曆三十五年內府刻本，國家圖書館藏，
//! 續道藏第 167—169 冊）。星系与《紫微斗數全書》的十四主星派不同：紫微、天虛、天貴、天印、天壽、天空、紅鸞、
//! 天庫、天貫（貫索）、文昌、天福、天祿十二星随生年逆布十二宫；天杖、天異、毛頭、天刃随生月；天刑、天姚随生月；
//! 天哭合太岁；另有三台、八座、龍池、鳳閣。命宫从天杖起生时，身宫从天杖起初一逆数。
//!
//! 起例出卷一「凶十八星於十二宮歌」及其后「起紫微例」「起天杖例」「起天刑例」「起天哭例」「安命例」「安身例」
//! 「起大限例」「起小運例」「三台入座」「龍池鳳閣」诸条；以刻本书影校维基文库录文（见 doc/folk/ziwei18.md）。

/// 十二支
pub const ZHI: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];

/// 紫微一系十二星（自紫微所在宫逆布，一宫一星；卷一「虛貴印壽逆相逐，空鸞庫貫文福祿」）
pub const ZI_SERIES: [&str; 12] = ["紫微", "天虛", "天貴", "天印", "天壽", "天空", "紅鸞", "天庫", "天貫", "文昌", "天福", "天祿"];

/// 十二宫（卷一宫次：一命宫、二財帛、三兄弟、四田宅、五男女、六奴仆、七妻妾、八疾厄、九遷移、十官祿、十一福德、十二相貌）
pub const PALACES: [&str; 12] = ["命宮", "財帛", "兄弟", "田宅", "男女", "奴仆", "妻妾", "疾厄", "遷移", "官祿", "福德", "相貌"];

/// 流年十二神（卷一「順排流年例」：自太岁所在宫顺排）
pub const LIU_NIAN: [&str; 12] = ["太歲", "太陽", "喪門", "太陰", "官符", "死符", "歲破", "龍德", "白虎", "福星", "吊客", "病符"];

/// 九阳星、九阴星（卷一「九陽星分禍福輕重」「九陰星分災福輕重」；天空不在十八星之数）
pub const YANG_STARS: [&str; 9] = ["紫微", "文昌", "天福", "天祿", "天印", "天壽", "天杖", "天庫", "天姚"];
pub const YIN_STARS: [&str; 9] = ["天貴", "紅鸞", "天異", "毛頭", "天虛", "天貫", "天刑", "天刃", "天哭"];

/// 十干阴阳之说（定大限、小限顺逆）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GanYinYang {
    /// 原书卷一「十乾所屬陰陽」：甲丙戊庚壬屬陰，乙丁己辛癸屬陽（与通例相反；缺省，照原书）
    YuanShu,
    /// 通例：甲丙戊庚壬为阳
    TongLi,
}

/// 十二宫排列之向（原书未明言；见文档）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PalaceDir {
    /// 自命宫逆布（星命家七政四余十二宫之通例，宫次随宫序 子→亥→戌…；缺省）
    Ni,
    /// 自命宫顺布
    Shun,
}

/// 「起大限例」「起小運例」中「申宮」之读法（刻本卷一第 31 页即如此作「申」）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShenReading {
    /// 照字作申宫（地支申；缺省，照刻本）
    ShenZhi,
    /// 读作「身宮」（申、身同音之讹；与「命宮」相对，于义较顺；未核）
    ShenGong,
}

/// 小限之法（同书两处不同）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XiaoXianRule {
    /// 卷一「起小運例」：陽男陰女從申宮逆數，陰男陽女從命宮「身數」（读作顺数），一年一宫（缺省）
    QiXiaoYun,
    /// 卷一末「運限定論」：本生本命宮起小限，陽男陰女順行，陰男陽女逆行
    YunXianDingLun,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ziwei18Options {
    pub gan_yin_yang: GanYinYang,
    pub palace_dir: PalaceDir,
    pub shen: ShenReading,
    pub xiao_xian: XiaoXianRule,
}

impl Default for Ziwei18Options {
    fn default() -> Self {
        Self { gan_yin_yang: GanYinYang::YuanShu, palace_dir: PalaceDir::Ni, shen: ShenReading::ShenZhi, xiao_xian: XiaoXianRule::QiXiaoYun }
    }
}

/// 排盘输入（与十四主星派同）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ziwei18Input {
    /// 生年干支序（0 甲子 … 59 癸亥）
    pub year_gz: u8,
    /// 农历月 1—12（闰月按所闰之月；原书不论闰）
    pub lunar_month: u8,
    pub is_leap_month: bool,
    /// 农历日 1—30
    pub lunar_day: u8,
    /// 时辰支序（0 子 … 11 亥）
    pub hour_zhi: u8,
    pub male: bool,
}

/// 一颗星及其所在宫
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Star18 {
    pub name: &'static str,
    pub zhi: u8,
}

/// 命盘
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chart18 {
    pub ming: u8,
    pub shen: u8,
    /// 十二宫：下标为地支，值为宫名
    pub palaces: [&'static str; 12],
    pub stars: Vec<Star18>,
    /// 大限：(起岁, 止岁, 宫地支)
    pub da_xian: Vec<(u8, u8, u8)>,
    /// 小限一岁之宫及顺逆（true 顺）
    pub xiao_xian_start: u8,
    pub xiao_xian_forward: bool,
    /// 流年十二神（下标地支）：以生年太岁排；流年者以当年太岁另排（[`liu_nian`]）
    pub liu_nian: [&'static str; 12],
}

impl Chart18 {
    pub fn star(&self, name: &str) -> Option<u8> {
        self.stars.iter().find(|s| s.name == name).map(|s| s.zhi)
    }
    pub fn stars_at(&self, zhi: u8) -> Vec<&'static str> {
        self.stars.iter().filter(|s| s.zhi == zhi).map(|s| s.name).collect()
    }
}

fn m12(k: i32) -> u8 {
    k.rem_euclid(12) as u8
}

/// 紫微：从未上起子，顺数至生年支（卷一「起紫微例」：寅年生，从未起子顺数至酉安紫）
pub fn zi_wei_pos(year_zhi: u8) -> u8 {
    m12(7 + year_zhi as i32)
}

/// 天杖：从子上起正月，逆数至生月（「起天杖例」）；天異、毛頭、天刃依次逆随
pub fn zhang_pos(month: u8) -> u8 {
    m12(-(month as i32 - 1))
}

/// 天刑：从酉上起正月顺数至生月（「起天刑例」）
pub fn xing_pos(month: u8) -> u8 {
    m12(9 + month as i32 - 1)
}

/// 天姚：从丑上起正月顺数至生月（「四星分宮」：天姚在丑順行；歌「刑酉姚丑皆順行，數至生月是其星」）
pub fn yao_pos(month: u8) -> u8 {
    m12(1 + month as i32 - 1)
}

/// 天哭：与生年支相合（六合：子丑、寅亥、卯戌、辰酉、巳申、午未；「起天哭例」）
pub fn ku_pos(year_zhi: u8) -> u8 {
    m12(1 - year_zhi as i32)
}

/// 命宫：于天杖之宫起生时，顺数至卯时之宫（歌「杖星宮裹起生時，順數卯處安命之」）
pub fn ming_pos(month: u8, hour_zhi: u8) -> u8 {
    m12(zhang_pos(month) as i32 + (3 - hour_zhi as i32).rem_euclid(12))
}

/// 身宫：从天杖上起初一逆数，二日半一宫（五日二宫），数至生日；不分男女皆逆。
/// 初三、初八、十三、十八、二十三、二十八六日，午时以前不过宫、未时以后过宫（「安身例」）。
pub fn shen_pos(month: u8, day: u8, hour_zhi: u8) -> u8 {
    let d = day.clamp(1, 30) as i32;
    // 每五日两宫：第 1、2 日及第 3 日上半在第 0 宫，第 3 日下半、4、5 日在第 1 宫
    let mut k = ((d - 1) * 2) / 5;
    if (d - 3) % 5 == 0 && hour_zhi >= 7 {
        k += 1;
    }
    m12(zhang_pos(month) as i32 - k)
}

/// 年干阴阳：true 为阳
pub fn gan_is_yang(year_gan: u8, rule: GanYinYang) -> bool {
    let tong_li_yang = year_gan % 2 == 0;
    match rule {
        GanYinYang::TongLi => tong_li_yang,
        GanYinYang::YuanShu => !tong_li_yang,
    }
}

/// 流年十二神：自太岁所在之支顺排
pub fn liu_nian(tai_sui_zhi: u8) -> [&'static str; 12] {
    let mut out = [""; 12];
    for (k, n) in LIU_NIAN.iter().enumerate() {
        out[m12(tai_sui_zhi as i32 + k as i32) as usize] = n;
    }
    out
}

pub fn chart(inp: &Ziwei18Input, o: &Ziwei18Options) -> Chart18 {
    let y_zhi = inp.year_gz % 12;
    let y_gan = inp.year_gz % 10;
    let m = inp.lunar_month.clamp(1, 12);
    let d = inp.lunar_day.clamp(1, 30);
    let h = inp.hour_zhi % 12;
    let mut stars = Vec::new();
    let zw = zi_wei_pos(y_zhi);
    for (k, n) in ZI_SERIES.iter().enumerate() {
        stars.push(Star18 { name: n, zhi: m12(zw as i32 - k as i32) });
    }
    let zh = zhang_pos(m);
    for (k, n) in ["天杖", "天異", "毛頭", "天刃"].iter().enumerate() {
        stars.push(Star18 { name: n, zhi: m12(zh as i32 - k as i32) });
    }
    stars.push(Star18 { name: "天刑", zhi: xing_pos(m) });
    stars.push(Star18 { name: "天姚", zhi: yao_pos(m) });
    stars.push(Star18 { name: "天哭", zhi: ku_pos(y_zhi) });
    // 三台：戌上起正月顺数至生月，起初一数至生日；八座：辰上起正月顺数至生月，起初一数至生日（「三台入座」）
    stars.push(Star18 { name: "三台", zhi: m12(10 + m as i32 - 1 + d as i32 - 1) });
    stars.push(Star18 { name: "八座", zhi: m12(4 + m as i32 - 1 + d as i32 - 1) });
    // 龍池：辰上起子时顺数至生时；鳳閣：戌上起子时逆数至生时（「龍池鳳閣」）
    stars.push(Star18 { name: "龍池", zhi: m12(4 + h as i32) });
    stars.push(Star18 { name: "鳳閣", zhi: m12(10 - h as i32) });

    let ming = ming_pos(m, h);
    let shen = shen_pos(m, d, h);
    let mut palaces = [""; 12];
    for (k, n) in PALACES.iter().enumerate() {
        let z = match o.palace_dir {
            PalaceDir::Ni => m12(ming as i32 - k as i32),
            PalaceDir::Shun => m12(ming as i32 + k as i32),
        };
        palaces[z as usize] = n;
    }
    // 大限（「起大限例」）：陽男陰女從命宮順數；陰男陽女從申宮逆數；十年一宫。起岁原书未言，取一岁起
    let yang = gan_is_yang(y_gan, o.gan_yin_yang);
    let fwd = yang == inp.male;
    let shen_gong = match o.shen {
        ShenReading::ShenZhi => 8,
        ShenReading::ShenGong => shen,
    };
    let da_xian = (0..12)
        .map(|k| {
            let z = if fwd { m12(ming as i32 + k) } else { m12(shen_gong as i32 - k) };
            ((k * 10 + 1) as u8, (k * 10 + 10) as u8, z)
        })
        .collect();
    let (xiao_xian_start, xiao_xian_forward) = match o.xiao_xian {
        XiaoXianRule::QiXiaoYun => {
            if fwd {
                (shen_gong, false)
            } else {
                (ming, true)
            }
        }
        XiaoXianRule::YunXianDingLun => (ming, fwd),
    };
    Chart18 { ming, shen, palaces, stars, da_xian, xiao_xian_start, xiao_xian_forward, liu_nian: liu_nian(y_zhi) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn z(s: &str) -> u8 {
        ZHI.iter().position(|x| *x == s).unwrap() as u8
    }

    /// 卷一「起紫微例」：寅年生，从未起子顺数至酉安紫，逆布虚、贵、印
    #[test]
    fn zi_wei_example() {
        let c = chart(&Ziwei18Input { year_gz: 2, lunar_month: 1, is_leap_month: false, lunar_day: 1, hour_zhi: 0, male: true }, &Ziwei18Options::default());
        assert_eq!(c.star("紫微"), Some(z("酉")));
        assert_eq!(c.star("天虛"), Some(z("申")));
        assert_eq!(c.star("天貴"), Some(z("未")));
        assert_eq!(c.star("天印"), Some(z("午")));
        // 十二星占满十二宫
        let mut seen: Vec<u8> = ZI_SERIES.iter().map(|n| c.star(n).unwrap()).collect();
        seen.sort();
        assert_eq!(seen, (0..12).collect::<Vec<_>>());
    }

    /// 卷一「起天杖例」：正月生人，子上安杖，亥上安異，戌上安毛，酉上安刃
    #[test]
    fn zhang_example() {
        let c = chart(&Ziwei18Input { year_gz: 0, lunar_month: 1, is_leap_month: false, lunar_day: 1, hour_zhi: 0, male: true }, &Ziwei18Options::default());
        assert_eq!(c.star("天杖"), Some(z("子")));
        assert_eq!(c.star("天異"), Some(z("亥")));
        assert_eq!(c.star("毛頭"), Some(z("戌")));
        assert_eq!(c.star("天刃"), Some(z("酉")));
        // 天刑酉起、天姚丑起，正月在本位
        assert_eq!(c.star("天刑"), Some(z("酉")));
        assert_eq!(c.star("天姚"), Some(z("丑")));
    }

    /// 「起天哭例」：子年生人，丑上安哭
    #[test]
    fn ku_example() {
        assert_eq!(ku_pos(0), z("丑"));
        for y in 0..12u8 {
            // 六合：两支之和 ≡ 1 (mod 12)
            assert_eq!((ku_pos(y) + y) % 12, 1);
        }
    }

    /// 「安命例」刻本（卷一第 30 页）：「從子上起申，順數至未逢卯，便是命宮」——天杖在子（正月）、申时生，命在未
    #[test]
    fn ming_rule() {
        assert_eq!(ming_pos(1, z("申")), z("未"));
        assert_eq!(ming_pos(1, z("寅")), z("丑"));
        assert_eq!(ming_pos(1, z("卯")), z("子"));
        // 卯时生，命即在天杖之宫
        for m in 1..=12 {
            assert_eq!(ming_pos(m, 3), zhang_pos(m));
        }
    }

    /// 「安身例」：杖上起初一逆数，二日半一宫；初三等六日午时以前不过宫、未时过宫
    #[test]
    fn shen_rule() {
        let zh = zhang_pos(1);
        assert_eq!(shen_pos(1, 1, 0), zh);
        assert_eq!(shen_pos(1, 2, 11), zh);
        assert_eq!(shen_pos(1, 3, 6), zh); // 初三午时不过宫
        assert_eq!(shen_pos(1, 3, 7), m12(zh as i32 - 1)); // 初三未时过宫
        assert_eq!(shen_pos(1, 5, 0), m12(zh as i32 - 1));
        assert_eq!(shen_pos(1, 6, 0), m12(zh as i32 - 2)); // 五日二宫
        assert_eq!(shen_pos(1, 8, 6), m12(zh as i32 - 2));
        assert_eq!(shen_pos(1, 8, 7), m12(zh as i32 - 3));
        assert_eq!(shen_pos(1, 30, 0), m12(zh as i32 - 11));
    }

    /// 大限、小限顺逆：原书十干阴阳与通例相反（甲为阴）
    #[test]
    fn xian_rules() {
        // 甲子年（原书甲属阴）男命：阴男，大限自申宫逆数
        let inp = Ziwei18Input { year_gz: 0, lunar_month: 1, is_leap_month: false, lunar_day: 1, hour_zhi: 3, male: true };
        let c = chart(&inp, &Ziwei18Options::default());
        assert_eq!(c.ming, z("子"));
        assert_eq!(&c.da_xian[..2], &[(1, 10, z("申")), (11, 20, z("未"))]);
        assert_eq!((c.xiao_xian_start, c.xiao_xian_forward), (z("子"), true));
        // 按通例甲为阳：阳男，大限自命宫顺数；小限自申宫逆数
        let o = Ziwei18Options { gan_yin_yang: GanYinYang::TongLi, ..Default::default() };
        let c = chart(&inp, &o);
        assert_eq!(&c.da_xian[..2], &[(1, 10, z("子")), (11, 20, z("丑"))]);
        assert_eq!((c.xiao_xian_start, c.xiao_xian_forward), (z("申"), false));
        // 「申」读作身宫
        let o = Ziwei18Options { shen: ShenReading::ShenGong, ..Default::default() };
        let c = chart(&inp, &o);
        assert_eq!(c.da_xian[0].2, c.shen);
        // 运限定论：本命宫起小限
        let o = Ziwei18Options { xiao_xian: XiaoXianRule::YunXianDingLun, ..Default::default() };
        assert_eq!(chart(&inp, &o).xiao_xian_start, z("子"));
    }

    #[test]
    fn liu_nian_order() {
        let l = liu_nian(z("寅"));
        assert_eq!(l[z("寅") as usize], "太歲");
        assert_eq!(l[z("卯") as usize], "太陽");
        assert_eq!(l[z("申") as usize], "歲破");
    }

    /// 十八星（九阳九阴）恰为紫系十一星（除天空）加杖異毛刃刑姚哭
    #[test]
    fn eighteen() {
        let mut all: Vec<&str> = YANG_STARS.iter().chain(YIN_STARS.iter()).copied().collect();
        all.sort();
        let mut want: Vec<&str> = ZI_SERIES.iter().copied().filter(|n| *n != "天空").chain(["天杖", "天異", "毛頭", "天刃", "天刑", "天姚", "天哭"]).collect();
        want.sort();
        assert_eq!(all, want);
    }
}
