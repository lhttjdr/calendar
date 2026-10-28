//! 肉眼点源可见性模型。全部公式由原文逐式实现，不参照任何现成程序；各处可选项并存，缺省由检验结果选定
//! （见 doc/astronomy/celestial-phenomena.md §九 P3「晨见夕伏」）。
//!
//! - 大气质量与消光（Schaefer 1993《Astronomy and the limits of vision》，Vistas Astron. 36, 311）：式 2b（Rozenberg，
//!   整层）、3a（指数分布，标高 H_e）、3b（H_L 处薄层，臭氧）；消光分量式 4b、5a,b、6a，合成式 7、8b。
//! - 天空亮度：
//!   - 散射相函数与月光：Krisciunas & Schaefer 1991（PASP 103, 1033）式 3、8、9、15–21，原文表 2 可逐值复现；
//!     1993 式 14 把三项散射直接相加，作选项保留（[`ScatteringFunction`]）。
//!   - 夜天光、白昼、城市光：1993 式 15a、15d、15e。式 15 中的大气质量 X 取 Krisciunas & Schaefer 1991 式 3
//!     X = (1 − 0.96 sin²Z)^{−1/2}——式 15b、15d 即该文式 15（以太阳星等 −26.74 代月亮），1993 只是转述。
//!   - 晨昏蒙影：1993 式 15c，或 Schaefer 1987（JHA 18, S19）式 1–2 为伏见拟合的含方位差公式（[`TwilightModel`]）。
//!   - 整套替代：Schaefer 1998《To the visual limits》（*Sky & Telescope* 95(5), 57）所附 VISLIMIT.BAS 的天空模型
//!     （[`SkyModel::Schaefer1998`]）、消光（[`Extinction1998`]）与阈值（[`ThresholdModel::Schaefer1998`]），作者最新版。
//! - 阈值（[`ThresholdModel`]）：
//!   - `Schaefer1993`：1993 式 34（Hecht 1947），观测者修正取 Schaefer 1990（PASP 102, 212）式 13–16 与 1993 式 31；
//!   - `Crumey2014`：Crumey 2014（MNRAS 442, 2600）式 34 与 Blackwell 1946 表 8 的系数（式 28），颜色修正式 16，
//!     单眼因子 √2 乘在阈值上（§1.6.4）。Crumey 指出 Schaefer 1990 的颜色修正与单眼因子用法有误（§1.3、§1.6.4）。
//!
//! 单位：亮度 nL（nanoLambert，= mμL）；照度 foot-candle（Schaefer）或 lux（Crumey）；星等 V。阈值为 50% 探测概率。

use crate::math::real::{real, Real, RealOps};

fn dex(x: Real) -> Real {
    (x * real(core::f64::consts::LN_10)).exp()
}

fn pow(a: Real, b: Real) -> Real {
    (b * a.ln()).exp()
}

fn deg(x: f64) -> Real {
    real(x.to_radians())
}

fn rad(d: Real) -> Real {
    d * real(core::f64::consts::PI / 180.0)
}

/// 天顶距上限 90°（地平）：式 2b 在 Z > 90° 时分母变号，不再有意义。
fn clamp_z(z_rad: Real) -> Real {
    z_rad.min(deg(90.0)).max(real(0.0))
}

/// 1993 式 2b（Rozenberg 1966）：整层大气质量，Z 为视天顶距（rad）。Z = 90° 时 X = 40。
pub fn air_mass(z_rad: Real) -> Real {
    let c = clamp_z(z_rad).cos();
    real(1.0) / (c + real(0.025) * (real(-11.0) * c).exp())
}

/// 1993 式 3a：标高 `h_e_km` 的指数分布成分（Rayleigh 8.2 km、气溶胶 1.5 km）的大气质量。
pub fn air_mass_scale_height(z_rad: Real, h_e_km: Real) -> Real {
    let c = clamp_z(z_rad).cos();
    let s = h_e_km.sqrt();
    real(1.0) / (c + real(0.01) * s * (real(-30.0) * c / s).exp())
}

/// 1993 式 3b：观测者之上 `h_l_km` 处薄层（平流层臭氧 20 km）的大气质量；R₀ = 6378 km。
pub fn air_mass_layer(z_rad: Real, h_l_km: Real) -> Real {
    let s = clamp_z(z_rad).sin() / (real(1.0) + h_l_km / real(6378.0));
    real(1.0) / (real(1.0) - s * s).sqrt()
}

/// Krisciunas & Schaefer 1991 式 3：天空亮度模型中的大气质量 X = (1 − 0.96 sin²Z)^{−1/2}（地平处 X = 5）。
/// Z > 90°（光源在地平下）时按 sin²Z 照算，与原文函数形式一致。
pub fn air_mass_sky(z_rad: Real) -> Real {
    let s = z_rad.sin();
    real(1.0) / (real(1.0) - real(0.96) * s * s).sqrt()
}

/// V 波段（0.55 μm）消光系数分量（星等/大气质量）。
#[derive(Clone, Copy, Debug)]
pub struct Extinction {
    pub rayleigh: Real,
    pub ozone: Real,
    pub aerosol: Real,
}

impl Extinction {
    /// 1993 式 4b、5a,b、6a。`height_km` 海拔，`latitude_rad` 纬度，`sun_ra_rad` 太阳赤经（代表季节），
    /// `relative_humidity` 相对湿度（0–1，须 < 1）。
    ///
    /// 式 6a 的季节项 1 + 0.33 sin α_S 按原文照用（作者据北半球资料拟合，原文言明另有随纬度、日期的趋势）；
    /// 式 5b 为明视中心波长值，暗视约为其 30%（原文），此处不区分——臭氧项小，对总消光影响很小。
    pub fn schaefer(height_km: Real, latitude_rad: Real, sun_ra_rad: Real, relative_humidity: Real) -> Self {
        let rayleigh = real(0.1066) * (-height_km / real(8.2)).exp();
        let d_oz = real(3.0) + real(0.4) * (latitude_rad * sun_ra_rad.cos() - (real(3.0) * latitude_rad).cos());
        let ozone = real(0.031) * d_oz / real(3.0);
        let hum = real(1.0) - real(0.32) / relative_humidity.ln();
        let aerosol = real(0.12) * (-height_km / real(1.5)).exp() * pow(hum, real(4.0 / 3.0)) * (real(1.0) + real(0.33) * sun_ra_rad.sin());
        Self { rayleigh, ozone, aerosol }
    }

    /// 1993 式 7：天顶总消光 k。
    pub fn total(&self) -> Real {
        self.rayleigh + self.ozone + self.aerosol
    }

    /// 1993 式 8b：视天顶距 Z 方向的总消光 Δm（星等）。
    pub fn delta_m(&self, z_rad: Real) -> Real {
        self.rayleigh * air_mass_scale_height(z_rad, real(8.2))
            + self.ozone * air_mass_layer(z_rad, real(20.0))
            + self.aerosol * air_mass_scale_height(z_rad, real(1.5))
    }
}

