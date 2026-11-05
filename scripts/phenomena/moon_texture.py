"""NASA SVS CGI Moon Kit 的 LRO 彩色月面图 → 前端月面贴图 JPEG。

输入 vendor/nasa_svs_cgi_moon_kit/lroc_color_poles_2k.tif（2048×1024，sRGB，等距圆柱，中央经线为月面经度 0°，
上为月球北极；月面经纬度为 IAU 平地球/极轴（ME）坐标），输出 data/sky/moon_lroc_color_2k.jpg。
只换容器（TIFF → JPEG），不改尺寸、不调色；打印近侧半球（|经度| < 90°）的平均线性反照率，供前端归一。

用法（仓库根）：uv run --project scripts --only-group sky-view python scripts/phenomena/moon_texture.py
"""
import numpy as np
from PIL import Image

SRC = "vendor/nasa_svs_cgi_moon_kit/lroc_color_poles_2k.tif"
OUT = "data/sky/moon_lroc_color_2k.jpg"

im = Image.open(SRC)
print("源图", im.size, im.mode)
rgb = im.convert("RGB")
rgb.save(OUT, quality=90, optimize=True, progressive=True)

a = np.asarray(rgb, dtype=np.float64) / 255.0
lin = np.where(a <= 0.04045, a / 12.92, ((a + 0.055) / 1.055) ** 2.4)
h, w, _ = lin.shape
lon = (np.arange(w) + 0.5) / w * 360.0 - 180.0
lat = 90.0 - (np.arange(h) + 0.5) / h * 180.0
# 近侧半球按投影面积（cos 视角）加权，即满月时看到的平均
cosv = np.cos(np.radians(lat))[:, None] * np.cos(np.radians(lon))[None, :]
wgt = np.clip(cosv, 0, None) * np.cos(np.radians(lat))[:, None]
mean = (lin * wgt[..., None]).sum(axis=(0, 1)) / wgt.sum()
print("近侧半球平均线性值 RGB", mean.round(4), "亮度", round(float(mean @ [0.2126, 0.7152, 0.0722]), 4))
