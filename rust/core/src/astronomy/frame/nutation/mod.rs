//! 章动（IAU 2000A 与 2000B 共用一套实现：从 data/IAU2000/tab5.3a.txt + tab5.3b.txt 双表加载并内存合并，2000B 为前 77 项）。
//!
//! ## IAU 2000B（77 项）与 IERS 规范
//! - **IAU 2000B**：IAU 2000 年通过的**简版**岁差章动模型，仅含 **77 个月日章动项**（MHB_2000_SHORT 序列），
//!   精度约 1 mas（1900–2100），无行星项级数、常用固定偏移近似行星效应。见 McCarthy & Luzum (2001)、IERS Conventions Ch.5。
//! - **IERS** 官方只提供完整 **Table 5.3a/5.3b**（IAU 2000A，千余项）；**没有单独的「77 项表」**。
//!   77 项 = 完整表中按标准顺序取前 77 个月日项，与 SOFA `iauNut00b` 对应。
//!
//! 应用初始化时调用 [`try_init_full_nutation`] 或 [`try_init_nutation`] 加载 IERS 5.3a+5.3b；77 项为合并表前 77 行，完整版为全表。

pub mod table_parser;
pub mod iau2000a;
pub mod load;
mod iau2000b;
pub use iau2000b::*;
pub use load::{
    load_iau2000a, load_iau2000a_from_binary, load_iau2000a_from_repo, parse_iau2000a_from_iers_lines,
    DEFAULT_TAB53A_BIN_PATH, DEFAULT_TAB53A_PATH, DEFAULT_TAB53B_PATH,
};