/// Schaefer 1998（*Sky & Telescope* 95(5), 57，程序 VISLIMIT.BAS 1000–1210 行）的消光：在 1993 式 4–6 基础上
/// 气溶胶系数改为 0.1(λ/0.55)^{−1.3}·e^{−H/1.5}·(1 − 0.32/ln S)^{1.33}·(1 + sgn φ·sin α_S)（季节项随半球反号），
/// 加水汽吸收 k_W = w_λ·0.94·S·e^{T/15}·e^{−H/8.2}；各分量大气质量用 1000–1120 行的常数（式 3a、3b 的取整形式），
/// 水汽与 Rayleigh 同用 X_G。
#[derive(Clone, Copy, Debug)]
pub struct Extinction1998 {
    pub rayleigh: Real,
    pub aerosol: Real,
    pub ozone: Real,
    pub water: Real,
}

/// VISLIMIT.BAS 的五个波段（U、B、V、R、I）：(中心波长 μm, 臭氧系数 OZ, 水汽系数 WT)，40、1020、1040 行。
pub const SCHAEFER1998_BANDS: [(f64, f64, f64); 5] =
    [(0.365, 0.000, 0.074), (0.44, 0.000, 0.045), (0.55, 0.031, 0.031), (0.7, 0.008, 0.020), (0.9, 0.000, 0.015)];

impl Extinction1998 {
    /// `band` 为 [`SCHAEFER1998_BANDS`] 下标（V = 2）；海拔 km、纬度 rad、太阳赤经 rad、相对湿度 0–1、气温 ℃。
    /// 程序以月份近似太阳赤经 α = (月 − 3)·30°，此处直接用太阳赤经。纬度 0 时取北半球（程序在此除以 0）。
    pub fn new(band: usize, height_km: Real, latitude_rad: Real, sun_ra_rad: Real, relative_humidity: Real, temperature_c: Real) -> Self {
        let (wa, oz, wt) = SCHAEFER1998_BANDS[band];
        let al_m = height_km * real(1000.0);
        let rayleigh = real(0.1066) * (-al_m / real(8200.0)).exp() * pow(real(wa / 0.55), real(-4.0));
        let sl = if latitude_rad < real(0.0) { real(-1.0) } else { real(1.0) };
        let aerosol = real(0.1) * pow(real(wa / 0.55), real(-1.3)) * (-al_m / real(1500.0)).exp()
            * pow(real(1.0) - real(0.32) / relative_humidity.ln(), real(1.33))
            * (real(1.0) + sl * sun_ra_rad.sin());
        let lt = latitude_rad;
        let ozone = real(oz) * (real(3.0) + real(0.4) * (lt * sun_ra_rad.cos() - (real(3.0) * lt).cos())) / real(3.0);
        let water = real(wt) * real(0.94) * relative_humidity * (temperature_c / real(15.0)).exp() * (-al_m / real(8200.0)).exp();
        Self { rayleigh, aerosol, ozone, water }
    }

    pub fn total(&self) -> Real {
        self.rayleigh + self.aerosol + self.ozone + self.water
    }

    /// 1100–1120、1210 行：Δm = k_R X_G + k_A X_A + k_O X_O + k_W X_G。
    pub fn delta_m(&self, z_rad: Real) -> Real {
        let (xg, xa, xo) = Self::air_masses(z_rad);
        self.rayleigh * xg + self.aerosol * xa + self.ozone * xo + self.water * xg
    }

    /// (X_G, X_A, X_O)，1100–1120 行。
    pub fn air_masses(z_rad: Real) -> (Real, Real, Real) {
        let c = clamp_z(z_rad).cos();
        let xg = real(1.0) / (c + real(0.0286) * (real(-10.5) * c).exp());
        let xa = real(1.0) / (c + real(0.0123) * (real(-24.5) * c).exp());
        (xg, xa, air_mass_layer(z_rad, real(20.0)))
    }
}

/// 消光的给定方式。
#[derive(Clone, Copy, Debug)]
pub enum ExtinctionModel {
    /// 由海拔、纬度、季节、湿度按 1993 式 4–6 估算，分层大气质量（式 8b）。
    Schaefer { relative_humidity: Real },
    /// 由海拔、纬度、季节、湿度、气温按 Schaefer 1998 估算（[`Extinction1998`]，V 波段）。
    Schaefer1998 { relative_humidity: Real, temperature_c: Real },
    /// 实测天顶总消光 k（星等/大气质量），Δm = kX（1993 式 8a，X 取式 2b）。原文建议有当地实测时优先用实测
    /// （1993 表 4 为若干台址年均值；Schaefer 2000 估古埃及夏季 0.35 ± 0.09）。
    Fixed { k: Real },
}

impl ExtinctionModel {
    /// 天顶总消光 k 与视天顶距 `z_rad` 方向的消光 Δm（星等）。海拔 km、纬度 rad、太阳赤经 rad（代表季节）。
    pub fn at(&self, height_km: Real, latitude_rad: Real, sun_ra_rad: Real, z_rad: Real) -> (Real, Real) {
        match *self {
            ExtinctionModel::Fixed { k } => (k, k * air_mass(z_rad)),
            ExtinctionModel::Schaefer { relative_humidity } => {
                let e = Extinction::schaefer(height_km, latitude_rad, sun_ra_rad, relative_humidity);
                (e.total(), e.delta_m(z_rad))
            }
            ExtinctionModel::Schaefer1998 { relative_humidity, temperature_c } => {
                let e = Extinction1998::new(2, height_km, latitude_rad, sun_ra_rad, relative_humidity, temperature_c);
                (e.total(), e.delta_m(z_rad))
            }
        }
    }
}

/// 散射相函数（ρ 为与光源角距，度）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScatteringFunction {
    /// Krisciunas & Schaefer 1991 式 16–19：Rayleigh 10^5.36[1.06 + cos²ρ] 加 Mie 项，ρ ≥ 10° 取 10^{6.15−ρ/40}，
    /// ρ < 10° 取 6.2×10⁷ρ⁻²（原文分段）。原文表 2 由此逐值复现。
    KrisciunasSchaefer1991,
    /// Schaefer 1993 式 14：三项直接相加（30° 处比 1991 原式大约 10%，5° 处大约 36%）。
    Schaefer1993,
}

/// 晨昏蒙影天空亮度模型。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TwilightModel {
    /// 1993 式 15c：B = max{1, dex[(ρ_S/90°) − 1.1]}·10^{8.45+0.4h_S}(1 − 10^{−0.4kX})，与白昼项 15d 取小。
    /// `direction`：方向因子按原文印刷（只在背日 ρ_S > 99° 增亮）或不计。
    Schaefer1993 { direction: TwilightDirectionTerm },
    /// Schaefer 1987 式 1–2：标准天空（k = 0.2、m_z = 6）lg b_ST = 4.75 − θZ/3 + h₀(12 + 8.21Z) + 2.86Z（弧度；θ 为与太阳
    /// 方位差，Z 天体天顶距，h₀ 太阳高度），对 −18° < h₀ < −6°、Z > 60°、θ < 90° 拟合 Koomen et al. 1952 等实测；
    /// 非标准天空 b = 夜天光 + (k/0.2)(b_ST − 118 nL)。原文式 2 的夜天光项印作 (10^Y + 1)²/K，按原文常数 118 反推应为
    /// (10^Y − 1)²/K（m_z = 6、k = 0.2 时得 118），此处夜天光改由 [`SkyConditions::night_sky_nl`] 给出。
    /// 太阳在地平以上时退回 1993 式 15d。
    Schaefer1987,
}

