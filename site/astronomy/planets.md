# 行星：位置、星等、会合与伏见

本章说明行星视位置的计算与检验、视星等、会合现象（合、冲、留、大距）、近距事件、晨见夕伏的可见性模型，以及月掩星。文中方括号数字指文末参考文献。

## 行星视位置

| 内容 | 验证 |
| --- | --- |
| （水星—海王星）；DE406 行星系质心段 | — |
| VSOP87B 各行星加载与地心合成（日心行星 − 日心地球，光行时对地心矢量迭代，与太阳同一做法） | 见下 |
| VSOP87 → DE406 改正：各行星（含地球）日心 L、B、R，Chapront (2000) 方法 [25] | 见下 |
| 行星地心视位置（当日真赤道、视黄经黄纬，含距离） | 见下 |

**VSOP87 与 DE406 的差异**。VSOP87 的积分常数拟合自 DE200 [26]，其文档所称「日心 1″」（水星、金星、地月质心、火星在 J2000
前后 4000 年，木星、土星 2000 年，天王星、海王星 6000 年； PRECISION 节）是相对 DE200 而言。
Chapront (2000) 指出「其常数来自 DE200，拟合离 DE405 很远；时间跨度超过一两千年后解迅速变差」，并给出前 3000—3000 年
VSOP87 与 DE406 平黄经最大差：木星 138″、土星 232″、天王星 84″、海王星 130″（表 III）。本项目实测（
前 2000—2000 年每 125 年一点）与之同量级：

| 行星 | 日心方向差 原样 → 改正 | 地心视位置差 原样 → 改正 |
| --- | --- | --- |
| 水星 | 2.21″ → 0.10″ | 1.49″ → 0.19″ |
| 金星 | 3.15″ → 0.06″ | 1.49″ → 0.19″ |
| 火星 | 4.55″ → 0.04″ | 10.61″ → 0.14″ |
| 木星 | 37.1″ → 0.17″ | 42.1″ → 0.16″ |
| 土星 | 56.7″ → 0.39″ | 58.0″ → 0.36″ |
| 天王星 | 65.5″ → 0.08″ | 66.2″ → 0.09″ |
| 海王星 | 96.9″ → 0.06″ | 97.4″ → 0.06″ |

**改正方法**（Chapront 2000 [25]）：残差 ρ = σ_DE406 − σ_VSOP87 取 VSOP87B 自身的日心 L、B、R，近似为长期多项式加泊松混合项
Σ_{k≤NSEC} a_k t^k + Σ_f Σ_{k≤MIXT} t^k (C_k cos f t + S_k sin f t)，频率取该变量 VSOP87 级数中振幅最大的若干项；区间取论文 (a)
前 3000—3000 年（DE406 全程）；线性最小二乘，不加人为边界。DE406 转回 VSOP87 架所用旋转与 Rust 管线一致（R_x(ε₀ = 84381.406″)
+ SOFA FK5→ICRS 偏差），使改正与管线配套。论文说明 NSEC、MIXT「可因行星和变量而略有不同」：水星、金星、地球、天王星、海王星
用 (3, 2, 60 个频率)；火星、木星、土星用 (5, 4, 300)——用 (3, 2, 60) 时木星、土星残差仍有 8″、13″，按独立检验点（不参与拟合的
4000 个随机时刻）逐步增加到 (5, 4, 300) 后降到与论文表 III 同量级。拟合脚本的独立检验（前 3000—3000 年最大日心方向残差）：
水星 0.32″、金星 0.11″、地球 0.29″、火星 0.14″、木星 0.31″、土星 1.06″、天王星 0.40″、海王星 0.18″（论文表 III：0.005、0.011、
0.033、0.157、0.500、1.160、0.142、0.046″，为平黄经，口径不同）。

**取舍**：本地有 DE406 时以 DE406 为准；在网页中（不带 DE406）用 VSOP87 + 改正，与 DE406 差 < 0.4″，对史料（角分级）绰绰有余。
地球改正同时用于节气、定朔与天象（缺省 DE406 版，可选 DE441 版）；原地心太阳赤道补丁与黄道补丁已移除；网页端只加载地球一段（约 10 KB）。

**外部核对：Skyfield**。Skyfield [27] 独立实现 IAU 视位置算法，读同一
de406.bsp 计算 7 颗行星在 5 个时刻（2000、2023、约 1000、约 100、约前 1000 年）的当日真赤道视位置（`scripts/ephemeris/
skyfield_planet_apparent.py`，用 uv 临时环境运行，不属本项目依赖），与本管线逐点比较：
- 公元 100 年以来 28 点，差 ≤ 0.07″（Skyfield 另计太阳引力光线偏折，约 0.004″·cot(ψ/2)，本管线未计）；
- 前 1000 年 7 点，差 0.27–1.20″，即 IAU 2006 岁差三角式（本管线）与四角式（Skyfield）两种截断多项式在远端的分歧（1.25″）。

**由此发现并修正岁差 bug**：本项目 P03 岁差矩阵中 ζ_A 与 z_A 位置对调，修正前与 Skyfield 差：公元 1000 年 7″、公元 100 年
51″、前 1000 年 196″。误差主要在黄纬、赤纬方向，黄经只受二阶影响，**节气、定朔不受影响**；
「本实现」。修正后全部库测试通过；《汉书》《后汉书》日月食检验已重跑核对（见下）。

