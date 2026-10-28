"""Skyfield 读 de406.bsp 计算行星地心视位置（当日真赤道与春分点），作为本仓库管线的外部核对。
输出：jd(TT) 行星 ra(rad) dec(rad) 距离(au) 距角(°)；结果抄入 rust/core/src/astronomy/apparent/tests_planets.rs 的 SKYFIELD 表。

Skyfield（Rhodes, ASCL 1907.024）独立实现 IAU 视位置：光行时、太阳等引力光线偏折、相对论光行差、IAU 2006 岁差 + IAU 2000A 章动。
不属本项目依赖，用 uv 临时环境运行（仓库根）：
  uv run --no-project --with skyfield python -I scripts/ephemeris/skyfield_planet_apparent.py
"""
from skyfield.api import load, load_file

ts = load.timescale(builtin=True)
eph = load_file('vendor/jpl/de406/de406.bsp')
earth, sun = eph[399], eph[10]
names = {'Mercury': 1, 'Venus': 2, 'Mars': 4, 'Jupiter': 5, 'Saturn': 6, 'Uranus': 7, 'Neptune': 8}
for jd in [2451545.0, 2460000.5, 2086308.0, 1757585.0, 1355818.0]:
    t = ts.tt_jd(jd)
    e = earth.at(t)
    s = e.observe(sun).apparent()
    for n, k in names.items():
        a = e.observe(eph[k]).apparent()
        ra, dec, dist = a.radec(epoch='date')
        print(f"{jd} {n} {float(ra.radians)!r} {float(dec.radians)!r} {float(dist.au)!r} {a.separation_from(s).degrees:.2f}")
