# 模型配套

一次计算要用到许多成分：太阳与月亮的历表、$\Delta T$、参考架、岁差与章动。每一样都是对某个基准拟合或推导出来的。把基准不同的成分混在一起用，任何一项单独检验都看不出问题，误差却会在古代累积成分钟级。本章说明本项目怎样让这些成分彼此配套。

## 源流：常数系与潮汐加速度

"配套"的思想并不新。天文常数之间相互牵连，1896 年巴黎会议首次约定一组通用的天文常数，此后 IAU 1964、1976 两次修订"天文常数系"：岁差常数、章动常数、黄赤交角、光行差常数、天文单位等作为一个整体采用，星表（FK5）、历表（DE200）、岁差（L77）、章动（IAU 1980）彼此配套。2000 年以后的 IAU 2000/2006 决议同样成套：IAU 2006 岁差、IAU 2000A 章动、国际天球参考系与 DE 历表。

古代天象计算中最要紧的配套，是 $\Delta T$ 与月历表之间的配套，关键量是月球的潮汐加速度 $\dot n$。潮汐使地球自转变慢，同时把角动量交给月球，使月球远离地球、平运动变慢，表现为月亮平黄经中的一个 $T^2$ 项，系数为 $\dot n/2$。从古代日月食反推 $\Delta T$ 时，结果取决于所用月历表的 $\dot n$；$\dot n$ 差 $1''/\mathrm{cy}^2$，两千年前的 $\Delta T$ 就差约 6 分钟。$\dot n$ 的测定史：

| 年代 | 测定 | $\dot n$（$''/\mathrm{cy}^2$） | 方法 |
|---|---|---|---|
| 1939 | Spencer Jones | −22.44（值据通说） | 太阳、水星、金星、月亮的历史观测比较 |
| 1975 | Morrison、Ward | −26 ± 2 | 1677—1973 年约 2400 次水星凌日观测 |
| 1994 | Dickey 等 | −25.88 ± 0.5 | 激光测月 |
| 2002 | Chapront 等（ELP/MPP02 所用） | −25.858 | 激光测月 |
| 2014 | DE430/DE431 | −25.82 | 激光测月，数值历表 |

激光测月以后，$\dot n$ 的不确定度降到 $0.5''/\mathrm{cy}^2$ 以内，但各家仍有零点几的出入；而古代 $\Delta T$ 是 Stephenson、Morrison 等先后用某一个 $\dot n$ 反推的（1984、1995、2004、2016、2021 各版），所以用 $\Delta T$ 时必须知道它配的是哪一个 $\dot n$、哪一部月历表。NASA《五千年日食典》（Espenak、Meeus 2006）在书中给出换算式

$$\delta(\Delta T) = -0.910\,72\,(\dot n + 26)\,u^2$$

即为此而设。

## 以 DE431 为基准

古代的 $\Delta T$ 是用某一部月历表从日月食记录反推的。计算时若换了月历表，$\Delta T$ 必须按两者月亮平黄经之差 $\Delta\lambda$ 修正：

$$\delta(\Delta T) = -\frac{\Delta\lambda}{n_\text{月}},\qquad n_\text{月} = 0.549\,016''/\mathrm{s}$$

月亮在天上每秒走约 0.55″，所以 1″ 的黄经差相当于约 1.8 秒的 $\Delta T$。

本项目所用的 $\Delta T$（Morrison–Stephenson 2021）是用 JPL DE430/DE431 的月亮反推的。于是把 DE431 定为配套的基准：

- 太阳（地球）的历表 VSOP87 拟合到 DE431；
- 月亮的历表 ELP/MPP02 拟合到 DE431（常数加残差改正）；
- $\Delta T$ 取 Morrison–Stephenson 2021。

三者同出一源。缺省月亮与 DE431 的平黄经每 400 年分段平均都在 ±0.16″ 以内，$\Delta T$ 的配套修正为零。

## 各成分的基准

| 成分 | 本项目的取法 | 基准 | 可选 |
|---|---|---|---|
| 太阳与行星 | VSOP87 加日心改正 | DE431 | 改正拟合到 DE406 或 DE441 |
| 月亮 | ELP/MPP02，重拟常数并加残差改正 | DE431 | 论文常数集（贴近 DE406）、拟合 DE441 |
| 月亮的参考架 | 理论自己的 J2000.0 平黄道，按论文所定的转角转到国际天球参考系 | 激光测月所定 | — |
| 太阳的参考架 | VSOP87 的动力学黄道，经 J2000.0 黄赤交角 84381.406″ 与架偏差转到国际天球参考系 | 改正就在这条链下拟合 | — |
| $\Delta T$ | Morrison–Stephenson 2021，按所用月历表修正 | DE430/DE431 | Espenak–Meeus 2006（假定 $\dot n = -26''/\mathrm{cy}^2$，须先换算到 DE431） |
| 岁差 | Vondrák 2011（长期） | J2000.0 黄赤交角 84381.406″ | IAU 2006（P03） |
| 章动 | IAU 2000A 全表 | — | IAU 2000B（77 项） |
| 历表的时间变量 | 以 TT 代 TDB | — | 两者之差不到 2 毫秒，月亮相差不到 0.001″ |

