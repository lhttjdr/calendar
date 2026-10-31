# AGENTS.md

面向在本仓库工作的编码 agent 的说明。本项目是**天文计算驱动的农历/朔望**项目：基于 VSOP87、ELPMPP02 等历表，结合岁差/章动/光行差计算日月视位置及合朔，农历以「朔」为月首。项目文档、代码注释、commit 习惯均以中文为主，回复与新增注释请保持中文。

## 项目结构

| 部分 | 目录 | 说明 |
|------|------|------|
| Rust 核心 | `rust/core` | 数学（Real/Decimal/Angle/Vector）、天文（VSOP87/ELPMPP02/岁差/章动/视位置管线）、历法（calendar，见下） |
| WASM 绑定 | `rust/wasm-lib`（TwoFloat 高精度）、`rust/wasm-lib-f64`（Real=f64，体积小/速度快） | wasm-bindgen 暴露给前端 |
| Web 前端 | `web/` | Vite + React + antd，使用 Rust WASM 作为农历计算后端 |
| 数据 | `vendor/`、`data/` | `vendor/`：原样保留的第三方/来源数据，不重排、不改动内部文件，即使部分文件未被引用——历表（`vsop87/`、`elpmpp02/`、`IAU2000/`、`jpl/`；`jpl/de441/`、`jpl/de431/` 为 GB 级本地验证用，不入库）、行星视星等程序与测试数据（`planetary_magnitudes/`，Mallama & Hilton 2018 的 Ap_Mag）、恒星星表（`hipparcos/`，CDS I/311 新归算与 I/239 原版主表；53 MB 的 `I_239/hip_main.dat` 不入库）、中国星空文化（`stellarium_skycultures/`，Stellarium 五套中国星官体系，CC BY-SA 4.0）、节气朔望参考表（`月相和二十四节气的计算/`）、`calendar/` 下按来源分三个子目录：`fang_shiming/`（方诗铭《中国历史纪年表》PDF+OCR）、`lifa_tongzhi/`（朱文鑫《历法通志》）、`chinese_calendar/`（ytliu0/ChineseCalendar 提取的 `年号表.csv`、`改元候选表.csv`、`实历朔日表.csv`，及主表以外各政权推步的 `分政权朔日表.csv`、`分政权年号表.csv`；均为复原结果，不是史实记录）。`data/`：本项目自己生成/整理——`fit/`（VSOP87 拟合到 DE406/DE441 的日心改正，Chapront 2000 方法）、`phenomena/`（史书日月食、凌犯记录解析与天象模块检验结果；`capitals.csv` 为各政权都城，Rust 检验与 Python 脚本共用；`checks/` 为逐条校验结果，由 `scripts/dev/phenomena_checks.sh` 生成）、`stars/`（由 Hipparcos 生成的亮星表 `hipparcos_bright.csv`）、`sky_culture/`（二十八宿距星表、《元史》历代实测宿度）、`calendar/`（`README_shuori.md`/`SOURCES_era_regime.md`；`汇总/` 子目录放跨来源汇总产物 `lifa_summary.csv`、`regime_era_calendar_matrix.*`；`fang_shiming/` 子目录放按单一来源整理的方诗铭纪年表转录）；三大域/历法域分层等架构说明在 `doc/architecture/`、`doc/calendar/`（见下「文档」行），不放 `data/` 下 |
| 本地参考（不入库） | `doc/references/history/点校本二十四史精装版*/` | 中华书局点校本二十四史扫描 PDF（2.8 GB，无文字层，已 gitignore）：按需读页作依据，引用时注明册与页，不要整本提交 |
| 文档 | `doc/` | 按《月相和二十四节气的计算》分章，索引见 [doc/index.md](doc/index.md)；AI 开发用总览见 [INVENTORY.md](INVENTORY.md)；文档配图/动画用 Manim 本地渲染、产物提交到 `doc/public/media/`，见 [scripts/manim/README.md](scripts/manim/README.md) |
| 脚本 | `scripts/` | 数据生成/转换脚本，按用途分子目录：`dev/`（构建/测试/覆盖率）、`ephemeris/`（历表拟合、章动表抓取、二进制生成与压缩）、`calendar_data/`（纪年/朔日表处理；`qinhan_eval/zhi_star_records.py` 为正史天文志通用解析器）、`phenomena/`（星表提取、Skyfield 星位对照、天象检验汇总）、`sky_culture/`（距星与史载宿度整理）、`manim/`（文档配图/动画） |

