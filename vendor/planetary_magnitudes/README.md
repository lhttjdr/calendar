# Ap_Mag：行星视星等程序与测试数据（Mallama & Hilton 2018）

原样保留的第三方数据，不改动内部文件。来源：SourceForge 项目 planetary-magnitudes
<https://sourceforge.net/projects/planetary-magnitudes/>（「Files」页各版本目录），2026-10-26 由用户用 SourceForge 批量下载脚本
取得；下载时文件名带有下载链接的查询串（`?viasf=1&fid=…`），归档时去掉，文件内容未改动。

| 目录 | 内容 |
|---|---|
| `Ap_Mag_Current_Version/` | 当前版本 V3：`Ap_Mag_V3.f90`（Fortran 源码）、`Ap_Mag_Input_V3.txt`（HORIZONS 几何量 + 预算星等，27 例）、`Ap_Mag_Output_V3.txt`（程序自检输出，误差全为 0） |
| `Ap_Mag_Version_2/` | V2（加入超出地心相位角范围的公式）：源码、输入、输出 |
| `Ap_Mag_Version_1/` | V1：源码、输出、`HORIZONS_plus_new_magnitudes.txt` |

文献：Mallama A., Hilton J.L. 2018, Computing apparent planetary magnitudes for The Astronomical Almanac, *Astronomy and Computing*
25, 10；arXiv:1808.01973（附录说明本程序与测试数据：与预算星等之差 ≤ 0.001 即实现正确）。火星自转、季节修正表取自
Mallama A. 2007, *Icarus* 192, 404（程序 `Mars_Stirling` 中的 L1、L2）。

许可：文件中未见许可声明；SourceForge 项目页（可能注明许可）本次访问被拒（HTTP 403），未能核对。仅作本地验证与对照用。

本仓库用法：`rust/core/src/astronomy/apparent/magnitude.rs` 的测试 `matches_ap_mag_v3_official_test_data` 直接解析
`Ap_Mag_Input_V3.txt`；火星修正表照录 `Ap_Mag_V3.f90`。程序中 `Mars_Stirling` 的差分循环 `do i = 0, 2` 只算到 δ(2)，四阶差分
用到未赋值的 δ(3)（注释原意为 `range(4)`），本仓库按完整 Stirling 公式实现，官方 27 例仍全部在 0.001 星等以内。
