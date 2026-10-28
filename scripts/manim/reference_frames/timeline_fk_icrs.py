"""对应 doc/astronomy/reference-frames.md §「天球参考的发展历程」。

渲染：
    uv run --project scripts --only-group manim-docs \
        manim -qh scripts/manim/reference_frames/timeline_fk_icrs.py TimelineFkIcrs
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from manim import (
    DOWN,
    LEFT,
    RIGHT,
    UP,
    Create,
    CurvedArrow,
    Dot,
    FadeIn,
    FadeOut,
    Line,
    Scene,
    Write,
)

from _style import (
    BACKGROUND,
    COLOR_ERA_PAST,
    COLOR_FIXED_FRAME,
    COLOR_HIGHLIGHT,
    COLOR_MEAN,
    COLOR_TRUE,
    caption,
    label,
)

ERAS = [
    {
        "name": "各国为政 / 区域星表",
        "date": "20 世纪前半",
        "note": "AGK・N30・GC・Yale：历元与岁差常数不一，跨星表需逐星比对",
        "color": COLOR_ERA_PAST,
    },
    {
        "name": "FK4",
        "date": "B1950.0（1963）",
        "note": "约 1535 颗星；动力学赤道与春分点，与 DE200 配套",
        "color": COLOR_MEAN,
    },
    {
        "name": "FK5",
        "date": "J2000.0（1988）",
        "note": "配套 IAU1976 岁差、VSOP87/DE200/DE405，1980–90 年代标准",
        "color": COLOR_MEAN,
    },
    {
        "name": "ICRS / ICRF",
        "date": "1998 年起",
        "note": "太阳系质心为原点；河外射电源（VLBI）定轴，长期无旋转",
        "color": COLOR_TRUE,
    },
]


class TimelineFkIcrs(Scene):
    """天球参考架的发展：各国为政 → FK4 → FK5 → ICRS，并标出 Frame Bias B。"""

    def construct(self):
        self.camera.background_color = BACKGROUND

        title = caption("天球参考架的发展历程", size=36, color=COLOR_FIXED_FRAME)
        title.to_edge(UP)
        self.play(Write(title))

        axis = Line(LEFT * 5.5, RIGHT * 5.5, color=COLOR_FIXED_FRAME)
        self.play(Create(axis))

        n = len(ERAS)
        xs = [-4.8 + i * (9.6 / (n - 1)) for i in range(n)]

        dots = []
        name_labels = []
        prev_note = None

        for era, x in zip(ERAS, xs):
            dot = Dot(point=[x, 0, 0], color=era["color"], radius=0.09)
            name_lbl = label(era["name"], size=28, color=era["color"])
            name_lbl.next_to(dot, UP, buff=0.35)
            date_lbl = label(era["date"], size=20, color=COLOR_FIXED_FRAME)
            date_lbl.next_to(dot, DOWN, buff=0.3)
            note_lbl = label(era["note"], size=22, color=COLOR_FIXED_FRAME)
            note_lbl.next_to(date_lbl, DOWN, buff=0.35)
            if note_lbl.width > 11:
                note_lbl.scale(11 / note_lbl.width)

            self.play(
                FadeIn(dot, scale=0.5),
                Write(name_lbl),
                Write(date_lbl),
                run_time=0.8,
            )
            self.play(Write(note_lbl), run_time=1.0)
            self.wait(1.0)

            if prev_note is not None:
                self.play(FadeOut(prev_note), run_time=0.5)
            prev_note = note_lbl

            dots.append(dot)
            name_labels.append(name_lbl)

        self.play(FadeOut(prev_note), run_time=0.5)

        # Frame Bias：FK5 → ICRS 的固定小旋转 B
        bias_arrow = CurvedArrow(
            dots[2].get_center() + UP * 0.15,
            dots[3].get_center() + UP * 0.15,
            color=COLOR_HIGHLIGHT,
            angle=-1.2,
        )
        bias_label = label("Frame Bias B（数十 mas，固定旋转）", size=22, color=COLOR_HIGHLIGHT)
        bias_label.next_to(bias_arrow, UP, buff=0.55)

        self.play(Create(bias_arrow), Write(bias_label))
        self.wait(2.5)
