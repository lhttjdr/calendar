"""唐长庆宣明历定朔（徐昂，《新唐书·历志六上》卷三十上），用以复核主表 822—892 年朔日与本纪明书朔日之异。

志文只载常数、表与「通法曰统法……」之改名（术同大衍历），故定朔用大衍历非交会简法：入定气线性插值损益率、入历线性插值损益率。
常数：统法 8400，章岁 3068055，章月 248057，历周 231458 秒 19（秒母 100），历中 115729 秒 9.5，上元甲子至长庆二年（822）壬寅积 7070138 算外。
定气 = 常气「先减、后加」先后数 × 100。历日分两半，各十四日；七日初数 7465、末数 935，十四日初数 6529。志文无进朔法，以 18 时试之。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/xuanming_dingshuo.py
"""
import sys
from fractions import Fraction as Fr
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import dingshuo_check as c  # noqa: E402
import gaiyuan_date as g  # noqa: E402

F = 8400
SUI = 3068055
SHUO = 248057
LI_ZHOU = Fr(23145819, 100)
LI_ZHONG = LI_ZHOU / 2
JI_822 = 7070138
QI = Fr(SUI, 24)
XIANHOU = [0, 60, 110, 150, 180, 198, 204, 198, 180, 150, 110, 60, 0, -60, -110, -150, -180, -198, -204, -198, -180, -150, -110, -60]
SUN_L = [449, 374, 299, 224, 135, 45, -45, -135, -224, -299, -374, -449,
         -449, -374, -299, -224, -135, -45, 45, 135, 224, 299, 374, 449]
# 前半（朒）、后半（朓）损益率，朒向为正；七日分初末，十四日只初
MOON_A = [830, 726, 606, 471, 337, 202, (53, -7), -82, -224, -366, -509, -643, -748, -646]
MOON_B = [-830, -726, -598, -464, -329, -195, (-53, 7), 82, 225, 366, 501, 628, 740, 646]


def half_ji(tab):
    acc, out = 0, []
    for v in tab:
        out.append(acc)
        acc += sum(v) if isinstance(v, tuple) else v
    return out, acc


JI_A, END_A = half_ji(MOON_A)
JI_B, END_B = half_ji(MOON_B)
assert (END_A, END_B) == (0, 0) and JI_A[7] == 3218 and JI_B[7] == -3188, (END_A, END_B, JI_A, JI_B)


def moon_def(x):
    """入历 x（分）→ 月离定数（分，朒 +）。"""
    tab, ji = (MOON_A, JI_A) if x < LI_ZHONG else (MOON_B, JI_B)
    if x >= LI_ZHONG:
        x -= LI_ZHONG
    i, r = min(int(x // F), 13), x - min(int(x // F), 13) * F
    if i == 6:
        return ji[6] + (tab[6][0] * r / 7465 if r <= 7465 else tab[6][0] + tab[6][1] * (r - 7465) / 935)
    if i == 13:
        return ji[13] + tab[13] * min(r, Fr(6529)) / 6529
    return ji[i] + tab[i] * r / F


def ding_qi(k):
    return k * QI - 100 * XIANHOU[k % 24]


def sun_def(t):
    k = int(t // QI) - 1
    while ding_qi(k + 1) <= t:
        k += 1
    return sum(SUN_L[:k % 24]) + SUN_L[k % 24] * (t - ding_qi(k)) / (ding_qi(k + 1) - ding_qi(k))


def year_moons(y, n=14):
    tong = SUI * (JI_822 + (y - 822))
    t0 = tong - tong % SHUO
    return [(t0 + m * SHUO, t0 + m * SHUO + sun_def(Fr(t0 + m * SHUO - tong)) + moon_def((t0 % LI_ZHOU + m * SHUO) % LI_ZHOU)) for m in range(n)]


def main():
    dz = SUI * JI_822 // F
    off = c.align_offset(dz, g.jdn(821, 12, 19))
    print("822 年天正冬至", g.GANZHI[dz % 60], g.from_jdn(dz + off))
    moons = sorted({dg for y in range(820, 896) for _, dg in year_moons(y)})
    c.compare("宣明", F, off, moons, 822, 892, "xuanming_dingshuo_check.csv",
              "822—892 年本纪明书朔日与宣明历定朔（scripts/calendar_data/xuanming_dingshuo.py，同大衍简法）、主表比较；进朔以 18 时试之。")


if __name__ == "__main__":
    main()
