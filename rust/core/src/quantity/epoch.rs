//! 历元：作为动态参考架的“日期”参数，与 TimePoint 分工明确但有对应关系。
//!
//! - **TimePoint**（`astronomy::time`）：带时标的**时刻**，表示“何时”（TT/TDB/UTC 等），用于历表求值、光行时、API 入参；可做时标换算、加减时长。
//! - **Epoch**（本模块）：**架所绑定的历元**（“of date”里的 date），用于 `ReferenceFrame::MeanEquator(epoch)`、`ApparentEcliptic(epoch)` 等；存 TT 儒略日，不携带时标。
//!
//! **关系**：历元约定为 TT 下的时刻，故与 TimePoint 可互转。标量用 Real。

use crate::math::real::{real, Real};

use super::julian_centuries::JulianCenturies;

/// 历元：儒略日，作为动态参考架的自变量。约定为 TT 儒略日（与岁差/章动等 of-date 架一致）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Epoch {
    pub jd: Real,
}

impl Epoch {
    pub fn new(jd: Real) -> Self {
        Self { jd }
    }

    /// J2000.0：JD 2451545.0
    pub fn j2000() -> Self {
        Self::new(real(2451545.0))
    }

    /// 相对 J2000 的儒略世纪数（无量纲物理量），用于历表幂级数 T。
    pub fn offset_in_julian_centuries(
        self,
        j2000_jd: Real,
        days_per_julian_century: Real,
    ) -> JulianCenturies {
        JulianCenturies::from_value((self.jd - j2000_jd) / days_per_julian_century)
    }
}
