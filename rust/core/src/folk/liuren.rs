//! 大六壬：月将与起课（天地盘、四课、三传）。
//!
//! 依《六壬大全》（四库本，明，不著撰人；维基文库录文存 vendor/folk/wikisource/六壬大全/）：
//! - 月将：卷二「十二神釋」各条「某中氣後日纒某次，某月將」（如「登明亥，雨水後日纒娵訾，正月將」），即太阳过宫、按中气换将。
//! - 起课：卷一「入手法」十干寄宫与九法（賊尅、比用、渉害、遙尅、昴星、别責、八專、伏吟、返吟），细则依卷五各课「訂訛」。
//! - 天乙贵人与十二天将：贵人起例三说并列（[`GuiRule`]：曹震圭俗例；协纪卷七引《考原》定法；书中课例与明文实用之表），
//!   昼夜分界（卯至申、卯至酉、当地日出日入 [`is_zhou_at`]），顺逆以贵人所临地盘定（亥至辰顺、巳至戌逆），临辰戌出狱二法（[`ChuYu`]）。
//!   见 doc/folk/liuren.md「天将」。
//! - 遥克俱比俱不比、渉害异说（尉山人、比用格）见 [`QiKeOpts`]。
//! - 盘面标注（旬空、遁干、驿马丁马天马、日德日禄羊刃日鬼、本命行年）见 [`shen_sha`]；课体（九宗门、课名、常见课格）见 [`ke_ti`]。

pub mod ke_ti;
pub mod shen_sha;

use crate::astronomy::aspects::solar_term_jds_for_year_cached;
use crate::astronomy::ephemeris::Vsop87;
use crate::calendar::gregorian::Gregorian;
use crate::math::real::Real;

pub const ZHI: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];
pub const GAN: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
/// 十二神名（月将名）：子神后、丑大吉……亥登明（卷二）
pub const SHEN: [&str; 12] = ["神后", "大吉", "功曹", "太冲", "天罡", "太乙", "勝光", "小吉", "傳送", "從魁", "河魁", "登明"];

/// 五行：0 木 1 火 2 土 3 金 4 水
const ZHI_WX: [u8; 12] = [4, 2, 0, 0, 2, 1, 1, 2, 3, 3, 2, 4];
const GAN_WX: [u8; 10] = [0, 0, 1, 1, 2, 2, 3, 3, 4, 4];
/// 某行所克：木克土、火克金、土克水、金克木、水克火
const KE: [u8; 5] = [2, 3, 4, 0, 1];
/// 十干寄宫（卷一「十干寄宫」）：甲寅 乙辰 丙戊巳 丁己未 庚申 辛戌 壬亥 癸丑
pub const GAN_JI: [u8; 10] = [2, 4, 5, 7, 5, 7, 8, 10, 11, 1];

fn ke(a: u8, b: u8) -> bool {
    KE[a as usize] == b
}
fn m12(x: i32) -> u8 {
    x.rem_euclid(12) as u8
}

/// 换将之法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JiangRule {
    /// 按中气换将（太阳过宫）：雨水后亥将……大寒后子将。《六壬大全》卷二。缺省
    ZhongQi,
    /// 按节换将（以月建定将：寅月亥将、卯月戌将……）：今人异说，原典未见，未核
    Jie,
}

/// 某时刻（JD，TT）的月将支序。按所取节气的定气时刻换将，精确到时刻。
pub fn yue_jiang(vsop: &Vsop87, jd_tt: Real, rule: JiangRule) -> u8 {
    let (y, _, _) = Gregorian::from_julian_day(jd_tt);
    // 节气索引 0 春分 … 23 驚蟄；中气为偶数，节为奇数
    let mut last: Option<(Real, usize)> = None;
    for yy in (y - 1)..=(y + 1) {
        for (k, t) in solar_term_jds_for_year_cached(vsop, yy).into_iter().enumerate() {
            let want = match rule {
                JiangRule::ZhongQi => k % 2 == 0,
                JiangRule::Jie => k % 2 == 1,
            };
            if want && t <= jd_tt && last.map_or(true, |(lt, _)| t > lt) {
                last = Some((t, k));
            }
        }
    }
    let (_, k) = last.expect("节气");
    match rule {
        // 春分后戌将：将 = 10 − k/2
        JiangRule::ZhongQi => m12(10 - (k as i32) / 2),
        // 清明（k=1）属辰月→酉将……：节 k 所起之月建 = (k+3)/2 + 2，将与月建六合：将 = 13 − 月建
        JiangRule::Jie => {
            let jian = ((k as i32 + 3) / 2 + 2).rem_euclid(12);
            m12(13 - jian)
        }
    }
}

/// 卷十一「太陽過宫」所列各宫宫首宿度（地支序 子…亥；宿下标角 0，入宿古度）：「立春前九日有竒日躔子分女初度……驚蟄前十日日躔亥分
/// 在危十二度……清明前八日日躔戌分在奎初度……立夏前六日日躔酉分在胃三度……芒種前五日日躔申分在畢六度……小暑前五日日躔未分在井八度……
/// 立秋前六日日躔午分在栁三度……白露前五日日躔已分在張十四度……寒露前三日日躔辰分在軫九度……立冬前一日有竒日躔夘分在氐一度……
/// 大雪前三日日躔寅分在尾二度……小寒前六日日躔丑分在斗三度」。同卷歌「角亢氐一總在辰　氐二房心尾夘存……」以各宫首度为氐二、尾三、
/// 斗四、女二……，较此表多一度（歌言宫内第一整度，表言宫界），从表
pub const GONG_JIE_DAQUAN: [(usize, f64); 12] =
    [(9, 0.0), (7, 3.0), (5, 2.0), (2, 1.0), (27, 9.0), (25, 14.0), (23, 3.0), (21, 8.0), (18, 6.0), (16, 3.0), (14, 0.0), (11, 12.0)];

/// 古法黄道宿度之界：二十八宿距星赤经圈与黄道交点之黄经（°，tan λ = tan α ／ cos ε）。古以赤道宿度经黄赤道差化为黄道宿度，
/// 宿界即距星赤经圈所过之黄道点，非距星本身之黄经（二者于斗、女、危等远黄道之宿差至数度）。卷十一宫界日数以此最合（见文档）
pub fn mansion_gu_huang_dao(frame: &crate::astronomy::sky_culture::chinese::MansionFrame, t: crate::astronomy::time::TimePoint) -> [Real; 28] {
    use crate::astronomy::frame::nutation::{nutation_for_model, NutationModel};
    use crate::astronomy::frame::precession::mean_obliquity;
    use crate::astronomy::time::TimeScale;
    use crate::math::real::{real, RealOps};
    let c = (t.to_scale(TimeScale::TT).jd - real(2451545.0)) / real(36525.0);
    let eps = mean_obliquity(c).rad() + nutation_for_model(c, NutationModel::IAU2000A).1.rad();
    let r2d = real(180.0 / core::f64::consts::PI);
    core::array::from_fn(|i| {
        let a = frame.ra[i];
        let l = a.sin().atan2(a.cos() * eps.cos()) * r2d;
        if l < real(0.0) { l + real(360.0) } else { l }
    })
}

/// 恒星过宫之月将：太阳视黄经 `sun_lon`（°）按卷十一宿度宫界（[`GONG_JIE_DAQUAN`]）所在之宫即月将。`mansion_lons` 二十八宿宿界黄经
/// （古法黄道宿度 [`mansion_gu_huang_dao`]；亦可传距星黄经 [`crate::folk::qizheng::mansion_longitudes`]）。卷二「以太陽纒度過宫而言則登眀為首逆布，逆布者謂之月将」，卷十一以宿度列太阳过宫，
/// 二者相合即此说；卷二各神注又以中气言之（「雨水後日纒娵訾」），即 [`JiangRule::ZhongQi`]。二说并存，缺省中气
pub fn yue_jiang_xiu(sun_lon: Real, mansion_lons: &[Real; 28]) -> u8 {
    crate::folk::qizheng::gong_of_xiu(mansion_lons, sun_lon, &GONG_JIE_DAQUAN).0
}

/// 天盘：地盘某支之上所临的天盘神（月将加占时）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TianPan {
    pub jiang: u8,
    pub shi: u8,
}

impl TianPan {
    /// 地盘 d 上的天盘神
    pub fn above(&self, d: u8) -> u8 {
        m12(self.jiang as i32 + d as i32 - self.shi as i32)
    }
    /// 天盘神 s 所临的地盘
    pub fn below(&self, s: u8) -> u8 {
        m12(s as i32 - self.jiang as i32 + self.shi as i32)
    }
}

/// 一课：下（课1为日干，余为支）与上神
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ke {
    /// 下之五行
    pub xia_wx: u8,
    /// 下所在地盘支（课1为日干寄宫）
    pub xia_zhi: u8,
    pub shang: u8,
}

