# 11. 项目与实现

## 11.1 项目结构（Rust + Web）

本仓库为**两部分**结构：Rust 核心库与 WASM、Web 前端；Web 使用 Rust WASM 作为农历计算后端。

| 部分 | 目录 | 构建命令 | 说明 |
|------|------|----------|------|
| **Rust** | `rust/` | `cargo build` | 核心库 `core`、WASM 包 `wasm-lib`（TwoFloat）、`wasm-lib-f64`（Real=f64）；共享 `data/`。 |
| **Web** | `web/` | `npm install && npm run dev` | Vite + React；可选依赖 `lunar-wasm` / `lunar-wasm-f64`。 |

刷新 WASM：`cd web && npm run refresh-wasm`（仅刷新 wasm-lib）。两版需分别构建：`wasm-pack build --target web` 在 `rust/wasm-lib` 与 `rust/wasm-lib-f64` 各执行一次。前端选项卡片中「Real 后端」下拉可切换 TwoFloat（高精度）与 f64（体积小/速度快），选择会持久化到 localStorage。

**数据与路径**：历表在仓库根 `data/vsop87/`、`data/elpmpp02/`；参考文献在 `doc/references/`；Rust 工作目录为仓库根；Web 从 `/data/...` 拉取。

## 11.2 Rust 实现概要

- **目标**：核心库一次编写，Native 与 WebAssembly 共用；无 GC、SIMD 友好。
- **架构**：`rust/core`（math、astronomy、calendar、platform trait）、`rust/wasm-lib`（wasm-bindgen 暴露）、可选 `desktop`（Tauri）。
- **数据**：通过 `Platform` trait 注入读路径；Native 用 `std::fs`，Wasm 用 `fetch` 或嵌入。
- **高精度**：`rust_decimal`（96-bit 定点）；三角/开方在 `Real` 内用 f64，其余算术高精度。

### 11.2.1 视位置管线架构（Rust）

视位置计算已统一到管线架构（历元/坐标表示/参考架三正交，见第 2、7 章），代码位于 `rust/core/src/astronomy/pipeline/`：**State6 / StateTransition6**、**EphemerisProvider**（Vsop87 太阳、Elpmpp02Data 月球）、**FrameMapper**（VsopToDe406IcrsFit）、**LightTimeCorrector**、**TransformGraph**（岁差 P03/Vondrak2011）、**Pipeline** Fluent API。对外仍通过 `astronomy::apparent` 暴露 `sun_position_icrs`、`sun_apparent_ecliptic_longitude*`、`moon_apparent_ecliptic_longitude*`，内部走上述管线，API 不变。详见第 7 章 §7.2。

### 11.2.2 性能与部署策略

历元矩阵缓存（同一时刻岁差/章动矩阵可缓存）、霍纳法则（多项式嵌套求值）、跨平台 Wasm（纯计算核心可编译至 WebAssembly）。

## 11.3 核心模块对应（Rust）

| 模块 | Rust (core) |
|------|-------------|
| 数学 / 实数 | math::real::Real、math::series |
| 历表 | astronomy::ephemeris::vsop87、astronomy::ephemeris::elpmpp02 |
| 管线 | astronomy::pipeline（EphemerisProvider、FrameMapper、LightTimeCorrector、TransformGraph） |
| 视位置 | astronomy::apparent（内部走 pipeline） |
| 合朔 / 节气 | astronomy::synodic + aspects |
| 农历 | calendar::chinese_lunar |

测试：Vsop87、ELPMPP02、Nutation、TimePoint、农历等均有对应 Rust 测试模块。
