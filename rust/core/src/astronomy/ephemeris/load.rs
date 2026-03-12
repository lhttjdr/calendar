use super::vsop87::{Vsop87, Vsop87Parse};
use crate::platform::LoadError;
use crate::repo::paths;

pub const DEFAULT_VSOP87_EARTH_PATH: &str = paths::VSOP87_EARTH;
pub const DEFAULT_ELPMPP02_PATH: &str = paths::ELPMPP02;

/// 从 DataLoader 加载（注入用，如 Wasm）。
pub fn load_earth_vsop87(
    loader: &dyn crate::platform::DataLoader,
    path: &str,
) -> Result<Vsop87, LoadError> {
    Vsop87Parse::parse(loader, path)
}

/// 从「repo」读 VSOP87 地球系数并解析（Native=本地文件，Wasm=宿主 set_loader 注入）。
pub fn load_earth_vsop87_from_repo() -> Result<Vsop87, LoadError> {
    let lines = crate::repo::read_lines(paths::VSOP87_EARTH)?;
    Vsop87Parse::parse_from_lines(&lines)
}
