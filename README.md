# Chinese Lunar Calendar

从太阳、月亮的实际运行出发，算出每一个合朔、每一个节气，排出农历；再按历代历法的术文复原古人实际行用的历日，用史书的日期与天象记录检验；民俗的纪时、择日、式占、命理按原典逐条实现，各家并列。

- **在线日历**：<https://lhttjdr.github.io/calendar/>
- **文档**：<https://lhttjdr.github.io/calendar/docs/>（应用顶栏「文档」也可进入）。原理、源流、各家做法与本项目的检验都在这里。

## 内容

文档站与代码按同一结构分四篇，依赖自上而下：

| 篇 | 内容 |
|---|---|
| 数学 | 向量与转动（含四元数、对偶四元数）、数值精度（f64 与双双精度）、级数求值、求根与极值、最小二乘拟合 |
| 天文 | 天球坐标、时标与 ΔT、参考架、岁差（Vondrák 2011 等）、章动（IAU 2000A）、历表（VSOP87 与 ELP/MPP02，拟合 JPL DE 数值历表的改正）、视位置；合朔与月相、二十四节气、日月食、行星、恒星与二十八宿、当地星空、史书天象检验 |
| 历法 | 历日与纪日、现行农历编排（GB/T 33661—2017）、历史上实际行用的历日（实物历日、按术推步与今人历表）与各政权历、古历复原与平气、历代历法总录、年号纪年与改元 |
| 民俗 | 干支历四柱、纳音与值日、更点、杂节气、建除与黄道、神煞宜忌、婚嫁择吉、奇门遁甲、大六壬、太乙、周易与六爻、七政四余、紫微斗数、三元玄空 |

计算全部在 Rust 中完成，编译成 WebAssembly 在浏览器里运行。

## 快速开始

Rust、Node、wasm-pack 的版本由根目录 `mise.toml` 统一管理：

```bash
mise trust && mise install
```

| 部分 | 命令 |
|------|------|
| Rust 核心库 | `cd rust && cargo build`；测试 `cd rust && cargo test -p lunar-core --no-default-features --features real-f64` |
| WASM 包 | `./scripts/dev/build_wasm.sh`（同时构建 `wasm-lib` 与 `wasm-lib-f64`） |
| Web 应用 | `cd web && npm install && npm run copy-data && npm run dev` |
| 文档站 | `cd site && npm install && npm run dev` |
| 数据脚本（Python） | `uv run --project scripts python scripts/...`，在仓库根执行 |

- Rust 运行、测试时工作目录须为仓库根（数据路径以仓库根为基准）。
- `cargo test -p lunar-core` 缺省启用 `python-test` 特性，需要 Python 环境，见 `scripts/dev/run_tests_with_venv.sh`。

## 目录

| 目录 | 内容 |
|------|------|
| `rust/` | Cargo workspace：`core`（数学、天文、历法、民俗）、`wasm-lib`（双双精度）、`wasm-lib-f64` |
| `web/` | 前端应用（Vite + React），以 WASM 为计算后端 |
| `site/` | 面向读者的文档站（VitePress），发布到 `/docs/` |
| `doc/` | 开发与 AI 用的知识库，含代码路径与逐条检验，不发布；索引见 [doc/index.md](doc/index.md) |
| `data/` | 本项目生成或整理的数据：历表拟合改正、历法与年号汇总、天象检验结果、星表与星官、民俗数据 |
| `vendor/` | 原样保留的第三方与来源数据（历表、星表、历日表、古籍录文等），不改动 |
| `scripts/` | 数据生成与转换脚本（历表拟合、历法数据、天象检验、统计图、动画） |

更详细的结构与开发约定见 [AGENTS.md](AGENTS.md)。
