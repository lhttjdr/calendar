"""《春秋》经文纪日检验春秋历与古六历（第 2 级资料检验第 4b、4d 级）。

经文取自维基文库《春秋左氏傳》各公「經」节（底本未对校点校本，第 2 级资料的转录）。逐条记 年、月、日干支、是否书朔，再看：
- 在月内：该干支是否落在某历该年该月之内。任意干支落入一个月的机会约 29.5/60 ≈ 49%，这是基线，不是 0；
- 书朔者：该干支是否恰为该月朔日（偶合机会约 1/60），这才是强检验。
闰月记录不计入（经文不言闰在何月之后，各历闰法也不同）。
历：上游春秋历（固定闰年表，朔策 30328/1027）；古六历（push-algorithms.md 5.1），各按建子、建丑两种岁首试算，因春秋前期岁首多在丑月。
春秋历的闰年表是今人按《春秋》排定的，合经文不是独立证据。

输出 data/calendar/chunqiu_jingwen.csv（经文纪日）、data/calendar/chunqiu_check.csv（逐条逐历）、data/phenomena/chunqiu_solar_eclipses.csv（日食，供 historical_eclipses）。
用法（仓库根）：uv run --project scripts python scripts/calendar_data/chunqiu_check.py <缓存目录>
然后（rust 目录）：cargo run --release -p lunar-core --example historical_eclipses --no-default-features --features real-f64 -- solar
../data/phenomena/chunqiu_solar_eclipses.csv ../data/phenomena/chunqiu_solar_eclipses_check.csv 116.98 35.6；
第二遍本脚本生成 chunqiu_eclipse_candidates.csv（无日食者的移月候选），同法算 chunqiu_eclipse_candidates_check.csv。
缓存目录下缺文件时自动从维基文库下载。
"""
import csv
import re
import sys
import urllib.parse
import urllib.request
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gaiyuan_date as g  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
DUKES = [("隱公", -721), ("桓公", -710), ("莊公", -692), ("閔公", -660), ("僖公", -658), ("文公", -625),
         ("宣公", -607), ("成公", -589), ("襄公", -571), ("昭公", -540), ("定公", -508), ("哀公", -493)]
NUM = {"正": 1, "一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9, "十": 10,
       "十一": 11, "十有一": 11, "十二": 12, "十有二": 12}
GZR = "|".join(g.GANZHI)
# 季节词只认句首，且其后须是标点、「王」或月份，以免误伤人名（如「劉夏」）
TOK = re.compile(r"(閏月)|(十有[一二]|十[一二]|十|正|[一二三四五六七八九])月|(" + GZR + r")"
                 r"|(?:^|(?<=[。．，年]))([春夏秋冬])(?=[，．、]|王|[正一二三四五六七八九十])|(日有食之)")

# 上游 chunqiu_cmonth（index_c.js）
CQ_LEAP = [int(c) for c in "010010101001001001001010001001010001000011011001000100101001001100000101010100010100010100001001101001001001001001000101100100101000100100100101100010100100100101000100110010001100010100101001000101010100010011000010101000101010010010100100100"]
CQ_LUNAR = 30328 / 1027
CQ_E = 1457727.761054236
SOLAR, LUNAR = 365.25, 29 + 499 / 940
PARAM = {"周": (-104, 1683430.5001, 1683430.5001), "黄帝": (170, 1783510.5001, 1783510.5001), "殷": (-47, 1704250.5001, 1704250.5001),
         "鲁": (-481, 1545730.5001, 1545730.5001 - 1.554255319148936), "颛顼": (14, 1726575.5001 - SOLAR / 8, 1726575.5001),
         "夏": (444, 1883590.5001, 1883590.5001),        # 冬至版，上游用于战国
         "夏雨水": (444, 1883650.5001 - 60.875, 1883650.5001)}  # 雨水版，上游用于春秋


