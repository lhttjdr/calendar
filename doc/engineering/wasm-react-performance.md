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
| **并发** | 岁数据（节气、朔、弦望）一律在 Web Worker 算，按所选 Real 后端加载；Worker 回传前後三年节气，主线程灌入节气缓存（§12.6.1） | 主线程只做查表级的逐日计算；新增需要求解方程的逐日功能，先看能否由 Worker 预算後灌缓存。 |
| **体积** | Release 已配置 `opt-level = "z"`、`codegen-units = 1`、`panic = "abort"` | 构建流水线中确保运行 `wasm-opt`（或 wasm-pack 默认/可选集成）。 |
| **React** | 整月农历/干支用 `useMemo(..., [wasm, year, month, ...])` 算一次；WASM 动态 import | 保持「按 (year, month, opts) 算一次、格子只读」的模式；避免把 WASM 返回的临时对象直接设为 state 导致引用总变。 |

### 12.6.1 岁数据与节气不在主线程求解（2026-11-08）

**症状**：线上页面「卡在加载 WASM / 加载历表」不动。选高精度（TwoFloat）后端时整页无响应；f64 也有数秒停顿。

**原因**（以 CDP 暂停取栈、`PerformanceObserver('longtask')` 量主线程长任务查得）：

1. `getYearData(lunarYear, backend)` 只在**不传** `backend` 时走 Worker；App 加载 WASM 后一律传入后端，岁数据于是全在主线程现算。原先 Worker 超时（45 s）还会回退主线程再算一遍。
2. Worker 只会加载 f64 WASM，TwoFloat 无从放进 Worker。
3. 月历逐日干支（`ganzhi_from_jd_solar_wasm`）、杂节（`zajie_for_day_wasm`）、铺注（`shensha_day_wasm`）都要用前後三四个公历年的二十四节气，首次调用在主线程现解（核心缓存 `solar_term_jds_for_year_cached` 只免重复，不免首解）。
4. 岁数据附朔弦望（铺注「朔弦望」）后，每岁多解约 50 个月相，原实现无粗算段，比整岁数据本身还慢三倍。

原生 release 实测（2026 农历岁）：

| 项 | f64 | TwoFloat |
|----|-----|----------|
| 一公历年 24 节气 | 0.11 s | 1.6 s |
| 岁数据（14 朔 + 中气） | 0.75 s | 13.5 s |
| 弦望，改前（53 个，无粗算） | 2.55 s | 49.8 s |
| 弦望，改后（39 个，粗算 + 朔取岁数据） | 0.97 s | 19.1 s |

**修法**：

- `getYearData` 由后端对象查其变体（`loadWasm` 时登记），交同一变体的 Worker 算；缓存键为「变体:农历年」。Worker 不设超时回退，只有 Worker 不可用、全二进制路径失败或 Worker 崩溃时才在主线程算。
- `YearDataResult` 附 `solar_term_years`/`solar_term_jds`（农历年前後各一公历年的 24 节气，同一历表与改正），主线程收到後先 `seed_solar_terms_wasm` 灌入本线程 WASM 的节气缓存（核心 `seed_solar_terms_cache`，已有者不覆盖），再交付岁数据；逐日函数随后只查缓存。经 JS 传为 f64（JD 分辨率约 40 μs），对逐日判定足够。
- 弦望求解加粗算段、朔取岁数据之朔（见 [lunar-phases-and-syzygy.md](../astronomy/lunar-phases-and-syzygy.md) §8.1）。

- 同键（变体 + 农历年）并发请求合并：App 会同时为月首、月末与所选日请求同一岁，Worker 串行，不合并则同一岁重算数遍。

**验证**（本地 `vite preview`，无头 Chromium，2026-11）：f64 约 3.6 s 载完；TwoFloat 约 123 s（未合并并发请求时 250 s）。两者主线程长任务（`PerformanceObserver('longtask')`）均不过 0.2 s，页面始终可操作。（以 CDP `Runtime.evaluate` 量往返时，TwoFloat 下偶见十余秒延迟，出在 CDP 挂接忙碌 Worker 一侧，页面内长任务记录无对应。）

**限制**：TwoFloat 只用于检验 f64 的误差，Worker 里一岁仍要数十秒；切到 TwoFloat 时状态栏会停在「加载历表…」直到算完。

### 12.6.2 首屏按需加载与预算岁数据（2026-11-08）

**症状**：线上首屏加载很久。实测（此处到 GitHub Pages 约 15 KB/s）主程序包 620 KB（gzip）传 40 秒才开始加载 WASM；其后为显示当月农历还要下载约 0.9 MB 历表（ELPMPP02 六表、章动表、地球改正）再在 Worker 里解方程；主线程另下载 343 KB 的 VSOP87 文本（扩展名 `.ear` 被当作 Java 包类型，服务器不压缩）。

**修法**：

1. **页面按需加载**：日月食图、星图、视位置变换图、择日、排盘改为 `React.lazy` 动态导入，主包 1.9 MB → 0.8 MB（未压缩）。
2. **主线程 VSOP87 用二进制**：新增 `init_vsop87_earth_from_binary`，主线程下载 `VSOP87B.ear.bin`（62 KB，brotli 37 KB）初始化本实例缓存，各函数的 `vsop87_ear` 传空串；按实例记录（f64、TwoFloat 两实例各一份），旧版 WASM 无此入口时退回文本。
3. **预算岁数据表**：`rust/core/examples/gen_year_table.rs` 于部署时（`pages.yml`「Precompute year data table」，不入库）以与 Worker `compute_year_data_full_binary` 相同的二进制历表与配置（VSOP87 + DE431 地球改正、ELPMPP02 缺省常数集 DE431、IAU2000 全章动表、f64）算 1900—2100 年每岁朔、中气（含下一冬至）、朔弦望（`chinese_lunar::year_phases`，与 wasm 共用）及 1899—2101 年每年二十四节气，f64 原值写入 `data/precomputed/yeardata-de431-f64.bin`（179 KB，gzip 109 KB）。前端 `getYearData` 在缺省配置（f64、`EARTH_PATCH_TARGET = 'de431'`）下先查此表，命中即用，并把表中节气一次灌入主线程节气缓存；表外年份、TwoFloat 或换历表配置照旧经 Worker 现算。状态栏数据源显示「预算表」。生成 4 核约 2 分钟。
4. **民俗栏缓存**：日历民俗栏（宿曜、杂节、更点、建除、神煞铺注）改为按日、分钟、观测地、农历缓存（`useMemo`），不随实时时钟每秒重算；排盘摘要移入排盘页按需计算。

**结果**：首屏请求只剩主包、WASM、预算表、VSOP87 二进制四样（线上约 0.25 + 0.51 + 0.11 + 0.04 ≈ 0.9 MB，原约 2.4 MB），不再下载月球历表、章动表，不起 Worker。本地限速 150 KB/s（预览服务器不压缩）农历显示由 30.5 s 降到 19.2 s。

## 12.7 总结表

| 优化维度 | 手段 | 预期收益 |
|----------|------|----------|
| 通信 | 批量处理，减少 wasm-bindgen 调用次数 | 极高（降低 CPU 与跨边界切换） |
| 内存 | 使用 TypedArray 视图直接读 Rust 内存 | 高（消除拷贝延迟） |
| 并发 | Web Worker 离屏计算 | 极高（解决 UI 卡顿） |
| 体积 | wasm-opt + opt-level = "z" | 中（加快首屏加载） |

与第 11 章「项目与实现」配合：Rust 侧管线与历表已为高性能设计，前端侧遵循上述原则即可最大化发挥 WASM 性能红利。
