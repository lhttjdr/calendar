# 视位置

历表给出的是天体的**几何位置**——某时刻它在哪里。我们看见的方向与之不同，节气、合朔要的是**视位置**。二者之间隔着两个与光速有限有关的效应：光行时与光行差。

## 源流

**光速有限。** 古人多以为光的传播是瞬时的。1676 年罗默（Ole Rømer）发现木卫一的食在木星远离地球时推迟、靠近时提前，推断光走过地球轨道直径要二十余分钟，这是光速有限的第一个证据，也是"光行时"的由来。

**光行差。** 布拉德雷本想测量恒星的周年视差，1725—1728 年观测天龙座 γ 星，发现它确有周年的位移，但方向与视差应有的方向差了 90°：位移最大时，不是地球离该星最远或最近之时，而是地球运动方向与星的方向垂直之时。他认识到这是地球运动速度与光速合成的结果，1729 年发表，由此测得光速，并给出地球公转的第一个直接证据。光行差常数约 20″，比当时任何恒星的视差都大得多；恒星视差直到 1838 年才由贝塞尔测出（天鹅座 61，约 0.3″）。

**相对论。** 1905 年爱因斯坦的狭义相对论给出光行差的严格公式；广义相对论又预言光线经过太阳附近会偏折，1919 年日全食时由爱丁顿等观测证实。1964 年夏皮罗（I. Shapiro）指出雷达回波经过太阳附近会有额外延迟。今天的视位置计算（IERS 规范、SOFA）把光行时、光行差、引力偏折统一在广义相对论的框架里处理，但对太阳和月亮，到 $v/c$ 一阶已足够。

中国古代历法没有光行时与光行差的概念；它们都在 20″ 量级，远在古代仪器的分辨率以下。

## 光行时

我们在时刻 $t$ 收到的光，是天体在更早的**推迟时** $t_r$ 发出的。设天体与观测者（地心）在质心系中的世界线为 $\mathbf x(t)$、$\mathbf x_E(t)$，光沿直线传播：

$$
t - t_r = \frac{\lvert \mathbf x(t_r) - \mathbf x_E(t)\rvert}{c}
$$

$t_r$ 出现在等式两侧，是隐式方程，需迭代：

$$
t_r^{(0)} = t - \frac{\lvert \mathbf x(t) - \mathbf x_E(t)\rvert}{c},\qquad
t_r^{(k+1)} = t - \frac{\lvert \mathbf x(t_r^{(k)}) - \mathbf x_E(t_r^{(k)})\rvert}{c}
$$

太阳系内收敛很快，本项目迭代 2 次。太阳 $\tau = t - t_r \approx 499$ s（8 分 19 秒），月球约 1.3 s。迭代式中用 $\mathbf x_E(t_r)$ 代替 $\mathbf x_E(t)$，与严格式差 $\sim v_E (t - t_r)/c$，可忽略。

在地心看来，太阳在这 499 秒里沿黄道移动约 20.5″；月亮 1.3 秒移动约 0.7″。

## 光行差

观测者以速度 $\mathbf v$ 运动时，看到的光的来向朝运动方向倾斜。设光在质心系中的方向为单位向量 $\mathbf e$，到 $v/c$ 一阶：

$$
\mathbf e' = \frac{\mathbf e + \mathbf v/c}{\lVert \mathbf e + \mathbf v/c \rVert} \approx \mathbf e + \frac{1}{c}\left[\mathbf v - (\mathbf e\cdot\mathbf v)\,\mathbf e\right]
$$

偏转角在 $\mathbf e \perp \mathbf v$ 时最大，为 $v/c$。地球公转 $v \approx 29.8$ km/s，$v/c \approx 10^{-4}$ rad ≈ 20.5″，即**周年光行差常数**。

| 名称 | 成因 | 量级 | 本项目 |
|---|---|---|---|
| 周年光行差 | 地球绕日公转 | 最大 20.5″ | 节气、合朔计入 |
| 周日光行差 | 地球自转（测站相对地心，赤道约 460 m/s） | 最大 0.32″ | 只在站心计算（日月食、星图）中计入 |
| 长期光行差 | 太阳系质心绕银心运动（约 220—250 km/s） | 近乎常值的数角秒 | 归入参考架，不计 |

