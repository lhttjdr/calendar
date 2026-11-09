"""Skyfield 近合日恒星、行星的地心视位置（当日真赤道与春分点），核对太阳引力光线偏折
（rust/core/src/astronomy/apparent/star.rs `star_deflection_near_sun_vs_skyfield`、
tests_planets.rs `planets_near_conjunction_vs_skyfield`）。

Skyfield 的 apparent() 含太阳、木星、土星的引力偏折与相对论光行差；另输出同一管线去掉偏折的位置（天体测量位置只加光行差），
两者之差即 Skyfield 的偏折量，供与本仓库「开 − 关」逐例比较（与两边其余差异无关），岁差 IAU 2006（P03）+ 章动 IAU 2000A；历表 DE406。
- 恒星：轩辕十四、角宿一、心宿二、毕宿五，在 2000 年前后距日最近之日及其后若干日（距角约 0.5°—60°）；
- 行星：水星、金星、火星、木星、土星在若干次合日附近（距角 0.5°—数度）。
输出 Rust 元组，抄入上述测试。

  uv run --no-project --with skyfield python -I scripts/phenomena/skyfield_deflection_near_sun.py
"""
import numpy as np
from skyfield.api import Star, load, load_file
from skyfield.functions import length_of
from skyfield.positionlib import Apparent
from skyfield.relativity import add_aberration

ts = load.timescale(builtin=True)
eph = load_file('vendor/jpl/de406/de406.bsp')
earth, sun = eph[399], eph[10]

rows = {}
with open('data/stars/hipparcos_bright.csv', encoding='utf-8') as f:
    for line in f:
        if line.startswith('#') or line.startswith('hip'):
            continue
        p = line.strip().split(',')
        rows[int(p[0])] = [float(x) for x in p[1:6]]


def star_of(hip):
    ra, dec, plx, pmra, pmde = rows[hip]
    return Star(ra_hours=ra / 15.0, dec_degrees=dec, ra_mas_per_year=pmra, dec_mas_per_year=pmde,
                parallax_mas=plx, epoch=ts.tt_jd(2448349.0625))


def radec_without_deflection(e, target):
    """Skyfield 同一管线去掉引力偏折：天体测量位置只加光行差（Skyfield add_aberration），再转当日真赤道。"""
    a = e.observe(target)
    p = a.position.au.copy()
    add_aberration(p, e.velocity.au_per_d, length_of(p) / 173.1446326742403)
    r, d, _ = Apparent(p, t=a.t, center=399).radec(epoch='date')
    return float(r.radians), float(d.radians)


def elong(target, jd):
    e = earth.at(ts.tt_jd(jd))
    return e.observe(target).apparent().separation_from(e.observe(sun).apparent()).degrees


def min_elong_jd(target, jd0, jd1):
    """在 [jd0, jd1] 内找距日最近之时（先 0.25 日网格，再黄金分割细化到约 1 分钟）。"""
    grid = np.arange(jd0, jd1, 0.25)
    k = int(np.argmin([elong(target, j) for j in grid]))
    a, b = grid[max(k - 1, 0)], grid[min(k + 1, len(grid) - 1)]
    g = (5 ** 0.5 - 1) / 2
    while b - a > 1e-3:
        c, d = b - g * (b - a), a + g * (b - a)
        if elong(target, c) < elong(target, d):
            b = d
        else:
            a = c
    return (a + b) / 2


print('// 恒星：(HIP, jd TT, ra rad, dec rad, 距角°, 不计偏折 ra rad, 不计偏折 dec rad)')
for hip in [49669, 65474, 80763, 21421]:
    s = star_of(hip)
    j0 = min_elong_jd(s, 2451545.0 - 200, 2451545.0 + 200)
    for dt in [0.0, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0]:
        jd = round(j0 + dt, 4)
        e = earth.at(ts.tt_jd(jd))
        r, d, _ = e.observe(s).apparent().radec(epoch='date')
        r0, d0 = radec_without_deflection(e, s)
        print(f"    ({hip}, {jd}, {float(r.radians)!r}, {float(d.radians)!r}, {elong(s, jd):.4f}, {r0!r}, {d0!r}),")

names = {'Mercury': 1, 'Venus': 2, 'Mars': 4, 'Jupiter': 5, 'Saturn': 6}
# 合日搜索窗口（TT JD）：2000—2025 年间的若干次合
windows = [('Mercury', 2459310, 2459340), ('Venus', 2459280, 2459320), ('Venus', 2451680, 2451720),
           ('Mars', 2459480, 2459510), ('Jupiter', 2459560, 2459700), ('Saturn', 2459600, 2459640)]
print('// 行星：(jd TT, 行星, ra rad, dec rad, 距角°, 不计偏折 ra rad, 不计偏折 dec rad)')
for n, j0, j1 in windows:
    p = eph[names[n]]
    jm = min_elong_jd(p, j0, j1)
    for dt in [0.0, 2.0, 6.0]:
        jd = round(jm + dt, 4)
        e = earth.at(ts.tt_jd(jd))
        r, d, _ = e.observe(p).apparent().radec(epoch='date')
        r0, d0 = radec_without_deflection(e, p)
        print(f"    ({jd}, Planet::{n}, {float(r.radians)!r}, {float(d.radians)!r}, {elong(p, jd):.4f}, {r0!r}, {d0!r}),")