太阳和月亮的历表各有自己的参考架，不能互相借用：两条转换链的春分点相差约 0.044″，折合合朔时刻约 0.08 秒。

1955 年以后有原子时，$\Delta T$ 由地球自转直接测得，与月历表无关，不做配套修正。

## 不配套会差多少

各月历表相对 DE431 的月亮平黄经之差，按三次式折算成 $\Delta T$ 的配套修正：

| 月历表 | 前 2000 年 | 前 700 年 | 前 200 年 | 公元 0 年 | 1000 年 |
|---|---:|---:|---:|---:|---:|
| ELP/MPP02 拟合 DE431（缺省） | 0.0 s | 0.0 s | 0.0 s | 0.0 s | 0.0 s |
| ELP/MPP02 论文常数集 | +6.1 s | +1.3 s | +0.5 s | +0.2 s | −0.2 s |
| JPL DE406 | +6.2 s | +1.3 s | +0.4 s | +0.2 s | −0.2 s |
| ELP/MPP02 拟合 DE441 | +367 s | +112 s | +60 s | +45 s | +5.3 s |
| JPL DE441 | +367 s | +112 s | +60 s | +45 s | +5.3 s |

$\Delta T$ 模型之间也要换算。Espenak–Meeus 2006 假定月球潮汐加速度 $\dot n = -26''/\mathrm{cy}^2$，换到 DE431 的 $-25.82''/\mathrm{cy}^2$：

$$\delta(\Delta T) = -0.910\,72\,(\dot n_\text{历表} - \dot n_{\Delta T})\,u^2,\qquad u = \frac{\text{年} - 1955}{100}$$

前 200 年约 −76 秒，公元 0 年约 −63 秒，1000 年约 −15 秒。

由此可见几种常见的混用各差多少：

| 混用 | 前 200 年的误差 |
|---|---|
| Espenak–Meeus 2006 直接配 ELP/MPP02，不换算 | 约 76 秒 |
| Morrison–Stephenson 2021 直接配 DE441 的月亮 | 约 60 秒 |
| 太阳用 DE406 版改正、月亮用 DE431 版 | 平黄经约 0.5″，约 1 秒 |
| 月亮借用太阳的参考架转换 | 0.044″，约 0.08 秒 |

## DE441 为什么不作基准

DE441 是 JPL 现行的长时段历表，月亮在古代与 DE431 差得很多：前 200 年约 33″，前 2900 年约 330″。它与 Morrison–Stephenson 2021 的 $\Delta T$ 配用，前 200 年要修正约一分钟；而且两者之间还有近地点、升交点运动不同造成的周期差（约 6″），这是轨道形状之差，$\Delta T$ 修正补不了。

所以只要 $\Delta T$ 还是用 DE430/DE431 的月亮反推的，基准就应是 DE431。将来若改用按 DE440/DE441 反推的 $\Delta T$，基准与缺省历表应一并换成 DE441。DE406、DE441 两套拟合作为选项保留，选用时太阳、月亮与 $\Delta T$ 的修正一起换。

## 参考文献

1. H. Spencer Jones, "The rotation of the earth, and the secular accelerations of the sun, moon and planets", *Monthly Notices of the Royal Astronomical Society* 99 (1939), 541–558。据 ADS 扫描本核首页；表中 −22.44 未在首页见到，据通说。
2. L. V. Morrison, C. G. Ward, "An analysis of the transits of Mercury: 1677–1973", *Monthly Notices of the Royal Astronomical Society* 173 (1975), 183–206。据 ADS 扫描本核摘要。
3. J. O. Dickey et al., "Lunar laser ranging: a continuing legacy of the Apollo program", *Science* 265 (1994), 482–490。据 Crossref 核书目，doi:10.1126/science.265.5171.482。
4. J. Chapront, M. Chapront-Touzé, G. Francou, "A new determination of lunar orbital parameters, precession constant and tidal acceleration from LLR measurements", *Astronomy and Astrophysics* 387 (2002), 700–709。
5. W. M. Folkner, J. G. Williams, D. H. Boggs, R. S. Park, P. Kuchynka, "The Planetary and Lunar Ephemerides DE430 and DE431", *IPN Progress Report* 42-196 (2014)。
6. L. V. Morrison, F. R. Stephenson, "Historical values of the Earth's clock error ΔT and the calculation of eclipses", *Journal for the History of Astronomy* 35 (2004), 327–336。
7. L. V. Morrison, F. R. Stephenson, C. Y. Hohenkerk, M. Zawilski, "Addendum 2020 to 'Measurement of the Earth's rotation: 720 BC to AD 2015'", *Proceedings of the Royal Society A* 477 (2021), 20200776。
8. F. Espenak, J. Meeus, *Five Millennium Canon of Solar Eclipses: −1999 to +3000*, NASA/TP-2006-214141, 2006。
