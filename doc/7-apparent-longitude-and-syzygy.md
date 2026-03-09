# 7. 视位置计算

视黄经 λ = atan2(Yec, Xec)，由几何位置经岁差、章动、光行差得到瞬时真黄道坐标。本章专门论述**视位置计算**：因素与两条路径（FK5 时代 vs 现代 ICRS→GCRS）、本程序 pipeline 概览、视位置中的其它效应（光行差分类、视差、自行）。月相与合朔见 [8-lunar-phases-and-syzygy.md](8-lunar-phases-and-syzygy.md)。

## 7.1 视位置计算的因素与两条路径

### 视位置计算涉及的因素

视位置（视黄经、视赤经赤纬等）由以下因素依次作用得到（顺序与物理含义见 [2-reference-frames.md](2-reference-frames.md)、[3-precession.md](3-precession.md)、[4-nutation.md](4-nutation.md)、[6-light-time-and-aberration.md](6-light-time-and-aberration.md)）：

| 因素 | 作用 | 说明 |
|------|------|------|
| **光行时** | 观测时刻 t → 推迟时 tr | 几何位置取 r(tr)、v(tr)；tr 由 t − D/c 迭代得到。 |
| **历表几何** | 在某一参考架下的位置与速度 | 如 J2000.0 平黄道/平赤道（FK5）或 ICRS/GCRS。 |
| **参考架与架变换** | 历表架 → 当日历元 | 含 FK5↔ICRS 的 **Frame Bias B**（若从 ICRS 进经典岁差章动链）、**岁差 P**、**章动 N**。 |
| **岁差** | J2000.0 平赤道/平黄道 → 当日历元**平**赤道/平黄道 | 长期平滑转动，矩阵 P。 |
| **章动** | 当日历元平赤道 → 当日历元**真**赤道（真黄道） | 短周期摆动，矩阵 N；真黄经 = 几何(tr) + 岁差 + 章动。 |
| **光行差（周年）** | 真方向 → **视**方向 | 观测者（地心）速度 v/c 的一阶改正，约 −20.5″。 |

未实现的效应（在 §7.3 讨论）：周日光行差、长期光行差、视差、自行。

### 两条实现路径：FK5 时代 vs 现代 ICRS→GCRS

视位置的**数学步骤**（岁差 → 章动 → 光行差）一致，但**历表给出的参考架**不同，形成两条常见实现路径。

**路径一：FK5 时代（历表在 J2000.0 平架）**

- **几何架**：历表直接给出 **J2000.0 平赤道 / 平黄道**（与 FK5、DE200、VSOP87 等一致），无 ICRS 步骤。
- **链**：**J2000 平**（黄道或赤道）→ **岁差 P**（如 L77 或 P03）→ **平 of date** → **章动 N**（如 IAU 1980 或 2000A）→ **真 of date** → **周年光行差** → 视黄道/视位置。
- **特点**：整链在「动力学 J2000 平架」下；岁差、章动、光行差依次作用即可。经典星表、DE200、VSOP87 等采用此路径。

**路径二：现代 ICRS→GCRS（历表在 ICRS/BCRS 或 GCRS）**

- **几何架**：历表给出 **ICRS**（质心 BCRS）或 **GCRS**（地心，轴向与 ICRS 平行）；如 DE406、本程序太阳经 FK5→ICRS 与 Vsop87De406IcrsPatchMixedTerms 后的 **J2000_EQU**（实质为 ICRS 对齐的赤道架）。
- **到「平 of date」**：若沿用**经典岁差、章动**（以 J2000.0 平赤道为中间架），需先从 ICRS 到 J2000 平赤道，即施 **Frame Bias 的逆 B^T**（见 [2-reference-frames.md](2-reference-frames.md)）；再 **岁差 P** → **章动 N** → 光行差 → 视位置。也可采用 **IAU 2000** 的 GCRS → 真 of date 的联合岁差章动矩阵，不再显式经过 J2000 平赤道。
- **特点**：几何在 ICRS 侧；与河外源一致，长期不随岁差漂移。IERS、DE406、SOFA 等采用；本程序太阳链在岁差前用 **R = P·B^T**，等价于先 B^T 到 J2000 平赤道再 P。

**对照小结**

| 路径 | 历表/几何架 | 到「平 of date」前 | 岁差 | 章动 | 光行差 |
|------|-------------|--------------------|------|------|--------|
| **FK5 时代** | J2000.0 平赤道/平黄道（FK5） | 无架变换 | P（L77 / P03） | N（IAU 1980 / 2000A） | 周年 |
| **现代 ICRS→GCRS** | ICRS 或 GCRS（如 DE406、patch 后 J2000_EQU） | B^T：ICRS→J2000 平赤道（或 IAU 2000 联合矩阵） | P（P03 等） | N（IAU 2000A） | 周年 |

