//! ΔT = TT − UT1 模型体系：多个模型并存，各自给出估计值、不确定度 σ 与其所隐含的月球潮汐加速度 ṅ。
//!
//! 古代天象（日月食局地可见性等）的主要误差来源是 ΔT。ΔT 是从史料日月食反推的，反推时用的月历表有一个潮汐加速度 ṅ；
//! 若计算天象所用月历表的 ṅ 与之不同，须配套修正，否则古代会产生分钟级系统误差。
//!
//! - [`MorrisonStephenson2021`]：Morrison, Stephenson, Hohenkerk & Zawilski (2021) 样条，-720～2019 按作者系数，范围外按作者
//!   逐年表及原论文长期抛物线；数据原样保存在 `vendor/delta_t/morrison_stephenson_2021/`（CC BY 4.0）。古代天象默认推荐。
//! - [`EspenakMeeus2006`]：NASA《五千年日食典》所用多项式（[`super::espenak_meeus_2006`]），与 NASA 日食典对照时使用。
//!
//! 配套以 **DE431 为枢纽**（doc/astronomy/model-pairing.md）：ΔT 模型先按其 ṅ 换算到「以 DE431 月亮为准」（MS2021 本就配
//! DE430/431，不变；Espenak–Meeus 2006 假定 ṅ = −26，按 ṅ 差换到 −25.82），所用月历表再按其相对 DE431 的实测月亮黄经差
//! Δλ(T) 修正（[`delta_t_for_lunar_offset`]）。只按 ṅ 差的二次修正（[`delta_t_for_ephemeris`]，NASA 做法）保留为选项。
//! 1955 年以后 ΔT 由原子时直接测得，与月历表无关，两种修正都只用于 1955 年以前。
//!
//! 时间尺度换算（[`super::delta_t`]，TT↔UT1，及 1972 年前以 UT1 近似 UTC）也用 MS2021，按 [`time_scale_lunar_offset`]
//! （缺省为缺省月历表 ELPMPP02(DE431)，修正 < 0.5 s）配套，与天象检验同一 ΔT。Espenak–Meeus 2006 只用于与 NASA 典对照。
//! 设计见 doc/astronomy/celestial-phenomena.md。

use std::sync::{OnceLock, RwLock};

use crate::math::real::{real, real_const, Real, RealOps};
use crate::quantity::duration::Duration;

use super::scales::{espenak_meeus_2006 as espenak_meeus_delta_t, jd_to_decimal_year};

/// 月球潮汐加速度 ṅ（角秒/儒略世纪²）的参考值。
pub mod lunar_tidal_acceleration {
    /// JPL DE430：Stephenson, Morrison & Hohenkerk (2016) §2 所述其 ΔT 应配合的值（激光测月，±0.03）。
    pub const DE430: f64 = -25.82;
    /// Morrison & Stephenson (2004) 及据其拟合的 Espenak–Meeus 多项式所假定的值（NASA ΔT 多项式说明页）。
    pub const MORRISON_STEPHENSON_2004: f64 = -26.0;
    /// ELP-2000/82（NASA 日食典所用月历表）。
    pub const ELP2000_82: f64 = -25.858;
    /// 本仓库月历表 ELP/MPP02(405)（DE405 拟合 + Table 6 长期项修正，接近 DE406）。
    ///
    /// 论文未直接给出数值，按论文推得：二次项系数 W₁⁽²⁾ 的潮汐部分等于 ṅ/2（Chapront & Francou 2003 §4.1.1 Remark），
    /// 而引力部分在 DE405 拟合与 LLR 拟合中相同，故 ṅ(405) = ṅ(LLR) + 2·(ΔW₁⁽²⁾(405) − ΔW₁⁽²⁾(LLR))
    /// = −25.858 + 2·(−0.03743 + 0.03794) ≈ −25.857；ṅ(LLR) = −25.858 取自 Chapront, Chapront-Touzé & Francou (2002, A&A 387, 700)。
    /// Table 6 只修正 t³、t⁴ 项（前 200 年合计约 0.4″），不改 ṅ。
    ///
    /// **2026-10-26 实测否定**：与 DE406、DE431 直接比对，ELPMPP02（DE406 改正版）的有效 ṅ 与二者相差都不到 0.01″/cy²
    /// （见 [`super::lunar_longitude_offset::ELPMPP02_DE406_MINUS_DE431`]），即约 −25.83，而非此处推得的 −25.857；
    /// 配合 MS2021 时宜用 [`super::delta_t_for_lunar_offset`]。此常数保留作对照。
    pub const ELPMPP02_DE405: f64 = -25.857;
}