**尚未做**：太阳引力光线偏折（行星远离太阳时毫角秒级，近合日可达角秒级）。

**行星视星等**
按 Mallama & Hilton (2018) [15] 式 2—17（V 波段，《天文年历》现行算法），系数核对过论文原文（arXiv:1808.01973；式 13、16 为图片，
渲染核对）；作者程序 Ap_Mag V3 与官方测试数据原件。
- 分支与作者程序一致：相位角超出观测范围时照算（`within_observed_phase_limits` 供调用方提示），只有土星含环超出 α ≤ 6.5°、
  β ≤ 27°，或海王星 2000 年前 α > 1.9° 时不给星等。
- 唯一系数出入：土星式 12 三次项论文正文作 −1.505×10⁻⁶，作者程序为 −1.506×10⁻⁶；官方测试输出（α = 169.9° 时 5.206）只与后者
  相符（前者 5.211），采用后者。
- 火星自转修正 L(λe)、季节修正 L(Ls)：表照录作者程序（Mallama 2007 表 6、表 8，10° 间隔），Stirling 四阶插值。作者程序差分
  循环少算 δ(3)（四阶差分用到未赋值项），本项目按完整公式实现，官方用例仍在 0.001 内。由历表算星等时只计 L(Ls)；L(λe) 需火星
  直下点经度（IAU 火星自转模型，未接入），记为 0（幅度约 ±0.06）。
- 几何：行星取光行时修正后地心位置（ICRS），太阳取地心几何位置；土星、天王星自转轴取 IAU J2000 值（α₀, δ₀ = 40.589°, 83.537°；
  257.311°, −15.175°）[28]，天王星直下点纬度按式 13 换算为行星面图纬度（f = 0.0022927）；火星日心黄经取 J2000 黄道。
- 验证：① Ap_Mag V3 官方测试 27 例（直接解析原件），全部 ≤ 0.001 星等，含火星两项修正与土星不给星等一例；② 同读 DE406 与
  Skyfield 对照 28 例（4 个时刻 × 7 行星），差 ≤ 0.001 星等（Skyfield 自身的 `planetary_magnitude` 把太阳当作位于太阳系质心，
  近日行星有 0.01—0.04 星等之差，对照时改用真太阳几何；火星扣除 Skyfield 未计的 L(Ls)）。
- 已知省略：火星 L(λe)（见上）；土星、天王星极轴的长期变化（土星到前 200 年约 0.8°，对星等 ≤ 0.02）。

**行星会合现象**：合（内行星分上合、下合）、冲、留
（转逆、转顺）、大距（东、西）。定义取《天文年历》说明书与 Meeus 通行定义：地心视位置，黄经取当日真黄道（含章动）；合、冲为
与太阳视黄经差 0°、180°；留为视黄经变化率为零；大距为内行星与太阳角距极大。取样（水星 1 日、余 2 日）找变号或局部极大，
再二分求精（大距自 改为求角距五点差分导数之根，见下）。日与行星同一提供者、同一管线（新增 ）。
- 验证：对照 Skyfield（同读 DE406、同用 P03 岁差，2020—2025 年，241 个事件；合、冲取其 `almanac.oppositions_conjunctions`，
  留、大距用其 `find_discrete` / `find_maxima` 独立求得）：事件个数逐类一致；
  合、冲时刻差 ≤ 0.3 s；大距 ≤ 0.3 s；留 ≤ 4.4 min（海王星；留附近黄经变化率极平缓，
  两边黄经约 0.01″ 的差即使留的时刻差数分钟）（容差合冲大距 2 s、留 10 min）。
- **极值时刻的数值问题**：大距原以黄金分割求精，比较两点函数值；极大附近 10 s 内角距只变约 1e-12 rad，
  函数的数值噪声即可使比较出错。实测（金星 2020-03-24 大距前后每 0.5 s 取样）：
  - DE406 读取器把时刻先折成「距段起点的秒数」（段起点在前 3000 年，约 1.6e11 s），f64 只到约 3e-5 s，行星位置台阶约 0.76 m、
    方向噪声 2.5e-12 rad，大距时刻抖动约 ±30 s；double-double 构建亦然。改为 (hi − 记录起点) + lo（
    时刻由 double-double 拆两段传入）后噪声降到 4e-15 rad，与 VSOP87 相同。DE406 位置因此改变 ≤ 约 1 m（时刻误差 ≤ 3e-5 s）。
  - f64 构建的儒略日本身只到约 40 µs，各历表方向均带约 3e-12 rad 噪声；改以五点差分导数（h = 30 min）求根
    ，噪声折合约 0.2 s。近距事件的最接近时刻同法。
  - 修正后两种构建的大距时刻差 ≤ 0.2 s，近距逐格相同；与 Skyfield 大距差由 31 s 降到 0.3 s。

**近距事件**：两天体（日、月、行星）地心视方向
角距（当日真赤道下大圆距离）的各次极小，给出时刻与最小角距；「犯」「守」等史书术语的阈值留给下游。取样（有月 0.2 日、否则 1 日）
找局部极小，再黄金分割求精。新增组合提供者 （VSOP87 日与行星 + ELPMPP02 月），DE406/DE441 内核本身即含全部天体。
- 验证：① 对照 Skyfield（同读 DE406、P03，
  `find_minima`）：木—土、金—木、火—木、水—金（2020—2023）与月—金、月—木、月—火（2023），角距 ≤ 5° 的极小共 49 次，次数逐对
  一致，时刻差 ≤ 0.8 s，最小角距差 ≤ 0.007″（2020-12-21 木土大合最小角距 0.1018°）；② VSOP87（DE406 版改正）+ ELPMPP02 对 DE406（月的配对取
  2023 年前 120 日，共 28 次）：时刻差 ≤ 3.3 s，最小角距差 ≤ 0.025″。

