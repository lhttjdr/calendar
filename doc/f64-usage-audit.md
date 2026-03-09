# Rust core：f64 使用审计

约定：**不需要“边界”概念——即便读入一个小数也直接 Real 型。** core 内标量一律 Real；f64 仅出现在：`math/real` 内部、与 `[f64;3]`/矩阵/线性代数交互时在写入处 `.as_f64()`、以及 wasm/FFI 导出层（JS 传入 f64 时在 wasm 内转 Real 再调 core，core 返回 Real 时在 wasm 内 `.as_f64()` 再给 JS）。

---

## 1. 已按约定处理（用 Real 贯穿，边界再 .as_f64()）

- **precession p03**：`precession_derivative_times_vector_for`、`precession_transform_for`、`mean_obliquity_rad_and_derivative_for` 等入口用 `impl ToReal`，内部用 `real(t)` 或仅在需 f64 的调用处 `.as_f64()`（如 vondrak2011、jd_from_t_cent）。
- **nutation iau2000b**：`nutation_derivative_times_vector`、`eps_true_dot`、`nutation_derivative` 入口用 `impl ToReal`，内部在需要 f64 处才转换（如 `nutation_derivative` 内 `let t = real(t).as_f64()`）。
- **apparent/longitude**：`nutation_matrix_transposed(t_cent)` 直接传 `t_cent` 给 `nutation_for_apparent` / `mean_obliquity`，不再引入 `t_f64`。

---

## 2. 边界 / 可接受的 f64（暂保留）

| 位置 | 用途 | 说明 |
|------|------|------|
| **math/real.rs** | 标量实现、from_f64/as_f64 | 约定允许的唯一 f64 计算层 |
| **[[f64; 3]; 3]、[f64; 3]** | 旋转矩阵、向量 | 与 Mat/线性代数、FFI 边界一致；由 Real 在填入时 .as_f64() |
| **常量表** | F1_F5_COEFFS、ZETA_COEFFS、LUNI_SOLAR_77 等 | 只读数据，可保留 f64 |
| **NUTATION_OVERRIDE** | `Fn(f64) -> (PlaneAngle, PlaneAngle)` | 回调接口，与外部/测试兼容 |
| **jd_from_t_cent(t: f64)** | 儒略世纪 → JD | 内部辅助，仅被 precession 在边界调用 |
| **vondrak2011** | epsilon(t)、precession_matrix(t)、pa_qa/xa_ya(t) 等 | 内部仍以 f64 为主；入口已支持 impl ToReal 处仅在入口转一次 |
| **测试 / 解析** | 测试用 JD、解析出的 f64 | 测试与 I/O 边界 |

---

## 3. 可后续收紧的 f64（按优先级）

- **fundamental_arguments(t: f64)** / **fundamental_arguments_derivative(t: f64)**  
  仅被 `nutation_derivative` 调用（其内部已 `t = real(t).as_f64()`）。可改为 `impl ToReal`，函数内首行 `let t = real(t).as_f64()`，减少对外暴露 f64。

- **nutation_77(t: f64)**  
  仅被 `nutation_for_apparent` 在内部用 `t_f64` 调用。可改为 `impl ToReal`，内部转一次，保持对外 API 不变。

- **new_moon::approximate_new_moon_jd(n) -> f64**  
  调用方已用 `from_f64_or_zero(approximate_new_moon_jd(n))`。若希望“core 不返回 f64”，可改为返回 `Real`，调用处直接接 Real。

- **longitude 中 NUMERICAL_VELOCITY_DELTA_JD 等**  
  步长常量用于构造 `TimePoint`，已通过 `from_f64_or_zero` 进 Real，属边界常量，可保留或日后改为 Real 常量。

- **pipeline/transform_graph、ephemeris、correction 等**  
  凡“仅在与矩阵/向量/外部 API 交界处”使用 f64 的，保持现状；若出现“中间变量用 f64 再转 Real”的，可改为中间变量用 Real、仅在边界 .as_f64()。

---

## 4. 检查清单（后续改动的自检）

- [ ] 新函数标量参数/返回值优先 `Real` 或 `impl ToReal`，不新增“整段逻辑用 t_f64”的写法。
- [ ] 入口层不写 `let t_f64 = real(t).as_f64()` 再一路传 t_f64；改为传 `real(t)` 或 `t`，仅在调用仍为 f64 的底层时写 `.as_f64()`。
- [ ] 矩阵/向量仍为 `[f64; 3]` 时，用 `x.as_f64()` 填入，不在上游提前把标量都转成 f64。
