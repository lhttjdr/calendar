# Real 底层切换方案细致比较

目标：在**编译时**在 twofloat（double-double）与 f64 之间切换标量类型 `Real`，便于 wasm 用 f64 减体积/提速、桌面或测试用 twofloat 保精度。

---

## 方案概览

| 维度       | 方案 A：条件编译两套实现           | 方案 B：泛型 Real\<R: Backend\> + 单份 impl |
|------------|------------------------------------|---------------------------------------------|
| 核心做法   | `#[cfg(feature = "real-f64")]` 下整份 Real 换实现 | `struct Real<R>(R)`，`impl<R: Backend> RealOps for Real<R>`，`type Real = Real<TwoFloat>` / `Real<F64>` |
| 上层改动   | 无                                 | 无（对外仍是 `Real` type alias）             |
| 重复代码   | 多（几乎所有 Real 的 impl 写两遍） | 少（只有 Backend 的“适配层”各写一份）      |
| 扩展第三后端 | 再加一套 cfg + 第三份 impl        | 仅新类型 impl Backend，Real 不动            |

---

## 方案 A：条件编译，两套实现

### 做法

- 在 `core/src/math/real.rs` 中：
  - **默认**（无 `real-f64`）：保持现有 `pub struct Real(pub(crate) TwoFloat)` 及全部 `impl Real`、`impl RealOps for Real`、`real_const`、`real()`、f64/Real 混合运算等。
  - **启用 `real-f64`**：改为例如 `pub struct Real(pub(crate) f64)`（或 `pub type Real = f64`；若用 type alias，需为 f64 实现 RealOps 及所有用到的 trait），并实现与默认分支**行为等价**的：
    - `real_const`、`real()`、`ToReal`
    - `Add/Sub/Mul/Div/Neg` 及 `*Assign`
    - `RealOps`（`from_f64`、`as_f64`、`zero`、`one`、`pi`、`two_pi`、`sin/cos/tan/sqrt/asin/atan2`、默认方法可沿用 trait 的基于 `as_f64/from_f64` 的实现）
    - `PartialEq<f64>`、`PartialOrd<f64>`、`Display`
    - f64 与 Real 的混合运算（`f64 * Real`、`Real + f64` 等）
- `Cargo.toml`：`twofloat` 设为 optional，仅当 `not(feature = "real-f64")` 时启用。
- 上层（除 `real.rs` 外）**不改**，仍只依赖 `Real`、`RealOps`、`real_const`、`real()`。

### 优点

- **实现路径直观**：复制现有 Real 实现，把 TwoFloat 换成 f64，再修 const（如 `RealOps::pi()` 用 `std::f64::consts::PI`）。
- **编译结果简单**：每个 crate 只选一种后端，二进制里只有一套 Real 实现，无泛型单态化带来的额外抽象。
- **对现有 const/static 友好**：两套都可提供 `const fn real_const` 和 `pub const J2000: Real = real_const(...)`，只要 f64 版 `real_const` 也是 const 即可。
- **类型简单**：始终是具体类型 `Real`，无泛型参数，IDE/错误信息里不会出现 `Real<TwoFloat>` 等。

### 缺点

- **重复多、易出错**：约 400+ 行的 impl（算上 RealOps 默认方法、混合运算、Display、PartialEq 等）要维护两遍；改一处（如 `wrap_to_2pi` 的边界）容易漏改另一处，一致性靠人工和测试。
- **扩展成本高**：若将来加第三种后端（如 rust_decimal 或 f128），要再开一个 feature 和第三套完整 impl，重复度线性增长。
- **real.rs 体积大**：同一文件内大量 `#[cfg(not(feature = "real-f64"))]` / `#[cfg(feature = "real-f64")]` 块，可读性差，review 时要同时看两路逻辑。

### 实现量粗估

- 新增：约 1 个 feature、1 个可选依赖、约 200–350 行与现有 Real 对称的 impl（视是否用 type alias 与多少默认实现可复用）。
- 修改：现有 Real 块包上 `#[cfg(not(feature = "real-f64"))]`，少量位置需 cfg（如 twofloat 的 `consts::PI` vs `core::f64::consts::PI`）。

---

## 方案 B：泛型 Real\<R: Backend\> + 单份 impl

### 做法

- **Backend trait**（或名 `RealBackend`）：定义“标量后端”必须提供的能力，且与 `RealOps` 解耦，只给底层用。例如：
  - 构造/输出：`from_f64(x: f64) -> Option<Self>`，`as_f64(self) -> f64`
  - 常数：`zero()`，`one()`，`pi()`，`two_pi()`
  - 运算：`sin/cos/tan/sqrt/asin/atan2(self, other)`（以及算术由 Rust 的 Add/Sub/Mul/Div/Neg 满足即可）
  - 要求：`Copy + Clone + Add<Output=Self> + Sub/Mul/Div/Neg + PartialOrd + Default`
