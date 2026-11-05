"""章动：真天极绕平天极的小幅摆动。

左：月球轨道升交点黄经 Ω 每 18.6 年逆行一周——章动主项的来源。
右：以平极为原点的切平面（单位角秒），横轴为沿岁差方向的 Δψ·sin ε，纵轴为背离黄极方向的 Δε；
    真极在其上描出章动轨迹。级数取 Meeus《Astronomical Algorithms》第 22 章的四项近似
    （18.6 年主项、半年项、半月项、9.3 年项），按真实时间推进，非示意。
末段：叠加岁差（平极沿岁差方向每年移 50.29″·sin ε ≈ 20.0″），横纵同比例画出真极的波浪轨迹。

渲染：scripts/render_manim.sh celestial_sphere/nutation
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import numpy as np
from manim import (
    DOWN,
    LEFT,
    RIGHT,
    UL,
    UP,
    Arrow,
    Circle,
    Create,
    DashedLine,
    Dot,
    FadeIn,
    FadeOut,
    Line,
    ParametricFunction,
    Scene,
    ValueTracker,
    VGroup,
    always_redraw,
    linear,
)

from _style import (
    BACKGROUND,
    COLOR_ECLIPTIC,
    COLOR_FIXED_FRAME,
    COLOR_HIGHLIGHT,
    COLOR_MEAN,
    COLOR_TRUE,
    GRID_DIM,
    caption,
    label,
    rich,
)

SIN_EPS = np.sin(np.radians(23.44))
PREC_RATE = 50.29 * SIN_EPS  # 平极沿岁差圆每年移动的角秒数
JY = 365.25
SPAN_YEARS = 18.6


def args(t_years: float) -> tuple[float, float, float]:
    """自 J2000 起 t 年的 Ω、L（太阳平黄经）、L'（月球平黄经），弧度。"""
    T = t_years / 100.0
    om = np.radians(125.04452 - 1934.136261 * T)
    L = np.radians(280.4665 + 36000.7698 * T)
    Lp = np.radians(218.3165 + 481267.8813 * T)
    return om, L, Lp


def nutation(t_years: float) -> tuple[float, float]:
    """(Δψ, Δε)，角秒；Meeus 22 章四项近似。"""
    om, L, Lp = args(t_years)
    dpsi = -17.20 * np.sin(om) - 1.32 * np.sin(2 * L) - 0.23 * np.sin(2 * Lp) + 0.21 * np.sin(2 * om)
    deps = 9.20 * np.cos(om) + 0.57 * np.cos(2 * L) + 0.10 * np.cos(2 * Lp) - 0.09 * np.cos(2 * om)
    return dpsi, deps


def node_years_start() -> float:
    """取 Ω = 0 的时刻作起点，主项椭圆从顶端开始，便于看清一周。"""
    return (125.04452 / 1934.136261) * 100.0


