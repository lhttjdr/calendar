//! 大气折射公式：Bennett（默认）、Bennett 改进、Saemundsson、Smart、Meeus、XJW。

mod factor;
pub mod bennett;
pub mod bennett_improved;
pub mod saemundsson;
pub mod smart;
pub mod meeus;
pub mod xjw;

pub use bennett::{bennett_refraction, bennett_refraction_default};
pub use bennett_improved::{bennett_improved_refraction, bennett_improved_refraction_default};
pub use saemundsson::saemundsson_refraction;
pub use smart::smart_refraction;
pub use meeus::meeus_refraction;
pub use xjw::xjw_refraction;

#[cfg(test)]
mod tests;
