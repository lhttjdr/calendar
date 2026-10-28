"""Skyfield 计算若干 Hipparcos 亮星的地心视位置（当日真赤道与春分点），核对 rust/core/src/astronomy/apparent/star.rs。
星表值取 data/stars/hipparcos_bright.csv（I/311 位置自行 + 视差，历元 J1991.25），历表 DE406。
Skyfield 含周年视差、太阳引力偏折、相对论光行差；本项目未计前两者（亮星视差 < 0.8″，偏折离日远时 < 0.01″）。
结果抄入 star.rs 测试的 SKYFIELD_STARS 表。

  uv run --no-project --with skyfield python -I scripts/phenomena/skyfield_star_apparent.py
"""
from skyfield.api import Star, load, load_file

ts = load.timescale(builtin=True)
eph = load_file('vendor/jpl/de406/de406.bsp')
earth = eph[399]
rows = {}
with open('data/stars/hipparcos_bright.csv', encoding='utf-8') as f:
    for line in f:
        if line.startswith('#') or line.startswith('hip'):
            continue
        p = line.strip().split(',')
        rows[int(p[0])] = [float(x) for x in p[1:6]]
for hip in [32349, 69673, 100345, 49669, 28358]:
    ra, dec, plx, pmra, pmde = rows[hip]
    star = Star(ra_hours=ra / 15.0, dec_degrees=dec, ra_mas_per_year=pmra, dec_mas_per_year=pmde,
                parallax_mas=plx, epoch=ts.tt_jd(2448349.0625))
    for jd in [2446145.5, 2451545.0, 2460676.5, 2268923.5, 1757585.0]:
        a = earth.at(ts.tt_jd(jd)).observe(star).apparent()
        r, d, _ = a.radec(epoch='date')
        print(f"    ({hip}, {jd}, {float(r.radians)!r}, {float(d.radians)!r}),")
