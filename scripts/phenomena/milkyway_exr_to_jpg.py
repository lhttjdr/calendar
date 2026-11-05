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

# 亮度定标：从 JPEG 还原线性值（与前端着色器同式），取明度，按 1° 块平均；银道面 |b| < 5° 的中位数定为积分星光 200 S10
back = np.asarray(Image.open(OUT).convert("RGB"), dtype=np.float64) / 255.0
lin = np.power(np.clip(back, 1e-4, 1.0) / a, 1.0 / g)
lum = lin @ np.array([0.2126, 0.7152, 0.0722])
k = w // 360
blk = lum[: h // k * k, : w // k * k].reshape(h // k, k, w // k, k).mean(axis=(1, 3))
b = 90.0 - (np.arange(h // k) + 0.5) * 180.0 / (h // k)
s10 = 200.0 / np.median(blk[np.abs(b) < 5.0])
print(f"定标 {s10:.0f} S10/单位（前端 MILKY_S10_PER_UNIT）")
for lo, hi in [(0, 5), (5, 10), (10, 20), (20, 40), (40, 60), (60, 90)]:
    m = (np.abs(b) >= lo) & (np.abs(b) < hi)
    print(f"  |b| {lo}—{hi}°：中位 {np.median(blk[m] * s10):.0f}，p95 {np.percentile(blk[m] * s10, 95):.0f} S10")
print(f"  1° 块最亮 {blk.max() * s10:.0f} S10，在 b = {b[np.unravel_index(blk.argmax(), blk.shape)[0]]:.1f}°")
