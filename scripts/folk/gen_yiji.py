#!/usr/bin/env python3
"""
解析《欽定協紀辨方書》卷十（宜忌，以神为纲）与卷十一（用事，以事为纲），
得到「神 × 事」的宜、忌两张关系表，并逐格互校。

输入：vendor/folk/wikisource/欽定協紀辨方書/卷10.wikitext、卷11.wikitext（维基文库四库本录文）
输出：
  data/folk/yiji_shi.csv   卷十一：每事的宜神、忌神（附加条件原文另列）
  data/folk/yiji_shen.csv  卷十：每神的宜事、忌事（条件句原文另列）
  data/folk/yiji_crosscheck.csv  两卷互校的不一致项
用法（仓库根）：python3 scripts/folk/gen_yiji.py
"""

import csv
import re
import sys

SRC = "vendor/folk/wikisource/欽定協紀辨方書/"

# 异体字归一（只用于比对；输出保留归一后的字形）
VARIANTS = str.maketrans({
    "㑹": "會", "徳": "德", "册": "冊", "劵": "券", "宫": "宮", "牆": "墻", "壊": "壞", "啓": "啟",
    "産": "產", "葢": "蓋", "醖": "醞", "䕶": "護", "逺": "遠", "迴": "廻", "隂": "陰", "刼": "劫",
    "夘": "卯", "虚": "虛", "摇": "搖", "徃": "往", "禄": "祿", "絶": "絕", "厯": "曆", "乂": "又",
    "牀": "牀", "補": "補", "嵗": "歲", "冠": "冠",
})


def norm(s: str) -> str:
    return s.translate(VARIANTS)


def load(vol: str) -> list[str]:
    s = open(SRC + f"卷{vol}.wikitext", encoding="utf-8").read()
    s = re.sub(r"\{\{SK anchor\|([^}|]*)\}\}", r"\1", s)
    s = re.sub(r"\{\{SK notes\|([^}]*)\}\}", r"〔\1〕", s)
    s = re.sub(r"\{\{[^{}]*\}\}", "", s)
    s = re.sub(r"<!--.*?-->", "", s, flags=re.S)
    lines = [norm(l.strip("　 ")) for l in s.split("\n")]
    return [l for l in lines if l and not l.startswith("<")]


L10, L11 = load("10"), load("11")


def block(lines, start, end):
    i = lines.index(start)
    j = lines.index(end, i + 1)
    return [w for l in lines[i + 1:j] for w in re.split("[　 ]+", l) if w]


# ---- 事目词表：卷十一御用六十七事、民用三十七事、通书六十事，另加卷十用到的合称与细目 ----
YU = block(L11, "御用六十七事", "民用三十七事")
MIN = block(L11, "民用三十七事", "通書選擇六十事")
i = L11.index("通書選擇六十事")
TONG = [w for l in L11[i + 1:i + 21] for w in re.split("[　 ]+", l) if w]
assert (len(YU), len(MIN), len(TONG)) == (67, 37, 60), (len(YU), len(MIN), len(TONG))
EXTRA = ["置產室", "冠", "般移", "遠廻", "宣布政事", "修倉庫", "鼓鑄", "苫蓋", "醞釀", "開倉庫出貨財", "塞穴", "針刺",
         "療目", "乘船渡水", "修築園圃", "破土", "安葬", "啟攢", "陳詞訟", "安神", "修祠宇", "運謀算畫計策",
         "破屋壞垣", "補垣塞穴", "整容", "補捉", "立券交易", "上冊進表章", "布政事", "求醫療目"]
SHI = sorted(set(YU + MIN + TONG + EXTRA), key=len, reverse=True)
# 合称拆成御用细目：卷十一御用分列，卷十有时合写
SPLIT = {"覃恩肆赦": ["覃恩", "肆赦"], "施恩惠恤孤惸": ["施恩惠", "恤孤惸"], "行惠愛雪寃枉緩刑獄": ["行惠愛", "雪寃枉", "緩刑獄"],
         "慶賜賞賀": ["慶賜", "賞賀"], "行幸遣使": ["行幸", "遣使"], "詔命公卿招賢": ["詔命公卿", "招賢"],
         "開市立券交易": ["開市", "立券", "交易"], "立券交易": ["立券", "交易"], "宣布政事": ["宣政事", "布政事"],
         "補垣塞穴": ["補垣", "塞穴"], "畋獵取魚": ["畋獵", "取魚"], "開倉庫出貨財": ["開倉庫出貨財"],
         "補捉": ["捕捉"], "求醫療目": ["療目"],
         # 卷十大時條「冠」：维基文库录文脱「帶」，浙大藏四库本影印（06056510.cn）PDF 第 180 頁作「冠帶」
         # 卷十月害條「置產室」：影印本 PDF 第 167 頁亦作「置産室」，原书省文或脱字；卷十一修置産室條列月害，從之
         "置產室": ["修置產室"], "冠": ["冠帶"]}


