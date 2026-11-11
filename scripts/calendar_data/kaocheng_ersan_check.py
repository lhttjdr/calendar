"""诊断：以《历象考成》甲子元复原（shixian_kaocheng.py）的初均、二均、三均算法，求任一（太阴引数 m，月距日 D）之二三均数，
供与《御制历象考成表》卷三「太阴二三均数表」逐格对照。用法：uv run --project scripts python scripts/calendar_data/kaocheng_ersan_check.py m度 D度 ..."""
import math
import sys
sys.path.insert(0, __file__.rsplit("/", 1)[0])
import shixian_kaocheng as ka


def ersan(m, D):
    m = m % 360
    h = ka.MOON_JUNLUN
    small = 3 * h * abs(ka.sin(m))
    big = ka.R + h * ka.cos(m)
    chu_abs = ka.atan2(small, big)
    d1 = math.hypot(small, big)
    chu_signed = -chu_abs if m <= 180 else chu_abs
    e2, side = ka.er_jun(m, chu_signed, chu_abs, d1, D % 360)
    two = ka.norm(2 * D)
    e3, _ = ka.tri_opp(side, ka.CI_JUNLUN, ka.fold(two))
    e3 = e3 if two < 180 else -e3
    return e2, e3, e2 + e3


def fmt(x):
    s = "+" if x >= 0 else "-"
    x = abs(x) * 60
    return f"{s}{int(x)}′{(x % 1) * 60:04.1f}″"


if __name__ == "__main__":
    a = [float(v) for v in sys.argv[1:]]
    for m, D in zip(a[::2], a[1::2]):
        e2, e3, t = ersan(m, D)
        print(f"引数 {m}° 月距日 {D}°：二均 {fmt(e2)} 三均 {fmt(e3)} 二三均 {fmt(t)}")
