# 2. 参考系与坐标

本章先约定**数学与物理量**，再说明**参考系、参考架、坐标系**的含义与关系，天球参考从**各国为政、FK4/FK5 到 ICRS** 的发展历程，以及**动力学坐标、平/真赤道**等概念；最后给出本程序中的架与几何链。

---

## 数学与物理量

本节约定后文与本程序使用的**数学对象**与**物理量及单位**，不展开推导；实现见 Rust `core/src/math`。

**角度与弧度**：内部计算与公式采用**弧度**（周角 = 2π rad）。角秒/角分/度换算 1″ = π/(180×3600) rad；岁差、章动、光行差等以角秒给出时需先转弧度。黄经、赤经由 atan2 得到，必要时 2π 归化。

**坐标与向量**：在某一参考架下用**直角坐标 (x,y,z)**，单位右手系；单位向量表示方向。赤道/黄道**球面坐标** (α,δ)、(λ,β) 与直角坐标为标准转换；λ = atan2(y,x) 在对应黄道架下。岁差 P、章动 N、Frame Bias B 为 **3×3 正交矩阵**，连续变换为矩阵乘法（如 v_true = N P v）。

**四元数与对偶四元数**：**四元数** q = 标量 + 向量，单位四元数表示**纯旋转**，v' = q v q 共轭（Grossman 积）。**对偶四元数** dq = q_real + ε·q_dual（ε²=0）表示**刚体变换**（旋转+平移）；连续刚体步 = 对偶四元数连乘并归一化。本项目中架变换（FK5→ICRS、黄道→赤道）、岁差/章动/光行差在 Rust 侧采用 pipeline 的 TransformGraph（6D 状态与 3×3 旋转矩阵），见 [7-apparent-longitude-and-syzygy.md](7-apparent-longitude-and-syzygy.md) §7.2。

**物理量与单位**：**时间**用儒略日 JD、TDB、TT，单位日（d）；见 [1-time-and-timescales.md](1-time-and-timescales.md)。**长度**历表与地心距常用 **AU**；**速度**与光速 **c** 同单位，光行差中 v/c 无量纲（周年约 10⁻⁴，约 20.5″）。岁差/章动/历表级数中的**儒略世纪、儒略千年**为**无量纲**归一化坐标 t（见第 1 章「儒略世纪与儒略千年」），非带量纲时长。程序中该参数使用类型 `NormalizedTime.JulianCentury`（由 `NormalizedTime.julianCentury(jd)` 得到），参与级数时用 `.value` 取数值。公式与代码中角度→弧度、时间→日或秒、距离→AU 或 m 需一致。

**速度与速率、加速度的区分**（实现见 `lunar.quantity`）：

- **速度（velocity）**：向量，有方向；类型为 `kinematics.Velocity`，绑定坐标系，分量为 L/T 量纲。
- **速率（speed）**：标量，即速度的大小或某方向分量；类型为 `dimension.Speed`，量纲 L/T。例如球面速度的径向率、直角分量均用 `Speed`。
- **加速度**：物理上为向量；程序中 **加速度向量** 为 `kinematics.Acceleration`（绑定坐标系），**加速度标量**（分量或径向等）为 `dimension.Acceleration`，量纲 L/T²。

**物理量运算约定**（实现见 `lunar.quantity.dimension`）：

- **有量纲量 op Decimal**：参与运算的裸 `Decimal` **视为无量纲**（如缩放因子、个数）；运算后**量纲不变**。例：`Length * Decimal` → `Length`，`Duration * Decimal` → `Duration`。
- **运算结果为无量纲时**：两有量纲量运算若结果量纲为 1（如 Length÷Length、角度÷角度），可采用 **显式 cast**：结果为 `Quantity(DIMENSIONLESS)` 或包装类型，通过 `.value` / `.toDecimal` 取数值；或 **自动 cast**：API 直接返回 `Decimal`。本程序采用**显式**：无量纲结果仍为带量纲类型，需用时再取 `.value`，避免误把有量纲当纯数。
- **常量带量纲**：天文常量用物理量类型表达（见 `lunar.astronomy.Constant`）：如 `AU: Length`、`c: Speed`、`cAuPerDay: Speed`；无量纲量（J2000 儒略日、质量比 M/M☉）仍为 `Decimal`。光行时用 `Length.div(Speed)→Duration`，兼容 `lightTimeDays(米)→日`。
- **岁差**：IAU2000/P03/L77/B03/Vondrak2011 的岁差角（ζ、z、θ、ε 等）为 `PlaneAngle`（弧度），角速率（ζ̇、ε̇ 等）为 `AngularRate`（rad/世纪，`Units.angularVelocity.rad_cy`）。矩阵构造与公式中用 `.rad`/`.value` 取数值参与旋转。

