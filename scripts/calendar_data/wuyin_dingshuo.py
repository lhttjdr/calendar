"""唐戊寅元历（傅仁均，崔善为校，《新唐书·历志一》卷二十五），复核主表 627—664 年朔日与本纪明书朔日。

志文：武德二年（619）起用定朔；贞观十九年（645）「詔用仁均平朔，訖麟德元年」。今所记为崔善为所较（「復用上元積算」），619—626 年的武德原术（以武德元年为历始）不传，故只比 627 年起。
常数：章岁 676，章月 8361，月法 384075，日法 13006，气法 9464，岁分 3456675，历周 798200，历法 28968；上元戊寅至武德九年（626）丙戌 164348 算外。
算法（志文）：定小余 = 平朔小余 + 入气定盈缩分（盈加缩减）− 入历定盈缩积分（盈减缩加）。
入气：损益率 × 入气日 / 15；入历：（积分 ± 入历余 × 损益率）/ 差法，差法 = 该日行分 − 676（「減去行分六百七十六」，此句可有别解）。
盈缩积分列传写多讹（二日、二十三日等，校勘记逐条指出），按校勘记所引公式重建；十七日损益率作 275、十八日行分作 8557，并从点校本校勘记（校勘记引李善兰《麟德术解》公式核算盈缩积分）。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/wuyin_dingshuo.py
"""
import sys
from fractions import Fraction as Fr
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import dingshuo_check as c  # noqa: E402
import gaiyuan_date as g  # noqa: E402

RI = 13006
YUE = 384075
QIFA = 9464
SUI = 3456675          # 岁分（余数 49635 只给 mod 60，不是真日数）
LIZHOU = 798200
LIFA = 28968
JI_626 = 164348
# 恒气（冬至起）损益率（盈 +、缩 −，有符号的「定盈缩」增量）：盈段冬至至小满，缩段芒种至大雪
SUN = [896, 398, 400, 228, 341, 450, -500, -455, -355, -555, -848, -739, -626, -456, -288, -40, -342, -455, 682, 625, 570, 513, 456, 100]
MOON = [392, 347, 295, 236, 169, 103, 36, -38, -112, -178, -238, -290, -341, -386,
        -371, -326, -275, -216, -156, -90, -23, 51, 118, 184, 243, 302, 347, 383]
XING = [9909, 9810, 9695, 9563, 9414, 9266, 9118, 8953, 8788, 8640, 8508, 8392, 8277, 8178,
        8211, 8310, 8425, 8557, 8689, 8837, 8986, 9151, 9299, 9447, 9578, 9710, 9809, 9891]


def moon_ji():
    """盈缩积分：逐日累计（行分 − 平行分 9037）× 日法（点校本校勘记引《麟德术解》公式；9037 = 章月 + 章岁）。"""
    acc, out = 0, []
    for x in XING:
        out.append(acc)
        acc += (x - 9037) * RI
    return out


MJI = moon_ji()
QI_DAYS = Fr(SUI, QIFA * 24)


def sun_def(t):
    """距冬至 t（日）→ 定盈缩分（日法分，盈 +）。"""
    k = int(t // QI_DAYS)
    return sum(SUN[:k % 24]) + SUN[k % 24] * (t - k * QI_DAYS) / 15


def moon_def(x):
    """入历 x（历法分）→ 定盈缩积分 / 差法（日法分，盈 +）。"""
    i = min(int(x // LIFA), 27)
    r = x - i * LIFA
    return (MJI[i] + r * MOON[i]) / (XING[i] - 676)


def year_moons(y, n=14):
    """冬至在 y−1 年之岁各月 [(平朔日, 定朔日)]（日，自上元甲子夜半）。"""
    nn = JI_626 + (y - 626)
    jiyue = nn * 8361 // 676
    dz = Fr(nn * SUI, QIFA)
    out = []
    for m in range(n):
        p = Fr((jiyue + m) * YUE, RI)
        x = (Fr(int(p) * LIFA % LIZHOU) + (p - int(p)) * LIFA) % LIZHOU
        out.append((p, p + (sun_def(p - dz) - moon_def(x)) / RI))
    return out


def main():
    z = Fr(JI_626 * SUI, QIFA)
    off = c.align_offset(int(z), g.jdn(625, 12, 20))
    print("626 年天正冬至", g.GANZHI[int(z) % 60], g.from_jdn(int(z) + off))
    ms = [m for y in range(624, 668) for m in year_moons(y)]
    # 以日法分为单位交给共用比较
    ding = sorted({d * RI for _, d in ms})
    ping = sorted({p * RI for p, _ in ms})
    c.compare("戊寅", RI, off, ding, 627, 645, "wuyin_ding_check.csv",
              "627—645 年本纪明书朔日与戊寅历定朔（scripts/calendar_data/wuyin_dingshuo.py）、主表比较；本纪不见进朔，不设限。", limit=lambda t: 24.0)
    c.compare("戊寅", RI, off, ping, 646, 664, "wuyin_ping_check.csv",
              "646—664 年本纪明书朔日与戊寅历平朔（scripts/calendar_data/wuyin_dingshuo.py）、主表比较；平朔无进朔。", limit=lambda t: 24.0)


if __name__ == "__main__":
    main()
