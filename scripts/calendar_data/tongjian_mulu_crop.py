"""《资治通鉴目录》扫描本取顶栏（岁名、朔闰、天象一栏）以便看图转录 data/calendar/tongjian_mulu_shuo.csv。

维基共享资源 SSID-12318597—12318606 十册 PDF（卷1-3 … 卷28-30），每页一幅半叶图。本脚本：
  1. 经 Commons API 查文件地址、下载 PDF（User-Agent "calendar-research-script/0.1"，请求间隔 ≥2 秒）；
  2. 逐页找上下框线，取框内上方 17% 为顶栏，归一化为 165px 高，按书页右→左接成一长条 strip.png（pages.csv 记各页起点）；
  3. 在长条上找岁名大字列（宽 38—72px 的墨迹列，相距 <62px 者合为一个岁名），自岁名右缘向左裁 470px 存 NNNN.png
     （index.csv：序号, 起页, 止页, 条中位置）；岁名跨页、朔闰写在框外等情形会漏裁，须对 strip 补看；
  4. 删除 PDF（磁盘紧张，全程只留裁图）。
RapidOCR 对双行小字识读率低（如「正己丑四戊午」识作「正已开四座」），本项目转录以看裁图为准，OCR 不入流程。

用法（仓库根）：
  uv run --no-project --with pymupdf --with numpy --with pillow python -I scripts/calendar_data/tongjian_mulu_crop.py 22-24 OUTDIR
"""
from __future__ import annotations

import json
import os
import sys
import time
import urllib.parse
import urllib.request

import numpy as np
import pymupdf
from PIL import Image

UA = {"User-Agent": "calendar-research-script/0.1"}
API = "https://commons.wikimedia.org/w/api.php"
SSID = {"1-3": 12318597, "4-6": 12318598, "7-9": 12318599, "10-12": 12318600, "13-15": 12318601,
        "16-18": 12318602, "19-21": 12318603, "22-24": 12318604, "25-27": 12318605, "28-30": 12318606}
BH = 165  # 归一化顶栏高（px）
CROP_W = 470


def commons_file(vols: str) -> str:
    return f"SSID-{SSID[vols]} 資治通鑑目錄 卷{vols}.pdf"


def download(vols: str, path: str) -> None:
    q = urllib.parse.urlencode({"action": "query", "prop": "imageinfo", "iiprop": "url",
                                "titles": "File:" + commons_file(vols), "format": "json"})
    with urllib.request.urlopen(urllib.request.Request(f"{API}?{q}", headers=UA)) as r:
        page = next(iter(json.load(r)["query"]["pages"].values()))
    url = page["imageinfo"][0]["url"]
    time.sleep(2)
    with urllib.request.urlopen(urllib.request.Request(url, headers=UA)) as r, open(path, "wb") as f:
        while chunk := r.read(1 << 20):
            f.write(chunk)
    time.sleep(2)


def page_img(doc, i):
    imgs = doc[i].get_images()
    if not imgs:
        return None
    pix = pymupdf.Pixmap(doc, imgs[0][0])
    if pix.n != 1 or pix.alpha:
        pix = pymupdf.Pixmap(pymupdf.csGRAY, pix)
    return np.frombuffer(pix.samples, dtype=np.uint8).reshape(pix.height, pix.width).copy()


def runs(mask):
    out, s = [], None
    for i, v in enumerate(mask):
        if v and s is None:
            s = i
        if not v and s is not None:
            out.append([s, i])
            s = None
    if s is not None:
        out.append([s, len(mask)])
    return out


def band_of(a):
    """顶栏 (top, bot, L, R)：上框为上 35% 中最黑之行，下框为下 40% 中最黑之行，顶栏取框高 17%。"""
    h, w = a.shape
    rows = (a < 110).sum(1)
    t0 = int(np.argmax(rows[: int(0.35 * h)]))
    b0 = int(0.6 * h) + int(np.argmax(rows[int(0.6 * h):]))
    if rows[t0] < 0.25 * w or rows[b0] < 0.25 * w:
        return None
    while t0 + 1 < h and rows[t0 + 1] > 0.2 * w:
        t0 += 1
    top, H = t0 + 2, b0 - t0
    if H < 0.4 * h:
        return None
    bot = top + int(0.17 * H)
    ink = (a[top:bot] < 120).sum(0)
    frame = [x for x in range(w) if ink[x] > 0.85 * (bot - top)]
    L = max([x for x in frame if x < w * 0.3], default=-1) + 3
    R = min([x for x in frame if x > w * 0.7], default=w + 1) - 3
    return top, bot, L, R


def extract(pdf: str, out: str) -> None:
    os.makedirs(out, exist_ok=True)
    doc = pymupdf.open(pdf)
    bands, pages = [], []
    for i in range(len(doc)):
        a = page_img(doc, i)
        b = band_of(a) if a is not None else None
        if b is None:
            continue
        top, bot, L, R = b
        im = Image.fromarray(a[top:bot, L:R])
        im = np.array(im.resize((max(1, int(im.width * BH / im.height)), BH), Image.LANCZOS))
        bands.append(np.concatenate([im, np.zeros((BH, 3), np.uint8)], axis=1))
        pages.append(i + 1)
    widths = [b.shape[1] for b in bands]
    strip = np.concatenate(bands[::-1], axis=1)
    Wt = strip.shape[1]
    starts = np.cumsum([0] + widths)
    Image.fromarray(strip).save(os.path.join(out, "strip.png"))
    with open(os.path.join(out, "pages.csv"), "w") as f:
        f.writelines(f"{p},{s}\n" for p, s in zip(pages, starts))
    ink = (strip[8:BH - 8] < 120).sum(0)[::-1]
    merged = []
    for r in runs(ink > 8):
        if merged and r[0] - merged[-1][1] <= 2:
            merged[-1][1] = r[1]
        else:
            merged.append(r)
    groups = []
    for r in (r for r in merged if 38 <= r[1] - r[0] <= 72):
        if groups and r[0] - groups[-1][-1][1] < 62:
            groups[-1].append(r)
        else:
            groups.append([r])
    page_at = lambda u: pages[int(np.searchsorted(starts, u, side="right") - 1)]
    with open(os.path.join(out, "index.csv"), "w") as f:
        for n, gp in enumerate(groups):
            u0 = max(0, gp[0][0] - 6)
            u1 = min(Wt, u0 + CROP_W)
            Image.fromarray(strip[:, Wt - u1: Wt - u0]).save(os.path.join(out, f"{n:04d}.png"))
            f.write(f"{n},{page_at(u0)},{page_at(u1 - 1)},{u0}\n")
    print(commons_file_note(pdf), "pages", len(doc), "bands", len(bands), "crops", len(groups))


def commons_file_note(pdf: str) -> str:
    return os.path.basename(pdf)


if __name__ == "__main__":
    vols, out = sys.argv[1], sys.argv[2]
    os.makedirs(out, exist_ok=True)
    pdf = os.path.join(out, f"v{vols}.pdf")
    download(vols, pdf)
    try:
        extract(pdf, out)
    finally:
        os.remove(pdf)