/// ΔT 估计：值与（若模型提供）1σ 不确定度。
#[derive(Clone, Copy, Debug)]
pub struct DeltaTEstimate {
    pub value: Duration,
    pub sigma: Option<Duration>,
}

/// ΔT 模型。年份为天文纪年十进制年（含公元 0 年）。
pub trait DeltaTModel {
    /// 模型名（用于结果标注）。
    fn name(&self) -> &'static str;
    /// 十进制年 → ΔT 估计。
    fn estimate_at_year(&self, year: Real) -> DeltaTEstimate;
    /// 该模型推导时所用（或所假定）的月球潮汐加速度 ṅ（″/cy²）。
    fn lunar_tidal_acceleration(&self) -> Real;
    /// TT 儒略日 → ΔT 估计（未做 ṅ 配套修正）。
    fn estimate(&self, jd_tt: Real) -> DeltaTEstimate {
        self.estimate_at_year(jd_to_decimal_year(jd_tt))
    }
}

/// ΔT 配套修正只用于此年以前：1955.5 起有原子时，ΔT 由地球自转直接测得，与月历表无关；此前的 ΔT（古代日月食、
/// 1600—1955 年月掩星）都经月历表反推。
pub const PAIRING_END_YEAR: f64 = 1955.0;

/// 月历表 ṅ 与 ΔT 模型 ṅ 不同时加到 ΔT 上的修正：c = −0.91072·(ṅ_历表 − ṅ_ΔT)·u²，u = (年 − 1955)/100；1955 年以后为 0。
///
/// 依据：NASA ΔT 多项式说明页给出的特例 c = −0.000012932·(y−1955)²（ṅ_ΔT = −26，ṅ_历表 = −25.858），
/// 即 −0.91072 × 0.142 × u²。
pub fn tidal_correction(year: Real, ndot_ephemeris: Real, ndot_model: Real) -> Duration {
    if year >= real(PAIRING_END_YEAR) {
        return Duration::in_seconds(real(0.0));
    }
    let u = (year - 1955.0) / 100.0;
    Duration::in_seconds(real_const(-0.91072) * (ndot_ephemeris - ndot_model) * u.clone() * u)
}

/// 给定月历表 ṅ，返回配套修正后的 ΔT 估计（σ 不变）。
pub fn delta_t_for_ephemeris(model: &dyn DeltaTModel, jd_tt: Real, ndot_ephemeris: Real) -> DeltaTEstimate {
    let year = jd_to_decimal_year(jd_tt.clone());
    let est = model.estimate(jd_tt);
    let c = tidal_correction(year, ndot_ephemeris, model.lunar_tidal_acceleration());
    DeltaTEstimate { value: est.value.add(c), sigma: est.sigma }
}

/// 月亮恒星平运动 n（″/s）：ELP W₁ 一次项 1732559343.736″/儒略世纪。0.5/n = 0.91072，即 [`tidal_correction`] 的系数。
pub const MOON_MEAN_MOTION_ARCSEC_PER_S: f64 = 1732559343.736 / (36525.0 * 86400.0);

/// 月历表相对某 ΔT 模型所配历表的月亮黄经差 Δλ(T) = Σ c_k T^k（角秒；T 为 J2000 起儒略世纪，TT），由两历表直接比对拟合。
///
/// 比只按 ṅ 修正（[`tidal_correction`]，只含 T² 项）更完整：常数、平运动之差也一并计入。配套修正 δΔT = −Δλ(T)/n
/// （n 见 [`MOON_MEAN_MOTION_ARCSEC_PER_S`]）：月历表若使月亮黄经超前 Δλ，同一 UT 时刻的月亮位置要对上 ΔT 反推时所用历表，
/// TT 须提前 Δλ/n。
#[derive(Clone, Copy, Debug)]
pub struct LunarLongitudeOffset {
    /// 标注，如「ELPMPP02(DE406) − DE431」。
    pub name: &'static str,
    /// 多项式系数 c₀、c₁、c₂…（″、″/cy、″/cy²…）
    pub coefficients: &'static [f64],
    /// 拟合所用年份范围（天文纪年），范围外为外推。
    pub fitted_years: (f64, f64),
}