impl Ke {
    /// 下贼上
    fn zei(&self) -> bool {
        ke(self.xia_wx, ZHI_WX[self.shang as usize])
    }
    /// 上克下
    fn ke_xia(&self) -> bool {
        ke(ZHI_WX[self.shang as usize], self.xia_wx)
    }
}

/// 取三传之法（卷一九法；渉害细分依卷五訂訛）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fa {
    /// 一下贼上
    ChongShen,
    /// 一上克下
    YuanShou,
    /// 比用（知一）
    ZhiYi,
    /// 渉害：受（所）克深者
    SheHai,
    /// 渉害相等，取孟上神
    JianJi,
    /// 渉害相等无孟，取仲上神
    ChaWei,
    /// 渉害复等：刚日干上、柔日支上先见者
    ZhuiXia,
    /// 遥克：神克日
    HaoShi,
    /// 遥克：日克神
    DanShe,
    /// 昴星：刚日虎视、柔日冬蛇掩目
    MaoXing,
    BieZe,
    BaZhuan,
    FuYin,
    FanYin,
    /// 返吟无克
    JingLan,
    /// 渉害之比用格（卷五渉害課「用神畏日干所勝以比和者為用」）：只在 [`SheHaiRule::BiYongGe`] 下出现
    BiYongGe,
}

impl Fa {
    pub fn name(&self) -> &'static str {
        match self {
            Fa::ChongShen => "重審",
            Fa::YuanShou => "元首",
            Fa::ZhiYi => "知一",
            Fa::SheHai => "渉害",
            Fa::JianJi => "見機",
            Fa::ChaWei => "察微",
            Fa::ZhuiXia => "綴瑕",
            Fa::HaoShi => "蒿矢",
            Fa::DanShe => "彈射",
            Fa::MaoXing => "昴星",
            Fa::BieZe => "别責",
            Fa::BaZhuan => "八專",
            Fa::FuYin => "伏吟",
            Fa::FanYin => "返吟",
            Fa::JingLan => "井欄",
            Fa::BiYongGe => "比用格",
        }
    }
}

/// 起课结果
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeShi {
    pub pan: TianPan,
    /// 四课（一课干上、二课干阴、三课支上、四课支阴）
    pub si_ke: [Ke; 4],
    /// 初、中、末传
    pub san_chuan: [u8; 3],
    pub fa: Fa,
}

/// 三刑（卷一伏吟法「迤邐刑之」）：子刑卯、卯刑子、寅刑巳、巳刑申、申刑寅、丑刑戌、戌刑未、未刑丑；辰午酉亥自刑
pub fn xing(z: u8) -> u8 {
    [3, 10, 5, 0, 4, 8, 6, 1, 2, 9, 7, 11][z as usize]
}
fn zi_xing(z: u8) -> bool {
    xing(z) == z
}

/// 驿马：申子辰马在寅、寅午戌在申、巳酉丑在亥、亥卯未在巳
fn yi_ma(z: u8) -> u8 {
    [2, 11, 8, 5][(z % 4) as usize]
}

/// 渉害深浅：上神 s 自所临地盘 d 顺行归本家，所经（不计起止）各支及寄干之克数。
/// 下贼上计受克（克上神者），上克下计所克（上神所克者）。卷五渉害「訂訛」：「從地盤歴数歸本家受尅深者」，
/// 例：亥加丑歷辰戊未己戌五重；午加庚（申）歷酉辛二重。
fn she_hai_depth(s: u8, d: u8, zei: bool) -> u32 {
    let sw = ZHI_WX[s as usize];
    let hit = |w: u8| if zei { ke(w, sw) } else { ke(sw, w) };
    let mut n = 0;
    let mut p = m12(d as i32 + 1);
    while p != s {
        if hit(ZHI_WX[p as usize]) {
            n += 1;
        }
        for g in 0..10u8 {
            if GAN_JI[g as usize] == p && hit(GAN_WX[g as usize]) {
                n += 1;
            }
        }
        p = m12(p as i32 + 1);
    }
    n
}

/// 遥克二神以上俱比或俱不比时之取法（卷一只言「擇與日干比者用」）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YaoTie {
    /// 卷五遥克課訂訛「遥克中克多論比渉害論深以後凢尅多者皆如此論」：比者仍多（或俱不比）则论渉害深浅，
    /// 深浅相等依見機、察微、綴瑕之例。神克日（蒿矢）计神所克、日克神（弹射）计神受克。缺省
    SheHai,
    /// 卷五遥克課引觀月經「俱比俱不比先舉用為真」：四课先见者（二、三、四课次序）
    XianJu,
}

/// 渉害之法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SheHaiRule {
    /// 卷五渉害訂訛：以地盘历数归本家受克深者；相等取临孟、临仲（以所临地盘论），再等刚干柔支先见。缺省
    DingE,
    /// 卷五渉害課首注「尉山人云上尅下取天盤孟仲為驗」：上克下者深浅相等时，孟仲以天盘上神本身论（下贼上仍以地盘）
    WeiShanRen,
    /// 卷五渉害課「凢渉害課用神畏日干所勝以比和者為用曰比用格」：所举诸例（戊子日午卯子舍寅鬼取午、甲戌日舍辰财取子等）
    /// 皆舍克日、受日克者而取生日或与日同类者；此类只一个即用之（[`Fa::BiYongGe`]），多个则在其中论深浅，无则照常
    BiYongGe,
}

/// 起课诸选项
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QiKeOpts {
    pub yao: YaoTie,
    pub she_hai: SheHaiRule,
}

impl Default for QiKeOpts {
    fn default() -> Self {
        QiKeOpts { yao: YaoTie::SheHai, she_hai: SheHaiRule::DingE }
    }
}

/// 起课：日干支序（0 甲子 … 59 癸亥）、月将支序、占时支序。诸选项取缺省（[`QiKeOpts::default`]）
pub fn qi_ke(day_gz: u8, jiang: u8, shi: u8) -> KeShi {
    qi_ke_with(day_gz, jiang, shi, QiKeOpts::default())
}

/// 五行 a 生 b
fn sheng(a: u8, b: u8) -> bool {
    (a + 1) % 5 == b
}

