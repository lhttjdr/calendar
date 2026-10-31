"""王韬《春秋朔闰日至考》（据张培瑜《中国古代历法》表 3-7 左栏转录，第 5 级转述）与张培瑜春秋历比较经文、传文纪日。

每格：月建 + 正朔干支 + 闰位（闰=年终，闰四=四月后）。王韬的逐月朔书中未列，这里在相邻两年正朔之间按月数均分取整，朔日可差 ±1 日，
故书朔比较另算 ±1 日；张培瑜历同样也算一遍均分版，以保证两边条件相同。
两谱都不独立：张谱由经文拟合，王谱也参考传文（如襄二十七改月以合传），比较只说明哪个月序更贴合文本。

输出 data/calendar/wangtao_3_7.csv、wangtao_compare.csv。
用法（仓库根）：uv run --project scripts python scripts/calendar_data/wangtao_compare.py
"""
import csv
import re
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import chunqiu_check as c  # noqa: E402
import gaiyuan_date as g  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
W = """丑辛巳 丑丙子闰 丑庚子 丑甲午 丑戊子闰 丑壬子 丑丁未闰 寅庚午 丑乙丑 丑己未闰四 丑癸未
丑丁丑 丑壬申闰 丑丙申 丑庚寅 丑甲申闰 丑戊申 丑癸卯闰 丑丙寅 丑辛酉 丑乙卯闰 丑己卯 丑癸酉闰 寅丁酉 丑壬辰 丑丙戌 丑庚辰闰 丑甲辰 丑己亥
子癸巳闰 丑丁巳 丑辛亥闰 丑乙亥 丑己巳 丑甲子 子戊午闰 丑壬午 子丁丑闰 丑庚子 丑乙未闰四 丑己未 丑癸丑 丑丁未闰五 丑辛未 丑丙寅 丑庚申闰 丑甲申 丑戊寅 子癸酉闰 丑丁酉 丑辛卯 子乙酉闰 丑己酉 丑甲辰 子戊戌闰 丑壬戌 子丙辰闰 丑庚辰 丑甲戌 子己巳闰 丑癸巳
丑丁亥 子壬午闰五
丑乙巳闰十一 寅己巳 丑甲子 丑戊午 子壬子 子丁未 子辛丑闰 子乙丑 子庚申闰 丑甲申 子戊寅闰 丑壬寅 子丙申闰 丑庚申 丑甲寅 子戊申 子癸卯闰 丑丁卯 子辛酉闰 丑乙酉 丑己卯 子甲戌 子戊辰闰 子壬辰 子丁亥闰 丑辛亥 子乙巳 子庚子 子甲午 亥戊子闰 子壬子 亥丙午 亥辛丑闰八
子甲子闰 子戊子 子癸未闰 丑丁未 子辛丑 子乙未闰 丑己未 子甲寅 子戊申闰八 子壬申 子丙寅 子辛酉闰四 子乙酉 子己卯 子癸酉闰 子丁酉 子壬辰 亥丙戌闰
子庚戌 子甲辰 亥戊戌 亥壬辰闰 子丙辰 亥辛亥闰 子乙亥 子己巳闰五 子癸巳 子戊子闰 子壬子 子丙午 子庚子闰 子甲子 子己未闰 丑癸未 子丁丑 子辛未
亥丙寅闰 子己丑 子甲申 亥戊寅闰 子壬寅 子丁酉 亥辛卯闰 子乙卯 子己酉 亥癸卯闰 子丁卯 亥壬戌闰 子丙戌 子庚辰闰七 子甲辰 子戊戌 子癸巳闰 子丙辰
子辛亥闰 子乙亥 子己巳 子甲子闰 子丁亥 子壬午 子丙子闰十 子庚子 子甲午 子己丑闰 子癸丑 子丁未闰 子辛未 子乙丑 子庚申 亥甲寅闰 子戊寅 子壬申 亥丁卯闰 子辛卯 亥乙酉闰八 子戊申 子癸卯闰 子丁卯 子辛酉 子丙辰 亥庚戌闰四 子甲戌 子戊辰闰五 子壬辰 子丁亥
亥辛巳闰十 子甲辰 子己亥 亥甲午闰四 子戊午 子壬子闰七 子丙子 子庚午闰八 子甲午 子戊子 子癸未闰 子丙午 子辛丑 子丙申 亥庚寅闰八 子甲寅 子戊申闰 子壬申 子丙寅 亥庚申闰八 子乙酉 子己卯闰 子壬寅 子丁酉 子壬辰闰 子丙辰 子庚戌 亥甲辰闰五 子戊辰 子壬戌闰五 子丙戌 子辛巳
子乙亥 亥己巳闰五 子甲午 子戊子闰十 子壬子 子丙午 亥辛丑闰 子甲子 子己未 亥癸丑闰六 子丁丑 子辛未闰 子乙未 子庚寅闰 子癸丑
子丁未 子辛丑闰 子丙寅 子庚申 子甲寅闰 子戊寅 子癸酉闰 丑丁酉 子辛卯 子丙戌闰五 子庚戌 子甲辰 子戊戌闰 子壬戌 子丁巳 子辛亥闰 子乙亥""".split()
# 宣五书印「子乙巳」，与书列日期 605.12.3 及前后年间隔不合，按丙辰。
NUMS = {"": 12, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "十": 10, "十一": 11}


