//! 时长（带量纲 D_TIME）。内部与 API 均用 Real。

use std::ops::{Add, Sub};

use super::dimension::{Dimension, Quantity};
use crate::math::real::{real, Real, RealOps};

/// 时长（秒，SI）。可正可负。
#[derive(Clone, Copy, Debug)]
pub struct Duration(Quantity);

impl Duration {
    pub fn from_quantity(q: Quantity) -> Result<Self, &'static str> {
        if q.dimension != Dimension::D_TIME {
            return Err("量纲须为时间");
        }
        Ok(Self(q))
    }

    pub fn in_seconds(seconds: Real) -> Self {
        Self(Quantity::new(seconds, Dimension::D_TIME))
    }

    pub fn in_days(days: Real) -> Self {
        let sec_per_day = real(86400.0);
        Self::in_seconds(days * sec_per_day)
    }

    pub fn seconds(self) -> Real {
        self.0.value
    }

    pub fn in_days_value(self) -> Real {
        let sec_per_day = real(86400.0);
        self.0.value / sec_per_day
    }

    pub fn to_quantity(self) -> Quantity {
        self.0
    }

    pub fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }

    pub fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }

    pub fn scale(self, s: Real) -> Self {
        Self(self.0.scale(s))
    }

    pub fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl Add for Duration {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Sub for Duration {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self::in_seconds(Real::zero())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::real::{real, RealOps};

    #[test]
    fn duration_from_quantity_and_seconds() {
        let q = crate::quantity::dimension::Quantity::new(real(100.0), crate::quantity::dimension::Dimension::D_TIME);
        let d = Duration::from_quantity(q).unwrap();
        assert!(d.seconds().is_near(real(100.0), 1e-10));
        let bad = crate::quantity::dimension::Quantity::new(real(1.0), crate::quantity::dimension::Dimension::D_LENGTH);
        assert!(Duration::from_quantity(bad).is_err());
    }

    #[test]
    fn duration_in_seconds_in_days_roundtrip() {
        let one_day = Duration::in_days(real(1.0));
        assert!(one_day.seconds().is_near(real(86400.0), 1e-10));
        assert!(one_day.in_days_value().is_near(real(1.0), 1e-10));
    }

    #[test]
    fn duration_add_sub_scale_neg() {
        let a = Duration::in_seconds(real(10.0));
        let b = Duration::in_seconds(real(3.0));
        assert!(a.add(b).seconds().is_near(real(13.0), 1e-10));
        assert!(a.sub(b).seconds().is_near(real(7.0), 1e-10));
        assert!(a.scale(real(2.0)).seconds().is_near(real(20.0), 1e-10));
        assert!(a.neg().seconds().is_near(real(-10.0), 1e-10));
    }

    #[test]
    fn duration_add_sub_trait_default() {
        let a = Duration::in_seconds(real(5.0));
        let b = Duration::in_seconds(real(2.0));
        assert!((a + b).seconds().is_near(real(7.0), 1e-10));
        assert!((a - b).seconds().is_near(real(3.0), 1e-10));
        let z = Duration::default();
        assert!(z.seconds().is_near(real(0.0), 1e-10));
    }
}

