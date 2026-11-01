"""独立核对地形地平线：不经 Rust，用 Python 取同一批 Terrarium 瓦片、同一公式，算几个地点几个方位的地平线视高度。
结果写入 doc/astronomy/sky-view-3d.md。需要联网。

用法（仓库根）：uv run --project scripts --only-group sky-view python scripts/phenomena/terrain_horizon_check.py
"""
import io
import math
import urllib.request

import numpy as np
from PIL import Image

R, K = 6371000.0, 0.13
cache = {}


def tile(z, x, y):
    if (z, x, y) not in cache:
        url = f"https://s3.amazonaws.com/elevation-tiles-prod/terrarium/{z}/{x}/{y}.png"
        with urllib.request.urlopen(url, timeout=30) as r:
            a = np.asarray(Image.open(io.BytesIO(r.read())).convert("RGB"), dtype=np.float64)
        cache[(z, x, y)] = a[..., 0] * 256 + a[..., 1] + a[..., 2] / 256 - 32768
    return cache[(z, x, y)]


def elev(lon, lat, z):
    n = 2**z
    fx = (lon + 180) / 360 * n
    phi = math.radians(lat)
    fy = (1 - math.log(math.tan(phi) + 1 / math.cos(phi)) / math.pi) / 2 * n
    return tile(z, int(fx), int(fy))[int((fy % 1) * 256), int((fx % 1) * 256)]


def horizon(lon, lat, az_deg, eye=1.7):
    h0 = elev(lon, lat, 12) + eye
    p, l, az = math.radians(lat), math.radians(lon), math.radians(az_deg)
    best, best_d = -99.0, 0.0
    d = 2000.0
    while d < 200000:
        a = d / R
        la = math.asin(math.sin(p) * math.cos(a) + math.cos(p) * math.sin(a) * math.cos(az))
        lo = l + math.atan2(math.sin(az) * math.sin(a) * math.cos(p), math.cos(a) - math.sin(p) * math.sin(la))
        alt = math.degrees(math.atan((elev(math.degrees(lo), math.degrees(la), 9) - h0) / d - (1 - K) * d / (2 * R)))
        if alt > best:
            best, best_d = alt, d
        d = max(d * 1.02, d + 30)
    return best, best_d, h0 - eye


for name, lon, lat in [("长安", 108.94, 34.27), ("开封", 114.35, 34.79), ("北京", 116.39, 39.91)]:
    for az in [0, 90, 180, 270]:
        b, d, g = horizon(lon, lat, az)
        print(f"{name} 地面 {g:.0f} m 方位 {az:3d}°：{b:+.2f}°（{d / 1000:.1f} km 处）")
