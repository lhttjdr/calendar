# 转动

求日月的视位置，大部分工作是把一个方向从一个参考架换到另一个参考架：从历表所用的黄道架换到赤道架，从 J2000 的平赤道换到观测当日的真赤道，从地心换到观测者所在的站心。每一步都是三维空间中的一次转动，有时还带一次平移。

本章先讲转动怎样用数来表示（矩阵、欧拉角、转轴与转角、四元数、对偶四元数），以及各自的推导；再讲这些表示是怎样一步步发展出来的；然后把各家并列比较，说明天文学和航天、图形学各自惯用哪一种、为什么；最后说明本项目的取法。

## 一、转动矩阵

### 坐标转动与物体转动

在一个右手直角坐标架中，方向写成单位向量 $\mathbf{v} = (x, y, z)^\mathsf{T}$。换一个原点相同、坐标轴转过一定角度的新架，同一个方向在新架里的分量是 $\mathbf{v}' = R\,\mathbf{v}$。$R$ 是 3×3 矩阵，它的三行就是新架三根坐标轴在旧架中的单位向量。新旧两架都是单位正交的，所以

$$R^\mathsf{T} R = I,\qquad \det R = +1$$

满足这两个条件的矩阵组成"特殊正交群" $SO(3)$。$R^\mathsf{T} R = I$ 给出 6 个独立的方程，9 个元素只剩 3 个自由度，这就是"转动有三个自由度"的代数意义。

同一个矩阵有两种读法。坐标轴转过 $\theta$、方向不动，叫"坐标转动"（被动转动）；坐标轴不动、方向本身转过 $\theta$，叫"物体转动"（主动转动）。两者的矩阵互为转置。天文学一律用坐标转动：问的总是"同一颗星在新架里的坐标是多少"。

绕三根坐标轴转 $\theta$ 的坐标转动矩阵，天文上记作 $R_1$、$R_2$、$R_3$：

$$
R_1(\theta)=\begin{pmatrix}1&0&0\\0&\cos\theta&\sin\theta\\0&-\sin\theta&\cos\theta\end{pmatrix},\quad
R_2(\theta)=\begin{pmatrix}\cos\theta&0&-\sin\theta\\0&1&0\\\sin\theta&0&\cos\theta\end{pmatrix},\quad
R_3(\theta)=\begin{pmatrix}\cos\theta&\sin\theta&0\\-\sin\theta&\cos\theta&0\\0&0&1\end{pmatrix}
$$

$\theta$ 为正时，从轴的正端看，坐标轴逆时针转；架中的点因此看起来顺时针转。以 $R_3$ 为例：新 $x'$ 轴在旧架中是 $(\cos\theta, \sin\theta, 0)$，与 $\mathbf{v}$ 的点积 $x\cos\theta + y\sin\theta$ 就是第一行。其余两个同理。这套记法通行于 IAU 的天文基本标准库 SOFA，以及 IERS 规范。

### 组合

几次转动相继进行，把矩阵依次左乘：先做 $R_a$、再做 $R_b$，合起来是 $R_b R_a$，读的次序从右到左。转动一般不可交换，$R_b R_a \ne R_a R_b$，所以次序错了结果就错。

天文上最常见的例子是赤道与黄道的互换。赤道架与黄道架共用春分点方向作 $x$ 轴，两者只差绕 $x$ 轴转一个黄赤交角 $\varepsilon$：

$$\mathbf{v}_\text{黄道} = R_1(\varepsilon)\,\mathbf{v}_\text{赤道}$$

### 正交性的维持

矩阵用 9 个数表达 3 个自由度，多出的 6 个约束在计算中只能近似满足。连乘多次后，舍入误差会使 $R^\mathsf{T}R$ 偏离单位阵，矩阵带上微小的伸缩与切变。修复的方法是重新正交化，例如 Gram–Schmidt 逐列正交化，或用极分解取最近的正交阵。这两种做法都不便宜，也不唯一。天文计算中，一次视位置只连乘几个到十几个矩阵，偏离在 $10^{-16}$ 量级，远小于观测精度，一般不必处理。