/// 天空亮度模型。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SkyModel {
    /// Schaefer 1993 式 15a–e，散射与月光按 Krisciunas & Schaefer 1991（见 [`ScatteringFunction`]），蒙影可选。
    Schaefer1993 { twilight: TwilightModel, scattering: ScatteringFunction },
    /// Schaefer 1998 VISLIMIT.BAS 2000–2460 行（V 波段）：
    /// - 夜天光同 1993 式 15a，大气质量用 Rozenberg 式 2b；
    /// - 月光、白昼：f(ρ) 取三项相加（1993 式 14），另加多次散射项：B = F(m)·(1 − 10^{−0.4kX})·[f(ρ)C + 440000(1 − C)]，
    ///   C = 10^{−0.4kX_源}（X_源 为日、月的 Rozenberg 大气质量，地平下取 40）；
    ///   F(m) = 10^{−0.4(m − M_O + 43.27)}/1.11×10⁻¹⁵ nL，V 波段 M_O = −11.05（60 行、260 行）；
    /// - 蒙影：B = 10^{−0.4(M_S − M_O + 32.5 − h_S − Z/(360k))}/1.11×10⁻¹⁵·(100/ρ_S)·(1 − 10^{−0.4k})（2280–2290 行，
    ///   Z、h_S、ρ_S 以度计；末项按原文不含 X）；
    /// - 合成：夜天光 + min(白昼, 蒙影) + 月光（月在地平上时）。
    Schaefer1998,
}

/// 1993 式 15c 方向因子的处理。原文印刷如此：该因子只在背日方向（ρ_S > 99°）使天空增亮，向日一侧恒为 1。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TwilightDirectionTerm {
    /// 按原文印刷。
    AsPrinted,
    /// 不计方向因子（恒为 1）。
    None,
}

/// 天空条件。
#[derive(Clone, Copy, Debug)]
pub struct SkyConditions {
    pub extinction: ExtinctionModel,
    /// 1993 式 15a 的 B₀：暗夜天顶夜天光（nL，大气外等效）。1993：暗台址典型值 180 nL；1998 程序 V 波段
    /// 1.0×10⁻¹³ erg cm⁻² s⁻¹ μm⁻¹ arcsec⁻² ≈ 90 nL，乘太阳活动因子（[`schaefer1998_night_sky_nl`]）。
    /// 也可由观测者天顶极限星等反推，见 [`night_sky_from_zenith_limit`]。
    pub night_sky_nl: Real,
    /// 1993 式 15e 的 B_city（nL）；古代取 0。
    pub city_nl: Real,
    pub model: SkyModel,
}

impl Default for SkyConditions {
    /// 缺省为 Schaefer 1998 整套（作者最新版；检验见 doc/astronomy/celestial-phenomena.md）：消光按湿度 50%、气温 15 ℃ 估算，
    /// 夜天光取太阳活动周平均。
    fn default() -> Self {
        Self::schaefer1998(real(0.5), real(15.0))
    }
}

impl SkyConditions {
    /// Schaefer 1998 整套：[`ExtinctionModel::Schaefer1998`]、[`SkyModel::Schaefer1998`]、夜天光 [`schaefer1998_night_sky_nl`]（周期平均）。
    pub fn schaefer1998(relative_humidity: Real, temperature_c: Real) -> Self {
        Self {
            extinction: ExtinctionModel::Schaefer1998 { relative_humidity, temperature_c },
            night_sky_nl: schaefer1998_night_sky_nl(None),
            city_nl: real(0.0),
            model: SkyModel::Schaefer1998,
        }
    }

    /// Schaefer 1993 整套：1993 消光估算、夜天光 180 nL、式 15c 蒙影（方向因子按原文）、Krisciunas & Schaefer 1991 散射。
    pub fn schaefer1993(relative_humidity: Real) -> Self {
        Self {
            extinction: ExtinctionModel::Schaefer { relative_humidity },
            night_sky_nl: real(180.0),
            city_nl: real(0.0),
            model: SkyModel::Schaefer1993 {
                twilight: TwilightModel::Schaefer1993 { direction: TwilightDirectionTerm::AsPrinted },
                scattering: ScatteringFunction::KrisciunasSchaefer1991,
            },
        }
    }
}

/// 1998 程序 V 波段换算：亮度 erg cm⁻² s⁻¹ μm⁻¹ arcsec⁻² → nL（260 行）。
const SCHAEFER1998_V_PER_NL: f64 = 1.11e-15;
/// 1998 程序 V 波段 M_O（60 行）与太阳 V 星等（2060 行）。
const SCHAEFER1998_MO_V: f64 = -11.05;
const SCHAEFER1998_MS_V: f64 = -26.74;

/// 1998 程序 2140 行的夜天光（V，nL）：B₀·[1 + 0.3 cos(2π(年 − 1992)/11)]，B₀ = 1.0×10⁻¹³/1.11×10⁻¹⁵。
/// `year` 为 None 时取太阳活动周平均（因子 1）——古代太阳周相位未知。
pub fn schaefer1998_night_sky_nl(year: Option<Real>) -> Real {
    let b0 = real(1.0e-13 / SCHAEFER1998_V_PER_NL);
    match year {
        Some(y) => b0 * (real(1.0) + real(0.3) * (real(2.0 * core::f64::consts::PI) * (y - real(1992.0)) / real(11.0)).cos()),
        None => b0,
    }
}

/// 散射相函数（ρ 为与光源角距，度）。ρ → 0 发散，取下限 0.1°。
pub fn scattering_function(rho_deg: Real, model: ScatteringFunction) -> Real {
    let rho = rho_deg.max(real(0.1));
    let c = rad(rho).cos();
    let rayleigh = dex(real(5.36)) * (real(1.06) + c * c);
    let mie_wide = dex(real(6.15) - rho / real(40.0));
    let aureole = real(6.2e7) / (rho * rho);
    match model {
        ScatteringFunction::KrisciunasSchaefer1991 => rayleigh + if rho < real(10.0) { aureole } else { mie_wide },
        ScatteringFunction::Schaefer1993 => rayleigh + mie_wide + aureole,
    }
}

/// 月亮 V 星等（Krisciunas & Schaefer 1991 式 9 = 1993 式 12），α 为月相角（度，满月 0°）。不含冲日效应（|α| < 7°）。
pub fn moon_magnitude(phase_deg: Real) -> Real {
    let a = phase_deg.abs();
    real(-12.73) + real(0.026) * a + real(4e-9) * a * a * a * a
}

/// 天空亮度所需的几何量（角度均为度；天顶距为视天顶距）。
#[derive(Clone, Copy, Debug)]
pub struct SkyGeometry {
    /// 视线方向天顶距
    pub z_deg: Real,
    /// 视线与太阳角距
    pub rho_sun_deg: Real,
    /// 视线与太阳方位差（0–180°，1987 蒙影式用）
    pub azimuth_diff_sun_deg: Real,
    /// 太阳天顶距
    pub z_sun_deg: Real,
    /// 月亮：(视线与月角距, 月天顶距, 月 V 星等)；月在地平下或不计月光时为 None。
    pub moon: Option<(Real, Real, Real)>,
}