/// 起课（可选遥克、渉害异说）
pub fn qi_ke_with(day_gz: u8, jiang: u8, shi: u8, opts: QiKeOpts) -> KeShi {
    let g = day_gz % 10;
    let zhi = day_gz % 12;
    let gang = g % 2 == 0; // 刚日（阳干）
    let gw = GAN_WX[g as usize];
    let pan = TianPan { jiang, shi };
    let ji = GAN_JI[g as usize];
    let k1 = Ke { xia_wx: GAN_WX[g as usize], xia_zhi: ji, shang: pan.above(ji) };
    let k2 = Ke { xia_wx: ZHI_WX[k1.shang as usize], xia_zhi: k1.shang, shang: pan.above(k1.shang) };
    let k3 = Ke { xia_wx: ZHI_WX[zhi as usize], xia_zhi: zhi, shang: pan.above(zhi) };
    let k4 = Ke { xia_wx: ZHI_WX[k3.shang as usize], xia_zhi: k3.shang, shang: pan.above(k3.shang) };
    let si_ke = [k1, k2, k3, k4];
    let gan_shang = k1.shang;
    let zhi_shang = k3.shang;
    let chain = |c: u8| [c, pan.above(c), pan.above(pan.above(c))];
    let done = |sc: [u8; 3], fa: Fa| KeShi { pan, si_ke, san_chuan: sc, fa };
    let fu_yin = jiang == shi;
    let fan_yin = m12(jiang as i32 - shi as i32) == 6;

    // 伏吟（卷一）：有克还为用；无克刚干柔辰；中末迤逦刑之。初自刑则「次傳顛倒日辰併」：初取干上者中取支上，
    // 初取支上者中取干上（注「陽日用辰隂日用日」即无克时之顛倒）；中又自刑则末取冲
    if fu_yin {
        let chu = si_ke.iter().find(|k| k.zei() || k.ke_xia()).map(|k| k.shang).unwrap_or(if gang { gan_shang } else { zhi_shang });
        let zhong = if zi_xing(chu) { if chu == gan_shang { zhi_shang } else { gan_shang } } else { xing(chu) };
        let mo = if zi_xing(zhong) { m12(zhong as i32 + 6) } else { xing(zhong) };
        return done([chu, zhong, mo], Fa::FuYin);
    }

    // 贼克 → 比用 → 渉害
    let pick = |ks: Vec<&Ke>, zei: bool| -> Option<(u8, Fa)> {
        let mut cand: Vec<&Ke> = Vec::new();
        for k in ks {
            if !cand.iter().any(|c| c.shang == k.shang) {
                cand.push(k);
            }
        }
        match cand.len() {
            0 => None,
            1 => Some((cand[0].shang, if zei { Fa::ChongShen } else { Fa::YuanShou })),
            _ => {
                let bi: Vec<&Ke> = cand.iter().copied().filter(|k| (k.shang % 2 == 0) == gang).collect();
                if bi.len() == 1 {
                    return Some((bi[0].shang, Fa::ZhiYi));
                }
                let mut pool = if bi.len() > 1 { bi } else { cand };
                if opts.she_hai == SheHaiRule::BiYongGe {
                    // 比用格：去日干所克者，取生日或同类者
                    let he: Vec<&Ke> = pool.iter().copied().filter(|k| {
                        let w = ZHI_WX[k.shang as usize];
                        w == gw || sheng(w, gw)
                    }).collect();
                    if he.len() == 1 && pool.len() > 1 {
                        return Some((he[0].shang, Fa::BiYongGe));
                    }
                    if he.len() > 1 {
                        pool = he;
                    }
                }
                let depth = |k: &Ke| she_hai_depth(k.shang, k.xia_zhi, zei);
                let max = pool.iter().map(|k| depth(k)).max().unwrap();
                let deep: Vec<&Ke> = pool.into_iter().filter(|k| depth(k) == max).collect();
                if deep.len() == 1 {
                    return Some((deep[0].shang, Fa::SheHai));
                }
                // 孟仲之位：缺省以上神所临地盘论；尉山人说上克下以天盘上神本身论
                let wei = opts.she_hai == SheHaiRule::WeiShanRen && !zei;
                for (cls, fa) in [(2u8, Fa::JianJi), (0u8, Fa::ChaWei)] {
                    // 孟（寅申巳亥）、仲（子午卯酉）
                    let on: Vec<&&Ke> = deep.iter().filter(|k| (if wei { k.shang } else { k.xia_zhi }) % 3 == cls).collect();
                    if on.len() == 1 {
                        return Some((on[0].shang, fa));
                    }
                }
                // 复等：刚日从干上、柔日从支上先见者
                let order: [usize; 4] = if gang { [0, 1, 2, 3] } else { [2, 3, 0, 1] };
                let first = order.iter().map(|&i| si_ke[i].shang).find(|s| deep.iter().any(|k| k.shang == *s)).unwrap();
                Some((first, Fa::ZhuiXia))
            }
        }
    };
    let zei: Vec<&Ke> = si_ke.iter().filter(|k| k.zei()).collect();
    let kx: Vec<&Ke> = si_ke.iter().filter(|k| k.ke_xia()).collect();
    let r = if !zei.is_empty() { pick(zei, true) } else { pick(kx, false) };
    if let Some((chu, fa)) = r {
        return done(chain(chu), if fan_yin { Fa::FanYin } else { fa });
    }

    // 以下四课无克
    if fan_yin {
        // 井栏（卷一返吟、卷五）：初取日支驿马，中支上，末干上
        return done([yi_ma(zhi), zhi_shang, gan_shang], Fa::JingLan);
    }
    let mut distinct: Vec<(u8, u8)> = Vec::new();
    for k in &si_ke {
        let key = (k.xia_zhi, k.shang);
        if !distinct.contains(&key) {
            distinct.push(key);
        }
    }
    // 八专（两课）：论克不论遥。刚日干阳神连本位顺数三，柔日支阴神连本位逆数三；中末并干上
    if distinct.len() <= 2 {
        let chu = if gang { m12(gan_shang as i32 + 2) } else { m12(k4.shang as i32 - 2) };
        return done([chu, gan_shang, gan_shang], Fa::BaZhuan);
    }
    // 遥克：先神克日（蒿矢），无则日克神（弹射）；多者取与日比者，仍多或俱不比依 `opts.yao`
    // shen_ke_ri：神克日（渉害计神所克，同上克下）；否则日克神（计神受克，同下贼上）
    let yao = |f: &dyn Fn(u8) -> bool, shen_ke_ri: bool| -> Option<u8> {
        let mut c: Vec<&Ke> = Vec::new();
        for k in &si_ke[1..] {
            if f(k.shang) && !c.iter().any(|x| x.shang == k.shang) {
                c.push(k);
            }
        }
        match c.len() {
            0 => None,
            1 => Some(c[0].shang),
            _ => {
                let bi: Vec<&Ke> = c.iter().copied().filter(|k| (k.shang % 2 == 0) == gang).collect();
                if bi.len() == 1 {
                    return Some(bi[0].shang);
                }
                let pool = if bi.len() > 1 { bi } else { c };
                match opts.yao {
                    YaoTie::XianJu => Some(pool[0].shang),
                    YaoTie::SheHai => {
                        let depth = |k: &Ke| she_hai_depth(k.shang, k.xia_zhi, !shen_ke_ri);
                        let max = pool.iter().map(|k| depth(k)).max().unwrap();
                        let deep: Vec<&Ke> = pool.into_iter().filter(|k| depth(k) == max).collect();
                        if deep.len() == 1 {
                            return Some(deep[0].shang);
                        }
                        for cls in [2u8, 0u8] {
                            let on: Vec<&&Ke> = deep.iter().filter(|k| k.xia_zhi % 3 == cls).collect();
                            if on.len() == 1 {
                                return Some(on[0].shang);
                            }
                        }
                        let order: [usize; 4] = if gang { [0, 1, 2, 3] } else { [2, 3, 0, 1] };
                        order.iter().map(|&i| si_ke[i].shang).find(|s| deep.iter().any(|k| k.shang == *s))
                    }
                }
            }
        }
    };
    if let Some(s) = yao(&|s| ke(ZHI_WX[s as usize], gw), true) {
        return done(chain(s), Fa::HaoShi);
    }
    if let Some(s) = yao(&|s| ke(gw, ZHI_WX[s as usize]), false) {
        return done(chain(s), Fa::DanShe);
    }
    // 别责（三课）：刚日干合（五合之干）寄宫上神，柔日支前三合；中末并干上
    if distinct.len() == 3 {
        let chu = if gang { pan.above(GAN_JI[((g + 5) % 10) as usize]) } else { m12(zhi as i32 + 4) };
        return done([chu, gan_shang, gan_shang], Fa::BieZe);
    }
    // 昴星：刚日取地盘酉上神，中支上末干上；柔日取天盘酉所临地盘，中干上末支上
    if gang {
        done([pan.above(9), zhi_shang, gan_shang], Fa::MaoXing)
    } else {
        done([pan.below(9), gan_shang, zhi_shang], Fa::MaoXing)
    }
}

/// 某时刻起课：按 [`yue_jiang`] 取月将；`day_gz` 与 `shi`（时支）由调用方按所用日界、时制给出
pub fn qi_ke_at(vsop: &Vsop87, jd_tt: Real, day_gz: u8, shi: u8, rule: JiangRule) -> KeShi {
    qi_ke(day_gz, yue_jiang(vsop, jd_tt, rule), shi)
}

/// 十二天将，自贵人起依次：前一螣蛇、前二朱雀、前三六合、前四勾陈、前五青龙、前六天空、后五白虎、后四太常、
/// 后三玄武、后二太阴、后一天后（卷二「天將總論」：前有五位一蛇二雀三合四勾五龍，後有五位一后二隂三𤣥四常五虎，天空在貴人對冲）
pub const TIAN_JIANG: [&str; 12] = ["貴人", "螣蛇", "朱雀", "六合", "勾陳", "青龍", "天空", "白虎", "太常", "玄武", "太陰", "天后"];

/// 天乙贵人起例
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuiRule {
    /// 俗例（曹震圭）：昼贵甲加丑逆行（甲丑乙子丙亥丁酉戊丑己申庚未辛午壬巳癸卯），
    /// 夜贵甲加未顺行（甲未乙申丙酉丁亥戊未己子庚丑辛寅壬卯癸巳），即「甲戊庚牛羊……六辛逢馬虎」。
    /// 协纪卷七引曹震圭；《六壬大全》提要谓是书「所取天乙尚沿俗例」，卷一谓先天贵人图「近不用」。缺省
    SuLi,
    /// 协纪卷七引《考原》定法：阳贵出先天坤自子顺（己子庚丑辛寅壬卯癸巳甲未乙申丙酉丁亥戊丑），
    /// 阴贵出后天坤自申逆（己申庚未辛午壬巳癸卯甲丑乙子丙亥丁酉戊未）；昼用阳贵、夜用阴贵（「曹氏則以陽為隂以隂為陽」）。
    /// 与俗例相较，除戊日外昼夜对调
    KaoYuan,
    /// 《六壬大全》书中实用（课例与明文所称昼夜）：昼贵 甲戊庚丑 乙己子 丙丁亥 辛午 壬癸巳，
    /// 夜贵 甲戊庚未 乙己申 丙丁酉 辛寅 壬癸卯。各干昼夜二贵与俗例同为一对（「甲戊庚牛羊……六辛逢馬虎」），
    /// 只丁、己、庚、癸四干昼夜之名与曹震圭相反。据：卷六引從課「丁酉日酉為夜貴……亥為晝貴」「庚辰日干上丑晝貴人」
    /// 「庚午己酉日伏吟拱夜貴」、卷一「丁酉日陽貴干上亥」、卷十「丁卯日酉加丁旦乘朱雀」「己未日卯加未夜」乘虎等，
    /// 卷五、卷六课例 49 则以卯至酉为昼全合（见 doc/folk/liuren.md「昼夜」）
    DaQuan,
}

