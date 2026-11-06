# 章动

**章动**（nutation）是叠加在岁差上的周期摆动：只计岁差得到的是**平**天极、平赤道、平春分点；再加章动，得到该时刻的**真**天极、真赤道、真春分点。最大振幅约 17″，主周期 18.6 年。

## 成因

岁差来自日月对地球赤道隆起的力矩。若日月轨道固定，这个力矩只是平均意义上的持续推力，天极平滑进动；但力矩随日、月的位置周期变化，天极便在平均进动之外来回摆动。

- **月球轨道交点**：月球轨道相对黄道倾斜约 5°，升交点沿黄道西退，18.6 年一周。交点在哪里，月球就在那附近穿越黄道，力矩的方向和大小随之变化——这是章动主项（18.6 年）的来源，振幅最大。
- **太阳的位置**：一年变化一周，力矩以半年为周期变化，产生半年项（Δψ 约 1.3″）。
- **月球在轨道上的位置**：产生半月项（约 0.2″）及更短周期的小项。

## 两个分量

- **黄经章动 $\Delta\psi$**：真春分点相对平春分点沿黄道的位移。主项振幅 17.206″。
- **交角章动 $\Delta\varepsilon$**：黄赤交角的周期变化，$\varepsilon_{\text{真}} = \varepsilon_A + \Delta\varepsilon$。主项振幅 9.205″。

在天极附近看，真天极绕平天极画出一个椭圆：沿岁差方向的半轴是 $17.2'' \sin\varepsilon \approx 6.9''$，背离黄极方向的半轴是 $9.2''$。按周期，章动项又分长周期（18.6 年、9.3 年、半年、一月等）与短周期（十几天、数天），分界在不同文献中约 34—90 天。

<Anim src="celestial-sphere/nutation" caption="左：月球轨道交点线 18.6 年逆行一周；右：真天极绕平天极的轨迹（单位角秒，Meeus 四项级数按真实时间推算）。末段叠加岁差，横纵同比例" />

<Chart src="nutation-series" />

18.6 年主项之外，可以看到半年项造成的锯齿，以及月周期的小起伏。

## 级数形式

每一项的幅角是五个**基本角**（Delaunay 幅角）的整数线性组合：

$$
\theta_i = n_{i1}\,l + n_{i2}\,l' + n_{i3}\,F + n_{i4}\,D + n_{i5}\,\Omega
$$

