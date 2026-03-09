//! 章动（IAU 2000B 77 项；可选加载 IAU2000A 完整月日项）。

pub mod table_parser;
#[cfg(not(target_arch = "wasm32"))]
pub mod iau2000a;
mod iau2000b;
pub use iau2000b::*;
