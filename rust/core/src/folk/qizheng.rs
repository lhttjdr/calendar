//! 七政四余（星命）：日、月、五星（七政）与罗睺、计都、月孛、紫炁（四余）的宫度、宿度，命宫与十二宫。
//!
//! - 七政：本项目历表的地心视黄经、视黄纬（当日真黄道、真春分点；`body_apparent_ecliptic`）。
//! - 罗睺、计都：白道交点。哪一个是升交点（正交），各家相反，见 [`LuoJi`]；交点取平交点或真（瞬时）交点，见 [`NodeKind`]。
//! - 月孛：月之最高（远地点），见 [`BeiKind`]。
//! - 紫炁：天无其象的虚点，约二十八年一周，见 [`ZiQiRule`]。
//! - 宫：以中气分宫（太阳过宫），黄经 0°—30° 为戌宫（降娄）……330°—360° 为亥宫（娵訾），与时宪书同。
//! - 宿度：黄道宿度——二十八宿距星的视黄经为各宿起点，天体视黄经减其西侧最近之距星，周天 360°（清时宪书即此）。
//! - 命宫：以太阳所躔之宫加生时，卯上所临之宫为命宫（《星學大成》卷一「天盤轉出地盤上，卯上分明是命宫」），
//!   或取天文上升点所在宫（选项）；十二宫自命宫逆布（通例）。
//!
//! 原典、实测核对与各说出处见 doc/folk/qizheng.md。

use crate::astronomy::apparent::{body_apparent_ecliptic, ApparentPipelineOptions};
use crate::astronomy::frame::nutation::{nutation_for_model, NutationModel};
use crate::astronomy::frame::precession::mean_obliquity;
use crate::astronomy::pipeline::{Body, EphemerisProvider, Planet};
use crate::astronomy::sky_culture::chinese::{ecliptic_longitude, MansionFrame, MANSIONS};
use crate::astronomy::time::{TimePoint, TimeScale};
use crate::math::real::{real, Real, RealOps};

/// 十一曜之序：太阳、太阴、水、金、火、木、土、罗睺、计都、月孛、紫炁
pub const YAO: [&str; 11] = ["太陽", "太陰", "水星", "金星", "火星", "木星", "土星", "羅睺", "計都", "月孛", "紫炁"];

/// 十二宫（次）名，下标为地支序（子 0 … 亥 11）
pub const GONG_CI: [&str; 12] = ["玄枵", "星紀", "析木", "大火", "壽星", "鶉尾", "鶉火", "鶉首", "實沈", "大梁", "降婁", "娵訾"];

/// 十二宫（人事），自命宫逆布
pub const PALACES: [&str; 12] = ["命宮", "財帛", "兄弟", "田宅", "男女", "奴僕", "妻妾", "疾厄", "遷移", "官祿", "福德", "相貌"];

/// 罗睺、计都何者为升交点（正交）。两说在中国前后更替（实物实测见 doc/folk/qizheng.md）：
/// 明回回历《七政推步》以计都为升交；崇祯新法（汤若望）、清初时宪书以罗睺为升交（杨光先斥之「以羅為計，以計為羅」）；
/// 清中期以后时宪书又以计都为升交。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuoJi {
    /// 计都为升交点、罗睺为降交点：回回历《七政推步》卷一「計都與月相離度……在初宮至五宮為黃道北」；
    /// 道光十三年（1833）七政经纬躔度时宪书实测（罗计差 25′ 内）。缺省
    JiAscending,
    /// 罗睺为升交点（正交）、计都为降交点：《明史》卷三十一崇祯改历议「羅喉、計都即正交、中交行度」；
    /// 康熙九年（1670）七政经纬躔度时宪历实测（罗睺差 30′、计都差 36′）；杨光先《不得已·摘謬十論》第八謬
    /// 「顛倒羅計之新……不過以羅為計，以計為羅」即斥此（相对回回旧法）。印度 Rāhu 亦为北交点
    LuoAscending,
}