/// 昼夜分界
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZhouYe {
    /// 卯至申为昼、酉至寅为夜（以占时地支定）
    MaoYou,
    /// 卯至酉为昼、戌至寅为夜（以占时地支定）：配 [`GuiRule::DaQuan`]，卷五、卷六课例 49 则全合
    MaoZhiYou,
    /// 日出至日入为昼（协纪卷七《考原》「自當以日出入為定」）：须给当地视太阳时与昼半弧，见 [`is_zhou_by_sun`]
    RiChuRu,
}

/// 卯酉为界：占时地支卯、辰、巳、午、未、申为昼
pub fn is_zhou_mao_you(shi: u8) -> bool {
    (3..=8).contains(&(shi % 12))
}

/// 卯至酉为昼（卯辰巳午未申酉）
pub fn is_zhou_mao_zhi_you(shi: u8) -> bool {
    (3..=9).contains(&(shi % 12))
}

/// 太阳视地平高度（度，不计折射）：`jd_ut` UT1 儒略日，`lat_deg` 北纬、`lon_deg` 东经。
/// 时角 H = GAST + 东经 − 视赤经，sin h = sinφ sinδ + cosφ cosδ cosH（视位置取本项目视位置管线）
pub fn sun_altitude_deg(vsop: &Vsop87, jd_ut: Real, lat_deg: Real, lon_deg: Real) -> Real {
    use crate::astronomy::apparent::{right_ascension_declination, sun_apparent_equatorial, ApparentPipelineOptions};
    use crate::astronomy::time::earth_rotation::gast;
    use crate::astronomy::time::{ut1_to_tt_jd, TimePoint, TimeScale};
    use crate::math::real::{real, RealOps};
    use crate::quantity::angle::PlaneAngle;
    let jd_tt = ut1_to_tt_jd(jd_ut);
    let sun = sun_apparent_equatorial(vsop, TimePoint::new(TimeScale::TT, jd_tt), &ApparentPipelineOptions::pipeline_default());
    let (ra, dec) = right_ascension_declination(&sun);
    let phi = PlaneAngle::from_deg(lat_deg).rad();
    let h = gast(jd_ut, jd_tt).rad() + PlaneAngle::from_deg(lon_deg).rad() - ra.rad();
    let d = dec.rad();
    let alt = (phi.sin() * d.sin() + phi.cos() * d.cos() * h.cos()).asin();
    alt * real(180.0) / Real::pi()
}

/// 以当地日出日入定昼：太阳视高度高于 `horizon_deg` 为昼。`horizon_deg` 取 0 为日心在地平、不计折射
/// （协纪卷十二、十三日出入之法，与 `gengdian` 同）；取 −50′（−0.8333°）为今天文日出没（上边缘、计蒙气差）
pub fn is_zhou_at(vsop: &Vsop87, jd_ut: Real, lat_deg: Real, lon_deg: Real, horizon_deg: Real) -> bool {
    sun_altitude_deg(vsop, jd_ut, lat_deg, lon_deg) > horizon_deg
}

/// 日出日入为界：`local_apparent_hours` 当地视太阳时（0—24），`half_arc_hours` 昼半弧（时，见 `gengdian::half_arc_hours`，地平取日心不计折射）
pub fn is_zhou_by_sun(local_apparent_hours: Real, half_arc_hours: Real) -> bool {
    use crate::math::real::{real, RealOps};
    (local_apparent_hours - real(12.0)).abs() < half_arc_hours
}

/// 日干所用天乙贵人（地支序）
pub fn gui_ren(day_gan: u8, zhou: bool, rule: GuiRule) -> u8 {
    // 俗例昼表、夜表（甲……癸）
    const SU_ZHOU: [u8; 10] = [1, 0, 11, 9, 1, 8, 7, 6, 5, 3];
    const SU_YE: [u8; 10] = [7, 8, 9, 11, 7, 0, 1, 2, 3, 5];
    // 《考原》阳贵（先天坤自子顺）、阴贵（后天坤自申逆）
    const KY_YANG: [u8; 10] = [7, 8, 9, 11, 1, 0, 1, 2, 3, 5];
    const KY_YIN: [u8; 10] = [1, 0, 11, 9, 7, 8, 7, 6, 5, 3];
    // 书中实用：昼 甲丑乙子丙亥丁亥戊丑己子庚丑辛午壬巳癸巳，夜 甲未乙申丙酉丁酉戊未己申庚未辛寅壬卯癸卯
    const DQ_ZHOU: [u8; 10] = [1, 0, 11, 11, 1, 0, 1, 6, 5, 5];
    const DQ_YE: [u8; 10] = [7, 8, 9, 9, 7, 8, 7, 2, 3, 3];
    let g = (day_gan % 10) as usize;
    match (rule, zhou) {
        (GuiRule::DaQuan, true) => DQ_ZHOU[g],
        (GuiRule::DaQuan, false) => DQ_YE[g],
        (GuiRule::SuLi, true) => SU_ZHOU[g],
        (GuiRule::SuLi, false) => SU_YE[g],
        (GuiRule::KaoYuan, true) => KY_YANG[g],
        (GuiRule::KaoYuan, false) => KY_YIN[g],
    }
}

/// 一课的十二天将
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TianJiang {
    /// 贵人（天盘神，地支序）及其所临地盘
    pub gui: u8,
    pub gui_lin: u8,
    /// 顺布为 true：贵人临地盘亥子丑寅卯辰顺行，临巳午未申酉戌逆行（卷二「順治謂在天門之前地戸之後」，天门亥、地户巳；
    /// 卷五、卷六课例 49 则无一例外，其中贵人临辰者 4 则皆顺、临戌者 6 则皆逆）
    pub shun: bool,
    /// `at[天盘神]` = 所乘天将序（[`TIAN_JIANG`]）
    pub at: [u8; 12],
}

impl TianJiang {
    /// 天盘神 z 所乘之将名
    pub fn name_of(&self, z: u8) -> &'static str {
        TIAN_JIANG[self.at[(z % 12) as usize] as usize]
    }
}

/// 贵人临辰戌（入狱）之处置（卷二「貴人從十干分晝治暮治獨不居辰戌之地……」下二条，注「二條論貴人出獄之法」）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChuYu {
    /// 不出狱：贵人仍在本神，临辰顺、临戌逆。卷五、卷六课例贵人临辰 4 则、临戌 6 则皆如此。缺省
    Bu,
    /// 「己上起子辰上起丑卯上起寅……午上見亥此謂太乙起子逆後尋也」：地盘 d 对天盘 5 − d（巳子、辰丑……午亥）。
    /// 今读作：临辰者贵人移于地盘丑上之神，临戌者移于地盘未上之神（辰↔丑、戌↔未为此表所配），依新位定顺逆。读法未核
    NiHouXun,
    /// 「貴人臨辰戌者以冲為神臨辰冲申臨戌冲寅乃天乙所臨之位諸將併隨天乙順逆前後取之」：
    /// 临辰者以地盘申为天乙所临（贵人移于申上之神，逆布），临戌者以寅（顺布）。辰申、戌寅即同条「四孟則斜冲」之配
    ChongShen,
}

/// 布十二天将：贵人加于天盘神 `gui_ren(日干)`，依所临地盘定顺逆，诸将随天盘神依次布之
pub fn tian_jiang(pan: &TianPan, day_gan: u8, zhou: bool, rule: GuiRule) -> TianJiang {
    tian_jiang_with(pan, day_gan, zhou, rule, ChuYu::Bu)
}

