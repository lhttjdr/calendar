"""唐景福崇玄历定朔（边岡，《新唐书·历志六下》卷三十下），用以复核主表 893—938 年朔日与本纪明书朔日之异（尤其《旧唐书》900 年两条）。

志文只载常数、日躔月离两表，不载求定朔之术；此处按两表体例用通行之法（假定，非志文）：入恒气线性插值损益数，入转日线性插值损益率，
「朓减、朒加」经朔为定朔。七、十四、二十一、二十八日按初数、末数分段。进朔之法志文亦无，以 18 时试之，由比较检验。
常数：通法 13500，岁实 4930801，朔实 398663，转周分 371986 秒 97（秒母 100），上元甲子距景福元年（892）壬子 53947308 算外。
以甲子日夜半为日序 0。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/chongxuan_dingshuo.py
"""
import sys
from fractions import Fraction as Fr
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import dingshuo_check as c  # noqa: E402
import gaiyuan_date as g  # noqa: E402


F = 13500
SUI = 4930801
SHUO = 398663
ZHUAN = Fr(37198697, 100)
JI_892 = 53947308
QI = Fr(SUI, 24)
# 恒气（冬至起）损益数（朒向为正）
SUN_L = [782, 613, 462, 328, 200, 67, -67, -200, -328, -462, -613, -782,
         -782, -613, -462, -328, -200, -67, 67, 200, 328, 462, 613, 782]
MOON_L = [1319, 1150, 978, 799, 617, 431, None, -285, -471, -650, -840, -1017, -1185, None,
          -1284, -1110, -941, -757, -578, -386, None, 324, 516, 697, 879, 1053, 1223, None]
SPLIT = {6: (Fr(47987, 4), 213, Fr(1503), -27), 13: (Fr(20987, 2), -1032, Fr(6013, 2), -293),
         20: (Fr(35961, 4), -160, Fr(18039, 4), 80), 27: (Fr(7487), 737, Fr(6013), 0)}


def moon_ji():
    acc, out = 0, []
    for i, l_ in enumerate(MOON_L):
        out.append(acc)
        acc += SPLIT[i][1] + SPLIT[i][3] if i in SPLIT else l_
    assert acc == 0, acc
    return out


MOON_JI = moon_ji()
assert MOON_JI[7] == 5480 and MOON_JI[14] == -293 and MOON_JI[21] == -5429, MOON_JI


def moon_def(x):
    """入转 x（分）→ 月离定数（分，朒 +）。"""
    i, r = int(x // F), x % F
    if i in SPLIT:
        n0, r0, n1, r1 = SPLIT[i]
        return MOON_JI[i] + (r0 * r / n0 if r <= n0 else r0 + r1 * (r - n0) / n1)
    return MOON_JI[i] + MOON_L[i] * r / F


def sun_def(t):
    """距天正冬至 t（分，可负）→ 日躔定数（分，朒 +）。"""
    k = int(t // QI)
    return sum(SUN_L[:k % 24]) + SUN_L[k % 24] * (t - k * QI) / QI


def year_moons(y, n=14):
    """冬至在 y−1 年之岁各月 [(经朔分, 定朔分)]。"""
    tong = SUI * (JI_892 + (y - 892))
    t0 = tong - tong % SHUO
    return [(t0 + m * SHUO, t0 + m * SHUO + sun_def(Fr(t0 + m * SHUO - tong)) + moon_def((t0 % ZHUAN + m * SHUO) % ZHUAN)) for m in range(n)]


def main():
    dz = SUI * JI_892 // F
    off = c.align_offset(dz, g.jdn(891, 12, 17))
    print("892 年天正冬至", g.GANZHI[dz % 60], g.from_jdn(dz + off))
    moons = sorted({dg for y in range(890, 942) for _, dg in year_moons(y)})
    c.compare("崇玄", F, off, moons, 893, 938, "chongxuan_dingshuo_check.csv",
              "893—938 年本纪明书朔日与崇玄历定朔（scripts/calendar_data/chongxuan_dingshuo.py；志文无求定朔术，线性插值为假定）、主表比较；进朔以 18 时试之。")


if __name__ == "__main__":
    main()
