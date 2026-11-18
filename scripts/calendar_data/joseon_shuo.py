"""从《朝鮮王朝實錄》逐年缓存（hanshu_cache/joseon_sillok/，joseon_sillok_fetch.py 抓取）抽出每月朔日干支，
写 data/calendar/joseon_shuo.csv。

体例：
- 中文维基文库（太祖至仁祖，光海君除外）：逐日标题 ===7月1日===、===閏6月1日===，其下首段「○朔壬午」「○壬午朔」
  「○十一月丁未朔」；他日首段「○甲申，……」。{{校|讹|正}} 取校正字。
- 韩文维基文库《光海君日記》太白山本（中草本）：标题 === 6月 1日 ===，首段多带年干支与月日，
  如「○壬戌六月初一日朔乙丑」「○丙寅六月初二日」「○壬戌六月初三日丁卯」；取首段前部、非本年岁次的干支为日干支。
- 逐日标题的「某月某日」是今人（韩国国史编纂委员会）据干支排定的日序，原文只有干支。
  故只有原文明书「朔」者记「直书」；无朔字的月份，以当月各日（标题日序、首段干支）逆推朔，取多数，记「反推」——
  反推所得实即整理者排日所据的朔，非原文独立证据。
- 一农历年分属前王末年、新王即位年两页者都取；同月两页皆有，取直书者，再按页序取前者；《宣祖修正實錄》只补《宣祖實錄》缺月。

用法（仓库根）：python3 scripts/calendar_data/joseon_shuo.py
"""
import csv
import re
from collections import Counter
from pathlib import Path

SRC = Path("hanshu_cache/joseon_sillok")
OUT = Path("data/calendar/joseon_shuo.csv")
GZ = "甲子乙丑丙寅丁卯戊辰己巳庚午辛未壬申癸酉甲戌乙亥丙子丁丑戊寅己卯庚辰辛巳壬午癸未甲申乙酉丙戌丁亥戊子己丑庚寅辛卯壬辰癸巳甲午乙未丙申丁酉戊戌己亥庚子辛丑壬寅癸卯甲辰乙巳丙午丁未戊申己酉庚戌辛亥壬子癸丑甲寅乙卯丙辰丁巳戊午己未庚申辛酉壬戌癸亥"
GANZHI = [GZ[i:i + 2] for i in range(0, 120, 2)]
GZRE = "[甲乙丙丁戊己庚辛壬癸][子丑寅卯辰巳午未申酉戌亥]"

# (页名前缀, 元年之公历年（农历年首所在）, 显示名)
REIGNS = [("太祖實錄", 1392, "太祖"), ("定宗實錄", 1399, "定宗"), ("太宗實錄", 1401, "太宗"), ("世宗實錄", 1419, "世宗"),
          ("文宗實錄", 1451, "文宗"), ("端宗實錄", 1453, "端宗"), ("世祖實錄", 1455, "世祖"), ("睿宗實錄", 1469, "睿宗"),
          ("成宗實錄", 1470, "成宗"), ("燕山君日記", 1495, "燕山君"), ("中宗實錄", 1506, "中宗"), ("仁宗實錄", 1545, "仁宗"),
          ("明宗實錄", 1546, "明宗"), ("宣祖實錄", 1568, "宣祖"), ("ko_광해군일기태백산본", 1609, "光海君"),
          ("仁祖實錄", 1623, "仁祖"), ("宣祖修正實錄", 1568, "宣祖修正")]
