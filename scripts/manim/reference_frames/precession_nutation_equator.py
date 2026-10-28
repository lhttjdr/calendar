"""对应 doc/astronomy/reference-frames.md §「动力学坐标、平赤道与真赤道」
与「本程序中的链」：历表 → 平架 → 岁差 P → 章动 N → 真赤道/真黄道。

真实周期（岁差约 25800 年、章动主项约 18.6 年）无法直接播放，本动画在保持
「岁差绕黄极缓慢进动、章动叠加在平极上做更快小幅摆动」这一几何关系不变的
前提下，把两者都放大/加速到可在数秒内看清的程度，仅作定性示意，不是等比例
仿真。

渲染：
    uv run --project scripts --only-group manim-docs \
        manim -qh scripts/manim/reference_frames/precession_nutation_equator.py \
        PrecessionNutationEquator
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import numpy as np
from manim import (
    DEGREES,
    DOWN,
    LEFT,
    ORIGIN,
    OUT,
    RIGHT,
    UL,
    Arrow3D,
    Circle,
    Create,
    Dot,
    FadeIn,
    Line3D,
    Mobject,
    Square,
    ThreeDScene,
    ValueTracker,
    VGroup,
    Write,
    always_redraw,
)

from _style import (
    BACKGROUND,
    COLOR_ECLIPTIC,
    COLOR_MEAN,
    COLOR_TRUE,
    caption,
    label,
)

OBLIQUITY = 23.44 * DEGREES
# 定性示意用的夸张参数，非真实比例（见模块文档字符串）
PRECESSION_TURNS = 1.5
NUTATION_CYCLES = 9
NUTATION_AMPLITUDE = 9 * DEGREES
POLE_LENGTH = 2.6
DISC_RADIUS = 1.8


def _rotate(vector: np.ndarray, angle: float, axis: np.ndarray) -> np.ndarray:
    """罗德里格斯旋转公式：vector 绕单位轴 axis 旋转 angle 弧度。"""
    axis = axis / np.linalg.norm(axis)
    return (
        vector * np.cos(angle)
        + np.cross(axis, vector) * np.sin(angle)
        + axis * np.dot(axis, vector) * (1 - np.cos(angle))
    )


def mean_pole_direction(t: float) -> np.ndarray:
    """平极方向：绕黄极（OUT）以固定倾角作圆锥进动，t ∈ [0, 1]。"""
    base = _rotate(OUT, OBLIQUITY, RIGHT)
    return _rotate(base, t * PRECESSION_TURNS * 2 * np.pi, OUT)


def true_pole_direction(t: float) -> np.ndarray:
    """真极方向：在平极基础上叠加章动（更快、更小幅度的摆动）。"""
    mean = mean_pole_direction(t)
    wobble_axis = np.cross(OUT, mean)
    if np.linalg.norm(wobble_axis) < 1e-6:
        wobble_axis = RIGHT
    angle = NUTATION_AMPLITUDE * np.sin(t * NUTATION_CYCLES * 2 * np.pi)
    return _rotate(mean, angle, wobble_axis)


def _disc_from_normal(normal: np.ndarray, *, color: str, opacity: float) -> Mobject:
    """以给定法向量为法线、圆心在原点的圆盘（示意某历元的赤道面）。"""
    normal = normal / np.linalg.norm(normal)
    disc = Circle(radius=DISC_RADIUS, color=color, fill_opacity=opacity, stroke_width=2)
    cos_angle = float(np.clip(np.dot(OUT, normal), -1.0, 1.0))
    axis = np.cross(OUT, normal)
    if np.linalg.norm(axis) < 1e-6:
        if cos_angle > 0:
            return disc
        axis = RIGHT
        angle = np.pi
    else:
        angle = np.arccos(cos_angle)
    disc.rotate(angle, axis=axis, about_point=ORIGIN)
    return disc


def _legend_row(color: str, text: str) -> VGroup:
    swatch = Square(side_length=0.28, color=color, fill_color=color, fill_opacity=1)
    text_mob = label(text, size=22)
    text_mob.next_to(swatch, RIGHT, buff=0.15)
    return VGroup(swatch, text_mob)


class PrecessionNutationEquator(ThreeDScene):
    """平赤道（岁差）叠加章动得到真赤道：黄极为轴的进动 + 极上的短周期摆动。"""

    def construct(self):
        self.camera.background_color = BACKGROUND
        self.set_camera_orientation(phi=65 * DEGREES, theta=-50 * DEGREES, distance=8)

        title = caption("平赤道（岁差） → 真赤道（+ 章动）", size=34)
        title.to_corner(UL)
        self.add_fixed_in_frame_mobjects(title)
        self.play(Write(title))

        legend = VGroup(
            _legend_row(COLOR_ECLIPTIC, "黄道 / 黄极（固定）"),
            _legend_row(COLOR_MEAN, "平赤道 / 平极（岁差：绕黄极进动）"),
            _legend_row(COLOR_TRUE, "真赤道 / 真极（+ 章动：平极上的短周期摆动）"),
        ).arrange(DOWN, aligned_edge=LEFT, buff=0.18)
        legend.scale(0.6)
        legend.next_to(title, DOWN, aligned_edge=LEFT, buff=0.3)
        self.add_fixed_in_frame_mobjects(legend)
        self.play(Write(legend))

        ecliptic_disc = Circle(
            radius=DISC_RADIUS, color=COLOR_ECLIPTIC, fill_opacity=0.12, stroke_width=2
        )
        ecliptic_pole = Line3D(ORIGIN, OUT * POLE_LENGTH, color=COLOR_ECLIPTIC, thickness=0.015)
        self.play(Create(ecliptic_disc), FadeIn(ecliptic_pole))

        t = ValueTracker(0.0)

        mean_pole_arrow = always_redraw(
            lambda: Arrow3D(
                ORIGIN, mean_pole_direction(t.get_value()) * POLE_LENGTH, color=COLOR_MEAN
            )
        )
        mean_disc = always_redraw(
            lambda: _disc_from_normal(
                mean_pole_direction(t.get_value()), color=COLOR_MEAN, opacity=0.15
            )
        )
        true_pole_arrow = always_redraw(
            lambda: Arrow3D(
                ORIGIN, true_pole_direction(t.get_value()) * POLE_LENGTH, color=COLOR_TRUE
            )
        )
        true_pole_dot = always_redraw(
            lambda: Dot(
                point=true_pole_direction(t.get_value()) * POLE_LENGTH,
                color=COLOR_TRUE,
                radius=0.06,
            )
        )

        self.play(FadeIn(mean_disc), FadeIn(mean_pole_arrow))

        self.play(t.animate.set_value(1.0), run_time=4, rate_func=lambda x: x)
        self.wait(0.3)

        self.play(FadeIn(true_pole_arrow), FadeIn(true_pole_dot))

        caption_bottom = caption(
            "历表 → 平架 → 岁差 P → 章动 N → 真赤道 / 真黄道", size=26, color=COLOR_TRUE
        )
        caption_bottom.to_edge(DOWN)
        self.add_fixed_in_frame_mobjects(caption_bottom)
        self.play(Write(caption_bottom))

        t.set_value(0.0)
        self.play(t.animate.set_value(1.0), run_time=8, rate_func=lambda x: x)

        self.wait(1.5)
