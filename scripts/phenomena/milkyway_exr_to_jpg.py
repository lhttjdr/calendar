"""NASA SVS Deep Star Maps 的 4K 银河图（OpenEXR 线性色）→ 前端贴图 JPEG。

输入 vendor/nasa_svs_deep_star_maps/milkyway_2020_4k_gal.exr（33 MB，不入库，下载见同目录 README），
输出 data/sky/milkyway_2020_4k_gal.jpg。NASA 的预览图 milkyway_2020_4k_gal_print.jpg 是官方的色调映射，
这里拟合「EXR 线性值 → 预览图灰度」的曝光与伽马，让 4K 图的观感与预览图一致；拟合结果打印出来，记入文档。

用法（仓库根）：uv run --project scripts --only-group sky-view python scripts/phenomena/milkyway_exr_to_jpg.py
"""
import numpy as np
import OpenEXR
from PIL import Image

SRC = "vendor/nasa_svs_deep_star_maps/milkyway_2020_4k_gal.exr"
PREVIEW = "vendor/nasa_svs_deep_star_maps/milkyway_2020_4k_gal_print.jpg"
OUT = "data/sky/milkyway_2020_4k_gal.jpg"

with OpenEXR.File(SRC) as f:
    channels = f.channels()
    rgb = channels["RGB"].pixels if "RGB" in channels else np.dstack([channels[c].pixels for c in "RGB"])
rgb = np.maximum(rgb.astype(np.float64), 0.0)
h, w, _ = rgb.shape
print("EXR", w, h, "线性值 p50/p99/max", *np.percentile(rgb, [50, 99, 100]).round(5))

prev = np.asarray(Image.open(PREVIEW).convert("RGB"), dtype=np.float64) / 255.0
ph, pw, _ = prev.shape
# 按块平均降到预览图尺寸
small = rgb.reshape(ph, h // ph, pw, w // pw, 3).mean(axis=(1, 3))
x = small.mean(axis=2).ravel()
y = prev.mean(axis=2).ravel()
mask = (x > 1e-5) & (y > 0.02) & (y < 0.95)
# lg y = g·lg x + lg a
g, lg_a = np.polyfit(np.log10(x[mask]), np.log10(y[mask]), 1)
a = 10.0**lg_a
pred = a * np.power(x[mask], g)
print(f"拟合 y = {a:.4f}·x^{g:.4f}，样本 {mask.sum()}，rms {np.sqrt(np.mean((pred - y[mask]) ** 2)):.4f}（0—1 灰度）")

out = np.clip(a * np.power(rgb, g), 0.0, 1.0)
Image.fromarray((out * 255.0 + 0.5).astype(np.uint8)).save(OUT, quality=88, optimize=True, progressive=True)
print("输出", OUT)