**晨见夕伏**：按原文逐式独立实现，未参照任何现成程序（Swiss Ephemeris `swehel.c` 为 AGPL，不引入代码）。
所据文献 16、29–39。各环节可选项并存，缺省由下面的检验结果选定：
**缺省为 Schaefer 1998 整套**（文献 37，作者最新版，所附程序 VISLIMIT.BAS）。

- （纯公式）：
  - **Schaefer 1998 整套（缺省）**：按 VISLIMIT.BAS 逐行实现（V 波段；消光五波段）。
    - 消光 `Extinction1998`：气溶胶 0.1(λ/0.55)^{−1.3}e^{−H/1.5}(1 − 0.32/ln S)^{1.33}(1 + sgn φ·sin α_S)，季节项随半球反号；
      加水汽项 0.94·w_λ·S·e^{T/15}e^{−H/8.2}；臭氧、Rayleigh 同 1993。
    - 天空 ：大气质量用 Rozenberg 式。月光、白昼的散射三项相加，另加多次散射项
      440000(1 − 10^{−0.4kX_源})。蒙影 10^{−0.4(M_S − M_O + 32.5 − h_S − Z/(360k))}·(100/ρ_S)·(1 − 10^{−0.4k})，
      含与太阳角距 ρ_S，向日一侧更亮——这正是 1993 式 15c 方向因子存疑之处，1998 版已改写。夜天光 V 波段约 90 nL，
      原式含太阳活动周因子 1 + 0.3cos(2π(年 − 1992)/11)，古代相位未知，缺省取平均。
    - 阈值 ：Hecht 两支（1500 nL 分界），m = −16.57 − 2.5 lg I − Δm + 5 lg SN，
      不含 1990 的颜色修正与暗视消光 0.48。
    - 实现细节：程序以月份近似太阳赤经（(月 − 3)·30°），此处用实际太阳赤经；纬度 0 时程序除以 0，此处取北半球。
      蒙影项末因子按原文不含 X。
  - 1993 整套与其它选项（、 等）：
  - 大气质量：1993 式 2b（Rozenberg，整层）、3a（指数分布，Rayleigh 8.2 km、气溶胶 1.5 km）、3b（20 km 臭氧层）。
  - 消光：1993 式 4b、5a,b、6a 分量估算，或给定实测天顶总消光
    （原文建议有当地实测时优先用；Schaefer 2000 估古埃及夏季 0.35 ± 0.09）。
  - 天空亮度：
    - 散射相函数与月光取 Krisciunas & Schaefer 1991（文献 33）原式，分段 Mie 项（ρ < 10° 用晕项 6.2×10⁷ρ⁻²）。
      1993 式 14 把三项直接相加，作选项 （30° 处大约 10%，5° 处大约 36%）。
    - 式 15 中的大气质量取 Krisciunas & Schaefer 1991 式 3：X = (1 − 0.96 sin²Z)^{−1/2}。1993 式 15b、15d 即该文式 15
      （15d 以太阳星等 −26.74 代月亮），1993 只是转述。此前误用 Rozenberg 式 2b，太阳在地平下时 B_day 过小、min 会错选；
      改用原式后自然取蒙影项，原先的权宜处理随之去掉。
    - 晨昏蒙影有两种（`TwilightModel`）：
      - 1993 式 15c（缺省）；
      - Schaefer 1987（文献 32）式 1–2：含与太阳方位差、对 −18° < h₀ < −6° 实测拟合。原文式 2 的夜天光项印作
        (10^Y + 1)²/K，按原文常数 118 mμL 反推应为 (10^Y − 1)²/K，此处夜天光改由 B₀ 给出。
    - 1993 式 15c 的方向因子 max{1, dex[(ρ_S/90°) − 1.1]} 按原文印刷（已放大核对版面），只在背日方向（ρ_S > 99°）增亮，
      不影响晨见、夕伏；保留开关 `TwilightDirectionTerm`。
  - 阈值（`ThresholdModel`）：
    - `Schaefer1993`（缺省）：1993 式 34（Hecht 1947），观测者修正取 1990 式 13–16（暗视颜色 F_c、敏锐度 F_s）与 1993 式 31
      （双眼 F_b、消光 F_e：明视 10^{0.4kX}、暗视 10^{0.48kX}）。
    - `Crumey2014 { field_factor }`（文献 34）：式 34 与 Blackwell 1946 表 8 系数（式 28），颜色修正式 16
      m* = m₂₈₅₀ + 0.72 − 0.27(B−V)，单眼 ×√2 乘在阈值上（§1.6.4）。Crumey 指出 Schaefer 1990 的颜色修正
      （1 − (B−V)/2）与单眼因子用法有误。
    - 实现细节：Crumey 式 34 在 lg B ≈ −1.7…−1.3 cd m⁻² 有约 2.5% 的回落平台，是拟合函数本身的形状。原文只给暗视的颜色修正，
      此处在其两支分界 7.08×10⁻² cd m⁻² 处切换，极限星等在此跳变约 0.6 等。
  - 眩光 `glare_nl`（Schaefer 1991，文献 38）：大气散射晕 1.14×10⁷I*θ⁻²(10^{−0.4kX} − 10^{−0.8kX})/0.18（式 1），
    眼内散射 4.63×10⁷Iθ⁻²（式 3，Holladay 1926）。前者与月光模型中的晕项同源，故伏见中只对月亮加眼内散射
    （缺省开；月亮按月心点源，角距下限 0.26°）。原文以 161 次月旁观测检验，误差约 0.35 等。
  - 夜天光可由观测者天顶极限星等反推（`night_sky_from_zenith_limit`，同一阈值模型），Schaefer 1987 即以 m_z 刻画天空。
  - 行星色指数 B−V 取 Mallama et al. 2017 表 3（文献 31）：水 0.97、金 0.70、火 1.36、木 0.86、土 1.07、天 0.50、海 0.39。