/// 视线方向天空亮度（nL）。`k` 为天顶总消光。
pub fn sky_brightness_nl(g: &SkyGeometry, k: Real, sky: &SkyConditions) -> Real {
    match sky.model {
        SkyModel::Schaefer1993 { twilight, scattering } => sky_1993(g, k, sky, twilight, scattering),
        SkyModel::Schaefer1998 => sky_1998(g, k, sky),
    }
}

/// 1993 式 15a–e。
fn sky_1993(g: &SkyGeometry, k: Real, sky: &SkyConditions, twilight: TwilightModel, scattering: ScatteringFunction) -> Real {
    let z = rad(g.z_deg);
    let x = air_mass_sky(z);
    let s = z.sin();
    let att = real(1.0) - dex(real(-0.4) * k * x);
    // 15a 夜天光
    let b_night = sky.night_sky_nl * (real(0.4) + real(0.6) / (real(1.0) - real(0.96) * s * s).sqrt()) * dex(real(-0.4) * k * x);
    // 15b 月光（Krisciunas & Schaefer 1991 式 15、20、21）
    let b_moon = match g.moon {
        Some((rho, z_moon, m_moon)) if z_moon < real(90.0) => {
            scattering_function(rho, scattering) * dex(real(-0.4) * (m_moon + real(16.57) + k * air_mass_sky(rad(z_moon)))) * att
        }
        _ => real(0.0),
    };
    // 15d 白昼（同上式，太阳星等 −26.74：10^{−0.4(−26.74+16.57)} = 11700）
    let b_day = real(11700.0) * scattering_function(g.rho_sun_deg, scattering) * dex(real(-0.4) * k * air_mass_sky(rad(g.z_sun_deg))) * att;
    let h_sun = real(90.0) - g.z_sun_deg;
    let b_sun = match twilight {
        TwilightModel::Schaefer1993 { direction } => {
            let dir = match direction {
                TwilightDirectionTerm::AsPrinted => real(1.0).max(dex(g.rho_sun_deg / real(90.0) - real(1.1))),
                TwilightDirectionTerm::None => real(1.0),
            };
            let b_twi = dir * dex(real(8.45) + real(0.4) * h_sun) * att;
            b_twi.min(b_day)
        }
        TwilightModel::Schaefer1987 => {
            if h_sun > real(0.0) {
                b_day
            } else {
                let (th, zz, h0) = (rad(g.azimuth_diff_sun_deg), z, rad(h_sun));
                let b_st = dex(real(4.75) - th * zz / real(3.0) + h0 * (real(12.0) + real(8.21) * zz) + real(2.86) * zz);
                (k / real(0.2)) * (b_st - real(118.0)).max(real(0.0))
            }
        }
    };
    b_night + b_moon + b_sun + sky.city_nl
}

/// Schaefer 1998 VISLIMIT.BAS 2000–2460 行（V 波段），见 [`SkyModel::Schaefer1998`]。
fn sky_1998(g: &SkyGeometry, k: Real, sky: &SkyConditions) -> Real {
    let z = rad(g.z_deg);
    let x = air_mass(z); // 2070 行 Rozenberg
    let s = z.sin();
    let att = real(1.0) - dex(real(-0.4) * k * x);
    let flux_nl = |m: Real| dex(real(-0.4) * (m - real(SCHAEFER1998_MO_V) + real(43.27))) / real(SCHAEFER1998_V_PER_NL);
    let scat = |rho: Real| scattering_function(rho, ScatteringFunction::Schaefer1993);
    // 2140–2160 夜天光（太阳活动因子已含在 night_sky_nl 中）
    let b_night = sky.night_sky_nl * (real(0.4) + real(0.6) / (real(1.0) - real(0.96) * s * s).sqrt()) * dex(real(-0.4) * k * x);
    // 2180–2250 月光（月在地平上时，2420 行）
    let b_moon = match g.moon {
        Some((rho, z_moon, m_moon)) if z_moon < real(90.0) => {
            let c3 = dex(real(-0.4) * k * air_mass(rad(z_moon)));
            flux_nl(m_moon) * att * (scat(rho) * c3 + real(440000.0) * (real(1.0) - c3))
        }
        _ => real(0.0),
    };
    // 2270–2290 蒙影
    let h_sun = real(90.0) - g.z_sun_deg;
    let b_twi = dex(real(-0.4) * (real(SCHAEFER1998_MS_V) - real(SCHAEFER1998_MO_V) + real(32.5) - h_sun - g.z_deg / (real(360.0) * k)))
        / real(SCHAEFER1998_V_PER_NL)
        * (real(100.0) / g.rho_sun_deg.max(real(0.1)))
        * (real(1.0) - dex(real(-0.4) * k));
    // 2310–2360 白昼（太阳在地平下 X = 40，2110 行）
    let c4 = dex(real(-0.4) * k * air_mass(rad(g.z_sun_deg)));
    let b_day = flux_nl(real(SCHAEFER1998_MS_V)) * att * (scat(g.rho_sun_deg) * c4 + real(440000.0) * (real(1.0) - c4));
    b_night + b_twi.min(b_day) + b_moon + sky.city_nl
}

/// 阈值模型。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ThresholdModel {
    /// Schaefer 1993 式 34（Hecht 1947）+ Schaefer 1990 观测者修正。
    Schaefer1993,
    /// Crumey 2014 式 34（Blackwell 1946 表 8 系数，式 28），`field_factor` 为其 F（含观测者个人因素、实验室换算等；
    /// 原文以 F = 2 为示例性的典型值，实际观测在 1.4–2.4 之间）。
    Crumey2014 { field_factor: Real },
    /// Schaefer 1998 VISLIMIT.BAS 260–320 行：Hecht 两支（天空 > 1500 nL 取明视常数），m = −16.57 − 2.5 lg I − Δm + 5 lg SN，
    /// `snellen` 为 Snellen 视力比（20/20 = 1，20/10 = 2）。不含 1990 的颜色修正、暗视消光 0.48 与单双眼因子。
    Schaefer1998 { snellen: Real },
}

/// 观测者（肉眼）。
#[derive(Clone, Copy, Debug)]
pub struct Eye {
    /// 双眼或单眼（单眼阈值 ×√2：Schaefer 1993 式 31b，Crumey 2014 §1.6.4）。
    pub binocular: bool,
    /// Schaefer 1990 F_s：观测者对点源的敏锐度修正，< 1 为敏锐（仅 `Schaefer1993` 用；Crumey 的 F 已含此项）。
    pub acuity_factor: Real,
    /// Schaefer 1990 暗视颜色修正用的天空色指数 B−V（1990：无月夜空色温 5500 K，对应 0.7；仅 `Schaefer1993` 用）。
    pub sky_color_bv: Real,
    pub threshold: ThresholdModel,
}

