"""《后汉书·天文志》（续汉书志第十至十二，今本卷100–102）月与五星「犯、入、乘、守、掩、食、在」记录：逐条解析纪年与日干支，
按实历朔日表换算为儒略日数（JDN），并把目标匹配到陈卓体系星官（data/sky_culture/chinese_asterisms.csv，system = chenzhuo）。
写出 data/phenomena/houhanshu_star_records.csv，供 rust/core 检验（天体与星官成员星的角距、入宿度）。

- 正文与刘昭注所引《古今注》都收（source 列区分）；注文多只写月日，年月从正文上下文继承。
- 日期有效规则：日干支与事件同句，或事件与上一条记录同句（以「。」为界）；否则不收（如「是岁太白经太微」、
  「荧惑犯舆鬼质星之应也」之类占辞复述）。
- 主语省略（「壬寅，犯心大星」）沿用上一条的天体。
- 「在某宿若干度」另记入宿度（lodge、lodge_deg）。
- 只按历法换算日期，不用天象。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/qinhan_eval/houhanshu_star_records.py
维基文库原文缓存于 scripts/sky_culture/_cache/。
"""
import csv
import re
import sys
import urllib.parse
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
import gaiyuan_date as g  # noqa: E402

ROOT = HERE.parents[2]
CACHE = ROOT / "scripts/sky_culture/_cache"
OUT = ROOT / "data/phenomena/houhanshu_star_records.csv"
ASTERISMS = ROOT / "data/sky_culture/chinese_asterisms.csv"

# 繁→简（只收星官名、天体名、动词中出现的字）
T2S = str.maketrans({
    "東": "东", "軒": "轩", "轅": "辕", "輿": "舆", "營": "营", "壁": "壁", "牽": "牵", "須": "须", "歲": "岁", "熒": "荧", "畢": "毕",
    "參": "参", "觿": "觽", "虛": "虚", "婁": "娄", "張": "张", "軫": "轸", "鎮": "镇", "藩": "藩", "籓": "藩", "掖": "掖", "門": "门",
    "宮": "宫", "紫": "紫", "執": "执", "法": "法", "將": "将", "軍": "军", "貫": "贯", "觸": "触", "經": "经", "後": "后", "頭": "头",
    "兩": "两", "質": "质", "為": "为", "陽": "阳", "陰": "阴", "庫": "库", "樓": "楼", "龍": "龙", "車": "车", "騎": "骑", "關": "关",
    "鍵": "键", "鉤": "钩", "鈐": "钤", "衛": "卫", "從": "从", "進": "进", "賢": "贤", "華": "华", "蓋": "盖", "長": "长", "閣": "阁", "壘": "垒",
    "陣": "阵", "鈇": "𫓧", "鉞": "钺", "積": "积", "屍": "尸", "興": "兴", "郎": "郎", "諸": "诸", "侯": "侯", "鍾": "钟", "鐘": "钟",
    "邊": "边", "傳": "传", "舍": "舍", "廷": "廷", "帝": "帝", "坐": "座", "座": "座", "於": "于", "閒": "间", "間": "间", "內": "内",
})

