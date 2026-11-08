#!/usr/bin/env python3
"""
把中国哲学书电子化计划（ctext）四库本《欽定協紀辨方書》月表（卷二十至三十一）的 OCR 初稿
切成「月 × 干支」的逐日文本，供与规则引擎的推算结果比对。OCR 只作比对底稿，以影印本为准。

OCR 页面不入库，先取到本地目录（参数 1），文件名为 ctext 章节号：
  855889 卷二十至二十一、207082 卷二十二至二十三、840990 卷二十四至二十五、
  795660 卷二十六至二十七、561484 卷二十八至二十九、184979 卷三十至三十一
  https://ctext.org/wiki.pl?if=gb&chapter=<章节号>
用法（仓库根）：python3 scripts/folk/yuebiao_ocr.py <OCR 目录>  → 输出 JSON 到标准输出
"""
import html
import json
import re
import sys

GAN = "甲乙丙丁戊己庚辛壬癸"
ZHI = "子丑寅卯辰巳午未申酉戌亥"
NAYIN = ["海中金", "鑪中火", "大林木", "路傍土", "劍鋒金", "山頭火", "澗下水", "城頭土", "白鑞金", "楊柳木",
         "井泉水", "屋上土", "霹靂火", "松柏木", "長流水", "砂石金", "山下火", "平地木", "壁上土", "金箔金",
         "覆燈火", "天河水", "大驛土", "釵釧金", "桑柘木", "大溪水", "沙中土", "天上火", "石榴木", "大海水"]
CHAPTERS = ["855889", "207082", "840990", "795660", "561484", "184979"]
JIE = ["立春", "驚蟄", "清明", "立夏", "芒種", "小暑", "立秋", "白露", "寒露", "立冬", "大雪", "小寒"]


def text_of(path):
    s = open(path, encoding="utf-8").read()
    i = s.find("該資料是通過")
    s = s[i:] if i >= 0 else s
    t = html.unescape(re.sub(r"<[^>]+>", "\n", s))
    t = re.sub(r"^\s*\d+\s*$", "", t, flags=re.M)  # ctext 段号
    return re.sub(r"\s+", "", t)


def fuzzy_find(t, word, start, maxerr=1):
    """在 t[start:] 找与 word 至多差 maxerr 字的第一处"""
    n = len(word)
    for i in range(start, len(t) - n + 1):
        if sum(a != b for a, b in zip(t[i:i + n], word)) <= maxerr:
            return i
    return -1


def main(d):
    full = "".join(text_of(f"{d}/{c}.html") for c in CHAPTERS)
    out = {}
    pos = 0
    JC = "建除滿平定執破危成收開閉"
    for m in range(12):
        starts = []
        for k in range(60):
            g, z = GAN[k % 10], ZHI[k % 12]
            jc = JC[(k % 12 - (m + 2)) % 12]
            want = g + z + NAYIN[k // 2]  # 后接寶義制專伐一字、建除一字、日
            best, bi = 0, -1
            for i in range(pos, min(len(full) - 8, pos + 2500)):
                sc = sum(a == b for a, b in zip(full[i:i + 5], want))
                sc += (full[i + 6] == jc) + (full[i + 7] == "日") + (full[i + 5] in "寶寳義制專伐")
                if sc > best:
                    best, bi = sc, i
                if sc == 8:
                    break
            if best < 5:
                print("找不到", m + 1, want + jc, file=sys.stderr)
                starts.append(None)
                continue
            starts.append(bi)
            pos = bi + 8
        for k in range(60):
            if starts[k] is None:
                continue
            nxt = next((s for s in starts[k + 1:] if s is not None), None)
            end = nxt if nxt is not None else min(len(full), starts[k] + 400)
            out[f"{m + 1}-{GAN[k % 10]}{ZHI[k % 12]}"] = full[starts[k]:end]
    json.dump(out, sys.stdout, ensure_ascii=False, indent=0)


if __name__ == "__main__":
    main(sys.argv[1])
