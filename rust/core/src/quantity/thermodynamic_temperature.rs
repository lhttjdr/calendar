//! 热力学温度（量纲 Θ，SI 开尔文 K）。内部与 API 均用 Real。

use super::dimension::{Dimension, Quantity};
use crate::math::real::{real, Real};

const ZERO_CELSIUS_KELVIN: f64 = 273.15;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThermodynamicTemperature(Quantity);

impl ThermodynamicTemperature {
    pub fn from_quantity(q: Quantity) -> Result<Self, &'static str> {
        if q.dimension != Dimension::D_THERMODYNAMIC_TEMPERATURE {
            return Err("量纲须为热力学温度");
        }
        Ok(Self(q))
    }

    pub fn from_kelvin(k: Real) -> Self {
        Self(Quantity::new(k, Dimension::D_THERMODYNAMIC_TEMPERATURE))
    }

    /// 摄氏温度（°C）转热力学温度（K）。
    pub fn from_celsius(t_c: Real) -> Self {
        Self::from_kelvin(t_c + real(ZERO_CELSIUS_KELVIN))
    }

    pub fn kelvin(self) -> Real {
        self.0.value
    }

    /// 热力学温度转摄氏温度。
    pub fn celsius(self) -> Real {
        self.0.value - real(ZERO_CELSIUS_KELVIN)
    }

    pub fn to_quantity(self) -> Quantity {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::{real, RealOps};

    #[test]
    fn thermodynamic_temperature_from_quantity_kelvin_celsius() {
        let q = Quantity::new(real(300.0), Dimension::D_THERMODYNAMIC_TEMPERATURE);
        let t = ThermodynamicTemperature::from_quantity(q).unwrap();
        assert!(t.kelvin().is_near(real(300.0), 1e-10));
        assert!(ThermodynamicTemperature::from_quantity(Quantity::new(real(1.0), Dimension::D_LENGTH)).is_err());
        let t2 = ThermodynamicTemperature::from_kelvin(real(273.15));
        assert!(t2.celsius().is_near(real(0.0), 1e-10));
        let t3 = ThermodynamicTemperature::from_celsius(real(0.0));
        assert!(t3.kelvin().is_near(real(273.15), 1e-10));
    }
}
