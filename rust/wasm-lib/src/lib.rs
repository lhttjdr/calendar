//! 浏览器端 lunar 库：对 core 的 wasm-bindgen 包装。
//!
//! 构建：`wasm-pack build --target web` 或 `--target nodejs`。
//! 产物的 JS 胶水层可被任意前端（React/Vue/Vanilla）引入。
//! 岁数据可在浏览器内现算：传入 VSOP87 / ELPMPP02 文件内容，调用 compute_year_data_wasm。

include!("lib_impl.rs");
