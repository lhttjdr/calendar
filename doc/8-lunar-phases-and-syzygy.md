# 8. 月相与合朔

本章专述**月相**（朔、望、上弦、下弦）与**合朔**的求法、平朔近似公式（W0）、以及合朔与视黄经精度事实。视黄经与视位置计算的 pipeline 见 [7-apparent-longitude-and-syzygy.md](7-apparent-longitude-and-syzygy.md)；八步表与节气朔望标准时刻表格式见 [9-solar-terms.md](9-solar-terms.md)。

## 8.1 合朔与其它月相

- **朔（合朔）**：存在 t(TT) 使 **视**黄经相等 λ_S = λ_M。
- **望**：视黄经差 λ_M − λ_S = π；**上弦** = π/2，**下弦** = 3π/2。求根与合朔同一 pipeline，仅目标角不同。节气朔望标准时刻表中列 Q0=朔、Q1=上弦、Q2=望、Q3=下弦（格式见 [9-solar-terms.md](9-solar-terms.md) §9.6）。
- **求法**：牛顿迭代，f = λ_M − λ_S，f' = dλ_M/d(JD)−dλ_S/d(JD)；导数用解析 `Apparent.*VelocityAnalytic`。
- **Coarse**：几何黄经差 + 平均 synodic 速度 ≈ 0.213 rad/日，得到 Fine 初值。
- **Fine**：视黄经 + 解析导数；可选光行时从第 2 步起；ELP 项数前 3 步用 fineMaxTerms，残差 &lt;1e-3 rad 后放宽到全项；光行时 Fast/Full 见代码；收敛判据 |λ_M−λ_S| &lt; tolerance（如 1e-8 rad）。
- **区间合朔**：`newMoonJDsInRange` 从 W0 按平均朔望月步进得近似 JD，再对每个求精。

## 8.2 平朔近似公式（W0）

以 2000-01-01 0h TT 为历元，第 N 个平朔（N=0 为 2000 年第一个）近似时刻：

**d = 5.597661 + 29.5305888610×N + (102.026×10⁻¹²)×N²** （日）

**JD(TT) = 2451544 + d**，即 `Calendar.approximateNewMoonJD(N)` 依据。

- **来源**：平黄经差 D 对儒略世纪 T 的多项式（Chapront 等 2002 表 4）求导得朔望月长度 P(N)，对 N 积分得 d(N)；二次项 102.026×10⁻¹²×N² 来自 D 的 T² 项（潮汐加速等）。常数项 5.597661 含历元 0h 偏移 0.5 日与光行差常数改正（平合→视合约 −0.000451 日）。
- **代码**：`W0_CONSTANT_DAYS`、`MEAN_SYNODIC_MONTH_W0`、`W0_QUADRATIC_COEFF`；见 `lunar.astronomy.synodic` / `lunar.calendar.Calendar`。

## 8.3 合朔与视黄经精度（事实）

- **视 vs 几何**：若对比「几何黄经合朔」与「视黄经合朔」，章动+光行差可导致数十秒至约百秒差；同定义对比取视对视。
- **历表**：太阳 FK5→ICRS + Vsop87De406IcrsPatchMixedTerms；月球 ELPMPP02，可选 DE406 常数。
- **与 DE406 整链对比**：本项目内部合朔在亚秒到数秒；与 OreKit DE406 视黄经合朔比，差异约数十～约 100 s，主要来自历表/参考架/岁差章动实现的综合有效残差。测试容差 90–120 s 用于「同定义下分钟级一致」。
- **参考文献**：`doc/references/sunMoon_simp.pdf`。