- **统一外壳**：`pub struct Real<R>(pub(crate) R)`，仅当 `R: Backend` 时对外使用。
- **单份实现**：`impl<R: Backend> RealOps for Real<R>` 中，所有方法**委托**给 `self.0`（如 `from_f64` → `Real(R::from_f64(x)?)`，`sin` → `Real(self.0.sin())`）；`RealOps` 里已有默认实现的（如 `wrap_to_2pi`、`power_series_at`）保持不变，只需用 `Self` 的 `from_f64`/`as_f64`，无需碰 Backend。
- **两个后端**：
  - 默认：`struct TwoFloatBackend(TwoFloat)`（或直接对 twofloat 做一层薄包装），`impl Backend for TwoFloatBackend`，内部转调 TwoFloat 的固有方法。
  - real-f64：`struct F64Backend(f64)`，`impl Backend for F64Backend`，内部用 `self.0.sin()` 等；若 f64 的 `Add::Output = f64` 等已满足，只需实现 Backend 的“非标准库已有”部分（如 `pi()`、`zero()`、`one()`、可能 `from_f64`/`as_f64`）。
- **类型别名**：  
  `pub type Real = Real<TwoFloatBackend>;` 或 `pub type Real = Real<F64Backend>;`（由 feature 选择）。
- **const**：`real_const` 需在两种 Backend 下都可 const。TwoFloat 需确认 `TwoFloat::from_f64` 是否为 const；F64Backend 的 `from_f64` 做 const 容易。若 TwoFloat 当前不是 const，可保留 `real_const` 在 twofloat 下非 const，仅 f64 下 const，或两路都非 const（用 lazy_static/once_cell 替代部分 const，不推荐改动面大）。
- 上层仍只用 `Real`、`RealOps`、`real_const`、`real()`，**无改动**。

### 优点

- **无重复的 Real 逻辑**：`RealOps`、混合运算、Display、PartialEq 等只写一次，全部在 `Real<R>` 上；新增后端只需为新类型实现 Backend，不用再复制整份 Real。
- **扩展性好**：加第三个后端 = 新类型 + `impl Backend` + 一个 feature 下的 type alias，Real 和业务代码都不动。
- **一致性由类型系统保证**：两套后端共用同一套 Real/RealOps 实现，行为差异只来自 Backend 的数学实现，不会出现“改了一处忘改另一处”的 cfg 分支不同步。
- **测试可复用**：同一套针对 `RealOps` 的测试，换 feature 即测另一后端。

### 缺点

- **设计成本**：要划清 Backend 与 RealOps 的边界（哪些在 Backend、哪些用 RealOps 默认实现），且 TwoFloat 是外库类型，不能直接 `impl Backend for TwoFloat`，必须包一层 newtype 并转调，略繁琐。
- **const 的约束**：若 TwoFloat 的 `from_f64` 不是 const，则 `real_const` 在默认后端下可能无法 const，会波及现有 `pub const J2000: Real = real_const(...)` 等；需要查 twofloat 文档或考虑只对 f64 分支提供 const。
- **单态化与体积**：最终会有 `Real<TwoFloatBackend>` 和 `Real<F64Backend>` 两套单态化，但逻辑只有一份，代码膨胀主要在 Backend 的 sin/cos 等；通常可接受。
- **类型错误信息**：在未使用 type alias 的泛型代码里可能看到 `Real<TwoFloatBackend>`；对外若始终通过 `Real` 使用，影响有限。

### 实现量粗估

- 新增：Backend trait、TwoFloatBackend 与 F64Backend 的包装及 impl、可选的 feature 与 type alias 切换。
- 修改：将现有 `struct Real(TwoFloat)` 改为 `struct Real<R>(R)`，所有 `impl Real` / `impl RealOps for Real` 改为 `impl<R: Backend> ... for Real<R>`，并把对 `self.0` 的调用从直接 TwoFloat 方法改为通过 Backend 的约束（或通过 trait 方法）调用。量约 150–250 行新增/重写，但删除大量重复。

---

## 对照表（简要）

| 项目           | 方案 A                         | 方案 B                                      |
|----------------|--------------------------------|---------------------------------------------|
| 实现量         | 中（约 200–350 行重复 impl）   | 中高（Backend 设计 + 两套 Backend impl）    |
| 维护成本       | 高（两处同步）                 | 低（逻辑集中在一处）                        |
| 扩展第三后端   | 再加一整份 impl                | 只加一个 Backend impl                       |
| const/static   | 两路都可做 const（若 f64 支持）| 依赖 TwoFloat 是否 const；可能需妥协         |
| 二进制/性能    | 单态，无额外泛型               | 两套单态，略增体积，通常可接受              |
| 类型/错误信息  | 始终 `Real`                    | 对外 `Real`，内部可能看到 `Real<…>`         |
| 一致性保证     | 靠测试与人工                   | 靠类型系统与单份实现                        |

---

## 建议

- **若短期只做 twofloat ⇄ f64 二选一、且希望改动集中、少动架构**：选 **方案 A** 即可，注意把两路 impl 和测试对齐，并加 CI 下两套 feature 都跑测试。
- **若预期会有更多后端或长期维护、希望减少重复与遗漏**：选 **方案 B**，一次性把 Backend 边界和 `Real<R>` 设计好，并处理好 `real_const` 与现有 const/static 的兼容（必要时 f64 分支 const、twofloat 分支用 lazy 或接受非 const）。

两种方案都**不能**在运行时动态切换；只能通过 Cargo feature 在编译时选一个后端。