## 二、欧拉角

### 三次绕轴转

任何转动都能分解为依次绕坐标轴的三次转动，三个角称为欧拉角。欧拉在 18 世纪 50—60 年代研究刚体转动时用三个角描述刚体的方位，后人因以命名。天文学描述轨道面的三个根数，升交点黄经 $\Omega$、轨道倾角 $i$、近点角距 $\omega$，也正是"绕 $z$、绕 $x$、再绕 $z$"的三次转动。

岁差是天文上最典型的欧拉角。从 J2000 平赤道架到观测日的平赤道架，Newcomb 以来的标准写法是

$$P = R_3(-z_A)\,R_2(\theta_A)\,R_3(-\zeta_A)$$

从右往左读：先绕 $z$ 轴转 $-\zeta_A$，使 $x$ 轴指向两赤道的交点；再绕新的 $y$ 轴转 $\theta_A$，把旧赤道面倾到新赤道面；最后绕新的 $z$ 轴转 $-z_A$，把 $x$ 轴转到新的春分点。三个角各是时间的多项式，详见天文篇《岁差》。

章动也是三次转动：

$$N = R_1(-\varepsilon-\Delta\varepsilon)\,R_3(-\Delta\psi)\,R_1(\varepsilon)$$

先由赤道转到平黄道，在黄道上转过黄经章动 $\Delta\psi$，再按真黄赤交角 $\varepsilon + \Delta\varepsilon$ 转回赤道。

### 同一转动的不同分解

欧拉角的分解不唯一，轴的次序有 12 种选法，各领域习惯不同。即使在岁差这一件事上，天文学也有多种分解并存：

| 分解 | 角 | 经典出处 | 后来的采用 |
|---|---|---|---|
| 赤道三角式 | $\zeta_A, \theta_A, z_A$ | Newcomb（1906） | Lieske 等 1977（IAU 1976）；Capitaine 等 2003（P03） |
| 黄道四角式 | $\varepsilon_0, \psi_A, \omega_A, \chi_A$ | Lieske 等 1977 | P03 |
| Fukushima–Williams 四角式 | $\bar\gamma, \bar\phi, \bar\psi, \varepsilon_A$ | Williams 1994；Fukushima 2003 | SOFA 中 IAU 2006 岁差的主要形式 |

三角式只有三个角，最省；但它把黄道的运动和赤道的运动混在一起，物理意义不明显。四角式多用一个角，换来每个角都有清楚的物理意义：赤道绕黄极的进动、黄道自身的转动各占一份。Fukushima–Williams 式把岁差和章动写进同一组角，章动只需在 $\bar\psi$、$\varepsilon_A$ 上加 $\Delta\psi$、$\Delta\varepsilon$，不必另乘一个章动矩阵。

各式都是截断的多项式，远离历元时各自的截断误差不同，于是互有出入。按 P03 三角式与四角式两组系数直接构造矩阵比较，公元前 1000 年两者相差 1.25″。

### 万向锁

欧拉角有一个固有的缺陷：当中间那次转动的角为 0 或 180° 时，第一次与第三次转动绕的是同一根轴，两个角只有和（或差）有意义，各自不确定，转动只剩两个自由度。这种退化在机械陀螺上叫"万向锁"（gimbal lock）。它不只是数值问题：在退化点附近，三个角随姿态的微小变化而剧烈变化，求导、插值都会出错。

岁差的 $\theta_A$ 从 0 开始增长，在 J2000 附近正处于这种退化附近：$\zeta_A$ 与 $z_A$ 都含有 2.65″ 的常数项，大小相等、符号相反，单看各自都不为零，合起来却抵消。这正是三角式在历元附近的一个不便之处。

## 三、欧拉转动定理与转轴转角

### 定理

无论几次转动连在一起，结果都等于绕某一根轴转一个角。欧拉在 1775 年写成、1776 年发表的《Formulae generales pro translatione quacunque corporum rigidorum》（刚体任意移动的一般公式）中证明了这一点。他先说"刚体无论怎样从一个位置移到另一个位置，总有一条直线，其方向不受任何改变"，接着为便于证明，设想一个与刚体固连的球：

