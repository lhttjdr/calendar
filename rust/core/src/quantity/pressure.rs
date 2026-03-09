//! 压强（量纲 M L⁻¹ T⁻²，SI 帕斯卡 Pa）。内部与 API 均用 Real。

use super::dimension::{Dimension, Quantity};
use crate::math::real::{real, Real};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pressure(Quantity);

impl Pressure {
    pub fn from_quantity(q: Quantity) -> Result<Self, &'static str> {
        if q.dimension != Dimension::D_PRESSURE {
            return Err("量纲须为压强");
        }
        Ok(Self(q))
    }

    pub fn from_pa(pa: Real) -> Self {
        Self(Quantity::new(pa, Dimension::D_PRESSURE))
    }

    /// 千帕（kPa），大气折射常用。
    pub fn from_kpa(kpa: Real) -> Self {
        Self::from_pa(kpa * real(1000.0))
    }

    pub fn pa(self) -> Real {
        self.0.value
    }

    pub fn kpa(self) -> Real {
        self.0.value / real(1000.0)
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
    fn pressure_from_quantity_from_pa_from_kpa() {
        let q = Quantity::new(real(101325.0), Dimension::D_PRESSURE);
        let p = Pressure::from_quantity(q).unwrap();
        assert!(p.pa().is_near(real(101325.0), 1e-10));
        assert!(Pressure::from_quantity(Quantity::new(real(1.0), Dimension::D_LENGTH)).is_err());
        let p2 = Pressure::from_pa(real(1000.0));
        assert!(p2.pa().is_near(real(1000.0), 1e-10));
        let p3 = Pressure::from_kpa(real(1.0));
        assert!(p3.kpa().is_near(real(1.0), 1e-10));
        assert!(p3.pa().is_near(real(1000.0), 1e-10));
    }
}
