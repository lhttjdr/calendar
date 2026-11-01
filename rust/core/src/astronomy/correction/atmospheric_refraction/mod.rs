//! 大气折射：拟合公式 Bennett、Bennett 改进、Saemundsson、Smart、Meeus、XJW，及分层大气光线追迹。
//!
//! 各公式入参不同：Bennett、Bennett 改进、Smart 为视高度，Saemundsson、Meeus、XJW 为真（几何）高度。
//! 由真高度求折射统一用 [`refraction_from_geometric`]，它按 [`AltitudeBasis`] 处理输入并保证地平附近连续。

mod factor;
mod apparent_altitude;
pub mod ray_trace;
pub mod bennett;
pub mod bennett_improved;
pub mod saemundsson;
pub mod smart;
pub mod meeus;
pub mod xjw;

pub use apparent_altitude::{refraction_from_geometric, refract_enu, AltitudeBasis, Atmosphere, Inversion, RefractionModel};
pub use ray_trace::{ray_trace_refraction_apparent, ray_trace_sea_horizon, RayTraceConstants};
pub use bennett::{bennett_refraction, bennett_refraction_default};
pub use bennett_improved::{bennett_improved_refraction, bennett_improved_refraction_default};
pub use saemundsson::saemundsson_refraction;
pub use smart::smart_refraction;
pub use meeus::meeus_refraction;
pub use xjw::xjw_refraction;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_checks;