> Theorema. Quomodocunque sphaera circa centrum suum convertatur, semper assignari potest diameter, cuius directio in situ translato conveniat cum situ initiali.（E478 第 24 节补遗，《新评论》第 20 卷第 202 页）

即"球无论怎样绕球心转动，总可以指出一条直径，其方向在转后与转前相同"。

用矩阵的语言，这等于说 $R$ 必有特征值 1。证明只需一行：

$$\det(R - I) = \det(R - RR^\mathsf{T}) = \det R\cdot\det(I - R^\mathsf{T}) = \det(I - R)^\mathsf{T} = -\det(R - I)$$

最后一步用到 3 阶矩阵 $\det(-A) = -\det A$。于是 $\det(R - I) = 0$，1 是特征值，对应的特征向量 $\mathbf{n}$ 满足 $R\mathbf{n} = \mathbf{n}$，就是转轴。

转角可从矩阵的迹读出。绕 $\mathbf{n}$ 转 $\varphi$ 的矩阵，在以 $\mathbf{n}$ 为第三轴的架中就是 $R_3(\varphi)$，迹为 $1 + 2\cos\varphi$。迹在换架时不变，所以

$$\cos\varphi = \frac{\operatorname{tr} R - 1}{2}$$

转轴由矩阵的反对称部分给出：$\mathbf{n}\sin\varphi$ 的三个分量分别是 $\tfrac12(R_{23} - R_{32})$、$\tfrac12(R_{31} - R_{13})$、$\tfrac12(R_{12} - R_{21})$（坐标转动的符号）。

### 一个天文实例

按 P03 的三角式，取公元前 1000 年（距 J2000 −30 世纪），三角为 $\zeta_A = -19.28°$、$\theta_A = -16.50°$、$z_A = -19.09°$，三次转动合成的矩阵用上式求转轴转角，得：

- 转角 41.64°；
- 转轴距 J2000 黄极 0.37°。

三步合成的，其实就是绕黄极的一次转动。这是因为岁差的主体是天极绕黄极的进动，41.64° 大致就是三千年间的黄经总岁差；转轴偏离黄极的那 0.37°，来自黄道本身的缓慢转动（行星岁差）。三个欧拉角都在 16° 到 19° 之间，看不出这一层意思；换成转轴转角，物理图像一目了然。

### 罗德里格斯公式

已知转轴 $\mathbf{n}$ 和转角 $\varphi$，物体转动后的向量是

$$\mathbf{v}' = \mathbf{v}\cos\varphi + (\mathbf{n}\times\mathbf{v})\sin\varphi + \mathbf{n}\,(\mathbf{n}\cdot\mathbf{v})(1 - \cos\varphi)$$

推导：把 $\mathbf{v}$ 分为平行于轴的部分 $\mathbf{v}_\parallel = \mathbf{n}(\mathbf{n}\cdot\mathbf{v})$ 与垂直于轴的部分 $\mathbf{v}_\perp = \mathbf{v} - \mathbf{v}_\parallel$。平行部分不动；垂直部分在与轴垂直的平面内转 $\varphi$，这个平面的两根正交方向是 $\mathbf{v}_\perp$ 与 $\mathbf{n}\times\mathbf{v}$，所以转后为 $\mathbf{v}_\perp\cos\varphi + (\mathbf{n}\times\mathbf{v})\sin\varphi$。两部分相加即得上式。

写成矩阵，记 $K$ 为叉乘 $\mathbf{n}\times$ 对应的反对称矩阵，则

$$R = I + K\sin\varphi + K^2(1 - \cos\varphi)$$

这个式子以 Olinde Rodrigues 命名，他在 1840 年的论文中给出了转动的组合公式。把 $\mathbf{n}\varphi$ 合成一个向量，就是"转动向量"，它只用 3 个数，没有多余的约束；但在 $\varphi = 180°$ 处，$\mathbf{n}$ 与 $-\mathbf{n}$ 表示同一个转动，表示不连续。

