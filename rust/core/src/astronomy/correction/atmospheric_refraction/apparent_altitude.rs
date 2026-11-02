//! 由真（几何）高度求折射：统一各公式的输入基准，并保证地平附近连续。

use crate::math::real::{real, real_const, zero, Real, RealOps};
use crate::quantity::angle::PlaneAngle;
use crate::quantity::length::Length;
use crate::quantity::unit::LengthUnit;
use crate::quantity::{pressure::Pressure, thermodynamic_temperature::ThermodynamicTemperature};

use super::bennett::bennett_arcmin;
use super::bennett_improved::bennett_improved_arcmin;
use super::factor::pressure_temperature_factor;
use super::meeus::meeus_arcsec;
use super::ray_trace::{ray_trace_refraction_apparent, ray_trace_refraction_geometric, RayTraceConstants};
use super::saemundsson::saemundsson_arcmin;
use super::smart::smart_arcsec;

/// tan z 多项式（Smart、Meeus）的适用下限 [°]；地平附近其 tan³z 项发散。
const POLYNOMIAL_MIN_ALTITUDE_DEG: f64 = 15.0;
/// 真高度低于此值 [°] 开始把折射线性收到零，到 `FADE_END_DEG` 为零。
const FADE_START_DEG: f64 = -1.0;
const FADE_END_DEG: f64 = -2.0;
/// 视高度式不动点迭代的收敛容差 [°]，约 4e-6″。
const SOLVE_TOLERANCE_DEG: f64 = 1e-9;

/// 公式的输入基准。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AltitudeBasis {
    /// 入参为视高度，由真高度求折射要迭代。
    Apparent,
    /// 入参为真（几何）高度，可直接求。
    Geometric,
}

/// 折射模型。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefractionModel {
    Bennett,
    BennettImproved,
    Saemundsson,
    Smart,
    Meeus,
    /// 分层大气光线追迹，唯一用到温度廓线（递减率、逆温）与观测站海拔的模型，见 [`super::ray_trace`]。
    RayTrace,
}

impl RefractionModel {
    pub const ALL: [RefractionModel; 6] = [
        RefractionModel::Bennett,
        RefractionModel::BennettImproved,
        RefractionModel::Saemundsson,
        RefractionModel::Smart,
        RefractionModel::Meeus,
        RefractionModel::RayTrace,
    ];

    /// 管线图重边的 key。
    pub fn edge_key(self) -> &'static str {
        match self {
            RefractionModel::Bennett => "Bennett",
            RefractionModel::BennettImproved => "BennettImproved",
            RefractionModel::Saemundsson => "Saemundsson",
            RefractionModel::Smart => "Smart",
            RefractionModel::Meeus => "Meeus",
            RefractionModel::RayTrace => "RayTrace",
        }
    }

    pub fn from_edge_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.edge_key() == key)
    }

    pub fn basis(self) -> AltitudeBasis {
        match self {
            RefractionModel::Bennett | RefractionModel::BennettImproved | RefractionModel::Smart | RefractionModel::RayTrace => {
                AltitudeBasis::Apparent
            }
            RefractionModel::Saemundsson | RefractionModel::Meeus => AltitudeBasis::Geometric,
        }
    }

    /// 只适用于较高高度的模型返回下限 [°]；None 表示可用到地平。
    pub fn min_altitude_deg(self) -> Option<f64> {
        match self {
            RefractionModel::Smart | RefractionModel::Meeus => Some(POLYNOMIAL_MIN_ALTITUDE_DEG),
            _ => None,
        }
    }

    /// 公式本身（无气压气温因子），入参按 [`Self::basis`]，单位 °，结果单位 °。
    fn formula_deg(self, altitude_deg: Real) -> Real {
        let d2r = Real::pi() / real_const(180.0);
        match self {
            RefractionModel::Bennett => bennett_arcmin(altitude_deg) / real_const(60.0),
            RefractionModel::BennettImproved => bennett_improved_arcmin(altitude_deg) / real_const(60.0),
            RefractionModel::Saemundsson => saemundsson_arcmin(altitude_deg) / real_const(60.0),
            RefractionModel::Smart => smart_arcsec(altitude_deg * d2r) / real_const(3600.0),
            RefractionModel::Meeus => meeus_arcsec(altitude_deg * d2r) / real_const(3600.0),
            RefractionModel::RayTrace => unreachable!("光线追迹不走拟合式"),
        }
    }
}

