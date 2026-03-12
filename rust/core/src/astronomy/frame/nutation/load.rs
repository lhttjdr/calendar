//! 章动数据表：仅保留 IERS 5.3a/5.3b 解析；读写由 [repo](crate::repo) 共通模块负责。

use super::iau2000a::Iau2000a;
use super::table_parser;
use crate::platform::{DataLoader, LoadError};
use crate::repo::paths;

/// IERS 表 5.3a 默认路径（经度，µas），相对项目根或资源根。
pub const DEFAULT_TAB53A_PATH: &str = paths::IAU2000_TAB53A;
/// IERS 表 5.3b 默认路径（倾角，µas）。
pub const DEFAULT_TAB53B_PATH: &str = paths::IAU2000_TAB53B;

/// 默认章动二进制路径（可选；.bin 或解压后的 .br）。
pub const DEFAULT_TAB53A_BIN_PATH: &str = paths::IAU2000_TAB53A_BIN;

/// 从已读入的 5.3a/5.3b 文本行解析并合并为 IAU2000A（仅 parser，无 I/O）。
pub fn parse_iau2000a_from_iers_lines(
    lines_a: &[String],
    lines_b: &[String],
) -> Result<Iau2000a, LoadError> {
    let a0 = table_parser::load_iers_53a_j0_keys_and_coeffs(lines_a);
    if a0.is_empty() {
        return Err(LoadError::Io("IERS tab5.3a j=0 luni-solar empty".to_string()));
    }
    let a1 = table_parser::load_iers_53a_j1_map(lines_a);
    let b0 = table_parser::load_iers_53b_j0_map(lines_b);
    let b1 = table_parser::load_iers_53b_j1_map(lines_b);
    let quads = table_parser::merge_iers_53a_53b_to_quads(a0, &a1, &b0, &b1);
    Ok(Iau2000a::from_quads(quads))
}

/// 从 DataLoader 加载章动（注入用，如 Wasm）；读由调用方/宿主提供。
pub fn load_iau2000a(loader: &dyn DataLoader, path_53a: &str, path_53b: &str) -> Result<Iau2000a, LoadError> {
    let lines_a = loader.read_lines(path_53a)?;
    let lines_b = loader.read_lines(path_53b)?;
    parse_iau2000a_from_iers_lines(&lines_a, &lines_b)
}

/// 从「repo」读 5.3a+5.3b 并解析（Native=本地文件，Wasm=宿主 set_loader 注入的 fetch 结果）。
/// 先尝试 IERS 原表格式（j=0/j=1 段落）；若 5.3a 无 j=0 段（如 VLBI/脚本合并格式），则用单文件 VLBI 解析。
pub fn load_iau2000a_from_repo() -> Result<Iau2000a, LoadError> {
    let lines_a = crate::repo::read_lines(paths::IAU2000_TAB53A)?;
    match parse_iau2000a_from_iers_lines(&lines_a, &[]) {
        Ok(iau) if iau.term_count() > 0 => return Ok(iau),
        _ => {}
    }
    let lines_b = match crate::repo::read_lines(paths::IAU2000_TAB53B) {
        Ok(l) => l,
        Err(_) => Vec::new(),
    };
    if !lines_b.is_empty() {
        if let Ok(iau) = parse_iau2000a_from_iers_lines(&lines_a, &lines_b) {
            if iau.term_count() > 0 {
                return Ok(iau);
            }
        }
    }
    let quads = table_parser::parse_vlbi_merged_to_quads(&lines_a);
    if quads.is_empty() {
        return Err(LoadError::Io(
            "IERS tab5.3a j=0 luni-solar empty and VLBI merged format had no rows".to_string(),
        ));
    }
    Ok(Iau2000a::from_quads(quads))
}

/// 从二进制 buffer 加载（.bin 或解压后的 .br）。
pub fn load_iau2000a_from_binary(bytes: &[u8]) -> Result<Iau2000a, LoadError> {
    Iau2000a::from_binary(bytes)
}