impl Default for Eye {
    /// 缺省阈值为 Schaefer 1998（Snellen 1，即 20/20 视力）。
    fn default() -> Self {
        Self { binocular: true, acuity_factor: real(1.0), sky_color_bv: real(0.7), threshold: ThresholdModel::Schaefer1998 { snellen: real(1.0) } }
    }
}

/// 一次可见性判定的结果。
#[derive(Clone, Copy, Debug)]
pub struct VisibilityResult {
    /// 视线方向天空亮度（nL）
    pub sky_nl: Real,
    /// 该方向（含消光）的极限星等：天体大气外 V 星等 ≤ 此值即可见（50% 概率）
    pub limiting_magnitude: Real,
}

/// Schaefer 1990 式 13：暗视颜色修正 F_c；明视为 1。
fn schaefer_color_factor(bv: Real, photopic: bool) -> Real {
    if photopic {
        real(1.0)
    } else {
        dex(real(-0.4) * (real(1.0) - bv / real(2.0)))
    }
}

/// 1 nL = 10⁻⁹ L = 10⁻⁹ × 10⁴/π cd m⁻²。
pub fn nl_to_cd_m2(nl: Real) -> Real {
    nl * real(1e-5 / core::f64::consts::PI)
}

/// Crumey 2014 式 34（a_i 取式 28）：点源阈值照度增量 ΔI（lux，不含 F），B 为背景亮度（cd m⁻²）。
pub fn crumey_point_threshold_lux(b_cd_m2: Real) -> Real {
    let (a1, a2, a3, a4, a5) = (real(5.949e-8), real(-2.389e-7), real(2.459e-7), real(4.120e-4), real(-4.225e-4));
    let b = b_cd_m2;
    let b14 = b.sqrt().sqrt();
    let b12 = b.sqrt();
    let inner = (a1 * b12 + a2 * b12 * b14 + a3 * b).max(real(0.0));
    let s = inner.sqrt() + a4 * b14 + a5 * b12;
    s * s
}

/// Crumey 2014 式 26、27 的分界亮度（暗视/明视两支交点）7.08×10⁻² cd m⁻²：其下按暗视施颜色修正（式 16）。
/// 原文只给暗视的颜色修正（§1.3：中间视觉的光度学无唯一定义），此处在两支分界处切换，极限星等在此跳变约 0.6 等。
pub const CRUMEY_SCOTOPIC_SPLIT_CD_M2: f64 = 7.08e-2;

/// 点源极限星等（大气外 V）。`sky_nl` 为天空 V 亮度，`delta_m` 为该方向 V 消光（星等），`source_bv` 为天体色指数。
pub fn limiting_magnitude(sky_nl: Real, delta_m: Real, source_bv: Real, eye: &Eye) -> VisibilityResult {
    let lim = match eye.threshold {
        ThresholdModel::Schaefer1993 => {
            // 1990 式 15：眼感背景 B = B_s / (F_b F_c,sky)；1993 式 34：I = c(1 + √(KB))²；1990 式 14：I* = I F_b F_e F_c F_s；
            // 1990 式 16：m = −16.57 − 2.5 lg I*（foot-candle）。明暗视分支按 lg B 与 3.17 比较（取天空亮度判定）。
            let photopic = sky_nl.log10() > real(3.17);
            let f_b = if eye.binocular { real(1.0) } else { real(2.0).sqrt() };
            let b = sky_nl / (f_b * schaefer_color_factor(eye.sky_color_bv, photopic));
            let (c, kk, fe_scale) = if photopic { (dex(real(-8.35)), dex(real(-5.90)), real(0.4)) } else { (dex(real(-9.80)), dex(real(-1.90)), real(0.48)) };
            let one_plus = real(1.0) + (kk * b).sqrt();
            let i_th = c * one_plus * one_plus;
            let i_star = i_th * f_b * dex(fe_scale * delta_m) * schaefer_color_factor(source_bv, photopic) * eye.acuity_factor;
            real(-16.57) - real(2.5) * i_star.log10()
        }
        ThresholdModel::Schaefer1998 { snellen } => {
            let (c1, c2) = if sky_nl > real(1500.0) { (dex(real(-8.35)), dex(real(-5.9))) } else { (dex(real(-9.8)), dex(real(-1.9))) };
            let one_plus = real(1.0) + (c2 * sky_nl).sqrt();
            let th = c1 * one_plus * one_plus;
            real(-16.57) - real(2.5) * th.log10() - delta_m + real(5.0) * snellen.log10()
        }
        ThresholdModel::Crumey2014 { field_factor } => {
            // 式 53 推广：ΔI = F·(式 34)，单眼 ×√2；m = −2.5 lg(ΔI/Z_V)，Z_V = 2.54×10⁻⁶ lx（§1.3）；
            // 消光：大气外所需照度 ×10^{0.4Δm}；颜色：暗视时 m* = m₂₈₅₀ + 0.72 − 0.27(B−V)（式 16）。
            let b = nl_to_cd_m2(sky_nl);
            let mono = if eye.binocular { real(1.0) } else { real(2.0).sqrt() };
            let di = crumey_point_threshold_lux(b) * field_factor * mono;
            let m = real(-2.5) * (di / real(2.54e-6)).log10() - delta_m;
            if b < real(CRUMEY_SCOTOPIC_SPLIT_CD_M2) {
                m + real(0.72) - real(0.27) * source_bv
            } else {
                m
            }
        }
    };
    VisibilityResult { sky_nl, limiting_magnitude: lim }
}

/// 眩光源在视线方向造成的附加背景（nL），Schaefer 1991《Glare and celestial visibility》（*PASP* 103, 645）肉眼部分。
#[derive(Clone, Copy, Debug)]
pub struct Glare {
    /// 式 1 第二项：大气散射晕 1.14×10⁷ I* θ⁻² (10^{−0.4kX} − 10^{−0.8kX})/0.18（第一项为 1″ 量级的视宁度核，肉眼可略）。
    /// 与月光、白昼模型中的晕项 6.2×10⁷ρ⁻² 同源（Krisciunas & Schaefer 1991 式 19 即引此），月、日作眩光源时不应重复计入。
    pub atmospheric: Real,
    /// 式 3（Holladay 1926）：眼内散射 4.63×10⁷ I θ⁻²，I = I*·10^{−0.4kX} 为进入眼睛的照度（式 4、23，F_s 不施于眩光源）。
    pub eye: Real,
}

/// 眩光：`glare_vmag` 为眩光源大气外 V 星等，`theta_deg` 为与视线角距（度，须大于眼的分辨角，原文式 3 的适用条件），
/// `k_x` 为眩光源方向的消光（kX，星等）。照度 I* = 10^{−0.4(m + 16.57)} foot-candle（式 2）。
pub fn glare_nl(glare_vmag: Real, theta_deg: Real, k_x: Real) -> Glare {
    let i_star = dex(real(-0.4) * (glare_vmag + real(16.57)));
    let th2 = theta_deg * theta_deg;
    let t1 = dex(real(-0.4) * k_x);
    let atmospheric = real(1.14e7) * i_star / th2 * (t1 - t1 * t1) / real(0.18);
    let eye = real(4.63e7) * i_star * t1 / th2;
    Glare { atmospheric, eye }
}

