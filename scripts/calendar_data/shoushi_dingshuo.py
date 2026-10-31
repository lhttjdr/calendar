"""元授时历（1281—1367）、明大统历（1368—1644）定朔，复核主表与《元史》《明史》本纪明书朔日。

术文：《元史·历志》卷五十四（授时历经）；《明史·历志》卷三十五（大统历法通轨）。日周 10000 分。
- 中积 = 距算 × 岁实 365.2425 日（距至元十八年辛巳 1281；授时「每百年消一」在 1281—1367 年不起作用，大统「實測無消長」）。
- 冬至 = 中积 + 气应 550600；闰余 = (中积 + 闰应) mod 朔实 29.530593；天正经朔 = 冬至 − 闰余。
- 入转 = (中积 + 转应 − 闰余) mod 转终 27.5546；入缩历 = 半岁周 − 闰余。
- 盈缩差（度）= (定差 − (平差 + 立差 x) x) x / 1e8：盈初缩末 5133200、24600、31；缩初盈末 4870600、22100、27。初限 88.909225 / 93.712025 日。
- 迟疾差（度）= (11110000 − (28100 + 325 x) x) x / 1e8，x = 入迟疾历日 × 12.2 限，初限 84，中限 168。
- 定朔 = 经朔 + (±盈缩差 ±迟疾差) × 820 ÷ 限行度（盈、迟为加）。限行度由迟疾差求导得月每限行度；大统「遲疾限行度內減去八百二十分」即减去日行。
- 应数：《元史》历经闰应 201850、转应 131904；《明史》称《通轨》所载为授时续定之数：闰应 202050、转应 130205。
无进朔（授时废之）。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/shoushi_dingshuo.py
"""
import sys
from fractions import Fraction as Fr
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import dingshuo_check as c  # noqa: E402
import gaiyuan_date as g  # noqa: E402

F = 10000
SUI = 3652425
SHUO = Fr(29530593, 100)                   # 朔实（分）= 295305.93
ZHUAN = 275546
ZHUAN_ZHONG = Fr(ZHUAN, 2)
HALF = Fr(SUI, 2)
QI_YING = 550600
L_YING_INIT, L_TONG = 88 * F + 9092 + Fr(1, 4), 93 * F + 7120 + Fr(1, 4)   # 盈初缩末、缩初盈末限（分）


def sun_poly(x, a, b, cc):
    return (a - (b + cc * x) * x) * x / 10 ** 8


def sun_diff(t):
    """入盈缩历：t 为距冬至分（0—岁周）。返回盈缩差（度，盈 +）。"""
    t = t % SUI
    ying = t < HALF
    x = (t if ying else t - HALF) / F
    if ying:
        if x * F <= L_YING_INIT:
            return sun_poly(x, 5133200, 24600, 31)
        return sun_poly(HALF / F - x, 4870600, 22100, 27)
    if x * F <= L_TONG:
        return -sun_poly(x, 4870600, 22100, 27)
    return -sun_poly(HALF / F - x, 5133200, 24600, 31)


def moon_diff(r):
    """入转 r（分）→ 迟疾差（度，疾 +）。"""
    r = r % ZHUAN
    fast = r < ZHUAN_ZHONG
    d = (r if fast else r - ZHUAN_ZHONG) / F
    x = d * Fr(122, 10)
    if x > 84:
        x = 168 - x
    v = (11110000 - (28100 + 325 * x) * x) * x / 10 ** 8
    return v if fast else -v


def moon_speed(r):
    """月每日行度 = 平行 + 迟疾差的导数。"""
    h = Fr(1, 1000) * F
    return Fr(1336875, 100000) + (moon_diff(r + h) - moon_diff(r - h)) / (2 * h / F)


class Shoushi:
    def __init__(self, name, run_ying, zhuan_ying, relative):
        self.name, self.run_ying, self.zhuan_ying, self.relative = name, run_ying, zhuan_ying, relative

    def moons(self, y0, y1):
        out = []
        for y in range(y0, y1):
            zj = (y - 1281) * SUI
            dz = zj + QI_YING
            run = (zj + self.run_ying) % SHUO
            t0 = dz - run
            ru0 = (zj + self.zhuan_ying - run) % ZHUAN
            for m in range(14):
                jing = t0 + m * SHUO
                deg = sun_diff(jing - dz) - moon_diff(ru0 + m * SHUO)    # 盈加、疾减
                v = moon_speed(ru0 + m * SHUO) * Fr(82, 1000) - (Fr(82, 1000) if self.relative else 0)
                out.append(jing + deg / v * 820)
        return sorted(set(out))


def main():
    dz = QI_YING // F
    off = c.align_offset(dz, g.jdn(1280, 12, 14))
    print("1281 年天正冬至", g.GANZHI[dz % 60], g.from_jdn(dz + off))
    for name, ry, zy, rel, y0, y1, csv in (
            ("授时（历经应数）", 201850, 131904, False, 1281, 1367, "shoushi_lijing_check.csv"),
            ("授时（通轨应数）", 202050, 130205, False, 1281, 1367, "shoushi_tonggui_check.csv"),
            ("大统", 202050, 130205, True, 1368, 1644, "datong_check.csv")):
        print("---", name)
        li = Shoushi(name, ry, zy, rel)
        c.compare(name, F, off, li.moons(y0 - 1, y1 + 2), y0, y1, csv,
                  f"{y0}—{y1} 年本纪明书朔日与{name}定朔（scripts/calendar_data/shoushi_dingshuo.py）、主表比较；授时以后无进朔。",
                  limit=lambda t: 24.0)


if __name__ == "__main__":
    main()