-
  - 用法：先建 `GeocentricTracks`（日、月，与观测者无关）与 `PreparedTarget`（行星、固定星等的行星或星表恒星），再建
    ，调用 `.events(&target, jd0, jd1)` 或 `.daily_visibility(...)`；
    单个天体也可用 `heliacal_events`。
  - 判定窗口：在晨（昏）蒙影时段内（太阳几何高度在 −18° 与 −50′ 之间，`twilight_limit_deg` 可调），若某时刻天体视高度 > 0
    （缺省光线追迹折射，大气取观测者海拔的标准状态，经 `refraction_from_geometric` 由几何高度求，地平下连续）且星等不暗于该方向极限星等，则当日晨（昏）可见。可见与否翻转之日即为事件（晨见、晨伏、夕见、夕伏），
    输出可见裕度最大时刻的高度、太阳高度、方位差、星等、极限星等与视弧。
  - 外行星与恒星的夕见、晨伏即 acronychal rising、cosmical setting，原文（1987 §4、1993 §3.10）指出定义含糊，其日期依赖蒙影下限。
  - 日、月、天体视位置与星等按 1 日取样、四点 Lagrange 插值（月插值误差 < 0.01°）。GAST 每半日算一次，其间按自转角速率外推。
    用 DE441 时 2 年 5 行星约 40 s；用 VSOP87+ELPMPP02 时月历表较慢（2 年日月取样约 4 分钟），两者日期一致。
- 恒星（P4 地基）：
  - Hipparcos 亮星表 （V ≤ 6.5，8870 颗）。位置、视差、自行取新归算
    I/311（文献 17），V、B−V 取原版 I/239；。
  - 空间运动（视向速度取 0）→ 周年视差 → 一阶周年光行差 → 当日真赤道。
  - 对 Skyfield（DE406、P03，；5 星 × 1985、2000、2025、−500、−900 年）：
    最大差 0.08″，主要来自未计的太阳引力偏折与以日心代质心。
  - 核对中发现 VSOP87 解析速度的原有缺陷：α = 0 块（含中心差等主要周期项）的导数被当作 0，地球速度因此小约 3.3%
    （表现为恒星光行差差 0.6″）。已修正， 对数值微分检验。
    默认路径（视黄经速度用数值微分、光行差用 Xproper 折叠）不受影响；受影响的是 `use_analytic_velocity` 与
    `earth_velocity_ecliptic_j2000_bcrs` 两个可选路径。
- 验证（模型公式）：
  - 大气质量对 1993 表 3（4 种 × 7 个天顶距，≤5%）。
  - 月光对 Krisciunas & Schaefer 1991 表 2：20 个值逐个复现（≤0.5 nL）。
  - 天顶极限星等对 1990 式 18 及其算例（136 nL、k = 0.3 → 6.05，差 < 0.01 等）。
  - Crumey 暗夜极限对其 §3.1 的 6.93 − 2.5 lg F 与式 54 线性近似。
  - 天空亮度对 1993 表 6 数量级；明视阈值对表 5。
  - 1987 式 2 常数 118 mμL 的复算。
  - **1998 原文算例逐项复现**（`schaefer1998_worked_example`）：天顶距 45°、2 月、30°N、海拔 1000 m、湿度 40%、15 ℃；
    UBVRI 消光系数 0.62/0.32/0.19/0.09/0.05、总消光 0.87/0.46/0.27/0.13/0.07、大气质量 1.41、V 天空 6.8×10⁻¹⁴（61 nL）、
    极限星等 6.29，全部在 0.006 以内。
  - 眩光：Schaefer 1991 §3.2 伽利略卫星（木星 −2.6 等、136 nL、kX = 0.2）：F_s = 1 时四颗都不可见、F_s = 0.15 时大距都可见，
    且眼内散射为主（`glare_galilean_satellites_schaefer1991`）。
