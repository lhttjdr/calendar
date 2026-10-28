//! 天象（天文域第 3 层派生层）：日月食、行星会合现象、近距事件、伏见（可见性）、月掩星等。只输出客观时刻与几何量，不含历法或占星术语。
//! 设计见 doc/astronomy/celestial-phenomena.md。

pub mod close_approach;
pub mod heliacal;
pub mod lunar_eclipse;
pub mod occultation;
pub mod planet_events;
pub mod search;
pub mod solar_eclipse;
pub(crate) mod track;
pub mod visibility;

pub use close_approach::*;
pub use heliacal::*;
pub use lunar_eclipse::*;
pub use occultation::*;
pub use planet_events::*;
pub use solar_eclipse::*;
pub use visibility::{ExtinctionModel, Eye, ScatteringFunction, SkyConditions, SkyModel, ThresholdModel, TwilightDirectionTerm, TwilightModel};

#[cfg(test)]
mod tests_close_approach;
#[cfg(test)]
mod tests_heliacal;
#[cfg(test)]
mod tests_nasa;
#[cfg(test)]
mod tests_occultation;
#[cfg(test)]
mod tests_planet_events;
