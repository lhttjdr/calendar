"""汪曰桢《历代长术辑要》按语抽取：从 ctext 识别文本（本地缓存 hanshu_cache/ctext_wang/ch1—10.html，不入库）逐年抽出按语，
输出 data/calendar/汪曰桢_按语.csv。

汪书逐年列「岁干支、年数、各月朔干支、闰月、气」，其后为「按……」考辨：引史书、碑志、《通鉴目录》等，指出与推步合否、当时实行之改动
（如「正月本癸未朔，當時強進為甲申朔」）。按语所引史料属第一类实历的线索，须对原书扫描页（doc/references/calendar/[汪曰桢]歷代長術輯要/）
与所引原书核正后才能用；本表文字为机器识别，**未校**。

年份对齐：每条以岁干支起、随列「正某某」（正月朔干支）。以岁干支与正月朔干支在张培瑜历表中定公元年——只用来给书页定位，不作证据。
识别错字多，对不上的条目公元年留空。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/wang_anyu_extract.py
"""
import csv
import html
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CACHE = ROOT / "hanshu_cache/ctext_wang"
G, Z = "甲乙丙丁戊己庚辛壬癸", "子丑寅卯辰巳午未申酉戌亥"
GZ = [G[i % 10] + Z[i % 12] for i in range(60)]
# 识别常见混淆
FIX = str.maketrans({"巴": "巳", "已": "巳", "戍": "戌", "末": "未", "成": "戌", "寸": "", "斗": ""})
SOURCE_PAT = re.compile(r"碑|墓|誌|志銘|銘|石刻|券|磚|塼|造像|題名|題記|告|敕|詔|文書|殘|帖")


def text_of(v):
    t = (CACHE / f"ch{v}.html").read_text(encoding="utf-8", errors="ignore")
    t = re.sub(r"<script.*?</script>", "", t, flags=re.S)
    x = html.unescape(re.sub(r"<[^>]+>", "\n", t))
    return [ln.strip() for ln in x.split("\n") if ln.strip()]


def jdn(y, m, d):
    a = (14 - m) // 12
    yy, mm = y + 4800 - a, m + 12 * a - 3
    if (y, m, d) >= (1582, 10, 15):
        return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - yy // 100 + yy // 400 - 32045
    return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - 32083


def zhang_zhengyue():
    out = {}
    for r in csv.DictReader(open(ROOT / "vendor/calendar/chinese_calendar/实历朔日表.csv", encoding="utf-8")):
        if r["lunar_month"] == "1" and r["is_leap"] == "0":
            j = jdn(int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"]))
            out[int(r["lunar_year"])] = GZ[(j + 49) % 60]
    return out


def main():
    zz = zhang_zhengyue()
    rows = []
    hdr = re.compile(r"([甲乙丙丁戊己庚辛壬癸][子丑寅卯辰巳午未申酉戌亥])[^正甲乙丙丁戊己庚辛壬癸]{0,4}正\S{0,2}?([甲乙丙丁戊己庚辛壬癸][子丑寅卯辰巳午未申酉戌亥])")
    for v in range(1, 11):
        # 页码行换成标记，全册连成一串
        buf, marks = [], []
        for ln in text_of(v):
            if re.fullmatch(r"\d{1,3}", ln):
                marks.append((len("".join(buf)), ln))
                continue
            buf.append(ln.translate(FIX))
        s = "".join(buf)
        page_at = lambda i: max([p for o, p in marks if o <= i] or [""], key=lambda p: int(p) if p else -1)
        heads = [(m.start(), m.group(1), m.group(2)) for m in hdr.finditer(s)]
        # 年份：岁干支与正月朔都合张表且唯一者为锚，前后按岁干支逐一递增传播
        ce = [None] * len(heads)
        for k, (_, sg, zy) in enumerate(heads):
            c = [y for y in zz if GZ[(y - 4) % 60] == sg and zz[y] == zy]
            if len(c) == 1:
                ce[k] = c[0]
        for k in range(1, len(heads)):
            if ce[k] is None and ce[k - 1] is not None and GZ[(ce[k - 1] + 1 - 4) % 60] == heads[k][1]:
                ce[k] = ce[k - 1] + 1
        for k in range(len(heads) - 2, -1, -1):
            if ce[k] is None and ce[k + 1] is not None and GZ[(ce[k + 1] - 1 - 4) % 60] == heads[k][1]:
                ce[k] = ce[k + 1] - 1
        for k, (i, sg, zy) in enumerate(heads):
            j = heads[k + 1][0] if k + 1 < len(heads) else len(s)
            seg = s[i:j]
            for a in re.findall(r"按(.*?)(?=按|$)", seg):
                rows.append({"册": v, "页": page_at(i), "岁干支": sg, "正月朔": zy, "公元（推定）": ce[k] if ce[k] is not None else "",
                             "引实物": "1" if SOURCE_PAT.search(a) else "", "按语（识别未校）": a})
    out = ROOT / "data/calendar/汪曰桢_按语.csv"
    with open(out, "w", encoding="utf-8", newline="") as fh:
        fh.write("# 汪曰桢《历代长术辑要》按语，自 ctext 机器识别文本抽取（scripts/calendar_data/wang_anyu_extract.py），文字未校；"
                 "公元年以岁干支与正月朔干支在张培瑜历表中定位，只作检索用。引用前须对原书扫描页核正。\n")
        w = csv.DictWriter(fh, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)
    print(out.relative_to(ROOT), len(rows), "条；定出公元年", sum(1 for r in rows if r["公元（推定）"] != ""),
          "；引实物线索", sum(1 for r in rows if r["引实物"]))


if __name__ == "__main__":
    main()