本程序实现已统一为**管线架构**：太阳与月球均经 EphemerisProvider →（可选）LightTimeCorrector → TransformGraph / FrameMapper（太阳）→ ApparentEcliptic → λ；见 §7.2。

## 7.2 本程序视黄经 pipeline 概览（Rust 管线架构）

视黄经定义不变：**λ = atan2(y, x)** 在瞬时真黄道 of date，由几何位置(tr) 经岁差、章动得到（光行差已含于 Xproper(tr) 约定，见第 6 章）。实现上采用**单向数据流管线**（历元/坐标表示/参考架三正交，见第 2 章），替代原「胖函数」手写架链。设计上：EphemerisProvider 返回本原 6D 状态；FrameMapper 做拟合并改 Frame 标签；LightTimeCorrector 做光行时迭代；TransformGraph 做纯旋转与图路由；OpticalCorrector 为同架光行差/折射（占位）。业务链可拼接：compute_state → apply_mapping → apply_light_time → transform_to → into_representation。

### 管线组件（`rust/core/src/astronomy/pipeline/`）

| 组件 | 职责 | 本程序实现 |
|------|------|------------|
| **EphemerisProvider** | 在给定时刻返回天体 6D 状态（位置+速度），架由实现约定 | **Vsop87**：Body::Sun → MeanEcliptic(J2000)；**Elpmpp02Data**：Body::Moon → MeanEcliptic(J2000) |
| **FrameMapper** | 跨架非线性映射（含拟合/残差补丁），改变状态 Frame 标签 | **VsopToDe406IcrsFit**：FK5 赤道 → ICRS（FK5↔ICRS + DE406 太阳赤道 patch） |
| **LightTimeCorrector** | 光行时回溯：t → tr，并返回 tr 时刻的 6D 状态 | 持有 EphemerisProvider（及可选 FrameMapper），迭代 2 次得 tr 与 state |
| **TransformGraph** | 纯旋转架变换路由；按目标架施加岁差 P、章动 N^T、黄赤交角等 | MeanEcliptic↔FK5、ICRS↔FK5、FK5→MeanEquator(epoch)→TrueEquator(epoch)→ApparentEcliptic(epoch)；岁差可选 **P03** / **Vondrak2011** |

### 太阳视黄经与 ICRS 位置

- **太阳 ICRS 位置**（`sun_position_icrs`）：EphemerisProvider(Sun) → State6(MeanEcliptic) → TransformGraph.transform_to(FK5) → VsopToDe406IcrsFit.apply → 取 `state.position`（ICRS）。
- **太阳视黄经**（`sun_apparent_ecliptic_longitude*`）：LightTimeCorrector(无 mapper) → (tr, state_MeanEcliptic) → transform_to(FK5) → VsopToDe406IcrsFit.apply → transform_to(ApparentEcliptic(tr)) → **λ = state.to_spherical().lon_rad**（wrap [0, 2π)）。诊断量（Δψ、Δε、P 对角、ε_mean、ε_true）仍由同一历元 t_cent 计算，供比对。

### 月球视黄经

- **月球视黄经**（`moon_apparent_ecliptic_longitude*`）：可选光行时由 **LightTimeCorrector(Elpmpp02Data, mapper=None)** 得到 (tr, state)；否则直接 **Elpmpp02Data.compute_state(Body::Moon, t)**。随后 **TransformGraph.transform_to(ApparentEcliptic(epoch), jd)** → **λ = state.to_spherical().lon_rad**。月球不经过 FrameMapper（无 DE406 赤道 patch）。

### 对外 API 与选项

- 对外接口不变：`sun_position_icrs(vsop, t)`、`sun_apparent_ecliptic_longitude(vsop, t)`、`sun_apparent_ecliptic_longitude_with_options`、`sun_apparent_ecliptic_longitude_diagnostic`、`moon_apparent_ecliptic_longitude(elp, t)`、`moon_apparent_ecliptic_longitude_with_options(elp, t, options)`。
- **ApparentPipelineOptions**：`use_p03_precession`（定气用 P03）、`use_light_time_moon`（月球是否施光行时）。节气/定朔调用链见 [9-solar-terms.md](9-solar-terms.md)。

### 6×6 状态转移与路由

架变换时需同时变换位置与速度以保持运动学一致。**6×6 状态转移**：\[r_new; v_new\] = \[R R_dot; 0 R\] \[r_old; v_old\]，其中 R 为 3×3 旋转（岁差/章动等），R_dot 为旋转的时间导数（科里奥利项）；底层可分块乘法以利缓存与 SIMD。**路由**：TransformGraph 注册有向边（如 MeanEquator(t)→TrueEquator(t)、FK5↔ICRS），按目标架寻径、逐段施加变换，避免所有变换都回退到 ICRS；边可带代价，以优先选用低成本路径（如 MeanEquator→ApparentEcliptic 仅需黄赤交角旋转）。

