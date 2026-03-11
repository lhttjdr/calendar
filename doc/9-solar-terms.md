# 9. 二十四节气与定气

定气：**视黄经 = 0°, 15°, 30°, …** 的时刻；pipeline 含光行时与光行差。实现仅保留**方案二**（VSOP87 几何 + P03 岁差 + IAU 2000A 章动，章动矩阵 R1(ε)R3(-Δψ)R1(-(ε+Δε)) 与 DE406/IERS 一致）。定气残差与时刻差容差 30 s（本实现 vs 参考表、本实现 vs DE406）。为满足国标 1 秒所做的**忽略与截断**（相对论/时空、矩阵与导数、观测者效应及站心 ENU）见 [6-light-time-and-aberration.md](6-light-time-and-aberration.md) §「为满足国标 1 秒的忽略与截断总览」。

## 9.1 定义与 pipeline 总览

**视黄经** = 几何黄经(tr) + 岁差 + 章动 - 光行差。输入：观测时刻 t → 光行时得 tr → r(tr), v(tr) = getSun(vsop, tr)。

**链**（太阳）：J2000_ECLIPTIC → J2000_EQUATORIAL_FK5 → J2000_EQUATORIAL_ICRS → J2000_EQUATORIAL（Vsop87De406IcrsPatchMixedTerms）→ MEAN_EQUATORIAL_OF_DATE → TRUE_EQUATORIAL_OF_DATE → [光行差] → APPARENT_ECLIPTIC；λ = atan2(Y_ec, X_ec)。

## 9.2 八步细化

| 步骤 | 名称 | 说明 |
|------|------|------|
| 1 | J2000黄道(入)，补丁前 | getSun(vsop, tr)，λ = atan2(y,x) |
| 2 | +FK5→赤道 | 黄赤旋转 |
| 3 | +ICRS | 固定旋转 |
| 4 | +patch→J2000_EQU | Vsop87De406IcrsPatchMixedTerms |
| 5 | +岁差→MEAN | Vondrak2011（或 P03），历元 tr |
| 6 | +章动→TRUE | SOFA iauNut00a |
| 7 | 真黄道 | 真黄经(tr) = 几何(tr) + 岁差 + 章动 |
| 8 | +光行差→视黄道 | e_app = e + (1/c)(v-(e·v)e)；视黄经 = 真黄经 - 光行差（约 -20.58″） |

时间自变量：整链在 **tr** 计算；岁差/章动/光行差用 tr 对应的 JD(TT)。

## 9.3 几何黄经(tr) 计算（太阳）

1. tr = t - τ（τ 约 497 s）；VSOP87 用 TDB。
2. (L_earth, B_earth, R_earth) = Vsop87.position(vsopEarth, tr)，日心系 J2000 平黄道。
3. 地心太阳：L_sun = L_earth + π，B_sun = -B_earth，R_sun = R_earth。
4. 球面→直角得 r(tr)、v(tr)；λ_geo(tr) = atan2(y, x)。与 DE406 同架比较时取**补丁后**（步骤 4 之后）的几何。

## 9.4 四步表（与 DE406 对比用）

- 几何(观测t) 真黄道：观测 t 的几何经整链到真黄道。
- 几何(推迟tr) 岁差后 平黄道：tr 几何 + 岁差。
- 岁差+章动后(tr) 真黄道：= 步骤 7。
- 光行差后(tr) 视黄道：= 步骤 8。

## 9.5 本实现 2026 春分示例

在本实现求出的「视黄经=0」的 JD 上：步骤 1 入 λ-0 = +20.58″；步骤 2～7 不变；步骤 8 光行差贡献 -20.58″，视黄经 = 0。

测试入口：`SolarTermTest`（「2026 二十四节气 vs DE406」「pipeline 节点打印与 DE406 逐步对比」）。岁差/章动/光行差模型详见 [3-precession.md](3-precession.md)、[4-nutation.md](4-nutation.md)、[6-light-time-and-aberration.md](6-light-time-and-aberration.md)。

## 9.6 节气朔望标准时刻表（《月相和二十四节气的计算》§7.4）

定气/定朔标准时刻对照表存放于 **data/月相和二十四节气的计算/**，格式依《月相和二十四节气的计算》§7.4，如下。

### 参考表的计算方法

参考表中各时刻由《月相和二十四节气的计算》所述方法计算得到，与本项目定气/定朔的物理定义一致：

- **历表**：主表用 **JPL DE441** 提供日、地、月几何（TDB 为时间自变量）；extended 表同源、仅岁差模型不同。
- **光行时**：观测时刻 t 下迭代得到推迟时 tr，取 tr 时刻的几何位置（或等价使用 Xproper = x(tr)−xE(tr) 折叠光行差）。
- **岁差与章动**：主表采用 **IAU 2006** 岁差 + **IAU 2000A** 章动；**TDBtimes_extended.txt** 采用 **Vondrák 2011** 岁差（见 [3-precession.md](3-precession.md)），章动仍为 IAU 2000A。
- **定气**：太阳视黄经 λ_S（经岁差、章动、光行差到视黄道）达到 0°, 15°, … 的 **TDB 时刻**。
- **定朔/望/弦**：地心视黄经差 λ_M − λ_S 达到 0, π, π/2, 3π/2 的 **TDB 时刻**。

因此本实现与参考表在「同一历表+岁差章动+光行时/光行差约定」下可比对；差异主要来自历表（本项目定气用 VSOP87+DE406 拟合、定朔用 ELPMPP02）与参考表所用 DE441 的差异，以及岁差/章动实现细节。

### 表格式

- **第 1 栏**：公历年。
- **第 2 栏**：jd0，该年 1 月 0 日（TDB+8）零时的儒略日数。
- **第 3 栏**：Z11a，最接近 jd0 的**冬至**时刻（相对 jd0 的日数）；对表涵盖年份，此为前一公历年的冬至。
- **第 4–27 栏**：Z11a 以后的二十四节气相对 jd0 的日数，表头为 **J12 Z12 J01 Z01 J02 Z02 … J11 Z11b**，即 **小寒、大寒、立春、雨水、惊蛰、春分、清明、…、冬至**（春分在第 8 列，冬至在第 27 列）。
- **第 28 栏起**：Q0_01 Q1_01 Q2_01 Q3_01 Q0_02 …，15 个朔望月内的 **朔(Q0)、上弦(Q1)、望(Q2)、下弦(Q3)**，各为相对 jd0 的日数。

主表 **TDBtimes.txt** 用 DE441 + IAU2006/2000A 岁差章动；**TDBtimes_extended.txt** 用 Vondrák 2011 岁差，见 [3-precession.md](3-precession.md)。定气列映射见 `rust/core/src/astronomy/aspects/solar_term/term_jd.rs` 中 `load_solar_terms_reference_jd`。