**矢量必须携带参考系**（实现见 `lunar.quantity.spatial`、`lunar.quantity.kinematics`）：

- 矢量必有基；本程序中**所有有物理意义的直角矢量均绑定** `CoordinateSystem`，不提供「类型不携带系」的 Vec3 类型。
- **Position / Displacement / Velocity**：类型携带 `CoordinateSystem`，分量用 `Quantity`，支持 `transformTo`、球面表示等。中间系（如 ELPMPP02 的 Laskar 直角系）也使用同一类型并赋予专用 frame（如 `Frames.ELPMPP02_LASKAR_CARTESIAN`），用 `Displacement.transformBy(P, target)`、`Velocity.transformBy6(V, pos, target)` 做矩阵变换到目标系。

---

## 参考系、参考架与坐标系

- **参考系（reference system）**：规定坐标**如何定义**的理论约定——原点、轴向的选取规则、与物理量（如岁差、章动）的关系。例如「以太阳系质心为原点、轴向由某历元平赤道与春分点规定」是一种参考系。
- **参考架（reference frame）**：参考系的**具体实现**，即通过实测（恒星、类星体、卫星等）给出的一组基准方向或基准点，使理论参考系可在操作上复现。例如 ICRS 的参考架是 ICRF（由 VLBI 观测的河外射电源实现）。
- **坐标系（coordinate system）**：在某一参考系或参考架下，用一组坐标（如赤经赤纬、黄经黄纬、直角坐标 x,y,z）表示点的数学框架。同一参考架下可有多种坐标系（赤道、黄道、银道等），彼此由固定变换联系。
- **关系小结**：参考系 = 概念与规则；参考架 = 用数据实现的参考系；坐标系 = 在该参考系/架下选定的参数化方式。日常常说「在 ICRS 下算坐标」即：在 ICRS 参考系（由 ICRF 实现）下，选用某种坐标系（如赤道直角坐标或赤经赤纬）表达位置。

**管线模型中的三正交维度与设计原则**（视位置管线见第 7 章）：(1) **历元与时间**：动态架绑定历元时用 JD 与 TimeScale 标识（TT/UTC/TDB 区分见第 1 章）。(2) **坐标表示法**：纯数学几何——Cartesian 直角、Spherical 球面（经度/纬度/距离），与参考架正交。(3) **参考架**：绝对基准（ICRS、FK5）；动态绑定（MeanEquator(epoch)、TrueEquator(epoch)、ApparentEcliptic(epoch)）强制持有历元。设计上物理与数学正交、类型安全、6D 状态流转（位置+速度一并变换）。

---

## 天球参考的发展历程

### 各国为政、区域星表时代

- 二十世纪前半叶，国际上**没有统一的天球参考**：各国或地区用各自星表与观测网建立赤道、黄道等，历元与岁差常数不一，彼此转换需经验公式或逐星比对。例如：德国 **AGK** 系列（AGK1/2/3）、美国 **N30**、**GC**（General Catalogue）、**Yale** 星表等，分别基于不同观测与岁差系统，跨星表使用时必须注意系统差。
- **基本星表**的雏形即在此背景下出现：用少量高精度恒星作为「骨架」，其它星表通过与其比对纳入同一系统。这一阶段的「惯性」主要靠恒星自行与岁差模型近似，精度与覆盖范围有限。

### FK4 与 B1950.0 时代

- **FK4（第四基本星表）**（1963 年）：由 IAU 采纳的**国际基本星表**，约 1 535 颗恒星，历元 **B1950.0**（1950 年 1 月 1 日 0 时 ET 的平赤道与平春分点）。FK4 定义了当时国际上相对统一的**动力学赤道与春分点**：赤道面与春分点由岁差理论（如 IAU 1976）与 FK4 星位置共同约束。
- **动力学赤道/春分点**：由**行星与月球历表**（如 DE200）与岁差模型给出的「平赤道」「平春分点」，与纯恒星实现的赤道可能差数十毫角秒（系统差）。FK4 与 DE200 等历表采用同一套动力学约定，形成 FK4/DE200 时代的标准架。

