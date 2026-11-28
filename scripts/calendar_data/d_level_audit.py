"""抽查「推步与张表互异而归 ③（无据待核）」之判是否得当（只读分析，不改任何检验产物）。

2026-11-27 第二轮：首轮（c04239d）所列系统性问题已在检验程序中修正（甲 D、E 级判①②须异源两条；乙 南明私家追记、《建康实录》吴部分
按 E；分政权接走马楼、辽石刻、明郑实物、北朝诸史与《金史》天文志；中原一线接明清实录书朔、定朔考证不及之月收纪日、「閏月」、
《汉书》秦汉初书朔晦）。本脚本按新产物重判，「原检验已用」之源见 CONNECTED；首轮「应改」20 月逐一复核见 PREV_FIX。

范围：中原一线互异之月（data/calendar/zhongyuan_reckoning_months.csv「互异」者）与分政权互异之月
（data/calendar/regime_benji_months.csv「互异」者）。

做法：
1. 对全体互异之月重新汇集证据——不仅用原检验所用之条（本纪明书朔、定朔考证、整年实物、出土具注历；分政权为
   regime_benji_check.csv 只合一家之纪日），另查原检验未接入之表：本纪纪日（中原一线互异之月原检验不收纪日；619 年后
   由定朔考证收入）、本纪「閏月」（闰位）、《汉书》秦汉初书朔晦、秦汉初出土历谱、楼兰简纸、吐鲁番唐西州文书、
   大统历实物、时宪书实物、明实录书朔、走马楼吴简、吴买地券、辽代石刻、明郑大统历实物、北朝诸史天文志之条，以及
   唐宋诸考证表、例外表、各政权考证表的结论文字（只作按语）。
2. 按 scripts/calendar_data/shiwu_fenlei.py 之等级与门槛重判（A/B/C 一条可判；D 纪日须同月独立两条；E 后人追修史书
   一条不足；同一史源合并），与原判比较。
3. 按朝代分层随机抽样（固定种子 SEED），中原一线、分政权各抽 N_SAMPLE 月；另把「同月有 ≥2 条证据却仍归 ③」之月全部列入。
4. 逐月给结论（判定得当／应改①／应改②／应改为相同类／等级标错／无法判断），人工复核之月按 MANUAL 覆盖自动结论。

输出 data/calendar/d_level_audit.csv（抽查之月逐月）与 doc/calendar/d-level-audit.md（方法、汇总、逐月表、系统性问题）。
用法（仓库根）：uv run --project scripts python scripts/calendar_data/d_level_audit.py
"""

import csv
import io
import random
import subprocess
from collections import Counter, defaultdict
from pathlib import Path