## 四、四元数

### 由来

1843 年 10 月 16 日，哈密顿（William Rowan Hamilton）沿都柏林皇家运河散步，想通了困扰他多年的问题：复数能表示平面转动，三维的"数"应该怎样乘？答案是用四个分量，三个虚单位满足

$$i^2 = j^2 = k^2 = ijk = -1$$

他当场把这个式子刻在了布鲁姆桥的石头上。二十二年后（1865 年 8 月 5 日），他在给儿子 Archibald 的信中回忆："Nor could I resist the impulse — unphilosophical as it may have been — to cut with a knife on a stone of Brougham Bridge"（桥名当时作 Brougham，今作 Broome）。今天桥上的纪念牌记的就是这件事。哈密顿自次年起在《Philosophical Magazine》上分期发表这一结果，一直连载到 1850 年。

两年后，凯莱（Arthur Cayley，1845）写出 $(1+\lambda i+\mu j+\nu k)^{-1}(ix+jy+kz)(1+\lambda i+\mu j+\nu k)$，指出它给出的恰是一组直角坐标轴到另一组直角坐标轴的变换，其中 $\lambda = \tan\frac{\theta}{2}\cos f$ 等（$\theta$ 为转角，$f$ 等为转轴的方向角），并说："In fact the formulae are precisely those given for such a transformation by M. Olinde Rodrigues"（凯莱《论四元数的若干结果》，《数学论文集》第一卷第 124 页）。这就是下文的 $q\,v\,q^*$。不过，这四个参数在四元数之前已经两次出现。第一次是欧拉。他在 1770 年写成的《一个因性质极特殊而值得注意的代数问题》（E407）开篇就提出：把九个数排成方阵，使每行、每列的平方和为 1，两行、两列对应相乘之和为 0，也就是求全部三维正交矩阵。他先用三次平面转动依次复合，得到含三个任意角的解（即今天的欧拉角）；文末又"e methodo Diophantea"（用丢番图的方法）给出有理数解：

> Sumantur pro lubitu quatuor numeri $p, q, r, s$ ac posita quadratorum eorum summa $pp + qq + rr + ss = u$ novem numeri quaesiti ita determinati reperiuntur（E407 §XXXIII，《新评论》第 15 卷第 101 页）

即"任取四个数 $p, q, r, s$，记其平方和为 $u$，则所求九数为"

$$
\begin{array}{lll}
A = \dfrac{pp+qq-rr-ss}{u}, & B = \dfrac{2qr+2ps}{u}, & C = \dfrac{2qs-2pr}{u},\\[2mm]
D = \dfrac{2qr-2ps}{u}, & E = \dfrac{pp-qq+rr-ss}{u}, & F = \dfrac{2pq+2rs}{u},\\[2mm]
G = \dfrac{2qs+2pr}{u}, & H = \dfrac{2rs-2pq}{u}, & I = \dfrac{pp-qq-rr+ss}{u}.
\end{array}
$$

取 $p$ 为标量部分、$(q, r, s)$ 为向量部分，这正是今天由单位四元数写出的转动矩阵（差一个转置，即坐标转动与物体转动之别）。紧接着的第 102 页，欧拉又对四维的情形用八个数 $a, b, c, d, p, q, r, s$ 排出十六个数，每行、每列的平方和都等于 $(aa+bb+cc+dd)(pp+qq+rr+ss)$。这就是四平方和恒等式，也就是后来四元数乘法保持模长的性质。第二次是罗德里格斯。他在 1840 年从几何上重新得到这四个参数，与转轴、转角的关系正是 $\cos\frac{\varphi}{2}$ 与 $\mathbf n\sin\frac{\varphi}{2}$，并写出了两次转动相继进行时参数的组合规则。这条规则就是后来的四元数乘法，只是他没有把它看成一种新的数。凯莱所说"与罗德里格斯给出的完全相同"，指的就是这一点。所以这四个参数今天称为"欧拉–罗德里格斯参数"。此外，高斯在 1819 年前后的手稿《Mutationen des Raumes》中也有类似公式，生前未发表。