### FK5 与 J2000.0 时代

- **FK5（第五基本星表）**（1988 年）：沿用约 1 535 颗星，历元改为 **J2000.0**（2000 年 1 月 1 日 12 时 TT 的平赤道与平春分点）；扩展版 FK5 Ext 增加约 3 117 颗星。岁差等常数更新，与 **IAU 1976 岁差**、**JPL DE200/DE405** 等历表配套，成为 1980–1990 年代的标准。
- **VSOP87**、**DE200** 等历表的时间与空间参考即 **J2000.0 平黄道/平赤道**，与 FK5 的 J2000.0 架一致（在约定精度内）。

### ICRS / ICRF 时代（1998 年起）

- **ICRS（国际天球参考系）**（IAU 1991 决议，1998 年 1 月 1 日起正式取代 FK5）：**原点**为太阳系质心；**轴向**由**河外射电源**（类星体等）通过 VLBI 确定，在太空中无长期旋转，与地球岁差/章动无关，是**动力学惯性**的更好近似。
- **ICRF（国际天球参考架）**：ICRS 的物理实现，已历多代——**ICRF1**（1998）、**ICRF2**（2009）、**ICRF3**（2019 年起），定义源与精度逐步提升。光学波段常用 **Hipparcos**、**Gaia** 等与 ICRF 对齐实现光学参考架。
- **GCRS（地心天球参考系）**：原点为**地球质心**，轴向与 ICRS 平行（无旋转），即「地心版的 ICRS」。历表与观测常用 GCRS 表示地心位置；与 ICRS 仅差原点平移，不差旋转。
- **J2000.0 平赤道架与 ICRS**：在 J2000.0 历元，IAU 约定 ICRS 与「J2000.0 平赤道与平春分点」仅差一个**固定小旋转**（约 20–50 mas），即 **Frame Bias**（见下节）。因此从 FK5(J2000) 到 ICRS 的变换可表为单一矩阵 B。

---

## 动力学坐标、平赤道与真赤道

- **动力学赤道 / 动力学春分点**：由**力学历表**（行星、月球）与**岁差模型**定义的赤道面与春分点方向，不直接依赖恒星观测。历表（VSOP87、ELP、DE）给出的直角坐标或轨道根数通常即在此类「动力学架」下。
- **平赤道 / 平春分点（of date）**：某历元 t 的赤道面与春分点，只含**岁差**，不含章动；随 t 缓慢变化。J2000.0 平赤道即 t = J2000.0 时的平赤道。
- **真赤道 / 真春分点（of date）**：在平赤道基础上加上**章动**，得到该时刻 t 的瞬时赤道与春分点；用于「真赤道坐标」「视位置」等。从平到真的变换为章动矩阵 N。
- **黄道**：可类似区分为某历元的**平黄道**（仅岁差）与**真黄道**（岁差 + 章动）。视黄经 λ 通常在**真黄道**下定义；历表输出若为「J2000 黄道」则多为**平黄道**。
- **本程序中的链**：历表 → 平架（J2000 赤道或黄道）→ 岁差 P → 章动 N → 真赤道/真黄道 → 取 λ 或 (α,δ)。

---

## Frame Bias（FK5 → ICRS）

**定义与量级**  
J2000.0 平动力学架（与 FK5/DE200 一致）与 ICRS 之间差一个**固定旋转**，称为 Frame Bias。记 **x_J2000 = B · x_ICRS**，B 为从 ICRS（或 GCRS）到 J2000.0 平赤道架的旋转矩阵；反之 **x_ICRS = B^T · x_J2000**。量级约数十毫角秒（赤道极约 17 mas、5 mas 两方向，春分点方向约 78 mas）。数值见 Hilton & Hohenkerk, A&A 413, 765 (2004)，SOFA 函数 `iauFk5hip`。本实现：`Fk5Icrs.rotationMatrix`；太阳几何链中 FK5 赤道后施 **B** 到 ICRS，岁差前用 **B^T**（J2000_EQU → MEAN 时 R = P·B^T）。

