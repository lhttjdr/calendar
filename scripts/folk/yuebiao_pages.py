#!/usr/bin/env python3
"""
取《欽定協紀辨方書》月表某月某日的影印页，与参考推算并排核对。

影印本：浙江大学图书馆藏四库本（archive.org 06067692.cn—06067697.cn，卷二十至三十一，每册两个月），
存 doc/references/folk/協紀辨方書/浙大乙本/（不入库）。每册单月甲子在第 7 页（索引 6），双月甲子在索引 73。
用法（仓库根，需 pymupdf、pillow）：
  python3 scripts/folk/yuebiao_pages.py 输出.png 1-乙亥,5-丁未,...
打印各日推算结果，并把对应影印页两两拼成一张图。
"""
import csv
import io
import sys

import fitz
from PIL import Image

B = "doc/references/folk/協紀辨方書/浙大乙本/"
VOL = {m: ("060676%02d" % (92 + (m - 1) // 2), 6 if m % 2 else 73) for m in range(1, 13)}
G, Z = "甲乙丙丁戊己庚辛壬癸", "子丑寅卯辰巳午未申酉戌亥"
GZ = [G[k % 10] + Z[k % 12] for k in range(60)]


def main(out, spec, width=640):
    days = [(int(x.split("-")[0]), x.split("-")[1]) for x in spec.split(",")]
    rows = {(int(r["月"]), r["干支"]): r for r in csv.DictReader(open("data/folk/xieji_yuebiao.csv", encoding="utf-8"))}
    ims = []
    for m, gz in days:
        vol, base = VOL[m]
        p = fitz.open(B + vol + ".cn.pdf")[base + GZ.index(gz)]
        z = width / p.rect.width
        ims.append(Image.open(io.BytesIO(p.get_pixmap(matrix=fitz.Matrix(z, z)).tobytes("png"))).convert("L"))
        r = rows[(m, gz)]
        print(f"{m}-{gz} {r['建除']} {r['等第']} |吉 {r['吉神']} |凶 {r['凶神']}\n  宜 {r['宜']}\n  忌 {r['忌']}")
    h = max(i.size[1] for i in ims)
    sheet = Image.new("L", (width * 2, h * ((len(ims) + 1) // 2)), 255)
    for k, im in enumerate(ims):  # 右起：与古书翻页次序一致
        sheet.paste(im, ((1 - k % 2) * width, (k // 2) * h))
    sheet.save(out)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