- 验证（实测与作者模型）：
  - **Schaefer 1987 表 1 的 45 次实测**（
    ）。按每条记录的纬度、k、m_z 判定观测日能否看见，与实测比对；「干净」指去掉黄道光、
    银河干扰与原文归因于低云的 ?? 后的 25 条：

    | 组合 | 一致 / 45 | 一致 / 干净 25 | 模型伏见日 − 作者 1987 预报（均值，\|中位\|） |
    |---|---|---|---|
    | **Schaefer 1998 整套（缺省）** | 28 | 20 | +1.4，1.0 日 |
    | Hecht + 1993 蒙影 | 30 | 19 | +3.4，3.1 日 |
    | Hecht + 1987 蒙影 | 31 | 20 | +3.0，3.1 日 |
    | Crumey F = 2 + 1993 蒙影 | 17 | 10 | +1.0，5.1 日 |
    | Crumey F = 4 + 1993 蒙影 | 25 | 16 | +1.8，2.1 日 |
    | Crumey F = 6 + 1993 蒙影 | 29 | 19 | +1.1，3.0 日 |
    | Crumey F = 8 + 1993 蒙影 | 30 | 19 | +3.6，3.9 日 |

    Crumey 模型取原文所称实际观测的 F（1.4–2.4）时，在蒙影亮背景下明显偏乐观，要 F ≈ 6–8 才与 Hecht 相当。可能是 Blackwell
    的实验室阈值（不限观察时间）不宜直接用于蒙影，或与天空模型的定标不配套，原文均无法判定，故只作选项。两种蒙影公式打平。
  - **Schaefer 1987 表 7**（`schaefer1987_table7_venus_yucatan`，作者 1987 模型对尤卡坦金星的计算值，m = −3.4、k = 0.30、
    m_z = 6.0，1977—1984 五个会合周期均值）：

    | 组合 | 下合伏 | 晨星 | 上合伏 | 昏星 |
    |---|---|---|---|---|
    | 表 7 | 11.2 | 251.6 | 69.4 | 250 |
    | **Schaefer 1998 整套** | 11.9 | 251.4 | 69.3 | 250.3 |
    | Hecht + 1987 蒙影 | 12.7 | 249.6 | 72.1 | 248.5 |
    | Hecht + 1993 蒙影 | 9.7 | 259.8 | 54.9 | 258.5 |

    1987 组合复现作者 1987 模型（±3 日），说明 1987 式实现无误。1993 式 15c 近日方向天空比 1987 式暗约 3 倍，上合伏短 2 周。
  - **Schaefer 2000 表 1**（`schaefer2000_sirius_egypt`，文献 35：埃及 30°N、k = 0.35 的天狼星偕日升，作者以其 1998 程序计算，
    春分后日数）：1998 整套与表值差 −0.7 至 +1.2 日（均值 +0.3）；Hecht + 1993 蒙影均值 +0.6，视弧约 10.4°（原文「约 11°」）；
    Hecht + 1987 蒙影均值 +1.5 日。作者 2000 年即以 1998 子程序加伏见程序计算，1998 整套最接近，与此相符。
  - **伏见与合日次序**（`heliacal_brackets_conjunctions`，DE441，巴比伦，2025—2026）：每次合（外行星）、下合、上合的前一事件为
    相应的伏、后一事件为相应的见，事件时太阳在蒙影内、行星在地平上且可见。合日时刻取自已对 Skyfield 验证的 `planet_events`。
    水星中纬度春晨、秋昏整次不可见属实（如 2025 年春晨），测试允许。
- 缺省的取舍：取 Schaefer 1998 整套。
  - 它是作者最新版；原文算例逐项复现。
  - 复现作者 1987 金星表（±0.7 日）与 2000 天狼星表（均值 +0.3 日）最好，与作者 1987 预报日最接近（|中位| 1.0 日）。
  - 在实测「干净」25 条上与最好的组合并列（20/25）。
  - 1993 式 15c 方向因子的疑点在 1998 版已由 100/ρ_S 改写。Crumey 阈值（F 取原文范围时）实测一致数远低，只作选项。
- 消光的影响最大（与原文强调一致），巴比伦 2025—2026、1998 整套的视弧：

  | 天顶消光 k | 金星 | 木星 | 土星 | 火星 | 火星 2026-01 合日前后不可见 |
  |---|---|---|---|---|---|
  | 缺省估算（湿度 50%、15 ℃，夏季 k ≈ 0.5） | 6—8.5° | ≈ 13° | 15—18° | 18—21° | ≈ 262 日 |
  | 0.35 | 7—8.5° | ≈ 11° | ≈ 16.5° | ≈ 17.5° | ≈ 246 日 |
  | 0.25 | 5.5—7° | ≈ 9° | ≈ 14° | ≈ 15° | ≈ 213 日 |
  | 0.15 | 4—5.5° | ≈ 7.5° | ≈ 11.5° | ≈ 12° | ≈ 170 日 |

  1998 消光估算含水汽项，海平面湿润时 k 偏大；古代史料对照时宜按当地气候给定 k（1993 表 4：各台址年均 0.11—0.47；
  Schaefer 2000 估古埃及夏季 0.35 ± 0.09）。传统视弧（1987 §4 引）：Ptolemy 金星 5°；中国历法金星「见伏度」均值 10.5°，
  乘 cos 35° 合视弧 8.6°；巴比伦金星约 6°（Schoch、Neugebauer、Huber）；Ptolemy 木星 9°—10°、水星 10°—12°（2000 §3）。
- Ruggles 1993 论文集（文献 36）的核对：第 15 章 Schaefer 综述无新公式，伏见部分概括文献 32、16；作者称蒙影结束取太阳 −18°
  与其观测吻合（本实现 `twilight_limit_deg` 缺省 −18°），并称个体视觉差异通常小于其他不确定度。第 12 章（Ecsedy & Barlai）
  引 Ecsedy et al. 1989：北京纬度心宿二（大火）偕日落与秋分重合约在前 2300 年（方法未详），可作 P4 恒星伏见的参照。