**Rust 运行/测试时工作目录须为仓库根**（数据路径以仓库根 `data/` 为基准）；Web 从 `/data/...` 拉取，需先 `npm run copy-data`。

## 三大域与历法域四层结构（重点，改动前务必读 `doc/architecture/three-domains.md` 与 `doc/calendar/calendar-layers.md`）

计算逻辑分三个大域，依赖单向：**天文**（astronomy，朔望/节气/月相/日月食）→ **历法**（calendar，干支历/农历各版本/政权年号）→ **民俗**（folk，八字/黄道黑道/吉凶星，尚未实现）。三个域内部都是同一种四段形状：独立基础层→聚合/实算层→派生层→仲裁/路由层（横切，职责是"选"不是"算"，不要和前三层的计算逻辑混在同一处，否则会像 `historical.rs`/`convert.rs` 那样难以复用）。完整说明见 [doc/architecture/three-domains.md](doc/architecture/three-domains.md)。

`rust/core/src/calendar/` 内文件对应四层：
- **1 独立基础层**：`gregorian`（前推格里高利历 `to_julian_day`；民用历 `civil_to_julian_day`，1582-10-15 前儒略历——史料与 `实历朔日表` 日期用后者）、`ancient_restore`（古历复原：分数推步、平朔平气，仅作验证）、`push_calendar`（历谱生成；定朔的金重修大明历在 `daming_li`）、`gan_zhi`（干支纯循环命名）、`options`。
- **2 聚合/实算层**：`chinese_lunar`（现算真朔真气）、`historical`（`HistoricalCalendar`，查历史朔日表为主表提供岁数据）。
- **3 派生层**：`era_names`（`EraNames` 解析 `data/calendar/汇总/era_names_merged.csv`，同一年可能有多个并存政权）、`era_gaiyuan`（处理改元）、`month_labels`（历史月序改制：主表一律建寅编月，新莽/景初建丑、武周/肃宗末建子的当时月名经 `data/calendar/month_label_reforms.csv` 换算）。
- **4 仲裁/路由层**：`lifa`（`LifaRegistry` 解析 `data/calendar/汇总/lifa_summary.csv`）、`convert`（"实历优先、缺省退回定气定朔"判断）。

`ancient_restore`/`push_calendar` 用于验证古历，与「实历优先」的 `convert`/`chinese_lunar` 换算路径是两条不同路径，不要混用。详见 [doc/calendar/historical-calendar.md](doc/calendar/historical-calendar.md)、[doc/calendar/ancient-calendar-restore.md](doc/calendar/ancient-calendar-restore.md)。

## 工具链（mise）