/// 贴地逆温层：从观测站往上 `depth` 内，温度以 `gradient_k_per_m` 升高。
#[derive(Clone, Copy, Debug)]
pub struct Inversion {
    pub depth: Length,
    pub gradient_k_per_m: Real,
}

/// 标准递减率 [K/m]。
const STANDARD_LAPSE_K_PER_M: Real = real_const(-0.0065);
/// 气压标高指数 g/(R·|L|)，由此 p ∝ T^指数。
const STANDARD_PRESSURE_EXPONENT: Real = real_const(5.2559);

/// 观测站的大气状态。拟合式只用气压、温度；递减率、海拔、逆温只有 [`RefractionModel::RayTrace`] 用到。
#[derive(Clone, Copy, Debug)]
pub struct Atmosphere {
    pub pressure: Pressure,
    pub temperature: ThermodynamicTemperature,
    /// 观测站海拔。
    pub height: Length,
    pub lapse_k_per_m: Real,
    pub inversion: Option<Inversion>,
}

/// 海平面 101 kPa、10°C（283.15 K；公式参考温度取 283 K，故因子为 0.9995，与现有缺省函数一致）。
impl Default for Atmosphere {
    fn default() -> Self {
        Self::standard_at_height(Length::from_value(zero(), LengthUnit::Meter))
    }
}

impl Atmosphere {
    /// 海平面 101 kPa、10°C 的标准大气在海拔 `height` 处的状态：温度按标准递减率，气压按静力平衡。
    pub fn standard_at_height(height: Length) -> Self {
        let sea_level_k = real_const(283.15);
        let t = sea_level_k + STANDARD_LAPSE_K_PER_M * height.meters();
        let p = real_const(101.0) * ((t / sea_level_k).ln() * STANDARD_PRESSURE_EXPONENT).exp();
        Self {
            pressure: Pressure::from_kpa(p),
            temperature: ThermodynamicTemperature::from_kelvin(t),
            height,
            lapse_k_per_m: STANDARD_LAPSE_K_PER_M,
            inversion: None,
        }
    }

    pub fn with_inversion(mut self, depth: Length, gradient_k_per_m: Real) -> Self {
        self.inversion = Some(Inversion { depth, gradient_k_per_m });
        self
    }
}

/// 真高度 [°] → 折射 [°]，含气压气温因子，不含地平下收束。
fn refraction_deg(h_deg: Real, model: RefractionModel, f: Real) -> Real {
    match model.basis() {
        AltitudeBasis::Geometric => model.formula_deg(h_deg) * f,
        AltitudeBasis::Apparent => {
            // 解 h_a = h + R(h_a)；|dR/dh_a| 远小于 1，不动点迭代收敛很快
            let mut r = zero();
            for _ in 0..30 {
                let next = model.formula_deg(h_deg + r) * f;
                let done = (next - r).abs() < real(SOLVE_TOLERANCE_DEG);
                r = next;
                if done {
                    break;
                }
            }
            r
        }
    }
}