/// Schaefer 1990「星群平均」F_c = 0.5 对应的色指数（1990 式 13：1 − (B−V)/2 = −2.5 lg 0.5）。由天顶极限星等反推夜天光时
/// 以此代表「天顶最暗可见星」的颜色。
pub fn ensemble_star_bv() -> Real {
    real(2.0) * (real(1.0) - real(2.5) * real(2.0).log10())
}

/// 由观测者暗夜天顶极限星等 m_z 与天顶消光 k 反推 1993 式 15a 的 B₀（nL）：在同一阈值模型下解
/// limiting_magnitude(B₀·10^{−0.4k}, k, B−V_星群) = m_z（Schaefer 1987 以 m_z 刻画天空，式 2）。无解（m_z 过亮或过暗）时取边界。
pub fn night_sky_from_zenith_limit(m_z: Real, k: Real, eye: &Eye) -> Real {
    let bv = ensemble_star_bv();
    let lim = |log_b: Real| limiting_magnitude(dex(log_b), k, bv, eye).limiting_magnitude;
    let (mut lo, mut hi) = (real(-1.0), real(9.0)); // 0.1 nL … 10⁹ nL；极限星等随 B 单调减
    for _ in 0..80 {
        let mid = (lo + hi) / real(2.0);
        if lim(mid) > m_z {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    dex((lo + hi) / real(2.0)) * dex(real(0.4) * k)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1993 表 3：大气质量随天顶距。
    #[test]
    fn air_mass_matches_table_3() {
        let rows: [(f64, f64, f64, f64, f64); 7] = [
            (60.0, 2.0, 2.0, 2.0, 2.0),
            (80.0, 5.6, 5.6, 5.7, 5.3),
            (85.0, 10.3, 10.1, 11.3, 8.5),
            (87.0, 15.1, 14.5, 17.9, 10.6),
            (88.0, 19.3, 18.3, 24.9, 11.6),
            (89.0, 26.3, 24.2, 39.3, 12.4),
            (90.0, 40.0, 34.9, 81.6, 12.7),
        ];
        let mut tab = crate::test_util::CheckTable::new("schaefer1993_table3_airmass", &["Z°", "X 原表", "本实现", "X_G 原表", "本实现", "X_A 原表", "本实现", "X_O 原表", "本实现"]);
        tab.meta("Schaefer 1993 表 3 大气质量逐值（式 2b、3a 标高 8.2/1.5 km、3b 20 km 层）");
        for (z, x, xg, xa, xo) in rows {
            let zr = deg(z);
            tab.row(vec![
                format!("{z}"),
                format!("{x}"),
                format!("{:.2}", air_mass(zr).as_f64()),
                format!("{xg}"),
                format!("{:.2}", air_mass_scale_height(zr, real(8.2)).as_f64()),
                format!("{xa}"),
                format!("{:.2}", air_mass_scale_height(zr, real(1.5)).as_f64()),
                format!("{xo}"),
                format!("{:.2}", air_mass_layer(zr, real(20.0)).as_f64()),
            ]);
            for (got, want) in [
                (air_mass(zr), x),
                (air_mass_scale_height(zr, real(8.2)), xg),
                (air_mass_scale_height(zr, real(1.5)), xa),
                (air_mass_layer(zr, real(20.0)), xo),
            ] {
                assert!((got.as_f64() - want).abs() <= 0.051 * want.max(1.0), "Z={z}: {} vs {want}", got.as_f64());
            }
        }
        tab.write();
    }

    /// Krisciunas & Schaefer 1991 表 2：月光亮度（nL），月天顶距 60°、k = 0.172，视线沿过月与天顶的大圆（ρ = 60° 为天顶）。
    /// 原文表值为整数，逐值相差 ≤ 0.5 nL 或 0.1%。
    #[test]
    fn moonlight_matches_krisciunas_schaefer_table_2() {
        let table: [(f64, [f64; 5]); 4] = [
            (30.0, [7216.0, 1160.0, 530.0, 437.0, 818.0]),
            (60.0, [3364.0, 541.0, 247.0, 204.0, 381.0]),
            (90.0, [1351.0, 217.0, 99.0, 82.0, 153.0]),
            (120.0, [391.0, 63.0, 29.0, 24.0, 44.0]),
        ];
        let sky = SkyConditions { night_sky_nl: real(0.0), ..SkyConditions::schaefer1993(real(0.5)) };
        let mut tab = crate::test_util::CheckTable::new("ks1991_table2_moonlight", &["月相角°", "ρ°", "原表 nL", "本实现 nL", "差 nL"]);
        tab.meta("Krisciunas & Schaefer 1991 表 2 月光亮度逐值（月天顶距 60°、k = 0.172）");
        for (alpha, row) in table {
            for (rho, want) in [5.0, 30.0, 60.0, 90.0, 120.0].iter().zip(row) {
                let g = SkyGeometry {
                    z_deg: real((rho - 60.0f64).abs()),
                    rho_sun_deg: real(180.0),
                    azimuth_diff_sun_deg: real(180.0),
                    z_sun_deg: real(150.0),
                    moon: Some((real(*rho), real(60.0), moon_magnitude(real(alpha)))),
                };
                let got = sky_brightness_nl(&g, real(0.172), &sky).as_f64();
                tab.row(vec![format!("{alpha}"), format!("{rho}"), format!("{want}"), format!("{got:.1}"), format!("{:+.1}", got - want)]);
                assert!((got - want).abs() <= 0.5f64.max(want * 1e-3), "α={alpha} ρ={rho}: {got} vs {want}");
            }
        }
        tab.write();
    }

    /// 1990 式 18：天顶极限星等 m_z = 8.68 − 1.2k − 5 lg(1 + 0.158 √B_s)（F_c = 0.5、F_s = 1）；
    /// 原文例：B_s = 136 nL、k = 0.3 → 6.05。此处天空色、星色均取 F_c = 0.5 对应的 B−V。
    #[test]
    fn zenith_limit_matches_1990_eq18() {
        let bv = ensemble_star_bv();
        let eye = Eye { sky_color_bv: bv, threshold: ThresholdModel::Schaefer1993, ..Default::default() };
        let mut tab = crate::test_util::CheckTable::new("schaefer1990_eq18_zenith_limit", &["B_s nL", "k", "式 18", "本实现", "差"]);
        tab.meta("Schaefer 1990 式 18 天顶极限星等（原文算例 136 nL、k = 0.3 → 6.05）");
        for (bs, k) in [(136.0, 0.3), (65.0, 0.2), (300.0, 0.25)] {
            let r = limiting_magnitude(real(bs), real(k), bv, &eye);
            let want = 8.68 - 1.2 * k - 5.0 * (1.0 + 0.158 * f64::sqrt(bs)).log10();
            tab.row(vec![format!("{bs}"), format!("{k}"), format!("{want:.3}"), format!("{:.3}", r.limiting_magnitude.as_f64()), format!("{:+.4}", r.limiting_magnitude.as_f64() - want)]);
            assert!((r.limiting_magnitude.as_f64() - want).abs() < 0.01, "{bs},{k}: {} vs {want}", r.limiting_magnitude.as_f64());
        }
        tab.write();
        let r = limiting_magnitude(real(136.0), real(0.3), bv, &eye);
        assert!((r.limiting_magnitude.as_f64() - 6.05).abs() < 0.01);
    }

    /// Crumey 2014：暗夜 B = 2×10⁻⁴ cd m⁻²（21.83 mag arcsec⁻²）时式 53 给 m₀ = 6.93 − 2.5 lg F（§3.1），
    /// 式 54 线性近似 m₀ = 0.3834μ − 1.4400 − 2.5 lg F（20 < μ < 22，误差 ≤ 0.01）。式 34 与式 53 在暗视区一致。
    #[test]
    fn crumey_dark_sky_limits() {
        let mu_to_cd = |mu: f64| 10f64.powf((12.58 - mu) / 2.5); // §1.3：μ_V = −2.5 lg B + 12.58
        let m0 = |mu: f64| -2.5 * (crumey_point_threshold_lux(real(mu_to_cd(mu))).as_f64() / 2.54e-6).log10();
        assert!((m0(21.83) - 6.93).abs() < 0.02, "{}", m0(21.83));
        for mu in [20.2, 21.0, 21.5, 21.9] {
            let want = 0.3834 * mu - 1.4400;
            assert!((m0(mu) - want).abs() < 0.03, "μ={mu}: {} vs {want}", m0(mu));
        }
        // 全范围 ΔI 随 B 基本单调增：式 34 在 lg B ≈ −1.7…−1.3（暗视/明视交接）有约 2.5% 的回落平台，为拟合函数本身形状
        let mut prev = 0.0;
        for i in 0..=100 {
            let b = 10f64.powf(-6.0 + 0.1 * i as f64);
            let di = crumey_point_threshold_lux(real(b)).as_f64();
            assert!(di > 0.97 * prev, "B={b}");
            prev = prev.max(di);
        }
    }

    /// 1993 表 5：明视极限与「白昼视觉极限 4.1」（约 1500 nL 背景）同量级。
    #[test]
    fn threshold_constants_vs_table_5() {
        let eye = Eye { threshold: ThresholdModel::Schaefer1993, ..Default::default() };
        let r = limiting_magnitude(real(1500.0), real(0.0), real(0.7), &eye);
        assert!(r.limiting_magnitude.as_f64() > 3.0 && r.limiting_magnitude.as_f64() < 4.4, "{}", r.limiting_magnitude.as_f64());
    }

    /// 1993 表 6 数量级：暗夜天顶 ~B₀，民用晨昏蒙影天顶 ~10⁵ nL，日落时天顶 ~3×10⁷ nL（×3 以内）。
    /// 白昼项用 Krisciunas & Schaefer 式 3 的 X 后，太阳在地平及以下时 min(B_twi, B_day) 自然取蒙影项。
    #[test]
    fn sky_brightness_orders_vs_table_6() {
        let sky = SkyConditions { extinction: ExtinctionModel::Fixed { k: real(0.25) }, ..SkyConditions::schaefer1993(real(0.5)) };
        let at = |h_sun: f64| {
            let g = SkyGeometry {
                z_deg: real(0.0),
                rho_sun_deg: real(90.0 - h_sun),
                azimuth_diff_sun_deg: real(0.0),
                z_sun_deg: real(90.0 - h_sun),
                moon: None,
            };
            sky_brightness_nl(&g, real(0.25), &sky).as_f64()
        };
        let ratio = |a: f64, b: f64| (a / b).max(b / a);
        assert!(ratio(at(-30.0), 150.0) < 1.5, "night {}", at(-30.0));
        assert!(ratio(at(-6.0), 1e5) < 3.0, "civil {}", at(-6.0));
        assert!(ratio(at(0.0), 3e7) < 3.0, "sunset {}", at(0.0));
    }

    /// Schaefer 1998 原文算例（第 58、60 页）：美国西南部 2 月暗夜（φ = 30°，海拔 1000 m，湿度 40%，15 ℃，1998 年，
    /// 无月、太阳在地平下 90°），天顶距 45°。原文输出：大气质量 1.41；UBVRI 消光系数 0.62、0.32、0.19、0.09、0.05，
    /// 总消光 0.87、0.46、0.27、0.13、0.07；V 天空 6.8×10⁻¹⁴（61 nL）；极限星等 6.29。太阳赤经按程序取 (2 − 3)·30°。
    #[test]
    fn schaefer1998_worked_example() {
        let (z, lat, ra) = (deg(45.0), deg(30.0), deg(-30.0));
        let (xg, xa, xo) = Extinction1998::air_masses(z);
        let mut tab = crate::test_util::CheckTable::new("schaefer1998_worked_example", &["量", "原文", "本实现"]);
        tab.meta("Schaefer 1998 原文算例：美国西南部 2 月暗夜，30°N、海拔 1000 m、湿度 40%、15 ℃、1998 年，天顶距 45°");
        tab.row(vec!["大气质量 X_G、X_A、X_O".into(), "1.41".into(), format!("{:.3} / {:.3} / {:.3}", xg.as_f64(), xa.as_f64(), xo.as_f64())]);
        for x in [xg, xa, xo] {
            assert!((x.as_f64() - 1.41).abs() < 0.006, "{}", x.as_f64());
        }
        let want_k = [0.62, 0.32, 0.19, 0.09, 0.05];
        let want_dm = [0.87, 0.46, 0.27, 0.13, 0.07];
        for b in 0..5 {
            let e = Extinction1998::new(b, real(1.0), lat, ra, real(0.40), real(15.0));
            let band = ["U", "B", "V", "R", "I"][b];
            tab.row(vec![format!("{band} 消光系数 k"), format!("{}", want_k[b]), format!("{:.3}", e.total().as_f64())]);
            tab.row(vec![format!("{band} 总消光 Δm"), format!("{}", want_dm[b]), format!("{:.3}", e.delta_m(z).as_f64())]);
            assert!((e.total().as_f64() - want_k[b]).abs() < 0.006, "k[{b}] = {}", e.total().as_f64());
            assert!((e.delta_m(z).as_f64() - want_dm[b]).abs() < 0.006, "Δm[{b}] = {}", e.delta_m(z).as_f64());
        }
        let ev = Extinction1998::new(2, real(1.0), lat, ra, real(0.40), real(15.0));
        let sky = SkyConditions {
            extinction: ExtinctionModel::Schaefer1998 { relative_humidity: real(0.40), temperature_c: real(15.0) },
            night_sky_nl: schaefer1998_night_sky_nl(Some(real(1998.0))),
            city_nl: real(0.0),
            model: SkyModel::Schaefer1998,
        };
        let g = SkyGeometry { z_deg: real(45.0), rho_sun_deg: real(180.0), azimuth_diff_sun_deg: real(180.0), z_sun_deg: real(180.0), moon: None };
        let b = sky_brightness_nl(&g, ev.total(), &sky).as_f64();
        assert!((b * 1.11e-15 - 6.8e-14).abs() < 0.05e-14 && (b - 61.0).abs() < 1.0, "{b} nL");
        let eye = Eye { threshold: ThresholdModel::Schaefer1998 { snellen: real(1.0) }, ..Default::default() };
        let m = limiting_magnitude(real(b), ev.delta_m(z), real(0.0), &eye).limiting_magnitude.as_f64();
        tab.row(vec!["V 天空亮度".into(), "6.8×10⁻¹⁴（61 nL）".into(), format!("{:.2}×10⁻¹⁴（{b:.1} nL）", b * 1.11e-15 / 1e-14)]);
        tab.row(vec!["V 极限星等".into(), "6.29".into(), format!("{m:.3}")]);
        tab.write();
        assert!((m - 6.29).abs() < 0.006, "{m}");
    }

    /// 1998 天空模型的数量级（同 1993 表 6）：民用蒙影天顶 ~10⁵ nL、日落天顶 ~3×10⁷ nL（×3 以内）；蒙影向日一侧更亮。
    #[test]
    fn schaefer1998_twilight_orders() {
        let sky = SkyConditions { night_sky_nl: schaefer1998_night_sky_nl(None), model: SkyModel::Schaefer1998, ..Default::default() };
        let at = |h_sun: f64, z: f64, rho: f64| {
            let g = SkyGeometry { z_deg: real(z), rho_sun_deg: real(rho), azimuth_diff_sun_deg: real(rho), z_sun_deg: real(90.0 - h_sun), moon: None };
            sky_brightness_nl(&g, real(0.25), &sky).as_f64()
        };
        let ratio = |a: f64, b: f64| (a / b).max(b / a);
        assert!(ratio(at(-6.0, 0.0, 96.0), 1e5) < 3.0, "civil {}", at(-6.0, 0.0, 96.0));
        assert!(ratio(at(-0.1, 0.0, 90.1), 3e7) < 3.0, "sunset {}", at(-0.1, 0.0, 90.1));
        assert!(at(-8.0, 85.0, 15.0) > 2.0 * at(-8.0, 85.0, 120.0));
    }

    /// Schaefer 1991 §3.2 与图 3：木星（−2.6 等）旁的伽利略卫星，天空 136 nL、kX = 0.2。原文结论：普通观测者（F_s = 1）
    /// 在最佳条件下也看不见任何一颗，极敏锐者（F_s = 0.15）在大距时四颗都能看见。卫星大距角距与冲日星等按原文图 3。
    #[test]
    fn glare_galilean_satellites_schaefer1991() {
        let sats = [(2.3, 5.0), (3.7, 5.3), (5.9, 4.6), (10.3, 5.6)]; // (角距′, V 等)：木卫一、二、三、四
        let mut tab = crate::test_util::CheckTable::new("schaefer1991_galilean", &["卫星", "大距角距′", "V", "背景 nL", "极限 F_s=1", "极限 F_s=0.15", "原文结论", "本实现"]);
        tab.meta("Schaefer 1991 §3.2 图 3：木星（−2.6 等）旁伽利略卫星肉眼可见性，天空 136 nL、kX = 0.2");
        for ((th, m), name) in sats.into_iter().zip(["木卫一", "木卫二", "木卫三", "木卫四"]) {
            let g = glare_nl(real(-2.6), real(th / 60.0), real(0.2));
            let b = real(136.0) + g.atmospheric + g.eye;
            let lim = |fs: f64| {
                let eye = Eye { acuity_factor: real(fs), threshold: ThresholdModel::Schaefer1993, ..Default::default() };
                limiting_magnitude(b, real(0.2), real(0.8), &eye).limiting_magnitude.as_f64()
            };
            tab.row(vec![
                name.into(),
                format!("{th}"),
                format!("{m}"),
                format!("{:.0}", b.as_f64()),
                format!("{:.2}", lim(1.0)),
                format!("{:.2}", lim(0.15)),
                "普通人不可见、极敏锐者可见".into(),
                format!("{}、{}", if lim(1.0) < m { "不可见" } else { "可见" }, if lim(0.15) > m { "可见" } else { "不可见" }),
            ]);
            assert!(lim(1.0) < m, "F_s = 1 不应可见：{th}′ 极限 {} vs {m}", lim(1.0));
            assert!(lim(0.15) > m, "F_s = 0.15 应可见：{th}′ 极限 {} vs {m}", lim(0.15));
            assert!(g.eye > real(3.0) * g.atmospheric, "原文：眼内散射为主");
        }
        tab.write();
    }

    /// Schaefer 1991 §3.1 与图 1：紧靠满月的天体肉眼极限约 0 等，紧靠上弦月约 2 等（天空 136 nL、kX = 0.3、月边外 0.3°）。
    #[test]
    fn glare_near_moon_schaefer1991_fig1() {
        let eye = Eye { threshold: ThresholdModel::Schaefer1993, ..Default::default() };
        let mut tab = crate::test_util::CheckTable::new("schaefer1991_fig1_near_moon", &["月相", "原文（图 1）", "本实现"]);
        tab.meta("Schaefer 1991 图 1：紧靠月亮（月边外约 0.3°）天体的肉眼极限星等，天空 136 nL、kX = 0.3");
        for (alpha, want) in [(0.0, 0.0), (90.0, 2.0)] {
            let g = glare_nl(moon_magnitude(real(alpha)), real(0.3), real(0.3));
            let lim = limiting_magnitude(real(136.0) + g.atmospheric + g.eye, real(0.3), real(0.7), &eye).limiting_magnitude.as_f64();
            tab.row(vec![if alpha == 0.0 { "满月".into() } else { "上弦".into() }, format!("约 {want} 等"), format!("{lim:.2} 等")]);
            if alpha == 90.0 {
                tab.write();
            }
            assert!((lim - want).abs() < 0.5, "α = {alpha}: {lim}");
        }
    }

    /// Schaefer 1987 式 2：标准天空（m_z = 6、k = 0.2）夜天光为 118 nL；同阈值模型反推应自洽。
    #[test]
    fn night_sky_from_zenith_limit_roundtrip() {
        for th in [ThresholdModel::Schaefer1993, ThresholdModel::Schaefer1998 { snellen: real(1.0) }, ThresholdModel::Crumey2014 { field_factor: real(2.0) }] {
            let eye = Eye { threshold: th, ..Default::default() };
            // Crumey 颜色修正只在暗视支施加，B = 7.08×10⁻² cd m⁻² 处极限星等跳变约 0.6 等，m_z 落在跳变区间时无精确解，故不取
            for (mz, k) in [(6.0, 0.2), (5.5, 0.3), (3.5, 0.15)] {
                let b0 = night_sky_from_zenith_limit(real(mz), real(k), &eye);
                let b = b0 * dex(real(-0.4 * k));
                let got = limiting_magnitude(b, real(k), ensemble_star_bv(), &eye).limiting_magnitude.as_f64();
                assert!((got - mz).abs() < 1e-6, "{th:?} {mz} {k}: {got}");
            }
        }
        // Hecht 暗视支（无颜色修正）与 1987 式 2 的 118 nL：(10^Y − 1)²/K，Y = −0.2(m_z + 16.57 + k + 2.5 lg C)
        let y = -0.2 * (6.0 + 16.57 + 0.2 + 2.5 * -9.8);
        let b118 = (10f64.powf(y) - 1.0).powi(2) / 10f64.powf(-1.9);
        assert!((b118 - 118.0).abs() < 1.0, "{b118}");
    }
}