CN = {"一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}

# 明年号元年（公历年）
MING = [("洪武", 1368), ("建文", 1399), ("永樂", 1403), ("洪熙", 1425), ("宣德", 1426), ("正統", 1436), ("景泰", 1450),
        ("天順", 1457), ("成化", 1465), ("弘治", 1488), ("正德", 1506), ("嘉靖", 1522), ("隆慶", 1567), ("萬曆", 1573),
        ("泰昌", 1620), ("天啓", 1621), ("崇禎", 1628), ("順治", 1644)]
NUMS = "元一二三四五六七八九十"


def cnum(n):
    if n == 1:
        return "元"
    t, u = divmod(n, 10)
    return (("" if t == 1 else "一二三四五六七八九"[t - 1]) + "十" if t else "") + ("一二三四五六七八九"[u - 1] if u else "")


def ming_year(y):
    if y == 1620:
        return "萬曆四十八年（泰昌元年）"
    if y >= 1645:
        return "崇禎" + cnum(y - 1627) + "年（明亡後朝鮮仍用崇禎紀年）"
    name, y0 = [e for e in MING if e[1] <= y and e[0] != "泰昌"][-1]
    if name == "建文" and y == 1402:
        return "建文四年（洪武三十五年）"
    return name + cnum(y - y0 + 1) + "年"


def reign_year(stem):
    """页名 → (显示名, 王年名, 公历年, 页序)"""
    for i, (pre, y1, show) in enumerate(REIGNS):
        if stem.startswith(pre + "_"):
            yn = stem[len(pre) + 1:]
            if yn in ("卽位年", "即位年", "즉위년"):
                return show, "卽位年", y1 - 1, i
            if pre.startswith("ko_"):
                n = int(yn[:-1])
            else:
                s = yn[:-1].replace("元", "一")
                if "十" in s:
                    a, _, b = s.partition("十")
                    n = (CN[a] if a else 1) * 10 + (CN[b] if b else 0)
                else:
                    n = CN[s]
            return show, cnum(n) + "年", y1 + n - 1, i
    return None


def clean(s):
    s = re.sub(r"\{\{校\|[^|{}]*\|([^|{}]*)\}\}", r"\1", s)
    s = re.sub(r"\{\{參\|([^|{}]*)\|[^{}]*\}\}", r"\1", s)
    s = re.sub(r"<[^>]+>", "", s)
    return s


HEAD = re.compile(r"^===\s*(閏|윤)?\s*(\d+)月\s*(\d+)日\s*===\s*$", re.M)


def first_para(body):
    for line in clean(body).split("\n"):
        line = line.strip()
        if line.startswith("○"):
            return line
    return ""


def cn_day(t):
    t = t.replace("廿", "二十")
    if "十" in t:
        a, _, b = t.partition("十")
        return (CN[a] if a else 1) * 10 + (CN[b] if b else 0)
    return CN.get(t)


def day_gz(para, year_gz, ko):
    """首段 → (日干支, 是否书朔, 原文日序或 None)"""
    m = re.match(rf"○(?:[春夏秋冬]?閏?[正一二三四五六七八九十]+月)?(朔)?({GZRE})(朔)?", para)
    if not ko:
        return (m.group(2), bool(m.group(1) or m.group(3)), None) if m else (None, False, None)
    # 中草本：「○壬戌六月初一日朔乙丑」「○丙寅六月初二日」「○辛亥九月初三日, 庚子」；
    # 涂改作「戊辰癸酉」（前字涂去）取末一干支，「庚午/申」（旁改地支）取改字
    head = re.sub(r"([甲乙丙丁戊己庚辛壬癸])[子丑寅卯辰巳午未申酉戌亥]/([子丑寅卯辰巳午未申酉戌亥])", r"\1\2", para[:50])
    m = re.search(rf"月\s*初?([一二三四五六七八九十廿]+)日[\s,，]*(朔)?[\s,，]*((?:{GZRE})+)?(朔)?", head)
    day = cn_day(m.group(1)) if m else None
    if m and m.group(3):
        return m.group(3)[-2:], bool(m.group(2) or m.group(4)), day
    k = re.search(rf"朔\s*({GZRE})|({GZRE})\s*朔", head)
    if k:
        return k.group(1) or k.group(2), True, day
    for g in re.finditer(GZRE, head[:m.start()] if m else head[:30]):
        if g.group(0) != year_gz:
            return g.group(0), False, day
    return None, False, day


def year_gz_of(y):
    return GANZHI[(y - 4) % 60]


def main():
    months = {}  # (年, 月, 闰) → [行候选]
    for f in sorted(SRC.glob("*.txt")):
        info = reign_year(f.stem)
        if not info:
            continue
        show, yname, y, order = info
        text = f.read_text(encoding="utf-8")
        ko = f.stem.startswith("ko_")
        m = re.search(r"section\s*=\s*（(" + GZRE + r")）", text)
        if m and m.group(1) != year_gz_of(y):
            # 元年之年按页首岁次校正（世祖、中宗等即位当年即称元年）
            y2 = next((z for z in (y - 1, y + 1) if year_gz_of(z) == m.group(1)), y)  # 只容一年之差；更远者为页首岁次误，不改
            print(f"  {f.stem} 岁次 {m.group(1)}：{y} → {y2}")
            y = y2
        sub = re.search(r"<sub>大明([^<]*)</sub>", text)
        heads = list(HEAD.finditer(text))
        bymonth = {}
        for k, h in enumerate(heads):
            body = text[h.end(): heads[k + 1].start() if k + 1 < len(heads) else len(text)]
            leap, mo, d = bool(h.group(1)), int(h.group(2)), int(h.group(3))
            gz, shuo, tday = day_gz(first_para(body), year_gz_of(y), ko)
            if tday and 1 <= tday <= 30:
                d = tday  # 太白山本原文自书「初N日」，不用标题日序
            if gz:
                bymonth.setdefault((mo, leap), []).append((d, gz, shuo))
        for (mo, leap), days in bymonth.items():
            implied = Counter(GANZHI[(GANZHI.index(gz) - (d - 1)) % 60] for d, gz, _ in days)
            best, cnt = implied.most_common(1)[0]
            d1 = [e for e in days if e[0] == 1]
            # 1 日书朔而紧接的 2—5 日（至少两日）一致指向另一朔者，多为录文讹（如太白山本「朔庚午/申」），从近日；
            # 只看近日，不看全月多数：今人排日偶漏一日，会使月中以后各日整体错开（如世宗二十一年八月「丙子朔，日食」）
            near = {GANZHI[(GANZHI.index(gz) - (d - 1)) % 60] for d, gz, _ in days if 2 <= d <= 5}
            near_n = sum(1 for d, *_ in days if 2 <= d <= 5)
            strong = near_n >= 2 and len(near) == 1 and d1 and next(iter(near)) != d1[0][1]
            if strong:
                best = next(iter(near))
            if d1 and d1[0][2] and not strong:
                shuo, how = d1[0][1], "直书"
                note = "" if shuo == best else f"他日逆推多作{best}"
            else:
                shuo, how = best, "反推"
                first = min((e for e in days if GANZHI[(GANZHI.index(e[1]) - (e[0] - 1)) % 60] == best), key=lambda e: e[0])
                note = f"据{'閏' if leap else ''}{mo}月{first[0]}日{first[1]}"
                if d1 and d1[0][2]:
                    note += f"；1日书朔{d1[0][1]}，与他日不合"
            bad = len(days) - implied[shuo]
            if bad:
                note += ("；" if note else "") + f"{bad}/{len(days)}日逆推不合"
            page = f.stem.replace("ko_광해군일기태백산본_", "광해군일기태백산본/").replace("_", "/")
            src = "韩文维基文库《光海君日記》太白山本" if ko else ("《宣祖修正實錄》" if show == "宣祖修正" else "")
            months.setdefault((y, mo, leap), []).append(dict(
                朝鲜年=("光海君" if ko else show) + yname, 明年号年=("大明" + sub.group(1)) if sub else ming_year(y), 公历年=y, 月=mo,
                闰=int(leap), 朔干支=shuo, 来源=src or "中文维基文库", 依据=how, 页面=page, 备注=note, _ord=(order, how != "直书")))
    rows = []
    for key in sorted(months):
        cands = months[key]
        main_ = [c for c in cands if c["朝鲜年"] and not c["朝鲜年"].startswith("宣祖修正")]
        pool = main_ or cands
        pool.sort(key=lambda c: (c["依据"] != "直书", c["_ord"][0]))
        r = pool[0]
        others = [c for c in cands if c is not r and c["朔干支"] != r["朔干支"]]
        if others:
            r["备注"] += ("；" if r["备注"] else "") + "；".join(f"{c['朝鲜年']}页作{c['朔干支']}（{c['依据']}）" for c in others)
        rows.append(r)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with OUT.open("w", encoding="utf-8", newline="") as fo:
        fo.write("# 《朝鮮王朝實錄》太祖至仁祖逐月朔日（scripts/calendar_data/joseon_shuo.py 由维基文库录文生成）。公历年为农历年首所在；"
                 "依据：直书＝原文书「朔」，反推＝以今人所排日序与日干支逆推（非原文独立证据）。光海君取韩文维基文库太白山本（中草本）。\n")
        cols = ["朝鲜年", "明年号年", "公历年", "月", "闰", "朔干支", "来源", "依据", "页面", "备注"]
        w = csv.DictWriter(fo, fieldnames=cols, extrasaction="ignore")
        w.writeheader()
        w.writerows(rows)
    c = Counter(r["依据"] for r in rows)
    print(f"{len(rows)} 月，{len({r['公历年'] for r in rows})} 年；{dict(c)}")


if __name__ == "__main__":
    main()
