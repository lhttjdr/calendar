# 太阳引力光线偏折

> 2026-11-09。补 [celestial-phenomena.md](celestial-phenomena.md)「尚未做：太阳引力光线偏折」。代码：
> `rust/core/src/astronomy/correction/light_deflection.rs`（公式）、`apparent/planets.rs`、`apparent/star.rs`（接入），
> 选项 `ApparentPipelineOptions::light_deflection`（`LightDeflection::{None, Sun}`，缺省 `Sun`）。

## 一、公式

广义相对论一阶（PPN γ = 1），与 SOFA `iauLd`、`iauLdsun` 同式（SOFA 注明所据为 Urban & Seidelmann 编《Explanatory Supplement to
the Astronomical Almanac》第 3 版 §7.2.4 与 Klioner 2003, AJ 125, 1580；IERS Conventions (2010) 中对应条文的章节号本项目未核）：

$$\mathbf p_1 = \mathbf p + \frac{2GM_\odot}{c^2 E}\,\frac{\mathbf p\times(\mathbf e\times\mathbf q)}{\max(1+\mathbf q\cdot\mathbf e,\ \delta)}$$

| 量 | 含义 |
|---|---|
| p | 观测者 → 天体方向（单位矢量，偏折前） |
| q | 太阳 → 天体方向（恒星取 q = p） |
| e | 太阳 → 观测者方向；E 为日地距离（AU） |
| 2GM☉/c² | 1.97412574336×10⁻⁸ AU（SOFA `ERFA_SRS`，GM☉ = 1.32712440041×10²⁰ m³/s²） |
| δ | 分母下限 10⁻⁶ / max(E², 1)（SOFA `iauLdsun` 规则，行星同用） |

无限远天体在距角 ψ 处偏折 ≈ 0.00407″·cot(ψ/2)：日面边缘 1.75″、1° 0.47″、10° 0.047″、90° 0.004″，方向背离太阳。

## 二、管线中的位置

- **行星**（`body_apparent_in`，即 `planet_apparent_equatorial`、`body_apparent_ecliptic` 等）：光行时迭代得推迟时 tr 的地心
  状态并转到 ICRS 后、岁差章动前施加。p 取 tr 时地心行星方向，q = 地心行星 − 地心太阳（同在 tr），e = −地心太阳（观测时刻 t）。
  每次多算两次太阳（tr、t）。p 用的是折叠法「行星与地球同在 tr」的矢量，与严格的「行星在 tr、地球在 t」差一个周年光行差（约
  20″），对偏折的相对影响约 10⁻⁴。速度不改。
- **恒星**（`star_apparent_equatorial`、`stars_apparent_equatorial`）：星表方向 → 周年视差 → **偏折** → 周年光行差 → 当日真
  赤道。e 取地心太阳的反向（太阳中心本身，不是质心）。
- **不施加**：太阳本身；月球（距地近，q 与 e 几乎同向，偏折 < 10 μas，且 ELPMPP02 提供者不带太阳）。所以太阳、月球视黄经、
  定气定朔、日月食都不受影响。
- **木星、土星的偏折**未计：木星边缘约 16 mas，离木星 1′ 外 < 1 mas（Skyfield 计这两项）。
- **日面内**（天体被太阳遮掩）时公式无物理意义，δ 只保证有限值。
- `LightDeflection::None` 复原 2026-11-09 前的行为。`astronomy::apparent` 各函数签名不变。

## 三、检验

逐条表在 [phenomena-checks.md](phenomena-checks.md)（`scripts/dev/phenomena_checks.sh` 生成）：

