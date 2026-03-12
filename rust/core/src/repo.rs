//! 外部数据文件与仓库根路径管理。
//!
//! **统一由 repo 管理不同实现**：Native 下为本地文件（`repo_root` + std::fs），Wasm 下由宿主在 fetch 后通过 [`set_loader`] 注入 DataLoader。
//! 各业务模块只保留 parser，通过本模块的 `read_lines` / `read_bytes` 读入后再解析；调用方不关心底层是本地读还是 fetch。

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
use std::path::PathBuf;

/// 所有外部数据文件的相对路径（相对仓库根），单一来源，避免拼写与层级错误。
pub mod paths {
    /// 章动 IERS 5.3a（经度，µas）
    pub const IAU2000_TAB53A: &str = "data/IAU2000/tab5.3a.txt";
    /// 章动 IERS 5.3b（倾角，µas）
    pub const IAU2000_TAB53B: &str = "data/IAU2000/tab5.3b.txt";
    /// 章动二进制（可选）
    pub const IAU2000_TAB53A_BIN: &str = "data/IAU2000/tab5.3a.bin";

    /// VSOP87 地球系数
    pub const VSOP87_EARTH: &str = "data/vsop87/VSOP87B.ear";
    /// ELPMPP02 数据目录
    pub const ELPMPP02: &str = "data/elpmpp02";

    /// VSOP87–DE406 赤道 patch
    pub const FIT_VSOP87_DE406_ICRS: &str = "data/fit/vsop87-de406-icrs.txt";
    /// VSOP87–DE406 黄道 patch
    pub const FIT_VSOP87_DE406_ECLIPTIC: &str = "data/fit/vsop87-de406-ecliptic.txt";

    /// 《月相和二十四节气的计算》§7.4 节气朔望标准时刻表
    pub const SOLAR_TERMS_REFERENCE: &str = "data/月相和二十四节气的计算/TDBtimes.txt";

    /// DE406 BSP 候选路径（按优先级）
    pub const DE406_BSP_CANDIDATES: &[&str] = &[
        "data/jpl/de406/de406.bsp",
        "data/jpl/de406.bsp",
    ];
    /// DE406 数据目录（用于跳过提示）
    pub const JPL_DATA_DIR: &str = "data/jpl";
    /// ELPMPP02 vs JPL DE406 样本 CSV（J2000 平黄道 km）
    pub const JPL_ELP_VS_JPL_SAMPLES_CSV: &str = "data/jpl/elp_vs_jpl_de406_samples.csv";
}

/// 返回仓库根目录（Native 下用于构造 DataLoader 的 base_path）。
///
/// 解析顺序：环境变量 `REPO_ROOT`（若为有效目录）→ `CARGO_MANIFEST_DIR` 上两级并 canonicalize。
/// 布局假定：`rust/core` 为 manifest 目录，故 `../..` 为仓库根（calendar），其下为 `data/`。
#[cfg(not(target_arch = "wasm32"))]
pub fn repo_root() -> PathBuf {
    std::env::var("REPO_ROOT")
        .ok()
        .and_then(|p| {
            let path = Path::new(&p);
            path.is_dir().then(|| path.canonicalize().ok()).flatten()
        })
        .unwrap_or_else(|| {
            let base_rel = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
            base_rel
                .canonicalize()
                .unwrap_or_else(|_| base_rel.into())
        })
}

#[cfg(target_arch = "wasm32")]
/// Wasm 无文件系统仓库根；占位用，实际读由 [`set_loader`] 注入的 loader 完成。
pub fn repo_root() -> PathBuf {
    PathBuf::new()
}

// ---------- 共通读写：Native = 本地文件，Wasm = 宿主注入的 loader -----------

#[cfg(not(target_arch = "wasm32"))]
/// 以仓库根为 base 的 DataLoader，供需要注入 loader 的调用方使用。
pub fn default_loader() -> crate::platform::DataLoaderNative {
    crate::platform::DataLoaderNative::new(repo_root())
}

#[cfg(not(target_arch = "wasm32"))]
/// 从仓库内相对路径读取文本行（共通读）。路径使用 [paths] 常量。
pub fn read_lines(path: &str) -> Result<Vec<String>, crate::platform::LoadError> {
    use crate::platform::DataLoader;
    default_loader().read_lines(path)
}

#[cfg(not(target_arch = "wasm32"))]
/// 从仓库内相对路径读取二进制（共通读）。路径使用 [paths] 常量。
pub fn read_bytes(path: &str) -> Result<Vec<u8>, crate::platform::LoadError> {
    use crate::platform::DataLoader;
    default_loader().read_bytes(path)
}

#[cfg(not(target_arch = "wasm32"))]
/// 向仓库内相对路径写入二进制（共通写）。必要时创建父目录。
pub fn write_bytes(path: &str, bytes: &[u8]) -> Result<(), std::io::Error> {
    let p = repo_root().join(path);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&p, bytes)
}

// ---------- Wasm：由宿主 fetch 后注入 loader，读通过该 loader 完成 -----------

#[cfg(target_arch = "wasm32")]
static REPO_LOADER: once_cell::sync::OnceCell<Box<dyn crate::platform::DataLoader + Send + Sync>> =
    once_cell::sync::OnceCell::new();

#[cfg(target_arch = "wasm32")]
/// 设置 Wasm 下「从 repo 读」的实现（通常为 fetch 得到的内容构建的 in-memory loader）。只需设置一次。
pub fn set_loader(loader: Box<dyn crate::platform::DataLoader + Send + Sync>) {
    let _ = REPO_LOADER.set(loader);
}

#[cfg(target_arch = "wasm32")]
/// Wasm 下取当前注入的 loader（如 load_all 等仍需传 loader 时使用）。
pub fn get_loader() -> Option<&'static (dyn crate::platform::DataLoader + Send + Sync)> {
    REPO_LOADER.get().map(|b| b.as_ref())
}

#[cfg(target_arch = "wasm32")]
/// 从仓库内相对路径读取文本行（共通读）。须已调用 [`set_loader`]，否则返回错误。
pub fn read_lines(path: &str) -> Result<Vec<String>, crate::platform::LoadError> {
    REPO_LOADER
        .get()
        .ok_or_else(|| crate::platform::LoadError::NotFound("repo loader not set (call set_loader first)".into()))?
        .read_lines(path)
}

#[cfg(target_arch = "wasm32")]
/// 从仓库内相对路径读取二进制（共通读）。须已调用 [`set_loader`] 且 loader 支持 read_bytes。
pub fn read_bytes(path: &str) -> Result<Vec<u8>, crate::platform::LoadError> {
    REPO_LOADER
        .get()
        .ok_or_else(|| crate::platform::LoadError::NotFound("repo loader not set (call set_loader first)".into()))?
        .read_bytes(path)
}