- 未做：
  - 与 Swiss Ephemeris 的数值对照：需安装其 Python 包，本次未执行；
  - 行星作眩光源（如金星旁的暗星）尚未接入伏见；观测者经验修正；黄道光与银河背景（1987 表 1 注 ZL、MW）。

**月掩星**

- 几何：
  - 月与天体的站心视位置（当日真赤道）；月面视半径 asin(k·a_e/d)，k = 0.2725076、a_e = 6378137 m（同日食半影 k）。
    不计月面地形，实际边缘起伏 ±2″，时刻相应差约 ±4 s。
  - 入掩、出掩为天体中心与月心站心角距等于月面视半径的时刻，二分到 0.1 s。另给出接触点位置角、亮边或暗边、尖角、
    月日高度、月相角与亮面比例。
  - 搜索：先逐 10 分钟用插值位置粗筛，再在候选附近精算。
- 验证：对 Skyfield（DE441）逐次对照。
  北京，2023—2026，轩辕十四、角宿一、心宿二、毕宿五、昴宿六与金、火、木、土，共 36 次，次数一致，入掩、出掩时刻差 ≤ 0.50 s。
- 肉眼可见性（`occultation_visibility`）：依 Schaefer, Bulder & Bourgeois 1992（文献 39），取肉眼情形（M = 1、无衍射与镜面散射、
  f_FOV = 1）：
  - 亮边：背景 = 月面 5.67×10¹⁰ I/S + 天空。
  - 暗边：背景 = 地照 + 天空 + 月光眩光（大气式 10 + 眼内式 13，有效角距式 11）。
  - 地照取 1993 式 13 = 1992 式 8、9。1992 式 9 的系数 78 只在月面反照率取 0.067 时成立，原文印作 0.67，为排印错误；
    1993 文明言 0.067，两式由此互证。
  - 式 46 的计时置信度偏移 ΔM（原文拟合望远镜计时 1.2 等）作为参数，缺省 0（50% 探测）。
- 验证（可见性）：
  - 满月面 1.65×10⁹ nL（原文）、地照两式一致。
  - 眩光对 Schaefer 1991 图 1：紧靠满月极限约 0 等、上弦约 2 等（本实现 0.4、2.2 等）。
  - 夜间暗边比亮边易见、亮面越小越易见、满月亮边肉眼只见 −4 等以上。
  - 原文 1739 次观测为望远镜计时，未逐条发表（只有表 I 观测者、仪器汇总与表 II 分相位汇总），故无法逐条对照；望远镜模型见下。
- 望远镜目视计时极限星等（`telescope_occultation_limit`）：按原文式 1–46 全套实现，输入为原文 15 项及 ΔM。
  - 背景：亮边为月面 + 天空；暗边为地照 + 天空 + 四种眩光（大气式 10、眼内式 13、衍射式 14、镜面式 15，有效角距式 11）。
  - 望远镜因子（式 25–35）、眼瞳（式 36–37，随年龄与背景）、临界视角（式 38–39）、视宁度圆面（式 40）、阈值（式 41–42）、
    经验与置信度（式 46）。眼感背景、瞳孔、临界视角相互依赖，迭代求解。
  - 原文需解释之处：
    1. 式 24 写作 B_eff = B·F_B，按 Schaefer 1990 式 14、15 的约定「实际量 = 眼感量 × F」，眼感背景应为 B ÷ F_B。
       照字面相乘，标准情形得 4.85 等，与原文 8.61 等相去甚远；相除则得 8.78 等。
    2. 式 30 印作 F_p = max(1, [D_s/(M·D_e)]²)，如此恒为 1。按 1990、1991 年原文的 [D/(M·D_e)]²（出瞳比眼瞳）实现。
    3. 式 33 中 Stiles–Crawford 常数 0.026、12.44 以毫米计（同 1990 年原文），瞳孔与口径换算为毫米代入。
    4. 式 42b 印作「log(B) > 3.17」，应为「< 3.17」（暗视）。
  - 验证（`telescope_model_vs_schaefer1992`，逐条见附录「天象逐条校验」）：
    - 标准情形 8.78 等对原文 8.61 等（+0.17）；
    - 图 1 读图值五例差 −0.37 至 +0.25；
    - §4.A Povenmire 四例差 −0.18 至 +0.23（其中两例原文未写明亮面比例与太阳高度，按假设代入）；
    - 作者自述模型精度约 1/3 等。

## 参考文献

**ΔT 与地球自转**

1. Morrison L.V., Stephenson F.R., Hohenkerk C.Y., Zawilski M. 2021, Addendum 2020 to "Measurement of the Earth's rotation: 720 BC
   to AD 2015", *Proc. R. Soc. A* 477: 20200776. <https://doi.org/10.1098/rspa.2020.0776>
2. Stephenson F.R., Morrison L.V., Hohenkerk C.Y. 2016, Measurement of the Earth's rotation: 720 BC to AD 2015, *Proc. R. Soc. A*
   472: 20160404. <https://doi.org/10.1098/rspa.2016.0404>（开放全文 <https://pmc.ncbi.nlm.nih.gov/articles/PMC5247521/>；§2 所述
   ṅ = −25.82″/cy² 与 DE430 配合使用，式 (4.1) 长期抛物线）
3. Morrison, Stephenson, Hohenkerk, Zawilski, Table-S15.2020 – Polynomial coefficients to evaluate DT and change in lod, figshare,
   CC BY 4.0. <https://doi.org/10.6084/m9.figshare.29920388>