/// 由**真（几何）高度**求折射量，视高度 = 真高度 + 折射量。
///
/// 地平以下仍连续：真高度到 −1° 照公式计算（观测点较高时可见地平之下，−1° 约相当于海拔 1000 m 的地平俯角）；
/// −1° 到 −2° 线性收到零，−2° 以下为零。这段只保证连续、单调，便于求根，并非地平下的大气模型。
/// 天顶附近公式给出的微小负值取零。
///
/// Smart、Meeus 的多项式在 15° 以下不适用，此时 panic，不做隐式降级。
pub fn refraction_from_geometric(h_geom: PlaneAngle, model: RefractionModel, atmosphere: &Atmosphere) -> PlaneAngle {
    let r2d = real_const(180.0) / Real::pi();
    let h_deg = h_geom.rad() * r2d;
    if let Some(min) = model.min_altitude_deg() {
        // 留一点余量，弧度来回换算的舍入不应让恰好 15° 的输入失败
        assert!(h_deg >= real(min - 1e-9), "折射模型 {model:?} 只适用于高度 ≥ {min}°，实为 {}°", h_deg.as_f64());
    }
    if model == RefractionModel::RayTrace {
        let fade = real(FADE_START_DEG - FADE_END_DEG) / r2d;
        return PlaneAngle::from_rad(ray_trace_refraction_geometric(h_deg / r2d, fade, atmosphere));
    }
    if h_deg <= real(FADE_END_DEG) {
        return PlaneAngle::from_rad(zero());
    }
    let f = pressure_temperature_factor(atmosphere.pressure, atmosphere.temperature) * Real::one();
    let r_deg = if h_deg >= real(FADE_START_DEG) {
        refraction_deg(h_deg, model, f)
    } else {
        refraction_deg(real(FADE_START_DEG), model, f) * (h_deg - real(FADE_END_DEG)) / real(FADE_START_DEG - FADE_END_DEG)
    };
    PlaneAngle::from_rad(r_deg.max(zero()) / r2d)
}

/// 由**视高度**求折射量，真高度 = 视高度 − 折射量，是 [`refraction_from_geometric`] 的反解。
/// 光线追迹以视高度为入参，直接求；视线撞地时 panic。其余模型反复代入正向公式，保证来回一致。
pub fn refraction_from_apparent(h_app: PlaneAngle, model: RefractionModel, atmosphere: &Atmosphere) -> PlaneAngle {
    if model == RefractionModel::RayTrace {
        let r = ray_trace_refraction_apparent(h_app.rad(), atmosphere, &RayTraceConstants::default())
            .unwrap_or_else(|| panic!("视高度 {} rad 的视线撞地，没有对应的天体方向", h_app.rad().as_f64()));
        return PlaneAngle::from_rad(r);
    }
    let mut r = zero();
    for _ in 0..50 {
        let next = refraction_from_geometric(PlaneAngle::from_rad(h_app.rad() - r), model, atmosphere).rad();
        let done = (next - r).abs() < real(SOLVE_TOLERANCE_DEG) * Real::pi() / real_const(180.0);
        r = next;
        if done {
            break;
        }
    }
    PlaneAngle::from_rad(r)
}

/// 站心地平直角坐标（ENU：东、北、天顶）施加折射：方位与距离不变，高度角加折射量。天顶处方位无定义，原样返回。
pub fn refract_enu(enu: [Real; 3], model: RefractionModel, atmosphere: &Atmosphere) -> [Real; 3] {
    let [e, n, u] = enu;
    let rho = (e * e + n * n).sqrt();
    if rho == zero() {
        return enu;
    }
    let dist = (rho * rho + u * u).sqrt();
    let h = u.atan2(rho);
    let h_app = h + refraction_from_geometric(PlaneAngle::from_rad(h), model, atmosphere).rad();
    let scale = dist * h_app.cos() / rho;
    [e * scale, n * scale, dist * h_app.sin()]
}

/// [`refract_enu`] 的反解：视方向 → 几何方向，方位与距离不变。
pub fn unrefract_enu(enu: [Real; 3], model: RefractionModel, atmosphere: &Atmosphere) -> [Real; 3] {
    let [e, n, u] = enu;
    let rho = (e * e + n * n).sqrt();
    if rho == zero() {
        return enu;
    }
    let dist = (rho * rho + u * u).sqrt();
    let h_app = u.atan2(rho);
    let h = h_app - refraction_from_apparent(PlaneAngle::from_rad(h_app), model, atmosphere).rad();
    let scale = dist * h.cos() / rho;
    [e * scale, n * scale, dist * h.sin()]
}
