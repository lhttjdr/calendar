"""以《隋书·律历志》复核北周天和历、大象历的推步常数（push_calendar.rs 的 tianhe_params、daxiang_params）。

《律历志》：
- 天和历（甄鸾《甲寅元历》）：上元甲寅至天和元年丙戌积 875792 算外，章岁 391，蔀法 23460，日法 290160，朔余 153991，斗分 5731；
- 大象历（马显《丙寅元历》）：上元丙寅至大象元年己亥积 41554 算上，日法 53563，章岁 448，蔀法 12992，斗分 3167。
设上元为甲子日夜半、冬至合朔，由积年 × 岁实得天正冬至（取与实际冬至同季之一，甲子约束只定到 60 日周期），
冬至前最近之朔为天正朔，与推步常数比较。

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


if __name__ == "__main__":
    main()
