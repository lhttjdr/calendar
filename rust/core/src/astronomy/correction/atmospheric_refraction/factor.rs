//! 大气折射的气压–温度修正因子计算。物理量类型在 quantity 层，此处仅根据气压与温度算出因子值。

use crate::math::real::{real_const, Real};
use crate::quantity::dimensionless::Dimensionless;
use crate::quantity::pressure_temperature_factor::PressureTemperatureFactor;
use crate::quantity::{pressure::Pressure, thermodynamic_temperature::ThermodynamicTemperature};

/// 参考气压 101 kPa、参考温度 283 K（约 10°C），折射公式常用。
const P_REF_KPA: Real = real_const(101.0);
const T_REF_K: Real = real_const(283.0);

/// 气压–温度因子 (p/p_ref)(T_ref/T)，具名无量纲物理量。用物理量的乘除运算得到。
pub(crate) fn pressure_temperature_factor(
    pressure: Pressure,
    temperature: ThermodynamicTemperature,
) -> PressureTemperatureFactor {
    let p_ref = Pressure::from_kpa(P_REF_KPA);
    let t_ref = ThermodynamicTemperature::from_kelvin(T_REF_K);
    let p_ratio = pressure.to_quantity() / p_ref.to_quantity();
    let t_ratio = t_ref.to_quantity() / temperature.to_quantity();
    let dim = Dimensionless::from_quantity(p_ratio * t_ratio).expect("无量纲×无量纲应为无量纲");
    PressureTemperatureFactor::from_dimensionless(dim)
}