impl LunarLongitudeOffset {
    /// Δλ(T)，角秒。
    pub fn arcsec_at(&self, jd_tt: Real) -> Real {
        let t = (jd_tt - real(2451545.0)) / real(36525.0);
        let mut sum = real(0.0);
        let mut tk = real(1.0);
        for c in self.coefficients {
            sum = sum + real(*c) * tk.clone();
            tk = tk * t.clone();
        }
        sum
    }

    /// 配套修正 δΔT = −Δλ/n；1955 年以后为 0（见 [`PAIRING_END_YEAR`]）。
    pub fn delta_t_correction(&self, jd_tt: Real) -> Duration {
        if jd_to_decimal_year(jd_tt.clone()) >= real(PAIRING_END_YEAR) {
            return Duration::in_seconds(real(0.0));
        }
        Duration::in_seconds(-self.arcsec_at(jd_tt) / real(MOON_MEAN_MOTION_ARCSEC_PER_S))
    }
}

/// 实测的月历表黄经差，一律相对枢纽 DE431。
pub mod lunar_longitude_offset {
    use super::LunarLongitudeOffset;

    /// DE431 自身（枢纽）：差为 0。
    pub const DE431: LunarLongitudeOffset = LunarLongitudeOffset { name: "DE431 − DE431", coefficients: &[], fitted_years: (-13000.0, 17000.0) };

    /// ELPMPP02 取拟合 DE431 的常数集并加残差改正（缺省，`data/fit/elpmpp02-de431.txt`、`elpmpp02-de431-patch.txt`）相对 DE431，
    /// 测法同下（2026-11-06）：三次拟合残差 RMS 0.18″；每 400 年分段均值都在 ±0.16″ 内，配套量各年代 0.0 s——缺省月历表与 MS2021
    /// 所配月亮实为同一月亮。（未加残差改正时为 [−0.0804, 0.04157, 0.002566, 0.0000348]，RMS 0.40″。）
    pub const ELPMPP02_DE431_MINUS_DE431: LunarLongitudeOffset = LunarLongitudeOffset {
        name: "ELPMPP02(DE431) − DE431",
        coefficients: &[-0.0113, 0.00243, 0.000186, 0.0000026],
        fitted_years: (-2900.0, 2900.0),
    };

    /// JPL DE406 内核相对 DE431，测法同下（2026-11-06）：三次拟合残差 RMS 0.27″，二次项 Δṅ = −0.0080″/cy²；每 400 年分段均值
    /// 前 2900 年 −5.9″。用 DE406 内核算古代日月食时配此修正。
    pub const DE406_MINUS_DE431: LunarLongitudeOffset = LunarLongitudeOffset {
        name: "DE406 − DE431",
        coefficients: &[-0.0092, -0.01814, 0.000230, 0.0000699],
        fitted_years: (-2900.0, 2900.0),
    };

    /// JPL DE441 内核相对 DE431，测法同下（2026-11-06）：三次拟合残差 RMS 5.92″（周期差，见 [`ELPMPP02_DE441_MINUS_DE431`]），
    /// 二次项 Δṅ = −0.38″/cy²；每 400 年分段均值前 2900 年 −328″。用 DE441 内核算古代日月食并配 MS2021 时须配此修正。
    pub const DE441_MINUS_DE431: LunarLongitudeOffset = LunarLongitudeOffset {
        name: "DE441 − DE431",
        coefficients: &[0.1276, -0.00655, 0.000959, 0.0031785],
        fitted_years: (-2900.0, 2900.0),
    };

