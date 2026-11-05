"""岁差矩阵的旋转分解：三次旋转（ζ_A, θ_A, z_A）、Lieske 四次旋转（ε₀, ψ_A, ω_A, χ_A）、
Fukushima–Williams 四次旋转（γ̄, φ̄, ψ̄, ε_A）。三种分解乘出同一个岁差矩阵，只是中间经过的
参考面不同。

角度取公元前 1000 年（T = −30 儒略世纪）的 IAU 2006（P03）多项式值，真实大小，无夸张；
三者在 T = −30 时互差约 1″（各自多项式截断），画面上不可分辨。FW 起点严格说是 GCRS，
与 J2000 平赤道差一个 frame bias（约 0.02″），画面中视为同一。

转动规则：P = R_a(φ₁)·R_b(φ₂)·R_c(φ₃)（R 为坐标旋转矩阵）等价于把坐标三轴依次绕
当前的 c、b、a 轴按右手方向转 φ₃、φ₂、φ₁——最右边的先转。

渲染：scripts/render_manim.sh celestial_sphere/rotation_sequences
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
    smooth,
)

from _sky import (
    OBLIQUITY,
    RADIUS,
    SkyScene,
    arc,
    equator_pole,
    great_circle,
    sphere_wireframe,
    unit,
)
from _style import (
    BACKGROUND,
    COLOR_ECLIPTIC,
    COLOR_EPOCH,
    COLOR_FIXED_FRAME,
    COLOR_HIGHLIGHT,
    COLOR_MEAN,
    caption,
    label,
    rich,
)

AS = np.pi / 180 / 3600
T = -30.0


def poly(c: list[float]) -> float:
    return sum(a * T**k for k, a in enumerate(c)) * AS


ZETA = poly([2.650545, 2306.083227, 0.2988499, 0.01801828, -5.971e-6, -3.173e-7])
Z = poly([-2.650545, 2306.077181, 1.0927348, 0.01826837, -0.000028596, -2.904e-7])
THETA = poly([0.0, 2004.191903, -0.4294934, -0.04182264, -7.089e-6, -1.274e-7])
EPS0 = 84381.406 * AS
PSI_A = poly([0.0, 5038.481507, -1.0790069, -0.00114045, 0.000132851, -9.51e-8])
OMEGA_A = poly([84381.406, -0.025754, 0.0512623, -0.00772503, -4.67e-7, 3.337e-7])
CHI_A = poly([0.0, 10.556403, -2.3814292, -0.00121197, 0.000170663, -5.60e-8])
GAMMA_B = poly([-0.052928, 10.556378, 0.4932044, -0.00031238, -2.788e-6, 2.60e-8])
PHI_B = poly([84381.412819, -46.811016, 0.0511268, 0.00053289, -4.40e-7, -1.76e-8])
PSI_B = poly([-0.041775, 5038.481484, 1.5584175, -0.00018522, -0.000026452, -1.48e-8])
EPS_A = poly([84381.406, -46.836769, -0.0001831, 0.00200340, -5.76e-7, -4.34e-8])

AXIS_COLORS = ["#FC8181", "#68D391", "#63B3ED"]  # x 红、y 绿、z 蓝
AXIS_NAMES = ["x", "y", "z"]


def active(axis: int, a: float) -> np.ndarray:
    """绕坐标轴 axis 右手转 a 的主动旋转矩阵。"""
    n = np.eye(3)[axis]
    K = np.array([[0, -n[2], n[1]], [n[2], 0, -n[0]], [-n[1], n[0], 0]])
    return np.eye(3) + np.sin(a) * K + (1 - np.cos(a)) * K @ K


def coord_rot(axis: int, a: float) -> np.ndarray:
    """坐标旋转矩阵 R_axis(a)（天文惯用，= 主动旋转 −a）。"""
    return active(axis, -a)


# 场景坐标：黄道为 xy 面。J2000 平赤道坐标三轴在场景坐标中的列向量：
E = np.column_stack([np.array([1.0, 0, 0]), np.cross(equator_pole(), [1.0, 0, 0]), equator_pole()])

P_TARGET = coord_rot(2, -Z) @ coord_rot(1, THETA) @ coord_rot(2, -ZETA)
TARGET_POLE = E @ P_TARGET[2]  # 当日平极（P 的第 3 行即当日 z 轴在 J2000 坐标中的方向）
TARGET_X = E @ P_TARGET[0]

# 每种分解：标题、公式（LaTeX）、步骤 (轴, 转角, 说明（$…$ 内为 LaTeX）, 角名（LaTeX）)，按执行顺序（公式最右边先）
SEQUENCES = [
    (
        "三次旋转（IAU 1976 起沿用，P03 系数）",
        r"$P = R_3(-z_A)\,R_2(\theta_A)\,R_3(-\zeta_A)$",
        [
            (2, -ZETA, r"绕天极转 $-\zeta_A$：$y$ 轴转到两赤道的交线上", r"\zeta_A"),
            (1, THETA, r"绕交线转 $\theta_A$：J2000 赤道倾成当日赤道", r"\theta_A"),
            (2, -Z, r"绕当日天极转 $-z_A$：$x$ 轴落到当日春分点", r"z_A"),
        ],
    ),
    (
        "四次旋转（Lieske 1977；P03 系数）",
        r"$P = R_3(\chi_A)\,R_1(-\omega_A)\,R_3(-\psi_A)\,R_1(\varepsilon_0)$",
        [
            (0, EPS0, r"绕 $x$ 转 $\varepsilon_0$：J2000 赤道 → J2000 黄道", r"\varepsilon_0"),
            (2, -PSI_A, r"绕黄极转 $-\psi_A$：沿 J2000 黄道到当日分点（日月岁差）", r"\psi_A"),
            (0, -OMEGA_A, r"绕 $x$ 转 $-\omega_A$：J2000 黄道 → 当日赤道", r"\omega_A"),
            (2, CHI_A, r"绕当日天极转 $\chi_A$：行星岁差引起的小改正", r"\chi_A"),
        ],
    ),
    (
        "四次旋转（Fukushima–Williams；IAU 2006 推荐）",
        r"$P = R_1(-\varepsilon_A)\,R_3(-\bar\psi)\,R_1(\bar\varphi)\,R_3(\bar\gamma)$",
        [
            (2, GAMMA_B, r"绕天极转 $\bar\gamma$：$x$ 轴到当日黄道与 J2000 赤道的交点（很小）", r"\bar\gamma"),
            (0, PHI_B, r"绕 $x$ 转 $\bar\varphi$：J2000 赤道 → 当日黄道", r"\bar\varphi"),
            (2, -PSI_B, r"绕当日黄极转 $-\bar\psi$：沿当日黄道到当日分点", r"\bar\psi"),
            (0, -EPS_A, r"绕 $x$ 转 $-\varepsilon_A$：当日黄道 → 当日赤道", r"\varepsilon_A"),
        ],
    ),
]


VALUES = {
    r"\zeta_A": ZETA, r"\theta_A": THETA, r"z_A": Z,
    r"\varepsilon_0": EPS0, r"\psi_A": PSI_A, r"\omega_A": OMEGA_A, r"\chi_A": CHI_A,
    r"\bar\gamma": GAMMA_B, r"\bar\varphi": PHI_B, r"\bar\psi": PSI_B, r"\varepsilon_A": EPS_A,
}


def fmt(v: float) -> str:
    return f"{v / AS:+.0f}″" if abs(v) < 1 * DEGREES else f"{np.degrees(v):+.2f}°"


def verify() -> None:
    """三种分解各自乘出的矩阵与三次旋转一致（≤2″），体轴执行顺序复现 Pᵀ。"""
    for _, _, steps in SEQUENCES:
        F = np.eye(3)
        for ax, a, _, _ in steps:
            F = F @ active(ax, a)
        M = F.T @ P_TARGET.T
        err = np.degrees(np.arccos(np.clip((np.trace(M) - 1) / 2, -1, 1))) * 3600
        assert err < 2.0, err


verify()


class RotationSequences(SkyScene):
    def construct(self):
        self.camera.background_color = BACKGROUND
        self.set_camera_orientation(phi=64 * DEGREES, theta=-55 * DEGREES, zoom=0.85, frame_center=LEFT * 1.6 + UP * 0.4)

        wire = sphere_wireframe()
        ecliptic = great_circle(OUT, COLOR_ECLIPTIC, width=2, stroke_opacity=0.6)
        eq_j = great_circle(E[:, 2], COLOR_EPOCH, width=3)
        eq_d = great_circle(TARGET_POLE, COLOR_MEAN, width=3, stroke_opacity=0.8)
        target_x = Dot3D(TARGET_X * RADIUS, radius=0.08, color=COLOR_MEAN)
        legend = VGroup(
            label("— J2000 平赤道（起点）", size=20, color=COLOR_EPOCH),
            label("— 公元前 1000 年平赤道（终点）", size=20, color=COLOR_MEAN),
            label("— 黄道", size=20, color=COLOR_ECLIPTIC),
        ).arrange(DOWN, aligned_edge=LEFT, buff=0.08).to_corner(DOWN + LEFT)
        self.fix(legend)
        head = caption("岁差矩阵：同一个转动，三种拆法", size=34).to_corner(UL)
        self.fix(head)
        self.add(head)
        self.play(FadeIn(wire), Create(ecliptic), Create(eq_j), Create(eq_d), FadeIn(target_x), FadeIn(legend), run_time=1.5)

        for title, formula, steps in SEQUENCES:
            self._run(title, formula, steps)

        end = caption("三种拆法终点相同：P 是同一个矩阵（本例互差约 1″，来自各自多项式截断）", size=24).to_corner(UL).shift(DOWN * 0.75)
        self.fix(end)
        self.play(FadeIn(end), run_time=0.6)
        self.wait(3)

    def _run(self, title: str, formula: str, steps) -> None:
        F = np.eye(3)  # 体轴（列）在 J2000 平赤道坐标中
        frame = {"F": F}
        tracker = ValueTracker(0.0)
        cur = {"ax": 2}

        def current() -> np.ndarray:
            return frame["F"] @ active(cur["ax"], tracker.get_value())

        def triad():
            Fn = E @ current()
            g = VGroup()
            for i in range(3):
                tip = Fn[:, i] * RADIUS * 1.15
                g.add(Line3D(np.zeros(3), tip, color=AXIS_COLORS[i], thickness=0.018))
                g.add(Dot3D(tip, radius=0.06, color=AXIS_COLORS[i]))
            return g

        def body_equator():
            return great_circle(E @ current()[:, 2], COLOR_FIXED_FRAME, width=2.5, stroke_opacity=0.9)

        tri = always_redraw(triad)
        beq = always_redraw(body_equator)
        tips = VGroup(*[rich(f"${AXIS_NAMES[i]}$", size=26, color=AXIS_COLORS[i]) for i in range(3)])
        for i, t in enumerate(tips):
            t.add_updater(lambda m, i=i: m.move_to(E @ current()[:, i] * RADIUS * 1.3))
        self.face(*tips)

        panel_title = caption(title, size=26).to_corner(UL).shift(DOWN * 0.7)
        panel_formula = rich(formula, size=26, color=COLOR_HIGHLIGHT).next_to(panel_title, DOWN, aligned_edge=LEFT, buff=0.15)
        lines = VGroup(*[
            rich(f"{k + 1}. {desc}（${name} =$ {fmt(VALUES[name])}）", size=20)
            for k, (_, a, desc, name) in enumerate(steps)
        ]).arrange(DOWN, aligned_edge=LEFT, buff=0.1).next_to(panel_formula, DOWN, aligned_edge=LEFT, buff=0.2)
        lines.set_opacity(0.35)
        self.fix(panel_title, panel_formula, lines)
        self.play(FadeIn(panel_title), FadeIn(panel_formula), FadeIn(lines), FadeIn(tri), FadeIn(beq), FadeIn(tips), run_time=1)

        arcs = VGroup()
        for k, (ax, a, _, name) in enumerate(steps):
            cur["ax"] = ax
            tracker.set_value(0.0)
            moving = 0 if ax == 2 else 2  # 绕 z 转看 x 怎么走；绕 x/y 转看 z 怎么走
            start_vec = E @ frame["F"][:, moving]
            self.play(lines[k].animate.set_opacity(1.0), run_time=0.3)
            self.play(tracker.animate.set_value(a), run_time=2.2 if abs(a) > 2 * DEGREES else 1.2, rate_func=smooth)
            end_vec = E @ current()[:, moving]
            if abs(a) > 0.5 * DEGREES:
                ar = arc(start_vec, end_vec, COLOR_HIGHLIGHT, r=RADIUS * 1.15, width=5)
                lab = rich(f"${name}$", size=24, color=COLOR_HIGHLIGHT).move_to(unit(start_vec + end_vec) * RADIUS * 1.32)
                self.face(lab)
                self.play(Create(ar), FadeIn(lab), run_time=0.5)
                arcs.add(ar, lab)
            frame["F"] = current()
            cur["ax"] = 2
            tracker.set_value(0.0)
            self.play(lines[k].animate.set_opacity(0.6), run_time=0.2)
        self.wait(1.2)
        tri.clear_updaters()
        beq.clear_updaters()
        for t in tips:
            t.clear_updaters()
        self.play(*[FadeOut(m) for m in (panel_title, panel_formula, lines, tri, beq, tips, arcs)], run_time=0.6)
