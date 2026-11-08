"""乾隆三十年时宪书逐日页放大：把一个半叶切成若干竖条并排，便于逐列读宜忌。

用法（仓库根）：uv run --no-project --with pymupdf --with pillow python scripts/folk/shixianshu_pages.py 页 x起 x止 输出.png [条数] [dpi]
x起、x止为该页宽度的比例（0—1，右侧为前）；条数缺省 6，dpi 缺省 420。书影：doc/references/folk/時憲書/乾隆三十年乙酉.pdf（不入库）。
"""
import sys

import fitz
from PIL import Image

PDF = "doc/references/folk/時憲書/乾隆三十年乙酉.pdf"


def main():
    page, x0, x1, out = int(sys.argv[1]), float(sys.argv[2]), float(sys.argv[3]), sys.argv[4]
    n = int(sys.argv[5]) if len(sys.argv) > 5 else 6
    dpi = int(sys.argv[6]) if len(sys.argv) > 6 else 420
    d = fitz.open(PDF)
    p = d[page]
    r = p.rect
    w = (x1 - x0) / n
    ims = []
    for k in range(n):  # 自右而左
        a = x1 - (k + 1) * w - w * 0.08
        b = x1 - k * w + w * 0.08
        pix = p.get_pixmap(dpi=dpi, clip=fitz.Rect(r.width * a, r.height * 0.18, r.width * b, r.height * 0.95))
        ims.append(Image.frombytes("RGB", (pix.width, pix.height), pix.samples))
    W = sum(im.width for im in ims) + 8 * (n - 1)
    H = max(im.height for im in ims)
    o = Image.new("RGB", (W, H), "white")
    x = W
    for im in ims:
        x -= im.width
        o.paste(im, (x, 0))
        x -= 8
    o.save(out)
    print(o.size)


if __name__ == "__main__":
    main()