    /// 本仓库月历表 ELPMPP02 取论文 DE405 + Table 6 常数集并加对 DE406 的残差改正（`data/fit/elpmpp02-de406-patch.txt`）相对
    /// JPL DE431（DE430 的长时段版，Morrison–Stephenson 2016/2021 ΔT 所配历表）。
    ///
    /// 由 `apparent::tests_lunar_ndot::elpmpp02_effective_ndot_vs_de406_de441` 在前 2900—2900 年 3000 个随机时刻的月亮视黄经差
    /// 三次拟合得到（2026-11-06，残差 RMS 0.31″）：加残差改正后与 DE406 内核相对 DE431 之差（[`DE406_MINUS_DE431`]）几乎相同。
    /// 配套量前 2000 年 +6.1 s、前 700 年 +1.3 s、前 200 年 +0.5 s。按 ṅ = −25.857（[`super::lunar_tidal_acceleration::ELPMPP02_DE405`]，
    /// 据论文推得）对 −25.82 的二次修正为前 700 年 +23.8 s、前 200 年 +15.6 s，系高估。
    /// 旧值：幅角修正前（2026-10-26，二次）[−0.013, −0.0457, −0.00327]；未加残差改正时（三次）[−0.3089, 0.03750, 0.003725, 0.0001151]。
    pub const ELPMPP02_DE406_MINUS_DE431: LunarLongitudeOffset = LunarLongitudeOffset {
        name: "ELPMPP02(DE406) − DE431",
        coefficients: &[-0.0218, -0.01583, 0.000410, 0.0000725],
        fitted_years: (-2900.0, 2900.0),
    };

    /// ELPMPP02 取拟合 DE441 的常数集并加残差改正（`data/fit/elpmpp02-de441.txt`、`elpmpp02-de441-patch.txt`）相对 DE431，测法同上
    /// （2026-11-06）：与 DE441 内核相对 DE431 之差（[`DE441_MINUS_DE431`]）几乎相同。三次拟合残差 RMS 5.92″，四至八次仍不减：余下的是
    /// 两历表近地点、升交点长期运动不同引起的周期差（振幅随 |T| 增大），不是平黄经之差，ΔT 无从配套。每 400 年分段均值：前 2900 年
    /// −328″、前 900 年 −62″、前 100 年 −21″、700 年 −4.0″、1500 年 −0.2″、1900 年 +0.04″。配套量前 700 年 +112 s、前 200 年 +60 s、
    /// 公元 0 年 +45 s、1000 年 +5.3 s：DE441 的月亮与 MS2021 所配 DE431 差得多，换用 DE441 常数集时必须配此修正。
    pub const ELPMPP02_DE441_MINUS_DE431: LunarLongitudeOffset = LunarLongitudeOffset {
        name: "ELPMPP02(DE441) − DE431",
        coefficients: &[0.1169, -0.00420, 0.001134, 0.0031809],
        fitted_years: (-2900.0, 2900.0),
    };
}

/// 按枢纽 DE431 配套修正 ΔT（σ 不变）：先按模型的 ṅ 换到 DE431（ṅ = −25.82，MS2021 为 0），再按 `offset`
/// （须是「所用月历表 − DE431」，见 [`lunar_longitude_offset`] 与 `Elpmpp02Correction::lunar_offset_vs_de431`）修正。
pub fn delta_t_for_lunar_offset(model: &dyn DeltaTModel, jd_tt: Real, offset: &LunarLongitudeOffset) -> DeltaTEstimate {
    let year = jd_to_decimal_year(jd_tt.clone());
    let est = model.estimate(jd_tt.clone());
    let to_hub = tidal_correction(year, real(lunar_tidal_acceleration::DE430), model.lunar_tidal_acceleration());
    DeltaTEstimate { value: est.value.add(to_hub).add(offset.delta_t_correction(jd_tt)), sigma: est.sigma }
}

static TIME_SCALE_LUNAR_OFFSET: RwLock<LunarLongitudeOffset> = RwLock::new(lunar_longitude_offset::ELPMPP02_DE431_MINUS_DE431);

/// 时间尺度换算（TT↔UT1）所配的月历表（相对 DE431 的黄经差）。缺省为缺省月历表 ELPMPP02(DE431)。
pub fn time_scale_lunar_offset() -> LunarLongitudeOffset {
    *TIME_SCALE_LUNAR_OFFSET.read().unwrap()
}

/// 换用别的月历表时一并设置（如 wasm 选 DE441 常数集），使 TT↔UT1 与所用月亮配套。
pub fn set_time_scale_lunar_offset(offset: LunarLongitudeOffset) {
    *TIME_SCALE_LUNAR_OFFSET.write().unwrap() = offset;
}