### 定义与乘法

四元数 $q = q_0 + q_1 i + q_2 j + q_3 k$，可写成标量与向量之和 $q = q_0 + \mathbf{q}$。由 $i^2 = j^2 = k^2 = ijk = -1$ 推出 $ij = k = -ji$ 等关系，两个四元数的乘积可整理为

$$pq = p_0 q_0 - \mathbf{p}\cdot\mathbf{q} + p_0\mathbf{q} + q_0\mathbf{p} + \mathbf{p}\times\mathbf{q}$$

点积和叉积两种向量运算，就是在这里一并出现的。由于叉积不可交换，四元数乘法也不可交换，正好与转动的不可交换相合。共轭为 $q^* = q_0 - \mathbf{q}$，模为 $|q|^2 = qq^* = q_0^2 + |\mathbf{q}|^2$。

### 转动的表示

绕单位向量 $\mathbf{n}$ 转 $\varphi$，对应单位四元数

$$q = \cos\frac{\varphi}{2} + \mathbf{n}\sin\frac{\varphi}{2}$$

把向量 $\mathbf{v}$ 写成纯向量四元数（标量为 0），物体转动为

$$\mathbf{v}' = q\,\mathbf{v}\,q^*$$

用上面的乘法公式逐项展开，再用 $\cos\varphi = \cos^2\frac{\varphi}{2} - \sin^2\frac{\varphi}{2}$、$\sin\varphi = 2\sin\frac{\varphi}{2}\cos\frac{\varphi}{2}$ 化简，结果与罗德里格斯公式完全相同。这就是半角出现的原因：$q$ 在两侧各乘一次，各转一半。

由此还有两个推论：

- **组合**：先转 $q_1$ 再转 $q_2$，合起来是 $q_2 q_1$，因为 $q_2(q_1 \mathbf{v} q_1^*)q_2^* = (q_2q_1)\mathbf{v}(q_2q_1)^*$。次序与矩阵相同。
- **双重覆盖**：$q$ 与 $-q$ 给出同一个转动。单位四元数构成三维球面 $S^3$，转动群 $SO(3)$ 是它把对径点粘合后的结果。

### 好处

- **紧凑**：4 个数，只有一个约束 $|q| = 1$。
- **易于纠偏**：舍入使模长偏离 1 时，除以模长即恢复，不像矩阵要重新正交化。
- **没有万向锁**：$S^3$ 上处处光滑，任何姿态附近都没有退化。
- **便于插值**：在两个姿态之间沿最短路径匀速过渡，有球面线性插值（slerp），由 Shoemake 在 1985 年为计算机动画提出：

$$\operatorname{slerp}(q_a, q_b; s) = q_a\,(q_a^* q_b)^s$$

即把相对转动 $q_a^* q_b$ 的转角按比例 $s$ 缩放。矩阵和欧拉角都没有这样简单的对应办法。

<Anim src="celestial-sphere/quaternions" caption="上：公元前 1000 年的岁差，三步合为绕一根轴的一次转动，三个元素四元数相乘得到同一个 q；下：日心黄道架到地心赤道架的螺旋运动" />

### 两种乘法约定

四元数在航天界有一段混乱的历史。哈密顿的原始约定是 $ij = k$；美国喷气推进实验室（JPL）等航天机构在 20 世纪后期为了让四元数乘法与坐标转动矩阵乘法次序一致，改用了 $ij = -k$ 的约定，并把标量放在最后。Shuster 1993 年的综述《A survey of attitude representations》对此有系统的整理。两种约定下，同一个公式符号不同，混用就会出错。Sommer 等 2018 年专文主张统一回到哈密顿约定。本章与本项目都用哈密顿约定，标量在前。

## 五、对偶四元数：转动加平移

### 沙勒定理

视位置计算不只有转动。太阳的位置先在日心坐标里给出，要换到地心，需要平移；换到观测者所在的站心，还要再平移一次。

