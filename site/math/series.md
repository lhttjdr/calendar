# 级数求值

太阳、月亮、行星的解析历表，岁差与章动的理论，都写成"时间的多项式加许多三角项"的形式。一个月亮位置要把三万多项加起来，求一次农历要算几千次月亮位置。本章讲这些级数从哪里来、各有什么样子，以及怎样又快又准地算出来：多项式的嵌套求值，三角函数的约化，导数，求和误差的控制，截断与精度的权衡。

## 一、级数从哪里来

### 摄动与三角级数

一颗行星若只受太阳吸引，轨道是固定的椭圆，位置可由开普勒方程直接解出。其他行星的吸引使椭圆缓慢变形，这就是摄动。拉格朗日与拉普拉斯在 18 世纪后期把摄动展开为轨道根数的幂级数与平近点角的三角级数，于是行星的经度、纬度、距离都写成

$$X(t) = \sum_i A_i \cos(B_i + C_i t)$$

的形式：每一项是一个周期运动，频率 $C_i$ 是各行星平运动的整数组合。周期项之外，轨道根数还有缓慢的长期变化，展开后出现 $t$、$t^2$ 乘在三角项外面的"混合项"，称为泊松项。合起来就是泊松级数：

$$X(t) = \sum_{\alpha} t^{\alpha} \sum_i A_{\alpha i} \cos(B_{\alpha i} + C_{\alpha i}\, t)$$

19 世纪的行星理论，从勒维耶（Le Verrier）到纽康（Newcomb），都是这个形式，只是项数与精度不断增加。1895 年纽康的太阳表，直到 1980 年代初仍是各国历书的依据。

### 月亮：德洛奈、希尔–布朗与 ELP

月亮受太阳的摄动远大于行星受彼此的摄动，级数收敛慢得多。德洛奈（Charles Delaunay）用二十年手算，于 1860、1867 年发表两卷月球理论，把摄动函数逐项消去，结果是字母系数的三角级数，四个基本幅角至今以他命名：

- $D$：月亮与太阳的平黄经之差（平距角）；
- $F$：月亮的平升交角距；
- $l$：月亮的平近点角；
- $l'$：太阳的平近点角。

希尔（G. W. Hill）在 1878 年改用更好的中间轨道；布朗（E. W. Brown）据此完成月球理论，1919 年出版《月球运动表》，用到 1960 年代。

计算机出现后，Chapront-Touzé 与 Chapront 用计算机代数重做半解析的月球理论，1983 年发表 ELP 2000-82，周期项两万余；2002—2003 年 Chapront 与 Francou 加入新的行星摄动并重新拟合常数，即 ELP/MPP02。

月亮级数的写法与行星稍有不同：

$$V = \sum_i A_i \sin\varphi_i(T),\qquad \varphi_i(T) = \sum_{k=0}^{4} \varphi_{ik}\, T^k$$

相位 $\varphi_i$ 是德洛奈幅角与行星平黄经的整数组合，每个幅角本身是时间 $T$ 的四次多项式，所以相位也是四次多项式。摄动部分另有一部分项带 $T$、$T^2$、$T^3$ 的泊松因子。

### 行星：VSOP

法国经度局的 Bretagnon 在 1982 年发表 VSOP82，1988 年与 Francou 发表 VSOP87，把八大行星的理论统一为泊松级数，六种版本分别给出日心黄道直角坐标、球坐标、日心黄道或赤道等不同参考架。2013 年又有 VSOP2013。地球一份（VSOP87B，日心球坐标）各变量的项数：

| 变量 | $t^0$ | $t^1$ | $t^2$ | $t^3$ | $t^4$ | $t^5$ | 合计 |
|---|---:|---:|---:|---:|---:|---:|---:|
| 黄经 $L$ | 623 | 379 | 144 | 23 | 11 | 4 | 1184 |
| 黄纬 $B$ | 184 | 134 | 62 | 14 | 6 | 2 | 402 |
| 距离 $R$ | 523 | 290 | 134 | 20 | 9 | 2 | 978 |

ELP/MPP02 的项数：

| 部分 | 经度 | 纬度 | 距离 | 合计 |
|---|---:|---:|---:|---:|
| 主问题（日、月、地三体） | 1023 | 918 | 704 | 2645 |
| 行星摄动（含 $T$、$T^2$、$T^3$ 的泊松项） | 12734 | 7030 | 13492 | 33256 |

合计约 3.6 万项，是地球级数的十余倍。

### 章动