光行差与另两个效应不同：**视差**来自观测者**位置**的改变（周年视差用于恒星测距，$\pi('') \approx 1/d(\mathrm{pc})$）；**自行**是天体本身在天球上的角运动。光行差来自观测者的**速度**，与天体距离无关。

## 两条路线与 Xproper 折叠法

**分别计算**（天体测量位置 + 光行差）：

1. 光行时：$\mathbf X_r = \mathbf x(t_r) - \mathbf x_E(t)$，地球取**接收时刻** $t$；
2. 再对其方向施加周年光行差。

**折叠法**（行星光行差）：视方向 $\propto \mathbf X_r/\lVert\mathbf X_r\rVert + \mathbf v_E/c$。把光行距离 $\Delta t = \lVert\mathbf X_r\rVert/c$ 乘进第二项，

$$
\mathbf X_r + \mathbf v_E\,\Delta t = \mathbf x(t_r) - \left[\mathbf x_E(t) - \mathbf v_E\,\Delta t\right] \approx \mathbf x(t_r) - \mathbf x_E(t_r) \equiv \mathbf X_{\text{proper}}
$$

即在 $v/c$ 一阶下，天体与地球**同取发光时刻** $t_r$ 的位置相减，就一并包含了光行时与光行差。

本项目默认用折叠法：先求出 $t_r$，再在同一 $t_r$ 上求地心太阳、地心月亮，后面**不再**施加光行差。两条路线**不能混用**：$\mathbf x(t_r) - \mathbf x_E(t)$ 之后必须再加光行差；$\mathbf x(t_r) - \mathbf x_E(t_r)$ 之后绝不能再加，否则周年光行差被计入两次，节气时刻差出约 8 分钟。本项目同时实现了显式光行差路线作为对照，二者在 $v/c$ 一阶一致，由单元测试保证。

### 为什么光行差可以"提前"算

直觉上应先求几何位置，经岁差、章动转到观测者的架，**最后**再施光行差；折叠法却在 ICRS 中就把光行差算进了向量。二者等价，因为一阶下行星光行差是三维向量的**加法**（几何向量 + $\mathbf v_E\Delta t$），而岁差、章动是线性的**旋转**。记总旋转为 $M$：

$$
M\left(\mathbf X_r + \mathbf v_E\Delta t\right) = M\,\mathbf X_r + M\,\mathbf v_E\Delta t
$$

先加再转与先转再加得到同一方向。常说"光行差必须在岁差章动之后算"，指的是**恒星**光行差：恒星只有方向单位向量，修正时要在单位向量上叠加速度再归一化，归一化是非线性的，顺序才有关系。太阳、月亮有完整的三维距离向量，修正是纯向量相加，顺序可交换。

## 视黄经的完整链条

1. 给定观测时刻 $t$（TT），迭代求 $t_r$；
2. 在 $t_r$ 求地心太阳（VSOP87B 地球日心位置加改正，取反）或地心月亮（ELP/MPP02），J2000.0 平黄道，6 维状态；
3. $R_x(\varepsilon_0)$ 转到 FK5 赤道，$B$ 转到 ICRS；
4. 岁差（从 ICRS 接经典岁差时 $R = P\,B^{\mathsf T}$）到当日平赤道；
5. 章动到当日真赤道；
6. $R_1(\varepsilon_A + \Delta\varepsilon)$ 到当日真黄道，$\lambda = \operatorname{atan2}(y, x)$，归化到 $[0, 2\pi)$。

整条链的时间自变量是 $t_r$。月亮不经过参考架偏差那一步（ELP/MPP02 直接由 J2000.0 平黄道接岁差）。

每一步变换的都是位置与速度：

$$
\begin{pmatrix}\mathbf r' \\ \mathbf v'\end{pmatrix} = \begin{pmatrix} R & 0 \\ \dot R & R \end{pmatrix}\begin{pmatrix}\mathbf r \\ \mathbf v\end{pmatrix}
$$

$\dot R$ 是转动矩阵对时间的导数。速度用于求根时的解析导数 $\mathrm d\lambda/\mathrm dt$。