**两套架的由来**  
FK5/J2000 架由**恒星位置**（基本星表 FK5）与**动力学春分点**（IAU 1976 岁差、历表与星位置拟合）实现；ICRS 由**河外射电源**（VLBI）实现，轴向在空间固定、不依赖春分点或岁差。两套**独立实现**，物理基准不同（光学恒星+动力学 vs 射电河外源），在天球上的轴向本就不是同一组方向，系统差在 J2000.0 处表现为上述固定小旋转（与岁差随时间累积无关）。

**历史与成因**  
IAU 1991 年建议（及 1997 年采纳 ICRS 的决议）要求新参考系的主平面**尽量接近 J2000.0 平赤道**、原点**尽量接近 J2000.0 动力学春分点**，即**设计意图是与 J2000.0 对齐**。实际实现 ICRF 后，与 J2000.0 平赤道/春分点比对发现存在**数十毫角秒的固定偏差**，需用旋转矩阵 B 做高精度换算。文献普遍将残差归因于**测量与链接限制**：FK5 极位置与赤经原点的不确定度较大（量级约 ±50 mas、±100 mas）；VLBI 河外源与光学 FK5 属不同源类，跨波段链接精度有限；早期 ICRF 定轴与动力学春分点的链接也有不确定性。因此「原设计为对齐，实现时受技术所限出现残差、该残差被采纳为 Frame Bias」有据可查（Hilton & Hohenkerk；IERS 等）。

**为何残差保留下来**  
技术上可对 ICRF 做**一次**固定旋转，使在 J2000.0 与 FK5 完全重合，从而消掉 B（两架在空间均固定，对准一次即永远对准）。但那样做等于在定义上令 ICRS = FK5，河外源架会退成「需旋转后才进 ICRS」的次要角色，与 IAU 以河外源架为国际天球参考的取向相反。因此选择**保持 ICRS = ICRF**（不旋转 ICRF 去消除残差），由旧历表与旧星表（FK5、DE、VSOP87）通过 B 迁就新标准；B 的「方便」只对旧数据成立，长期则以河外源架为主标准。

**小结**  
设计是与 J2000.0 对齐，测量与链接限制导致实现后存在数十 mas 残差；该残差被正式采纳为 Frame Bias（矩阵 B），且未通过旋转 ICRF 消除。当前标准做法是接受 B，用于 FK5↔ICRS 的换算。

---

## 历表对齐到 ICRS 与地心坐标

**对齐到 DE406 即 ICRS**  
DE406（及整个 JPL DE400 系列历表）的官方参考架即为 **ICRS**。VSOP87 最初基于动力学 J2000 平黄道与 FK5 赤道构建，与 ICRS 存在约数十毫角秒的 **Frame Bias**。本程序中「Vsop→DE406 拟合修正」或「Frame bias 逆」的本质，是应用该微小旋转矩阵，将 FK5 的**坐标轴方向**对齐到 ICRS。ELPMPP02 经 Table 7 等到 DE406/ICRS 后，与 VSOP87+patch 在架意上一致。因此管线中的 **ICRS** 节点统一的是坐标轴的**方向**。

**岁差→章动之后为地心坐标**  
岁差与章动描述的是**地球自转轴**的物理摆动，「历元平赤道」「历元真赤道」「历元视黄道」等概念均以**地球质心**为原点才有意义。因此，从「FK5 J2000（已修正）」开始经岁差 P、章动 N 直至真赤道/视黄道的整条流水线，其**原点已统一为地心**，输出为**地心视坐标**（geocentric apparent）。数据源层面：**ELPMPP02（月球）** 理论本身给出即为地心坐标；**VSOP87（行星/太阳）** 原始为日心或质心坐标，在进入上述岁差→章动通用流水线之前，必须先做一次**向量平移**（目标天体质心坐标 − 地球质心坐标），转换为地心坐标。该地心视坐标是后续转换为站心（ENU）等的直接前置条件。

**代码中平移发生位置**：太阳的日心→地心在 `rust/core/src/astronomy/pipeline/ephemeris_provider.rs` 的 `EphemerisProvider for Vsop87` 中完成——VSOP87 提供的是地球的日心位置，对 Body::Sun 请求时用「地心太阳 = − 地心」得到地心太阳状态后再进入架变换与岁差/章动管线。月球由 ELPMPP02 提供，其理论直接输出地心月，无需平移。

