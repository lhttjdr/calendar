# Lunar Calendar — Rust 工作区

与 [doc/11-project-and-implementation.md](../doc/11-project-and-implementation.md) §11.3 Rust 迁移方案配套的迁移骨架。

## 结构

- **core**：纯逻辑核心库。历法（公历↔儒略日、农历岁数据与换算）、天文常数、角度与级数、**Real 数值抽象**（泛型后端，默认 twofloat 双字浮点）、**VSOP87** / **ELPMPP02（平均根数）**、时间尺度、章动/岁差、定气/定朔、数据加载 trait。Native 使用 `DataLoaderNative`。
- **wasm-lib**：对 core 的 wasm-bindgen 包装，供浏览器或 Node 使用。

## Real 标量后端（编译时切换）

默认使用 **twofloat**（double-double）作为 `Real` 标量，精度高、适合桌面与测试。若需减小体积或提速（如 wasm），可改用 **f64**：

```bash
# 默认：twofloat
cargo build -p lunar-core
cargo test -p lunar-core

# 使用 f64 后端（不依赖 twofloat，wasm 更小更快）
cargo build -p lunar-core --no-default-features --features real-f64
cargo test -p lunar-core --no-default-features --features real-f64
```

实现见 `core/src/math/real.rs`（Backend trait + RealInner\<R\>）；方案对比见 [doc/11-project-and-implementation.md](../doc/11-project-and-implementation.md) §11.4。

## 命令（需先安装 Rust：<https://rustup.rs>）

```bash
# 检查编译
cargo check

# 运行 core 测试
cargo test -p lunar-core

# 构建 wasm（需 wasm-pack：cargo install wasm-pack）
wasm-pack build wasm-lib --target web
```

## 数据路径

数据路径：`data/vsop87/`、`data/elpmpp02/` 等，由调用方设置 `DataLoader` 的 base path（例如仓库根目录）。