章动是日月对地球赤道隆起的力矩引起的地轴摆动，周期由月亮轨道与地球轨道的基本幅角决定。章动的黄经分量 $\Delta\psi$ 与交角分量 $\Delta\varepsilon$ 写成

$$\Delta\psi = \sum_i (A_i + A_i' T)\sin\Phi_i + (A_i'' + A_i''' T)\cos\Phi_i$$

$\Phi_i$ 是五个日月基本幅角（德洛奈幅角加月亮升交点黄经 $\Omega$），行星项中还有行星黄经。章动级数的发展：

| 序列 | 年代 | 项数 | 地球模型 | 精度 |
|---|---|---|---|---|
| IAU 1980（Wahr） | 1980—1981 | 106 | 弹性地球、液态外核 | 约 1—5 mas |
| IAU 2000A（Mathews、Herring、Buffett） | 2002 | 日月 678，行星 687，合 1365 | 非刚体地球、地核耦合、海洋与大气，拟合甚长基线干涉测量 | 约 0.2 mas |
| IAU 2000B（McCarthy、Luzum） | 2003 | 77 | 2000A 的截断 | 约 1 mas |

IAU 2000A 公布的 678 个日月项中有幅角相同或互为相反数的重复项，合并后为 662 组。

### 数值历表与切比雪夫级数

1960 年代以后，美国喷气推进实验室（JPL）改用数值积分求太阳系天体的运动，即 DE 系列历表。数值积分的结果是逐点的位置，不便分发，于是把每一小段时间（月亮约 4 日、行星 8 至 32 日）的坐标拟合成切比雪夫多项式：

$$X(\tau) = \sum_{n=0}^{N} c_n T_n(\tau),\qquad \tau \in [-1, 1]$$

$T_n(\cos\theta) = \cos n\theta$。切比雪夫多项式在区间内的误差分布最均匀，拟合同样阶数时最大误差最小（见《最小二乘拟合》）。这种逐段系数表就是 DE 历表分发的形式，与解析历表的三角级数是两条路线：前者精确但只能查表，后者可以外推、可以推导。

## 二、时间变量与数量级

历表与岁差、章动都以 J2000.0（2000 年 1 月 1 日 12 时 TT）为起点：

$$T = \frac{\mathrm{JD} - 2451545.0}{36525}$$

即儒略世纪数；VSOP87 用儒略千年 $t = T/10$。公元前 3000 年时 $T \approx -50$、$t \approx -5$。项中出现 $T^4$ 时数值可达 $6\times10^6$，所以高次项的系数都很小，求值时要留意数量级：大数与小数相加，小数的低位会被舍掉。

## 三、多项式：嵌套求值

### 霍纳法则

$a_0 + a_1T + a_2T^2 + a_3T^3 + a_4T^4$ 不逐项求幂，而是嵌套着算：

$$a_0 + T\,(a_1 + T\,(a_2 + T\,(a_3 + T\,a_4)))$$

$n$ 次多项式只要 $n$ 次乘法、$n$ 次加法，逐项求幂则要约 $2n$ 次乘法。可以证明，对一般的多项式，这已是乘法次数的下限（Ostrowski 1954、Pan 1966）。嵌套求值的舍入误差也小：每一步只把已有的部分和乘以 $T$ 再加一个系数，不出现单独的大数 $T^4$。岁差角、德洛奈幅角、月亮相位多项式都这样求值。

### 源流

这一方法以英国人霍纳（W. G. Horner）命名，他在 1819 年的论文中用它逐位求多项式方程的根；意大利人鲁菲尼（Paolo Ruffini）在 1804 年已提出同样的做法。在中国，南宋秦九韶《數書九章》（1247）的"正負開方術"用同样的逐步变换求高次方程的数值根，北宋贾宪的"增乘開方法"（见于杨辉《詳解九章算法》，1261）是其先声。这一渊源是数学史界的通说，如李约瑟《中国科学技术史》第三卷所述（本章未核原书）。

## 四、三角函数

### 约化

级数的耗时几乎全在三角函数上。求 $\sin\varphi$ 时，先减去 $2\pi$ 的整数倍，再利用对称性把 $\varphi$ 约化到 $[-\pi/4, \pi/4]$ 附近，然后用多项式逼近。