def chunqiu_year(y):
    """春秋历 y 年各月朔 JDN（闰在岁末）。"""
    o = y + 721
    acc = sum(CQ_LEAP[:o])
    m0 = CQ_E + (12 * o + acc) * CQ_LUNAR
    return [int((m0 + j * CQ_LUNAR + 0.5) // 1) for j in range(12 + CQ_LEAP[o] + 1)]


def guliuli_year(name, y):
    """古六历：(朔 JDN 列, 无中气月下标)，自子月起（上游 guliuli_cmonth）。"""
    ye, je, jl = PARAM[name]
    w0 = je + (y - ye - 1) * SOLAR
    w1 = w0 + SOLAR
    m0 = jl + ((((w0 + 1.5) // 1) - 0.5 - jl) // LUNAR) * LUNAR
    n = 13 if ((m0 + 13 * LUNAR + 0.5) // 1) < ((w1 + 0.5) // 1) + 0.1 else 12
    days, leap = [], -1
    for e in range(n):
        t = m0 + e * LUNAR
        days.append(int((t + 0.5) // 1))
        nx = (t + LUNAR + 0.5) // 1
        a = ((12 / SOLAR * (((t + 0.5) // 1) - 0.5 - w0)) // 1) + 1
        if ((w0 + a * SOLAR / 12 + 0.5) // 1) > nx - 0.1:
            leap = e
    return days, leap


def guliuli_span(name, y, month, jian):
    """岁首为 jian（0 子、1 丑）时 y 年 month 月的 (朔, 下月朔)。闰月跳过，岁首前的闰月归上一年。"""
    seq = []
    for yy in (y, y + 1):
        days, leap = guliuli_year(name, yy)
        if len(days) == 12:
            leap = -1
        k = 0
        for i, d in enumerate(days):
            if i == leap:
                continue
            nxt = days[i + 1] if i + 1 < len(days) else guliuli_year(name, yy + 1)[0][0]
            if i + 1 == leap:
                nxt = days[i + 2] if i + 2 < len(days) else guliuli_year(name, yy + 1)[0][0]
            seq.append(((yy - y) * 12 + k, d, nxt))
            k += 1
    t = month - 1 + jian
    for k, d, nxt in seq:
        if k == t:
            return d, nxt
    raise ValueError((name, y, month))


def fetch(cache, title):
    p = cache / (title.replace("/", "_") + ".txt")
    if not p.exists():
        url = "https://zh.wikisource.org/w/index.php?title=" + urllib.parse.quote(title) + "&action=raw"
        req = urllib.request.Request(url, headers={"User-Agent": "lunar-calendar-research/0.1"})
        p.write_bytes(urllib.request.urlopen(req, timeout=60).read())
    return p.read_text(encoding="utf-8")


def source(cache):
    cache.mkdir(parents=True, exist_ok=True)
    for name, _ in DUKES:
        # 个别年份（莊公十八年、襄公二十五年）另立子页，以模板嵌入
        text = re.sub(r"\{\{:(春秋左氏傳/[^}]+)\}\}", lambda m: fetch(cache, m[1]), fetch(cache, f"春秋左氏傳/{name}"))
        yield name, text


def parse(cache):
    start = dict(DUKES)
    out = []
    for duke, text in source(cache):
        for sec in re.split(r"(?m)^(?===[^=])", text):
            m = re.match(rf"=={duke}([元一二三四五六七八九十]+)年", sec)
            j = re.search(r"===\s*經\s*===\n(.*?)(?=\n===|\Z)", sec, re.S)
            if not m or not j:
                continue
            n = g_num(m[1])
            y = start[duke] + n - 1
            cur, leap = None, False
            for line in j[1].split("\n"):
                line = re.sub(r"\{\{YL\|([^|}]*)\|[^}]*\}\}", r"\1", line)
                line = re.sub(r"\{\{\+\|", "", line).replace("}}", "").strip()
                line = re.sub(r"([乙丁己辛癸])已", r"\1巳", line)  # 底本偶混「已」「巳」（如成公十七年「丁已朔」）
                if not line:
                    continue
                last_end = -99
                for t in TOK.finditer(line):
                    if t[4]:
                        cur, leap = None, False
                    elif t[1]:
                        cur, leap = None, True
                    elif t[2]:
                        cur, leap = NUM[t[2]], False
                    elif t[5]:
                        if t.start() - last_end > 3:   # 未书日干支的日食
                            pre = line[max(0, t.start() - 2):t.start()]
                            out.append(rec(duke, n, y, cur, leap, "", int("朔" in pre), 1, line))
                    else:
                        tail = line[t.end():t.end() + 12]
                        m2 = re.match(r"[，．、。]?(朔)?[，．、。]?(日有食之)?", tail)
                        out.append(rec(duke, n, y, cur, leap, t[3], int(bool(m2[1])), int(bool(m2[2])), line))
                        if m2[2]:
                            last_end = t.end() + m2.end()
    return out


# 张书表 3-8（页 201—202）：(公, 公年, 干支, 书所列月)。「襄二 六月壬辰」书作壬辰，今本经文作庚辰，按庚辰对照
ZHANG_3_8 = [("隱公", 2, "庚辰", "八月"), ("隱公", 4, "戊申", "（三）月"), ("桓公", 2, "戊申", "四月"), ("桓公", 5, "己丑", "正月"), ("桓公", 12, "壬辰", "八月"),
             ("桓公", 17, "丙午", "二月"), ("莊公", 28, "甲寅", "三月"), ("閔公", 2, "辛丑", "八月"), ("僖公", 1, "丁巳", "十二月"), ("僖公", 9, "甲子", "九月"),
             ("僖公", 18, "丁亥", "八月"), ("僖公", 27, "乙巳", "（九）月"), ("僖公", 28, "壬申", "（十）月"), ("僖公", 33, "乙巳", "十二月"), ("文公", 9, "癸酉", "九月"),
             ("文公", 12, "戊午", "十二月"), ("文公", 13, "己丑", "十二月"), ("文公", 14, "乙亥", "五月"), ("宣公", 2, "壬子", "二月"), ("宣公", 9, "辛酉", "九月"),
             ("宣公", 12, "乙卯", "六月"), ("宣公", 17, "丁未", "（二）月"), ("成公", 2, "庚寅", "（九）月"), ("成公", 4, "壬申", "三月"), ("成公", 9, "丙子", "七月"),
             ("成公", 10, "丙午", "（六）月"), ("成公", 17, "壬申", "十一月"), ("襄公", 2, "庚辰", "六月"), ("襄公", 3, "戊寅", "（七）月"), ("襄公", 4, "己酉", "三月"),
             ("襄公", 7, "丙戌", "十二月"), ("襄公", 9, "己亥", "十二月"), ("襄公", 25, "己巳", "八月"), ("襄公", 27, "辛巳", "七月"), ("襄公", 28, "乙未", "十二月"),
             ("襄公", 29, "庚午", "（六）月"), ("昭公", 1, "己酉", "十一月"), ("昭公", 8, "壬午", "十月"), ("昭公", 22, "乙丑", "四月"), ("昭公", 24, "丁酉", "八月"),
             ("昭公", 28, "癸巳", "七月"), ("定公", 4, "癸巳", "二月"), ("定公", 4, "庚午", "十一月"), ("定公", 4, "庚辰", "十一月"), ("定公", 15, "辛巳", "（十）月"),
             ("哀公", 3, "甲午", "四月")]


def compare_zhang(rows):
    """本项目「外」与张书表 3-8 逐条对照，写 chunqiu_zhang_3_8.csv。"""
    mine = {(r["公"], r["公年"], r["干支"]): r for r in rows if r["春秋历"] == "外"}
    book = {(d, n, gz): m for d, n, gz, m in ZHANG_3_8}
    out = []
    for k in sorted(set(mine) | set(book), key=lambda k: ([d for d, _ in DUKES].index(k[0]), k[1])):
        r = mine.get(k)
        out.append({"公": k[0], "公年": k[1], "干支": k[2], "张书表3-8": book.get(k, ""), "本项目（不合）": f"{r['月']}月" if r else "",
                    "日食": ("食" if r and r["日食"] else ""), "原文": r["原文"] if r else ""})
    with open(ROOT / "data/calendar/chunqiu_zhang_3_8.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 张培瑜《中国古代历法》表 3-8 不合历日与本项目不合条目对照（chunqiu_check.py）。\n")
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    both = sum(1 for o in out if o["张书表3-8"] and o["本项目（不合）"])
    print(f"表 3-8 {len(book)} 行，本项目不合 {len(mine)}，两边都有 {both}")
    print("只本项目：", " ".join(f"{o['公'][0]}{o['公年']}{o['干支']}{o['日食']}" for o in out if not o["张书表3-8"]))
    print("只张书：", " ".join(f"{o['公'][0]}{o['公年']}{o['干支']}" for o in out if not o["本项目（不合）"]))


def rec(duke, n, y, cur, leap, gz, shuo, ec, line):
    return {"公": duke, "公年": n, "年": y, "月": cur or "", "闰": int(leap), "干支": gz, "朔": shuo, "日食": ec, "原文": line[:60]}


def g_num(s):
    if s == "元":
        return 1
    if "十" in s:
        a, b = s.split("十")
        return (NUM[a] if a else 1) * 10 + (NUM[b] if b else 0)
    return NUM[s]


def write_eclipses(ecl):
    """日食记录日：取春秋历该月内（或最近）的该干支；未书干支者取该月朔。日期只由历法换算，不用天象。"""
    out = []
    for r in ecl:
        y, m, gz = r["年"], r["月"], r["干支"]
        ms = chunqiu_year(y) if y + 721 < len(CQ_LEAP) else None
        a = ms[m - 1]
        note = ""
        if gz:
            k = g.GANZHI.index(gz)
            mid = a + 14
            j = min((mid + i for i in range(-30, 31) if (mid + i + 49) % 60 == k), key=lambda x: abs(x - mid))
            if not a <= j < ms[m]:
                note = "干支不在春秋历该月内"
        else:
            j, note = a, "经文未书日干支，取春秋历该月朔"
        yy, mm, dd = g.from_jdn(j)
        out.append({"emp": r["公"], "era": "", "year_in_era": r["公年"], "year": y, "month": m, "ganzhi": gz, "position": "朔" if r["朔"] else "",
                    "extent": "既" if "既" in r["原文"] else "", "lodge": "", "calendar": "春秋历", "jdn": j, "julian": f"{yy}-{mm:02d}-{dd:02d}",
                    "lunar_day": j - a + 1, "note": note, "text": r["原文"], "site_lon": 116.98, "site_lat": 35.6})
    with open(ROOT / "data/phenomena/chunqiu_solar_eclipses.csv", "w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    # 无日食者的候选：有干支者前后移 60 日倍数（月份抄误），未书干支者前后移一至三朔望月；襄公二十七年另试《左传》「乙亥」
    chk = ROOT / "data/phenomena/chunqiu_solar_eclipses_check.csv"
    if chk.exists():
        with open(chk, encoding="utf-8") as f:
            miss = {(r["emp"], r["year_in_era"], r["month"]) for r in csv.DictReader(f) if r["verdict"] == "无日食"}
        cand = []
        for o in out:
            if (o["emp"], str(o["year_in_era"]), str(o["month"])) not in miss:
                continue
            for k in (-3, -2, -1, 1, 2, 3):
                cand.append({**o, "jdn": o["jdn"] + (60 * k if o["ganzhi"] else round(k * 29.53059)), "note": f"移{k}"})
            if o["emp"] == "襄公" and o["ganzhi"] == "乙卯":
                j0 = o["jdn"]
                for j in range(j0 - 90, j0 + 60):
                    if (j + 49) % 60 == g.GANZHI.index("乙亥"):
                        cand.append({**o, "jdn": j, "ganzhi": "乙亥", "note": "传文异文乙亥"})
        for c in cand:
            yy, mm, dd = g.from_jdn(c["jdn"])
            c["julian"], c["lunar_day"] = f"{yy}-{mm:02d}-{dd:02d}", ""
        with open(ROOT / "data/phenomena/chunqiu_eclipse_candidates.csv", "w", encoding="utf-8", newline="") as f:
            w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
            w.writeheader()
            w.writerows(cand)
    print(f"日食 {len(out)} 条：书朔 {sum(1 for o in out if o['position'])}，书干支 {sum(1 for o in out if o['ganzhi'])}")


def main():
    recs = parse(Path(sys.argv[1] if len(sys.argv) > 1 else "chunqiu_cache"))
    fields = list(recs[0].keys())
    with open(ROOT / "data/calendar/chunqiu_jingwen.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 《春秋》经文纪日（scripts/calendar_data/chunqiu_check.py，底本维基文库《春秋左氏傳》經节）；年为天文纪年，月空为经文未书月。\n")
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        w.writerows(recs)
    dated = [r for r in recs if r["干支"] and r["月"] and not r["闰"] and r["年"] + 721 < len(CQ_LEAP)]  # 上游闰表止于前 479
    print(f"经文纪日 {len([r for r in recs if r['干支']])} 条，有月可验 {len(dated)}，书朔 {sum(r['朔'] for r in dated)}，日食 {sum(r['日食'] for r in recs)}")

    write_eclipses([r for r in recs if r["日食"]])

    def in_cq(r):
        ms = chunqiu_year(r["年"])
        return ms[r["月"] - 1], ms[r["月"]]

    models = {"春秋历": in_cq}
    for name in PARAM:
        for jian, jn in ((0, "子"), (1, "丑")) + (((2, "寅"),) if name.startswith("夏") else ()):
            models[f"{name}·建{jn}"] = (lambda nm, ji: lambda r: guliuli_span(nm, r["年"], r["月"], ji))(name, jian)
    rows, tally = [], defaultdict(lambda: defaultdict(lambda: [0, 0, 0, 0]))
    for r in dated:
        k = g.GANZHI.index(r["干支"])
        row = {x: r[x] for x in ("公", "公年", "年", "月", "干支", "朔", "日食")}
        for name, fn in models.items():
            a, b = fn(r)
            hit = any((d + 49) % 60 == k for d in range(a, b))
            sh = (a + 49) % 60 == k
            row[name] = ("朔" if sh else "内") if hit else "外"
            for d in (r["公"], "全"):
                t = tally[name][d]
                t[0] += hit
                t[1] += 1
                if r["朔"]:
                    t[2] += sh
                    t[3] += 1
        row["原文"] = r["原文"]
        rows.append(row)
    with open(ROOT / "data/calendar/chunqiu_check.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 《春秋》经文纪日逐条逐历（内：干支在该月内；朔：恰为朔日；外：不在月内）。闰月、未书月者不计。\n")
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)
    compare_zhang(rows)
    print("历            在月内           书朔恰合")
    for name in models:
        t = tally[name]["全"]
        print(f"{name:8} {t[0]}/{t[1]} {t[0] / t[1]:.1%}   {t[2]}/{t[3]}")
    for name in ("春秋历", "周·建子", "周·建丑", "鲁·建子", "鲁·建丑", "殷·建丑"):
        print(name, " ".join(f"{d[:1]}{tally[name][d][0]}/{tally[name][d][1]}" for d, _ in DUKES if tally[name][d][1]))


if __name__ == "__main__":
    main()
