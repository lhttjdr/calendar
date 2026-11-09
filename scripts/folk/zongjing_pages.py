"""《選擇宗鏡》（NCL-06639 抄本）书影出图：每页一开两面，按页输出或若干页上下拼合，便于逐页读图编目次。

用法（仓库根）：scripts/.venv/bin/python scripts/folk/zongjing_pages.py 起页 页数 输出.png [dpi] [x0 x1 y0 y1]
起页为 PDF 页（1 起，与文档所注页同）；可选裁剪框为页宽高比例（0—1）。书影：doc/references/folk/通書/選擇宗鏡/選擇宗鏡.pdf（不入库）。
"""
import sys

import fitz
from PIL import Image

PDF = "doc/references/folk/通書/選擇宗鏡/選擇宗鏡.pdf"


def main():
    p0, n, out = int(sys.argv[1]), int(sys.argv[2]), sys.argv[3]
    dpi = int(sys.argv[4]) if len(sys.argv) > 4 else 80
    box = [float(x) for x in sys.argv[5:9]] if len(sys.argv) > 8 else [0, 1, 0, 1]
    d = fitz.open(PDF)
    ims = []
    for i in range(p0 - 1, min(p0 - 1 + n, len(d))):
        r = d[i].rect
        clip = fitz.Rect(r.width * box[0], r.height * box[2], r.width * box[1], r.height * box[3])
        pix = d[i].get_pixmap(dpi=dpi, clip=clip)
        ims.append(Image.frombytes("RGB", (pix.width, pix.height), pix.samples))
    H = sum(im.height for im in ims) + 8 * (len(ims) - 1)
    o = Image.new("RGB", (max(im.width for im in ims), H), "white")
    y = 0
    for im in ims:
        o.paste(im, (0, y))
        y += im.height + 8
    o.save(out)
    print(o.size)


if __name__ == "__main__":
    main()