/// 布十二天将，可选贵人出狱之法
pub fn tian_jiang_with(pan: &TianPan, day_gan: u8, zhou: bool, rule: GuiRule, chu_yu: ChuYu) -> TianJiang {
    let mut gui = gui_ren(day_gan, zhou, rule);
    let mut gui_lin = pan.below(gui);
    if gui_lin == 4 || gui_lin == 10 {
        let to = match chu_yu {
            ChuYu::Bu => None,
            ChuYu::NiHouXun => Some(m12(5 - gui_lin as i32)),
            ChuYu::ChongShen => Some(if gui_lin == 4 { 8 } else { 2 }),
        };
        if let Some(d) = to {
            gui_lin = d;
            gui = pan.above(d);
        }
    }
    let shun = matches!(gui_lin, 11 | 0 | 1 | 2 | 3 | 4);
    let mut at = [0u8; 12];
    for z in 0..12u8 {
        let k = m12(z as i32 - gui as i32);
        at[z as usize] = if shun { k } else { m12(-(k as i32)) };
    }
    TianJiang { gui, gui_lin, shun, at }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::real;

    fn gz(s: &str) -> u8 {
        let c: Vec<char> = s.chars().collect();
        let g = GAN.iter().position(|x| x.starts_with(c[0])).unwrap() as u8;
        let z = ZHI.iter().position(|x| x.starts_with(c[1])).unwrap() as u8;
        (0..60u8).find(|k| k % 10 == g && k % 12 == z).unwrap()
    }
    fn z(s: &str) -> u8 {
        ZHI.iter().position(|x| *x == s).unwrap() as u8
    }
    fn sc(s: &str) -> [u8; 3] {
        let v: Vec<u8> = s.chars().map(|c| z(&c.to_string())).collect();
        [v[0], v[1], v[2]]
    }

    /// 《六壬大全》所举课例：(出处, 日, 时, 将, 三传, 法)
    #[test]
    fn book_examples() {
        let cases: &[(&str, &str, &str, &str, &str, Fa)] = &[
            ("卷五重審課", "丙戌", "巳", "申", "申亥寅", Fa::ChongShen),
            ("卷五重審課", "乙亥", "辰", "酉", "寅未子", Fa::ChongShen),
            ("卷五元首課", "甲子", "卯", "子", "午卯子", Fa::YuanShou),
            ("卷五元首課（鄭妃）", "丁丑", "子", "申", "巳丑酉", Fa::YuanShou),
            ("卷五知一課", "壬辰", "巳", "辰", "戌酉申", Fa::ZhiYi),
            ("卷五渉害課", "丁卯", "丑", "亥", "亥酉未", Fa::SheHai),
            // 书名見機；按訂訛深浅，午二重、戌一重，以深者取，三传同
            ("卷五渉害課見機格", "庚子", "戌", "申", "午辰寅", Fa::SheHai),
            ("卷五渉害課察微格", "庚戌", "辰", "申", "辰申子", Fa::ChaWei),
            // 书名綴瑕；按訂訛，深浅相等而辰临孟（寅），即見機，三传同
            ("卷五渉害課綴瑕格", "甲午", "辰", "午", "辰午申", Fa::JianJi),
            ("卷五渉害課比用格", "乙卯", "寅", "子", "亥酉未", Fa::SheHai),
            ("卷五蒿矢", "壬辰", "巳", "申", "戌丑辰", Fa::HaoShi),
            ("卷五彈射", "壬申", "申", "亥", "巳申亥", Fa::DanShe),
            ("卷五昴星虎視", "戊申", "卯", "辰", "戌酉午", Fa::MaoXing),
            ("卷五昴星冬蛇掩目", "丁丑", "辰", "丑", "子辰戌", Fa::MaoXing),
            ("卷五昴星觀月經注", "乙未", "辰", "寅", "亥寅巳", Fa::MaoXing),
            ("卷五别責", "丙辰", "卯", "辰", "亥午午", Fa::BieZe),
            ("卷五八專", "甲寅", "辰", "丑", "丑亥亥", Fa::BaZhuan),
            ("卷五八專帷簿不修", "丁未", "丑", "辰", "亥戌戌", Fa::BaZhuan),
            ("卷五八專獨足", "己未", "未", "酉", "酉酉酉", Fa::BaZhuan),
            ("卷五伏吟", "癸巳", "午", "午", "丑戌未", Fa::FuYin),
            ("卷五返吟", "庚戌", "寅", "申", "寅申寅", Fa::FanYin),
            ("卷五返吟井欄射", "辛丑", "巳", "亥", "亥未辰", Fa::JingLan),
        ];
        for (src, d, s, j, want, fa) in cases {
            let k = qi_ke(gz(d), z(j), z(s));
            assert_eq!(k.san_chuan, sc(want), "{src} {d}日{s}時{j}將");
            assert_eq!(k.fa, *fa, "{src} {d}日{s}時{j}將");
        }
    }

    /// 卷五伏吟訂訛：「甲日自用寅三傳寅巳申，丙戊日自用巳三傳巳申寅，庚日自用申三傳申寅巳」
    #[test]
    fn fu_yin_zi_ren() {
        for (d, want) in [("甲子", "寅巳申"), ("丙子", "巳申寅"), ("戊子", "巳申寅"), ("庚子", "申寅巳")] {
            assert_eq!(qi_ke(gz(d), 0, 0).san_chuan, sc(want), "{d}");
        }
    }

    /// 渉害深浅：卷五訂訛二例
    #[test]
    fn she_hai_examples() {
        assert_eq!(she_hai_depth(z("亥"), z("丑"), true), 5); // 亥加丑：辰戊未己戌五重
        assert_eq!(she_hai_depth(z("丑"), z("卯"), true), 1); // 丑加卯：辰中乙木一重
        assert_eq!(she_hai_depth(z("午"), z("申"), false), 2); // 午加庚（申）：酉辛二重
        assert_eq!(she_hai_depth(z("戌"), z("子"), false), 1); // 戌加子：癸水一重
        assert_eq!(she_hai_depth(z("戌"), z("寅"), true), 2); // 觀月經注：河魁加寅，卯乙二重
        assert_eq!(she_hai_depth(z("子"), z("辰"), true), 4); // 神后加辰：戊未己戌四重
    }

    /// 月将：中气后换。用本项目历表，2026 年春分前后
    #[test]
    fn yue_jiang_switch() {
        let vsop = crate::astronomy::ephemeris::load_earth_vsop87_from_repo().unwrap();
        let t = solar_term_jds_for_year_cached(&vsop, 2026);
        let eps = real(1.0 / 86400.0);
        assert_eq!(yue_jiang(&vsop, t[0] - eps, JiangRule::ZhongQi), z("亥")); // 春分前一秒仍亥将
        assert_eq!(yue_jiang(&vsop, t[0] + eps, JiangRule::ZhongQi), z("戌")); // 春分后戌将
        assert_eq!(yue_jiang(&vsop, t[18] + eps, JiangRule::ZhongQi), z("丑")); // 冬至后丑将
        // 按节：清明（k=1）后辰月，酉将；惊蛰（k=23）后卯月，戌将
        assert_eq!(yue_jiang(&vsop, t[1] + eps, JiangRule::Jie), z("酉"));
        assert_eq!(yue_jiang(&vsop, t[0] + eps, JiangRule::Jie), z("戌"));
    }

    /// 恒星过宫（卷十一「太陽過宫」）：以授时（元）距星、卷十一宿度宫界，求某年太阳入各宫之日距所注节气之日数，与书「某節前若干日」比。
    /// 节气用当时行用历之平气（授时、大统）。宿界三读：距星本身之黄经、赤道宿度（赤经）、古法黄道宿度（距星赤经圈交黄道）；
    /// 对 1550（明）各算，古法另算 1280（授时）。逐宫之差见 doc/folk/liuren.md 第一节
    #[test]
    fn heng_xing_guo_gong() {
        use crate::astronomy::apparent::{sun_apparent_ecliptic_longitude, ApparentPipelineOptions};
        use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
        use crate::astronomy::pipeline::Vsop87Planets;
        use crate::astronomy::sky_culture::chinese::{load_lunar_mansion_systems_from_repo, MansionFrame};
        use crate::astronomy::stars::load_hipparcos_bright_from_repo;
        use crate::astronomy::time::{TimePoint, TimeScale};
        use crate::math::real::RealOps;
        let _ = try_init_full_nutation_from_repo();
        let vsop = crate::astronomy::ephemeris::load_earth_vsop87_from_repo().unwrap();
        let eph = Vsop87Planets { earth: &vsop, planets: vec![] };
        let cat = load_hipparcos_bright_from_repo().unwrap();
        let sys = load_lunar_mansion_systems_from_repo().unwrap().into_iter().find(|s| s.id == "yuan").unwrap();
        // (宫, 书中所注节气序（JIE_QI：0 春分…23 驚蟄）, 书言节前日数)
        let book: [(u8, usize, f64); 12] = [
            (0, 21, 9.0),
            (11, 23, 10.0),
            (10, 1, 8.0),
            (9, 3, 6.0),
            (8, 5, 5.0),
            (7, 7, 5.0),
            (6, 9, 6.0),
            (5, 11, 5.0),
            (4, 13, 3.0),
            (3, 15, 1.0),
            (2, 17, 3.0),
            (1, 19, 6.0),
        ];
        let mut rows = Vec::new();
        let mut summary: Vec<(usize, i32, f64, f64)> = Vec::new();
        for (chi_dao, year) in [(0, 1550), (1, 1550), (2, 1280), (2, 1550)] {
            // 冬至前后一年：自上年十一月至本年十二月，步长 0.05 日找太阳入宫之时
            let mid = Gregorian::to_julian_day(year, 7, 1);
            let t_mid = TimePoint::new(TimeScale::TT, mid);
            let frame = MansionFrame::new(&eph, &cat, &sys, t_mid, &ApparentPipelineOptions::default()).unwrap();
            let r2d = real(180.0 / core::f64::consts::PI);
            let ml = match chi_dao {
                1 => core::array::from_fn(|i| frame.ra[i] * r2d),
                // 古法黄道宿度：距星赤经圈交黄道之点的黄经（tan λ = tan α / cos ε）
                2 => mansion_gu_huang_dao(&frame, t_mid),
                _ => crate::folk::qizheng::mansion_longitudes(&frame, t_mid),
            };
            let terms: Vec<(Real, usize)> = [year - 1, year, year + 1]
                .iter()
                .flat_map(|y| {
                    solar_term_jds_for_year_cached(&vsop, *y)
                        .into_iter()
                        .enumerate()
                        // 当时行用历之平气（授时、大统），见 calendar::pingqi
                        .map(|(k, t)| (crate::calendar::pingqi::pingqi_tt(t, (k as i32 + 6) % 24).map_or(t, |x| x.1), k))
                        .collect::<Vec<_>>()
                })
                .collect();
            let start = Gregorian::to_julian_day(year, 1, 1);
            let lon = |jd: Real| {
                let t = TimePoint::new(TimeScale::TT, jd);
                if chi_dao == 1 {
                    let (a, _) = crate::astronomy::apparent::right_ascension_declination(&crate::astronomy::apparent::sun_apparent_equatorial(&vsop, t, &ApparentPipelineOptions::default()));
                    a.wrap_to_2pi().rad() * r2d
                } else {
                    sun_apparent_ecliptic_longitude(&vsop, t).wrap_to_2pi().rad() * r2d
                }
            };
            let mut prev = yue_jiang_xiu(lon(start), &ml);
            let mut jd = start;
            let mut entry: [Option<Real>; 12] = [None; 12];
            let mut diffs: Vec<f64> = Vec::new();
            while jd < start + real(366.0) {
                jd += real(0.25);
                let g = yue_jiang_xiu(lon(jd), &ml);
                if g != prev {
                    entry[g as usize] = Some(jd);
                    prev = g;
                }
            }
            for (g, k, want) in book {
                let e = entry[g as usize].unwrap();
                // 入宫之后最近的该节气
                let jie = terms.iter().filter(|(t, kk)| *kk == k && *t > e - real(20.0)).map(|(t, _)| *t).fold(real(f64::MAX), |a, b| if b < a { b } else { a });
                let before = (jie - e).as_f64();
                rows.push(format!("| {} | {year} | {} | {}前{want}日 | {before:.1} | {:+.1} |", ["黄道（距星黄经）", "赤道", "黄道（赤经圈交黄道）"][chi_dao], ZHI[g as usize], super::super::qimen::JIE_QI[k], before - want));
                diffs.push(before - want);
            }
            let mean = diffs.iter().sum::<f64>() / 12.0;
            let range = diffs.iter().fold(f64::MIN, |a, b| a.max(*b)) - diffs.iter().fold(f64::MAX, |a, b| a.min(*b));
            summary.push((chi_dao, year, mean, range));
        }
        eprintln!("| 宿度 | 年 | 宫 | 书 | 推算（日） | 差 |\n|---|---|---|---|---|---|\n{}", rows.join("\n"));
        eprintln!("{summary:?}");
        let r = |m: usize, y: i32| summary.iter().find(|x| x.0 == m && x.1 == y).unwrap().3;
        // 古法黄道宿度、平气：十二宫之差近乎一常数（岁差所积），远较以距星黄经、赤道宿度为齐
        assert!(r(2, 1550) < 4.5 && r(2, 1550) < r(0, 1550), "{summary:?}");
    }

    /// 《六壬大全》卷五、卷六课例所注三传天将（逐例：卷、日、时、将、三传、三传所乘之将、书中所用为昼贵否）。
    /// 49 例皆合俗例昼表或夜表、贵人所临地盘定顺逆。书中所用昼夜与占时无一致关系（35 昼、14 夜，如丁丑日子时用昼贵、
    /// 癸巳日午时用夜贵），故此处记书中实所用者，不以时定。卷六「如丙申日卯時子將……」下第二盘实为同条所引
    /// 「庚辰日亥加寅」之例；戊申日子时一例第三将录文作「亥」，当为「𤣥」之讹
    #[test]
    fn book_tian_jiang() {
        let cases: &[(&str, &str, &str, &str, &str, [&str; 3], bool)] = &[
            ("卷五", "甲子", "卯", "子", "午卯子", ["青龍", "朱雀", "天后"], true),
            ("卷五", "丁丑", "子", "申", "巳丑酉", ["太常", "勾陳", "貴人"], true),
            ("卷五", "丙戌", "巳", "申", "申亥寅", ["六合", "貴人", "玄武"], true),
            ("卷五", "乙亥", "辰", "酉", "寅未子", ["太陰", "青龍", "貴人"], true),
            ("卷五", "壬辰", "巳", "辰", "戌酉申", ["白虎", "太常", "玄武"], true),
            ("卷五", "丁卯", "丑", "亥", "亥酉未", ["朱雀", "貴人", "太陰"], true),
            ("卷五", "庚子", "戌", "申", "午辰寅", ["螣蛇", "六合", "青龍"], true),
            ("卷五", "庚戌", "辰", "申", "辰申子", ["玄武", "青龍", "螣蛇"], false),
            ("卷五", "甲午", "辰", "午", "辰午申", ["六合", "青龍", "白虎"], true),
            ("卷五", "乙卯", "寅", "子", "亥酉未", ["玄武", "天后", "螣蛇"], false),
            ("卷五", "壬辰", "巳", "申", "戌丑辰", ["青龍", "太常", "天后"], true),
            ("卷五", "壬申", "申", "亥", "巳申亥", ["貴人", "六合", "天空"], true),
            ("卷五", "戊申", "卯", "辰", "戌酉午", ["玄武", "太常", "青龍"], true),
            ("卷五", "丁丑", "辰", "丑", "子辰戌", ["螣蛇", "青龍", "天后"], false),
            ("卷五", "丙辰", "卯", "辰", "亥午午", ["貴人", "青龍", "青龍"], true),
            ("卷五", "甲寅", "辰", "丑", "丑亥亥", ["貴人", "太陰", "太陰"], true),
            ("卷五", "丁未", "丑", "辰", "亥戌戌", ["太陰", "天后", "天后"], true),
            ("卷五", "己未", "未", "酉", "酉酉酉", ["六合", "六合", "六合"], false),
            ("卷五", "癸巳", "午", "午", "丑戌未", ["勾陳", "白虎", "太陰"], false),
            ("卷五", "丙辰", "申", "申", "巳申寅", ["天空", "玄武", "六合"], true),
            ("卷五", "丁丑", "未", "未", "丑戌未", ["朱雀", "天后", "太常"], false),
            ("卷五", "壬辰", "酉", "酉", "亥辰戌", ["天空", "螣蛇", "白虎"], true),
            ("卷五", "庚戌", "寅", "申", "寅申寅", ["白虎", "螣蛇", "白虎"], true),
            ("卷五", "辛丑", "巳", "亥", "亥未辰", ["青龍", "螣蛇", "太陰"], true),
            ("卷五", "戊寅", "午", "寅", "丑午酉", ["貴人", "白虎", "勾陳"], true),
            ("卷五", "乙丑", "酉", "戌", "寅卯辰", ["朱雀", "六合", "勾陳"], true),
            ("卷五", "丙辰", "寅", "未", "寅未子", ["青龍", "太陰", "六合"], false),
            ("卷五", "戊寅", "戌", "卯", "子巳戌", ["青龍", "太陰", "六合"], false),
            ("卷五", "癸酉", "酉", "巳", "巳丑酉", ["貴人", "勾陳", "太常"], false),
            ("卷六", "丁亥", "巳", "戌", "巳戌卯", ["天空", "螣蛇", "太常"], false),
            ("卷六", "辛巳", "丑", "戌", "寅亥申", ["貴人", "六合", "天空"], false),
            ("卷六", "丙子", "未", "子", "巳戌卯", ["天空", "螣蛇", "太常"], true),
            ("卷六", "辛丑", "辰", "亥", "卯戌巳", ["玄武", "勾陳", "天后"], true),
            ("卷六", "壬子", "巳", "戌", "巳戌卯", ["貴人", "青龍", "太陰"], true),
            ("卷六", "丙戌", "申", "亥", "申亥寅", ["六合", "貴人", "玄武"], true),
            ("卷六", "壬申", "未", "巳", "午辰寅", ["天后", "螣蛇", "六合"], true),
            ("卷六", "丙申", "卯", "子", "巳寅亥", ["天空", "六合", "貴人"], true),
            ("卷六", "庚辰", "寅", "亥", "巳寅亥", ["朱雀", "青龍", "太常"], true),
            ("卷六", "戊子", "戌", "卯", "巳戌卯", ["太陰", "六合", "太常"], false),
            ("卷六", "戊申", "子", "申", "子申辰", ["青龍", "螣蛇", "玄武"], false),
            ("卷六", "壬午", "巳", "丑", "戌午寅", ["白虎", "天后", "六合"], true),
            ("卷六", "甲寅", "亥", "未", "戌午寅", ["六合", "天后", "白虎"], false),
            ("卷六", "甲申", "卯", "子", "巳寅亥", ["勾陳", "螣蛇", "太陰"], true),
            ("卷六", "乙未", "卯", "寅", "戌卯午", ["太陰", "六合", "天空"], true),
            ("卷六", "戊子", "午", "酉", "卯午酉", ["太陰", "白虎", "勾陳"], true),
            ("卷六", "甲戌", "卯", "亥", "戌午寅", ["六合", "白虎", "天后"], true),
            ("卷六", "丙申", "辰", "丑", "巳寅亥", ["天空", "六合", "貴人"], true),
            ("卷六", "辛未", "申", "辰", "卯亥未", ["六合", "白虎", "天后"], true),
            ("卷六", "戊戌", "辰", "午", "子寅辰", ["天后", "螣蛇", "六合"], true),
        ];
        assert_eq!(cases.len(), 49);
        let mut su_mao_you = 0;
        // 贵人临辰戌之例数、出狱二法合者数
        let (mut yu, mut yu_ok) = (0, 0);
        for (vol, d, shi, jiang, chuan, jiangs, zhou) in cases {
            let g = gz(d);
            let pan = TianPan { jiang: z(jiang), shi: z(shi) };
            let sc3 = sc(chuan);
            let tj = tian_jiang(&pan, g % 10, *zhou, GuiRule::SuLi);
            let got: Vec<&str> = sc3.iter().map(|c| tj.name_of(*c)).collect();
            assert_eq!(got, jiangs.to_vec(), "{vol} {d}日{shi}時{jiang}將");
            // 书中实用表 + 卯至酉为昼：由占时即可定，49 则全合
            let dq = tian_jiang(&pan, g % 10, is_zhou_mao_zhi_you(z(shi)), GuiRule::DaQuan);
            let got3: Vec<&str> = sc3.iter().map(|c| dq.name_of(*c)).collect();
            assert_eq!(got3, jiangs.to_vec(), "书中实用 {vol} {d}日{shi}時{jiang}將");
            // 贵人临辰戌者：不出狱合；二出狱法皆不合
            if matches!(tj.gui_lin, 4 | 10) {
                yu += 1;
                for cy in [ChuYu::NiHouXun, ChuYu::ChongShen] {
                    let t = tian_jiang_with(&pan, g % 10, *zhou, GuiRule::SuLi, cy);
                    if sc3.iter().map(|c| t.name_of(*c)).collect::<Vec<_>>() == jiangs.to_vec() {
                        yu_ok += 1;
                    }
                }
            }
            // 俗例 + 卯至申：只 35 则合
            let su = tian_jiang(&pan, g % 10, is_zhou_mao_you(z(shi)), GuiRule::SuLi);
            if sc3.iter().map(|c| su.name_of(*c)).collect::<Vec<_>>() == jiangs.to_vec() {
                su_mao_you += 1;
            }
            // 《考原》定法除戊日外昼夜对调，同一盘亦可由之得出：课例不能分辨两说
            let ky = tian_jiang(&pan, g % 10, if g % 10 == 4 { *zhou } else { !*zhou }, GuiRule::KaoYuan);
            let got2: Vec<&str> = sc3.iter().map(|c| ky.name_of(*c)).collect();
            assert_eq!(got2, jiangs.to_vec(), "考原 {vol} {d}日");
        }
        assert_eq!(su_mao_you, 35);
        assert_eq!((yu, yu_ok), (10, 0));
    }

    /// 书中明言昼（旦）夜（暮）而举所乘之将者（卷一、卷九、卷十）：(出处, 日干, 天盘神, 所临地盘（None 为不定，任一盘可得即算）, 昼否, 所乘)
    /// 俗例 19/26、考原 13/26、书中实用 26/26
    #[test]
    fn book_explicit_zhou_ye() {
        fn ji(gs: &str) -> u8 {
            GAN_JI[GAN.iter().position(|x| *x == gs).unwrap()]
        }
        let cases: Vec<(&str, &str, &str, Option<u8>, bool, &str)> = vec![
            ("卷十 六辛日丑加戌旦將乘白虎", "辛", "丑", Some(z("戌")), true, "白虎"),
            ("卷十 乙亥日未加亥旦占乘白虎", "乙", "未", Some(z("亥")), true, "白虎"),
            ("卷十 六己日卯加己夜", "己", "卯", Some(ji("己")), false, "白虎"),
            ("卷十 六壬日戌加壬旦", "壬", "戌", Some(ji("壬")), true, "白虎"),
            ("卷十 六壬日戌加壬夜", "壬", "戌", Some(ji("壬")), false, "白虎"),
            ("卷十 六癸日戌加癸晝", "癸", "戌", Some(ji("癸")), true, "白虎"),
            ("卷十 丙子日辰加子旦", "丙", "辰", Some(z("子")), true, "白虎"),
            ("卷十 丁亥日辰加亥夜", "丁", "辰", Some(z("亥")), false, "白虎"),
            ("卷十 壬子日戌加子旦", "壬", "戌", Some(z("子")), true, "白虎"),
            ("卷十 丁丑日寅加丑夜", "丁", "寅", Some(z("丑")), false, "白虎"),
            ("卷十 壬寅日返吟夜", "壬", "申", Some(z("寅")), false, "白虎"),
            ("卷十 癸卯日申加卯夜", "癸", "申", Some(z("卯")), false, "白虎"),
            ("卷十 壬午日返吟晝", "壬", "子", Some(z("午")), true, "白虎"),
            ("卷十 乙未日卯加未夜", "乙", "卯", Some(z("未")), false, "白虎"),
            ("卷十 己未日卯加未夜", "己", "卯", Some(z("未")), false, "白虎"),
            ("卷十 丙申日午加申晝", "丙", "午", Some(z("申")), true, "白虎"),
            ("卷十 癸亥日戌加亥旦", "癸", "戌", Some(z("亥")), true, "白虎"),
            ("卷十 癸亥日戌加亥暮", "癸", "戌", Some(z("亥")), false, "白虎"),
            ("卷十 丁卯日酉加丁旦乘朱雀", "丁", "酉", Some(ji("丁")), true, "朱雀"),
            ("卷十 丙寅日申加巳夜乘蛇", "丙", "申", Some(z("巳")), false, "螣蛇"),
            ("卷十 七月癸亥日辰作天后加亥夜", "癸", "辰", Some(z("亥")), false, "天后"),
            ("卷一 辛亥日未加亥夜作白虎", "辛", "未", Some(z("亥")), false, "白虎"),
            ("卷九 辛巳日晝將順行丑乗白虎", "辛", "丑", None, true, "白虎"),
            ("卷九 辛夘日干上酉晝乗玄", "辛", "酉", Some(ji("辛")), true, "玄武"),
            ("卷九 辛夘日干上酉夜乗虎", "辛", "酉", Some(ji("辛")), false, "白虎"),
            ("卷九 六己日夜貴逆布朱雀乘午", "己", "午", None, false, "朱雀"),
        ];
        let count = |r: GuiRule| {
            cases
                .iter()
                .filter(|(_, g, s, lin, zhou, want)| {
                    let gi = GAN.iter().position(|x| x == g).unwrap() as u8;
                    let s = z(s);
                    let ok = |jiang: u8| {
                        let pan = TianPan { jiang, shi: 0 };
                        tian_jiang(&pan, gi, *zhou, r).name_of(s) == *want
                    };
                    match lin {
                        // 天盘 s 临地盘 lin：jiang + lin − 0 = s
                        Some(l) => ok(m12(s as i32 - *l as i32)),
                        None => (0..12).any(ok),
                    }
                })
                .count()
        };
        assert_eq!(cases.len(), 26);
        assert_eq!(count(GuiRule::DaQuan), 26);
        assert_eq!(count(GuiRule::SuLi), 19);
        assert_eq!(count(GuiRule::KaoYuan), 13);
        // 卷六引從課：丁酉日酉為夜貴、亥為晝貴；庚辰日丑晝貴；庚午、己酉日伏吟拱夜貴（未、申）；甲子日伏吟拱晝貴（丑）；
        // 卷六富貴課訂訛：乙酉日子加申為晝貴坐夜貴；卷五六儀課：乙丑己巳日夜占、乙酉己丑日晝占旬首作簾幙（他一贵）
        assert_eq!((gui_ren(3, false, GuiRule::DaQuan), gui_ren(3, true, GuiRule::DaQuan)), (z("酉"), z("亥")));
        assert_eq!(gui_ren(6, true, GuiRule::DaQuan), z("丑"));
        assert_eq!((gui_ren(6, false, GuiRule::DaQuan), gui_ren(5, false, GuiRule::DaQuan)), (z("未"), z("申")));
        assert_eq!(gui_ren(0, true, GuiRule::DaQuan), z("丑"));
        assert_eq!((gui_ren(1, true, GuiRule::DaQuan), gui_ren(1, false, GuiRule::DaQuan)), (z("子"), z("申")));
        // 簾幙：夜占用昼贵作簾幙，乙丑、己巳（甲子旬首子）；昼占用夜贵，乙酉、己丑（甲申旬首申）
        assert_eq!((gui_ren(1, true, GuiRule::DaQuan), gui_ren(5, true, GuiRule::DaQuan)), (z("子"), z("子")));
        assert_eq!((gui_ren(1, false, GuiRule::DaQuan), gui_ren(5, false, GuiRule::DaQuan)), (z("申"), z("申")));
        // 卷九兩貴引從：壬子日巳為晝貴、卯為夜貴
        assert_eq!((gui_ren(8, true, GuiRule::DaQuan), gui_ren(8, false, GuiRule::DaQuan)), (z("巳"), z("卯")));
        // 书中实用表与俗例同为一对
        for g in 0..10u8 {
            let mut a = [gui_ren(g, true, GuiRule::DaQuan), gui_ren(g, false, GuiRule::DaQuan)];
            let mut b = [gui_ren(g, true, GuiRule::SuLi), gui_ren(g, false, GuiRule::SuLi)];
            a.sort();
            b.sort();
            assert_eq!(a, b);
        }
    }

    /// 遥克、渉害异说
    #[test]
    fn she_hai_options() {
        let wsr = QiKeOpts { she_hai: SheHaiRule::WeiShanRen, ..QiKeOpts::default() };
        let byg = QiKeOpts { she_hai: SheHaiRule::BiYongGe, ..QiKeOpts::default() };
        // 书中渉害诸例：缺省皆合（见 book_examples）；尉山人说只改上克下相等者，庚戌日察微例（辰临子、申临辰？）
        for (d, s, j) in [("庚子", "戌", "申"), ("庚戌", "辰", "申"), ("甲午", "辰", "午"), ("丁卯", "丑", "亥"), ("乙卯", "寅", "子")] {
            let a = qi_ke(gz(d), z(j), z(s));
            let b = qi_ke_with(gz(d), z(j), z(s), wsr);
            println!("{d}日{s}時{j}將 訂訛 {:?} {:?}；尉山人 {:?} {:?}；比用格 {:?}", a.san_chuan, a.fa, b.san_chuan, b.fa, qi_ke_with(gz(d), z(j), z(s), byg).san_chuan);
        }
        // 比用格：卷五所举「戊子日午卯子壬戌日申丑午庚子日戌申午乙卯日亥酉未皆用比不用渉害」「甲辰三傳……當取子水……三傳子申辰」，
        // 时将未举：于一日 144 盘中求比用格所得三传
        for (d, want) in [("戊子", "午卯子"), ("壬戌", "申丑午"), ("庚子", "戌申午"), ("乙卯", "亥酉未"), ("甲辰", "子申辰")] {
            let hits: Vec<(u8, u8, Fa, [u8; 3])> = (0..12u8)
                .flat_map(|j| (0..12u8).map(move |s| (j, s)))
                .filter_map(|(j, s)| {
                    let k = qi_ke_with(gz(d), j, s, byg);
                    (k.san_chuan == sc(want)).then(|| (j, s, k.fa, qi_ke(gz(d), j, s).san_chuan))
                })
                .collect();
            println!("{d} {want}: {:?}", hits[0]);
            // 每日恰一盘（将时差相同者同盘）
            assert_eq!(hits.len(), 12, "{d}");
            assert!(hits.iter().all(|h| h.2 == Fa::BiYongGe || d == "乙卯"), "{d}");
        }
    }

    /// 以日出入定昼夜：北京（北纬 39.9°、东经 116.4°）2024 年夏至日，北京时 12 时太阳高约 73.5°，0 时在地平下
    #[test]
    fn zhou_by_sunrise() {
        use crate::math::real::RealOps;
        let vsop = crate::astronomy::ephemeris::load_earth_vsop87_from_repo().unwrap();
        let jd = crate::calendar::gregorian::Gregorian::to_julian_day(2024, 6, 21); // 0h UT
        let noon = jd + real(4.0 / 24.0); // 北京时 12 时
        let alt = sun_altitude_deg(&vsop, noon, real(39.9), real(116.4)).as_f64();
        assert!((alt - 73.5).abs() < 0.5, "{alt}");
        assert!(is_zhou_at(&vsop, noon, real(39.9), real(116.4), real(0.0)));
        assert!(!is_zhou_at(&vsop, jd - real(8.0 / 24.0), real(39.9), real(116.4), real(0.0)));
        // 卯时初（5 时）夏至北京已出、冬至未出
        let t5 = jd + real(-3.0 / 24.0);
        assert!(is_zhou_at(&vsop, t5, real(39.9), real(116.4), real(0.0)));
        let w = crate::calendar::gregorian::Gregorian::to_julian_day(2024, 12, 21) - real(3.0 / 24.0);
        assert!(!is_zhou_at(&vsop, w, real(39.9), real(116.4), real(0.0)));
    }

    #[test]
    fn gui_ren_tables() {
        // 俗例：甲戊庚牛羊、乙己鼠猴、丙丁猪鸡、壬癸兔蛇、六辛马虎（昼夜各取其一）
        let pair = |g: u8| {
            let mut v = [gui_ren(g, true, GuiRule::SuLi), gui_ren(g, false, GuiRule::SuLi)];
            v.sort();
            v
        };
        for g in [0u8, 4, 6] {
            assert_eq!(pair(g), [1, 7]); // 丑未
        }
        for g in [1u8, 5] {
            assert_eq!(pair(g), [0, 8]); // 子申
        }
        for g in [2u8, 3] {
            assert_eq!(pair(g), [9, 11]); // 酉亥
        }
        for g in [8u8, 9] {
            assert_eq!(pair(g), [3, 5]); // 卯巳
        }
        assert_eq!(pair(7), [2, 6]); // 寅午
        // 曹震圭：昼贵甲加丑逆行
        assert_eq!(gui_ren(0, true, GuiRule::SuLi), 1);
        // 《考原》：阳贵先天坤自子顺，己得子；阴贵后天坤自申逆，己得申；戊阳丑阴未（范蠡「戊日丑為陽貴」，子胥「甲日丑為隂貴」）
        assert_eq!(gui_ren(5, true, GuiRule::KaoYuan), 0);
        assert_eq!(gui_ren(5, false, GuiRule::KaoYuan), 8);
        assert_eq!(gui_ren(4, true, GuiRule::KaoYuan), 1);
        assert_eq!(gui_ren(0, false, GuiRule::KaoYuan), 1);
        // 两说：除戊外昼夜对调
        for g in 0..10u8 {
            for zhou in [true, false] {
                let flip = if g == 4 { zhou } else { !zhou };
                assert_eq!(gui_ren(g, zhou, GuiRule::KaoYuan), gui_ren(g, flip, GuiRule::SuLi), "干 {g}");
            }
        }
        // 两说贵人皆不临辰戌（卷二「獨不居辰戌之地，以辰為宰殺，戌為牢獄」）
        for g in 0..10u8 {
            for zhou in [true, false] {
                for r in [GuiRule::SuLi, GuiRule::KaoYuan] {
                    assert!(![4u8, 10].contains(&gui_ren(g, zhou, r)));
                }
            }
        }
    }

    #[test]
    fn tian_jiang_order_and_direction() {
        // 甲子日卯時子將（卷五元首）：昼贵丑，临地盘辰，顺布：丑贵、寅螣、卯朱……午青
        let pan = TianPan { jiang: 0, shi: 3 };
        let tj = tian_jiang(&pan, 0, true, GuiRule::SuLi);
        assert_eq!((tj.gui, tj.gui_lin, tj.shun), (1, 4, true));
        assert_eq!(tj.name_of(2), "螣蛇");
        assert_eq!(tj.name_of(7), "天空"); // 贵人对冲
        assert_eq!(tj.name_of(0), "天后"); // 后一
        // 乙卯日寅時子將：夜贵申，临地盘戌，逆布
        let pan = TianPan { jiang: 0, shi: 2 };
        let tj = tian_jiang(&pan, 1, false, GuiRule::SuLi);
        assert_eq!((tj.gui, tj.gui_lin, tj.shun), (8, 10, false));
        assert_eq!(tj.name_of(7), "螣蛇");
        // 昼夜分界
        assert!(is_zhou_mao_you(3) && is_zhou_mao_you(8) && !is_zhou_mao_you(9) && !is_zhou_mao_you(2));
        assert!(is_zhou_by_sun(real(9.0), real(5.0)) && !is_zhou_by_sun(real(18.5), real(6.0)));
    }
}