相位本身可以很大：月亮的平黄经每世纪走约 8400 弧度，公元前 3000 年时约 $4\times10^5$ 弧度。双精度浮点数有 53 位二进制有效数字，约 16 位十进制；$4\times10^5$ 的相位，最低一位约为 $10^{-10}$ 弧度，约化后的相位只能准到这个量级。这对角秒级的位置无关紧要（$10^{-10}$ 弧度约 $2\times10^{-5}$″），但说明一个事实：相位多项式在远离历元时，有效位数是被约化"吃掉"的，不是被三角函数算错的。

### 同时求正弦与余弦

同一个相位常常既要正弦又要余弦：泊松级数的导数、旋转矩阵、复数形式的递推都要两者。一次约化同时得到两者，比分别调用省去一半的约化与多项式。

### 多倍角递推

月亮和章动的相位是基本幅角的整系数组合，$\varphi = k_1 D + k_2 F + k_3 l + k_4 l'$。可以先算出 $e^{iD}$、$e^{iF}$ 等的值，再用复数乘法组合出 $e^{i\varphi}$，不必对每一项调用三角函数：

$$e^{i(k_1 D + k_2 F)} = (e^{iD})^{k_1}(e^{iF})^{k_2}$$

这在三角函数昂贵的环境（早年的计算机、高精度算术）里能快数倍。代价是乘法累积误差，系数大时要注意。双精度下三角函数已经很快，收益有限。

## 五、导数

位置之外常常还要速度：求合朔、节气的时刻要用黄经对时间的导数（见《求根与极值》），光行时迭代也要速度。对级数逐项求导即可，不必数值差分：

$$\frac{d}{dt}\Big[t^{\alpha} A\cos(B + Ct)\Big] = \alpha\, t^{\alpha-1} A\cos(B+Ct) - t^{\alpha} A C \sin(B + Ct)$$

求导与求值共用同一次正余弦，代价只多一次乘加。数值差分的误差则同时受步长与舍入两方面限制：步长大，截断误差大；步长小，两个相近的数相减，舍入误差被放大。解析导数没有这个两难。

## 六、求和的误差

几万项相加，每加一次都有一次舍入。朴素求和的误差上界随项数 $n$ 线性增长，约 $n\epsilon\sum|x_i|$，$\epsilon$ 是机器精度；实际上舍入有正有负，典型误差约 $\sqrt{n}\,\epsilon\sum|x_i|$。

减小求和误差的办法：

| 方法 | 做法 | 误差 | 代价 |
|---|---|---|---|
| 朴素求和 | 依次相加 | $\sim n\epsilon$ | 最低 |
| 由小到大 | 项按振幅从小到大相加，小项先积累 | 改善，但无保证 | 需排序 |
| 成对求和 | 两两相加、逐层归并 | $\sim \epsilon\log n$ | 几乎不增加 |
| 补偿求和（Kahan 1965；Neumaier 1974） | 每步记下被舍掉的低位，下一步加回 | $\sim 2\epsilon$，与 $n$ 无关 | 约 4 倍加法 |
| 双倍精度（double-double） | 用两个双精度数表示一个数，约 32 位有效数字 | $\sim \epsilon^2$ | 十倍以上 |

历表级数的项按振幅从大到小排列，最大项与最小项相差十个量级以上，几万次累加的误差并不突出。实测上，在公元前 3000 年至公元 3000 年之间每百年取一点，全程双精度求值与全程 double-double 求值相比，月亮完整位置之差不超过 2 厘米（约 $10^{-5}$″），比历表本身的误差小四五个量级。所以双精度朴素求和已经足够；double-double 只作为检验双精度误差的参考。

## 七、截断：速度与精度的权衡

项按振幅从大到小排列，只取前若干项，就是截断。截断能换来速度，代价是精度。

截断误差可以估计。Bretagnon 与 Francou 给出的经验规则：保留 $n$ 项、保留项中最小的振幅为 $A$，则截去的部分误差约为

$$\eta\,\sqrt{n}\,A,\qquad \eta < 2$$

$\sqrt{n}$ 来自截去的众多小项相位杂乱、正负相消，与上一节随机舍入的 $\sqrt{n}$ 同理。Meeus《天文算法》第 31 章以此估计书中所附 VSOP87 简本：水星日心黄经的 $L_0$ 保留 38 项，最小振幅 $100\times10^{-8}$ 弧度，误差约 $2\sqrt{38}\times100\times10^{-8}$ 弧度 $\approx 2.54″$；$L_1$、$L_2$ 的截断另贡献约 $0.41″\tau$、$0.08″\tau^2$（$\tau$ 为儒略千年数）。

各家常见的截断：

