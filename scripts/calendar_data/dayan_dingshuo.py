"""唐大衍历定朔（《新唐书·历志四》卷二十八），用以复核主表 729—761 年朔日与本纪明书朔日之异。

常数：通法 3040，策实 1110343，揲法 89773，转终 6701279（秒法 80），开元十二年（724）甲子积 96961740 算。
算法（通法之分为单位，用志文「若非朔望有交者」之简法，即注历所用）：
1. 中积分 = 策实 × 积算；归余之挂 = 中积分 mod 揲法；朔积分 = 中积分 − 归余之挂；累加揲法得各月经朔。
2. 入气：常气 = 冬至 + k × 策实/24，定气 =「先减、后加」先后数。朓朒定数 = 该定气朓朒积 + 损益率 × 入定气时 / 定气长。
3. 入转：朔积分 × 秒法 mod 转终 ÷ 秒法，累加揲法。朓朒定数 = 朓朒积 + 损益率 × 入余 / 通法；七、十四、二十一、二十八日按初数、末数分段。
4. 定朔 =「朓减、朒加」二定数；小余满通法进日。进朔：大衍历志无明文，《新唐书·历志五》述五纪历云「依大衍戌初進初朔」，取戌初 19 时。
以甲子日夜半为日序 0。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/dayan_dingshuo.py
"""
import sys
from fractions import Fraction as Fr
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import dingshuo_check as c  # noqa: E402
import gaiyuan_date as g  # noqa: E402


F = 3040                # 通法
CE = 1110343            # 策实
SHE = 89773             # 揲法
ZHUAN = 6701279         # 转终（秒法 80）
MIAO = 80
JI_724 = 96961740
QI = Fr(CE, 24)
# 定气（冬至起）先后数（先 +、后 −，「先减、后加」常气）、损益率（朒向为正）；朓朒积由累加得
XIANHOU = [0, 2353, 4198, 5588, 6564, 7152, 7366, 7152, 6564, 5588, 4198, 2353,
           0, -2353, -4198, -5588, -6564, -7152, -7366, -7152, -6564, -5588, -4198, -2353]
SUN_L = [176, 138, 104, 73, 44, 16, -16, -44, -73, -104, -138, -176,
         -176, -138, -104, -73, -44, -16, 16, 44, 73, 104, 138, 176]
# 月离：转日一—二十八损益率（朒向为正）；分段日另列 (初数, 初率, 末数, 末率)
MOON_L = [297, 259, 220, 180, 139, 97, None, -64, -106, -148, -189, -229, -267, None,
          -289, -250, -211, -171, -130, -87, None, 73, 116, 157, 198, 237, 276, None]
SPLIT = {6: (2701, 48, 339, -6), 13: (2363, -231, 677, -66), 20: (2024, -36, 1016, 18), 27: (1686, 165, 1354, 0)}


def moon_ji():
    acc, out = 0, []
    for i, l_ in enumerate(MOON_L):
        out.append(acc)
        acc += SPLIT[i][1] + SPLIT[i][3] if i in SPLIT else l_
    assert acc == 0, acc
    return out


MOON_JI = moon_ji()
assert MOON_JI[7] == 1234 and MOON_JI[14] == -66 and MOON_JI[21] == -1222, MOON_JI


def moon_def(x):
    """入转 x（分）→ 月离定数（分，朒 +）。"""
    i, r = int(x // F), x % F
    if i in SPLIT:
        n0, r0, n1, r1 = SPLIT[i]
        return MOON_JI[i] + (r0 * r / n0 if r <= n0 else r0 + r1 * (r - n0) / n1)
    return MOON_JI[i] + MOON_L[i] * r / F


def ding_qi(k):
    """距天正冬至第 k 个定气之始（分，相对冬至）。k 可超出 0—23。"""
    return k * QI - XIANHOU[k % 24]


def sun_def(t):
    """距天正冬至 t（分，可负）→ 日躔定数（分，朒 +）。"""
    k = int(t // QI) - 1
    while ding_qi(k + 1) <= t:
        k += 1
    ji = sum(SUN_L[:k % 24])
    return ji + SUN_L[k % 24] * (t - ding_qi(k)) / (ding_qi(k + 1) - ding_qi(k))


def year_moons(y, n=14):
    """冬至在 y−1 年之岁各月 [(经朔分, 定朔分)]，自上元甲子夜半起算。"""
    ji = JI_724 + (y - 724)
    zhong = CE * ji
    t0 = zhong - zhong % SHE
    out = []
    for m in range(n):
        jing = t0 + m * SHE
        sd = sun_def(Fr(jing - zhong))
        md = moon_def(Fr((t0 * MIAO + m * SHE * MIAO) % ZHUAN, MIAO))
        out.append((jing, jing + sd + md))
    return out


def main():
    dz = CE * JI_724 // F
    off = c.align_offset(dz, g.jdn(723, 12, 21))
    print("724 年天正冬至", g.GANZHI[dz % 60], g.from_jdn(dz + off))
    moons = sorted({dg for y in range(727, 765) for _, dg in year_moons(y)})
    c.compare("大衍", F, off, moons, 729, 761, "dayan_dingshuo_check.csv",
              "729—761 年本纪明书朔日与大衍历定朔（scripts/calendar_data/dayan_dingshuo.py，志文非交会简法）、主表比较；进朔限戌初 19 时（据五纪历志）。", limit=lambda t: 19.0)


if __name__ == "__main__":
    main()