刚体运动（转动加平移）也有一个类似欧拉定理的结论：任何刚体运动都等于绕某一根空间直线转动，同时沿这条直线平移，即一次"螺旋运动"。这一定理通常以沙勒（Michel Chasles）命名。他在 1831 年 2 月向巴黎爱智学会宣读的短文中，先证明刚体任意位移后总有一条直线回到原处，接着说：

> …l'on peut toujours transporter un corps solide libre d'une position dans une autre position quelconque déterminée, par le mouvement continu d'une vis à laquelle ce corps serait fixé invariablement.（《费吕萨克通报》数学部分第 14 卷，1830 年 11 月号，第 324 页）

即"自由刚体从一个位置到任一指定位置，总可以由固定在它上面的一根螺杆的连续运动来完成"。同文又指出无穷小运动"n'est autre que celui d'une vis dans son écrou"（就是螺杆在螺母中的运动）。不过意大利人莫齐（Giulio Mozzi）在 1763 年已经对无穷小运动给出了瞬时螺旋轴。

### 对偶数与对偶四元数

克利福德（William Kingdon Clifford）在 1873 年引入"双四元数"（biquaternion），用一个满足 $\varepsilon^2 = 0$ 的"对偶单位"把两个四元数拼成一个对象；后来斯图迪（Eduard Study）把它系统地用于刚体运动学。克利福德引入的符号 $\omega$ 把绕某轴的转动变成沿该轴的平移，他写道：

> …if made to operate twice upon a motor, it reduces it to zero; or $\omega^2 A = 0$ always. The portion of any expression which involves $\omega$ must therefore be treated as an infinitesimal of the first order; all higher orders being uniformly neglected.（克利福德《双四元数初探》，《Mathematical Papers》第 186 页）

这就是今天对偶数 $a + \varepsilon b$ 的要义：$\varepsilon^2 = 0$，泰勒展开到一阶就截断，所以

$$f(a + \varepsilon b) = f(a) + \varepsilon\,b\,f'(a)$$

这使对偶角 $\hat\theta = \theta + \varepsilon d$（转角加沿轴的位移）能像普通角一样代入三角函数。

转动 $q$ 后再平移 $\mathbf{t}$ 的刚体运动，写成对偶四元数为

$$\hat q = q + \varepsilon\,\tfrac12\,\mathbf{t}\,q$$

两次刚体运动的组合，就是两个对偶四元数相乘；转动和平移一并处理，不必像齐次坐标那样用 4×4 矩阵。单位对偶四元数有 8 个分量、两个约束（实部模为 1，实部与对偶部正交），恰好 6 个自由度。

动画下半部分演示从"日心黄道架"到"地心赤道架"：原点从太阳移到地球，同时坐标轴绕春分点方向转过黄赤交角 23.44°。取春分日，地球恰在太阳的秋分点方向，此时螺旋轴正好是日地连线：坐标架一边沿日地连线滑过去，一边绕它转。

对偶四元数如今多用于机器人运动学、计算机动画的骨骼蒙皮，在天文计算中用得很少。原因是天文的平移与转动性质不同：平移来自历表（日、地、月的位置），转动来自岁差章动，两者分别求得，合在一个代数对象里并不省事。

## 六、各家并列

| 表示 | 数的个数 | 约束 | 组合一次（乘法/加法） | 转一个向量 | 退化 | 惯用领域 |
|---|---|---|---|---|---|---|
| 转动矩阵 | 9 | 6 | 27 / 18 | 9 / 6 | 无 | 天文（SOFA、IERS 规范）、一般科学计算 |
| 欧拉角 | 3 | 0 | 需换成矩阵或四元数 | 同左 | 万向锁 | 岁差章动的参数化、轨道根数 |
| 转轴转角、转动向量 | 4 或 3 | 1 或 0 | 无直接公式 | 罗德里格斯公式 | 180° 处不连续 | 解释、可视化、小转动的线性化 |
| 单位四元数 | 4 | 1 | 16 / 12 | 两次乘法，或换成矩阵 | 无（双重覆盖） | 航天器姿态、计算机图形、动画插值 |
| 单位对偶四元数 | 8 | 2 | 48 / 40 | 同上加平移 | 无 | 机器人、骨骼动画 |

