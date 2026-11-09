"""类编卷十一逐月日表书影拼页：把相邻两（或多）个半叶裁去版框外空白后自右而左并排，便于逐日读图转录。

用法（仓库根）：scripts/.venv/bin/python scripts/folk/leibian_juan11_pages.py 起页 页数 输出.png [dpi]
起页为 02094408 册 PDF 页（1 起，与 CSV「頁」列同）；输出图右为前页。书影：doc/references/folk/通書/類編曆法通書大全/02094408.cn.pdf（不入库）。
卷十一逐月日表：正月首页在 PDF 第 2 页，以后每月 12 页（月首页为月建、天道、月德、节气、時刻与日吉日凶，
次页起甲子至癸亥，十一个半叶），十二月止于第 145 页。转录结果见 data/folk/leibian_juan11_days.csv。
"""
import sys

import fitz
from PIL import Image

PDF = "doc/references/folk/通書/類編曆法通書大全/02094408.cn.pdf"


def main():
    p0, n, out = int(sys.argv[1]), int(sys.argv[2]), sys.argv[3]
    dpi = int(sys.argv[4]) if len(sys.argv) > 4 else 140
    d = fitz.open(PDF)
    ims = []
    for i in range(p0 - 1, min(p0 - 1 + n, len(d))):
        r = d[i].rect
        pix = d[i].get_pixmap(dpi=dpi, clip=fitz.Rect(r.width * 0.05, r.height * 0.09, r.width * 0.93, r.height * 0.92))
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