4. 同上作者，Table of Delta T (DT) & change in the length of day (lod) from −2000 to +2500 (2020), figshare, CC BY 4.0.
   <https://doi.org/10.6084/m9.figshare.30111661>
5. HM Nautical Almanac Office, ΔT 数据说明文件 hmnao_a_2021.txt（IERS EOP Product Center 托管，所写 ṅ = −25.85″/cy²）.
   <https://hpiers.obspm.fr/eoppc/series/longterm/hmnao_a_2021.txt>
6. Espenak F., Meeus J. 2006, *Five Millennium Canon of Solar Eclipses: −1999 to +3000*, NASA/TP-2006-214141；ΔT 多项式说明
   （所假定 ṅ = −26″/cy²，配合 ELP-2000/82 的修正 c = −0.000012932·(y−1955)²）. <https://eclipse.gsfc.nasa.gov/SEcat5/deltatpoly.html>
7. Petit G., Luzum B. (eds.) 2010, *IERS Conventions (2010)*, IERS Technical Note No. 36（第 5 章：ERA、GMST、二分差互补项
   表 5.2e、Delaunay 基本幅角）.
8. IAU SOFA Board, *IAU SOFA Software Collection*（`iauEra00`、`iauGmst06`、`iauEe06a`、`iauGst06a`、`iauGd2gc` 及测试程序
   t_sofa_c.c 的标准值）. <https://www.iausofa.org/>

**月历表**

9. Chapront J., Francou G. 2003, The lunar theory ELP revisited. Introduction of new planetary perturbations, *A&A* 404, 735；
   仓库内 （§4.1.1 W₁⁽²⁾ 潮汐部分 = ṅ/2，表 3 ΔW₁⁽²⁾，表 6 长期项修正）.
10. Chapront J., Chapront-Touzé M., Francou G. 2002, A new determination of lunar orbital parameters, precession constant and tidal
    acceleration from LLR measurements, *A&A* 387, 700（ELP/MPP02(LLR) 的 ṅ = −25.858″/cy²）.
11. Park R.S., Folkner W.M., Williams J.G., Boggs D.H. 2021, The JPL Planetary and Lunar Ephemerides DE440 and DE441, *AJ* 161, 105.
    <https://doi.org/10.3847/1538-3881/abd414>

**算法与标准**

12. Urban S.E., Seidelmann P.K. (eds.) 2013, *Explanatory Supplement to the Astronomical Almanac*, 3rd ed., University Science
    Books（第 7 章站心视差，第 11 章日月食）.
13. Meeus J. 1998, *Astronomical Algorithms*, 2nd ed., Willmann-Bell（例 13.b 地平坐标；第 49 章真朔）.
14. Vondrák J., Capitaine N., Wallace P. 2011, New precession expressions, valid for long time intervals, *A&A* 534, A22.
15. Mallama A., Hilton J.L. 2018, Computing apparent planetary magnitudes for The Astronomical Almanac, *Astronomy and Computing* 25, 10；
    arXiv:1808.01973；作者程序与测试数据 <https://sourceforge.net/projects/planetary-magnitudes/>（Ap_Mag V3）.
16. Schaefer B.E. 1993, Astronomy and the limits of vision, *Vistas in Astronomy* 36, 311–361. doi:10.1016/0083-6656(93)90113-X；
    仓库内 （式 2–16、34、表 3–6、§3.10）.
17. van Leeuwen F. 2007, Validation of the new Hipparcos reduction, *A&A* 474, 653. doi:10.1051/0004-6361:20078357；
    星表 CDS I/311（原版 ESA 1997 SP-1200，CDS I/239），见 .
18. Espenak F., Meeus J. 2009, *Five Millennium Canon of Lunar Eclipses: −1999 to +3000*, NASA/TP-2009-214172（月食地影按
    Danjon 规则放大；Danjon 原文出处待核后补）.

20. Espenak F., Meeus J., NASA 五千年日月食典目录页（日食 −299～300、2001～2100，月食 −199～−100、1～200、2001～2100），
    NASA GSFC Eclipse Web Site，公有领域），如 <https://eclipse.gsfc.nasa.gov/SEcat5/SE2001-2100.html>.
21. Espenak F., NASA 月食典地影说明页（Danjon 地影放大规则）. <https://eclipse.gsfc.nasa.gov/LEcat5/shadow.html>

22. 班固《汉书》卷二十七下之下《五行志》，维基文库 wikitext（`action=raw`，取）.
    <https://zh.wikisource.org/wiki/漢書/卷027>

23. 司马彪《续汉书·五行志六》（今本《后汉书》志第十八，刘昭注），维基文库 wikitext（取）.
    <https://zh.wikisource.org/wiki/後漢書/卷108>
24. 司马彪《续汉书·律历志中》「论月食」（今本《后汉书》志第二），维基文库 wikitext（取）.
    <https://zh.wikisource.org/wiki/後漢書/卷92>

25. Chapront J. 2000, Improvements of planetary theories over 6000 years, *Celestial Mechanics and Dynamical Astronomy* 78, 75–82；
    仓库内 （表 II、III；§4 拟合方法，NSEC、MIXT）.
26. Bretagnon P., Francou G. 1988, Planetary theories in rectangular and spherical variables: VSOP87 solution, *A&A* 202, 309；
    仓库内 、`vsop87.doc`（PRECISION 节）.