$$
\Delta\psi = \sum_i \left[(A_i + A_i'\,T)\sin\theta_i + A_i''\cos\theta_i\right],\qquad
\Delta\varepsilon = \sum_i \left[(B_i + B_i'\,T)\cos\theta_i + B_i''\sin\theta_i\right]
$$

基本角是月球、太阳平轨道根数的组合，$T$ 的多项式（角秒，IERS Conventions 2010 式 5.43）：

| 符号 | 含义 | 表达式 |
|---|---|---|
| $l$ | 月球平近点角 | $485868.249036 + 1717915923.2178\,T + 31.8792\,T^2 + 0.051635\,T^3 - 0.00024470\,T^4$ |
| $l'$ | 太阳平近点角 | $1287104.79305 + 129596581.0481\,T - 0.5532\,T^2 + 0.000136\,T^3 - 0.00001149\,T^4$ |
| $F$ | 月球平升交角距 | $335779.526232 + 1739527262.8478\,T - 12.7512\,T^2 - 0.001037\,T^3 + 0.00000417\,T^4$ |
| $D$ | 日月平距角 | $1072260.70369 + 1602961601.2090\,T - 6.3706\,T^2 + 0.006593\,T^3 - 0.00003169\,T^4$ |
| $\Omega$ | 月球平升交点黄经 | $450160.398036 - 6962890.5431\,T + 7.4722\,T^2 + 0.007702\,T^3 - 0.00005939\,T^4$ |

最大的几项（IAU 2000A，单位毫角秒）：

| $l$ | $l'$ | $F$ | $D$ | $\Omega$ | 周期 | $A$（Δψ 正弦） | $B$（Δε 余弦） |
|---|---|---|---|---|---|---|---|
| 0 | 0 | 0 | 0 | 1 | 6798.4 日（18.6 年） | −17206.4242 | 9205.2331 |
| 0 | 0 | 2 | −2 | 2 | 182.6 日 | −1317.0912 | 573.0336 |
| 0 | 0 | 2 | 0 | 2 | 13.7 日 | −227.6418 | 97.8461 |
| 0 | 0 | 0 | 0 | 2 | 3399.2 日（9.3 年） | 207.4554 | −89.7492 |
| 0 | 1 | 0 | 0 | 0 | 365.3 日 | 147.5877 | 7.3871 |

## 源流

**发现。** 章动是布拉德雷（James Bradley）发现的。他在 1727 年发现光行差之后，继续以天顶仪观测恒星，发现星的赤纬还有一种周期约 18.6 年的微小变化，与月球轨道交点的运动同周期。他观测了整整一个周期，到 1748 年才在致麦克尔斯菲尔德伯爵的信中发表。

**理论。** 次年（1749），达朗贝尔把日月对地球赤道隆起的力矩写成运动方程，同时解出了岁差与章动。欧拉进而指出，刚体地球除受迫章动外还应有一种自由摆动（周期约 305 日）；1891 年钱德勒（S. C. Chandler）在纬度变化中找到了它，周期却是约 433 日，差别来自地球并非刚体。从此章动理论与地球内部结构联系在一起。

**级数。** 20 世纪的章动级数一步步引入地球内部的物理：Woolard（1953）用刚体地球；Wahr（1981）加入弹性地幔与液态外核，成为 IAU 1980 章动的基础；甚长基线干涉测量（VLBI）把章动观测精度提高到 0.1 毫角秒量级，暴露出 IAU 1980 的毫角秒级误差，于是有 Mathews、Herring、Buffett（2002）拟合观测的 MHB2000，即 IAU 2000A。其中的"自由核章动"（液核相对地幔的摆动，周期约 430 日）的参数，反过来成为测量地核扁率的手段。

中国古代历法没有章动的概念：其振幅不到 20″，远在古代仪器的分辨率（最好约 5′）以下。

## 模型沿革

| 模型 | 说明 | 项数 | 精度 |
|---|---|---|---|
| Wahr 1981 | 弹性地球与流体地核的章动理论，为后续级数提供物理基础 | — | 理论框架 |
| IAU 1980 | 刚体地球加 Wahr 形变改正；与 IAU 1976 岁差配套，DE200/VSOP87 时代通用 | 106 | 约 1—5 mas |
| IAU 2000A（MHB2000） | Mathews、Herring、Buffett；考虑非刚体地球、液核、海洋等，拟合甚长基线干涉测量的章动观测；与 IAU 2006 岁差配套，IERS、SOFA 采用 | 日月项 678、行星项 687，合 1365 | 约 0.2 mas |
| IAU 2000B | 2000A 的简化版 | 77 | 约 1 mas |


## 章动矩阵

本文的 $R_i(\theta)$ 是坐标旋转（被动）矩阵，与 IERS Conventions 一致。从当日平赤道到当日真赤道：

$$
\mathbf v_{\text{真}} = N\,\mathbf v_{\text{平}},\qquad N = R_1(-\varepsilon_A - \Delta\varepsilon)\;R_3(-\Delta\psi)\;R_1(\varepsilon_A)
$$

从右往左读，三步各有几何含义：

1. $R_1(\varepsilon_A)$：绕平春分点方向转过平黄赤交角，平赤道坐标换成平黄道坐标（$z$ 轴从平天极转到黄极）。
2. $R_3(-\Delta\psi)$：在黄道面内绕黄极转过 $\Delta\psi$，$x$ 轴从平春分点移到真春分点——这就是黄经章动。
3. $R_1(-\varepsilon_A - \Delta\varepsilon)$：绕真春分点按真黄赤交角转回赤道，$z$ 轴从黄极转到真天极——交角章动在这一步。

这样 $\Delta\psi$、$\Delta\varepsilon$ 各自只出现在绕一根轴的旋转里，正对应二者的定义。（若用主动旋转矩阵书写，同一变换写作 $R_1(\varepsilon_A+\Delta\varepsilon)\,R_3(\Delta\psi)\,R_1(-\varepsilon_A)$，二者互为转置关系，不要混用。）

求真黄经时还要从真赤道转到真黄道：$R_1(\varepsilon_A + \Delta\varepsilon)$。合起来，太阳的真黄经恰比平黄经多 $\Delta\psi$。

## 对历法的影响

太阳每天在黄道上走约 1°，即每秒约 0.041″。黄经章动直接加在太阳黄经上，17″ 的振幅对节气时刻的影响约 7 分钟，节气必须计章动。

合朔是日月黄经**相等**的时刻，章动对两者加同一个 $\Delta\psi$，差值不变——定朔可以不计章动。

## 本项目的取法

- 默认完整的 IAU 2000A（IERS 表 5.3a、5.3b，日月项与行星项全部），网页端若完整系数表未能加载，退回 IAU 2000B；二者之差在 1 毫角秒量级，对节气时刻的影响远小于 1 秒。
- 基本角按 IERS Conventions 2010 式 5.43，以 SOFA 测试集在 $T = 0.8$ 的标准值核对，五个基本角模 $2\pi$ 相差不超过 $10^{-11}$ rad。核对取远离 J2000 的时刻，是因为高次项的系数错误在 $T = 0$ 附近显不出来。
- 节气计章动；合朔时章动对日月黄经的影响相同，两者相减即抵消。
- 配 IAU 2006 岁差时，IAU 2006 规定对章动的微小调整（量级 10 µas）未施加。

## 参考文献

1. J. Bradley, "A letter … concerning an apparent motion observed in some of the fixed stars", *Philosophical Transactions of the Royal Society* 45 (1748), 1–43。未核原文。
2. J. le R. d'Alembert, *Recherches sur la précession des equinoxes et sur la nutation de l'axe de la terre*, Paris, 1749。未核原书。
3. S. C. Chandler, "On the variation of latitude", *Astronomical Journal* 11 (1891), 59 起连载。据 ADS 扫描本核首页。
4. E. W. Woolard, "Theory of the rotation of the Earth around its center of mass", *Astronomical Papers of the American Ephemeris* 15, part 1 (1953)。未核原书。
5. J. M. Wahr, "The forced nutations of an elliptical, rotating, elastic and oceanless Earth", *Geophysical Journal of the Royal Astronomical Society* 64 (1981), 705–727。
6. P. M. Mathews, T. A. Herring, B. A. Buffett, "Modeling of nutation and precession: New nutation series for nonrigid Earth and insights into the Earth's interior", *Journal of Geophysical Research* 107(B4) (2002), 2068。
7. D. D. McCarthy, B. J. Luzum, "An abridged model of the precession-nutation of the celestial pole", *Celestial Mechanics and Dynamical Astronomy* 85 (2003), 37–49。
8. G. Petit, B. Luzum (eds.), *IERS Conventions (2010)*, IERS Technical Note 36, 第 5 章（式 5.43、表 5.3a—5.3b）。
