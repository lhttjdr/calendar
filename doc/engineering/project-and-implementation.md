# 11. 项目与实现

## 11.1 项目结构（Rust + Web）

本仓库为**两部分**结构：Rust 核心库与 WASM、Web 前端；Web 使用 Rust WASM 作为农历计算后端。

| 部分 | 目录 | 构建命令 | 说明 |
|------|------|----------|------|
| **Rust** | `rust/` | `cargo build` | 核心库 `core`、WASM 包 `wasm-lib`（TwoFloat）、`wasm-lib-f64`（Real=f64）；共享 `data/`。 |
| **Web** | `web/` | `npm install && npm run dev` | Vite + React；可选依赖 `lunar-wasm` / `lunar-wasm-f64`。 |

刷新 WASM：`cd web && npm run refresh-wasm`（仅刷新 wasm-lib）。两版需分别构建：`wasm-pack build --target web` 在 `rust/wasm-lib` 与 `rust/wasm-lib-f64` 各执行一次。前端选项卡片中「Real 后端」下拉可切换 TwoFloat（高精度）与 f64（体积小/速度快），选择会持久化到 localStorage。

**数据与路径**：历表在仓库根 `vendor/vsop87/`、`vendor/elpmpp02/`；参考文献在 `doc/references/`；Rust 工作目录为仓库根；Web 从 `/data/...` 拉取。

## 11.2 Rust 实现概要

- **目标**：核心库一次编写，Native 与 WebAssembly 共用；无 GC、SIMD 友好。
- **架构**：`rust/core`（math、astronomy、calendar、platform trait）、`rust/wasm-lib`（wasm-bindgen 暴露）、可选 `desktop`（Tauri）。
- **数据**：由 **repo** 模块统一路径与读写（`repo::paths`、`repo_root()`、`read_lines`/`read_bytes`）；Native 用 `default_loader()` 与各 `*_from_repo()`，Wasm 由宿主 `set_loader` 注入后同样调用 `*_from_repo()`。章动/拟合/历表加载与管线（doc 15 §6）一致。
- **高精度**：`rust_decimal`（96-bit 定点）；三角/开方在 `Real` 内用 f64，其余算术高精度。

### 11.2.1 视位置管线架构（Rust）

视位置计算已统一到管线架构（历元/坐标表示/参考架三正交，见第 2、7 章），代码位于 `rust/core/src/astronomy/pipeline/`：**State6 / StateTransition6**、**EphemerisProvider**（Vsop87 太阳，含加载时选定的 DE406/DE441 改正；Vsop87Planets 行星；Elpmpp02Data 月球；De406Kernel 读 DE406/DE441 BSP）、**FrameMapper**（Fk5ToIcrsBias，或 Compose 组合）、**LightTimeCorrector**、**TransformGraph**（岁差 Vondrak2011 缺省 / P03 可选）、**Pipeline** Fluent API。对外仍通过 `astronomy::apparent` 暴露 `sun_position_icrs`、`sun_apparent_ecliptic_longitude*`、`moon_apparent_ecliptic_longitude*`，内部走上述管线，API 不变。详见第 7 章 §7.2。

### 11.2.2 性能与部署策略

历元矩阵缓存（同一时刻岁差/章动矩阵可缓存）、霍纳法则（多项式嵌套求值）、跨平台 Wasm（纯计算核心可编译至 WebAssembly）。

#### 实测耗时与加速（2026-10-27，twofloat 后端，release，4 核机器上另有 3 个计算进程，数字偏大）

单次调用耗时（`Real` = double-double）：

| 操作 | 改动前 | 改动后 |
|---|---|---|
| twofloat sin + cos（分别调用） | 约 0.9 µs | 约 0.9 µs |
| twofloat 复数乘法 | 约 0.08 µs | — |
| f64 sin + cos | 约 0.023 µs | — |
| ELPMPP02 一次求值（约 3.5 万项，位置 + 速度） | 69 ms | 42 ms |
| 月球视位置（`body_apparent_equatorial`，含光行时迭代，约 4 次 ELPMPP02） | 281 ms | 184 ms |
| 地球 VSOP87 一次求值 | 2.9 ms | — |
| 太阳视位置 | 14 ms | — |
| `gast`（含 IAU 2000A 章动） | 0.7 ms | — |

结论：时间几乎全花在级数逐项的 double-double 三角函数（每项约 0.9 µs）与逐项的量纲换算上；没有并行。

已做（结果不变，均有测试）：
1. **`Real::sin_cos`**（`math::real`）：一次约化同时得 sin、cos，与分别调用逐位相同；VSOP87、ELPMPP02、IAU 2000A/B 章动、
   VSOP87 改正项改用它。
2. **ELPMPP02 Table 6 相位修正提到项循环外**：修正对 (i_F, i_l) 线性，每次求值只算一次系数，逐项两次乘加，免去每项重建常数、
   角秒换算。测试 `elpmpp02::tests::evaluation_refactor_matches_reference` 以改动前输出为参考（位置差 < 1e-6 m、速度差 < 1e-12 m/s）。
