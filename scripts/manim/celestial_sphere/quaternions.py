"""四元数与对偶四元数：转动与刚体运动的紧凑表示。

1. 欧拉转动定理：公元前 1000 年（T = −30）的岁差矩阵 P 是绕某一根轴 n 转一个角 φ；
   四元数 q = cos(φ/2) + n·sin(φ/2) 记下这根轴与这个角，slerp 插值即沿这条最短路径匀速转过去。
2. 三次旋转的三个元素四元数相乘 q₃⊗q₂⊗q₁ 得到同一个 q（与矩阵连乘对应）。
3. 对偶四元数：日心黄道架 → 地心赤道架 是"平移到地球 + 绕 x 转 ε"的刚体运动；按沙勒定理
   它等价于绕某一根空间直线的螺旋运动（边转边沿该线平移），ScLERP 沿此螺旋插值。
   地球取日心黄经 180°（春分日，太阳在春分点方向），日地距画作 3 个单位；ε = 23.44° 为真实值。
   此时螺旋轴恰好是日地连线（x 轴），三轴端点画出螺旋线。

渲染：scripts/render_manim.sh celestial_sphere/quaternions
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
    DashedLine,
    Dot3D,
    FadeIn,
    FadeOut,
    Line3D,
    TracedPath,
    ValueTracker,
    VGroup,
    always_redraw,
    linear,
    smooth,
)

from _sky import (
    OBLIQUITY,
    RADIUS,
    SkyScene,
    equator_pole,
    great_circle,
    small_circle,
    sphere_wireframe,
    unit,
)
from _style import (
    BACKGROUND,
    COLOR_ECLIPTIC,
    COLOR_EPOCH,
    COLOR_HIGHLIGHT,
    COLOR_MEAN,
    COLOR_SUN,
    caption,
    label,
    rich,
)

AS = np.pi / 180 / 3600
T = -30.0
AXIS_COLORS = ["#FC8181", "#68D391", "#63B3ED"]


def poly(c: list[float]) -> float:
    return sum(a * T**k for k, a in enumerate(c)) * AS


ZETA = poly([2.650545, 2306.083227, 0.2988499, 0.01801828, -5.971e-6, -3.173e-7])
Z = poly([-2.650545, 2306.077181, 1.0927348, 0.01826837, -0.000028596, -2.904e-7])
THETA = poly([0.0, 2004.191903, -0.4294934, -0.04182264, -7.089e-6, -1.274e-7])


def active(n: np.ndarray, a: float) -> np.ndarray:
    n = unit(n)
    K = np.array([[0, -n[2], n[1]], [n[2], 0, -n[0]], [-n[1], n[0], 0]])
    return np.eye(3) + np.sin(a) * K + (1 - np.cos(a)) * K @ K


# ---- 四元数（标量在前）
def q_axis(n: np.ndarray, a: float) -> np.ndarray:
    return np.concatenate([[np.cos(a / 2)], unit(n) * np.sin(a / 2)])


def q_mul(p: np.ndarray, q: np.ndarray) -> np.ndarray:
    w1, v1, w2, v2 = p[0], p[1:], q[0], q[1:]
    return np.concatenate([[w1 * w2 - v1 @ v2], w1 * v2 + w2 * v1 + np.cross(v1, v2)])


def axis_angle(R: np.ndarray) -> tuple[np.ndarray, float]:
    ang = np.arccos(np.clip((np.trace(R) - 1) / 2, -1, 1))
    n = np.array([R[2, 1] - R[1, 2], R[0, 2] - R[2, 0], R[1, 0] - R[0, 1]]) / (2 * np.sin(ang))
    return unit(n), ang


EX = np.array([1.0, 0, 0])
EZ = np.array([0, 0, 1.0])
EY = np.array([0, 1.0, 0])

# J2000 平赤道三轴 → 当日平赤道三轴 的主动转动（在 J2000 赤道坐标中）：体轴依次 z(−ζ)、y(θ)、z(−z)
F1 = active(EZ, -ZETA)
F2 = F1 @ active(EY, THETA)
F3 = F2 @ active(EZ, -Z)
N_EQU, PHI = axis_angle(F3)
# 体轴序列对应四元数：q = q₁ ⊗ q₂ ⊗ q₃（体轴转动右乘）
Q1, Q2, Q3 = q_axis(EZ, -ZETA), q_axis(EY, THETA), q_axis(EZ, -Z)
Q = q_mul(q_mul(Q1, Q2), Q3)
assert np.allclose(Q if Q[0] > 0 else -Q, q_axis(N_EQU, PHI), atol=1e-12)

# 场景坐标（黄道为 xy 面）中的 J2000 赤道三轴
E = np.column_stack([EX, np.cross(equator_pole(), EX), equator_pole()])
N_SCENE = E @ N_EQU

# ---- 对偶四元数：x' = R x + t，R = 绕 x 转 −ε（黄道三轴 → 赤道三轴），t = 地球日心位置
R_EQ = active(EX, -OBLIQUITY)
EARTH_LON = 180 * DEGREES
T_EARTH = 3.0 * np.array([np.cos(EARTH_LON), np.sin(EARTH_LON), 0.0])
N_SCREW, TH_SCREW = axis_angle(R_EQ)
D_SCREW = T_EARTH @ N_SCREW
T_PERP = T_EARTH - D_SCREW * N_SCREW
C_SCREW = 0.5 * (T_PERP + np.cross(N_SCREW, T_PERP) / np.tan(TH_SCREW / 2))


def screw(s: float) -> tuple[np.ndarray, np.ndarray]:
    """ScLERP 第 s 份：返回 (转动矩阵, 原点位置)。"""
    Rs = active(N_SCREW, s * TH_SCREW)
    return Rs, C_SCREW - Rs @ C_SCREW + s * D_SCREW * N_SCREW


_R1, _o1 = screw(1.0)
assert np.allclose(_R1, R_EQ, atol=1e-12) and np.allclose(_o1, T_EARTH, atol=1e-12)


def fq(q: np.ndarray) -> str:
    """四元数分量，供放进 $…$ 的 LaTeX 用；舍入为零的分量去掉负号。"""
    q = np.where(np.abs(q) < 5e-5, 0.0, q)
    return rf"({q[0]:.4f},\ {q[1]:.4f},\ {q[2]:.4f},\ {q[3]:.4f})"


class Quaternions(SkyScene):
    def construct(self):
        self.camera.background_color = BACKGROUND
        self.set_camera_orientation(phi=64 * DEGREES, theta=-55 * DEGREES, zoom=0.9, frame_center=LEFT * 1.6 + UP * 0.5)

        head = caption("欧拉转动定理：岁差矩阵 = 绕一根轴转一个角", size=34).to_corner(UL)
        self.fix(head)
        self.add(head)
        wire = sphere_wireframe()
        eq_j = great_circle(E[:, 2], COLOR_EPOCH, width=3)
        eq_d = great_circle(E @ F3[:, 2], COLOR_MEAN, width=3, stroke_opacity=0.8)
        ecl = great_circle(OUT, COLOR_ECLIPTIC, width=2, stroke_opacity=0.6)
        self.play(FadeIn(wire), Create(ecl), Create(eq_j), Create(eq_d), run_time=1.2)

        s = ValueTracker(0.0)

        def frame_now() -> np.ndarray:
            return E @ active(N_EQU, s.get_value() * PHI)

        def triad():
            Fn = frame_now()
            g = VGroup()
            for i in range(3):
                tip = Fn[:, i] * RADIUS * 1.15
                g.add(Line3D(np.zeros(3), tip, color=AXIS_COLORS[i], thickness=0.018), Dot3D(tip, radius=0.06, color=AXIS_COLORS[i]))
            return g

        tri = always_redraw(triad)
        axis_line = Line3D(-N_SCENE * (RADIUS + 0.6), N_SCENE * (RADIUS + 0.6), color=COLOR_SUN, thickness=0.02)
        lab_n = rich(r"转轴 $\mathbf{n}$", size=24, color=COLOR_SUN).move_to(N_SCENE * (RADIUS + 0.95))
        self.face(lab_n)
        # x 轴端点绕 n 走过的小圆
        cone_angle = np.arccos(np.clip(E[:, 0] @ N_SCENE, -1, 1))
        cone = small_circle(N_SCENE, cone_angle, COLOR_SUN, r=RADIUS * 1.15, width=1.5, stroke_opacity=0.45)
        info = VGroup(
            rich(r"公元前 1000 年：$P = R_3(-z_A)\,R_2(\theta_A)\,R_3(-\zeta_A)$", size=22),
            rich(rf"合成后只是绕 $\mathbf{{n}}$ 转 $\varphi = {np.degrees(PHI):.2f}^\circ$", size=22, color=COLOR_HIGHLIGHT),
            rich(r"$\mathbf{n}$ 几乎就是黄极，$\varphi$ 即三千年的黄经总岁差", size=22, color=COLOR_HIGHLIGHT),
            rich(rf"四元数 $q = \cos\tfrac{{\varphi}}{{2}} + \mathbf{{n}}\sin\tfrac{{\varphi}}{{2}}$" "\n" rf"$\phantom{{q}} = {fq(Q)}$", size=22),
            caption("slerp：沿这条最短路径匀速插值", size=22),
        ).arrange(DOWN, aligned_edge=LEFT, buff=0.12).to_corner(UL).shift(DOWN * 0.75)
        self.fix(info)
        self.play(FadeIn(tri), Create(axis_line), FadeIn(lab_n), Create(cone), FadeIn(info[:3]), run_time=1.2)
        self.play(FadeIn(info[3:]), run_time=0.6)
        self.play(s.animate.set_value(1.0), run_time=4, rate_func=smooth)
        self.wait(0.8)

        # 元素四元数相乘
        prod = VGroup(
            rich(rf"绕 $z$ 转 $-\zeta_A$：$q_1 = {fq(Q1)}$", size=20),
            rich(rf"绕 $y$ 转 $\theta_A$：$q_2 = {fq(Q2)}$", size=20),
            rich(rf"绕 $z$ 转 $-z_A$：$q_3 = {fq(Q3)}$", size=20),
            rich(rf"$q_1 \otimes q_2 \otimes q_3 = {fq(Q)}$，与上面的 $q$ 相同", size=20, color=COLOR_HIGHLIGHT),
            caption("矩阵 9 个数、四元数 4 个数；连乘后只需归一化即可消除舍入漂移", size=20),
        ).arrange(DOWN, aligned_edge=LEFT, buff=0.12).to_corner(DOWN + LEFT)
        self.fix(prod)
        self.play(FadeIn(prod), run_time=1)
        self.play(s.animate.set_value(0.0), run_time=1.5, rate_func=smooth)
        self.play(s.animate.set_value(1.0), run_time=3, rate_func=smooth)
        self.wait(2.5)
        tri.clear_updaters()
        self.play(*[FadeOut(m) for m in (head, info, prod, tri, axis_line, lab_n, cone, wire, eq_j, eq_d, ecl)], run_time=0.8)

        # ---- 对偶四元数：螺旋运动
        head2 = caption("对偶四元数：平移 + 转动 = 一次螺旋运动", size=34).to_corner(UL)
        self.move_camera(phi=62 * DEGREES, theta=-35 * DEGREES, zoom=0.95, run_time=0.01)
        self.fix(head2)
        orbit = great_circle(OUT, COLOR_ECLIPTIC, r=3.0, width=2.5)
        sun = Dot3D(np.zeros(3), radius=0.2, color=COLOR_SUN)
        earth = Dot3D(T_EARTH, radius=0.12, color="#4299E1")
        lab_sun = label("太阳", size=24, color=COLOR_SUN).move_to(np.array([0, 0, -0.5]))
        lab_earth = label("地球", size=24, color="#4299E1").move_to(T_EARTH + np.array([0, 0, -0.5]))
        lab_orbit = label("地球轨道（黄道面）", size=22, color=COLOR_ECLIPTIC).move_to(np.array([0, 3.4, 0]))
        self.face(lab_sun, lab_earth, lab_orbit)
        self.play(FadeIn(head2), Create(orbit), FadeIn(sun), FadeIn(earth), FadeIn(lab_sun), FadeIn(lab_earth), FadeIn(lab_orbit), run_time=1.2)

        u = ValueTracker(0.0)

        def triad2():
            Rs, o = screw(u.get_value())
            g = VGroup()
            for i in range(3):
                g.add(Line3D(o, o + Rs[:, i] * 1.3, color=AXIS_COLORS[i], thickness=0.02), Dot3D(o + Rs[:, i] * 1.3, radius=0.05, color=AXIS_COLORS[i]))
            return g

        tri2 = always_redraw(triad2)
        a0, a1 = C_SCREW - N_SCREW * 4.5, C_SCREW + N_SCREW * 4.5
        screw_axis = DashedLine(a0, a1, color=COLOR_HIGHLIGHT, stroke_width=3)
        lab_ax = label("螺旋轴", size=22, color=COLOR_HIGHLIGHT).move_to(a0 + np.array([0, 0, 0.35]))
        self.face(lab_ax)
        txt = VGroup(
            rich(r"起点：日心黄道架（原点在太阳，$z$ 指黄极）", size=22),
            rich(r"终点：地心赤道架（原点在地球，$z$ 指天极）：平移 $\mathbf{t}$，再绕 $x$ 转 $\varepsilon$", size=22),
            caption("沙勒定理：任何刚体运动都等于绕某根直线转、同时沿它移", size=22, color=COLOR_HIGHLIGHT),
            rich(rf"春分日：螺旋轴就是日地连线；转 ${np.degrees(TH_SCREW):.2f}^\circ = \varepsilon$，沿轴移 1 天文单位", size=22),
            rich(r"对偶四元数 $\hat q = q + \hat\varepsilon\,\tfrac{1}{2}\,\mathbf{t}\,q$：一次记下转动与平移，复合即相乘", size=22),
        ).arrange(DOWN, aligned_edge=LEFT, buff=0.12).to_corner(UL).shift(DOWN * 0.75)
        self.fix(txt)
        self.play(FadeIn(tri2), FadeIn(txt[:2]), run_time=1)
        self.play(Create(screw_axis), FadeIn(lab_ax), FadeIn(txt[2:4]), run_time=1.2)
        trails = VGroup(*[
            TracedPath(lambda i=i: (lambda Rs, o: o + Rs[:, i] * 1.3)(*screw(u.get_value())), stroke_color=AXIS_COLORS[i], stroke_width=3)
            for i in (1, 2)
        ])
        self.add(trails)
        self.begin_ambient_camera_rotation(rate=0.05)
        self.play(u.animate.set_value(1.0), run_time=5, rate_func=linear)
        self.play(FadeIn(txt[4]), run_time=0.6)
        self.wait(3)
        self.stop_ambient_camera_rotation()
