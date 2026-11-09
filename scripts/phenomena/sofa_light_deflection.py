"""SOFA（pyerfa）太阳引力光线偏折算例，核对 rust/core/src/astronomy/correction/light_deflection.rs。

前 12 例 erfa.ldsun（恒星，q = p），距角 ψ 自 0.25° 到 179°；后 6 例 erfa.ld（有限距离天体，太阳 → 天体方向 q ≠ p），
含近合日（天体在日后，1 + q·e 触及下限 dlim）的情形。日地距离 em 取 0.983—1.017 AU。
dlim 按 SOFA iauLdsun 的规则 1e-6 / max(em², 1) 给出（ld 例同用此值）。
输出 Rust 数组字面量到 rust/core/src/astronomy/correction/light_deflection_erfa_cases.in。

  uv run --no-project --with pyerfa --with numpy python -I scripts/phenomena/sofa_light_deflection.py
"""
import math

import erfa
import numpy as np

OUT = 'rust/core/src/astronomy/correction/light_deflection_erfa_cases.in'


def unit(v):
    v = np.asarray(v, dtype=float)
    return v / np.linalg.norm(v)


def rot(v, ax, ang):
    """绕单位轴 ax 转 ang（Rodrigues）。"""
    ax = unit(ax)
    v = np.asarray(v, dtype=float)
    return v * math.cos(ang) + np.cross(ax, v) * math.sin(ang) + ax * np.dot(ax, v) * (1 - math.cos(ang))


def dlim_of(em):
    return 1e-6 / max(em * em, 1.0)


rng = np.random.default_rng(20261109)
rows = []
# 恒星：e 为太阳 → 地，太阳方向 −e；星方向 p 与 −e 成 ψ
for k, psi in enumerate([0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 90.0, 120.0, 150.0, 179.0]):
    e = unit(rng.normal(size=3))
    em = 0.983 + 0.034 * rng.random()
    axis = np.cross(e, unit(rng.normal(size=3)))
    p = unit(rot(-e, axis, math.radians(psi)))
    p1 = erfa.ldsun(p, e, em)
    rows.append((p, p, e, em, dlim_of(em), p1))

# 有限距离天体：地在 e·em（日心），天体日心位置 b；p = 地 → 天体，q = 日 → 天体
cases = [
    # (天体日距 AU, 日心经向与地的夹角°, 说明)
    (0.723, 178.0, '金星近上合'),
    (0.723, 179.9, '金星在日后（触及下限）'),
    (0.387, 170.0, '水星近上合'),
    (5.20, 175.0, '木星近合'),
    (1.52, 60.0, '火星'),
    (9.54, 2.0, '土星近冲'),
]
for r_b, ang, _ in cases:
    e = unit(rng.normal(size=3))
    em = 0.983 + 0.034 * rng.random()
    axis = np.cross(e, unit(rng.normal(size=3)))
    q = unit(rot(e, axis, math.radians(ang)))
    b = q * r_b
    pv = b - e * em
    p = unit(pv)
    dl = dlim_of(em)
    p1 = erfa.ld(1.0, p, q, e, em, dl)
    rows.append((p, q, e, em, dl, p1))


def f(v):
    return '[' + ', '.join(repr(float(x)) for x in v) + ']'


with open(OUT, 'w', encoding='utf-8') as fo:
    fo.write('// 由 scripts/phenomena/sofa_light_deflection.py 生成（pyerfa %s），勿手改\n&[\n' % erfa.__version__)
    for p, q, e, em, dl, p1 in rows:
        fo.write(f'    ({f(p)}, {f(q)}, {f(e)}, {em!r}, {dl!r}, {f(p1)}),\n')
    fo.write(']\n')
print(f'写 {OUT}：{len(rows)} 例')
