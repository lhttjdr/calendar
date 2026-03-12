//! 日历/天文核心库：历法换算、天文历算。
//!
//! 纯逻辑实现，通过 [platform::DataLoader] 注入数据依赖，便于 Native 与 WebAssembly 共用。
//! 结构：时间与历元 [astronomy::time]、[quantity::Epoch]；物理量与参考架 [quantity]；
//! 天文管线 [astronomy::pipeline]（岁差 [precession]、架变换 [frame]、历表 [ephemeris]、视位置 [apparent] 等）。

pub mod astronomy;
pub mod basic;
pub mod calendar;
pub mod math;
pub mod platform;
pub mod quantity;
pub mod repo;

/// 测试用数据文件管理（仅 `cargo test` 时编译，不进入库/wasm 产物）。
#[cfg(test)]
mod test_util;
