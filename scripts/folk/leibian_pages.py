"""类编任一册书影拼页：若干半叶自右而左并排（右为前页），便于读图。

用法（仓库根）：scripts/.venv/bin/python scripts/folk/leibian_pages.py 册号 起页 页数 输出.png [dpi]
册号如 02094410；起页为该册 PDF 页（1 起，与文档所注「册:页」同）。书影在 doc/references/folk/通書/類編曆法通書大全/（不入库）。
"""
import sys

import fitz
from PIL import Image


def main():
    vol, p0, n, out = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), sys.argv[4]
    dpi = int(sys.argv[5]) if len(sys.argv) > 5 else 120
    d = fitz.open(f"doc/references/folk/通書/類編曆法通書大全/{vol}.cn.pdf")
    ims = []
    for i in range(p0 - 1, min(p0 - 1 + n, len(d))):
        r = d[i].rect
        pix = d[i].get_pixmap(dpi=dpi, clip=fitz.Rect(r.width * 0.04, r.height * 0.06, r.width * 0.96, r.height * 0.95))
        ims.append(Image.frombytes("RGB", (pix.width, pix.height), pix.samples))
    W = sum(im.width for im in ims) + 10 * (len(ims) - 1)
    o = Image.new("RGB", (W, max(im.height for im in ims)), "white")
    x = W
    for im in ims:
        x -= im.width
        o.paste(im, (x, 0))
        x -= 10
    o.save(out)
    print(o.size)


if __name__ == "__main__":
    main()