def seg_shi(s: str):
    out, bad, i = [], "", 0
    while i < len(s):
        for v in SHI:
            if s.startswith(v, i):
                out += SPLIT.get(v, [v])
                i += len(v)
                break
        else:
            bad += s[i]
            i += 1
    return out, bad


# ---- 卷十：以神为纲 ----
i0 = L10.index("表例附") + 1
i1 = L10.index("以上所忌", i0) if "以上所忌" in L10[i0:] else L10.index("宜忌等第表")
body = L10[i0:L10.index("宜忌等第表")]
LEAD = ("宜", "忌", "與", "又", "按", "見", "義", "止", "不", "凡", "舊", "母倉義", "十月", "五月")


def is_head(l: str) -> bool:
    t = re.sub(r"〔[^〕]*〕", "", l)
    if t in ("以上所宜", "以上所忌"):
        return False
    toks = [x for x in re.split("[　 ]+", t) if x]
    return all(len(x) <= 5 for x in toks) and not t.startswith(("宜", "忌", "見", "義")) and "併" not in t


shen_rows = []  # (神组, 类别, 事目, 原文)
cur = None
for l in body:
    if l in ("以上所宜",):
        continue
    if is_head(l):
        cur = l
        continue
    raw = l
    t = re.sub(r"〔[^〕]*〕", "", l)
    if t.startswith("按") or t.startswith("見") or t.startswith("義") or t.startswith("舊本"):
        continue
    kind = None
    for k, pat in [("宜", r"^宜(.*)$"), ("忌", r"^忌(.*)$")]:
        m = re.match(pat, t)
        if m:
            kind = k
            main = m.group(1)
            break
    if kind:
        # 「○止不忌……」之后是例外，不并入忌事
        main = main.split("○")[0]
        # 宜行内接「忌……」（天赦）
        if kind == "宜" and "忌" in main and not main.startswith("忌"):
            main, tail = main.split("忌", 1)
            ev, bad = seg_shi(tail)
            shen_rows.append((cur, "忌", "|".join(ev), raw))
        # 句尾条件、指引
        main = re.split(r"正月建日|見附錄", main)[0]
        if main.startswith("同"):
            shen_rows.append((cur, "忌同", main[1:], raw))
            continue
        # 句中「又為某宜……」另起
        parts = re.split(r"又為(\S{2,4}?)宜", main)
        first = parts[0]
        ev, bad = seg_shi(first.split("與")[0])
        shen_rows.append((cur, kind, "|".join(ev), raw if not bad else raw + f" 〔未切：{bad}〕"))
        for k in range(1, len(parts), 2):
            ev, bad = seg_shi(parts[k + 1])
            shen_rows.append((parts[k], "宜", "|".join(ev), raw))
        continue
    m = re.match(r"^又為(\S{2,4}?)(專忌|宜|忌)(.*)$", t)
    if m:
        ev, bad = seg_shi(m.group(3).split("與")[0])
        shen_rows.append((m.group(1), "宜" if m.group(2) == "宜" else "忌", "|".join(ev), raw))
        continue
    m = re.match(r"^(\S{2,3}?)又(宜|忌)(.*)$", t)
    if m and not t.startswith(("與", "子", "寅", "辰", "巳", "午", "未", "申", "酉", "戌", "亥", "卯", "丑")):
        ev, bad = seg_shi(m.group(3))
        shen_rows.append((m.group(1), m.group(2), "|".join(ev), raw))
        continue
    shen_rows.append((cur, "條件", "", raw))

# ---- 卷十一：以事为纲 ----
SHEN_EXTRA = ["寅日", "丁日", "甲日", "乙日", "庚日", "辛日", "巳日", "午日", "未日", "申日", "酉日", "亥日", "丑日",
              "亥子日", "午申日", "伏社日", "土王用事後", "朔望日", "每月十五日", "每月十二日十五日",
              "每月一日六日十五日十九日二十一日二十三日", "立冬後立春前", "霜降後立春前", "雨水後立夏前",
              "又收日母倉六合五富天倉與月德四相時德開日併者"]
heads10 = set()
for l in body:
    if is_head(l):
        for x in re.split("[　 ]+", re.sub(r"[〔〕]", "　", l)):
            if x:
                heads10.add(x)
SHEN = sorted(heads10 | {"月建", "土府", "大耗", "死神", "月虛", "天火", "招搖", "咸池", "地火", "九焦", "血支", "血忌",
                         "天后", "吉期", "兵寶", "兵福", "天巫", "福德", "天喜", "天醫", "生氣", "時陽", "時陰", "天狗",
                         "除神", "解神", "復日", "重日", "四相", "時德", "天賊", "天倉", "不將", "厭對", "六儀",
                         "臨日", "陽德", "陰德", "普護", "福生", "聖心", "益後", "續世", "要安", "敬安", "玉宇", "金堂",
                         "鳴吠", "鳴吠對", "專日", "伐日", "八專", "往亡", "歸忌", "觸水龍", "九空", "九坎", "八風",
                         "五虛", "五墓", "四忌", "四窮", "四廢", "四耗", "四擊", "兵禁", "大煞", "兵吉", "六合", "五合",
                         "五富", "五離", "土符", "地囊", "月空", "天恩", "天赦", "天願", "母倉", "月恩"}
              | set(SHEN_EXTRA), key=len, reverse=True)


