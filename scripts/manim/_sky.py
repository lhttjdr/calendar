"""天球几何的共用构件：大圆、球面点、球面弧、坐标换算，供各天球动画复用。

坐标约定（所有天球动画统一）：
- 黄道面为 xy 面，黄极为 +z（OUT）；春分点（J2000 或场景所设的基准分点）在 +x。
- 赤道法向（北天极）= R_x(−ε)·z = (0, sin ε, cos ε)，于是太阳赤纬 sin δ = sin ε · sin λ，
  λ=0 时太阳自南向北过赤道，即春分点；λ 逆时针（自 +z 俯视）增大。
- 岁差使天极绕黄极顺时针（自 +z 俯视）转动，春分点沿黄道西退（λ 减小方向）。
"""

import numpy as np
from manim import (
    DEGREES,
    OUT,
    RIGHT,
    UP,
    Dot3D,
    ParametricFunction,
    ThreeDScene,
    VGroup,
)

from _style import COLOR_FIXED_FRAME, GRID_DIM

OBLIQUITY = 23.44 * DEGREES
RADIUS = 3.0


def unit(v: np.ndarray) -> np.ndarray:
    return v / np.linalg.norm(v)


def rotate(v: np.ndarray, angle: float, axis: np.ndarray) -> np.ndarray:
    """罗德里格斯公式：v 绕单位轴 axis 转 angle 弧度（右手）。"""
    a = unit(axis)
    return v * np.cos(angle) + np.cross(a, v) * np.sin(angle) + a * np.dot(a, v) * (1 - np.cos(angle))


def equator_pole(eps: float = OBLIQUITY, node_lon: float = 0.0) -> np.ndarray:
    """北天极方向。node_lon 为春分点黄经（岁差使其减小），eps 为黄赤交角。"""
    return rotate(np.array([0.0, np.sin(eps), np.cos(eps)]), node_lon, OUT)


def basis_of(pole: np.ndarray, origin_hint: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    """以 pole 为极的球面坐标基：返回 (e0, e90)，e0 为经度零点方向（origin_hint 在该大圆上的投影）。"""
    p = unit(pole)
    e0 = unit(origin_hint - np.dot(origin_hint, p) * p)
    return e0, np.cross(p, e0)


def sph(pole: np.ndarray, origin_hint: np.ndarray, lon: float, lat: float, r: float = RADIUS) -> np.ndarray:
    """以 pole 为极、经度零点取 origin_hint 方向的球面坐标 (lon, lat) → 直角坐标。"""
    e0, e90 = basis_of(pole, origin_hint)
    return r * (np.cos(lat) * (np.cos(lon) * e0 + np.sin(lon) * e90) + np.sin(lat) * unit(pole))


def ecl(lon: float, lat: float = 0.0, r: float = RADIUS) -> np.ndarray:
    """黄道坐标（基准春分点在 +x）→ 直角坐标。"""
    return sph(OUT, RIGHT, lon, lat, r)


def great_circle(pole: np.ndarray, color: str, *, r: float = RADIUS, width: float = 3, **kw) -> ParametricFunction:
    e0, e90 = basis_of(pole, RIGHT if abs(np.dot(unit(pole), RIGHT)) < 0.9 else UP)
    return ParametricFunction(
        lambda t: r * (np.cos(t) * e0 + np.sin(t) * e90),
        t_range=[0, 2 * np.pi, 2 * np.pi / 180],
        color=color,
        stroke_width=width,
        **kw,
    )


def small_circle(axis: np.ndarray, angle: float, color: str, *, r: float = RADIUS, width: float = 2, **kw) -> ParametricFunction:
    """与 axis 夹角为 angle 的小圆（如天极绕黄极的岁差圆）。"""
    e0, e90 = basis_of(axis, RIGHT if abs(np.dot(unit(axis), RIGHT)) < 0.9 else UP)
    a = unit(axis)
    return ParametricFunction(
        lambda t: r * (np.sin(angle) * (np.cos(t) * e0 + np.sin(t) * e90) + np.cos(angle) * a),
        t_range=[0, 2 * np.pi, 2 * np.pi / 180],
        color=color,
        stroke_width=width,
        **kw,
    )


def arc(u: np.ndarray, v: np.ndarray, color: str, *, r: float = RADIUS, width: float = 5, **kw) -> ParametricFunction:
    """球面上 u→v 的劣弧（大圆弧）。"""
    a, b = unit(u), unit(v)
    omega = np.arccos(np.clip(np.dot(a, b), -1, 1))
    if omega < 1e-9:
        return ParametricFunction(lambda t: r * a, t_range=[0, 1], color=color, stroke_width=width, **kw)
    return ParametricFunction(
        lambda t: r * (np.sin((1 - t) * omega) * a + np.sin(t * omega) * b) / np.sin(omega),
        t_range=[0, 1, 1 / 90],
        color=color,
        stroke_width=width,
        **kw,
    )


def arc_along(pole: np.ndarray, origin_hint: np.ndarray, lon0: float, lon1: float, color: str, *,
              lat: float = 0.0, r: float = RADIUS, width: float = 5, **kw) -> ParametricFunction:
    """沿以 pole 为极的纬圈（lat=0 即大圆）从 lon0 到 lon1 的弧，可超过 180°。"""
    return ParametricFunction(
        lambda t: sph(pole, origin_hint, lon0 + t * (lon1 - lon0), lat, r),
        t_range=[0, 1, 1 / 180],
        color=color,
        stroke_width=width,
        **kw,
    )


def sphere_wireframe(*, r: float = RADIUS, color: str = GRID_DIM, n_meridians: int = 6) -> VGroup:
    """淡色天球轮廓：黄道坐标的几条经线与纬线，只作空间感参照。"""
    g = VGroup()
    for k in range(n_meridians):
        lon = k * np.pi / n_meridians
        g.add(great_circle(np.array([np.cos(lon + np.pi / 2), np.sin(lon + np.pi / 2), 0.0]), color, r=r, width=1))
    for lat in (-60, -30, 30, 60):
        g.add(small_circle(OUT, (90 - lat) * DEGREES, color, r=r, width=1))
    return g


class SkyScene(ThreeDScene):
    """天球场景基类。

    add_fixed_in_frame_mobjects / add_fixed_orientation_mobjects 会立即把对象加入场景，
    若登记与 FadeIn 之间还有别的动画，文字会提前整块出现。fix/face 只登记、不显示，
    由随后的 FadeIn 引入。
    """

    def fix(self, *ms):
        # 设了 frame_center 时，屏幕固定层会随之整体偏移，这里平移回来
        offset = np.array(self.camera.frame_center)
        for m in ms:
            m.shift(offset)
        self.add_fixed_in_frame_mobjects(*ms)
        self.remove(*ms)

    def face(self, *ms):
        self.add_fixed_orientation_mobjects(*ms)
        self.remove(*ms)


def star(pos: np.ndarray, color: str = COLOR_FIXED_FRAME, radius: float = 0.06) -> Dot3D:
    return Dot3D(pos, radius=radius, color=color)


__all__ = [
    "OBLIQUITY",
    "RADIUS",
    "unit",
    "rotate",
    "equator_pole",
    "basis_of",
    "sph",
    "ecl",
    "great_circle",
    "small_circle",
    "arc",
    "arc_along",
    "sphere_wireframe",
    "SkyScene",
    "star",
]