Rust / Node / `wasm-pack` 版本由根目录 [mise.toml](mise.toml) 统一管理（[mise](https://mise.jdx.dev/)）。进入仓库后执行一次 `mise trust`（首次信任配置），再 `mise install` 安装 pin 住的版本；激活 mise 的 shell 会自动切到这些版本，未激活时可用 `mise exec -- <命令>`。

- `scripts/` 下的 Python 脚本自成一个 uv 工程：`scripts/pyproject.toml`/`scripts/uv.lock`/`scripts/uv.toml`，**不**放在仓库根——它只是数据生成/转换的辅助工具，不属于本项目（Rust + Web）的主工具链，不要往根目录加 `pyproject.toml`/`uv.lock`，也不要往根 `mise.toml` 里加 `python`/`uv`。
  - 脚本内部按「工作目录为仓库根」写死相对路径（如 `vendor/vsop87/...`），所以调用 uv 时**留在仓库根**执行，用 `uv run --project scripts ...` 指定工程目录，**不要** `cd scripts` 再跑：例如 `uv run --project scripts --only-group chapront-fit python scripts/ephemeris/vsop87_de_planet_fit.py de406`。
  - venv 相应地在 `scripts/.venv`（`uv --project scripts venv`），而不是仓库根 `.venv`。

## 构建与测试

```bash
# Rust：核心库 + wasm 包
cd rust && cargo build
cd rust && cargo test -p lunar-core

# 含 Python/DE406 对比测试（VSOP87/ELPMPP02 vs jplephem），需先 uv --project scripts venv 并安装 jplephem
scripts/dev/run_tests_with_venv.sh

# 天象模块逐条校验：重算 data/phenomena/checks/*.csv 并生成 doc/astronomy/phenomena-checks.md
# 缺省用 f64 构建（约 20 倍快，误差 ≤1e-5″，见 doc/engineering/project-and-implementation.md §11.2.2）；PHENOMENA_REAL=dd 用 double-double
scripts/dev/phenomena_checks.sh
scripts/dev/phenomena_checks.sh compare   # 两种构建各跑一组校验并逐格比对，不改仓库文件

# 覆盖率（同 CI，见 .github/workflows/coverage.yml）
cd rust && cargo llvm-cov --all-features -p lunar-core --lcov --output-path lcov.info

# Web
cd web && npm install && npm run dev
cd web && npm run copy-data      # 同步历表/纪年数据到 web/public/data
cd web && npm run refresh-wasm   # 仅刷新 wasm-lib（TwoFloat）
cd .. && ./scripts/dev/build_wasm.sh # 同时构建 wasm-lib 与 wasm-lib-f64
```

- `cargo test -p lunar-core` 默认启用 `python-test` feature，需要能找到 Python（见 `run_tests_with_venv.sh` 里的 `PYO3_PYTHON`/`PYTHONHOME` 设置），否则部分测试无法编译/运行。
- `wasm-lib` / `wasm-lib-f64` 依赖 `cfg(target_arch = "wasm32")` 代码，无法在 host 上直接跑测试/覆盖率，只对 `lunar-core` 跑。
- Python 侧用 `uv`（工程在 `scripts/`，见上「工具链」一节），历表拟合是可选 dependency-group（`chapront-fit`）。

## Rust 代码约定

- **标量类型统一用 `Real`**（`rust/core/src/math/real`），编译期在 twofloat（高精度）与 f64（wasm 体积/速度）间切换；**不要**在 core 业务逻辑里直接用裸 `f64`，只在以下边界允许：`math/real` 内部、与 `[f64;3]`/矩阵交互的写入处（`.as_f64()`）、wasm/FFI 导出层。
- 有物理含义的量优先用物理量类型（`PlaneAngle`、`Length`、`Duration` 等）而非裸数值，历史遗留的裸 f64/元组正在逐步收紧（见 [doc/engineering/project-and-implementation.md](doc/engineering/project-and-implementation.md) §11.5、§11.6）。
- 视位置计算走统一管线架构（`astronomy::pipeline`：State6/StateTransition6、EphemerisProvider、FrameMapper、LightTimeCorrector、TransformGraph），对外 API 仍是 `astronomy::apparent` 里的 `sun_position_icrs` 等函数，改动内部实现时不要破坏这层外部 API。
- 历表/章动等数据统一经 `repo` 模块读写路径（`repo::paths`、`repo_root()`），Native 与 Wasm（宿主注入 loader）共用同一套 `*_from_repo()`，新增数据加载遵循这个模式。

## 其它

- 资料可靠性有明确排序（实物历日 > 史书明书日期 > 史志术文 > 按术文的推算 > 今人历表 > 天文计算；问天象时天文计算最高），见 [doc/calendar/calendar-compendium.md](doc/calendar/calendar-compendium.md)「资料可靠性排序」；证据只能由上往下检验。

- 提交信息、文档、注释使用中文，与仓库现有风格一致。
- 大改动前先看 [doc/index.md](doc/index.md) 定位对应章节文档，避免与已有设计冲突；数据格式变更需同步更新对应 `data/calendar/README_shuori.md`（或 `data/calendar/fang_shiming/README.md` 等按来源整理的子目录 README）。