/// 交点、最高取平行或真（瞬时）位置
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    /// 平交点：ELP 平均根数（Meeus《Astronomical Algorithms》式 47.7）。古今历家「為平行」（《明史》卷三十一）
    Mean,
    /// 真交点：瞬时轨道面与黄道之交（由月视位置与速度求角动量方向）。崇祯改历「設一加減，為交行均數」，
    /// 道光十三年时宪书罗计逐旬行度不匀，与真交点合（差 ±25′ 内）、与平交点差至 74′，故缺省
    True,
}

/// 月孛
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeiKind {
    /// 平远地点（ELP 平均根数，近地点 + 180°）。缺省：道光十三年时宪书月孛与之相差约 0.3°
    MeanApogee,
    /// 瞬时远地点（由月视位置速度求偏心率矢量）
    TrueApogee,
}

/// 紫炁
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZiQiRule {
    /// 清时宪书：自道光十三年七政经纬躔度时宪书逐旬紫炁宿度反推之匀速运动（周期 28 年、历元黄经见 [`ZIQI_SHIXIAN`]）。
    /// 推法原文未见，属据实物反推。缺省
    ShiXian,
    /// 不用紫炁（汤若望新法删之；《明史》卷三十一「天行無紫氣……知為妄增」）
    None,
}

/// 时宪书紫炁：(历元 JD(TT), 该时刻视黄经 °, 周期 日)。由道光十三年（1833）正月至四月逐旬宿度反推（见文档）
pub const ZIQI_SHIXIAN: (f64, f64, f64) = (2390598.5, 204.10, 28.0 * 365.2422);

