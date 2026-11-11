"""从《明实录》逐卷缓存（hanshu_cache/mingshilu/，由 mingshilu_fetch.py 抓取）抽出「某年某月某某朔」，
写 data/calendar/mingshilu_shuo.csv：农历年（以年号推得）、月、闰、朔干支、卷名、原句。

正文为识典古籍服务端渲染的简体全文，每段一行；月首只认段首的「某月某某朔」。年号年取本句所带者；只书「二年」者承最近明书的年号（太宗实录卷一至九革除建文，作建文）；不带年号的月首承本卷（或前卷）最近一次出现的年号年，月序倒退时进一年。
同一月重复者只录一次；同一月出现两个不同干支者两条都录，备注「异文」。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/mingshilu_shuo.py
"""
import csv
import re
from pathlib import Path

SRC = Path("hanshu_cache/mingshilu")
OUT = Path("data/calendar/mingshilu_shuo.csv")

# 年号元年（公历年，农历年首所在）
ERAS = {
    "至正": 1341, "吳": 1367, "吴": 1367, "洪武": 1368, "建文": 1399, "永樂": 1403, "永乐": 1403, "洪熙": 1425,
    "宣德": 1426, "正統": 1436, "正统": 1436, "景泰": 1450, "天順": 1457, "天顺": 1457, "成化": 1465,
    "弘治": 1488, "正德": 1506, "嘉靖": 1522, "隆慶": 1567, "隆庆": 1567, "萬曆": 1573, "万历": 1573,
    "泰昌": 1620, "天啟": 1621, "天启": 1621, "崇禎": 1628, "崇祯": 1628,
}
GZ = "甲子乙丑丙寅丁卯戊辰己巳庚午辛未壬申癸酉甲戌乙亥丙子丁丑戊寅己卯庚辰辛巳壬午癸未甲申乙酉丙戌丁亥戊子己丑庚寅辛卯壬辰癸巳甲午乙未丙申丁酉戊戌己亥庚子辛丑壬寅癸卯甲辰乙巳丙午丁未戊申己酉庚戌辛亥壬子癸丑甲寅乙卯丙辰丁巳戊午己未庚申辛酉壬戌癸亥"
GANZHI = [GZ[i:i + 2] for i in range(0, 120, 2)]
NUM = {"元": 1, "正": 1, "一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9, "十": 10, "冬": 11, "臘": 12, "腊": 12}


def cn(s: str) -> int:
    s = s.replace("有", "")
    if s in ("元", "正"):
        return 1
    if "十" in s:
        a, _, b = s.partition("十")
        return (NUM[a] if a else 1) * 10 + (NUM[b] if b else 0)
    return NUM[s]


ERA_RE = "|".join(sorted(ERAS, key=len, reverse=True))
NUMC = "[元正一二三四五六七八九十有]+"
PAT = re.compile(
    rf"(?:(?P<era>{ERA_RE})?(?P<y>{NUMC})年)?(?:春|夏|秋|冬)?(?P<leap>閏|闰)?(?P<m>正|十有[一二]|十[一二]?|[一二三四五六七八九])月(?P<gz>[甲乙丙丁戊己庚辛壬癸][子丑寅卯辰巳午未申酉戌亥])朔")


def main():
    rows, seen = [], {}
    cur = None  # (农历年, 月)
    cur_era = None  # 最近一次明书的年号：不带年号的「二年正月」承之
    for f in sorted(SRC.glob("*.txt")):
        juan = f.stem.split("_", 1)[1]
        paras = f.read_text(encoding="utf-8").split("\n")[1:]
        # 太宗实录卷一至卷九记靖难事，革除建文年号，只书「元年」「二年」……，即建文纪年
        if re.search(r"太宗.*實錄卷之[一二三四五六七八九][上下]?$", juan):
            cur_era = "建文"
        # 月首只认段首（实录每月另起一段，以「某月某某朔」开头）；段中引及他月者不取。
        # 段首「○」与「九月○戊辰朔」月、日间的「○」先去掉
        hits = [(0, m) for p in paras for m in [PAT.match(p.lstrip("○").replace("月○", "月", 1))] if m]
        for _, m in hits:
            if m.group("gz") not in GANZHI:
                continue
            mon = cn(m.group("m"))
            leap = m.group("leap") is not None
            if m.group("era"):
                cur_era = m.group("era")
            if m.group("y") and cur_era:
                y = ERAS[cur_era] + cn(m.group("y")) - 1
            elif cur is None:
                continue
            else:
                y = cur[0]
                if (mon, leap) < (cur[1], False) and not (leap and mon == cur[1]):
                    y += 1
            cur = (y, mon)
            key = (y, mon, leap)
            gz = m.group("gz")
            if key in seen:
                if gz in seen[key]:
                    continue
                seen[key].add(gz)
                note = "异文"
            else:
                seen[key] = {gz}
                note = ""
            rows.append((y, mon, int(leap), gz, juan, m.group(0), note))
    rows.sort(key=lambda r: (r[0], r[1], r[2]))
    with OUT.open("w", encoding="utf-8", newline="") as fo:
        fo.write("# 《明实录》（识典古籍 LS0026 录文）月首「某月某某朔」，scripts/calendar_data/mingshilu_shuo.py 生成；年为农历年（年号推得）\n")
        w = csv.writer(fo)
        w.writerow(["年", "月", "闰", "朔", "卷", "原句", "备注"])
        w.writerows(rows)
    print(OUT, len(rows), "条；异文", sum(1 for r in rows if r[6]))


if __name__ == "__main__":
    main()
