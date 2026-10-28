"""Skyfield 读 de406.bsp 求两天体地心视方向角距的极小（≤ 5°），作为本仓库 `phenomena::close_approach` 的独立对照。
行星之间取 2020—2023 年，月与行星取 2023 年；Skyfield `searchlib.find_minima` 求极小。输出：jd(TT) 天体A 天体B 最小角距(°)。
结果抄入 rust/core/src/astronomy/phenomena/tests_close_approach.rs。

用 uv 临时环境运行（仓库根）：uv run --no-project --with skyfield python -I scripts/ephemeris/skyfield_close_approaches.py
"""
from skyfield.api import load, load_file
from skyfield.searchlib import find_minima

ts = load.timescale(builtin=True)
eph = load_file('vendor/jpl/de406/de406.bsp')
earth = eph[399]
ids = {'Mercury': 1, 'Venus': 2, 'Mars': 4, 'Jupiter': 5, 'Saturn': 6, 'Moon': 301}
pairs = [('Jupiter', 'Saturn', 2458849.5, 2460310.5), ('Venus', 'Jupiter', 2458849.5, 2460310.5),
         ('Mars', 'Jupiter', 2458849.5, 2460310.5), ('Mercury', 'Venus', 2458849.5, 2460310.5),
         ('Moon', 'Venus', 2459945.5, 2460310.5), ('Moon', 'Jupiter', 2459945.5, 2460310.5),
         ('Moon', 'Mars', 2459945.5, 2460310.5)]
for a, b, j0, j1 in pairs:
    def sep(t, a=a, b=b):
        e = earth.at(t)
        return e.observe(eph[ids[a]]).apparent().separation_from(e.observe(eph[ids[b]]).apparent()).degrees
    sep.step_days = 0.2 if 'Moon' in (a, b) else 1.0
    for t, v in zip(*find_minima(ts.tt_jd(j0), ts.tt_jd(j1), sep)):
        if v <= 5.0:
            print(f"{float(t.tt)!r} {a} {b} {float(v)!r}")