/// 时间尺度换算所用 ΔT：Morrison–Stephenson 2021（本就配 DE431，无须 ṅ 换算）按 [`time_scale_lunar_offset`] 配套。
///
/// 取 MS2021 而非 Espenak–Meeus 2006：后者假定 ṅ = −26、须换算才配得上本项目的月亮，且 2019 年以后外推偏大（2026 年 75 s、
/// 2100 年 203 s，MS2021 为 68 s、80 s）。见 doc/astronomy/model-pairing.md。
pub fn time_scale_delta_t(jd_tt: Real) -> Duration {
    delta_t_for_lunar_offset(&MorrisonStephenson2021, jd_tt, &time_scale_lunar_offset()).value
}

// ---------------------------------------------------------------------------------------------------------------------

/// NASA 五千年日食典 ΔT 多项式（Espenak & Meeus 2006，据 Morrison & Stephenson 2004）。不提供 σ。
#[derive(Clone, Copy, Debug, Default)]
pub struct EspenakMeeus2006;

impl DeltaTModel for EspenakMeeus2006 {
    fn name(&self) -> &'static str {
        "Espenak–Meeus 2006"
    }
    fn estimate_at_year(&self, year: Real) -> DeltaTEstimate {
        // scales::delta_t 以 TT 儒略日为参数，按 365.25 日/年与 jd_to_decimal_year 互逆
        let jd = (year - 2000.0) * 365.25 + crate::astronomy::constant::J2000;
        DeltaTEstimate { value: espenak_meeus_delta_t(jd), sigma: None }
    }
    fn lunar_tidal_acceleration(&self) -> Real {
        real(lunar_tidal_acceleration::MORRISON_STEPHENSON_2004)
    }
}

// ---------------------------------------------------------------------------------------------------------------------

const MS2021_SPLINE_TXT: &str =
    include_str!("../../../../../vendor/delta_t/morrison_stephenson_2021/Table-S15.2020.txt");
const MS2021_TABLE_TXT: &str =
    include_str!("../../../../../vendor/delta_t/morrison_stephenson_2021/Table-DT-lod4500yrs.2020.txt");

/// 样条一段：[k0, k1] 年内 ΔT = a0 + a1 t + a2 t² + a3 t³，t = (Y − k0)/(k1 − k0)。
struct SplineRow {
    k0: f64,
    k1: f64,
    a: [f64; 4],
}

/// 作者逐年表一行：年、ΔT、ΔT 误差（秒）。
struct TableRow {
    year: f64,
    dt: f64,
    err: f64,
}

struct Ms2021Data {
    spline: Vec<SplineRow>,
    table: Vec<TableRow>,
}

fn parse_numbers(line: &str) -> Vec<f64> {
    line.split_whitespace().filter_map(|w| w.parse::<f64>().ok()).collect()
}

fn ms2021_data() -> &'static Ms2021Data {
    static DATA: OnceLock<Ms2021Data> = OnceLock::new();
    DATA.get_or_init(|| {
        // 数据行：「i  K_i  K_{i+1}  a0 a1 a2 a3」共 7 个数；说明行、分隔行解析不出 7 个数而被跳过
        let spline: Vec<SplineRow> = MS2021_SPLINE_TXT
            .lines()
            .map(parse_numbers)
            .filter(|v| v.len() == 7)
            .map(|v| SplineRow { k0: v[1], k1: v[2], a: [v[3], v[4], v[5], v[6]] })
            .collect();
        // 数据行：「Year DT DTerr lod loderr」共 5 个数
        let table: Vec<TableRow> = MS2021_TABLE_TXT
            .lines()
            .map(parse_numbers)
            .filter(|v| v.len() == 5)
            .map(|v| TableRow { year: v[0], dt: v[1], err: v[2] })
            .collect();
        assert!(spline.len() >= 50 && table.len() >= 100, "Morrison–Stephenson 2021 ΔT 数据解析失败");
        Ms2021Data { spline, table }
    })
}

/// 原论文式 (4.1) 长期抛物线 ΔT = −320.0 + 32.5·u²，u = (年 − 1825)/100；系数不确定度 ±0.6。
fn ms_parabola(year: f64) -> (f64, f64) {
    let u = (year - 1825.0) / 100.0;
    (-320.0 + 32.5 * u * u, 0.6 * u * u)
}