运算次数是直接按定义计数的通常值，可通过特殊的分组再省一些。由表可见：

- 只做"转一个向量"，矩阵最快；岁差章动的矩阵每个时刻只构造一次，却要作用于日、月、行星多个向量，所以天文计算惯用矩阵。
- 要频繁组合、长时间累积、在姿态之间插值的场合，四元数最合适；航天器的姿态控制每秒要积分多次角速度，所以航天惯用四元数。
- 欧拉角是给人读的：每个角有物理意义，适合作为模型的参数（岁差的 $\zeta_A, \theta_A, z_A$，章动的 $\Delta\psi, \Delta\varepsilon$），但不适合直接拿来算。

## 七、源流小结

| 年代 | 人物 | 贡献 |
|---|---|---|
| 1763 | 莫齐 | 刚体无穷小运动的瞬时螺旋轴 |
| 18 世纪 50—60 年代 | 欧拉 | 用三个角描述刚体方位（欧拉角） |
| 1770—1771 | 欧拉 | 三维正交矩阵的四参数有理表示，即后来的四元数转动矩阵（E407） |
| 1775—1776 | 欧拉 | 转动定理：任何转动都是绕一根轴的转动 |
| 约 1819 | 高斯 | 手稿中的转动组合公式（身后发表） |
| 1830—1831 | 沙勒 | 刚体运动都是螺旋运动 |
| 1840 | 罗德里格斯 | 四个参数的几何意义与转动的组合公式 |
| 1843—1844 | 哈密顿 | 四元数 |
| 1845 | 凯莱 | 用四元数表示三维转动 |
| 1873 | 克利福德 | 双四元数（对偶四元数） |
| 1891—1903 | 斯图迪 | 对偶四元数用于刚体运动学 |
| 1906 | Newcomb | 岁差的三角式 $\zeta, \theta, z$ |
| 1977 | Lieske 等 | IAU 1976 岁差，三角式与黄道四角式 |
| 1985 | Shoemake | 四元数球面线性插值 |
| 1993 | Shuster | 姿态表示的系统综述 |
| 1994、2003 | Williams；Fukushima | 岁差章动的四角式，后为 IAU 2006 所用 |

## 八、本项目的取法

- **主计算链用转动矩阵**，采用坐标转动与 $R_1, R_2, R_3$ 的记法，与 SOFA、IERS 规范一致。岁差、章动、黄赤互换、参考架偏差各构造一个 3×3 矩阵，依次相乘后作用于日、月、行星的位置与速度。
- **岁差用三角式**（P03 的 $\zeta_A, \theta_A, z_A$），或 Vondrák 等 2011 年的长期岁差式，见天文篇《岁差》；与四角式在公元前 1000 年相差 1.25″，远小于古代观测精度。
- **四元数与对偶四元数**已在数学库中实现，乘法用哈密顿约定、标量在前，提供共轭、模长归一、由转动矩阵求四元数等运算；目前用于解释与动画，主计算链未用。尚未提供球面线性插值。

## 参考文献

