# 12. Rust + WASM + React 性能调优

本章针对「Rust 核心 → wasm-bindgen → React 前端」架构，以**最小化 JS–WASM 跨边界开销与优化内存布局**为目标，给出调优原则与本项目的对照与建议。

## 12.1 核心原则：「黄金法则」

**保持数据在 WASM 内部，减少跨边界通信。**

- **避免 Chatty API**：不要在 React 的循环中逐次调用 WASM。  
  - 反例：在 `map` 里对每一天调用一次 Rust 函数。  
  - 正向：整月/整批数据一次性传入 Rust，在 Rust 内循环，只返回聚合结果。
- **状态下沉**：对计算密集型任务，尽量把状态机放在 Rust 内，React 只做「观察者」，定时取快照渲染。

## 12.2 数据传输：零拷贝与内存视角

- JS 与 WASM 之间默认会有序列化/拷贝（尤其使用复杂对象时）。
- **TypedArray 零拷贝**：Rust 侧返回线性内存指针，JS 通过 `wasm.memory.buffer` 建视图读取，避免拷贝。本项目中岁数据已用 `Float64Array` 在 JS 侧接收 `Vec<f64>`，但当前仍是「WASM 返回 Vec → wasm-bindgen 拷贝出 → JS 建 Float64Array」，若改为「Rust 暴露 ptr + length，JS 用 buffer 视图」可进一步省拷贝。
- **字符串**：JS 为 UTF-16、Rust 为 UTF-8，频繁传字符串有编码成本。尽量用索引、整型或枚举代替字符串标识。

## 12.3 并行与异步

- **Web Worker**：默认 WASM 跑在主线程，重计算会阻塞 UI。将 WASM 放在 Web Worker 中可保证 60fps 不卡顿。
- **Rayon**：在支持 `SharedArrayBuffer` 与多线程 Worker 的环境下，可用 Rayon 在 Rust 内做多线程，进一步利用多核。

## 12.4 React 侧配套

- **避免无效重渲染**：WASM 返回的往往是新引用，直接放进 state 会导致整树刷新。用 `useMemo` 或 React Compiler 的自动 memo，且仅在数据真正变化时更新引用。
- **按需加载**：WASM 体积较大，用 `React.lazy` 或动态 `import()` 分包，避免阻塞首屏。本项目已通过 `lunar-backend-loader` 动态 `import('lunar-wasm')` 实现按需加载。

## 12.5 编译与工程化

发布版建议（已在 `rust/wasm-lib/Cargo.toml` 中配置）：

```toml
[profile.release]
opt-level = "z"   # 体积优化（z 或 s）
lto = true
codegen-units = 1
panic = "abort"
```

- **wasm-opt**：`wasm-pack build` 后运行 `wasm-opt`（或依赖 wasm-pack 的集成）可进一步缩小与优化 WASM 指令。

## 12.6 本项目当前状态与建议

| 维度 | 当前状态 | 建议 |
|------|----------|------|
| **通信 / 批量** | 农历整月已用 `gregorian_month_to_lunar` 一次调用；干支历整月已用 `ganzhiForGregorianMonthWasm` 一次调用（前端 `getGanzhiForMonth`，无此 API 时自动逐日兜底）。 | 已实现；可进一步对其它「按日循环」场景做批量 API。 |
| **内存** | 岁数据 `new_moon_jds` / `zhong_qi_jds` 以 `Vec<f64>` 返回，JS 用 `Float64Array` 接收；`MonthLunarResult` 的 getter 每次 `.clone()` | 若需进一步压榨：可提供「指针 + 长度」接口，JS 用 `wasm.memory.buffer` 视图零拷贝读；或至少避免在 JS 侧多次调用 getter 导致多次 clone，改为一次取回整块数据。 |
| **并发** | WASM 在主线程加载与运行 | 重计算（如批量岁数据、多年代算）可迁到 Web Worker，避免卡顿。 |
| **体积** | Release 已配置 `opt-level = "z"`、`codegen-units = 1`、`panic = "abort"` | 构建流水线中确保运行 `wasm-opt`（或 wasm-pack 默认/可选集成）。 |
| **React** | 整月农历/干支用 `useMemo(..., [wasm, year, month, ...])` 算一次；WASM 动态 import | 保持「按 (year, month, opts) 算一次、格子只读」的模式；避免把 WASM 返回的临时对象直接设为 state 导致引用总变。 |

## 12.7 总结表

| 优化维度 | 手段 | 预期收益 |
|----------|------|----------|
| 通信 | 批量处理，减少 wasm-bindgen 调用次数 | 极高（降低 CPU 与跨边界切换） |
| 内存 | 使用 TypedArray 视图直接读 Rust 内存 | 高（消除拷贝延迟） |
| 并发 | Web Worker 离屏计算 | 极高（解决 UI 卡顿） |
| 体积 | wasm-opt + opt-level = "z" | 中（加快首屏加载） |

与第 11 章「项目与实现」配合：Rust 侧管线与历表已为高性能设计，前端侧遵循上述原则即可最大化发挥 WASM 性能红利。
