"""文档动画统一配色/字体，供 scripts/manim/ 下各 Scene 复用。

背景取深空蓝而非纯黑/纯白：作为内嵌 <video> 固定背景，在文档站浅色/深色两种主题下
都不违和（不依赖 VitePress 当前主题）。中文文本用 Pango 渲染（Text，非 LaTeX 的
Tex/MathTex），避免额外引入 texlive 依赖；渲染机器需装有下列字体族之一。
"""

import re

from manim import DOWN, LEFT, RIGHT, UP, MathTex, Text, VGroup

# 背景与网格
BACKGROUND = "#0B1021"
GRID_DIM = "#2A3352"

# 语义色：与文档术语对应，同一概念的颜色跨动画保持一致
COLOR_FIXED_FRAME = "#E2E8F0"   # 惯性架 / ICRS：近白，代表“固定不动”
COLOR_MEAN = "#4FD1C5"          # 平：岁差相关（平赤道、平极、平春分点）
COLOR_TRUE = "#F6AD55"          # 真：章动相关（真赤道、真极）
COLOR_ECLIPTIC = "#B794F4"      # 黄道：淡紫，与赤道（青）、真（橙）区分
COLOR_HIGHLIGHT = "#F56565"     # 强调 / 连接箭头（如 Frame Bias B）
COLOR_ERA_PAST = "#5A6488"      # 时间线上已略过的历史阶段：调暗
COLOR_SUN = "#FACC15"           # 太阳
COLOR_STAR = "#E2E8F0"          # 恒星
COLOR_EPOCH = "#F687B3"         # 另一历元（如 J2000）的平赤道/分点，与当日平（青）对照

# 中文优先，找不到时按序退回到常见跨平台 CJK 字体
CJK_FONT = "Noto Sans CJK SC, Noto Sans SC, PingFang SC, Microsoft YaHei, sans-serif"


def label(text: str, *, size: int = 32, color: str = COLOR_FIXED_FRAME) -> Text:
    return Text(text, font=CJK_FONT, font_size=size, color=color)


def caption(text: str, *, size: int = 28, color: str = COLOR_FIXED_FRAME) -> Text:
    """置于画面固定位置的说明性文字（配合 add_fixed_in_frame_mobjects 使用）。多行时行距取字号的 1.6 倍。"""
    return Text(text, font=CJK_FONT, font_size=size, color=color, line_spacing=0.6)


def rich(text: str, *, size: int = 28, color: str = COLOR_FIXED_FRAME) -> VGroup:
    """中文与 LaTeX 混排：`$…$` 内交给 MathTex，其余用 Pango Text；`\n` 分行，各行左对齐。

    动态读数只让数字落在 `$…$` 之外（如 `f"$\\lambda =$ {x:.1f}°"`），公式部分每帧内容相同，
    MathTex 按内容缓存，不会逐帧重新编译 LaTeX。
    """
    rows = VGroup()
    for line in text.split("\n"):
        row = VGroup()
        for part in re.split(r"(\$[^$]+\$)", line):
            if not part.strip():
                continue
            if part.startswith("$"):
                # MathTex 与 Pango Text 的 font_size 刻度不同，同字号时公式明显偏小，放大 1.5 倍与中文等高
                row.add(MathTex(part[1:-1], font_size=size * 1.5, color=color))
            else:
                row.add(Text(part.strip(), font=CJK_FONT, font_size=size, color=color))
        rows.add(row.arrange(RIGHT, buff=size * 0.005))
    return rows.arrange(DOWN, aligned_edge=LEFT, buff=size * 0.008)


__all__ = [
    "BACKGROUND",
    "GRID_DIM",
    "COLOR_FIXED_FRAME",
    "COLOR_MEAN",
    "COLOR_TRUE",
    "COLOR_ECLIPTIC",
    "COLOR_HIGHLIGHT",
    "COLOR_ERA_PAST",
    "COLOR_SUN",
    "COLOR_STAR",
    "COLOR_EPOCH",
    "CJK_FONT",
    "label",
    "caption",
    "rich",
    "UP",
    "DOWN",
    "LEFT",
    "RIGHT",
]
