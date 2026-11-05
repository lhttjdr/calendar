"""历元与分点：同一颗星，坐标随所用的分点而变。

1. J2000 平赤道/平春分点 与 公元前 1000 年的平赤道/平春分点：按 IAU 2006 岁差
   p_A = 5028.796195″T + 1.1054348″T²、ε_A = 84381.406″ − 46.836769″T − 0.0001831″T²（T 为自 J2000 起儒略世纪），
   真实角度，无夸张；黄道自身的转动（行星岁差，三千年约 0.4°）不画。
2. 角宿一（Spica）的黄经、赤经赤纬在两个分点下的数值。
3. 真春分点与平春分点（差黄经章动 Δψ，≤17″）、ICRS 与 J2000 平赤道（frame bias，约 0.02″）：
   画面中放大，字幕注明真实大小。

渲染：scripts/render_manim.sh celestial_sphere/epochs_equinoxes
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import numpy as np
from manim import (
    DEGREES,
    DOWN,
    LEFT,
    OUT,
    RIGHT,
    UL,
    UP,
    Create,
    Dot3D,
    FadeIn,
    FadeOut,
    VGroup,
)

from _sky import (
    SkyScene,
    RADIUS,
    arc,
    arc_along,
    basis_of,
    ecl,
    equator_pole,
    great_circle,
    sphere_wireframe,
    star,
    unit,
)
from _style import (
    BACKGROUND,
    COLOR_ECLIPTIC,
    COLOR_EPOCH,
    COLOR_FIXED_FRAME,
    COLOR_MEAN,
    COLOR_STAR,
    COLOR_TRUE,
    caption,
    label,
    rich,
)

ASEC = np.pi / 180 / 3600
T_DATE = (-1000 - 2000) / 100.0  # 公元前 1000 年 ≈ 天文年 −999，取整到世纪示意即可
P_A = (5028.796195 * T_DATE + 1.1054348 * T_DATE**2) * ASEC
EPS_J2000 = 84381.406 * ASEC
EPS_DATE = (84381.406 - 46.836769 * T_DATE - 0.0001831 * T_DATE**2) * ASEC
NODE_DATE = -P_A  # 当日春分点在 J2000 黄道上的黄经（公元前为正：彼时春分点偏东）

SPICA_LON, SPICA_LAT = 203.841 * DEGREES, -2.054 * DEGREES


def equatorial(v: np.ndarray, pole: np.ndarray, vernal: np.ndarray) -> tuple[float, float]:
    e0, e90 = basis_of(pole, vernal)
    u = unit(v)
    return np.degrees(np.arctan2(u @ e90, u @ e0)) % 360, np.degrees(np.arcsin(u @ unit(pole)))


class EpochsEquinoxes(SkyScene):
    def construct(self):
        self.camera.background_color = BACKGROUND
        self.set_camera_orientation(phi=55 * DEGREES, theta=-100 * DEGREES, zoom=0.8)

        title = caption("历元与分点：同一颗星，坐标随所取的春分点而变", size=34).to_corner(UL)
        self.add_fixed_in_frame_mobjects(title)

        wire = sphere_wireframe()
        ecliptic = great_circle(OUT, COLOR_ECLIPTIC, width=4)
        lab_ecl = label("黄道", size=24, color=COLOR_ECLIPTIC).move_to(ecl(300 * DEGREES, 0, RADIUS * 1.12))
        self.face(lab_ecl)
        self.play(FadeIn(wire), Create(ecliptic), FadeIn(lab_ecl), run_time=1.2)

        # J2000 平赤道与平春分点
        pole_j = equator_pole(EPS_J2000, 0.0)
        v_j = ecl(0)
        eq_j = great_circle(pole_j, COLOR_EPOCH, width=4)
        dot_j = Dot3D(v_j, radius=0.09, color=COLOR_EPOCH)
        lab_j = label("J2000 春分点", size=22, color=COLOR_EPOCH).move_to(v_j * 1.2 - OUT * 0.3)
        lab_eqj = label("J2000 平赤道", size=22, color=COLOR_EPOCH).move_to(
            _point_on(pole_j, ecl(250 * DEGREES)) * 1.16
        )
        self.face(lab_j, lab_eqj)
        self.play(Create(eq_j), FadeIn(dot_j), FadeIn(lab_j), FadeIn(lab_eqj), run_time=1.5)

        # 公元前 1000 年平赤道与平春分点
        pole_d = equator_pole(EPS_DATE, NODE_DATE)
        v_d = ecl(NODE_DATE)
        eq_d = great_circle(pole_d, COLOR_MEAN, width=4)
        dot_d = Dot3D(v_d, radius=0.09, color=COLOR_MEAN)
        lab_d = label("前 1000 年春分点", size=22, color=COLOR_MEAN).move_to(v_d * 1.22 + OUT * 0.3)
        lab_eqd = label("前 1000 年平赤道", size=22, color=COLOR_MEAN).move_to(
            _point_on(pole_d, ecl(130 * DEGREES)) * 1.16
        )
        self.face(lab_d, lab_eqd)
        shift_arc = arc_along(OUT, RIGHT, 0, NODE_DATE, COLOR_FIXED_FRAME, width=7)
        note = rich(
            rf"从前 1000 年到 J2000，春分点沿黄道西移 {np.degrees(P_A) * -1:.1f}°（真实角度）" "\n"
            rf"黄赤交角 $\varepsilon_A$：{np.degrees(EPS_DATE):.2f}° → {np.degrees(EPS_J2000):.2f}°",
            size=22,
        ).to_edge(DOWN)
        self.fix(note)
        self.play(Create(eq_d), FadeIn(dot_d), FadeIn(lab_d), FadeIn(lab_eqd), run_time=1.5)
        self.play(Create(shift_arc), FadeIn(note), run_time=1.2)
        self.wait(2)

        # 角宿一的两套坐标
        s = ecl(SPICA_LON, SPICA_LAT)
        st = star(s, COLOR_STAR, radius=0.09)
        lab_s = label("角宿一", size=22).move_to(s * 1.12 - OUT * 0.3)
        self.face(lab_s)
        lon_j = arc_along(OUT, RIGHT, 0, SPICA_LON, COLOR_EPOCH, r=RADIUS * 1.03, width=6)
        lon_d = arc_along(OUT, RIGHT, NODE_DATE, SPICA_LON, COLOR_MEAN, r=RADIUS * 0.97, width=6)
        lam_j = np.degrees(SPICA_LON)
        lam_d = np.degrees(SPICA_LON - NODE_DATE)
        ra_j, de_j = equatorial(s, pole_j, v_j)
        ra_d, de_d = equatorial(s, pole_d, v_d)
        self.play(FadeOut(note), FadeIn(st), FadeIn(lab_s), run_time=0.8)
        rows = [
            ("角宿一", "黄经", "赤经", "赤纬", COLOR_FIXED_FRAME),
            ("J2000 分点", f"{lam_j:.1f}°", f"{ra_j:.1f}°", f"{de_j:+.1f}°", COLOR_EPOCH),
            ("前 1000 年分点", f"{lam_d:.1f}°", f"{ra_d:.1f}°", f"{de_d:+.1f}°", COLOR_MEAN),
        ]
        tab = VGroup(*[caption(c, size=24, color=row[4]) for row in rows for c in row[:4]])
        tab.arrange_in_grid(rows=3, cols=4, buff=(0.45, 0.15), col_alignments="lrrr").to_corner(UL).shift(DOWN * 0.8)
        self.fix(tab)
        self.play(Create(lon_j), Create(lon_d), FadeIn(tab), run_time=2)
        note2 = caption("星本身没动：变的是量度起点（春分点）和参考大圆（赤道）\n查古代星位，须用当时的分点", size=22).to_edge(DOWN)
        self.fix(note2)
        self.play(FadeIn(note2), run_time=0.5)
        self.begin_ambient_camera_rotation(rate=0.08)
        self.wait(5)
        self.stop_ambient_camera_rotation()

        # 平春分点与真春分点（放大）
        self.play(*[FadeOut(m) for m in (note2, tab, lon_j, lon_d, st, lab_s, shift_arc, eq_d, dot_d, lab_d, lab_eqd)], run_time=1)
        title2 = rich(r"平春分点与真春分点：差一个黄经章动 $\Delta\psi$", size=34).to_corner(UL)
        self.fix(title2)
        self.play(FadeOut(title), run_time=0.3)
        self.play(FadeIn(title2), run_time=0.4)
        dpsi_show, deps_show = 7 * DEGREES, 3 * DEGREES  # 放大示意
        pole_t = equator_pole(EPS_J2000 + deps_show, dpsi_show)
        v_t = ecl(dpsi_show)
        eq_t = great_circle(pole_t, COLOR_TRUE, width=4)
        dot_t = Dot3D(v_t, radius=0.09, color=COLOR_TRUE)
        lab_t = label("真春分点", size=22, color=COLOR_TRUE).move_to(v_t * 1.22 + OUT * 0.35)
        lab_eqt = label("真赤道", size=22, color=COLOR_TRUE).move_to(_point_on(pole_t, ecl(120 * DEGREES)) * 1.16)
        self.face(lab_t, lab_eqt)
        dpsi_arc = arc(v_j, v_t, COLOR_TRUE, width=7)
        note3 = rich(
            r"平赤道（只计岁差）加上章动 → 真赤道；两分点沿黄道差 $\Delta\psi$" "\n"
            r"实际 $|\Delta\psi| \le 17''$，图中放大约 1500 倍。节气、合朔用的是从真春分点量起的黄经",
            size=22,
        ).to_edge(DOWN)
        self.fix(note3)
        lab_j2 = label("平春分点", size=22, color=COLOR_EPOCH).move_to(v_j * 1.2 - OUT * 0.3)
        self.face(lab_j2)
        self.play(FadeOut(lab_j), FadeIn(lab_j2), Create(eq_t), FadeIn(dot_t), FadeIn(lab_t), FadeIn(lab_eqt), run_time=1.5)
        self.play(Create(dpsi_arc), FadeIn(note3), run_time=1)
        self.wait(4)

        # ICRS 与 J2000 平赤道（放大）
        self.play(*[FadeOut(m) for m in (note3, eq_t, dot_t, lab_t, lab_eqt, dpsi_arc)], run_time=0.8)
        title3 = caption("ICRS 与 J2000 平赤道：几乎重合，差一个 frame bias", size=34).to_corner(UL)
        self.fix(title3)
        self.play(FadeOut(title2), run_time=0.3)
        self.play(FadeIn(title3), run_time=0.4)
        # 极偏 ξ0 = −16.6 mas、η0 = −6.8 mas，赤经原点差 dα0 = −14.6 mas；图中按同一方向放大
        bias = np.array([-16.6, -6.8]) / 17.9 * 4 * DEGREES
        e0, e90 = basis_of(pole_j, v_j)
        pole_i = unit(pole_j + np.tan(bias[0]) * e0 + np.tan(bias[1]) * e90)
        eq_i = great_circle(pole_i, COLOR_FIXED_FRAME, width=4)
        lab_i = label("ICRS 赤道", size=22).move_to(_point_on(pole_i, ecl(-120 * DEGREES)) * 1.14)
        self.face(lab_i)
        note4 = caption(
            "ICRS 由河外射电源定义，不随岁差转动；极与 J2000 平极差约 0.018″（图中放大约 80 万倍）\n"
            "现代历表在 ICRS 中给出，接经典岁差章动前先扣除这个偏差",
            size=22,
        ).to_edge(DOWN)
        self.fix(note4)
        self.play(FadeOut(lab_j2), FadeOut(lab_eqj), Create(eq_i), FadeIn(lab_i), FadeIn(note4), run_time=1.5)
        self.begin_ambient_camera_rotation(rate=0.08)
        self.wait(5)
        self.stop_ambient_camera_rotation()


def _point_on(pole: np.ndarray, hint: np.ndarray) -> np.ndarray:
    """pole 对应大圆上最靠近 hint 的点（用来摆标签）。"""
    q = hint - np.dot(hint, unit(pole)) * unit(pole)
    return unit(q) * RADIUS
