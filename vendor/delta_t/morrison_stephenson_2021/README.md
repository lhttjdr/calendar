# ΔT：Morrison, Stephenson, Hohenkerk & Zawilski (2021)

原样保留的第三方数据，不改动内部文件。

| 文件 | 内容 | 来源 |
|---|---|---|
| `Table-S15.2020.txt` | ΔT 样条的分段三次多项式系数，-720.0 ～ 2019.0 | figshare [10.6084/m9.figshare.29920388](https://doi.org/10.6084/m9.figshare.29920388)，2025-10-16 |
| `Table-DT-lod4500yrs.2020.txt` | ΔT 与日长变化及其误差估计，-2000 ～ +2500（样条范围外为作者外推） | figshare [10.6084/m9.figshare.30111661](https://doi.org/10.6084/m9.figshare.30111661)，2025-10-16 |

- 作者：Leslie V. Morrison, F. Richard Stephenson, Catherine Y. Hohenkerk, Marek Zawilski
- 许可：CC BY 4.0（使用须署名）
- 文献：Morrison L.V., Stephenson F.R., Hohenkerk C.Y., Zawilski M. 2021, "Addendum 2020 to 'Measurement of the Earth's
  rotation: 720 BC to AD 2015'", *Proc. R. Soc. A* 477: 20200776, <https://doi.org/10.1098/rspa.2020.0776>；
  原论文 Stephenson F.R., Morrison L.V., Hohenkerk C.Y. 2016, *Proc. R. Soc. A* 472: 20160404,
  <https://doi.org/10.1098/rspa.2016.0404>。
- 月球潮汐加速度：原论文称这些 ΔT「应与 JPL DE430 月历表配合使用，或与潮汐加速度接近 DE430 隐含值 −25.82″/cy² 的历表配合」
  （2016 原文 §2）。HMNAO 随数据发布的说明文件另写作 −25.85″/cy²，二者差 0.03（前 200 年约 13 s，远小于该时期 σ≈110 s）；
  本仓库以原论文 −25.82 为准。
- 长期外推：原论文式 (4.1) ΔT = −320.0 + (32.5 ± 0.6)·u² s，u = (年 − 1825)/100。

使用见 `rust/core/src/astronomy/time/delta_t.rs`。

**已知数据问题**：`Table-DT-lod4500yrs.2020.txt` 中 −700.0 年一行 ΔT 作 20250 s，与同表 −720（20370）、−600（18470）及样条
系数在 −700 的值（≈20050）均不协调，疑为 20050 之误。代码在样条范围（−720～2019）内只用样条系数，不受影响；文件原样保留。