### 管线步骤的粒度：谁做哪一步、不多合一

图中一条「边」只对应**一个组件的一步**，不把其他组件或多步合成一条标签。

- **光行时（t → tr）**：仅由 **LightTimeCorrector** 完成；太阳路径上 `mapper` 为 **None**，不含赤道拟合。边「VSOP87 → MeanEcliptic(epoch)」标签为「光行时→tr；历表输出 MeanEcliptic」。
- **赤道拟合**：仅由 **FrameMapper（VsopToDe406IcrsFit）** 完成，在 `transform_to(FK5)` 之后显式调用；边「FK5 → VsopToDe406IcrsFit」为「Frame bias B + DE406 拟合修正」，「VsopToDe406IcrsFit → ICRS」为恒等（架已为 ICRS）。
- **TransformGraph** 只做纯旋转（岁差、章动、黄赤交角、Frame bias B^T）；每条边对应一次 `get_transition` 的矩阵，无多合一。
- **月球**：边「ELPMPP02 → ELPMPP02_MEAN_LUNAR」仅表历表求值（含 DE405/Table6 修正）→ 月心平架；「ELPMPP02_MEAN_LUNAR → MeanEcliptic(epoch)」仅表 **Laskar P,Q 旋转**（Table6 在求值步，Table7 为 J2000→ICRS 的另一步，不写在此边）。

## 7.3 视位置计算中的其它效应：光行差分类、视差、自行

本节讨论在**视位置**中可能涉及的、超出 [6-light-time-and-aberration.md](6-light-time-and-aberration.md) 中「光行时 + 周年光行差」的效应：光行差的其它分类（周日光行差、长期光行差）、视差、自行；并说明它们与光行差/光行时的区别及在本程序中的取舍。

### 光行差的分类：周年、周日光行差与长期光行差

光行差均来自**观测者速度**与光速有限。按观测者运动的来源可分为：

| 名称 | 成因 | 量级（约） | 说明 |
|------|------|------------|------|
| **周年光行差** | 地球绕日公转 | 最大约 **20.5″** | 本程序实现；见第 6 章。 |
| **周日光行差** | 地球自转（测站相对地心） | 约 **0″.3** 量级 | 测站线速度远小于公转；高精度测站位置时才单独考虑；本程序不实现。 |
| **长期光行差** | 太阳系质心在银河系中运动 | 常数偏移，约 **数角秒** 量级 | 太阳绕银心约 220–250 km/s；相对河外源为近似常值，常归入参考架；历法/节气中可忽略；本程序不实现。 |

- **太阳系自行 / 银河系自行**：指太阳（或质心）在银河系中的运动。该运动造成的视方向偏移即**长期光行差**，属于**光行差**，不是视差。若参考架已相对河外源定义（如 ICRS），长期光行差可体现为参考架与「银心静止系」之间的系统差。
- **行星自行**：行星在天球上的角位置变化主要来自**轨道运动**及光行时/光行差改正，一般不称「行星自行」；**自行（proper motion）** 多指**恒星**在天球上的真实角速度 μ（mas/yr）。

### 视差、自行与光行差的区别

三者都影响「天体视位置」，但**物理原因与名称**不同：

| 效应 | 英文 | 物理原因 | 谁在动 | 典型量级 / 用途 |
|------|------|----------|--------|------------------|
| **光行差** | aberration | 光速有限 + **观测者速度** | 观测者（或观测者系） | 周年最大 20.5″；长期光行差数角秒。 |
| **视差** | parallax | **观测者位置**改变导致视线方向改变 | 观测者位置（如地球公转使基线变化） | 周年视差 π：用于恒星测距，π(″)≈1/距离(秒差距)；近星可达 1″ 量级。 |
| **自行** | proper motion | **天体本身**在天球上的真实角运动 | 天体（如恒星本动） | μ (mas/yr)；与观测者无关，长期累积。 |

- **视差**：从**不同位置**看，方向不同；**周年视差**是观测者位置周年变化导致恒星视位置的周期变化。光行差是**速度**带来的偏转，视差是**位置**带来的角度差；视差用于测距，光行差不依赖天体距离。
- **自行**：天体在参考架中的**真实角速度**，既不是光行差也不是视差。太阳系质心在银河系中的运动若说成「太阳系自行」，其**视方向效应**仍归为**长期光行差**。

**小结**：观测者（或观测者系）**速度**+ 光速有限 → **光行差**（周年、周日光行差、长期光行差）。观测者**位置**变化 → **视差**。天体**真实角运动** → **自行**。本程序只做太阳、月球视位置，实现**光行时 + 周年光行差**；周日光行差、长期光行差与恒星视差、自行不实现。
