# Chinese Lunar Calendar

天文计算驱动的农历/朔望项目：基于 VSOP87、ELPMPP02 等历表与岁差/章动/光行差计算日月视位置及合朔，农历以「朔」为月首。详见 [wiki Introduction](https://github.com/lhttjdr/calendar/wiki/Introduction)。

## 快速开始

| 部分 | 命令 |
|------|------|
| **Rust**（core + wasm-lib） | `cd rust && cargo build` |
| **Web**（Vite + React，Rust WASM 后端） | `cd web && npm install && npm run dev` |
| **文档**（VitePress） | `cd doc && npm install && npm run dev` |

- **数据**：历表在仓库根 `vendor/vsop87/`、`vendor/elpmpp02/`；Rust 运行或测试时工作目录为仓库根。
- **参考文献**：`doc/references/`；文档按《月相和二十四节气的计算》分章见 [doc/index.md](doc/index.md)，项目结构见 [doc/engineering/project-and-implementation.md](doc/engineering/project-and-implementation.md)。

## 项目结构

- **rust/** — Cargo workspace（core、wasm-lib）
- **web/** — 前端，Rust WASM 后端
- **vendor/** — 原样保留的第三方/来源数据，不重排/不改动内部文件：历表（`vsop87/`、`elpmpp02/`、`IAU2000/`、`jpl/`）、节气朔望参考表（`月相和二十四节气的计算/`）、`calendar/`（按来源分 `fang_shiming/`、`lifa_tongzhi/`、`chinese_calendar/` 三个子目录）
- **data/** — 本项目自己生成/整理：`fit/`（VSOP87–DE406 拟合）、`calendar/`（跨来源汇总的 `lifa.csv`、`regime_era_calendar_matrix.*`、说明文档）
- **doc/** — VitePress 文档站与 `doc/references/` 参考文献；按《月相和二十四节气的计算》章节索引见 [doc/index.md](doc/index.md)，构建后随 CI 发布到 `/docs/` 子路径
- **scripts/** — 数据生成/转换脚本，按用途分 `dev/`、`ephemeris/`、`calendar_data/`、`manim/` 子目录

## 功能状态

见仓库 wiki。已实现：数学（Decimal、Angle、Vector、Expression 等）、天文坐标与历表（VSOP87、ELPMPP02）、岁差/章动/光行差/大气折射、视位置流水线、日月合朔、儒略日↔公历等。
