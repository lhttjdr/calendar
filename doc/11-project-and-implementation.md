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

## 11.4 Real 底层切换方案比较

目标：在**编译时**在 twofloat（double-double）与 f64 之间切换标量类型 `Real`，便于 wasm 用 f64 减体积/提速、桌面或测试用 twofloat 保精度。

### 方案概览

| 维度       | 方案 A：条件编译两套实现           | 方案 B：泛型 Real\<R: Backend\> + 单份 impl |
|------------|------------------------------------|---------------------------------------------|
| 核心做法   | `#[cfg(feature = "real-f64")]` 下整份 Real 换实现 | `struct Real<R>(R)`，`impl<R: Backend> RealOps for Real<R>`，`type Real = Real<TwoFloat>` / `Real<F64>` |
| 上层改动   | 无                                 | 无（对外仍是 `Real` type alias）             |
| 重复代码   | 多（几乎所有 Real 的 impl 写两遍） | 少（只有 Backend 的“适配层”各写一份）      |
| 扩展第三后端 | 再加一套 cfg + 第三份 impl        | 仅新类型 impl Backend，Real 不动            |

**方案 A**：条件编译两套实现；实现路径直观、const 友好，但重复多、扩展成本高。**方案 B**：泛型 Real\<R: Backend\> + 单份 impl；无重复逻辑、扩展性好，但需划清 Backend 与 RealOps 边界、注意 const 与单态化。两种方案都**不能**在运行时动态切换，只能通过 Cargo feature 在编译时选一个后端。短期二选一且少动架构可选 A；预期多后端或长期维护可选 B。

## 11.5 f64 使用约定与审计（Rust core）

约定：core 内标量一律 **Real**；f64 仅出现在：`math/real` 内部、与 `[f64;3]`/矩阵/线性代数交互时在写入处 `.as_f64()`、以及 wasm/FFI 导出层。

- **已按约定**：precession/nutation/apparent 入口用 `impl ToReal`，内部仅在需 f64 处 `.as_f64()`。
- **边界/可接受**：math/real、旋转矩阵与向量、常量表、NUTATION_OVERRIDE、jd_from_t_cent、vondrak2011 内部、测试与解析。
- **可后续收紧**：fundamental_arguments(t)、nutation_77(t)、approximate_new_moon_jd 返回值、longitude 步长常量等可改为 `impl ToReal` 或返回 Real；矩阵/向量交界处保持“用 Real 填入时 .as_f64()”。

**自检清单**：新函数标量优先 `Real` 或 `impl ToReal`；入口层不一路传 t_f64；矩阵/向量用 `x.as_f64()` 填入。

## 11.6 物理量与类型审计（历史）

**历史审计**：原实现中 astronomy 下使用 Decimal、元组处的排查，建议有物理概念的改为物理量（`PlaneAngle`、`Length`、`Duration`、`Position`、`Velocity` 等）。

- **建议改为物理量**：大气折射角度入参/返回、章动 (Δψ, Δε)、坐标点各字段、光行时 initialDistance、合朔/节气中的 prevSunDist/prevMoonDist、TimeScaleContext.deltaT 返回值等。
- **可保留或轻量改进**：时间与归一化时间、历表/级数内部、矩阵/向量运算、无量纲或约定单位、容差/配置。
- **Rust core 部分**：角度/长度/角速度/时间等可用 `PlaneAngle`/`Length`/`AngularRate`/`Duration` 替代的裸 f64，多数已在实施顺序中完成（大气折射、定气定朔容差、常量角速度、delta_t、fundamental_arguments、sun_position_icrs 等已做）。