1. L. Euler, "Formulae generales pro translatione quacunque corporum rigidorum", *Novi Commentarii academiae scientiarum Petropolitanae* 20 (1776), 189–207；Opera Omnia, ser. 2, vol. 9, 84–98（Eneström 编号 E478）。据 archive.org 所存原刊第 20 卷核第 24—25 节（第 201—202 页）。
2. L. Euler, "Problema algebraicum ob affectiones prorsus singulares memorabile", *Novi Commentarii academiae scientiarum Petropolitanae* 15 (1771), 75–106；Opera Omnia, ser. 1, vol. 6, 287–315（Eneström 编号 E407，1770 年写成）。据欧拉档案所存原刊扫描本核 §XIV—XIX（第 85—88 页）、§XXXIII—XXXIV（第 101—102 页）。
3. O. Rodrigues, "Des lois géométriques qui régissent les déplacements d'un système solide dans l'espace, et de la variation des coordonnées provenant de ces déplacements considérés indépendamment des causes qui peuvent les produire", *Journal de Mathématiques Pures et Appliquées*, 1re série, 5 (1840), 380–440。据 Numdam 数字化本核题目卷页。
4. W. R. Hamilton, "On quaternions; or on a new system of imaginaries in algebra", *Philosophical Magazine*, 3rd ser., 1844—1850 年分期连载（首期在第 25 卷）。据 D. R. Wilkins 所编录文核其刊载情况；卷页未逐期核对。
5. A. Cayley, "On certain results relating to quaternions", *Philosophical Magazine* 26 (1845), 141–145；重印于 *The Collected Mathematical Papers of Arthur Cayley*, vol. 1, Cambridge, 1889, no. 20, 123–126。据 archive.org 所存重印本核。
6. C. F. Gauss, "Mutationen des Raumes"（约 1819 年手稿），收入 *Werke* 第 8 卷（1900）。未核原书。
7. M. Chasles, "Note sur les propriétés générales du système de deux corps semblables entr'eux, et placés d'une manière quelconque dans l'espace; et sur le déplacement fini, ou infiniment petit, d'un corps solide libre", *Bulletin des Sciences Mathématiques, Astronomiques, Physiques et Chimiques*（费吕萨克编）14（1830 年 11 月号，题下注 1831 年 2 月 5 日宣读）, no. 198, 321–326。据 archive.org 所存本核。
8. G. Mozzi, *Discorso matematico sopra il rotamento momentaneo dei corpi*, Napoli, 1763。未核原书。
9. W. K. Clifford, "Preliminary sketch of biquaternions", *Proceedings of the London Mathematical Society* 4 (1873), nos. 64–65, 381–395；重印于 R. Tucker (ed.), *Mathematical Papers by William Kingdon Clifford*, London: Macmillan, 1882, 181–200。据 archive.org 所存重印本核。
10. E. Study, *Geometrie der Dynamen*, Leipzig, 1903。未核原书。
11. Yan-Bin Jia, *Dual Quaternion*，爱荷华州立大学 ComS 477/577 讲义。对偶四元数的定义与运算据此。
12. S. Newcomb, *A Compendium of Spherical Astronomy*, New York: Macmillan, 1906。未核原书。
13. J. H. Lieske, T. Lederle, W. Fricke, B. Morando, "Expressions for the precession quantities based upon the IAU (1976) system of astronomical constants", *Astronomy and Astrophysics* 58 (1977), 1–16。
14. N. Capitaine, P. T. Wallace, J. Chapront, "Expressions for IAU 2000 precession quantities", *Astronomy and Astrophysics* 412 (2003), 567–586。
15. J. G. Williams, "Contributions to the Earth's obliquity rate, precession, and nutation", *Astronomical Journal* 108 (1994), 711–724。据 ADS 扫描本核首页。
16. T. Fukushima, "A new precession formula", *Astronomical Journal* 126 (2003), 494–534。据 Crossref 核书目，doi:10.1086/375641。
17. K. Shoemake, "Animating rotation with quaternion curves", *Computer Graphics* (SIGGRAPH '85) 19(3) (1985), 245–254。据 Crossref 核书目，doi:10.1145/325165.325242。
18. M. D. Shuster, "A survey of attitude representations", *Journal of the Astronautical Sciences* 41(4) (1993), 439–517。未核原文。
19. H. Sommer, I. Gilitschenski, M. Bloesch, S. Weiss, R. Siegwart, J. Nieto, "Why and how to avoid the flipped quaternion multiplication", *Aerospace* 5(3) (2018), 72。据 Crossref 核书目，doi:10.3390/aerospace5030072。
20. IAU SOFA Board, *SOFA Tools for Earth Attitude*；G. Petit, B. Luzum (eds.), *IERS Conventions (2010)*, IERS Technical Note 36, §5。
21. W. R. Hamilton, 致 A. H. Hamilton 书，1865 年 8 月 5 日。据都柏林三一学院数学系 D. R. Wilkins 所编哈密顿文献录文。