/// Morrison, Stephenson, Hohenkerk & Zawilski (2021) ΔT。
///
/// - −720 ～ 2019：作者样条系数（Table-S15.2020）；
/// - 其外、−2000 ～ +2500：作者逐年表（作者外推值）线性插值；
/// - 再外：原论文长期抛物线，平移使在表端连续；
/// - σ：逐年表误差列线性插值；表外取 max(表端误差, 抛物线系数不确定度 0.6·u²)。
#[derive(Clone, Copy, Debug, Default)]
pub struct MorrisonStephenson2021;

impl MorrisonStephenson2021 {
    fn value_seconds(year: f64) -> f64 {
        let d = ms2021_data();
        let (first, last) = (d.spline.first().unwrap(), d.spline.last().unwrap());
        if year >= first.k0 && year <= last.k1 {
            let row = d.spline.iter().find(|r| year >= r.k0 && year <= r.k1).unwrap();
            let t = (year - row.k0) / (row.k1 - row.k0);
            return row.a[0] + t * (row.a[1] + t * (row.a[2] + t * row.a[3]));
        }
        let (t0, tn) = (d.table.first().unwrap(), d.table.last().unwrap());
        if year < t0.year {
            return ms_parabola(year).0 - ms_parabola(t0.year).0 + t0.dt;
        }
        if year > tn.year {
            return ms_parabola(year).0 - ms_parabola(tn.year).0 + tn.dt;
        }
        interpolate(&d.table, year, |r| r.dt)
    }

    fn sigma_seconds(year: f64) -> f64 {
        let d = ms2021_data();
        let (t0, tn) = (d.table.first().unwrap(), d.table.last().unwrap());
        if year < t0.year {
            return t0.err.max(ms_parabola(year).1);
        }
        if year > tn.year {
            return tn.err.max(ms_parabola(year).1);
        }
        interpolate(&d.table, year, |r| r.err)
    }
}

fn interpolate(table: &[TableRow], year: f64, f: impl Fn(&TableRow) -> f64) -> f64 {
    let i = table.windows(2).position(|w| year >= w[0].year && year <= w[1].year).unwrap();
    let (a, b) = (&table[i], &table[i + 1]);
    let s = (year - a.year) / (b.year - a.year);
    f(a) + s * (f(b) - f(a))
}

