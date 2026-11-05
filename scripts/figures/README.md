# 读者文档站的图表与长章节生成

读者文档站（`site/`）的统计图与内容较长的章节由本目录脚本从仓库数据和开发知识库（`doc/`）生成。动画另见 `scripts/manim/README.md`。

## 统计图：`make_charts.py`

输出 `site/public/charts/<名>.json`，由站点的 `<Chart src="名" />` 组件（`site/.vitepress/theme/Chart.vue`）绘制：折线（可带误差带）、散点、分组柱状，悬停提示，可展开数据表，深浅两种主题。配色取 dataviz 参考调色板前 4 个分类色（已用其校验脚本检查深浅两种模式）。

数据全部来自仓库内原始数据或检验结果，不手填：

| 图 | 数据来源 |
|---|---|
| `fit-earth-l`、`fit-earth-b` | VSOP87B 全级数与 DE406（复用 `scripts/ephemeris/vsop87_de_planet_fit.py`），改正系数 `data/fit/vsop87-de406-planets.txt` |
| `delta-t`、`delta-t-diff` | Espenak–Meeus 2006 多项式（与 `rust/core` 同系数）、`vendor/delta_t/morrison_stephenson_2021/` |
| `precession-p03-vondrak` | P03 三角式与 Vondrák 2011（与 `rust/core` 同系数） |
| `nutation-series` | `vendor/IAU2000/tab5.3a.txt` 日月项，IERS 2010 基本角 |
| `synodic-month-length`、`solar-term-intervals` | 参考表 `vendor/月相和二十四节气的计算/TDBtimes.txt` |
| `nasa-*` | `data/phenomena/checks/nasa_*.csv` |
| `records-conjunction` | `doc/astronomy/phenomena-checks.md` 第七部分总表（检验脚本生成） |
| `main-table-benji` | `doc/calendar/historical-calendar.md` 以本纪核验主表汇总 |

```bash
# 仓库根；需 DE406（vendor/jpl/de406/de406.bsp）
uv run --project scripts --only-group chapront-fit python scripts/figures/make_charts.py
```

## 长章节与附录：`build_site_pages.py`

天象（日月食、行星、恒星与二十八宿、当地所见的星空、史书天象检验）、历法（实历与主表、历代历法总录、各朝推步算法、秦汉初复原）各章及三个附录，由知识库对应文档转换生成：

- `kb_to_site.py` 做机械清理：去掉文件路径与模块路径、指向知识库其它文档的链接、开发日期戳（2025—2026 年）、表格中的「代码」列；函数名、体系代号等名字保留为等宽字体。专业内容（公式、参数、表格、检验数字、文献）不动。
- `build_site_pages.py` 按章节切分、拼装，删去讲实现位置与调用方式的小节，对清理后读不通之处逐条修补，并插入统计图。

知识库更新后重跑即可同步；某条修补不再匹配时会打印出来，须人工核对。手写的章节（天球、时间、岁差、章动、参考架、转动、历表、视位置、合朔、节气、农历编排、古历推步）不由脚本生成，改知识库时须同步改写。

```bash
python3 scripts/figures/build_site_pages.py
```
