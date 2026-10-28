"""Skyfield 计算某地的月掩星（入掩、出掩时刻，TT），核对 rust/core/src/astronomy/phenomena/occultation.rs。

月面半径取 k·a_e，k = 0.2725076、a_e = 6378.137 km（与本项目相同）；站心视位置（Skyfield apparent：光行时、光行差、
引力偏折；观测者 WGS84）；天体中心与月心站心角距 = 月面视半径的时刻。恒星取 data/stars/hipparcos_bright.csv
（I/311，历元 J1991.25）。历表 DE441。输出抄入 tests_occultation.rs 的 SKYFIELD_OCC 表。

  uv run --no-project --with skyfield python -I scripts/phenomena/skyfield_occultations.py
"""
import numpy as np
from skyfield.api import Star, load, load_file, wgs84

ts = load.timescale(builtin=True)
eph = load_file('vendor/jpl/de441/de441.bsp')
earth, moon = eph[399], eph[301]
site = earth + wgs84.latlon(39.9067, 116.3912, elevation_m=50.0)
R_MOON_KM = 0.2725076 * 6378.137
stars = {}
with open('data/stars/hipparcos_bright.csv', encoding='utf-8') as f:
    for line in f:
        if line.startswith('#') or line.startswith('hip'):
            continue
        p = line.strip().split(',')
        stars[int(p[0])] = [float(x) for x in p[1:6]]
targets = {}
for hip in [49669, 65474, 80763, 21421, 17702]:
    ra, dec, plx, pmra, pmde = stars[hip]
    targets[f'HIP{hip}'] = Star(ra_hours=ra / 15.0, dec_degrees=dec, ra_mas_per_year=pmra, dec_mas_per_year=pmde,
                                parallax_mas=plx, epoch=ts.tt_jd(2448349.0625))
for name, k in [('Venus', 2), ('Mars', 4), ('Jupiter', 5), ('Saturn', 6)]:
    targets[name] = eph[k]


def gap(t, target):
    o = site.at(t)
    m = o.observe(moon).apparent()
    b = o.observe(target).apparent()
    return m.separation_from(b).radians - np.arcsin(R_MOON_KM / m.distance().km)


def root(f, a, b):
    fa = f(a)
    for _ in range(60):
        c = 0.5 * (a + b)
        fc = f(c)
        if (fc < 0) == (fa < 0):
            a, fa = c, fc
        else:
            b = c
    return 0.5 * (a + b)


j0, j1 = 2459945.5, 2461406.5  # 2023-01-01 … 2027-01-01
step = 5.0 / 1440
for name, target in targets.items():
    for c0 in np.arange(j0, j1, 30.0):
        jd = np.arange(c0, min(c0 + 30.0, j1), step)
        g = gap(ts.tt_jd(jd), target)
        for i in range(1, len(jd) - 1):
            if g[i] < 0 and g[i] <= g[i - 1] and g[i] < g[i + 1]:
                f = lambda x: float(gap(ts.tt_jd(x), target))
                a = jd[i]
                while f(a) < 0:
                    a -= step
                b = jd[i]
                while f(b) < 0:
                    b += step
                ti = root(f, a, jd[i])
                te = root(f, jd[i], b)
                dt = float(ts.tt_jd(ti).delta_t)
                print(f'    ("{name}", {float(ti)!r}, {float(te)!r}, {dt:.3f}),')
