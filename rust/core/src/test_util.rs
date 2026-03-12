//! 测试专用：测试用数据文件路径与解析辅助。
//!
//! 本模块仅随 `cargo test` 编译，**不会编入库或 wasm 等最终产物**。
//! 测试中需要仓库根、BSP、参考表等路径时，可统一通过此处获取，避免在各测试里重复拼路径。

#![cfg(test)]
#[allow(dead_code)] // 供各测试按需选用，未全部迁移前会有未使用

use std::path::Path;

/// 测试用仓库根目录（与 [crate::repo::repo_root] 一致，仅限 test 下使用）。
#[cfg(not(target_arch = "wasm32"))]
pub fn base() -> std::path::PathBuf {
    crate::repo::repo_root()
}

/// 解析 DE406 BSP 路径：环境变量 `DE406_BSP`（若为文件）→ 候选路径 [0] → [1] → 否则 `data/jpl` 目录路径（用于 skip 提示）。
/// 返回的路径不保证存在，调用方需自行 `Path::is_file()` 判断。
#[cfg(not(target_arch = "wasm32"))]
pub fn de406_bsp_path() -> String {
    let base = base();
    std::env::var("DE406_BSP")
        .ok()
        .filter(|p| Path::new(p).is_file())
        .or_else(|| {
            let p = base.join(crate::repo::paths::DE406_BSP_CANDIDATES[0]);
            if p.is_file() {
                Some(p.to_string_lossy().into_owned())
            } else {
                base.join(crate::repo::paths::DE406_BSP_CANDIDATES[1])
                    .is_file()
                    .then(|| base.join(crate::repo::paths::DE406_BSP_CANDIDATES[1]).to_string_lossy().into_owned())
            }
        })
        .unwrap_or_else(|| base.join(crate::repo::paths::JPL_DATA_DIR).to_string_lossy().into_owned())
}

/// DE406 数据目录或 BSP 所在目录的路径（用于 Python/jplephem 等传目录的场景）。
#[cfg(not(target_arch = "wasm32"))]
pub fn de406_ephem_path() -> String {
    std::env::var("DE406_PATH")
        .unwrap_or_else(|_| base().join(crate::repo::paths::JPL_DATA_DIR).to_string_lossy().into_owned())
}

/// 节气朔望参考表完整路径（《月相和二十四节气的计算》§7.4 TDBtimes.txt）。
#[cfg(not(target_arch = "wasm32"))]
pub fn solar_terms_reference_path() -> std::path::PathBuf {
    base().join(crate::repo::paths::SOLAR_TERMS_REFERENCE)
}

/// ELPMPP02 vs JPL DE406 样本 CSV 完整路径。
#[cfg(not(target_arch = "wasm32"))]
pub fn jpl_elp_vs_jpl_samples_csv_path() -> std::path::PathBuf {
    base().join(crate::repo::paths::JPL_ELP_VS_JPL_SAMPLES_CSV)
}

/// 若 BSP 路径指向的文件不存在，返回 `None`；否则返回 `Some(path)`。
#[cfg(not(target_arch = "wasm32"))]
pub fn de406_bsp_path_if_exists() -> Option<String> {
    let p = de406_bsp_path();
    Path::new(&p).is_file().then_some(p)
}