impl DeltaTModel for MorrisonStephenson2021 {
    fn name(&self) -> &'static str {
        "Morrison–Stephenson 2021"
    }
    fn estimate_at_year(&self, year: Real) -> DeltaTEstimate {
        let y = year.as_f64();
        DeltaTEstimate {
            value: Duration::in_seconds(real(Self::value_seconds(y))),
            sigma: Some(Duration::in_seconds(real(Self::sigma_seconds(y)))),
        }
    }
    fn lunar_tidal_acceleration(&self) -> Real {
        real(lunar_tidal_acceleration::DE430)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secs(e: DeltaTEstimate) -> f64 {
        e.value.seconds().as_f64()
    }

    /// 月亮平运动与 NASA 二次修正系数一致：0.5/n = 0.91072。
    #[test]
    fn moon_mean_motion_matches_nasa_coefficient() {
        assert!((0.5 / MOON_MEAN_MOTION_ARCSEC_PER_S - 0.91072).abs() < 1e-5);
    }

    /// 黄经差修正：Δλ 只含二次项时与 [`tidal_correction`] 等价；ELPMPP02(DE406) − DE431 实测修正在前 200 年约 +0.5 s。
    #[test]
    fn lunar_offset_correction() {
        // Δλ = ½Δṅ·T²（T 自 J2000）与 tidal_correction（u 自 1955）只差常数与线性项，取 Δṅ = −0.1、年份 −1000 比二次部分
        let off = LunarLongitudeOffset { name: "test", coefficients: &[0.0, 0.0, -0.05], fitted_years: (-3000.0, 3000.0) };
        let jd = 2451545.0 + (-1000.0 - 2000.0) * 365.25;
        let t = (-1000.0 - 2000.0) / 100.0;
        let expect = 0.91072 * 0.1 * t * t;
        assert!((off.delta_t_correction(real(jd)).seconds().as_f64() - expect).abs() < 0.01 * expect.abs());
        let jd = 2451545.0 + (-200.0 - 2000.0) * 365.25;
        let c = lunar_longitude_offset::ELPMPP02_DE406_MINUS_DE431.delta_t_correction(real(jd)).seconds().as_f64();
        assert!((c - 0.5).abs() < 0.1, "{c}");
    }

    /// 样条与作者逐年表（由同一样条生成，取整到 10 s 或更细）一致。
    #[test]
    fn ms2021_spline_reproduces_author_table() {
        let m = MorrisonStephenson2021;
        for (y, dt, tol) in [(-720.0, 20370.0, 10.0), (-600.0, 18470.0, 10.0), (-200.0, 12770.0, 10.0), (0.0, 10440.0, 10.0), (1000.0, 1650.0, 10.0), (2016.0, 68.11, 0.02)] {
            let v = secs(m.estimate_at_year(real(y)));
            assert!((v - dt).abs() <= tol, "{y}: {v} vs {dt}");
        }
        let s = m.estimate_at_year(real(-200.0)).sigma.unwrap().seconds().as_f64();
        assert!((s - 110.0).abs() < 1e-9, "σ(-200) = {s}");
    }

    /// 作者逐年表 −700 年一行作 20250，与同表 −720（20370）、−600（18470）及样条（≈20050）均不协调，疑为 20050 之误；
    /// 本模块在样条范围内只用样条系数。此测试记录该差异，若作者更正数据可据此发现。
    #[test]
    fn ms2021_table_row_minus700_inconsistent_with_spline() {
        let v = secs(MorrisonStephenson2021.estimate_at_year(real(-700.0)));
        assert!((v - 20050.0).abs() < 5.0, "{v}");
        let row = ms2021_data().table.iter().find(|r| r.year == -700.0).unwrap();
        assert_eq!(row.dt, 20250.0);
    }

    /// 样条范围外取作者逐年表；更外用抛物线，表端连续。
    #[test]
    fn ms2021_outside_spline_range() {
        let m = MorrisonStephenson2021;
        assert!((secs(m.estimate_at_year(real(-1500.0))) - 34800.0).abs() < 1e-6);
        assert!((secs(m.estimate_at_year(real(2100.0))) - 80.0).abs() < 1e-6);
        let edge = secs(m.estimate_at_year(real(-2000.0)));
        let beyond = secs(m.estimate_at_year(real(-2000.001)));
        assert!((edge - beyond).abs() < 1.0, "抛物线在表端不连续：{edge} vs {beyond}");
        let s = m.estimate_at_year(real(-3000.0)).sigma.unwrap().seconds().as_f64();
        assert!(s >= 1000.0);
    }

    /// 配套修正公式还原 NASA 给出的特例 c = −0.000012932·(y−1955)²；1955 年以后 ΔT 由原子时测得，不配套，为 0。
    #[test]
    fn tidal_correction_matches_nasa_special_case() {
        let c = tidal_correction(real(2100.0), real(lunar_tidal_acceleration::ELP2000_82), real(lunar_tidal_acceleration::MORRISON_STEPHENSON_2004));
        assert_eq!(c.seconds().as_f64(), 0.0);
        for y in [-1000.0, -200.0, 1000.0, 1900.0] {
            let c = tidal_correction(real(y), real(lunar_tidal_acceleration::ELP2000_82), real(lunar_tidal_acceleration::MORRISON_STEPHENSON_2004));
            let nasa = -0.000012932 * (y - 1955.0) * (y - 1955.0);
            // NASA 系数 0.000012932 为 0.91072×0.142=0.12932224 取整，相对差约 2e-5
            assert!((c.seconds().as_f64() - nasa).abs() < 1e-4 * nasa.abs().max(1.0), "{y}");
        }
    }

    /// Espenak–Meeus 包装与现有全局 delta_t 一致。
    #[test]
    fn espenak_meeus_wraps_existing_polynomial() {
        let v = secs(EspenakMeeus2006.estimate_at_year(real(-200.0)));
        let jd = (real(-200.0) - 2000.0) * 365.25 + crate::astronomy::constant::J2000;
        assert!((v - espenak_meeus_delta_t(jd).seconds().as_f64()).abs() < 1e-9);
    }
}
