# 文档索引（按《月相和二十四节气的计算》章节）

本仓库文档分 15 章，每章一个文件。参考文献见 `doc/references/`；编算农历网页见 [ChineseCalendar](https://ytliu0.github.io/ChineseCalendar/computation_simp.html)。

| 章 | 文件 | 内容 |
|----|------|------|
| **1** | [1-time-and-timescales.md](1-time-and-timescales.md) | 时间与儒略日、时标、ΔT |
| **2** | [2-reference-frames.md](2-reference-frames.md) | 参考系与坐标、Frame Bias、管线三正交与设计原则、几何与架 |
| **3** | [3-precession.md](3-precession.md) | 岁差（P03、Vondrák 2011） |
| **4** | [4-nutation.md](4-nutation.md) | 章动（IAU 2000A） |
| **5** | [5-ephemerides-and-de-align.md](5-ephemerides-and-de-align.md) | 历表与 DE 对齐（VSOP87、Chapront、ELPMPP02） |
| **6** | [6-light-time-and-aberration.md](6-light-time-and-aberration.md) | 光行时与光行差 |
| **7** | [7-apparent-longitude-and-syzygy.md](7-apparent-longitude-and-syzygy.md) | 视位置计算（因素与两条路径、Rust 管线 §7.2、其它效应） |
| **8** | [8-lunar-phases-and-syzygy.md](8-lunar-phases-and-syzygy.md) | 月相与合朔、望/上弦/下弦、W0 公式、精度事实 |
| **9** | [9-solar-terms.md](9-solar-terms.md) | 二十四节气与定气、八步 pipeline、节气朔望标准时刻表格式 |
| **10** | [10-lunar-calendar.md](10-lunar-calendar.md) | 农历编算、流程与分层 |
| **11** | [11-project-and-implementation.md](11-project-and-implementation.md) | 项目结构、Rust 实现、Real 后端比较、f64 约定与物理量审计 |
| **12** | [12-wasm-react-performance.md](12-wasm-react-performance.md) | Rust+WASM+React 性能调优与边界优化 |
| **13** | [13-ephemeris-binary-format.md](13-ephemeris-binary-format.md) | 历表二进制化与零解析方案（VSOP87/ELPMPP02） |
| **14** | [14-data-paths-summary.md](14-data-paths-summary.md) | 历表与数据路径总结（VSOP87/ELPMPP02/DE406 路径、标架、时间尺度） |
| **15** | [15-pipeline-graph-taxonomy.md](15-pipeline-graph-taxonomy.md) | 管线图：节点与边的分类与抽象（OriginRole、TransitionKind/Form） |

---

**参考文献**：`doc/references/`；仓库根 [README.md](../README.md) 快速开始与项目结构总览。