def cells():
    out = []
    for i, x in enumerate(W):
        m = re.match(r"(.)(..)(?:闰(.*))?$", x)
        out.append({"年": -721 + i, "建": m[1], "gz": m[2], "闰": None if m[3] is None else NUMS[m[3]]})
    return out


def wang_table():
    """y → {月: (朔, 下月朔)}，只含正常月；另带正朔 JDN。"""
    cs = cells()
    # 取最近张谱正朔的同干支日
    for x in cs:
        i = x["年"] + 721
        z = int((c.CQ_E + (12 * i + sum(c.CQ_LEAP[:i])) * c.CQ_LUNAR + 0.5) // 1)
        k = g.GANZHI.index(x["gz"])
        x["j"] = min((z + d for d in range(-40, 41) if (z + d + 49) % 60 == k), key=lambda j: abs(j - z))
    tab = {}
    for a, b in zip(cs, cs[1:]):
        n = 12 + (a["闰"] is not None)
        st = [a["j"] + int((b["j"] - a["j"]) * t / n + 0.5) for t in range(n + 1)]
        m, d = 1, {}
        for t in range(n):
            if a["闰"] is not None and t == a["闰"]:
                continue
            d[m] = (st[t], st[t + 1])
            m += 1
        tab[a["年"]] = d
    return cs, tab


def zhang_table():
    tab, tab2 = {}, {}
    for y in range(-721, -479):
        ms = c.chunqiu_year(y)
        tab[y] = {m: (ms[m - 1], ms[m]) for m in range(1, 13)}
        n = len(ms) - 1
        nx = c.chunqiu_year(y + 1)[0]
        st = [ms[0] + int((nx - ms[0]) * t / n + 0.5) for t in range(n + 1)]
        tab2[y] = {m: (st[m - 1], st[m]) for m in range(1, 13)}
    return tab, tab2


def hit(tab, r):
    d = tab.get(int(r["年"]))
    if not d or int(r["月"]) not in d:
        return None
    a, b = d[int(r["月"])]
    k = g.GANZHI.index(r["干支"])
    inn = any((x + 49) % 60 == k for x in range(a, b))
    diff = (k - (a + 49)) % 60
    diff = diff - 60 if diff > 30 else diff
    return inn, diff


def main():
    cs, wt = wang_table()
    gaps = []
    for a, b in zip(cs, cs[1:]):
        d = (g.GANZHI.index(b["gz"]) - g.GANZHI.index(a["gz"])) % 60
        ok = d in ((54, 55) if a["闰"] is None else (23, 24))
        if not ok:
            gaps.append((a["年"], a["gz"], a["闰"], b["gz"], d))
    print(f"王栏 {len(cs)} 年；闰 {sum(x['闰'] is not None for x in cs[:242])}（前 242 年，书称 88）；建 {Counter(x['建'] for x in cs)}（书 245 年：寅3 丑67 子151 亥24）")
    print("相邻年间隔不合：", gaps)
    with open(ROOT / "data/calendar/wangtao_3_7.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 王韬《朔闰日至考》据张培瑜《中国古代历法》表 3-7 左栏转录（wangtao_compare.py）；闰空为无闰，12 为年终闰。\n")
        w = csv.DictWriter(f, fieldnames=["年", "建", "正朔", "闰", "JDN"])
        w.writeheader()
        for x in cs:
            w.writerow({"年": x["年"], "建": x["建"], "正朔": x["gz"], "闰": x["闰"] if x["闰"] is not None else "", "JDN": x["j"]})
    zt, zt2 = zhang_table()
    def load(n):
        with open(ROOT / "data/calendar" / n, encoding="utf-8") as f:
            return list(csv.DictReader(line for line in f if not line.startswith("#")))
    jing = load("chunqiu_check.csv")
    zhuan = [r for r in load("zuozhuan_check.csv") if r["同经"] == "0"]
    out = []
    for name, rows in (("经", jing), ("传", zhuan)):
        res = {"张": [0, 0, 0, 0], "张均分": [0, 0, 0, 0], "王": [0, 0, 0, 0]}
        both = Counter()
        for r in rows:
            hs = {}
            for nm, tab in (("张", zt), ("张均分", zt2), ("王", wt)):
                h = hit(tab, r)
                hs[nm] = h
                if h:
                    res[nm][0] += h[0]
                    res[nm][1] += 1
                    if r["朔"] == "1":
                        res[nm][2] += abs(h[1]) <= 1
                        res[nm][3] += 1
            if hs["张"] and hs["王"]:
                both[(hs["张"][0], hs["王"][0])] += 1
                if hs["张"][0] != hs["王"][0]:
                    out.append({"文": name, "公": r["公"], "公年": r["公年"], "年": r["年"], "月": r["月"], "干支": r["干支"], "合": "张" if hs["张"][0] else "王", "原文": r["原文"]})
        print(name, {k: f"{v[0]}/{v[1]} 朔±1 {v[2]}/{v[3]}" for k, v in res.items()}, "只张合", both[(True, False)], "只王合", both[(False, True)], "都不合", both[(False, False)])
    with open(ROOT / "data/calendar/wangtao_compare.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 经、传纪日在张培瑜春秋历与王韬谱间判定不同者（wangtao_compare.py）；合=只合哪一谱。\n")
        w = csv.DictWriter(f, fieldnames=list(out[0]))
        w.writeheader()
        w.writerows(out)
    yrs = Counter((o["公"], o["公年"], o["文"], o["合"]) for o in out)
    print("按年：", sorted(yrs.items(), key=lambda kv: -kv[1])[:30])


if __name__ == "__main__":
    main()