| 检验 | 测试 | 外部依据 | 结果 |
|---|---|---|---|
| 公式对 SOFA | `correction::light_deflection::tests::matches_erfa_ld_and_ldsun`（表 `light_deflection_vs_erfa`） | pyerfa 2.0.1.5 `erfa.ldsun` 12 例（ψ 0.25°—179°）、`erfa.ld` 6 例（水、金、火、木、土，含金星在日后触及下限 δ） | 18 例差 0.0000 μas |
| 量级 | `magnitude_matches_cot_half_angle` | 0.00407″·cot(ψ/2) | 6 个距角相对差 < 10⁻⁶，方向背离太阳 |
| 近合日恒星 | `apparent::star::tests::star_deflection_near_sun_vs_skyfield` | Skyfield 1.55（DE406，P03），轩辕十四、角宿一、心宿二、毕宿五 28 例，距角 0.46°—61° | 偏折量逐例：本实现「开 − 关」与 Skyfield「apparent − 去掉偏折」两矢量差 ≤ 0.014 mas；轩辕十四 0.46° 处两边都是 0.9930″ |
| 近合日行星 | `apparent::tests_planets::planets_near_conjunction_vs_skyfield` | 同上，五星 18 例，距角 0.10°—9° | 偏折矢量差 ≤ 0.002 mas；金星 2000-06-11（距角 0.10°，日面内）两边都是 1.8992″ |
| 远离太阳的行星、恒星 | `planets_pipeline_vs_skyfield`、`star_apparent_vs_skyfield` | 原有 Skyfield 对照 | 计不计偏折都在原阈值内（见下） |

Skyfield 一侧的「去掉偏折」位置：同一 `observe()` 的天体测量位置只加 Skyfield 自己的 `add_aberration`，再转当日真赤道
（`scripts/phenomena/skyfield_deflection_near_sun.py`）。这样比较的是偏折这一项本身，与两边的其余差异（下段）无关。

**其余差异**：计偏折后，近合日恒星与 Skyfield 的位置差 0.02—0.08″、行星 0.02—0.07″，与距角无关（轩辕十四在 0.46° 与 59° 都约
0.07″），来自以日心代质心的光行差、VSOP87 地球对 DE406 的残差、恒星空间运动等原有差别；不计偏折时近合日差可达 1—1.9″。
远离太阳的旧对照里偏折只有几 mas，比这些差别小，所以开关前后看不出系统改善。

## 四、对天象检验结果的影响

做法：改动前后各构建一次（f64），用同一组测试写两套 `data/phenomena/checks/*.csv`，逐格比较（34 项：Skyfield 对照、NASA 日月食典、
伏见、宿度、《汉书》至《清史稿》各史凌犯检验等）。改动前的表与仓库已提交的相同，所以差异只来自偏折。

| 检验 | 变化 |
|---|---|
| NASA 五千年日月食典（前 199—前 100、2001—2100） | 全同（日月不施偏折） |
| 史书日月食（`historical_eclipses`）、日食日躔、月食宿度 | 只用日月，代码路径未变，未重跑 |
| 正史天文志凌犯（`*_star_records_check`，16 表 11762 条） | 判定（合 / 不合、可见与否）全同；只有 4 格角距末位变 0.01°：《后汉书》永和三年八月己酉荧惑入太微 2.39→2.38°，《宋史》大中祥符元年十一月甲午月犯牵牛 81.24→81.25°，《清史稿》雍正元年十二月丁未 2.80→2.79°、乾隆三十九年正月丙寅 6.81→6.82° |
| 伏见（Schaefer 1987 表 1、表 7，Schaefer 2000 天狼星，巴比伦 2025—2026） | 全同 |
| 宿度、距星、石氏星表、先秦天象 | 全同 |
| 近距事件对 Skyfield（49 条） | 角距差平均 0.00102″ → 0.00010″，最大 0.007″ → 0.004″；时刻差平均 0.139 → 0.110 s |
| 月掩星对 Skyfield（36 次） | 入掩差平均 0.134 → 0.126 s，出掩差 0.121 → 0.127 s（最大 0.5、0.4 → 0.44、0.45 s），属求根噪声 |
| 行星会合现象对 Skyfield（241 条） | 37 格变，最大 5.3 s，全是「留」（经度变化率过零，极平坦，原有容差 10 s）；水星 2022-08-27 东大距显示时刻 16:16 → 16:15；平均 \|时刻差\| 18.31 → 18.29 s |
| 恒星视位置对 Skyfield（25 条） | 平均 0.0336″ → 0.0302″，最大 0.078″ → 0.079″ |

结论：史料检验的判定不变。偏折的量级（近合日最多约 2″，一般为毫角秒）远小于史书记录与凌犯判据（角分至度）的精度，日月食更不涉及。
对现代对照（Skyfield 近距）角距差缩小一个量级，说明此前那部分差就是偏折。
