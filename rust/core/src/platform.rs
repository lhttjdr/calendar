//! 跨平台数据加载抽象：由 Native（std::fs）或 Wasm（fetch/嵌入）实现。

/// 按路径读取文本行；path 为相对或绝对路径/资源名。
/// Native 实现用 std::fs，Wasm 可用 fetch 或编译期嵌入。
pub trait DataLoader: Send + Sync {
    /// 读取全部行（同步）。Wasm 侧可为 async 的同步封装或预加载。
    fn read_lines(&self, path: &str) -> Result<Vec<String>, LoadError>;
}

/// 数据加载错误
#[derive(Debug, Clone)]
pub enum LoadError {
    Io(String),
    NotFound(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Io(s) => write!(f, "IO: {}", s),
            LoadError::NotFound(s) => write!(f, "Not found: {}", s),
        }
    }
}

impl std::error::Error for LoadError {}

#[cfg(not(target_arch = "wasm32"))]
/// Native 实现：基于 std::fs，path 相对 base_path 或绝对路径。
pub struct DataLoaderNative {
    pub base_path: std::path::PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
impl DataLoaderNative {
    pub fn new(base_path: impl AsRef<std::path::Path>) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl DataLoader for DataLoaderNative {
    fn read_lines(&self, path: &str) -> Result<Vec<String>, LoadError> {
        let p = if std::path::Path::new(path).is_absolute() {
            std::path::PathBuf::from(path)
        } else {
            self.base_path.join(path)
        };
        let s = std::fs::read_to_string(&p)
            .map_err(|e| LoadError::Io(format!("{}: {}", p.display(), e)))?;
        Ok(s.lines().map(String::from).collect())
    }
}