CN = {"元": 1, "一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9, "十": 10}


def cnnum(s):
    if s == "十":
        return 10
    if s.startswith("十"):
        return 10 + CN[s[1]]
    if "十" in s:
        a, b = s.split("十")
        return CN[a] * 10 + (CN[b] if b else 0)
    if len(s) == 2 and s[0] in CN and s[1] in CN:  # 「三二」之类不出现，保守
        raise ValueError(s)
    return CN[s]


ERA = {"建武": 25, "中元": 56, "永平": 58, "建初": 76, "元和": 84, "章和": 87, "永元": 89, "元興": 105, "元兴": 105,
       "延平": 106, "永初": 107, "元初": 114, "永寧": 120, "永宁": 120, "建光": 121, "延光": 122, "永建": 126,
       "陽嘉": 132, "阳嘉": 132, "永和": 136, "漢安": 142, "汉安": 142, "建康": 144, "永嘉": 145, "本初": 146,
       "建和": 147, "和平": 150, "元嘉": 151, "永興": 153, "永兴": 153, "永壽": 155, "永寿": 155, "延熹": 158,
       "永康": 167, "建寧": 168, "建宁": 168, "熹平": 172, "光和": 178, "中平": 184, "初平": 190, "興平": 194,
       "兴平": 194, "建安": 196}
GZ = "[甲乙丙丁戊己庚辛壬癸][子丑寅卯辰巳已午未申酉戌戍亥]"
NUM = "[元一二三四五六七八九十]+"
DATE = re.compile(
    rf"(?P<era>{'|'.join(ERA)})?(?:(?P<yr>{NUM})年)?[春夏秋冬]?(?:(?P<mon>閏|闰|正|十二|十一|十|[二三四五六七八九])月)?(?P<gz>{GZ})?$"
)
BODY = {"月": "Moon", "熒惑": "Mars", "荧惑": "Mars", "火": "Mars", "太白": "Venus", "金": "Venus", "歲星": "Jupiter", "岁星": "Jupiter",
        "木": "Jupiter", "填星": "Saturn", "鎮星": "Saturn", "镇星": "Saturn", "土": "Saturn", "辰星": "Mercury", "水": "Mercury"}
BODY_RE = "|".join(sorted(BODY, key=len, reverse=True))
VERB_RE = "逆行守|犯|入|乘|守|掩|食|在|貫|贯|觸|触"
EVENT = re.compile(rf"^(?P<body>{BODY_RE})?(?:星)?(?P<verb>{VERB_RE})(?P<target>.+)$")
LODGES = "角亢氐房心尾箕斗牛女虚危室壁奎娄胃昴毕觜参井鬼柳星张翼轸"
LODGE_FULL = {"东井": "井", "舆鬼": "鬼", "鬼": "鬼", "营室": "室", "东壁": "壁", "牵牛": "牛", "须女": "女", "南斗": "斗", "七星": "星",
              "觜觽": "觜"}


def fetch(title):
    CACHE.mkdir(parents=True, exist_ok=True)
    f = CACHE / (title.replace("/", "_") + ".txt")
    if not f.exists():
        url = "https://zh.wikisource.org/w/index.php?title=" + urllib.parse.quote(title) + "&action=raw"
        req = urllib.request.Request(url, headers={"User-Agent": "lunar-calendar-research/0.1"})
        f.write_bytes(urllib.request.urlopen(req, timeout=60).read())
    return f.read_text(encoding="utf-8")


def segments(t):
    """去 <ref>、展开 {{YL|…}}，把 {{*|…}} 注文标为 note。返回 [(文本, 是否注)]，按原文顺序。"""
    t = re.sub(r"<ref>.*?</ref>", "", t, flags=re.S)
    t = re.sub(r"\{\{YL\|([^}|]*)[^}]*\}\}", r"\1", t)
    out, buf, depth, i = [], [], 0, 0
    while i < len(t):
        if t.startswith("{{*|", i):
            if buf:
                out.append(("".join(buf), depth > 0))
                buf = []
            depth += 1
            i += 4
            continue
        if depth and t.startswith("{{", i):
            depth += 1
            i += 2
            continue
        if depth and t.startswith("}}", i):
            depth -= 1
            if depth == 0:
                out.append(("".join(buf), True))
                buf = []
            i += 2
            continue
        buf.append(t[i])
        i += 1
    if buf:
        out.append(("".join(buf), depth > 0))
    return out


QUOTE = re.compile(r"([^，。「」：]{0,12})曰：「([^「」]*)」?")


def drop_other_quotes(seg):
    """占辞等引文（「某曰：『…』」）去掉，只留《古今注》所载（其为实录）。"""
    def repl(m):
        speaker = m.group(1)
        return m.group(0) if speaker.endswith("古今注") else speaker + "。" + "　" * len(m.group(2)) + "。"
    return QUOTE.sub(repl, seg)


ERA_YEAR = re.compile(rf"({'|'.join(ERA)})((?:{NUM})年)")


def load_asterisms():
    names = {}
    for row in csv.reader(l for l in open(ASTERISMS, encoding="utf-8") if not l.startswith("#")):
        if row[0] != "chenzhuo":
            continue
        names[row[2].translate(T2S)] = row[1]
    # 带括注的星官名（「钺(附东井)」「钩钤(附房)」）另以主名索引；主名重复者（积尸：大陵、舆鬼）不建，由别名表指定
    base = {}
    for n, i in list(names.items()):
        b = re.split(r"[（(]", n)[0]
        if b != n:
            base.setdefault(b, []).append(i)
    for b, ids in base.items():
        if len(ids) == 1 and b not in names:
            names[b] = ids[0]
    return names


# 史书惯用名 → 陈卓体系星官名；合成星官（垣）以「+」连接多个星官，检验时取成员合集
ALIASES = {"斗": "南斗", "婺女": "须女", "女": "须女", "太微": "太微左垣+太微右垣", "天市": "天市左垣+天市右垣",
           "紫宫": "紫微东垣+紫微西垣", "紫微": "紫微东垣+紫微西垣", "右执法": "太微右垣", "左执法": "太微左垣",
           "西蕃上将": "太微右垣", "西藩上将": "太微右垣", "东蕃上将": "太微左垣", "东藩上将": "太微左垣",
           "西上将": "太微右垣", "东上将": "太微左垣"}


def match_target(target, ast):
    """目标 → (类别, 星官 id 或天体, 星官名, 限定语)。合成星官 id 以「+」连接。"""
    t = target.translate(T2S)
    for alias in sorted(ALIASES, key=len, reverse=True):
        if t.startswith(alias) and not any(t.startswith(n) and len(n) > len(alias) for n in ast):
            ids = [ast.get(x) for x in ALIASES[alias].split("+")]
            if all(ids):
                return ("asterism", "+".join(ids), alias, t[len(alias):])
    for b, en in sorted(BODY.items(), key=lambda x: -len(x[0])):
        if t.startswith(b.translate(T2S)) and b not in ("火", "水", "金", "木", "土") or t.startswith(b + "星"):
            return ("body", en, b, t[len(b):])
    best = ""
    for name in ast:
        if t.startswith(name) and len(name) > len(best):
            best = name
    if best:
        return ("asterism", ast[best], best, t[len(best):])
    return ("unknown", "", "", t)


def lodge_degree(target):
    """「舆鬼一度」「东井二十度」「斗初」→ (宿, 度)；不成则 None。"""
    t = target.translate(T2S)
    m = re.match(r"^(东井|舆鬼|营室|东壁|牵牛|须女|南斗|七星|觜觽|[" + LODGES + r"])(初|[一二三四五六七八九十]+)度?(半|少|太)?", t)
    if not m:
        return None
    lodge = LODGE_FULL.get(m.group(1), m.group(1))
    d = 0.0 if m.group(2) == "初" else float(cnnum(m.group(2)))
    d += {"半": 0.5, "少": 0.25, "太": 0.75}.get(m.group(3) or "", 0.0)
    return lodge, d


def main():
    ast = load_asterisms()
    rows = []
    for juan in ("100", "101", "102"):
        text = fetch(f"後漢書/卷{juan}")
        era = yr = mon = gz = None
        leap = False
        last_body = None
        fresh = False
        fresh_sentence = -1
        last_sentence = -1
        sent_no = 0
        for seg, note in segments(text):
            # 校勘记之后不再收
            if "==校勘記==" in seg or "==校勘记==" in seg:
                break
            if note and not seg.lstrip().startswith("古今注"):
                continue  # 非《古今注》的注文（占辞、训诂）
            seg = drop_other_quotes(seg)
            for sent in re.split(r"[。！？]", seg):
                sent_no += 1
                for clause in re.split(r"[，；：「」『』、\s]", sent):
                    clause = clause.strip("　 ")
                    if not clause or clause.startswith("=="):
                        continue
                    # 子句内带帝号的纪年（「孝和永元元年」「延熹四年」）：先取年号与年，再把前缀剥去
                    ey = ERA_YEAR.search(clause)
                    if ey:
                        clause = clause[ey.start():]
                    m = DATE.match(clause)
                    if m and any(m.group(k) for k in ("era", "yr", "mon", "gz")) and m.group(0):
                        if m["era"]:
                            era = m["era"]
                        if m["yr"]:
                            yr, mon, gz = cnnum(m["yr"]), None, None
                        if m["mon"]:
                            leap = m["mon"] in ("閏", "闰")
                            mon = None if leap else (1 if m["mon"] == "正" else cnnum(m["mon"]))
                            if leap:
                                mon = "leap"
                            gz = None
                        if m["gz"]:
                            gz = m["gz"].replace("已", "巳").replace("戍", "戌")
                            fresh = True
                            fresh_sentence = sent_no
                        continue
                    # 日期与事件同在一个子句（如「元年三月甲寅月犯…」少见），先剥日期前缀
                    e = EVENT.match(clause)
                    if not e:
                        continue
                    # 省略主语：只在与上一条记录同句时沿用其天体
                    body = e["body"] or (last_body if sent_no == last_sentence else None)
                    if not body:
                        continue
                    # 日期须与事件同句（或与上一条记录同句，如「甲子，月犯…，壬寅，犯…」）；否则多为占辞对前事的复述
                    if not ((fresh and fresh_sentence == sent_no) or sent_no == last_sentence):
                        continue
                    if era is None or yr is None or mon is None or gz is None:
                        continue
                    year = ERA[era] + yr - 1
                    try:
                        if mon == "leap":
                            lm = next(lm for ly, lm, lp, _ in g._months() if ly == year and lp)
                            start, end = g.month_span(year, lm, True)
                            month_label = f"闰{lm}"
                        else:
                            start, end = g.month_span(year, mon, False)
                            month_label = str(mon)
                    except Exception:
                        continue
                    off = (g.GANZHI.index(gz) - (start + 49) % 60) % 60
                    in_month = start + off < end
                    jdn = start + off if in_month else min((start + off - 60, start + off), key=lambda x: abs(x - (start + end) // 2))
                    kind, tid, tname, qual = match_target(e["target"], ast)
                    ld = lodge_degree(e["target"]) if e["verb"] == "在" else None
                    if e["verb"] == "在" and not ld:
                        if kind != "asterism":
                            continue
                    if kind == "unknown" and not ld:
                        continue
                    yy, mm, dd = g.from_jdn(jdn)
                    rows.append(dict(
                        juan=juan, source="古今注" if note else "正文", era=era, year_in_era=yr, year=year, month=month_label,
                        ganzhi=gz, jdn_start=jdn, jdn_end=jdn, julian=f"{yy}-{mm:02d}-{dd:02d}", in_month=int(in_month),
                        bodies=BODY[body] if body in BODY else body, verb=e["verb"].translate(T2S), target_kind=kind, target_id=tid,
                        target_name=tname, qualifier=qual[:12], lodge=ld[0] if ld else "", lodge_deg=ld[1] if ld else "",
                        text=sent.strip()[:120],
                    ))
                    last_body = body
                    last_sentence = sent_no
                    fresh = False
    # 字段内不留半角逗号、引号，便于下游按逗号切分
    for r in rows:
        for k, v in r.items():
            if isinstance(v, str):
                r[k] = v.replace(",", "，").replace('"', "＂").replace("\n", " ").replace("\r", "")
    cols = list(rows[0].keys())
    with open(OUT, "w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=cols)
        w.writeheader()
        w.writerows(rows)
    from collections import Counter
    print(len(rows), "条", Counter(r["bodies"] for r in rows), Counter(r["target_kind"] for r in rows), Counter(r["verb"] for r in rows))
    print("干支不在该月：", sum(1 for r in rows if not r["in_month"]))


if __name__ == "__main__":
    main()
