"""以《隋书·律历志》复核北周天和历、大象历的推步常数（push_calendar.rs 的 tianhe_params、daxiang_params）。

《律历志》：
- 天和历（甄鸾《甲寅元历》）：上元甲寅至天和元年丙戌积 875792 算外，章岁 391，蔀法 23460，日法 290160，朔余 153991，斗分 5731；
- 大象历（马显《丙寅元历》）：上元丙寅至大象元年己亥积 41554 算上，日法 53563，章岁 448，蔀法 12992，斗分 3167。
设上元为甲子日夜半、冬至合朔，由积年 × 岁实得天正冬至（取与实际冬至同季之一，甲子约束只定到 60 日周期），
冬至前最近之朔为天正朔，与推步常数比较。另列天和年间闰位的几种算法（无中气置闰换用明克让、正光历；天和历闰余法）。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/check_lvli_epochs.py
"""
import math
import sys
from fractions import Fraction as F
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import discriminating_records as d  # noqa: E402
import gaiyuan_date as g  # noqa: E402


def julian_jdn(y, m, day):
    a = (14 - m) // 12
    y2, m2 = y + 4800 - a, m + 12 * a - 3
    return day + (153 * m2 + 2) // 5 + 365 * y2 + y2 // 4 - 32083


def derive(years, Y, L, approx):
    """上元甲子夜半起积 years 年：返回 (天正冬至 JD, 天正朔 JD)。"""
    D = years * Y
    jdn_e = math.floor(approx - D) - 30
    while (jdn_e - 11) % 60:  # 甲子：(JDN + 49) mod 60 = 0
        jdn_e += 1
    cands = [jdn_e + 60 * k for k in range(-1, 3)]
    jdn_e = min(cands, key=lambda j: abs(j - F(1, 2) + D - approx))
    sol = jdn_e - F(1, 2) + D
    return sol, jdn_e - F(1, 2) + math.floor(D / L) * L


def main():
    rows = []
    for name, years, Y, L, approx, push_epoch in (
        ("天和", 875792, 365 + F(5731, 23460), 29 + F(153991, 290160), julian_jdn(565, 12, 22), F(1928976) + F(1, 2) + F(246737, 290160)),
        ("大象", 41553, 365 + F(3167, 12992), 29 + F(28422, 53563), julian_jdn(578, 12, 22), F(1932520) + F(1, 2) + F(36950, 53563)),
    ):
        sol, shuo = derive(years, Y, L, approx)
        k = round(float((shuo - push_epoch) / L))
        diff = float(push_epoch + k * L - shuo)
        rows.append((name, float(sol), g.ganzhi_of_jdn(math.floor(sol + F(1, 2))), float(shuo), diff))
        print(f"{name}：律历志天正冬至 {g.ganzhi_of_jdn(math.floor(sol + F(1, 2)))}，天正朔 JD {float(shuo):.4f}；推步朔历元 − 律历志 = {diff:+.4f} 日")
    leap_alternatives()



def year_leap(lunar, solar, jde, jdw, n):
    """平朔平气、无中气置闰（同 push_calendar.rs::year_new_moons）：农历年 n 的闰月（闰在第几月之后，0 为无闰）。"""
    jde, jdw = jde + 1e-6, jdw + 1e-6
    day = lambda jd: math.floor(jd + 0.5)  # noqa: E731
    z1 = jdw + (n + 1 / 6) * solar
    m1 = jde + math.floor((day(z1) + 0.5 - jde) / lunar) * lunar
    if m1 + 13 * lunar > day(z1 + solar) + 0.5:
        return 0
    st = [day(m1 + j * lunar) for j in range(14)]
    zq = [day(z1 + k * solar / 12) for k in range(13)]
    return next(j for j in range(13) if not any(st[j] <= q < st[j + 1] for q in zq))


def leap_alternatives():
    """天和年间闰位：无中气（天和历、明克让历、正光历）与天和历闰余法（章岁 391、章闰 144，自天正十一月算外）。"""
    params = {"天和": (29 + 153991 / 290160, 365 + 5731 / 23460, 1928976.5 + 246737 / 290160, 1721047.5 + 1331 / 3910),
              "明克让": (29 + 39769 / 74952, 365 + 1477 / 6060, 1911671.5 + 6427 / 9369, 1721048.5 + 569 / 1515 + 0.4),
              "正光": (29 + 39769 / 74952, 365 + 1477 / 6060, 1911671.5 + 6427 / 9369, 1721048.5 + 569 / 1515)}
    months = ["十一", "十二", "正", "二", "三", "四", "五", "六", "七", "八", "九", "十"]
    main_leaps = {k[0]: k[1] for k in d.tables()["主表"] if k[2]}
    print("年 | 无中气：天和/明克让/正光 | 天和历闰余法 | 主表")
    for y in range(566, 579):
        alt = {k: year_leap(*v, y) for k, v in params.items()}
        ry = (875792 + y - 566) * 144 % 391
        ry_leap = f"闰{months[(391 - ry) * 12 // 144]}月" if ry >= 391 - 144 else "无"
        if any(alt.values()) or ry_leap != "无" or y in main_leaps:
            print(y, "|", "/".join(str(alt[k]) for k in params), "|", ry_leap, "|", main_leaps.get(y, 0))


if __name__ == "__main__":
    main()
