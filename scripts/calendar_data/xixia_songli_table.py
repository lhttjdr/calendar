"""彭向前《俄藏西夏历日文献整理研究》（社会科学文献出版社 2018）下篇二「西夏历与宋历朔闰对照表（1120—1207 年）」（书页 469—514）转录。

书为扫描本（本地 PDF，不入库），以 RapidOCR 逐页识别（缓存 JSON 由 --ocr 生成），按坐标还原表格行列：每行宋历月序、大小、朔日干支，
西夏历月序、大小、朔日干支，二历异同，文献。「己/已/巳」等形近字按六十干支表纠正。公元年按宋历月序回到正月递增。
宋历一栏与本项目张表（实历朔日表）逐月比对，校验识别与转录；不合之行列入输出的「核」列，须看图复核。

输出 data/calendar/xixia_song_shuorun.csv。
用法（仓库根）：
  uv run --project scripts python scripts/calendar_data/xixia_songli_table.py --ocr <PDF> <缓存目录>   # 识别（耗时）
  uv run --project scripts python scripts/calendar_data/xixia_songli_table.py <缓存目录>                # 解析
"""
import csv
import glob
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GZ = ["甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12] for i in range(60)]
MN = ["正", "二", "三", "四", "五", "六", "七", "八", "九", "十", "十一", "十二"]
PAGES = range(469, 515)
# 看图补录（页, 公元年, 宋月, 宋大小, 宋朔, 夏月, 夏大小, 夏朔, 二历异同）
PATCH = [
    (474, 1130, "2", "小", "甲戌", "2", "小", "甲戌", "朔同"),
    (474, 1131, "2", "大", "戊辰", "2", "大", "戊辰", "朔同"),
    (483, 1148, "11", "大", "乙酉", "11", "大", "乙酉", "朔同"),
    (483, 1148, "12", "小", "乙卯", "12", "小", "乙卯", "朔同"),
    (483, 1149, "1", "大", "甲申", "1", "大", "甲申", "朔同"),
    (483, 1149, "2", "小", "甲寅", "2", "小", "甲寅", "朔同"),
    (508, 1196, "11", "大", "丙子", "11", "", "", "朔日原缺，待考"),
]


def ocr(pdf, cache):
    import fitz
    from rapidocr_onnxruntime import RapidOCR
    d, eng = fitz.open(pdf), RapidOCR()
    Path(cache).mkdir(parents=True, exist_ok=True)
    for b in PAGES:
        f = Path(cache) / f"p{b}.json"
        if f.exists():
            continue
        png = Path(cache) / f"p{b}.png"
        d[b + 30].get_pixmap(dpi=200).save(png)
        res, _ = eng(str(png))
        f.write_text(json.dumps([(r[0][0][0], r[0][0][1], r[0][2][0], r[0][2][1], r[1], r[2]) for r in (res or [])], ensure_ascii=False))


def fix_gz(t):
    t = t.replace("已", "巳").replace("巳", "巳")
    if t in GZ:
        return t
    # 天干位的「巳/已」应为「己」
    if t and t[0] in "巳已" and ("己" + t[1:]) in GZ:
        return "己" + t[1:]
    if len(t) == 2 and t[1] in "己已" and (t[0] + "巳") in GZ:
        return t[0] + "巳"
    return None


def month_of(t):
    # 书中十一、十二月有时省「月」字
    m = re.fullmatch(r"(闰)?(正|十一|十二|十|[二三四五六七八九])月", t) or re.fullmatch(r"(闰)?(十一|十二)", t)
    if not m:
        return None
    return (MN.index(m.group(2)) + 1, bool(m.group(1)))


def zhang_months():
    """张表 1119—1208 年各月（按时间先后）：(公元年, 月, 闰, 朔 JDN)。"""
    def jdn(y, m, d):
        a = (14 - m) // 12
        yy, mm = y + 4800 - a, m + 12 * a - 3
        return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - 32083  # 儒略历
    out = []
    for r in csv.DictReader(open(ROOT / "vendor/calendar/chinese_calendar/实历朔日表.csv", encoding="utf-8")):
        y = int(r["lunar_year"])
        if 1119 <= y <= 1208:
            out.append((y, int(r["lunar_month"]), r["is_leap"] == "1",
                        jdn(int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"]))))
    return sorted(out, key=lambda t: t[3])


def clusters(xs, gap=60):
    xs = sorted(xs)
    g = [[xs[0]]] if xs else []
    for x in xs[1:]:
        (g[-1].append(x) if x - g[-1][-1] < gap else g.append([x]))
    return [(sum(c) / len(c), len(c)) for c in g]


def parse(cache):
    """以宋历月序（月名）为行锚；各栏之字归纵坐标最近之锚；年月由「宋月序 + 朔干支」对张表宋历月序定，漏识之月另列。"""
    zm = zhang_months()
    zi = next(i for i, t in enumerate(zm) if t[0] == 1120 and t[1] == 1)
    rows, missing = [], []
    for b in PAGES:
        toks = json.loads((Path(cache) / f"p{b}.json").read_text())
        toks = [dict(x=(t[0] + t[2]) / 2, y=(t[1] + t[3]) / 2, h=t[3] - t[1], t=t[4].replace(" ", "")) for t in toks]
        ms = [t for t in toks if month_of(t["t"])]
        if not ms:
            continue
        cl = [c for c, n in clusters([t["x"] for t in ms], 80) if n >= 2]
        song_x, xia_x = cl[0], (cl[1] if len(cl) > 1 else cl[0] + 460)
        dx = xia_x - song_x
        anchors = sorted([t for t in ms if abs(t["x"] - song_x) < 60], key=lambda t: t["y"])
        cols = {"宋大小": (song_x + 0.15 * dx, song_x + 0.45 * dx), "宋朔": (song_x + 0.45 * dx, song_x + 0.8 * dx),
                "夏月": (xia_x - 0.2 * dx, xia_x + 0.15 * dx), "夏大小": (xia_x + 0.15 * dx, xia_x + 0.45 * dx),
                "夏朔": (xia_x + 0.45 * dx, xia_x + 0.8 * dx), "异同": (xia_x + 0.8 * dx, xia_x + 1.42 * dx), "文献": (xia_x + 1.42 * dx, 1e9)}
        cells = [dict() for _ in anchors]
        for t in toks:
            if t in anchors or not anchors:
                continue
            i = min(range(len(anchors)), key=lambda k: abs(anchors[k]["y"] - t["y"]))
            if abs(anchors[i]["y"] - t["y"]) > 55:
                continue
            for k, (x0, x1) in cols.items():
                if x0 <= t["x"] < x1:
                    cells[i].setdefault(k, []).append(t)
        for a, c in zip(anchors, cells):
            sm, sl = month_of(a["t"])
            txt = lambda k: "".join(t["t"] for t in sorted(c.get(k, []), key=lambda t: (round(t["y"] / 20), t["x"])))
            # 大小栏取纵坐标最近之字（页首行上方有表头「月大小」）
            near = lambda k: min(c.get(k, []), key=lambda t: abs(t["y"] - a["y"]))["t"][:1] if c.get(k) else ""
            g = next((fix_gz(t["t"]) for t in c.get("宋朔", []) if fix_gz(t["t"])), None)
            # 张表中自当前起 4 月内找同月序（且同干支，若识得）之月
            j = next((k for k in range(zi, min(zi + 4, len(zm))) if (zm[k][1], zm[k][2]) == (sm, sl)
                      and (g is None or GZ[(zm[k][3] + 49) % 60] == g)), None)
            note = ""
            if j is None:
                j = next((k for k in range(zi, min(zi + 4, len(zm))) if (zm[k][1], zm[k][2]) == (sm, sl)), None)
                note = "宋朔与张表不合"
                if j is None:
                    continue
            for k in range(zi, j):
                missing.append(zm[k])
            y, m, leap, jd = zm[j]
            zi = j + 1
            xm = next((month_of(t["t"]) for t in c.get("夏月", []) if month_of(t["t"])), None)
            xg = next((fix_gz(t["t"]) for t in c.get("夏朔", []) if fix_gz(t["t"])), None)
            rows.append(dict(页=b, 公元年=y, 宋月=("闰" if leap else "") + str(m), 宋大小=near("宋大小"), 宋朔=g or "",
                             夏月=(("闰" if xm[1] else "") + str(xm[0])) if xm else "", 夏大小=near("夏大小"), 夏朔=xg or "",
                             异同=txt("异同"), 文献=re.sub(r"[MHИW]{1,3}[BВ]\.?N[oO]", "Инв.No", txt("文献")), 宋朔JDN=jd, 核=note))
    return rows, missing


def check_song(rows):
    for r in rows:
        j = r["宋朔JDN"]
        r["夏朔JDN"] = ""
        if r["夏朔"]:
            dd = (GZ.index(r["夏朔"]) - (j + 49) % 60 + 30) % 60 - 30
            if abs(dd) <= 2:
                r["夏朔JDN"] = j + dd
            else:
                r["核"] = (r["核"] + "；" if r["核"] else "") + f"夏朔距宋朔 {dd} 日"


if __name__ == "__main__":
    if sys.argv[1] == "--ocr":
        ocr(sys.argv[2], sys.argv[3])
        sys.exit()
    rows, missing = parse(sys.argv[1])
    # 看图补录：识别漏行（月名残缺或「十二月」误识为「二月」）
    rows = [r for r in rows if not (r["公元年"] == 1149 and r["宋月"] == "2" and r["宋朔"] == "乙卯")]
    zj = {(y, ("闰" if l else "") + str(m)): j for y, m, l, j in zhang_months()}
    for pg, y, sm, ssz, sg, xm, xsz, xg, note in PATCH:
        rows.append(dict(页=pg, 公元年=y, 宋月=sm, 宋大小=ssz, 宋朔=sg, 夏月=xm, 夏大小=xsz, 夏朔=xg, 异同=note,
                         文献="Инв.No.8085", 宋朔JDN=zj[(y, sm)], 核="看图补录"))
    rows.sort(key=lambda r: r["宋朔JDN"])
    for r in rows:
        if r["公元年"] == 1168 and r["宋月"] == "11":
            r["核"] = "书误：夏朔印作戊子，二历异同作朔同，当作戊午（下月朔方为戊子）"
            r["夏朔"] = "戊午"
    for r in rows:
        if r["公元年"] == 1158 and r["宋月"] in ("6", "7"):
            r["核"] = "书中夏历大小与其朔日不合（六月己丑至七月己未 30 日、七月己未至八月戊子 29 日，书作六月小、七月大），照录"
    check_song(rows)
    print("张表有而未识得之宋历月：", [(y, ("闰" if l else "") + str(m)) for y, m, l, _ in missing])
    out = ROOT / "data/calendar/xixia_song_shuorun.csv"
    with open(out, "w", encoding="utf-8", newline="") as f:
        f.write("# 彭向前《俄藏西夏历日文献整理研究》（2018）下篇二「西夏历与宋历朔闰对照表（1120—1207 年）」书页 469—514 转录"
                "（scripts/calendar_data/xixia_songli_table.py，OCR 后按坐标还原、看图复核）。宋朔JDN 取本项目张表，夏朔JDN 由夏朔干支与宋朔之差推得。\n")
        w = csv.DictWriter(f, fieldnames=["页", "公元年", "宋月", "宋大小", "宋朔", "夏月", "夏大小", "夏朔", "异同", "文献", "宋朔JDN", "夏朔JDN", "核"])
        w.writeheader()
        w.writerows(rows)
    bad = [r for r in rows if r["核"]]
    print(len(rows), "行；待核", len(bad))
    for r in bad[:40]:
        print(r["页"], r["公元年"], r["宋月"], r["宋朔"], r["夏月"], r["夏朔"], r["异同"], r["核"])