class Nutation(Scene):
    def construct(self):
        self.camera.background_color = BACKGROUND
        title = caption("章动：真天极绕平天极的摆动（主周期 18.6 年）", size=34).to_corner(UL)
        self.add(title)

        t0 = node_years_start()
        t = ValueTracker(t0)

        # ---- 左：月球轨道升交点逆行（自北黄极俯视黄道面）
        lc = np.array([-4.4, 0.0, 0])
        R = 1.5
        ecl_circle = Circle(radius=R, color=COLOR_ECLIPTIC, stroke_width=3).move_to(lc)
        vernal_ray = DashedLine(lc, lc + RIGHT * (R + 0.35), color=GRID_DIM)
        lab_v = label("春分点方向", size=20).next_to(lc + RIGHT * (R + 0.35), DOWN, buff=0.1)
        lab_l = label("黄道（自北黄极俯视）", size=22, color=COLOR_ECLIPTIC).next_to(ecl_circle, UP, buff=0.2)

        def node_dir():
            om, _, _ = args(t.get_value())
            return np.array([np.cos(om), np.sin(om), 0])

        node_line = always_redraw(lambda: Line(lc - node_dir() * R, lc + node_dir() * R, color=COLOR_HIGHLIGHT, stroke_width=4))
        node_dot = always_redraw(lambda: Dot(lc + node_dir() * R, color=COLOR_HIGHLIGHT, radius=0.09))
        lab_node = always_redraw(lambda: label("升交点", size=20, color=COLOR_HIGHLIGHT).move_to(lc + node_dir() * (R + 0.45)))
        lab_cause = label("月球轨道与黄道交线（交点线）\n每 18.6 年逆行一周", size=22).next_to(ecl_circle, DOWN, buff=0.3)

        # ---- 右：平极切平面，单位角秒
        rc = np.array([2.4, -0.3, 0])
        k = 0.24  # 每角秒的画面单位
        x_axis = Arrow(rc + LEFT * 3.0, rc + RIGHT * 3.0, buff=0, color=GRID_DIM, stroke_width=2, tip_length=0.15)
        y_axis = Arrow(rc + DOWN * 2.9, rc + UP * 2.9, buff=0, color=GRID_DIM, stroke_width=2, tip_length=0.15)
        lab_x = rich("$\\Delta\\psi\\,\\sin\\varepsilon$\n沿岁差方向", size=20).next_to(rc + RIGHT * 3.0, RIGHT, buff=0.15)
        lab_y = rich(r"$\Delta\varepsilon$（背离黄极）", size=20).next_to(rc + UP * 2.9, RIGHT, buff=0.15)
        ticks = VGroup()
        for v in (-5, 5):
            ticks.add(Line(rc + RIGHT * v * k + DOWN * 0.06, rc + RIGHT * v * k + UP * 0.06, color=GRID_DIM))
            ticks.add(label(f"{v:+d}″", size=16, color=GRID_DIM).next_to(rc + RIGHT * v * k, DOWN, buff=0.1))
            ticks.add(Line(rc + UP * v * k + LEFT * 0.06, rc + UP * v * k + RIGHT * 0.06, color=GRID_DIM))
            ticks.add(label(f"{v:+d}″", size=16, color=GRID_DIM).next_to(rc + UP * v * k, LEFT, buff=0.1))
        mean_pole = Dot(rc, color=COLOR_MEAN, radius=0.08)
        lab_mp = label("平极", size=20, color=COLOR_MEAN).next_to(rc, UP + LEFT, buff=0.05)

        def tp(tt: float) -> np.ndarray:
            dpsi, deps = nutation(tt)
            return rc + RIGHT * dpsi * SIN_EPS * k + UP * deps * k

        path = always_redraw(
            lambda: ParametricFunction(tp, t_range=[t0, max(t.get_value(), t0 + 1e-3), 1 / 120], color=COLOR_TRUE, stroke_width=2.5)
        )
        true_pole = always_redraw(lambda: Dot(tp(t.get_value()), color=COLOR_TRUE, radius=0.09))
        lab_tp = always_redraw(lambda: label("真极", size=20, color=COLOR_TRUE).next_to(tp(t.get_value()), UP, buff=0.1))

        def readout():
            dpsi, deps = nutation(t.get_value())
            yr = t.get_value() + 2000
            return rich(rf"{yr:7.1f} 年    $\Delta\psi =$ {dpsi:+6.2f}″    $\Delta\varepsilon =$ {deps:+5.2f}″", size=26).next_to(title, DOWN, aligned_edge=LEFT)

        ro = always_redraw(readout)

        self.play(FadeIn(ecl_circle), FadeIn(vernal_ray), FadeIn(lab_v), FadeIn(lab_l), run_time=1)
        self.play(Create(node_line), FadeIn(node_dot), FadeIn(lab_node), FadeIn(lab_cause), run_time=1)
        self.play(FadeIn(x_axis), FadeIn(y_axis), FadeIn(lab_x), FadeIn(lab_y), FadeIn(ticks), FadeIn(mean_pole), FadeIn(lab_mp), run_time=1)
        self.add(path, true_pole, lab_tp, ro)
        note = rich(r"交点线转一周，真极绕平极画一个椭圆：长半轴 9.2″（$\Delta\varepsilon$），短半轴 6.9″（$17.2'' \sin\varepsilon$）", size=24).to_edge(DOWN)
        self.play(FadeIn(note), run_time=0.5)
        self.play(t.animate.set_value(t0 + SPAN_YEARS), run_time=14, rate_func=linear)
        self.wait(0.5)
        self.play(FadeOut(note), run_time=0.4)
        note_b = caption("椭圆上的细小抖动：太阳引起的半年项（±1.3″）与月球引起的半月项（±0.2″）", size=24).to_edge(DOWN)
        self.play(FadeIn(note_b), run_time=0.5)
        self.wait(3)

        # ---- 叠加岁差：等比例
        left_group = VGroup(ecl_circle, vernal_ray, lab_v, lab_l, node_line, node_dot, lab_node, lab_cause)
        right_group = VGroup(x_axis, y_axis, lab_x, lab_y, ticks, mean_pole, lab_mp, path, true_pole, lab_tp)
        ro.clear_updaters()
        for m in (node_line, node_dot, lab_node, path, true_pole, lab_tp):
            m.clear_updaters()
        self.play(FadeOut(left_group), FadeOut(right_group), FadeOut(note_b), FadeOut(ro), run_time=1)

        title2 = caption("叠加岁差：平极匀速前进，真极沿波浪线前进（横纵同比例）", size=34).to_corner(UL)
        self.play(FadeOut(title), FadeIn(title2), run_time=0.6)

        q = 0.032  # 每角秒的画面单位
        origin = np.array([-6.2, -0.2, 0])
        span = SPAN_YEARS * PREC_RATE
        base = Arrow(origin + LEFT * 0.2, origin + RIGHT * (span * q + 0.5), buff=0, color=GRID_DIM, stroke_width=2, tip_length=0.15)
        lab_base = label("岁差方向", size=20).next_to(base.get_end(), DOWN, buff=0.15)
        scale_bar = Line(origin + DOWN * 1.6, origin + DOWN * 1.6 + RIGHT * 100 * q, color=COLOR_FIXED_FRAME, stroke_width=3)
        lab_scale = label("100″", size=20).next_to(scale_bar, DOWN, buff=0.1)

        s = ValueTracker(t0)

        def mp(tt: float) -> np.ndarray:
            return origin + RIGHT * (tt - t0) * PREC_RATE * q

        def tp2(tt: float) -> np.ndarray:
            dpsi, deps = nutation(tt)
            return mp(tt) + RIGHT * dpsi * SIN_EPS * q + UP * deps * q

        mean_path = always_redraw(lambda: Line(mp(t0), mp(max(s.get_value(), t0 + 1e-3)), color=COLOR_MEAN, stroke_width=3))
        true_path = always_redraw(
            lambda: ParametricFunction(tp2, t_range=[t0, max(s.get_value(), t0 + 1e-3), 1 / 120], color=COLOR_TRUE, stroke_width=2.5)
        )
        mp_dot = always_redraw(lambda: Dot(mp(s.get_value()), color=COLOR_MEAN, radius=0.07))
        tp_dot = always_redraw(lambda: Dot(tp2(s.get_value()), color=COLOR_TRUE, radius=0.08))
        legend = VGroup(
            label("— 平极（只计岁差）", size=22, color=COLOR_MEAN),
            label("— 真极（岁差 + 章动）", size=22, color=COLOR_TRUE),
        ).arrange(DOWN, aligned_edge=LEFT).to_corner(UL).shift(DOWN * 1.0)
        self.play(FadeIn(base), FadeIn(lab_base), FadeIn(scale_bar), FadeIn(lab_scale), FadeIn(legend), run_time=0.8)
        self.add(mean_path, true_path, mp_dot, tp_dot)
        self.play(s.animate.set_value(t0 + SPAN_YEARS), run_time=10, rate_func=linear)
        note_c = caption("18.6 年间平极走了约 370″，章动只让真极偏离它不到 10″", size=24).to_edge(DOWN)
        self.play(FadeIn(note_c), run_time=0.5)
        self.wait(3)
