//! 章动（IAU 2000B 77 项；可选加载 IAU2000A 完整月日项）。有 tab5.3a 时默认启用完整章动；迭代粗算阶段仍用 77 项。
//! 完整版与星历表一样通过 DataLoader + 路径加载数据表（[`load::load_iau2000a`]）。

pub mod table_parser;
pub mod iau2000a;
pub mod load;
mod iau2000b;
pub use iau2000b::*;
pub use load::{load_iau2000a, load_iau2000a_from_binary, DEFAULT_TAB53A_BIN_PATH, DEFAULT_TAB53A_PATH};