| 用法 | 项数 | 精度 |
|---|---|---|
| 章动 IAU 2000A 全表 | 1365 | 约 0.2 mas |
| 章动 IAU 2000B | 77 | 约 1 mas |
| 章动 IAU 1980 | 106 | 约 1—5 mas |
| Meeus《天文算法》第 31 章附录 VSOP87 简本 | 每颗行星每个序列数项至数十项 | 按上式估计，水星黄经约 2.5″ |
| Meeus《天文算法》第 45 章月球位置（据 ELP-2000/82 主要项） | 黄经、距离各 60 项，黄纬 60 项 | 黄经约 10″，黄纬约 4″ |

对于定朔、定气，月亮黄经的 1″ 相当于约 2 秒时间，太阳黄经的 1″ 相当于约 24 秒时间。要使节气、合朔时刻准到秒级，就要用到完整的级数。

## 八、本项目的取法

- **全部项，不截断**：VSOP87 地球与行星、ELP/MPP02 月亮、IAU 2000A 章动都用全部项。级数本身的截断与建模误差（ELP 在公元前 3000 年约 1″），由拟合到 JPL 数值历表的改正来补（见天文篇《历表》）。
- **双精度**：库缺省用双精度浮点；与 double-double 的差在月亮位置上不超过 2 厘米。double-double 构建只用来检验双精度的误差。
- **一次约化同得正余弦**，导数解析求出。
- **速度**：双精度下月亮级数一次求和约 0.4 毫秒（double-double 约 10 毫秒），含光行时迭代的月亮视位置约 8 毫秒，对逐日排历与天象搜索足够快。多倍角递推与切比雪夫插值层尚未采用。

## 参考文献

1. P. Bretagnon, G. Francou, "Planetary theories in rectangular and spherical variables. VSOP 87 solutions", *Astronomy and Astrophysics* 202 (1988), 309–315。
2. M. Chapront-Touzé, J. Chapront, "The lunar ephemeris ELP 2000", *Astronomy and Astrophysics* 124 (1983), 50–62。
3. J. Chapront, G. Francou, "The lunar theory ELP revisited. Introduction of new planetary perturbations", *Astronomy and Astrophysics* 404 (2003), 735–742。
4. C. Delaunay, *La théorie du mouvement de la lune*, *Mémoires de l'Académie des Sciences* 28 (1860), 29 (1867)。未核原书。
5. E. W. Brown, *Tables of the Motion of the Moon*, New Haven: Yale University Press, 1919。未核原书。
6. S. Newcomb, "Tables of the motion of the Earth on its axis and around the Sun", *Astronomical Papers of the American Ephemeris* 6, part 1 (1895)。未核原书。
7. E. W. Woolard, "Theory of the rotation of the Earth around its center of mass", *Astronomical Papers of the American Ephemeris* 15, part 1 (1953)。未核原书。
8. J. M. Wahr, "The forced nutations of an elliptical, rotating, elastic and oceanless Earth", *Geophysical Journal of the Royal Astronomical Society* 64 (1981), 705–727。
9. P. M. Mathews, T. A. Herring, B. A. Buffett, "Modeling of nutation and precession: New nutation series for nonrigid Earth and insights into the Earth's interior", *Journal of Geophysical Research* 107(B4) (2002), 2068。
10. D. D. McCarthy, B. J. Luzum, "An abridged model of the precession-nutation of the celestial pole", *Celestial Mechanics and Dynamical Astronomy* 85 (2003), 37–49。未核原文。
11. X X Newhall, "Numerical representation of planetary ephemerides", *Celestial Mechanics* 45 (1989), 305–310。未核原文。
12. W. G. Horner, "A new method of solving numerical equations of all orders, by continuous approximation", *Philosophical Transactions of the Royal Society* 109 (1819), 308–335。未核原文。
13. W. Kahan, "Further remarks on reducing truncation errors", *Communications of the ACM* 8(1) (1965), 40。未核原文。
14. A. Neumaier, "Rundungsfehleranalyse einiger Verfahren zur Summation endlicher Summen", *Zeitschrift für Angewandte Mathematik und Mechanik* 54 (1974), 39–51。未核原文。
15. J. Meeus, *Astronomical Algorithms*, 2nd ed., Willmann-Bell, 1998，第 31 章《行星位置》、第 45 章《月球位置》。论断据 Meeus 原书；查阅所用为许剑伟的业余中译本（2008），未与英文原书逐字核对。
16. 李约瑟（J. Needham），《中国科学技术史》第三卷《数学、天学和地学》。秦九韶、贾宪一节据数学史通说，未核原书。