---

## 几何与架（本程序）

- **几何（geometry）**：历表或理论给出的**位置与速度**，经**光行时**与**变换到共同参考架**后，得到 pipeline 使用的向量；输出一般为 GCRS/ICRS 或 J2000 赤道（太阳路径含 Vsop87De406IcrsPatchMixedTerms）。
- **架（frame）**：从「岁差前」的赤道到**视黄经 λ** 的变换链：岁差 P → 章动 N →（可选）光行差 → 真/视黄道取 λ。换天体 = 换整块「几何」（历表 + 到 J2000_EQU 的链），岁差/章动/光行差共用同一套。
- **坐标系**：中间步骤可用赤道 (x,y,z) 或黄道 (x,y,z)；最终取 λ = atan2(y,x) 等在真黄道下的角度，均为同一参考架下的不同坐标表示。

---

## 星表与标架（太阳、月球）

- **太阳**：VSOP87 为 **DE200/FK5** 系（J2000 平黄道）；几何链为 黄道 → 赤道(FK5) → **B (FK5→ICRS)** → Vsop87De406IcrsPatchMixedTerms → J2000_EQU。
- **月球**：ELPMPP02 输出 **J2000 平黄道**；经 Table 7 到 ICRS（J2000_EQU）。

**参考文献**：IERS Conventions；IAU SOFA；`doc/references/sunMoon_simp.pdf`。

---

## 实现状态：裸 Decimal / 裸 Vec\[3] 的消除

目标：尽量用**有量纲/有物理含义的类型**（如 `PlaneAngle`、`Length`、`JulianCentury`、`Position`/`Velocity`）替代裸 `Decimal` 和裸 `Vec[3]`。从星表到岁差章动、光行时/视差、视位置，当前状态如下。

| 环节 | 已改 | 未改 / 部分改 |
|------|------|----------------|
| **星表 / 坐标点** | 输出用 `Position`（带 frame）；**ToPosition** 已增加 `PlaneAngle`/`Length` 重载，**AstroPoint.eclipticToPosition** 等内部改为调用物理量重载。 | 点类型（`EclipticPoint` 等）字段仍为 **Decimal**，仅在与 ToPosition 边界处用 `PlaneAngle`/`Length`。 |
| **岁差 / 章动** | 时间参数 `NormalizedTime.JulianCentury`；岁差角 `PlaneAngle`、角速率 `AngularRate`；历表输出带系。 | 岁差极向量、矩阵乘向量等仍用 **Vec\[3]**（如 `Vondrak2011.eclipticPole`、`precessionMatrixDerivativeTimesVector(r: Vec[3], …)`）；矩阵/级数内部用 `Decimal` 属实现细节。 |
| **光行时** | `lightTime(distance: Length): Duration`；**retardedTime** / **retardedTimeFast** 已改为 **getDistance: TimePoint => Length**、**initialDistance: Option[Length]**；`withLightTime` 内部用 `position.norm`（Length）构造 getDistance。 | `lightTimeDays(Decimal)` 保留为兼容 API。 |
| **光行差** | **Aberration.applyAberration(r: Position, v: Velocity): Position** 已增加；管线中光行差步改为用 `Position`/`Velocity` 调用，内部仍用 Vec\[3] 实现公式。 | `annualAberrationDirection`、`applyAberrationWithDualQuaternion` 等仍为 Vec\[3] 入参/返回值（供内部与导数用）。 |
| **视位置管线** | 入口/出口为 **Position / Velocity**（带 frame）；历表（如 ELPMPP02）用 **Displacement/Velocity** + Frame；常量 `AU`、`c` 等带量纲。 | 中间步骤大量「Position → 取 .value 拼 **Vec\[3]** → 矩阵/旋转 → 再包回 Position」；**FrameTransform** 接口仍为 `position(p: Vec[3]): Vec[3]`，实现层用 Vec\[3] 与四元数/矩阵衔接。 |

**小结**：光行时接口、光行差对外 API、星表→Position 的边界已改为物理量；岁差极向量与 FrameTransform 仍以 Vec\[3] 为主，后续可再收口。