27. Rhodes B. 2019, Skyfield: High precision research-grade positions for planets and Earth satellites generator,
    Astrophysics Source Code Library, ascl:1907.024. <https://rhodesmill.org/skyfield/>（岁差 `precessionlib.compute_precession`：
    Capitaine et al. 2003 四角式）

28. Archinal B.A. et al. 2018, Report of the IAU Working Group on Cartographic Coordinates and Rotational Elements: 2015,
    *Celestial Mechanics and Dynamical Astronomy* 130, 22（土星、天王星自转轴 α₀、δ₀ 的 J2000 值；与 Skyfield 极向量核对）.

29. Schaefer B.E. 1990, Telescopic limiting magnitudes, *PASP* 102, 212–229. doi:10.1086/132629
    （式 13 颜色修正、14–16、18 天顶极限星等）.
30. Schaefer B.E., Liller W. 1990, Refraction near the horizon, *PASP* 102, 796–805. doi:10.1086/132705
    （近地平折射的逆温变化，本实现未用）.
31. Mallama A., Krobusek B., Pavlov H. 2017, Comprehensive wide-band magnitudes and albedos for the planets, with applications to
    exo-planets and Planet Nine, *Icarus* 282, 19–33；arXiv:1609.05048）.
32. Schaefer B.E. 1987, Heliacal rise phenomena, *Journal for the History of Astronomy* 18 (Archaeoastronomy 11), S19–S33.
    doi:10.1177/002182868701801103
    （式 1–2 蒙影天空、表 1 实测、表 7 金星、§4 传统视弧）.
33. Krisciunas K., Schaefer B.E. 1991, A model of the brightness of moonlight, *PASP* 103, 1033–1039. doi:10.1086/132921；
    仓库内 （式 3、8、9、15–21，表 2）.
34. Crumey A. 2014, Human contrast threshold and astronomical visibility, *MNRAS* 442, 2600–2619. doi:10.1093/mnras/stu992；
    仓库内 （式 16、26–28、34、53–56；§1.3、§1.6.4）.
35. Schaefer B.E. 2000, The heliacal rise of Sirius and ancient Egyptian chronology, *Journal for the History of Astronomy* 31,
    149–155. doi:10.1177/002182860003100204）.

36. Ruggles C.L.N. (ed.) 1993, *Archaeoastronomy in the 1990s*, Group D Publications（电子版 Ocarina Books 2014，
    ISBN 978-0-9540867-4-9）
    Ecsedy & Barlai 第 123–127 页；第 15 章 Schaefer, Basic research in astronomy and its applications to archaeoastronomy，
    第 155–177 页）.

37. Schaefer B.E. 1998, To the visual limits, *Sky & Telescope* 95(5), 57–60（附程序 VISLIMIT.BAS）
    （程序 10–2460 行、原文算例）.
38. Schaefer B.E. 1991, Glare and celestial visibility, *PASP* 103, 645–660. doi:10.1086/132865
    （式 1–4、21–23，§3.1–3.2）.
39. Schaefer B.E., Bulder H.J.J., Bourgeois J. 1992, Lunar occultation visibility, *Icarus* 100, 60–72.
    doi:10.1016/0019-1035(92)90018-3

40. Stellarium 中国星空文化（skycultures/chinese、chinese_chenzhuo、chinese_song_dynasty、chinese_yuan_dynasty、
    chinese_xianglin），commit 8e3062e，CC BY-SA 4.0。所据：潘鼐《中国恒星观测史》
    （学林出版社 2009）、伊世同《中西对照恒星图表 1950.0》、杨伯顺对陈卓星表与《天文汇钞》的研究等（见各 description.md）.
41. 宋濂等《元史》卷五十二《历志一·授时历议》（「岁差」「周天列宿度」「日躔」）、卷五十四《历志三·授时历经》（「赤道宿度」），
    维基文库 wikitext（、，取）；班固《汉书》卷二十一《律历志》.

42. 潘鼐《中国恒星观测史》（增订版），学林出版社，2009
    （扫描本，PDF 页 = 书页 + 15；第 14 页表 1.2.1，第 639–641 页及表 9.1.4）.
43. 黄一农《星占、事应与伪造天象——以「荧惑守心」为例》。题名据俄文书目转引，刊物、年份、页码**待核**，原文未取得
    ；第十节西晋「荧惑守心」不合天象的结论由本模块推算独立得出，只引此文作背景.

**参照实现**

19. 许剑伟《寿星天文历》5.10（sxwnl）. <https://github.com/sxwnl/sxwnl>；Gitee <https://gitee.com/sxwnl>；
    署名与授权说明参见其移植版 <https://pub.dev/packages/sxwnl_spa_dart>.

> 核对状态：1–6、9（仓库内 PDF）、15（arXiv 原文）、16、29、31–39（仓库内 PDF，所用公式与表逐式对原版面核对）、17（CDS ReadMe、Crossref）、19–26 的内容本次已直接查阅原件或官方页面；27 只查阅了 Skyfield 源码
> （岁差、星等实现），ASCL 编目信息按通行引文著录；28 未查阅原文，所用 J2000 极向值与 Skyfield 一致；7、8、10–14、18 的
> 卷页按通行引文著录，使用到具体公式或数值时须再对原件核实（P1 中 SOFA 测试值即有两处凭记忆写错、经测试发现后更正）。
