//! 章动数据表加载（与星历表同一方式：DataLoader + 路径；bin/解压后 br 用 from_binary）。

use super::iau2000a::Iau2000a;
use super::table_parser;
use crate::platform::{DataLoader, LoadError};

/// 默认 tab5.3a 路径，与星历表路径约定一致（相对项目根或资源根）。
pub const DEFAULT_TAB53A_PATH: &str = "data/IAU2000/tab5.3a.txt";

/// 默认 tab5.3a 二进制路径（可选；.br 由前端解压后传入 `load_iau2000a_from_binary`）。
pub const DEFAULT_TAB53A_BIN_PATH: &str = "data/IAU2000/tab5.3a.bin";

/// 从 DataLoader 加载并解析 tab5.3a 文本，返回完整 IAU2000A 章动模型。与 `load_earth_vsop87` 等星历表加载方式一致。
pub fn load_iau2000a(
    loader: &dyn DataLoader,
    path: &str,
) -> Result<Iau2000a, LoadError> {
    let quads = table_parser::load_tab53a(loader, path)?;
    if quads.is_empty() {
        return Err(LoadError::Io("tab5.3a empty".to_string()));
    }
    Ok(Iau2000a::from_quads(quads))
}

/// 从二进制 buffer 加载（.bin 或解压后的 .br），与 `Vsop87::from_binary`、`load_all_from_binary` 一致。
pub fn load_iau2000a_from_binary(bytes: &[u8]) -> Result<Iau2000a, LoadError> {
    Iau2000a::from_binary(bytes)
}
