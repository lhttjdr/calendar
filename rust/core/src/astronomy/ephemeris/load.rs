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

/// 从「repo」读某行星的 VSOP87B（日心球坐标，J2000 动力学黄道与春分点）。
pub fn load_planet_vsop87_from_repo(planet: crate::astronomy::pipeline::Planet) -> Result<Vsop87, LoadError> {
    let lines = crate::repo::read_lines(&format!("{}/VSOP87B.{}", paths::VSOP87_DIR, planet.vsop87_suffix()))?;
    Vsop87Parse::parse_from_lines(&lines)
}

/// 地球 VSOP87B，施加拟合到指定 JPL 历表的日心 L、B、R 改正（[`super::vsop87_patch`]）。节气、定朔、天象默认用此。
pub fn load_earth_vsop87_corrected_from_repo(target: super::vsop87_patch::PatchTarget) -> Result<Vsop87, LoadError> {
    let patch = super::vsop87_patch::Vsop87DePatch::from_repo(target)?;
    let lbr = patch.body("ear").cloned().ok_or_else(|| LoadError::Parse(format!("{} 无地球段", target.repo_path())))?;
    Ok(load_earth_vsop87_from_repo()?.with_patch(lbr))
}

/// 行星 VSOP87B，施加拟合到指定 JPL 历表的改正。
pub fn load_planet_vsop87_corrected_from_repo(
    planet: crate::astronomy::pipeline::Planet,
    target: super::vsop87_patch::PatchTarget,
) -> Result<Vsop87, LoadError> {
    let patch = super::vsop87_patch::Vsop87DePatch::from_repo(target)?;
    let lbr = patch
        .body(planet.vsop87_suffix())
        .cloned()
        .ok_or_else(|| LoadError::Parse(format!("{} 无 {} 段", target.repo_path(), planet.vsop87_suffix())))?;
    Ok(load_planet_vsop87_from_repo(planet)?.with_patch(lbr))
}
