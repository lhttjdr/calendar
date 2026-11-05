"""天球基本几何：赤道与黄道、黄赤交角、二分二至，太阳一年沿黄道运行时赤纬的变化，
以及同一颗星的黄道坐标 (λ, β) 与赤道坐标 (α, δ)。

几何按真实比例（ε = 23.44°），无夸张。坐标约定见 _sky.py。

渲染：scripts/render_manim.sh celestial_sphere/sphere_basics
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
    ValueTracker,
    VGroup,
    always_redraw,
    linear,
)

from _sky import (
    SkyScene,
    OBLIQUITY,
    RADIUS,
    arc,
    arc_along,
    ecl,
    equator_pole,
    great_circle,
    sphere_wireframe,
    sph,
    star,
)
from _style import (
    BACKGROUND,
    COLOR_ECLIPTIC,
    COLOR_FIXED_FRAME,
    COLOR_HIGHLIGHT,
    COLOR_MEAN,
    COLOR_SUN,
    COLOR_TRUE,
    caption,
    label,
    rich,
)

POLE = equator_pole()
STAR_LON, STAR_LAT = 60 * DEGREES, 32 * DEGREES


def ecl_to_equ(lon: float, lat: float) -> tuple[float, float]:
    """黄道坐标 → 赤道坐标（同一分点）。"""
    e = OBLIQUITY
    x = np.cos(lat) * np.cos(lon)
    y = np.cos(lat) * np.sin(lon) * np.cos(e) - np.sin(lat) * np.sin(e)
    z = np.cos(lat) * np.sin(lon) * np.sin(e) + np.sin(lat) * np.cos(e)
    return np.arctan2(y, x) % (2 * np.pi), np.arcsin(z)


class SphereBasics(SkyScene):
    def construct(self):
        self.camera.background_color = BACKGROUND
        self.set_camera_orientation(phi=68 * DEGREES, theta=-40 * DEGREES, zoom=0.8, frame_center=OUT * 0.7)

        title = caption("天球：赤道、黄道与黄赤交角", size=34).to_corner(UL)
        self.add_fixed_in_frame_mobjects(title)

        wire = sphere_wireframe()
        earth = Dot3D(radius=0.16, color="#4299E1")
        self.play(FadeIn(wire), FadeIn(earth), run_time=1)

        # 赤道与北天极
        equator = great_circle(POLE, COLOR_MEAN, width=4)
        pole_axis = Line3D(-POLE * (RADIUS + 0.4), POLE * (RADIUS + 0.4), color=COLOR_MEAN, thickness=0.01)
        lab_eq = label("天赤道", size=26, color=COLOR_MEAN).move_to(sph(POLE, RIGHT, 115 * DEGREES, 0, RADIUS * 1.12))
        lab_ncp = label("北天极", size=26, color=COLOR_MEAN).move_to(POLE * (RADIUS + 0.55) + RIGHT * 0.9)
        self.face(lab_eq, lab_ncp)
        self.play(Create(equator), Create(pole_axis), FadeIn(lab_eq), FadeIn(lab_ncp), run_time=2)

        # 黄道与黄极
        ecliptic = great_circle(OUT, COLOR_ECLIPTIC, width=4)
        ecl_axis = Line3D(-OUT * (RADIUS + 0.4), OUT * (RADIUS + 0.4), color=COLOR_ECLIPTIC, thickness=0.01)
        lab_ecl = label("黄道", size=26, color=COLOR_ECLIPTIC).move_to(ecl(215 * DEGREES, 0, RADIUS * 1.12))
        lab_kp = label("黄极", size=26, color=COLOR_ECLIPTIC).move_to(OUT * (RADIUS + 0.55) + LEFT * 0.8)
        self.face(lab_ecl, lab_kp)
        self.play(Create(ecliptic), Create(ecl_axis), FadeIn(lab_ecl), FadeIn(lab_kp), run_time=2)

        # 黄赤交角：两极之间的夹角 = 两个大圆的夹角
        eps_arc = arc(OUT, POLE, COLOR_HIGHLIGHT, r=RADIUS + 0.25, width=6)
        lab_eps = rich(r"$\varepsilon \approx 23.4^\circ$", size=26, color=COLOR_HIGHLIGHT).move_to(
            (OUT + POLE) / np.linalg.norm(OUT + POLE) * (RADIUS - 0.35) + LEFT * 0.75
        )
        self.face(lab_eps)
        note = rich(r"两极夹角 = 两个大圆的夹角 = 黄赤交角 $\varepsilon$", size=26).to_edge(DOWN)
        self.fix(note)
        self.play(Create(eps_arc), FadeIn(lab_eps), FadeIn(note), run_time=1.5)
        self.wait(1)

        # 二分二至：黄道上黄经 0/90/180/270° 四点；二分点即两个大圆的交点
        names = [("春分点", 0), ("夏至点", 90), ("秋分点", 180), ("冬至点", 270)]
        marks = VGroup()
        for name, lon in names:
            p = ecl(lon * DEGREES)
            d = Dot3D(p, radius=0.07, color=COLOR_FIXED_FRAME)
            t = label(name, size=24).move_to(ecl(lon * DEGREES, 0, RADIUS * 1.2) + OUT * 0.25)
            self.face(t)
            marks.add(d, t)
        note2 = caption("赤道与黄道相交于春分点、秋分点；黄经从春分点起沿黄道逆时针量", size=26).to_edge(DOWN)
        self.fix(note2)
        self.play(FadeOut(note), FadeIn(marks), run_time=1)
        self.play(FadeIn(note2), run_time=0.5)
        self.wait(1)

        # 太阳沿黄道走一年：赤纬 δ 随黄经 λ 变化，sin δ = sin ε · sin λ
        lam = ValueTracker(0.0)

        def sun_pos():
            return ecl(lam.get_value())

        def foot_on_equator():
            p = sun_pos()
            q = p - np.dot(p, POLE) * POLE
            return q / np.linalg.norm(q) * RADIUS

        sun = always_redraw(lambda: Dot3D(sun_pos(), radius=0.13, color=COLOR_SUN))
        dec_arc = always_redraw(lambda: arc(foot_on_equator(), sun_pos(), COLOR_TRUE, width=6))

        def readout():
            lv = lam.get_value()
            dec = np.degrees(np.arcsin(np.sin(OBLIQUITY) * np.sin(lv)))
            t = rich(rf"太阳黄经 $\lambda_\odot =$ {np.degrees(lv) % 360:5.1f}°    赤纬 $\delta_\odot =$ {dec:+5.1f}°", size=28)
            return t.next_to(title, DOWN, aligned_edge=LEFT)

        ro = readout()
        self.add_fixed_in_frame_mobjects(ro)
        def refresh(m):
            # become 换掉子对象后须重新登记为屏幕固定层，否则会随 3D 相机投影
            m.become(readout())
            self.add_fixed_in_frame_mobjects(m)

        ro.add_updater(refresh)
        note3 = rich(r"太阳一年沿黄道走一周；橙色弧为它到赤道的角距——赤纬 $\delta$", size=26).to_edge(DOWN)
        self.fix(note3)
        self.play(FadeOut(note2), run_time=0.5)
        self.play(FadeIn(note3), FadeIn(sun), Create(dec_arc), run_time=1)
        self.begin_ambient_camera_rotation(rate=0.06)
        self.play(lam.animate.set_value(2 * np.pi), run_time=12, rate_func=linear)
        self.stop_ambient_camera_rotation()
        ro.clear_updaters()
        self.play(FadeOut(sun), FadeOut(dec_arc), FadeOut(ro), FadeOut(note3), FadeOut(marks), run_time=1)

        # 同一颗星的两套坐标
        s_pos = ecl(STAR_LON, STAR_LAT)
        ra, de = ecl_to_equ(STAR_LON, STAR_LAT)
        st = star(s_pos, radius=0.09)
        lab_star = label("某星", size=24).move_to(s_pos * 1.12 + OUT * 0.2)
        self.face(lab_star)
        vernal = Dot3D(ecl(0), radius=0.08, color=COLOR_FIXED_FRAME)
        lab_v = label("春分点", size=24).move_to(ecl(0, 0, RADIUS * 1.18) + OUT * 0.25)
        self.face(lab_v)
        title2 = rich(r"同一颗星：黄道坐标 $(\lambda, \beta)$ 与赤道坐标 $(\alpha, \delta)$", size=34).to_corner(UL)
        self.play(FadeOut(title), FadeOut(eps_arc), FadeOut(lab_eps), run_time=0.6)
        self.fix(title2)
        self.play(FadeIn(title2), FadeIn(st), FadeIn(lab_star), FadeIn(vernal), FadeIn(lab_v), run_time=1)

        lon_arc = arc_along(OUT, RIGHT, 0, STAR_LON, COLOR_ECLIPTIC, width=8)
        lat_arc = arc(ecl(STAR_LON), s_pos, COLOR_ECLIPTIC, width=8)
        ecl_txt = rich(
            rf"黄经 $\lambda =$ {np.degrees(STAR_LON):.1f}°（沿黄道）   黄纬 $\beta =$ {np.degrees(STAR_LAT):+.1f}°",
            size=26, color=COLOR_ECLIPTIC,
        ).to_edge(DOWN).shift(0.55 * UP)
        self.fix(ecl_txt)
        self.play(Create(lon_arc), run_time=1.5)
        self.play(Create(lat_arc), FadeIn(ecl_txt), run_time=1.2)

        foot = sph(POLE, RIGHT, ra, 0)
        ra_arc = arc_along(POLE, RIGHT, 0, ra, COLOR_MEAN, width=8)
        de_arc = arc(foot, s_pos, COLOR_MEAN, width=8)
        equ_txt = rich(
            rf"赤经 $\alpha =$ {np.degrees(ra):.1f}°（沿赤道）   赤纬 $\delta =$ {np.degrees(de):+.1f}°",
            size=26, color=COLOR_MEAN,
        ).to_edge(DOWN)
        self.fix(equ_txt)
        self.play(Create(ra_arc), run_time=1.5)
        self.play(Create(de_arc), FadeIn(equ_txt), run_time=1.2)
        self.begin_ambient_camera_rotation(rate=0.12)
        self.wait(5)
        self.stop_ambient_camera_rotation()
        self.wait(1)