/// 命宫之法
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MingRule {
    /// 太阳所躔之宫加生时，卯上所临为命宫（《星學大成》卷一）。缺省
    SunOnHour,
    /// 天文上升点（东方地平与黄道之交）所在之宫，需观测地经纬度
    Ascendant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QizhengOptions {
    pub luo_ji: LuoJi,
    pub node: NodeKind,
    pub bei: BeiKind,
    pub zi_qi: ZiQiRule,
    pub ming: MingRule,
}

impl Default for QizhengOptions {
    fn default() -> Self {
        Self { luo_ji: LuoJi::JiAscending, node: NodeKind::True, bei: BeiKind::MeanApogee, zi_qi: ZiQiRule::ShiXian, ming: MingRule::SunOnHour }
    }
}

/// 一曜所在
#[derive(Clone, Debug, PartialEq)]
pub struct YaoPos {
    pub name: &'static str,
    /// 当日视黄经（°，0—360）
    pub lon: Real,
    /// 视黄纬（°）；四余为 None
    pub lat: Option<Real>,
    /// 所在宫（地支序）与入宫度（°，0—30）
    pub gong: u8,
    pub gong_deg: Real,
    /// 所在宿（0 角 … 27 轸）与入宿度（°，黄道宿度）；未给距星时为 None
    pub xiu: Option<(usize, Real)>,
    /// 逆行（七政按一日黄经变化判；四余之罗计恒逆，孛炁恒顺）
    pub retro: bool,
}

/// 命盘
#[derive(Clone, Debug)]
pub struct QizhengChart {
    pub yao: Vec<YaoPos>,
    /// 命宫（地支序）
    pub ming: u8,
    /// 十二宫：下标为地支，值为宫名
    pub palaces: [&'static str; 12],
}

fn deg(x: Real) -> Real {
    x * real(180.0 / core::f64::consts::PI)
}
fn rad(x: Real) -> Real {
    x * real(core::f64::consts::PI / 180.0)
}
fn norm360(x: Real) -> Real {
    deg(rad(x).wrap_to_2pi())
}

/// 黄经（°）所在之宫（地支序）：以中气分宫，0°—30° 戌宫，逆支而行
pub fn gong_of(lon: Real) -> (u8, Real) {
    let l = norm360(lon);
    let k = (l / real(30.0)).to_i32_floor();
    (((10 - k).rem_euclid(12)) as u8, l - real(30.0) * real(k))
}

/// 儒略世纪数（TT，J2000 起）
fn centuries(t: TimePoint) -> Real {
    (t.to_scale(TimeScale::TT).jd - real(2451545.0)) / real(36525.0)
}

/// 月球平升交点黄经（°，平春分点）：Meeus 式 47.7
pub fn mean_ascending_node(t: TimePoint) -> Real {
    let c = centuries(t);
    norm360(real(125.0445479) - real(1934.1362891) * c + real(0.0020754) * c * c + c * c * c / real(467441.0) - c * c * c * c / real(60616000.0))
}

/// 月球平近地点黄经（°）：Meeus 式 47.7；平远地点 = 此 + 180°
pub fn mean_perigee(t: TimePoint) -> Real {
    let c = centuries(t);
    norm360(real(83.3532465) + real(4069.0137287) * c - real(0.01032) * c * c - c * c * c / real(80053.0) + c * c * c * c / real(18999000.0))
}

/// 黄经章动（°）
fn delta_psi(t: TimePoint) -> Real {
    deg(nutation_for_model(centuries(t), NutationModel::IAU2000A).0.rad())
}

/// 真黄赤交角（rad）
fn true_obliquity(t: TimePoint) -> Real {
    let c = centuries(t);
    mean_obliquity(c).rad() + nutation_for_model(c, NutationModel::IAU2000A).1.rad()
}

/// 月之视位置（当日真黄道直角坐标，单位：地球半径不论，取 距离 m）
fn moon_vec<P: EphemerisProvider>(eph: &P, t: TimePoint, opts: &ApparentPipelineOptions) -> [Real; 3] {
    let (lon, lat, r) = body_apparent_ecliptic(eph, Body::Moon, t, opts);
    let (l, b, d) = (lon.rad(), lat.rad(), r.meters());
    [d * b.cos() * l.cos(), d * b.cos() * l.sin(), d * b.sin()]
}

fn cross(a: [Real; 3], b: [Real; 3]) -> [Real; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// 月之瞬时升交点与远地点（当日真黄道视黄经，°）：由前后各半小时的视位置差分得速度，
/// 角动量 h = r × v 之黄道投影定升交点（n = ẑ × h），偏心率矢量 e = v × h / μ − r̂ 之反向为远地点。
pub fn osculating_node_apogee<P: EphemerisProvider>(eph: &P, t: TimePoint, opts: &ApparentPipelineOptions) -> (Real, Real) {
    let dt = real(1.0 / 48.0);
    let jd = t.to_scale(TimeScale::TT).jd;
    let p0 = moon_vec(eph, TimePoint::new(TimeScale::TT, jd - dt), opts);
    let p1 = moon_vec(eph, TimePoint::new(TimeScale::TT, jd + dt), opts);
    let r = moon_vec(eph, t, opts);
    let secs = real(2.0 * 86400.0) * dt;
    let v = [(p1[0] - p0[0]) / secs, (p1[1] - p0[1]) / secs, (p1[2] - p0[2]) / secs];
    let h = cross(r, v);
    let node = norm360(deg(h[0].atan2(-h[1])));
    // μ = G(M⊕ + M☾)
    let mu = real(3.986004418e14 * (1.0 + 1.0 / 81.30056));
    let vh = cross(v, h);
    let rn = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
    let e = [vh[0] / mu - r[0] / rn, vh[1] / mu - r[1] / rn, vh[2] / mu - r[2] / rn];
    // e 指近地点；远地点反向。取其黄道投影之黄经
    let apogee = norm360(deg(e[1].atan2(e[0])) + real(180.0));
    (node, apogee)
}

/// 二十八宿距星的视黄经（°）：由距星视赤经赤纬与真黄赤交角求
pub fn mansion_longitudes(frame: &MansionFrame, t: TimePoint) -> [Real; 28] {
    let eps = true_obliquity(t);
    core::array::from_fn(|i| deg(ecliptic_longitude(frame.ra[i], frame.dec[i], eps)))
}

/// 黄道宿度：黄经 `lon`（°）所在之宿（西侧最近之距星）与入宿度（°）
pub fn xiu_of(mansion_lons: &[Real; 28], lon: Real) -> (usize, Real) {
    let mut best = (0usize, real(f64::MAX));
    for (i, m) in mansion_lons.iter().enumerate() {
        let d = norm360(lon - *m);
        if d < best.1 {
            best = (i, d);
        }
    }
    best
}

/// 命宫（地支序）：太阳所躔之宫 `sun_gong` 加生时 `hour_zhi`，卯上所临之宫
pub fn ming_sun_on_hour(sun_gong: u8, hour_zhi: u8) -> u8 {
    ((sun_gong as i32 + 3 - hour_zhi as i32).rem_euclid(12)) as u8
}

/// 天文上升点黄经（°）：地方恒星时 `lst`（°）、纬度 `lat`（°）、真黄赤交角 `eps`（rad）
pub fn ascendant(lst_deg: Real, lat_deg: Real, eps: Real) -> Real {
    let th = rad(lst_deg);
    let phi = rad(lat_deg);
    // 黄道与地平之交有二（升点、降点）：tan λ = −cos θ ／（sin ε tan φ + cos ε sin θ）；取在东方者（时角 sin H < 0）
    let l1 = (-th.cos()).atan2(eps.sin() * phi.tan() + eps.cos() * th.sin());
    let east = |l: Real| {
        let ra = (l.sin() * eps.cos()).atan2(l.cos());
        (th - ra).sin() < real(0.0)
    };
    let l = if east(l1) { l1 } else { l1 + real(core::f64::consts::PI) };
    norm360(deg(l))
}

/// 排七政四余。`t` 为时刻；`frame` 为同历元二十八宿距星视位置（None 则不算宿度）；`hour_zhi` 为生时地支；
/// `lst_lat` 为（地方视恒星时 °，纬度 °），仅 [`MingRule::Ascendant`] 用。
pub fn chart<P: EphemerisProvider>(
    eph: &P,
    t: TimePoint,
    frame: Option<&MansionFrame>,
    hour_zhi: u8,
    lst_lat: Option<(Real, Real)>,
    o: &QizhengOptions,
    opts: &ApparentPipelineOptions,
) -> QizhengChart {
    let mlon = frame.map(|f| mansion_longitudes(f, t));
    let jd = t.to_scale(TimeScale::TT).jd;
    let mk = |name: &'static str, lon: Real, lat: Option<Real>, retro: bool| {
        let (gong, gong_deg) = gong_of(lon);
        YaoPos { name, lon: norm360(lon), lat, gong, gong_deg, xiu: mlon.as_ref().map(|m| xiu_of(m, lon)), retro }
    };
    let bodies = [
        Body::Sun,
        Body::Moon,
        Body::Planet(Planet::Mercury),
        Body::Planet(Planet::Venus),
        Body::Planet(Planet::Mars),
        Body::Planet(Planet::Jupiter),
        Body::Planet(Planet::Saturn),
    ];
    let mut yao = Vec::with_capacity(11);
    for (k, b) in bodies.iter().enumerate() {
        let (lon, lat, _) = body_apparent_ecliptic(eph, *b, t, opts);
        let (lon2, _, _) = body_apparent_ecliptic(eph, *b, TimePoint::new(TimeScale::TT, jd + real(1.0)), opts);
        let dl = (lon2.rad() - lon.rad()).wrap_to_signed_pi();
        yao.push(mk(YAO[k], deg(lon.rad()), Some(deg(lat.rad())), dl < real(0.0)));
    }
    // 四余：平根数在平春分点，加黄经章动归当日真春分点，与七政同一基准
    let dpsi = delta_psi(t);
    let (asc_node, apogee) = match (o.node, o.bei) {
        (NodeKind::Mean, BeiKind::MeanApogee) => (mean_ascending_node(t) + dpsi, mean_perigee(t) + real(180.0) + dpsi),
        _ => {
            let (n, a) = osculating_node_apogee(eph, t, opts);
            (
                if o.node == NodeKind::Mean { mean_ascending_node(t) + dpsi } else { n },
                if o.bei == BeiKind::MeanApogee { mean_perigee(t) + real(180.0) + dpsi } else { a },
            )
        }
    };
    let desc_node = asc_node + real(180.0);
    let (luo, ji) = match o.luo_ji {
        LuoJi::JiAscending => (desc_node, asc_node),
        LuoJi::LuoAscending => (asc_node, desc_node),
    };
    yao.push(mk("羅睺", luo, None, true));
    yao.push(mk("計都", ji, None, true));
    yao.push(mk("月孛", apogee, None, false));
    if o.zi_qi == ZiQiRule::ShiXian {
        let (j0, l0, p) = ZIQI_SHIXIAN;
        yao.push(mk("紫炁", real(l0) + real(360.0) * (jd - real(j0)) / real(p), None, false));
    }
    let sun_gong = yao[0].gong;
    let ming = match (o.ming, lst_lat) {
        (MingRule::Ascendant, Some((lst, lat))) => gong_of(ascendant(lst, lat, true_obliquity(t))).0,
        _ => ming_sun_on_hour(sun_gong, hour_zhi),
    };
    let palaces = core::array::from_fn(|z| PALACES[((ming as i32 - z as i32).rem_euclid(12)) as usize]);
    QizhengChart { yao, ming, palaces }
}

/// 宿名
pub fn xiu_name(i: usize) -> &'static str {
    MANSIONS[i % 28]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gong_boundaries() {
        assert_eq!(gong_of(real(0.5)).0, 10); // 戌 降婁
        assert_eq!(gong_of(real(330.85)).0, 11); // 亥：道光十三年正月初一日躔亥宫初度五十一分
        assert_eq!(gong_of(real(275.0)).0, 1); // 丑 星紀（冬至后）
        assert!((gong_of(real(330.85)).1.as_f64() - 0.85).abs() < 1e-9);
    }

    #[test]
    fn ming_rule() {
        // 卯时生，命宫即太阳宫；午时生，命宫为太阳宫逆三（天文上升点约在太阳东 90°）
        assert_eq!(ming_sun_on_hour(11, 3), 11);
        assert_eq!(ming_sun_on_hour(11, 6), 8);
        assert_eq!(ming_sun_on_hour(0, 0), 3);
    }

    #[test]
    fn ascendant_sanity() {
        // 赤道上、恒星时 0（春分点上中天）：上升点为黄经 90°
        let eps = rad(real(23.44));
        assert!((ascendant(real(0.0), real(0.0), eps).as_f64() - 90.0).abs() < 1e-9);
        // 恒星时 90°（夏至点附近上中天）：上升点为秋分点 180°
        assert!((ascendant(real(90.0), real(0.0), eps).as_f64() - 180.0).abs() < 1e-9);
        // 北纬 40°、恒星时 0：上升点在巨蟹宫一带（90° 以上）
        assert!(ascendant(real(0.0), real(40.0), eps).as_f64() > 90.0);
    }

    /// 《大清道光十三年七政經緯躔度時憲書》逐旬四余宿度（转录见 examples/qizheng_1833.rs、doc/folk/qizheng.md）：
    /// 缺省（计都升交、真交点、平远地点）下罗计、月孛差 30′ 内；正月初一太阳差 2′ 内
    #[test]
    fn daoguang13_shixianshu() {
        use crate::astronomy::ephemeris::elpmpp02::{load_all, Elpmpp02Correction};
        use crate::astronomy::ephemeris::vsop87_patch::PatchTarget;
        use crate::astronomy::ephemeris::{load_earth_vsop87_corrected_from_repo, load_planet_vsop87_corrected_from_repo};
        use crate::astronomy::frame::nutation::try_init_full_nutation_from_repo;
        use crate::astronomy::pipeline::{Vsop87Elp, Vsop87Planets};
        use crate::astronomy::sky_culture::chinese::load_lunar_mansion_systems_from_repo;
        use crate::astronomy::stars::load_hipparcos_bright_from_repo;
        let _ = try_init_full_nutation_from_repo();
        let earth = load_earth_vsop87_corrected_from_repo(PatchTarget::DEFAULT).unwrap();
        let planets: Vec<_> = [Planet::Mercury, Planet::Venus, Planet::Mars, Planet::Jupiter, Planet::Saturn]
            .iter()
            .map(|&p| (p, load_planet_vsop87_corrected_from_repo(p, PatchTarget::DEFAULT).unwrap()))
            .collect();
        let elp = load_all(&crate::repo::default_loader(), crate::repo::paths::ELPMPP02, Elpmpp02Correction::DEFAULT).unwrap();
        let eph = Vsop87Elp { planets: Vsop87Planets { earth: &earth, planets: planets.iter().map(|(p, v)| (*p, v)).collect() }, moon: &elp };
        let cat = load_hipparcos_bright_from_repo().unwrap();
        let sys = load_lunar_mansion_systems_from_repo().unwrap().into_iter().find(|s| s.id == "qing").unwrap();
        let opts = ApparentPipelineOptions::pipeline_default();
        // (距正月初一日数, 月孛, 罗睺, 计都)：(宿, 度+分/60)
        let book: [(i64, (&str, f64), (&str, f64), (&str, f64)); 6] = [
            (0, ("女", 4.733), ("斗", 15.733), ("井", 20.6)),
            (20, ("女", 7.017), ("斗", 14.517), ("井", 19.383)),
            (39, ("女", 9.15), ("斗", 12.717), ("井", 17.583)),
            (59, ("女", 11.367), ("斗", 10.583), ("井", 15.45)),
            (79, ("虚", 1.867), ("斗", 8.767), ("井", 13.633)),
            (108, ("虚", 4.967), ("斗", 7.35), ("井", 12.217)),
        ];
        let jd0 = 2390599.5 - 116.4 / 360.0;
        for (dd, bei, luo, ji) in book {
            let t = TimePoint::new(TimeScale::TT, real(jd0 + dd as f64));
            let frame = MansionFrame::new(&eph, &cat, &sys, t, &opts).unwrap();
            let c = chart(&eph, t, Some(&frame), 0, None, &QizhengOptions::default(), &opts);
            for (name, (x, d)) in [("月孛", bei), ("羅睺", luo), ("計都", ji)] {
                let y = c.yao.iter().find(|y| y.name == name).unwrap();
                let (xi, du) = y.xiu.unwrap();
                assert_eq!(MANSIONS[xi], x, "{dd} {name}");
                assert!((du.as_f64() - d).abs() * 60.0 < 30.0, "{dd} {name}: {} vs {d}", du.as_f64());
            }
            if dd == 0 {
                assert!((c.yao[0].lon.as_f64() - (330.0 + 51.0 / 60.0)).abs() * 60.0 < 2.0);
                assert_eq!(c.yao[0].gong, 11);
            }
        }
    }

    #[test]
    fn mean_elements_1833() {
        // 道光十三年正月初一（1833-02-19，JD 2390598.5）
        let t = TimePoint::new(TimeScale::TT, real(2390598.5));
        assert!((mean_ascending_node(t).as_f64() - 112.40).abs() < 0.01);
        assert!(((mean_perigee(t).as_f64() + 180.0).rem_euclid(360.0) - 313.67).abs() < 0.01);
    }
}
