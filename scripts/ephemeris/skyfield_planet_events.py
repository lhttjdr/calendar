"""Skyfield 读 de406.bsp 求 2020—2025 年行星会合现象，作为本仓库 `phenomena::planet_events` 的独立对照。
- 合、冲：`almanac.oppositions_conjunctions`（视黄经，当日真黄道）；
- 留：视黄经变化率（中心差分 0.01 日）正负号的 `find_discrete`；
- 大距（水星、金星）：日行角距的 `find_maxima`。
输出：jd(TT) 行星 事件（conj / opp / stat / elong）。结果抄入 rust/core/src/astronomy/phenomena/tests_planet_events.rs。

用 uv 临时环境运行（仓库根）：uv run --no-project --with skyfield python -I scripts/ephemeris/skyfield_planet_events.py
"""
from skyfield import almanac
from skyfield.api import load, load_file
from skyfield.framelib import ecliptic_frame
from skyfield.searchlib import find_discrete, find_maxima

ts = load.timescale(builtin=True)
eph = load_file('vendor/jpl/de406/de406.bsp')
earth, sun = eph[399], eph[10]
t0, t1 = ts.tt_jd(2458849.5), ts.tt_jd(2461041.5)  # 2020-01-01 — 2026-01-01
names = {'Mercury': 1, 'Venus': 2, 'Mars': 4, 'Jupiter': 5, 'Saturn': 6, 'Uranus': 7, 'Neptune': 8}

def lon(t, k):
    return earth.at(t).observe(eph[k]).apparent().frame_latlon(ecliptic_frame)[1].radians

for n, k in names.items():
    f = almanac.oppositions_conjunctions(eph, eph[k])
    for t, _ in zip(*find_discrete(t0, t1, f)):
        # 判别合 / 冲：此刻视黄经差接近 0 为合
        import math
        d = (lon(t, k) - earth.at(t).observe(sun).apparent().frame_latlon(ecliptic_frame)[1].radians + math.pi) % (2 * math.pi) - math.pi
        print(f"{t.tt!r} {n} {'conj' if abs(d) < 1 else 'opp'}")

    def prograde(t, k=k):
        h = 0.01
        a = lon(ts.tt_jd(t.tt - h), k)
        b = lon(ts.tt_jd(t.tt + h), k)
        import numpy as np
        return (((b - a + np.pi) % (2 * np.pi) - np.pi) > 0).astype('int8')
    prograde.step_days = 2.0
    for t, _ in zip(*find_discrete(t0, t1, prograde)):
        print(f"{t.tt!r} {n} stat")

    if k in (1, 2):
        def elong(t, k=k):
            e = earth.at(t)
            return e.observe(eph[k]).apparent().separation_from(e.observe(sun).apparent()).degrees
        elong.step_days = 1.0
        for t, _ in zip(*find_maxima(t0, t1, elong)):
            print(f"{t.tt!r} {n} elong")
