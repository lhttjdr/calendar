"""岁差：北天极绕黄极作圆锥进动，赤道随之转动，春分点沿黄道西退；末段演示黄道自身的转动。

日月岁差部分按真实几何：ε = 23.44°，周期按当前速率约 25 772 年匀速近似（实际速率与 ε 都缓慢变化，
千年尺度内此近似足够示意）。右枢、北极星、织女星用 J2000 赤经赤纬换算的黄道坐标（不计自行）。
行星岁差（黄道面的转动，约 0.47″/年）在末段放大显示，明确标注为夸张。

渲染：scripts/render_manim.sh celestial_sphere/precession
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
    Line3D,
    TracedPath,
    ValueTracker,
    VGroup,
    always_redraw,
    linear,
    there_and_back,
)

from _sky import (
    SkyScene,
    OBLIQUITY,
    RADIUS,
    arc_along,
    ecl,
    equator_pole,
    great_circle,
    rotate,
    small_circle,
    sphere_wireframe,
    star,
)
from _style import (
    BACKGROUND,
    COLOR_ECLIPTIC,
    COLOR_HIGHLIGHT,
    COLOR_MEAN,
    COLOR_STAR,
    caption,
    label,
    rich,
)

PERIOD_YEARS = 25772.0
YEAR0, YEAR1 = -3000.0, 16000.0

# J2000 赤经、赤纬（度）
STARS = {
    r"右枢 $\alpha$ Dra": (211.097, 64.376),
    r"北极星 $\alpha$ UMi": (37.955, 89.264),
    r"织女一 $\alpha$ Lyr": (279.235, 38.784),
}


def equ_to_ecl(ra_deg: float, dec_deg: float) -> tuple[float, float]:
    ra, dec, e = np.radians(ra_deg), np.radians(dec_deg), OBLIQUITY
    x = np.cos(dec) * np.cos(ra)
    y = np.cos(dec) * np.sin(ra) * np.cos(e) + np.sin(dec) * np.sin(e)
    z = -np.cos(dec) * np.sin(ra) * np.sin(e) + np.sin(dec) * np.cos(e)
    return np.arctan2(y, x), np.arcsin(z)


def node_lon(year: float) -> float:
    """春分点相对 J2000 春分点的黄经（西退为负）。"""
    return -2 * np.pi * (year - 2000.0) / PERIOD_YEARS


def year_text(y: float) -> str:
    y = int(round(y))
    return f"公元前 {1 - y} 年" if y <= 0 else f"公元 {y} 年"


class Precession(SkyScene):
    def construct(self):
        self.camera.background_color = BACKGROUND
        self.set_camera_orientation(phi=20 * DEGREES, theta=-90 * DEGREES, zoom=0.78)

        title = caption("岁差：天极绕黄极转一圈约 2.6 万年", size=34).to_corner(UL)
        self.add_fixed_in_frame_mobjects(title)

        wire = sphere_wireframe()
        earth = Dot3D(radius=0.16, color="#4299E1")
        ecliptic = great_circle(OUT, COLOR_ECLIPTIC, width=4)
        ecl_axis = Line3D(np.zeros(3), OUT * (RADIUS + 0.5), color=COLOR_ECLIPTIC, thickness=0.01)
        lab_ecl = label("黄道（不动）", size=24, color=COLOR_ECLIPTIC).move_to(ecl(-150 * DEGREES, 0, RADIUS * 1.15))
        lab_kp = label("黄极", size=24, color=COLOR_ECLIPTIC).move_to(OUT * (RADIUS + 0.75))
        self.face(lab_ecl, lab_kp)

        cone = small_circle(OUT, OBLIQUITY, COLOR_MEAN, width=1.5, stroke_opacity=0.45)
        stars = VGroup()
        for name, (ra, de) in STARS.items():
            lo, la = equ_to_ecl(ra, de)
            p = ecl(lo, la)
            stars.add(star(p, COLOR_STAR, radius=0.06))
            t = rich(name, size=20, color=COLOR_STAR).move_to(p * 1.08 + OUT * 0.22)
            self.face(t)
            stars.add(t)

        self.play(FadeIn(wire), FadeIn(earth), Create(ecliptic), Create(ecl_axis), FadeIn(lab_ecl), FadeIn(lab_kp), run_time=1.5)

        year = ValueTracker(YEAR0)

        def pole():
            return equator_pole(OBLIQUITY, node_lon(year.get_value()))

        def vernal():
            return ecl(node_lon(year.get_value()))

        equator = always_redraw(lambda: great_circle(pole(), COLOR_MEAN, width=4))
        axis = always_redraw(lambda: Line3D(np.zeros(3), pole() * (RADIUS + 0.5), color=COLOR_MEAN, thickness=0.012))
        pole_dot = always_redraw(lambda: Dot3D(pole() * RADIUS, radius=0.08, color=COLOR_MEAN))
        v_dot = always_redraw(lambda: Dot3D(vernal(), radius=0.09, color=COLOR_HIGHLIGHT))
        lab_eq = label("赤道", size=24, color=COLOR_MEAN)
        lab_v = label("春分点", size=24, color=COLOR_HIGHLIGHT)
        self.face(lab_eq, lab_v)
        lab_eq.add_updater(lambda m: m.move_to(_on_equator(pole(), 1.15)))
        lab_v.add_updater(lambda m: m.move_to(vernal() * 1.2 - OUT * 0.3))

        note = caption("日、月对地球赤道隆起的引力矩，使自转轴绕黄极缓慢画圆锥", size=26).to_edge(DOWN)
        self.fix(note)
        self.play(Create(cone), FadeIn(stars), Create(equator), Create(axis), FadeIn(pole_dot),
                  FadeIn(v_dot), FadeIn(lab_eq), FadeIn(lab_v), FadeIn(note), run_time=2)
        self.wait(1.5)

        def readout():
            y = year.get_value()
            moved = (y - 2000.0) / PERIOD_YEARS * 360.0
            t = caption(f"{year_text(y)}    春分点相对 J2000 已移 {-moved:+6.1f}°（向西为负）", size=26)
            return t.next_to(title, DOWN, aligned_edge=LEFT)

        ro = readout()
        self.add_fixed_in_frame_mobjects(ro)

        def refresh(m):
            m.become(readout())
            self.add_fixed_in_frame_mobjects(m)

        ro.add_updater(refresh)

        pole_trail = TracedPath(lambda: pole() * RADIUS, stroke_color=COLOR_MEAN, stroke_width=5)
        v_trail = TracedPath(vernal, stroke_color=COLOR_HIGHLIGHT, stroke_width=3, stroke_opacity=0.7)
        self.add(pole_trail, v_trail)
        self.play(FadeOut(note), run_time=0.5)
        note2 = caption("天极依次靠近右枢、北极星、织女一；春分点沿黄道向西退，约 50″/年", size=26).to_edge(DOWN)
        self.fix(note2)
        self.play(FadeIn(note2), run_time=0.5)
        self.begin_ambient_camera_rotation(rate=0.02)
        self.play(year.animate.set_value(YEAR1), run_time=16, rate_func=linear)
        self.stop_ambient_camera_rotation()
        ro.clear_updaters()
        self.wait(1)

        # 行星岁差：黄道面本身也转（放大显示）
        self.play(FadeOut(note2), run_time=0.5)
        note3 = caption("行星摄动使黄道面本身也在转：约 0.47″/年（图中转角放大，只示意方向）", size=26).to_edge(DOWN)
        self.fix(note3)
        tilt = ValueTracker(0.0)
        # 黄道绕约 174° 黄经方向的轴转动（当前行星岁差的转轴方向），放大到数度
        node_axis = ecl(174 * DEGREES)
        moving_ecl = always_redraw(lambda: great_circle(rotate(OUT, tilt.get_value(), node_axis), COLOR_ECLIPTIC, width=4))
        old_ecl = great_circle(OUT, COLOR_ECLIPTIC, width=2, stroke_opacity=0.35)
        self.remove(ecliptic)
        self.add(old_ecl, moving_ecl)
        self.play(FadeIn(note3), run_time=0.5)
        self.play(tilt.animate.set_value(6 * DEGREES), run_time=4, rate_func=there_and_back)
        self.wait(1)


def _on_equator(pole: np.ndarray, scale: float) -> np.ndarray:
    """赤道上靠近画面前下方的一点，用来放「赤道」标签。"""
    hint = ecl(-60 * DEGREES)
    q = hint - np.dot(hint, pole) * pole
    return q / np.linalg.norm(q) * RADIUS * scale
