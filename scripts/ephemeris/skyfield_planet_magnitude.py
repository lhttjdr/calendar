"""Skyfield 读 de406.bsp 计算行星地心视星等（Mallama & Hilton 2018，Skyfield `magnitudelib`，土星含环），作为本仓库
`apparent::magnitude` 的对照。输出：jd(TT) 行星 V_skyfield V_真太阳（nan 表示超出公式有效范围）。

Skyfield 的 `planetary_magnitude` 把太阳当作位于太阳系质心（源码注释「Shamelessly treat the Sun as sitting at the Solar System
Barycenter」），水星等近日行星因此有 0.01—0.04 星等之差；第二列改用 eph[10] 真太阳位置、调用 Skyfield 同一套公式函数
（`magnitudelib._FUNCTIONS`），与本仓库同法，作为对照基准。结果抄入 rust/core/src/astronomy/apparent/
tests_planets.rs 的 SKYFIELD_MAG 表。

用 uv 临时环境运行（仓库根）：uv run --no-project --with skyfield python -I scripts/ephemeris/skyfield_planet_magnitude.py
"""
from skyfield.api import load, load_file
from skyfield.magnitudelib import planetary_magnitude, _FUNCTIONS, _saturn_magnitude, _uranus_magnitude, _neptune_magnitude
from skyfield.magnitudelib import _SATURN_POLE, _URANUS_POLE
from skyfield.functions import angle_between, length_of
from skyfield.constants import RAD2DEG

ts = load.timescale(builtin=True)
eph = load_file('vendor/jpl/de406/de406.bsp')
earth = eph[399]
names = {'Mercury': 1, 'Venus': 2, 'Mars': 4, 'Jupiter': 5, 'Saturn': 6, 'Uranus': 7, 'Neptune': 8}
for jd in [2451545.0, 2455000.5, 2460000.5, 2458850.5]:
    t = ts.tt_jd(jd)
    for n, k in names.items():
        astro = earth.at(t).observe(eph[k])
        m = planetary_magnitude(astro)
        obs_to_planet = astro.xyz.au
        sun_to_obs = -earth.at(t).observe(eph[10]).xyz.au
        sun_to_planet = sun_to_obs + obs_to_planet
        r, delta = length_of(sun_to_planet), length_of(obs_to_planet)
        ph = angle_between(sun_to_planet, obs_to_planet) * RAD2DEG
        f = _FUNCTIONS[k]
        if f is _saturn_magnitude or f is _uranus_magnitude:
            pole = _SATURN_POLE if f is _saturn_magnitude else _URANUS_POLE
            m2 = f(r, delta, ph, angle_between(pole, sun_to_planet) * RAD2DEG - 90.0, angle_between(pole, obs_to_planet) * RAD2DEG - 90.0)
        elif f is _neptune_magnitude:
            m2 = f(r, delta, ph, t.J)
        else:
            m2 = f(r, delta, ph)
        print(f"{jd} {n} {float(m)!r} {float(m2)!r}")