3. **校验测试并行**：`test_util::par_map`（`std::thread::scope`，不加依赖，不影响 wasm），逐项领取、保序返回；
   `PHENOMENA_THREADS` 可指定线程数（1 为串行）。史书凌犯检验、NASA 日月食典对照已改用；串行与并行输出逐字节相同。
   实测（机器有他负载）：汉书 + 后汉书凌犯 300 s → 130 s；NASA 日食典抽样 1/20 101 s → 37 s。

#### 精度与速度的权衡（2026-10-27 实测，`examples/precision_tradeoff.rs`、`examples/bench_ephemeris.rs`）

三种做法，前 3000—公元 3000 年每百年一点：

| 做法 | 级数速度 | 与 double-double 之差 | 是否违反 11.5 约定 |
|---|---|---|---|
| A 全 double-double（库缺省） | 基准 | — | 否 |
| B 双精度约化相位 + f64 sin/cos，double-double 求和 | 约 3 倍 | 月经度 4e-12″、月距 4e-9 m；地球 L 5e-13″、R 5e-7 m | 是 |
| C 全 f64 构建（`--no-default-features --features real-f64`，wasm 已用） | 约 20 倍（ELPMPP02 求和 0.42 ms 比 9.7 ms；月视位置 8.3 ms 比 184 ms；`gast` 40 µs 比 700 µs） | 月球完整位置 ≤ 2 cm（≤ 1e-5″） | 否（编译期后端切换，已有） |

B 只快 3 倍：三角函数便宜后，相位多项式、约化与乘加的 double-double 运算成了大头。此前按「单次 sin/cos 快 40 倍」估的
「级数快 40 倍」不成立，B 不值得为之破例，放弃。

同一校验程序两种构建的输出（表中所印位数）：
- 《汉书》《后汉书》凌犯（角距印到 0.01°）：逐格相同；double-double 并行 130 s，f64 并行 4.5 s。
- NASA 日月食典抽样 1/20（时刻印到 0.1 s，γ、食分印到 1e-4）：四表逐格相同；double-double 50 s，f64 3.1 s。

f64 的误差与其他误差源的量级（月球）：

| 误差源 | 量级 |
|---|---|
| f64 构建与 double-double 之差 | ≤ 1e-5″（2 cm） |
| f64 儒略日的时间分辨率（约 40 µs） | 月球移动约 2e-5″ |
| ELPMPP02 与 DE406 | 数百年内数毫角秒，数千年内数角秒（文献） |
| VSOP87（加改正）与 DE406：太阳视黄经 / 行星 | ≤ 0.16″ / ≤ 0.36″（本仓库实测） |
| ΔT 不确定度 σ（Morrison–Stephenson 2021）：前 200 年约 110 s、公元 150 年约 74 s | 月球约 1′、太阳约 4″ |

结论：天象检验、史书核验、批量搜索用 f64 构建，误差比模型误差小 4—5 个量级、比 ΔT 小约 7 个量级，速度约 20 倍（再乘并行约 3.5 倍）。
double-double 保留为库缺省与参考精度：回归测试、与外部参考逐位对照、发现数值问题时用它判断是否 f64 所致。
比对中发现极值时刻（大距、近距最接近）两种构建差至 16 s，查明为极值附近函数的数值噪声与黄金分割的比较方式所致，已修正：
DE406 读取器的时刻精度（段起点远在前 3000 年，f64 只到 3e-5 s）与极值求法（改为差分导数之根），详见
doc/astronomy/celestial-phenomena.md 第九节「极值时刻的数值问题」。修正后两种构建差 ≤ 0.2 s。

据此（2026-10-27 定）：`scripts/dev/phenomena_checks.sh` 缺省用 f64 构建，`PHENOMENA_REAL=dd` 改用 double-double；
`scripts/dev/phenomena_checks.sh compare` 两种构建各跑一组校验（NASA 日月食典抽样 1/20、行星、近距、掩星、恒星、伏见、宿度、
史书凌犯），输出到临时目录逐格比对（# 说明行除外），不一致即报错。每张校验表的说明行记所用数值后端（`test_util::NUMERIC_BACKEND`）。
实测（2026-10-27）：全量重算（含 NASA 日月食典全表、汉书至南齐书凌犯）f64 构建 300 s，原 double-double 串行约 1.5 小时；`compare` 694 s，13 张表一致。

后续：
- **多倍角递推**（ELPMPP02、章动的相位是基本辐角的整系数组合，可用复数乘法由基本辐角的 sin/cos 递推）：double-double 下约 2—3 倍；
  f64 构建下收益有限（三角函数已便宜），双精度下仍可考虑。
- **切比雪夫插值层**（JPL 星历表的做法：分段在切比雪夫节点取值，经 DCT 得系数，缓存后求值仅需微秒）：只在取样密于约 0.3 日
  的场合（日月食接触时刻迭代、节气定朔逐次逼近）显著；史书凌犯检验按 1 日取样（`phenomena::track`），节点数与之相当，收益小。
  做时应替换 `phenomena::track` 的 1 日四点拉格朗日插值（同一目标，精度更差），并给出相对直接级数求值的误差界。
- 大数 FFT 乘法不适用：热路径没有大整数运算（`BigRational` 只在古历复原，非热点）。

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
