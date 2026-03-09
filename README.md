# Chinese Lunar Calendar

天文计算驱动的农历/朔望项目：基于 VSOP87、ELPMPP02 等历表与岁差/章动/光行差计算日月视位置及合朔，农历以「朔」为月首。详见 [wiki Introduction](https://github.com/lhttjdr/calendar/wiki/Introduction)。

## 快速开始

| 部分 | 命令 |
|------|------|
| **Rust**（core + wasm-lib） | `cd rust && cargo build` |
| **Web**（Vite + React，Rust WASM 后端） | `cd web && npm install && npm run dev` |

- **数据**：历表在仓库根 `data/vsop87/`、`data/elpmpp02/`；Rust 运行或测试时工作目录为仓库根。
- **参考文献**：`doc/references/`；文档按《月相和二十四节气的计算》分章见 [doc/INDEX.md](doc/INDEX.md)，项目结构见 [doc/11-project-and-implementation.md](doc/11-project-and-implementation.md)。

## 项目结构

- **rust/** — Cargo workspace（core、wasm-lib）
- **web/** — 前端，Rust WASM 后端
- **data/** — 历表数据
- **doc/** — 文档与 `doc/references/` 参考文献；按《月相和二十四节气的计算》章节索引见 [doc/INDEX.md](doc/INDEX.md)
- **scripts/** — 脚本

## 功能状态

见仓库 wiki。已实现：数学（Decimal、Angle、Vector、Expression 等）、天文坐标与历表（VSOP87、ELPMPP02）、岁差/章动/光行差/大气折射、视位置流水线、日月合朔、儒略日↔公历等。