SEED = 20261127
N_SAMPLE = 60
D = Path("data/calendar")
GZ = ["甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12] for i in range(60)]
GI = {g: i for i, g in enumerate(GZ)}
STD_NAME = {1: "正月", 2: "二月", 3: "三月", 4: "四月", 5: "五月", 6: "六月", 7: "七月", 8: "八月", 9: "九月", 10: "十月",
            11: "十一月", 12: "十二月"}


def rows(path, skip_comment=True):
    with open(path, encoding="utf-8") as f:
        lines = [l for l in f if not (skip_comment and l.startswith("#"))]
    return list(csv.DictReader(io.StringIO("".join(lines))))


def gz_of(jdn):
    return GZ[(jdn + 49) % 60]


def civil_to_jdn(y, m, d):
    """民用历（1582-10-15 前儒略历，天文纪年）→ JDN"""
    a = (14 - m) // 12
    yy = y + 4800 - a
    mm = m + 12 * a - 3
    if (y, m, d) >= (1582, 10, 15):
        return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - yy // 100 + yy // 400 - 32045
    return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - 32083


def nearest(gz, ref):
    """干支 gz 最接近 ref 之 JDN"""
    k = (GI[gz] - (ref + 49)) % 60
    return ref + k if k < 30 else ref + k - 60


def label(m, leap):
    return f"{'闰' if leap else ''}{m}"


def parse_label(s):
    s = s.strip()
    leap = s.startswith("闰") or s.startswith("閏")
    return int(s.lstrip("闰閏")), leap


# ---------------------------------------------------------------- 证据等级与史源（与 rust/core/examples/*_eval.rs 一致）
E_BOOKS = ("houhanshu", "jinshu", "nanshi", "beishi", "xinwudaishi")
# 原检验按 C/D 计、而成书远在其后或为私家追记者（系统性问题候选，见文档）
LATER_BOOKS = {
    "jiankang_shilu_wu": "《建康实录》（唐许嵩撰，记吴事后约五百年）",
    "nanming_xiaotian_jinian": "《小腆纪年》（清徐鼒撰，咸丰、同治间）",
    "nanming_xingzai_yangqiu": "《行在阳秋》（清初戴笠撰，私家追记）",
    "nanming_haidong_yishi": "《海东逸史》（清翁洲老民撰，纪传体追记）",
    "nanming_mingji_nanlue": "《明季南略》（清初计六奇撰，私家追记）",
    "nanming_hongguang_shilu": "《弘光实录钞》（清初黄宗羲撰，私家追记）",
}


def book_level(book, shuo):
    # 乙：南明私家追记、《建康实录》吴部分按 E（检验程序已同）
    if book.startswith(E_BOOKS) or book in LATER_BOOKS:
        return "E"
    return "C" if shuo else "D"


def lineage(book):
    b = book.replace("_benji", "").replace("_zhi", "")
    return {"jiutangshu": "唐", "xintangshu": "唐", "jiuwudaishi": "五代", "xinwudaishi": "五代",
            "beishi": "北朝", "weishu": "北朝", "beiqishu": "北朝", "zhoushu": "北朝", "suishu": "北朝",
            "nanshi": "南朝", "songshu": "南朝", "nanqishu": "南朝", "liangshu": "南朝", "chenshu": "南朝",
            "jinshu": "魏晋", "sanguozhi": "魏晋", "sanguozhi_wushu": "吴", "jiankang_shilu_wu": "吴",
            "hanshu": "汉"}.get(b, b)


def zhongyuan_regime(regime, y):
    if regime in ("西汉", "东汉", "魏", "晋", "宋", "齐", "梁", "陈", "唐", "武周", "后梁", "后唐", "后晋", "后汉", "后周"):
        return True
    return {"隋": y >= 590, "元": y >= 1280, "明": y <= 1644, "清": y >= 1645}.get(regime, False)


# ---------------------------------------------------------------- 张表、月序改制
ZHANG = defaultdict(list)  # 年 → [(jdn, 月, 闰)]
for r in rows("vendor/calendar/chinese_calendar/实历朔日表.csv"):
    j = civil_to_jdn(int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"]))
    ZHANG[int(r["lunar_year"])].append((j, int(r["lunar_month"]), r["is_leap"] == "1"))
for v in ZHANG.values():
    v.sort()
ZHANG_ALL = sorted(x for v in ZHANG.values() for x in v)


def zhang_month(y, m, leap):
    return next((e for e in ZHANG.get(y, []) if e[1] == m and e[2] == leap), None)


def zhang_next(j):
    return next((e[0] for e in ZHANG_ALL if e[0] > j), None)


REFORM = {}
for r in rows(D / "month_label_reforms.csv"):
    REFORM[(r["regime"], int(r["label_year"]), r["label_month"], r["label_is_leap"] == "1")] = (
        int(r["table_lunar_year"]), int(r["table_lunar_month"]), r["table_is_leap"] == "1")


def to_table(regime, y, m, leap, name=""):
    reg = {"西汉": "汉", "东汉": "汉"}.get(regime, regime)
    for nm in ([name.replace("臘", "腊")] if name else []) + [STD_NAME.get(m, "")]:
        t = REFORM.get((reg, y, nm, leap))
        if t:
            return t
    return (y, m, leap)


# ---------------------------------------------------------------- 中原一线：推步逐月（由逐月表干支与张表 JDN 定出推步 JDN）
ZY_MONTHS = rows(D / "zhongyuan_reckoning_months.csv")
RECK = defaultdict(list)  # 年 → [(jdn, 月, 闰, 历)]
by_year = defaultdict(list)
for r in ZY_MONTHS:
    by_year[int(r["年"])].append(r)
for y, rs in by_year.items():
    prev = None
    for r in rs:
        m, leap = parse_label(r["月"])
        zm = zhang_month(y, m, leap)
        if r["张表朔"] in GI and zm and gz_of(zm[0]) == r["张表朔"]:
            ref = zm[0]
        elif prev is not None:
            ref = prev + 30
        elif zm:
            ref = zm[0]
        else:
            ref = None
        if ref is None:
            continue
        j = nearest(r["推步朔"], ref)
        RECK[y].append((j, m, leap, r["历"]))
        prev = j
RECK_ALL = sorted(x for v in RECK.values() for x in v)


def reck_next(j):
    return next((e[0] for e in RECK_ALL if e[0] > j), None)


def judge(a, b, gz, pos):
    """纪日干支在 [a, b) 月内否（书朔、晦、日序者须合其日）"""
    if a is None or b is None:
        return None
    d = a + (GI[gz] - (a + 49)) % 60
    if d >= b:
        return False
    if pos == "朔":
        return d == a
    if pos == "晦":
        return d == b - 1
    if pos.startswith("日") and pos[1:].isdigit():
        return d == a + int(pos[1:]) - 1
    return True


# ---------------------------------------------------------------- 证据项
def ev(side, grade, src, book, text, used, kind="", event=""):
    return {"side": side, "grade": grade, "src": src, "book": book, "lin": lineage(book) if book else src,
            "text": text.replace("\n", " ")[:80], "used": used, "kind": kind, "event": event}


# ======== 中原一线互异之月 ========
ZY_DIFF = rows(D / "zhongyuan_reckoning_diffs.csv")
DING = {(r["年"], r["月"], r["推步朔"]): r for r in rows(D / "dingshuo_zhangbiao_diffs.csv")}
KAO = defaultdict(list)
for f in ["tang_shuo_kaozheng.csv", "song_shuo_kaozheng.csv", "tangsong_cunyi_kaozheng.csv", "wudai_shuo_kaozheng.csv"]:
    for r in rows(D / f):
        KAO[(r["年"], label(int(r["月"]), r["闰"] == "1") if r["月"].isdigit() else r["月"])].append((f, r))
for r in rows(D / "yuan_shuo_kaozheng.csv"):
    KAO[(r["年"], r["月"])].append(("yuan_shuo_kaozheng.csv", r))
EXC = {(r["年"], label(int(r["月"]), r["闰"] == "1")): r for r in rows(D / "dingshuo_exceptions.csv") if r["月"].isdigit()}
ERRATA = {(r["书"], r["年"], r["月"]) for r in rows(D / "benji_shuo_errata.csv")}

zy_books = ["hanshu", "houhanshu", "sanguozhi", "jinshu", "songshu", "nanqishu", "liangshu", "chenshu", "suishu", "beishi",
            "jiutangshu", "xintangshu", "jiuwudaishi", "xinwudaishi", "songshi", "yuanshi", "mingshi", "qingshigao"]
JIRI = defaultdict(list)  # (表年, 月, 闰) → [(书, 行)]
LEAPS = defaultdict(list)  # 年 → [(书, 所接之月, 原文)]
for b in zy_books:
    p = D / "本纪纪日" / f"{b}_benji.csv"
    if p.exists():
        for r in rows(p):
            try:
                y, m = int(r["year"]), int(r["month"])
            except ValueError:
                continue
            if not zhongyuan_regime(r["regime"], y) or r["ganzhi"] not in GI:
                continue
            if b == "hanshu" and y <= -104:
                continue  # 秦汉初由 qinhan_benji_check 计
            t = to_table(r["regime"], y, m, r["leap"] == "1", r["month_name"])
            JIRI[t].append((b, r))
    p = D / "本纪纪日" / f"{b}_benji_閏月.csv"
    if p.exists():
        for r in rows(p):
            try:
                y = int(r["year"])
            except ValueError:
                continue
            if zhongyuan_regime(r["regime"], y):
                LEAPS[y].append((b, r["leap_after"], r["text"]))
SHUO = defaultdict(list)
for r in rows(D / "benji_shuo_check.csv"):
    if r["书"] in ("liaoshi_benji", "jinshi_benji", "weishu_benji", "beiqishu_benji", "zhoushu_benji") or r["政权"] in ("吴", "蜀"):
        continue
    try:
        y = int(r["年"])
        m, leap = parse_label(r["月"])
        jd = int(r["本纪朔 JDN"])
    except ValueError:
        continue
    if not zhongyuan_regime(r["政权"], y):
        continue
    SHUO[to_table(r["政权"], y, m, leap)].append((r, jd))
QINHAN = defaultdict(list)
for r in rows(D / "qinhan_benji_check.csv"):
    QINHAN[(r["夏正年"], r["月"])].append(r)
QINHAN_SW = defaultdict(list)
for r in rows(D / "qinhan_shiwu_months.csv"):
    # 纪年年为建亥年（以其正月所在年记）：十、十一、十二月属夏正前一年
    _y = int(r["年"]) - (1 if int(r["月"]) >= 10 and r["闰"] != "1" else 0)
    QINHAN_SW[(str(_y), label(int(r["月"]), r["闰"] == "1"))].append(r)
LOULAN = defaultdict(list)
for r in rows(D / "loulan_jiri.csv"):
    if r.get("核对") == "1" and r["年"].lstrip("-").isdigit() and r["月"].isdigit():
        LOULAN[(int(r["年"]), int(r["月"]), r["闰"] == "1")].append(r)
TULUFAN = defaultdict(list)
for r in rows(D / "tulufan_jiri.csv"):
    if r["政权"] == "唐" and r["公元"].isdigit() and r["月"].isdigit():
        TULUFAN[(int(r["公元"]), int(r["月"]), r["闰"] == "1")].append(r)
SHIWU = defaultdict(list)  # 大统历、时宪书实物
for f in ["datong_shiwu_months.csv", "shixianshu_months.csv"]:
    for r in rows(D / f):
        if r["年"].isdigit() and r["月"].isdigit():
            SHIWU[(int(r["年"]), int(r["月"]), r["闰"] == "1")].append((f, r))
MSL = defaultdict(list)
for r in rows(D / "mingshilu_shuo.csv"):
    if r["年"].isdigit() and r["月"].isdigit():
        MSL[(int(r["年"]), int(r["月"]), r["闰"] == "1")].append(r)
QSL = defaultdict(list)
for r in rows(D / "qingshilu_shuo_xinfa.csv"):
    if r["年"].isdigit() and r["月"].isdigit():
        QSL[(int(r["年"]), int(r["月"]), r["闰"] == "1")].append(r)
MSL_JK = {}
for r in rows(D / "mingshilu_jiaokan.csv"):
    MSL_JK[(r["年"], label(int(r["月"]), r["闰"] == "1"))] = r
JOSEON = defaultdict(list)
for r in rows(D / "joseon_shuo.csv"):
    if r["公历年"].isdigit() and r["月"].isdigit() and r["依据"] == "直书":
        JOSEON[(r["公历年"], label(int(r["月"]), r["闰"] == "1"))].append(r)
CHUTU = defaultdict(list)
for r in rows(D / "chutu_rili_shuo.csv"):
    if r["年"].lstrip("-").isdigit() and r["月"].isdigit():
        CHUTU[(int(r["年"]), int(r["月"]), r["闰"] == "1")].append(r)


def era_of(y):
    if y <= -104:
        return "秦汉"
    if y < 590:
        return "魏晋南北朝"
    if y < 907:
        return "隋唐"
    if y < 960:
        return "五代"
    if y < 1280:
        return "宋"
    if y < 1368:
        return "元"
    if y < 1645:
        return "明"
    return "清"


def zy_audit(d):
    y = int(d["年"])
    m, leap = parse_label(d["月"])
    tr = next((e for e in RECK.get(y, []) if e[1] == m and e[2] == leap), None)
    a_r = tr[0] if tr else None
    b_r = reck_next(a_r) if a_r else None
    tl = d["张表对应月"]
    if tl and tl != "—":
        tm, tleap = parse_label(tl)
        zt = zhang_month(y, tm, tleap)
    else:
        zt = None
    a_t = zt[0] if zt else None
    b_t = zhang_next(a_t) if a_t else None
    # 张表中与推步同标签之月（闰位之异时，与推步同名之月可能不是「张表对应月」）
    zs = zhang_month(y, m, leap)
    a_s = zs[0] if zs else None
    b_s = zhang_next(a_s) if a_s else None
    items = []
    # 原检验之证据：只取定朔考证之判与实物（A）；书朔、纪日、閏月、秦汉初、实录之条下面逐条另收（标「原检验已用」），免得重计
    for part in [p for p in d["证据"].split("；") if p]:
        if (part.endswith("合推步") or part.endswith("合张表")) and (part.startswith("定朔考证") or part[-4] == "A"):
            g = part[-4]
            items.append(ev("r" if part.endswith("合推步") else "t", g, "原检验", "", part, True))
    ding = DING.get((d["年"], d["月"], d["推步朔"]))
    # 定朔考证按朔日相近配月、张表月名同者，其判即本月之判（纪日以考证所收为准）；否则检验程序按月名另收纪日、閏月
    ding_proper = bool(ding and not ding["张表月名"])
    notes = []
    if ding:
        if ding["纪日证据"]:
            notes.append(f"定朔考证纪日：{ding['纪日证据'][:120]}")
        if ding["已有考证"]:
            notes.append(f"已有考证 {ding['已有考证']}：{ding['判定'][:80]}")
    # 1 本纪明书朔（含原检验已用者；原检验只在 619 年前用之，619 年后经定朔考证）
    for r, jd in SHUO.get((y, m, leap), []):
        if (r["书"], r["年"], r["月"]) in ERRATA:
            notes.append(f"{r['书']}「{r['本纪朔']}朔」已考讹不计")
            continue
        sr, st = jd == a_r, jd == a_s
        if sr != st:
            used = True  # 619 年前与定朔考证不及之月由中原一线检验直接收，余由定朔考证收
            items.append(ev("r" if sr else "t", book_level(r["书"], True), "本纪明书朔", r["书"],
                            f"{r['书']}「{r['本纪朔']}朔」{r['原文'][:40]}", used, "朔", r["本纪朔"]))
    # 2 本纪纪日（原检验于互异之月不收纪日；619 年后之月由定朔考证收入，标 used）
    for b, r in JIRI.get((y, m, leap), []):
        pos = r["position"]
        if pos == "朔":
            continue
        okr = judge(a_r, b_r, r["ganzhi"], pos)
        okt = judge(a_s, b_s, r["ganzhi"], pos) if a_s else judge(a_t, b_t, r["ganzhi"], pos)
        if okr is None or okt is None or okr == okt:
            continue
        used = bool(ding and ding["纪日证据"] and r["text"][:6] in ding["纪日证据"]) or not ding_proper
        items.append(ev("r" if okr else "t", book_level(b, False), "本纪纪日", b, f"{b}：{r['text'][:50]}", used,
                        pos or "日", r["ganzhi"]))
    # 2b 前月纪日：前月两家同朔而月终不同（本月月首之异），前月末日之条可判本月月首
    if a_r is not None and a_s is not None and a_r != a_s:
        prev = [e for e in RECK_ALL if e[0] < a_r]
        if prev:
            p = prev[-1]
            pz = next((e for e in ZHANG_ALL if e[0] == p[0] and e[1] == p[1] and e[2] == p[2]), None)
            py = next((yy for yy, v in RECK.items() if p in v), None)
            if pz and py is not None:
                for b, r in JIRI.get((py, p[1], p[2]), []):
                    pos = r["position"]
                    if pos == "朔":
                        continue
                    okr = judge(p[0], a_r, r["ganzhi"], pos)
                    okt = judge(p[0], zhang_next(p[0]), r["ganzhi"], pos)
                    if okr is None or okt is None or okr == okt:
                        continue
                    used = bool(ding and ding["纪日证据"] and r["text"][:6] in ding["纪日证据"])
                    items.append(ev("r" if okr else "t", book_level(b, False), "本纪纪日（前月）", b,
                                    f"{b}（前月{label(p[1], p[2])}）：{r['text'][:50]}", used, pos or "日", r["ganzhi"]))
    # 3 本纪閏月（闰位）：推步或张表此年之闰
    rl = [e[1] for e in RECK.get(y, []) if e[2]]
    zl = [e[1] for e in ZHANG.get(y, []) if e[2]]
    sh0 = (GI[d["推步朔"]] - GI[d["张表朔"]] + 30) % 60 - 30 if d["张表朔"] in GI else 0
    if rl != zl and (leap or d["异之类"] != "朔日之异" or (tr and zs is None) or abs(sh0) >= 29):
        for b, after, text in LEAPS.get(y, []):
            a = int(after) if after.isdigit() else None
            if a is None:
                continue
            sr, st = a in rl, a in zl
            if sr != st:
                items.append(ev("r" if sr else "t", book_level(b, False), "本纪閏月", b, f"{b}：閏月接{after}月 {text[:40]}", not ding_proper,
                                "闰位"))
    # 4 秦汉初：《汉书》书朔晦与纪日、出土历谱
    if y <= -104:
        for r in QINHAN.get((d["年"], d["月"]), []):
            if r["推步该月朔"] != d["推步朔"]:
                continue
            if r["推步合"] != r["复原合"]:
                g = "C" if r["朔晦"] else "D"
                items.append(ev("r" if r["推步合"] == "1" else "t", g, "汉书本纪（秦汉初）", "hanshu",
                                f"《汉书·{r['卷']}》{r['月']}月{r['干支']}{r['朔晦']}：{r['原文'][:40]}", True, r["朔晦"], r["干支"]))
        for r in QINHAN_SW.get((d["年"], d["月"]), []):
            s = r["朔干支"]
            if s in (d["推步朔"], d["张表朔"]) and r["依据"] != "补":
                items.append(ev("r" if s == d["推步朔"] else "t", "A", "秦汉初出土历谱", r["来源"], f"{r['来源']} {r.get('简号', '')}「{s}」", False))
    # 5 出土、实物、实录
    for r in LOULAN.get((y, m, leap), []):
        sh = r["朔干支"]
        if r["日"].isdigit() and r["干支"] in GI:
            sh = GZ[(GI[r["干支"]] - int(r["日"]) + 1) % 60]
        if sh in (d["推步朔"], d["张表朔"]) and d["推步朔"] != d["张表朔"]:
            items.append(ev("r" if sh == d["推步朔"] else "t", "B", "楼兰简纸", "loulan", f"{r['编号']} {r['原文'][:40]}", False))
        elif r["干支"] in GI:
            okr, okt = judge(a_r, b_r, r["干支"], ""), judge(a_t, b_t, r["干支"], "")
            if okr is not None and okt is not None and okr != okt:
                items.append(ev("r" if okr else "t", "B", "楼兰简纸", "loulan", f"{r['编号']} {r['原文'][:40]}", False))
    for r in TULUFAN.get((y, m, leap), []):
        if r["计入"] != "1":
            continue
        sh = r["朔干支"]
        if not sh and r["日"].isdigit() and r["干支"] in GI:
            sh = GZ[(GI[r["干支"]] - int(r["日"]) + 1) % 60]
        if sh in (d["推步朔"], d["张表朔"]) and d["推步朔"] != d["张表朔"]:
            items.append(ev("r" if sh == d["推步朔"] else "t", "B", "吐鲁番唐西州文书", "tulufan", f"{r['文书名']}：{r['原文'][:40]}", False))
    for f, r in SHIWU.get((y, m, leap), []):
        s = r["朔干支"]
        if s in (d["推步朔"], d["张表朔"]):
            items.append(ev("r" if s == d["推步朔"] else "t", "A", "历书实物", f, f"{f}：{r.get('文件', r.get('来源', ''))[:40]}「{s}」", False))
    for r in CHUTU.get((y, m, leap), []):
        s = r["朔干支"]
        if s in (d["推步朔"], d["张表朔"]):
            items.append(ev("r" if s == d["推步朔"] else "t", "A", "出土具注历", "chutu", r["件"][:40], True))
    jk0 = MSL_JK.get((d["年"], d["月"]))
    for r in MSL.get((y, m, leap), []):
        s = r["朔"]
        if s in (d["推步朔"], d["张表朔"]) and d["推步朔"] != d["张表朔"]:
            if jk0 and "张表误" not in jk0["结论"]:
                notes.append(f"{r['卷']}「{s}朔」：明实录校勘表判「{jk0['结论'][:20]}」，照校勘表不计")
                continue
            items.append(ev("r" if s == d["推步朔"] else "t", "C", "明实录书朔", "mingshilu", f"{r['卷']}：{r['原句'][:30]}", True, "朔", s))
    for r in QSL.get((y, m, leap), []):
        s = r["实录朔"]
        if s in (d["推步朔"], d["张表朔"]) and d["推步朔"] != d["张表朔"]:
            items.append(ev("r" if s == d["推步朔"] else "t", "C", "清实录书朔", "qingshilu", r["原文"][:40], True, "朔", s))
    for f, r in KAO.get((d["年"], d["月"]), []):
        notes.append(f"{f.replace('.csv', '')}（{r.get('证据出处', r.get('出处', ''))[:30]}）：{r['结论'][:90]}")
    jk = MSL_JK.get((d["年"], d["月"]))
    if jk:
        notes.append(f"明实录校勘表（{jk['实录卷']}，校勘记册{jk['校勘记册']}书页{jk['校勘记书页']}）：{jk['结论'][:120]}")
    for r in JOSEON.get((d["年"], d["月"]), []):
        if r["朔干支"] in (d["推步朔"], d["张表朔"]):
            notes.append(f"旁证（异国，不作证据）：朝鲜实录{r['朝鲜年']}「{r['朔干支']}朔」合{'推步' if r['朔干支'] == d['推步朔'] else '张表'}")
    if (d["年"], d["月"]) in EXC:
        e = EXC[(d["年"], d["月"])]
        notes.append(f"例外表：{e['类别']} {e['依据'][:60]}")
    # 差异性质
    shift = (GI[d["推步朔"]] - GI[d["张表朔"]] + 30) % 60 - 30 if d["张表朔"] in GI else None
    if d["异之类"] != "朔日之异" or (shift is not None and abs(shift) >= 29):
        nature = "闰位（月名）"
    elif ding and ding.get("距限（时）", "—") not in ("", "—"):
        try:
            x = abs(float(ding["距限（时）"]))
            nature = f"进朔临界（距限 {x:.2f} 时）" if x < 1 else f"朔差 {shift:+d} 日（距限 {x:.2f} 时）"
        except ValueError:
            nature = f"朔差 {shift:+d} 日"
    elif ding and ding.get("距子夜（时）", "") not in ("", "—"):
        nature = f"朔差 {shift:+d} 日（距子夜 {ding['距子夜（时）']} 时）"
    else:
        nature = f"朔差 {shift:+d} 日" if shift is not None else "—"
    return {"scope": "中原一线", "key": d["历"], "stratum": era_of(y), "year": y, "month": d["月"],
            "tui": d["推步朔"], "zhang": d["张表朔"] if d["张表对应月"] == d["月"] else f"{d['张表朔']}（张表{d['张表对应月']}）",
            "orig": d["分类"], "orig_grade": d["证据等级"], "items": items, "notes": notes, "nature": nature}


# ======== 分政权互异之月 ========
REG_MONTHS = rows(D / "regime_benji_months.csv")
REG_CHECK = rows(D / "regime_benji_check.csv")
RKEY = {"吴": "吴", "蜀": "蜀", "后秦": "后秦", "北凉": "北凉", "北魏": "北魏/西魏/北周/隋", "西魏": "北魏/西魏/北周/隋",
        "北周": "北魏/西魏/北周/隋", "隋": "北魏/西魏/北周/隋", "东魏": "东魏/北齐", "北齐": "东魏/北齐", "辽": "辽", "金": "金",
        "西夏": "西夏", "南明": "南明/南明·绍武/鲁监国/明郑", "明郑": "南明/南明·绍武/鲁监国/明郑", "鲁监国": "南明/南明·绍武/鲁监国/明郑"}
REG_EV = defaultdict(list)
for r in REG_CHECK:
    k = RKEY.get(r["政权"])
    if k and r["推步合"] != r["张表合"] and r["推步月首"] != "—":
        mm = r["月"].split("→")[-1].replace("（考证改月）", "")
        if mm.startswith("闰（推步闰"):
            mm = "闰" + mm[len("闰（推步闰"):].split("，")[0]
        REG_EV[(k, r["年"], r["推步月首"], mm)].append(r)
RKAO = {}
for f in ["beichao_regime_kaozheng.csv", "liaojin_regime_kaozheng.csv", "nanming_xixia_regime_kaozheng.csv"]:
    for r in rows(D / f):
        b = r["书"] if r["书"].endswith(("_benji",)) or r["书"].startswith(("nanming", "jiankang", "sanguozhi")) else r["书"] + "_benji"
        RKAO[(b, r["年"], r["月"], r["干支"])] = (f, r.get("处置", r.get("类", "")), r.get("考证", r.get("判", "")))
ZOUMA = defaultdict(list)
for r in rows(D / "zoumalou_jiri.csv"):
    if r.get("核对") == "1" and r["年"].isdigit() and r["月"].isdigit():
        ZOUMA[(int(r["年"]), int(r["月"]), r["闰"] == "1")].append(r)
CHUTU_JR = defaultdict(list)
for r in rows(D / "出土纪日.csv"):
    if r["lunar_year"].isdigit():
        CHUTU_JR[(r["regime"], int(r["lunar_year"]), int(r["month"]), r["is_leap"] == "1")].append(r)
LIAO_SK = defaultdict(list)
for r in rows(D / "liao_shike_shuo.csv"):
    mm = r["月"].replace("月", "")
    if r["年"].isdigit() and mm.lstrip("閏闰").isdigit():
        LIAO_SK[(int(r["年"]), mm.replace("閏", "闰"))].append(r)
MZ = defaultdict(list)
for r in rows(D / "mingzheng_shiwu_months.csv"):
    if r["年"].isdigit():
        MZ[(int(r["年"]), label(int(r["月"]), r["闰"] == "1"))].append(r)
# 天文志之条：分政权检验现已读入（regime_benji_eval 逐条写 regime_zhi_check.csv，推步含史载修正）
ZHI_EV = defaultdict(list)
for r in rows(D / "regime_zhi_check.csv"):
    k = RKEY.get(r["政权"])
    if k and r["推步合"] != r["张表合"] and r["推步月首"] != "—":
        mm = r["月"]
        if mm.startswith("闰（推步"):
            mm = "闰" + mm[len("闰（推步"):].split("，")[0].lstrip("闰")
        ZHI_EV[(k, r["年"], r["推步月首"], mm)].append(r)

REG_STRATUM = {"吴": "三国吴", "蜀": "三国蜀", "北魏/西魏/北周/隋": "北魏西魏北周隋", "东魏/北齐": "东魏北齐", "辽": "辽", "金": "金",
               "南明/南明·绍武/鲁监国/明郑": "南明明郑"}


def reg_audit(d):
    k, y = d["推步之键"], int(d["年"])
    m, leap = parse_label(d["月"])
    zm = zhang_month(y, m, leap)
    zg = gz_of(zm[0]) if zm else "—"
    items, notes = [], []
    seen = set()
    for r in REG_EV.get((k, d["年"], d["推步月首"], d["月"]), []):
        b = r["书"]
        pos = r["位置"]
        g = book_level(b, pos == "朔")
        dup = (b, r["干支"], r["原文"][:12]) in seen
        seen.add((b, r["干支"], r["原文"][:12]))
        it = ev("r" if r["推步合"] == "1" else "t", g, "史书纪日" if pos != "朔" else "史书书朔", b,
                f"{b}：{r['月']}月{r['干支']}{pos} {r['原文'][:40]}", True, pos or "日", r["干支"])
        it["dup"] = dup
        if b in LATER_BOOKS:
            it["later"] = LATER_BOOKS[b]
        kz = RKAO.get((b, r["年"], r["月"].split("→")[0], r["干支"]))
        if kz:
            notes.append(f"{kz[0].replace('.csv', '')}：{kz[1]} {kz[2][:70]}")
        items.append(it)
    # 前月之条：前月两家同为「相同」之月而月终不同（本月月首之异），原检验将其归前月之键，互异之月未收
    for r in REG_ADJ.get((k, d["年"], d["推步月首"], d["月"]), []):
        b = r["书"]
        pos = r["位置"]
        it = ev("r" if r["推步合"] == "1" else "t", book_level(b, pos == "朔"), "史书纪日（前月）", b,
                f"{b}（前月{r['月']}）：{r['干支']}{pos} {r['原文'][:40]}", False, pos or "日", r["干支"])
        if b in LATER_BOOKS:
            it["later"] = LATER_BOOKS[b]
        items.append(it)
    tui = d["推步月首"]
    # 走马楼吴简、吴买地券（B）
    if k == "吴":
        for r in ZOUMA.get((y, m, leap), []):
            s = r["朔干支"]
            if s in (tui, zg) and tui != zg:
                items.append(ev("r" if s == tui else "t", "B", "走马楼吴简", "zoumalou",
                                f"{r['册']} {r['简号'] or r['书页']}：{r['原文'][:36]}", True))
        for r in CHUTU_JR.get(("吴", y, m, leap), []):
            s = r["shuo_ganzhi"] or (GZ[(GI[r["day_ganzhi"]] - int(r["day"]) + 1) % 60] if r["day"].isdigit() else "")
            if s in (tui, zg) and tui != zg:
                it = ev("r" if s == tui else "t", "B", "吴买地券等", "maidiquan", f"{r['object']}：{r['text']}（{r['checked']}）", False)
                it["unverified"] = "未核" in r["checked"]
                items.append(it)
    if k == "辽":
        for r in LIAO_SK.get((y, d["月"]), []):
            s = r["石刻朔"].split("(")[0]
            if s in (tui, zg) and tui != zg:
                items.append(ev("r" if s == tui else "t", "B", "辽代石刻", "liao_shike", f"{r['书']} {r['书页']}：{r['原文'][:36]}", True))
    if k.startswith("南明"):
        for r in MZ.get((y, d["月"]), []):
            s = r["朔干支"]
            if s in (tui, zg) and tui != zg:
                items.append(ev("r" if s == tui else "t", "A", "明郑大统历实物", "mingzheng", r["出处"][:40], True))
    for r in ZHI_EV.get((k, d["年"], d["推步月首"], d["月"]), []):
        items.append(ev("r" if r["推步合"] == "1" else "t", book_level(r["书"], r["位置"] == "朔"), "天文志", r["书"],
                        f"{r['书']}：{r['月']}月{r['干支']}{r['位置']} {r['原文'][:40]}", True, r["位置"] or "日", r["干支"]))
    shift = (GI[tui] - GI[zg] + 30) % 60 - 30 if zg in GI else None
    rl = sorted(x for x in REG_MONTHS_LEAP.get((k, y), []))
    zl = [e[1] for e in ZHANG.get(y, []) if e[2]]
    if zg == "—" or (shift is not None and abs(shift) >= 29) or (rl != zl and (leap or zm is None)):
        nature = f"闰位（推步闰{','.join(map(str, rl)) or '无'}，张表闰{','.join(map(str, zl)) or '无'}）"
    else:
        nature = f"朔差 {shift:+d} 日" + (f"（闰位亦异：推步闰{','.join(map(str, rl)) or '无'}，张表闰{','.join(map(str, zl)) or '无'}）" if rl != zl else "")
    return {"scope": "分政权", "key": k, "stratum": REG_STRATUM.get(k, k), "year": y, "month": d["月"], "tui": tui, "zhang": zg,
            "orig": d["分类"], "orig_grade": d["证据等级"], "items": items, "notes": notes, "nature": nature}


# 前月之条归入本月：逐政权推步月序，相同之月的只合一家之条归其后一月（若为互异之月）
REG_ADJ = defaultdict(list)
_order = defaultdict(list)
for r in REG_MONTHS:
    _order[r["推步之键"]].append(r)
_pos = {}
for k, v in _order.items():
    for i, r in enumerate(v):
        _pos[(k, r["年"], r["推步月首"], r["月"])] = (v, i)
for key, es in REG_EV.items():
    if key not in _pos:
        continue
    v, i = _pos[key]
    if v[i]["异同"] == "相同" and i + 1 < len(v) and v[i + 1]["异同"] == "互异":
        n = v[i + 1]
        REG_ADJ[(key[0], n["年"], n["推步月首"], n["月"])].extend(es)

REG_MONTHS_LEAP = defaultdict(list)
for r in REG_MONTHS:
    if r["月"].startswith("闰"):
        REG_MONTHS_LEAP[(r["推步之键"], int(r["年"]))].append(int(r["月"][1:]))


# ---------------------------------------------------------------- 重判
def reclassify(a):
    """按 shiwu_fenlei 门槛重判：A/B/C 一条；D、E 须独立两条（史源合并、同书同月同日同事只算一条）。"""
    its = [dict(i, grade="E") if i.get("later") else i for i in a["items"]]
    hard = [i for i in its if i["grade"] in "ABC"]
    hr = [i for i in hard if i["side"] == "r"]
    ht = [i for i in hard if i["side"] == "t"]
    if hr and ht:
        return "③（证据冲突）", min(i["grade"] for i in hard)
    if hr or ht:
        g = min(i["grade"] for i in hard)
        if not [i for i in its if i["side"] != ("r" if hr else "t")]:
            return ("①" if hr else "②"), g
        return "③（证据冲突）", g
    soft = [i for i in its if i["grade"] in "DE"]
    sides = {i["side"] for i in soft}
    if len(sides) == 2:
        return "③（证据冲突）", "—"
    if not soft:
        return "③", "—"
    # 独立：每史源（lineage）一组；同一史源内同日（同干支）只算一条
    lins = {i["lin"] for i in soft}
    events = {(i["lin"], i["event"] or i["text"][:12]) for i in soft if not i.get("dup")}
    side = soft[0]["side"]
    if len(lins) >= 2:
        return ("①" if side == "r" else "②"), min(i["grade"] for i in soft)
    if len(events) >= 2:
        return "③（只一源）", "—"  # 甲：同一史源（同书同月）不同日之事只算一源，不足两条
    return "③（一见）", "—"


def conclude(a):
    rc, g = reclassify(a)
    orig = a["orig"]
    a["recl"], a["recl_grade"] = rc, g
    unused_hard = [i for i in a["items"] if not i["used"] and i["grade"] in "ABC" and not i.get("later")]
    if orig.startswith("③"):
        if rc in ("①", "②") and any(i.get("unverified") for i in a["items"]):
            return "无法判断", f"倾向{'推步' if rc == '①' else '张表'}：所据为转录未核之条，须核图版"
        if rc in ("①", "②"):
            src = "；".join(sorted({i["src"] for i in a["items"] if i["side"] == ("r" if rc == "①" else "t")}))
            return f"应改{rc}", f"{g} 级（{src}）{'原检验未接入' if unused_hard or any(not i['used'] for i in a['items']) else '原检验已有'}"
        if rc.startswith("③（证据冲突）"):
            return "判定得当", "证据两向"
        if rc == "③（一见）":
            return "判定得当", "纪日一见，等级不足"
        if rc == "③（只一源）":
            n = len({(i["event"] or i["text"][:12]) for i in a["items"] if not i.get("dup")})
            return "判定得当", f"同一史源同月 {n} 事，按甲只算一源，不足异源两条"
        return "判定得当", "无判别之据"
    return "（非③）", ""


# ---------------------------------------------------------------- 人工复核（逐条看过原文与考证表之月；键：范围、历/政权、年、月）
MANUAL = {
    ("中原一线", "麟德历", 697, "11"): (
        "判定得当",
        "《新唐书·则天纪》圣历元年正月甲子改元、丙寅、丁亥三条（武周建子，正月＝表 697 年十一月）只容于张表十一月甲子朔之月，"
        "推步十一月癸巳朔之月不容（丁亥距癸巳 54 日）；同书「閏月甲寅」接神功元年十月，与张表闰十同、与推步闰十二不同。"
        "四条同出《新唐书》（与《旧唐书》同源），按甲只算一源，不足异源两条，③得当（首轮按旧规则判应改②）。"
        "中原一线检验现已按月名收此月纪日与「閏月」"),
    ("中原一线", "大统历", 1495, "7"): (
        "判定得当",
        "《孝宗实录》卷一百二「弘治八年七月辛巳朔」（C）合张表；项目明实录校勘表判「存疑，从推步」（实录有按讹朔整月排日之例，"
        "1622 年六月实物乙丑而实录整月按丙寅），照校勘表结论不计（2026-11-27 定），③得当（首轮判应改②）"),
    ("中原一线", "戊寅元历", 642, "4"): (
        "判定得当",
        "吐鲁番唐西州墓表三方：怀保妻左氏、张隆悦妻文姿二方「四月乙酉朔」合推步，张谦祐妻殿氏一方「四月丙戌朔」合张表；"
        "B 级两向，冲突归③。推步定朔距子夜 0.31 时，西州或有一日之差。中原一线检验未接 tulufan_jiri（唐 34 条），接入后仍③"),
    ("分政权", "南明/南明·绍武/鲁监国/明郑", 1650, "12"): (
        "判定得当",
        "唯一之条出《小腆纪年》（清徐鼒撰，成书去南明二百年），检验程序现按 E 计（乙），只一见，③得当（首轮判等级标错）"),
    ("分政权", "北魏/西魏/北周/隋", 494, "4"): (
        "判定得当",
        "《魏书·天象志》太和十八年「四月庚申，月在斗蝕」「四月甲寅，熒惑入軒轅」二事，推步四月乙巳朔之月内（第 16、10 日），"
        "张表四月乙亥朔之月外（张表闰四月乙巳）；分政权检验现已读天象志，但二事同出《魏书》，按甲只算一源，③得当（首轮按旧规则判应改①）"),
    ("分政权", "北魏/西魏/北周/隋", 513, "3"): (
        "判定得当",
        "《魏书·世宗纪》延昌二年「三月丙寅，高昌國遣使朝獻」与《天象志》同年「三月乙丑，填星守房」「三月辛酉，熒惑又犯太微」三事，"
        "皆在推步三月乙卯朔之月内、张表三月乙酉朔之月外；本纪与天象志同书同源，按甲只算一源，③得当（首轮判应改①）"),
}

# 首轮（c04239d）全扫「应改」之 20 月：按新产物复核是否已改过来
PREV_FIX = [
    ("中原一线", "麟德历", 697, "11", "应改②"), ("中原一线", "大统历", 1495, "7", "应改②"), ("中原一线", "时宪历", 1708, "2", "应改②"),
    ("分政权", "北魏/西魏/北周/隋", 494, "4", "应改①"), ("分政权", "北魏/西魏/北周/隋", 513, "3", "应改①"),
    ("分政权", "南明/南明·绍武/鲁监国/明郑", 1650, "12", "等级标错"), ("分政权", "南明/南明·绍武/鲁监国/明郑", 1671, "1", "应改①"),
    ("分政权", "吴", 227, "10", "应改①"), ("分政权", "吴", 231, "7", "应改②"), ("分政权", "吴", 232, "4", "应改①"),
    ("分政权", "吴", 232, "6", "应改①"), ("分政权", "吴", 232, "8", "应改①"), ("分政权", "吴", 232, "12", "应改①"),
    ("分政权", "吴", 233, "3", "应改②"), ("分政权", "吴", 233, "5", "应改①"), ("分政权", "吴", 233, "10", "应改①"),
    ("分政权", "吴", 234, "2", "应改①"), ("分政权", "吴", 234, "7", "应改①"), ("分政权", "吴", 234, "11", "应改①"),
    ("分政权", "吴", 236, "3", "无法判断"), ("分政权", "辽", 994, "4", "应改①"),
]


def main():
    head = subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()
    inputs = ["zhongyuan_reckoning_months.csv", "zhongyuan_reckoning_diffs.csv", "dingshuo_zhangbiao_diffs.csv", "regime_benji_months.csv",
              "regime_benji_check.csv", "benji_shuo_check.csv", "discriminating_records.csv", "beichao_regime_kaozheng.csv",
              "liaojin_regime_kaozheng.csv", "tulufan_jiri.csv", "zoumalou_jiri.csv", "liao_shike_shuo.csv", "mingshilu_shuo.csv",
              "qingshilu_shuo_xinfa.csv", "dingshuo_exceptions.csv", "tang_shuo_kaozheng.csv", "song_shuo_kaozheng.csv"]
    dirty = subprocess.run(["git", "status", "--porcelain", "--"] + [str(D / f) for f in inputs], capture_output=True, text=True).stdout
    dirty = [l[3:].split("/")[-1] for l in dirty.splitlines()]
    if dirty:
        head += "（工作区另有未提交改动：" + "、".join(dirty) + "）"
    zy = [zy_audit(d) for d in ZY_DIFF]
    rg = [reg_audit(d) for d in REG_MONTHS if d["异同"] == "互异"]
    for a in zy + rg:
        a["concl"], a["basis"] = conclude(a)
        mk = (a["scope"], a["key"], a["year"], a["month"])
        if mk in MANUAL:
            a["concl"], a["basis"], a["manual"] = MANUAL[mk][0], MANUAL[mk][1], True
    zy3 = [a for a in zy if a["orig"].startswith("③")]
    rg3 = [a for a in rg if a["orig"].startswith("③")]
    # 抽样：分层（中原一线按朝代，分政权按推步之键），最大余数法分配，每层至少 1
    rnd = random.Random(SEED)

    def strat_sample(pop, n):
        strata = defaultdict(list)
        for a in pop:
            strata[a["stratum"]].append(a)
        tot = len(pop)
        quota = {s: max(1, n * len(v) // tot) for s, v in strata.items()}
        rem = sorted(strata, key=lambda s: -(n * len(strata[s]) / tot - n * len(strata[s]) // tot))
        i = 0
        while sum(quota.values()) < n:
            quota[rem[i % len(rem)]] += 1
            i += 1
        while sum(quota.values()) > n:
            s = max(quota, key=lambda s: quota[s])
            quota[s] -= 1
        out = []
        for s in sorted(strata):
            v = sorted(strata[s], key=lambda a: (a["key"], a["year"], a["month"]))
            out.extend(rnd.sample(v, min(quota[s], len(v))))
        return out, {s: (len(strata[s]), quota[s]) for s in strata}

    zs, zq = strat_sample(zy3, N_SAMPLE)
    rs, rq = strat_sample(rg3, N_SAMPLE)
    for a in zs + rs:
        a["sampled"] = True
    risky = [a for a in zy3 + rg3 if len(a["items"]) >= 2]
    for a in risky:
        a["risky"] = True
    audit = [a for a in zy3 + rg3 if a.get("sampled") or a.get("risky") or a["concl"] != "判定得当"]
    audit.sort(key=lambda a: (a["scope"] != "中原一线", a["key"], a["year"], a["month"]))

    def ev_txt(a):
        return "；".join(
            f"{'推' if i['side'] == 'r' else '张'}{i['grade']}{'' if i['used'] else '*'}{'(重)' if i.get('dup') else ''}:{i['text']}"
            for i in a["items"]).replace(",", "，")

    out = io.StringIO()
    out.write(f"# 互异而归 ③ 之判抽查（scripts/calendar_data/d_level_audit.py 生成，勿手改）；随机种子 {SEED}，各范围分层抽 {N_SAMPLE} 月；"
              f"所据文件 git HEAD {head}（data/calendar/zhongyuan_reckoning_months.csv、zhongyuan_reckoning_diffs.csv、"
              f"dingshuo_zhangbiao_diffs.csv、regime_benji_months.csv、regime_benji_check.csv 及文中所列各表）。\n")
    out.write("# 入选：抽样=分层随机所抽；多证=同月有 ≥2 条证据；重判异=全体扫描中重判结论与原判不同。证据：推/张=支持推步/张表，"
              "字母=等级，*=原检验未用，(重)=同书同干支同文重出。结论人工复核者「人工」列为 1。\n")
    w = csv.writer(out, lineterminator="\n")
    w.writerow(["范围", "历或政权", "层", "年", "月", "推步朔", "张表朔", "差异性质", "证据", "原判", "原判等级", "重判", "重判等级",
                "抽查结论", "依据", "按语", "入选", "人工"])
    for a in audit:
        sel = "+".join(x for x, f in (("抽样", a.get("sampled")), ("多证", a.get("risky")), ("重判异", a["concl"] != "判定得当")) if f)
        w.writerow([a["scope"], a["key"], a["stratum"], a["year"], a["month"], a["tui"], a["zhang"], a["nature"], ev_txt(a), a["orig"],
                    a["orig_grade"], a["recl"], a["recl_grade"], a["concl"], a["basis"].replace(",", "，"),
                    "；".join(a["notes"]).replace(",", "，").replace("\n", " ")[:400], sel, int(bool(a.get("manual")))])
    (D / "d_level_audit.csv").write_text(out.getvalue(), encoding="utf-8")
    # 首轮应改之月复核：现分类取新产物逐月表
    cur = {("中原一线", d["历"], int(d["年"]), d["月"]): (d["分类"], d["证据等级"], d["证据"]) for d in ZY_DIFF}
    cur.update({("分政权", d["推步之键"], int(d["年"]), d["月"]): (d["分类"], d["证据等级"], d["注"]) for d in REG_MONTHS if d["异同"] == "互异"})
    by = {(a["scope"], a["key"], a["year"], a["month"]): a for a in zy + rg}
    prev = []
    for sc, k, y, m, was in PREV_FIX:
        # 中原一线逐月表以历名为键，此处按年月找
        kk = next((x for x in cur if x[0] == sc and x[2] == y and x[3] == m and (sc == "中原一线" or x[1] == k)), None)
        c, g, e = cur.get(kk, ("—", "—", ""))
        a = by.get(kk)
        if c[0] in "①②":
            why = f"已改：{g} 级——{e[:120]}"
        else:
            why = f"仍③：{a['basis'] if a else ''}"
        prev.append((sc, k, y, m, was, c, why))
    write_doc(head, zy, rg, zy3, rg3, zs, rs, zq, rq, risky, audit, prev)


CONCLS = ["判定得当", "应改①", "应改②", "应改为相同类", "等级标错", "无法判断"]


def pct(n, d):
    return f"{n}（{100 * n / d:.1f}%）" if d else "0"


def write_doc(head, zy, rg, zy3, rg3, zs, rs, zq, rq, risky, audit, prev):
    L = []
    A = L.append
    A("# 互异而归 ③ 之判抽查（自动生成）")
    A("")
    A("> 由 `scripts/calendar_data/d_level_audit.py` 生成，勿手改；逐月见 `data/calendar/d_level_audit.csv`。只读分析：不改检验程序与产物。"
      "首轮（c04239d）所列系统性问题已于 2026-11-27 在检验程序中修正（见「系统性问题之处理」），本文为按新产物重判之第二轮，"
      "首轮「应改」之月逐月复核见「首轮应改之月复核」。")
    A("")
    A("## 方法")
    A("")
    A(f"- **范围**：中原一线推步与张表互异之月 {len(zy)}（`zhongyuan_reckoning_months.csv`，原判 ①② {len(zy) - len(zy3)}、③ {len(zy3)}）；"
      f"分政权互异之月 {len(rg)}（`regime_benji_months.csv`，原判 ①② {len(rg) - len(rg3)}、③ {len(rg3)}）。所据文件 git HEAD `{head}`。")
    A("- **等级**（与 `scripts/calendar_data/shiwu_fenlei.py`、`rust/core/examples/zhongyuan_reckoning_eval.rs`、`regime_benji_eval.rs` 一致）："
      "A 实物历日，B 当时文字（碑志、墓志、简牍、文书），C 当朝或近时所修史书明书朔，D 当朝史书纪日（同月独立两条才判），"
      "E 后人追修史书（《后汉书》《晋书》，《南史》《北史》《新五代史》前史无同条者；南明诸书即清人私家追记与《建康实录》吴部分亦为 E）。"
      "今人表不是证据。")
    A("- **重新汇集证据**：每个互异之月逐条重收（「*」为检验程序未用者）：中原一线——定朔考证之判与实物（取原检验）、本纪明书朔、本纪纪日"
      "（定朔考证按月名配之月以考证所收为准，余者检验程序按月名收）、本纪前月末日之条（*）、本纪「閏月」、《汉书》秦汉初书朔晦"
      "（`qinhan_benji_check.csv`）、秦汉初出土历谱（`qinhan_shiwu_months.csv`，*）、楼兰简纸（*）、吐鲁番唐西州文书（`tulufan_jiri.csv`，*）、"
      "大统历与时宪书实物（*）、明实录书朔（`mingshilu_shuo.csv`，校勘表判讹或存疑者照其结论不计）、清实录书朔（`qingshilu_shuo_xinfa.csv`）；"
      "分政权——史书本纪纪日、走马楼吴简（`zoumalou_jiri.csv`，核对过的）、出土纪日（`出土纪日.csv`，*）、辽代石刻（`liao_shike_shuo.csv`）、"
      "明郑大统历实物（`mingzheng_shiwu_months.csv`）、北朝诸史与《金史》天文志之条（`regime_zhi_check.csv`）、前月末日之条（*）。"
      "唐宋诸考证表（`tang_shuo_kaozheng.csv`、`song_shuo_kaozheng.csv`、"
      "`tangsong_cunyi_kaozheng.csv`、`wudai_shuo_kaozheng.csv`、`yuan_shuo_kaozheng.csv`）、例外表、各政权考证表、明实录校勘表与朝鲜实录"
      "（异国，只作旁证）的结论只记入按语。")
    A("- **重判**：A、B、C 一条即可判（两向即冲突归③）；D、E 须异源两条（甲）。同一史源（旧、新唐书；旧、新五代史；《北史》与魏、北齐、周、隋书；"
      "《南史》与南朝四书；《晋书》与《三国志》；《建康实录》吴部分与《三国志·吴书》；同书本纪与志）合为一源，同一史源同月诸事只算一源。"
      "成书远在其后的私家追记（《小腆纪年》等）按 E 计（乙）。")
    A(f"- **抽样**：随机种子 `{SEED}`（Python `random.Random`），中原一线按朝代分层、分政权按推步之键分层，最大余数法按层大小分配 {N_SAMPLE} 月，"
      "每层至少 1 月，层内按（历/政权、年、月）排序后 `sample`。另把「同月有 ≥2 条证据却仍归 ③」之月、全体扫描中重判与原判不同之月全部列入。")
    A("")
    A("| 范围 | 层 | ③ 月数 | 抽 |")
    A("|---|---|---|---|")
    for sc, q in (("中原一线", zq), ("分政权", rq)):
        for st in sorted(q):
            A(f"| {sc} | {st} | {q[st][0]} | {q[st][1]} |")
    A("")
    A("## 汇总")
    A("")
    A("### 抽样所得")
    A("")
    A("| 结论 | 中原一线（抽 " + str(len(zs)) + "） | 分政权（抽 " + str(len(rs)) + "） |")
    A("|---|---|---|")
    for c in CONCLS:
        A(f"| {c} | {pct(sum(a['concl'] == c for a in zs), len(zs))} | {pct(sum(a['concl'] == c for a in rs), len(rs))} |")
    A("")
    # 分层加权推断
    def est(sample, q, pop):
        out = {}
        for c in CONCLS:
            e = 0.0
            for st, (n, k) in q.items():
                ss = [a for a in sample if a["stratum"] == st]
                if ss:
                    e += n * sum(a["concl"] == c for a in ss) / len(ss)
            out[c] = e
        return out
    ez, er = est(zs, zq, zy3), est(rs, rq, rg3)
    A("按层加权推断全体（层月数 × 层内抽中比例）与全体逐月扫描（同一套证据汇集与重判规则跑全部 ③ 月；人工复核之月按复核结论）对照：")
    A("")
    A("| 结论 | 中原一线：推断 | 中原一线：全扫 | 分政权：推断 | 分政权：全扫 |")
    A("|---|---|---|---|---|")
    for c in CONCLS:
        A(f"| {c} | {ez[c]:.1f} | {sum(a['concl'] == c for a in zy3)} | {er[c]:.1f} | {sum(a['concl'] == c for a in rg3)} |")
    A("")
    nz = sum(a["concl"] in ("应改①", "应改②", "应改为相同类") for a in zy3)
    nr = sum(a["concl"] in ("应改①", "应改②", "应改为相同类") for a in rg3)
    A(f"**判断**：抽样中绝大多数 ③ 月本无任何能判别两家之条（两家只差一日，判别之日只有月首一天），③ 判得当。按新产物全扫："
      f"中原一线 ③ {len(zy3)} 月中应改 {nz} 月，分政权 ③ {len(rg3)} 月中应改 {nr} 月。首轮应改之月多因检验程序未接入已有的高等级材料，"
      "现已接入（见「首轮应改之月复核」「系统性问题之处理」）。")
    A("")
    A("### 应改之月（全扫）")
    A("")
    A("| 范围 | 历/政权 | 年 | 月 | 推步朔 | 张表朔 | 结论 | 依据 |")
    A("|---|---|---|---|---|---|---|---|")
    for a in zy3 + rg3:
        if a["concl"] != "判定得当":
            A(f"| {a['scope']} | {a['key']} | {a['year']} | {a['month']} | {a['tui']} | {a['zhang']} | {a['concl']} | {a['basis']} |")
    A("")
    A("## 多证之月")
    A("")
    A(f"同月有 ≥2 条证据却仍归 ③ 者 {len(risky)} 月（全部列入抽查）。")
    A("")
    A("| 范围 | 历/政权 | 年 | 月 | 证据 | 原判 | 抽查结论 | 依据 |")
    A("|---|---|---|---|---|---|---|---|")
    for a in risky:
        evs = "；".join(f"{'推' if i['side'] == 'r' else '张'}{i['grade']}{'' if i['used'] else '*'}：{i['text'][:30]}" for i in a["items"])
        A(f"| {a['scope']} | {a['key']} | {a['year']} | {a['month']} | {evs.replace('|', '｜')} | {a['orig']} | {a['concl']} | {a['basis']} |")
    A("")
    A("多数是《北史》与魏、北齐、周书同记一事（同一史源，只算一条），合规则；489 年以后隋书与北史亦然。")
    A("")
    A("## 首轮应改之月复核")
    A("")
    A("首轮（c04239d）全扫判「应改」「等级标错」「无法判断」之月，按新产物逐一复核：")
    A("")
    A("| 范围 | 历/政权 | 年 | 月 | 首轮结论 | 现分类 | 说明 |")
    A("|---|---|---|---|---|---|---|")
    for sc, k, y, m, was, c, why in prev:
        A(f"| {sc} | {k} | {y} | {m} | {was} | {c} | {why.replace('|', '｜')} |")
    A("")
    A(f"已改 {sum(c[0] in '①②' for *_, c, _ in prev)} 月，仍③ {sum(c[0] == '③' for *_, c, _ in prev)} 月；仍③者皆因甲（同书同月诸事只算一源）、"
      "照明实录校勘表「存疑」不计，或乙（南明私家追记按 E），或所据为转录未核之条，均非漏接材料。")
    A("")
    A("## 系统性问题之处理（2026-11-27）")
    A("")
    o12 = [a for a in rg if not a["orig"].startswith("③")]
    d1 = [a for a in o12 if a["orig"].startswith("①") and a["orig_grade"] == "D"]
    d1_one = [a for a in d1 if len({i["lin"] for i in a["items"] if i["used"] and i["grade"] in "DE"}) == 1
              and not any(i["grade"] in "ABC" for i in a["items"])]
    A("首轮所列七项，处理如下（各项改判月数见 `doc/calendar/regime-benji-checks.md`、`benji-checks.md` 与本次报告）：")
    A("")
    A("1. **分政权接入实物与当时文字**：`regime_benji_eval.rs` 读走马楼吴简（逐月所据之简取 `zoumalou_shuo_check.py` 写的 "
      "`zoumalou_months.csv`，与 zoumalou-checks 同一分类）、辽代石刻（`liao_shike_shuo.csv`，含由日序倒推之朔）、明郑大统历实物"
      "（`mingzheng_shiwu_months.csv`）；A、B 一条即判，A 级压过其余诸证。")
    A("2. **中原一线接入明清实录书朔**：`dingshuo_zhangbiao_diffs.rs`（619 年后朔日之异）与 `zhongyuan_reckoning_eval.rs`（其余互异之月）"
      "读 `mingshilu_shuo.csv`、`qingshilu_shuo_xinfa.csv` 作 C 级；明实录校勘表判馆本讹、诸本同讹、录文抽取误或存疑者照其结论不计"
      "（1495 年七月因此仍③）；校勘表判「张表误」而录文书朔表未收之 1378 年八月以校勘表所录馆本朔补入。")
    A("3. **定朔考证不及之月收纪日、「閏月」、《汉书》秦汉初书朔晦**：619 年前诸月与 619 年后定朔考证按次序配月（张表月名不同）之月，"
      "中原一线检验按月名收本纪纪日与「閏月」；秦汉初《汉书》书朔、晦为 C，余为 D。")
    A(f"4. **门槛**：甲——D、E 级判①②须异源两条（同书同月诸事只算一源），中原一线、定朔考证、分政权三处一致；"
      f"乙——`nanming_*`、`jiankang_shilu_wu` 为 E。现分政权①中 D 级 {len(d1)} 月，只一史源者 {len(d1_one)} 月"
      "（应为 0：有则是 A、B、C 级他证并存）。")
    A("5. **E 级一条**：中原一线与定朔考证亦改为 E 级明书朔须两条（与分政权同）。")
    A("6. **「异之类」**：两家同名之月而朔差约一月者（719、852 年八月）改标「月名之异（闰位）」。")
    A("7. **天文志之条**：北朝诸史（《魏书》《隋书》）与《金史》天文志之凌犯、日月食纪日接入分政权检验（书名 `<书>_zhi`，与同书本纪同源；"
      "逐条见 `regime_zhi_check.csv`）。494 年四月、513 年三月之条同出《魏书》，按甲只算一源，仍③。")
    A("")
    A("## 逐月全表")
    A("")
    A("入选：抽样／多证／重判异。证据：推、张=支持推步、张表，字母为等级，* 为原检验未用。")
    A("")
    A("| 范围 | 历/政权 | 年 | 月 | 推步朔 | 张表朔 | 差异性质 | 证据 | 原判 | 抽查结论 | 依据 | 入选 |")
    A("|---|---|---|---|---|---|---|---|---|---|---|---|")
    for a in audit:
        sel = "+".join(x for x, f in (("抽样", a.get("sampled")), ("多证", a.get("risky")), ("重判异", a["concl"] != "判定得当")) if f)
        evs = "；".join(f"{'推' if i['side'] == 'r' else '张'}{i['grade']}{'' if i['used'] else '*'}：{i['text'][:24]}" for i in a["items"]) or "—"
        A(f"| {a['scope']} | {a['key']} | {a['year']} | {a['month']} | {a['tui']} | {a['zhang']} | {a['nature']} | {evs.replace('|', '｜')} | "
          f"{a['orig']} | {a['concl']} | {a['basis'][:160]} | {sel} |")
    A("")
    A("## 限制")
    A("")
    A("- 证据只来自仓库已整理的表；没有回到点校本、拓片或简牍图版逐条重读（人工复核的月份只核了仓库中的录文与考证表）。"
      "因此「判定得当」的意思是：仓库现有材料中没有能判别两家的条目，不是说史料中一定没有。")
    A("- 汉简历日（`hanjian_*`）此次未读（另一检验正在用），秦汉初只用了《汉书》本纪与 `qinhan_shiwu_months.csv`。")
    A("- 吐鲁番唐西州文书只取「计入」=1 之条；走马楼只取核对过的；辽代石刻含「(推)」（由日序与日干支倒推之朔）之条，与检验程序同；"
      "出土纪日表中「转录未核」者判「无法判断」。")
    A("- 中原一线推步各月的 JDN 由逐月表干支以张表同名月为锚还原（差不过 ±30 日），闰位之异处以前月 +30 日为锚。")
    A("- 前月末日之条只对「前月两家同朔」的情形归入本月；连续多月互异时由各月自身之条判。")
    A("")
    Path("doc/calendar/d-level-audit.md").write_text("\n".join(L), encoding="utf-8")


if __name__ == "__main__":
    import sys

    sys.path.insert(0, str(Path(__file__).parent))
    main()
