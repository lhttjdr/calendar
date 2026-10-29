# 文档索引

本仓库文档按主题分为五部分：**总体架构**（三大域与域内四段结构）、**天文基础**（时标、参考系、岁差章动、历表、视位置、月相节气等推导）、**农历编算**（由天文量算出农历、实历与古历复原）、**工程实现**（Rust/WASM/前端与数据格式）、**案例分析**（具体问题的排查记录）。参考文献见 [doc/references/](https://github.com/lhttjdr/calendar/tree/master/doc/references)；编算农历网页见 [ChineseCalendar](https://ytliu0.github.io/ChineseCalendar/computation_simp.html)。

## 〇、总体架构（`architecture/`）

| 文件 | 内容 |
|------|------|
| [three-domains.md](architecture/three-domains.md) | 天文/历法/民俗三大域，域内共同的"独立基础层→聚合/实算层→派生层→仲裁/路由层"四段结构，各域具体模块映射 |

## 一、天文基础（`astronomy/`）

依《月相和二十四节气的计算》章节顺序，从时标一路推导到视位置、月相与节气。

| 文件 | 内容 |
|------|------|
| [time-and-timescales.md](astronomy/time-and-timescales.md) | 时间与儒略日、时标、ΔT |
| [reference-frames.md](astronomy/reference-frames.md) | 参考系与坐标、Frame Bias、管线三正交与设计原则、几何与架 |
| [precession.md](astronomy/precession.md) | 岁差（P03、Vondrák 2011） |
| [nutation.md](astronomy/nutation.md) | 章动（IAU 2000A） |
| [ephemerides-and-de-align.md](astronomy/ephemerides-and-de-align.md) | 历表与 DE 对齐（VSOP87、Chapront、ELPMPP02） |
| [light-time-and-aberration.md](astronomy/light-time-and-aberration.md) | 光行时与光行差 |
| [apparent-longitude-and-syzygy.md](astronomy/apparent-longitude-and-syzygy.md) | 视位置计算（因素与两条路径、Rust 管线 §7.2、其它效应） |
| [lunar-phases-and-syzygy.md](astronomy/lunar-phases-and-syzygy.md) | 月相与合朔、望/上弦/下弦、W0 公式、精度事实 |
| [solar-terms.md](astronomy/solar-terms.md) | 二十四节气与定气、八步 pipeline、节气朔望标准时刻表格式 |
| [celestial-phenomena.md](astronomy/celestial-phenomena.md) | 天象模块：日月食、行星会合与视星等、近距与月掩星、伏见可见性、恒星与二十八宿；设计、实施进展与校验结论 |
| [phenomena-checks.md](astronomy/phenomena-checks.md) | 天象模块逐条校验结果（脚本生成）：NASA 日月食典、史书日月食与凌犯、Skyfield 对照、原文表格复现等 |

## 二、农历编算（`calendar/`）

从天文量到农历日期的编算逻辑，以及历史区间的实历与古历复原。

| 文件 | 内容 |
|------|------|
| [lunar-calendar.md](calendar/lunar-calendar.md) | 农历编算、流程与分层 |
| [calendar-layers.md](calendar/calendar-layers.md) | 历法域内部四层结构（独立基础层/聚合实算层/派生层/仲裁路由层），`rust/core/src/calendar/` 模块划分依据 |
| [historical-calendar.md](calendar/historical-calendar.md) | 实历模块：历史朔日表、优先实历的公历↔农历换算 |
| [main-table-checks.md](calendar/main-table-checks.md) | 以本纪核验主表逐条结果（脚本生成）：明书朔日、实际合朔、閏月位置 |
| [ancient-calendar-restore.md](calendar/ancient-calendar-restore.md) | 古历复原：分数推步、平朔平气、历谱生成；三千五百年仅作验证 |
| [push-algorithms.md](calendar/push-algorithms.md) | 各朝代/政权历法推步算法与历术参数总结 |
| [qin-han-calendar-evaluation.md](calendar/qin-han-calendar-evaluation.md) | 秦与汉初（前221—太初元年四月）历法复原接入评估：与主表衔接、《汉书》检验、待补的日月食与出土材料验证 |

## 三、工程实现（`engineering/`）

项目结构、Rust/WASM/前端实现与数据格式。

| 文件 | 内容 |
|------|------|
| [project-and-implementation.md](engineering/project-and-implementation.md) | 项目结构、Rust 实现、Real 后端比较、f64 约定与物理量审计 |
| [wasm-react-performance.md](engineering/wasm-react-performance.md) | Rust+WASM+React 性能调优与边界优化 |
| [ephemeris-binary-format.md](engineering/ephemeris-binary-format.md) | 历表二进制化与零解析方案（VSOP87/ELPMPP02） |
| [data-paths-summary.md](engineering/data-paths-summary.md) | 历表与数据路径总结（VSOP87/ELPMPP02/DE406 路径、标架、时间尺度） |
| [pipeline-graph-taxonomy.md](engineering/pipeline-graph-taxonomy.md) | 管线图：节点与边的分类与抽象（OriginRole、TransitionKind/Form） |

## 四、案例分析（`case-studies/`）

| 文件 | 内容 |
|------|------|
| [de406-solar-term-failure-analysis.md](case-studies/de406-solar-term-failure-analysis.md) | DE406 定气 vs TDBtimes 失败原因分析 |

---

**参考文献**：[doc/references/](https://github.com/lhttjdr/calendar/tree/master/doc/references)；仓库根 [README.md](https://github.com/lhttjdr/calendar/blob/master/README.md) 快速开始与项目结构总览。AI 辅助开发用的项目总览见仓库根 [INVENTORY.md](https://github.com/lhttjdr/calendar/blob/master/INVENTORY.md)（不纳入本文档站）。