**两条历史路径**：FK5 时代的历表直接在 J2000.0 平架里，链为"J2000 平 → 岁差 → 章动 → 光行差"，无架变换；现代历表在 ICRS/GCRS 中，接经典岁差前先扣参考架偏差 $B^{\mathsf T}$，或改用 IAU 2000 的 GCRS → 真赤道联合矩阵。本项目两者兼有：VSOP87 经 FK5 → ICRS 再扣回，DE 历表直接从 ICRS 进入。

一个例子：本项目求得 2026 年春分（太阳视黄经恰为 0°）的时刻。在这一刻，在同一推迟时刻 $t_r$ 上量得的太阳真黄经（已计岁差、章动，未计光行差）为 $+20.58''$；光行差贡献 $-20.58''$，视黄经恰为 0。

## 误差预算

### 折叠法本身

相对严格的相对论 ICRS → GCRS 变换，折叠法的误差有三个来源：GCRS 与 BCRS 的度规修正 $(v_E/c)^2$；光行时迭代的相对误差 $\sim (v/c)^2$（月球在质心系中也以约 30 km/s 运动）；光行差只准到 $v/c$ 一阶。合计角度误差约 $(v_E/c)^2 \sim 10^{-8}$，即约 0.002″，对应节气约 0.05 秒、定朔约 0.004 秒。国家标准《农历的编算和颁行》（GB/T 33661—2017）要求 1 秒，远有余裕。

### 有意忽略的项

**相对论与时空效应**

- 光线引力偏折：对日、月视位置的修正极小，略去（行星远离太阳时为毫角秒级，近合日可达角秒级，行星计算尚未计入）。
- Shapiro 延迟：严格光行时含广义相对论时间修正，对太阳 $< 10^{-4}$ s，对月球约 $10^{-8}$ s。
- TT 与 TDB 之差：数千年内不超过约 2 ms，求时刻时视为等同。

**矩阵与求导的截断**

- 求月相时令 $\Delta\psi = 0$：章动对日、月加同一修正，相减抵消。
- 求 $\dot\lambda$ 时忽略坐标轴转动项（$\dot R$ 中岁差章动的部分）：相对误差太阳约 $4\times10^{-5}$、月球约 $3\times10^{-6}$，只影响牛顿迭代的步长，不影响收敛点。
- 推迟位置变化率的多普勒分母 $(1 + v_r/c)$：日月轨道接近圆，径向速度极小，略去。

**观测者效应**（节气、合朔按地心定义，不需要；站心计算要补回）

- 周日光行差：站心计算中已计入（一阶）。
- 极移：站心东北天坐标依赖地固架，高精度地平坐标需计入地轴相对地壳的摆动。

牛顿迭代的收敛只以角残差为准（如 $10^{-8}$ rad），不设提前退出，所以实际精度由上面这些模型误差决定，而不是由迭代决定。

## 参考文献

1. O. Rømer, "Démonstration touchant le mouvement de la lumière", *Journal des Sçavans* (1676), 233–236。未核原文。
2. J. Bradley, "A letter … to Dr. Edmond Halley … giving an account of a new discovered motion of the fix'd stars", *Philosophical Transactions of the Royal Society* 35（卷期标 1727—1728 年，1729 年刊出）, 636–660。据 archive.org 所存 JSTOR 早期期刊本核题目卷页。
3. F. W. Bessel, "Bestimmung der Entfernung des 61sten Sterns des Schwans", *Astronomische Nachrichten* 16 (1839), 65–96。未核原文。
4. I. I. Shapiro, "Fourth test of general relativity", *Physical Review Letters* 13 (1964), 789–791。未核原文。
5. G. H. Kaplan, *The IAU Resolutions on Astronomical Reference Systems, Time Scales, and Earth Rotation Models*, USNO Circular 179, 2005。
6. G. Petit, B. Luzum (eds.), *IERS Conventions (2010)*, IERS Technical Note 36；IAU SOFA。
7. 中华人民共和国国家标准 GB/T 33661—2017《农历的编算和颁行》。
8. 廖育栋：《月相和二十四节气的计算》（网络文稿，2018 年初稿，2025 年修订）。业余整理的计算说明，本章只用作对照；推导与公式以上列 IERS 规范、Kaplan 2005 为准。