def seg_shen(s: str):
    out, bad, i = [], "", 0
    while i < len(s):
        for v in SHEN:
            if s.startswith(v, i):
                out.append(v)
                i += len(v)
                break
        else:
            bad += s[i]
            i += 1
    return out, bad


ia = L11.index("通書選擇六十事") + 21
shi_rows = []  # (事, 类别, 神, 原文)
cur = None
for l in L11[ia:]:
    if l.startswith("右用事") or l.startswith("欽定"):
        break
    t = re.sub(r"〔[^〕]*〕", "", l)
    if not any(c in t for c in "宜忌併"):
        # 录文脱「卿」，浙大藏四库本影印（06056511.cn）PDF 第 17 頁作「詔命公卿招賢」
        cur = {"詔命公招賢": "詔命公卿招賢"}.get(t, t)
        continue
    if not t.startswith(("宜", "忌")):
        shi_rows.append((cur, "條件", "", l))
        continue
    segs = [(t[0], t[1:])]
    if t[0] == "宜" and "忌月建" in t:  # 安葬：宜、忌同行
        a, b = t[1:].split("忌月建", 1)
        segs = [("宜", a), ("忌", "月建" + b)]
    for kind, seg in segs:
        if seg == "無":
            continue
        if kind == "宜" and "又" in seg:  # 「又某某與某某併者」为条件
            seg, cond = seg.split("又", 1)
            shi_rows.append((cur, "條件", "", "又" + cond))
        names, bad = seg_shen(seg)
        shi_rows.append((cur, kind, "|".join(names), l + (f" 〔未切：{bad}〕" if bad else "")))

with open("data/folk/yiji_shen.csv", "w", newline="", encoding="utf-8") as f:
    w = csv.writer(f)
    w.writerow(["神", "類", "事", "原文"])
    w.writerows(shen_rows)
with open("data/folk/yiji_shi.csv", "w", newline="", encoding="utf-8") as f:
    w = csv.writer(f)
    w.writerow(["事", "類", "神", "原文"])
    w.writerows(shi_rows)

bad = [r for r in shen_rows if "未切" in r[3]] + [r for r in shi_rows if "未切" in r[3]]
print(f"卷十 {len(shen_rows)} 条，卷十一 {len(shi_rows)} 条，未切开 {len(bad)} 条", file=sys.stderr)
for r in bad:
    print("  ", r, file=sys.stderr)


# ---- 两卷互校 ----
def names_of(head: str) -> list[str]:
    """卷十标题「天德　月德〔…〕」取正名（〔〕内异名不随正名继承宜忌）"""
    return [x for x in re.split("[　 ]+", re.sub(r"〔[^〕]*〕", "", head)) if x]


R10 = set()
for shen, kind, shi, _ in shen_rows:
    if kind in ("宜", "忌") and shi:
        for n in names_of(shen):
            for e in shi.split("|"):
                R10.add((n, e, kind))
for shen, kind, ref, _ in shen_rows:  # 「忌同劫煞」
    if kind == "忌同":
        for (n, e, k) in list(R10):
            if n == ref and k == "忌":
                for m in names_of(shen):
                    R10.add((m, e, "忌"))
R11 = set()
for shi, kind, shen, _ in shi_rows:
    if kind in ("宜", "忌") and shen:
        evs, _ = seg_shi(shi)
        for e in evs:
            for n in shen.split("|"):
                R11.add((n, e, kind))
# 只比两卷都有的神、都有的事
shen10 = {n for n, _, _ in R10}
shen11 = {n for n, _, _ in R11}
shi10 = {e for _, e, _ in R10}
shi11 = {e for _, e, _ in R11}
common = lambda r: r[0] in shen10 & shen11 and r[1] in shi10 & shi11
only10 = sorted(r for r in R10 - R11 if common(r))
only11 = sorted(r for r in R11 - R10 if common(r))
with open("data/folk/yiji_crosscheck.csv", "w", newline="", encoding="utf-8") as f:
    w = csv.writer(f)
    w.writerow(["神", "事", "宜忌", "卷十", "卷十一"])
    for r in only10:
        w.writerow([*r, "有", "無"])
    for r in only11:
        w.writerow([*r, "無", "有"])
print(f"互校：卷十 {len(R10)} 格，卷十一 {len(R11)} 格；只见于卷十 {len(only10)}，只见于卷十一 {len(only11)}", file=sys.stderr)
print("  卷十独有神：", sorted(shen10 - shen11), file=sys.stderr)
print("  卷十一独有神：", sorted(shen11 - shen10), file=sys.stderr)
