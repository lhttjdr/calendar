"""中原一线推步与张表相同、而所据证据与两家皆不合之月（「同误」）逐条查成因。

输入（只读）：data/calendar/zhongyuan_reckoning_months.csv（由 rust/core/examples/zhongyuan_reckoning_eval.rs 生成）中分类为「同误」之月；
证据按同一程序之规则重取——本纪明书朔（data/calendar/benji_shuo_check.csv）与本纪纪日（data/calendar/本纪纪日/*_benji.csv，只取能分辨
朔日之条：不在两家该月内、书日序或晦而日序不合者）；已有考证：benji_shuo_errata.csv（已考讹误）、benji_qita_kaozheng_notes.csv
（「其他」类逐条判语）、tang_shuo_kaozheng.csv 等（只作按语引用）。实际合朔用 skyfield + DE406（vendor/jpl/de406/de406.bsp）、
都城地方平时（data/phenomena/capitals.csv），同 scripts/phenomena/benji_shuo_sky.py。旁参《资治通鉴目录》（刘羲叟《长历》）朔闰
（data/calendar/tongjian_mulu_shuo.csv，宋人复原表，只作旁参，不作证据）。

输出：data/calendar/tongwu_causes.csv（逐条：一行一条证据）与 doc/calendar/tongwu-causes.md（方法、汇总、逐条表、候选改推步清单）。
本脚本只读共享产物，不改推步、不重跑生成器。

用法（仓库根）：uv run --project scripts --with skyfield python scripts/calendar_data/tongwu_causes.py [逐月表路径]
逐月表缺省为 data/calendar/zhongyuan_reckoning_months.csv；他程序正在改写时可传快照路径。所据之表的 sha256 写入两份产物。
另读维基文库原文缓存 scripts/sky_culture/_cache/（qinhan_eval/houhanshu_star_records.py 所取）以判纪日抽取误与取校勘记。
"""
from __future__ import annotations

import csv
import os
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/calendar_data"))
import gaiyuan_date as gd  # noqa: E402

GZ = gd.GANZHI
OUT_CSV = ROOT / "data/calendar/tongwu_causes.csv"
OUT_MD = ROOT / "doc/calendar/tongwu-causes.md"


def gzi(g: str) -> int:
    return GZ.index(g)


def gz(j: int) -> str:
    return gd.ganzhi_of_jdn(j)


def wrap(d: int) -> int:
    """干支差 → [-30, 30)。"""
    return (d + 30) % 60 - 30


def read_rows(path: Path):
    with open(path, encoding="utf-8") as f:
        return list(csv.reader(l for l in f if not l.startswith("#")))


def read_dicts(path: Path):
    with open(path, encoding="utf-8") as f:
        return list(csv.DictReader(l for l in f if not l.startswith("#")))


# ---------------------------------------------------------------- 逐月表：推步、张表

def lab_parse(s: str):
    leap = s.startswith("闰")
    return int(s[1:] if leap else s), leap


def lab(m: int, leap: bool) -> str:
    return f"{'闰' if leap else ''}{m}"


# 逐月表可由命令行另指（另一程序重跑 eval 改写共享产物时，用快照）：python tongwu_causes.py [逐月表路径]
import hashlib  # noqa: E402

MONTHS_PATH = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "data/calendar/zhongyuan_reckoning_months.csv"
MONTHS_SHA = hashlib.sha256(MONTHS_PATH.read_bytes()).hexdigest()
MONTHS = read_dicts(MONTHS_PATH)
ZHANG = {(y, m, l): j for y, m, l, j in gd._months()}
ZHANG_SORTED = sorted(gd._months(), key=lambda t: t[3])
ZHANG_NEXT = {(t[0], t[1], t[2]): ZHANG_SORTED[i + 1][3] for i, t in enumerate(ZHANG_SORTED[:-1])}
ZHANG_PREV = {(t[0], t[1], t[2]): ZHANG_SORTED[i - 1][3] for i, t in enumerate(ZHANG_SORTED) if i}

# 推步逐年月序（推步月名），JDN 由同月张表 JDN（相同之月）或干支递推
RECK: dict[int, list] = defaultdict(list)  # y -> [(m, leap, gz, 分类, 历, 张表朔)]
for r in MONTHS:
    m, l = lab_parse(r["月"])
    RECK[int(r["年"])].append((m, l, r["推步朔"], r["分类"], r["历"], r["张表朔"], r["证据等级"], r["注"]))
for y in RECK:
    RECK[y].sort(key=lambda e: (e[0], e[1]))
RECK_SEQ = [(y,) + e for y in sorted(RECK) for e in RECK[y]]
RECK_IDX = {(e[0], e[1], e[2]): i for i, e in enumerate(RECK_SEQ)}
LIFA_YEARS: dict[str, list[int]] = defaultdict(list)
for y in sorted(RECK):
    for n in {e[4] for e in RECK[y]}:
        LIFA_YEARS[n].append(y)


def reck_span(y, m, l):
    """推步该月 [朔, 下朔) 与上月朔（JDN）：同月张表 JDN 为锚，下月、上月以干支推。"""
    i = RECK_IDX[(y, m, l)]
    a = ZHANG[(y, m, l)]
    g0 = gzi(RECK_SEQ[i][3])
    assert gz(a) == RECK_SEQ[i][3]
    nxt = a + (gzi(RECK_SEQ[i + 1][3]) - g0) % 60 if i + 1 < len(RECK_SEQ) else None
    prv = a - (g0 - gzi(RECK_SEQ[i - 1][3])) % 60 if i else None
    return a, nxt, prv


# ---------------------------------------------------------------- 证据：同 zhongyuan_reckoning_eval.rs

def lineage(book: str) -> str:
    b = book.replace("_benji", "")
    return {"jiutangshu": "唐", "xintangshu": "唐", "jiuwudaishi": "五代", "xinwudaishi": "五代", "beishi": "北朝", "weishu": "北朝",
            "beiqishu": "北朝", "zhoushu": "北朝", "suishu": "北朝", "nanshi": "南朝", "songshu": "南朝", "nanqishu": "南朝",
            "liangshu": "南朝", "chenshu": "南朝", "jinshu": "魏晋", "sanguozhi": "魏晋"}.get(b, b)


def book_level(book: str, shuo: bool) -> str:
    if any(book.startswith(b) for b in ["houhanshu", "jinshu", "nanshi", "beishi", "xinwudaishi"]):
        return "E"
    return "C" if shuo else "D"


BOOK_NAME = {"hanshu": "《汉书》本纪", "houhanshu": "《后汉书》本纪", "sanguozhi": "《三国志》", "jinshu": "《晋书》本纪", "songshu": "《宋书》本纪",
             "nanqishu": "《南齐书》本纪", "liangshu": "《梁书》本纪", "chenshu": "《陈书》本纪", "suishu": "《隋书》本纪", "beishi": "《北史》本纪",
             "jiutangshu": "《旧唐书》本纪", "xintangshu": "《新唐书》本纪", "jiuwudaishi": "《旧五代史》本纪", "xinwudaishi": "《新五代史》本纪",
             "songshi": "《宋史》本纪", "yuanshi": "《元史》本纪", "mingshi": "《明史》本纪", "qingshigao": "《清史稿》本纪",
             "后汉书": "《后汉书》本纪", "晋书": "《晋书》本纪", "旧唐书": "《旧唐书》本纪", "新唐书": "《新唐书》本纪", "宋史": "《宋史》本纪"}


def bname(b: str) -> str:
    return BOOK_NAME.get(b.replace("_benji", ""), b)


ERRATA = {(r["书"], r["年"], r["月"]): r for r in read_dicts(ROOT / "data/calendar/benji_shuo_errata.csv")}
QITA = {(r["书"], r["年"], r["月"]): r for r in read_dicts(ROOT / "data/calendar/benji_qita_kaozheng_notes.csv")}

# 已有考证（唐宋元五代等）：(年, 月, 闰) → [结论]
KAOZ: dict = defaultdict(list)
for f in ["tang_shuo_kaozheng", "song_shuo_kaozheng", "tangsong_cunyi_kaozheng", "wudai_shuo_kaozheng"]:
    for r in read_dicts(ROOT / f"data/calendar/{f}.csv"):
        try:
            KAOZ[(int(r["年"]), int(r["月"]), r.get("闰", "0") == "1")].append(f"{f}：{r['证据出处'][:40]}→{r['结论'][:80]}")
        except (ValueError, KeyError):
            pass
for r in read_dicts(ROOT / "data/calendar/yuan_shuo_kaozheng.csv"):
    try:
        KAOZ[(int(r["年"]), int(r["月"]), False)].append(f"yuan_shuo_kaozheng：{r['出处'][:40]}→{r['结论'][:80]}")
    except (ValueError, KeyError):
        pass

SAME = {(int(r["年"]),) + lab_parse(r["月"]) for r in MONTHS if r["异同"] == "相同"}
TONGWU = [r for r in MONTHS if r["分类"] == "同误"]

shuo_ev: dict = defaultdict(list)   # key -> [dict]
IN_ROWS: dict = defaultdict(list)   # (key, 书) -> 该月内纪日之行号（定月外之条在月前还是月后）
jiri_ev: dict = defaultdict(list)   # key -> [dict]


def collect_shuo():
    skip = ["liaoshi_benji", "jinshi_benji", "weishu_benji", "beiqishu_benji", "zhoushu_benji"]
    for r in read_dicts(ROOT / "data/calendar/benji_shuo_check.csv"):
        if r["书"] in skip or r["政权"] in ("吴", "蜀"):
            continue
        try:
            y, rec = int(r["年"]), int(r["本纪朔 JDN"])
        except ValueError:
            continue
        if r["政权"] == "元" and y < 1280:
            continue
        mm = r["月"]
        leap = mm.startswith("闰") or mm.startswith("閏")
        try:
            m = int(mm[1:] if leap else mm)
        except ValueError:
            continue
        t = gd.table_month(r["政权"], y, gd.MONTH_NAMES[m - 1], leap)
        ty, tm, tl = t if t else (y, m, leap)
        if r["书"] == "hanshu_benji" and ty <= -104:
            continue
        key = (ty, tm, tl)
        if key not in SAME:
            continue
        a = ZHANG[key]
        k3 = (r["书"], r["年"], r["月"])
        judged = k3 in ERRATA or (r["类"] == "其他" and k3 in QITA and not QITA[k3]["判"].startswith("未明"))
        shuo_ev[key].append(dict(kind="明书朔", book=r["书"], regime=r["政权"], ok=(rec == a), obs=gz(rec), jdn=rec, grade=book_level(r["书"], True),
                                 judged=judged, text=r["原文"], label=f"{r['纪年']}{'闰' if leap else ''}{m}月", lei=r["类"], eclipse=r["日食"],
                                 other=r["他书同记"], k3=k3, era=r["纪年"]))


ZY_BOOKS = ["hanshu", "houhanshu", "sanguozhi", "jinshu", "songshu", "nanqishu", "liangshu", "chenshu", "suishu", "beishi", "jiutangshu", "xintangshu",
            "jiuwudaishi", "xinwudaishi", "songshi", "yuanshi", "mingshi", "qingshigao"]


def ok_regime(reg, y):
    if reg in ("西汉", "东汉", "魏", "晋", "宋", "齐", "梁", "陈", "唐", "武周", "后梁", "后唐", "后晋", "后汉", "后周"):
        return True
    return {"隋": y >= 590, "元": y >= 1280, "明": y <= 1644, "清": y >= 1645}.get(reg, False)


def collect_jiri():
    for b in ZY_BOOKS:
        p = ROOT / f"data/calendar/本纪纪日/{b}_benji.csv"
        if not p.exists():
            continue
        rows = read_rows(p)
        for ri, r in enumerate(rows[1:]):
            if len(r) < 15:
                continue
            try:
                y, m = int(r[3]), int(r[4])
            except ValueError:
                continue
            pos = r[12]
            if pos == "朔" or not ok_regime(r[0], y):
                continue
            leap = r[6] == "1"
            name = r[5].replace("臘", "腊") if r[5] else gd.MONTH_NAMES[m - 1]
            t = gd.table_month(r[0], y, name, leap) or gd.table_month(r[0], y, gd.MONTH_NAMES[m - 1], leap)
            if not t or t not in SAME or t not in ZHANG:
                continue
            a, br, _ = reck_span(*t)
            bt = ZHANG_NEXT[t]
            if br is None or r[7] not in GZ:
                continue
            d = a + (gzi(r[7]) - gzi(gz(a))) % 60
            in_r, in_t = d < br, d < bt
            if in_r and in_t:
                IN_ROWS[(t, b)].append(ri)
            nth = int(pos[1:]) if pos.startswith("日") and pos[1:].isdigit() else None
            full = "，".join(r[14:])
            text = full[:20]
            ok = None
            if not in_r and not in_t:
                ok = False
            elif in_r and in_t:
                if pos == "晦":
                    ok = br == bt and d == br - 1
                elif nth is not None:
                    ok = d == a + nth - 1
                elif d == a or (br == bt and d == br - 1):
                    ok = True
            if ok is None:
                continue
            jiri_ev[t].append(dict(kind="纪日", book=b, regime=r[0], ok=ok, obs=r[7], day=d, pos=pos, nth=nth, grade=book_level(b, False), text=full, era=r[1], eyear=r[2], mname=name, leap_lab=leap, ri=ri,
                                   key20=text, label=f"{r[1]}{r[2]}年{'闰' if leap else ''}{name}", next_r=br, next_t=bt))


def jiri_count(js):
    seen, per = set(), Counter()
    lv = "E"
    for j in js:
        if (j["book"], j["key20"]) in seen:
            continue
        seen.add((j["book"], j["key20"]))
        per[(lineage(j["book"]), j["book"])] += 1
    by = defaultdict(int)
    for (l, b), n in per.items():
        by[l] = max(by[l], n)
        lv = min(lv, book_level(b, False))
    return sum(by.values()), lv


def reproduce_class(key):
    """按 eval 规则重判该月（以核对重取之证据与生成器一致）。"""
    sup, con = [], []
    for s in shuo_ev.get(key, []):
        (sup if s["ok"] else con).append((s["grade"], s["judged"]))
    js = [j for j in jiri_ev.get(key, []) if j["ok"]]
    jc = [j for j in jiri_ev.get(key, []) if not j["ok"]]
    ns, ls = jiri_count(js)
    nc, lc = jiri_count(jc)
    if ns >= 2:
        sup.append((ls, False))
    if nc >= 2:
        con.append((lc, False))
    unj = [c for c in con if not c[1]]
    if sup and not unj:
        return "有证"
    if con:
        return "同误"
    return "无证"


# ---------------------------------------------------------------- 实际合朔

SKY = None


def sky():
    global SKY
    if SKY is None:
        from skyfield import almanac
        from skyfield.api import load, load_file
        ts = load.timescale(builtin=True)
        bsp = ROOT / ("vendor/jpl/de441/de441.bsp" if (ROOT / "vendor/jpl/de441/de441.bsp").exists() else "vendor/jpl/de406/de406.bsp")
        eph = load_file(str(bsp))
        SKY = (ts, eph, almanac, bsp.name)
    return SKY


CAPITALS = read_dicts(ROOT / "data/phenomena/capitals.csv")


def lon_of(regime, year):
    for r in CAPITALS:
        if r["regime"] == regime and (not r["from_year"] or int(r["from_year"]) <= year) and (not r["to_year"] or year <= int(r["to_year"])):
            return float(r["lon"])
    return 108.94


_conj_cache = {}


def conjunction(jdn, lon):
    """JDN 附近（±4 日）合朔：(地方平时所在日 JDN, 时)。"""
    k = (jdn, lon)
    if k in _conj_cache:
        return _conj_cache[k]
    ts, eph, almanac, _ = sky()
    t0, t1 = ts.ut1_jd(jdn - 5), ts.ut1_jd(jdn + 5)
    times, phases = almanac.find_discrete(t0, t1, almanac.moon_phases(eph))
    best = None
    for t, p in zip(times, phases):
        if p != 0:
            continue
        local = t.ut1 + lon / 360
        day = int(local + 0.5)
        hour = (local + 0.5 - day) * 24
        if best is None or abs(day - jdn) < abs(best[0] - jdn):
            best = (day, hour)
    _conj_cache[k] = best
    return best


# ---------------------------------------------------------------- 成因判定

# 形近：同位（天干对天干、地支对地支）而字形相近，史书校勘常见者
XINGJIN_GAN = [("乙", "己"), ("丙", "两"), ("壬", "王"), ("戊", "戌"), ("己", "已"), ("辛", "幸"), ("甲", "申"), ("丁", "乙"), ("庚", "康"), ("癸", "葵")]
XINGJIN_ZHI = [("未", "末"), ("午", "牛"), ("巳", "己"), ("巳", "已"), ("子", "于"), ("申", "甲"), ("亥", "亦"), ("戌", "戊"), ("寅", "演"),
               ("未", "午"), ("辰", "長")]
XJ = {frozenset(p) for p in XINGJIN_GAN + XINGJIN_ZHI}
# 跨位形近（如干「戊」被读成支「戌」）：ganzhi 中一字换成另一位置的形近字后整体仍为干支——不可能，故只能整干支形近相通（如「己巳」/「乙巳」）


def one_char(a: str, b: str):
    """两干支只差一字 → (位置, 原字, 今字)；否则 None。"""
    if a[0] == b[0] and a[1] != b[1]:
        return ("支", a[1], b[1])
    if a[1] == b[1] and a[0] != b[0]:
        return ("干", a[0], b[0])
    return None


def xingjin(a: str, b: str) -> bool:
    oc = one_char(a, b)
    return bool(oc) and frozenset((oc[1], oc[2])) in XJ


def shuo_of(y, m, l, table="推步"):
    k = (y, m, l)
    if table == "张表":
        return ZHANG.get(k)
    return ZHANG.get(k) if k in SAME else None


def neighbor_months(key):
    """推步序列中上一月、下一月的 (标签, 朔 JDN)。"""
    i = RECK_IDX[key]
    a, nxt, prv = reck_span(*key)
    out = {}
    if i:
        p = RECK_SEQ[i - 1]
        out["上月"] = (f"{p[0]}年{lab(p[1], p[2])}月", prv, p[2])
    if i + 1 < len(RECK_SEQ):
        n = RECK_SEQ[i + 1]
        out["下月"] = (f"{n[0]}年{lab(n[1], n[2])}月", nxt, n[2])
    return out


def change_year(name, y):
    ys = LIFA_YEARS[name]
    if y == min(ys):
        return f"{name}行用首年（{y}）"
    if y == max(ys):
        return f"{name}行用末年（{y}）"
    return ""


def proxied(name):
    return any(k in name for k in ("术不传", "代推", "借", "补修", "宣明气朔"))


def analyze_shuo(e, key, regime_lon):
    """明书朔一条：差日与成因候选 [(成因大类, 细目, 依据, 强度)]。强度：据（有依据）/疑（形式上可能）。"""
    y, m, l = key
    a, nxt, prv = reck_span(*key)
    d = wrap(gzi(e["obs"]) - gzi(gz(a)))
    e["diff"] = d
    e["shift"] = ("朔", d)
    cands = []
    k3 = e["k3"]
    jk = jiaokan(e, locate(e))
    e["jk"] = jk
    if jk:
        cands.append(("史文讹误", "点校本校勘记涉及此条干支", jk[0], "据"))
    if k3 in ERRATA:
        er = ERRATA[k3]
        cands.append(("史文讹误", f"已考讹误：当作{er['改正']}", f"benji_shuo_errata.csv：{er['依据'][:90]}", "据"))
    if k3 in QITA:
        q = QITA[k3]
        j = q["判"]
        sub = "无从判断" if j.startswith("未明") else None
        if j.startswith("一字误"):
            cands.append(("史文讹误", f"干支一字之讹（{j}）", f"benji_qita_kaozheng_notes.csv：{q['按语'][:120]}", "疑" if "存疑" in j else "据"))
        elif j.startswith("朔衍"):
            cands.append(("史文讹误", f"「朔」字衍或其上脱事（{j}）", f"benji_qita_kaozheng_notes.csv：{q['按语'][:120]}", "疑" if "存疑" in j else "据"))
        elif j.startswith("月名误"):
            cands.append(("史文讹误", f"月名讹（{j}）", f"benji_qita_kaozheng_notes.csv：{q['按语'][:120]}", "疑" if "存疑" in j else "据"))
        elif j.startswith("年误"):
            cands.append(("纪年换算误", f"系年误（{j}）", f"benji_qita_kaozheng_notes.csv：{q['按语'][:120]}", "疑" if "存疑" in j else "据"))
        elif j.startswith("食日书朔"):
            cands.append(("记日习惯", f"食日书朔（{j}）", f"benji_qita_kaozheng_notes.csv：{q['按语'][:120]}", "据"))
        elif sub:
            cands.append(("无从判断", "「其他」类判未明", f"benji_qita_kaozheng_notes.csv：{q['按语'][:120]}", "疑"))
    # 北朝政权之条
    if e["regime"] in ("北魏", "东魏", "西魏", "北齐", "北周") or (e["regime"] == "隋" and y < 590):
        cands.append(("行用之历不同", f"{e['regime']}行本政权之历，非中原（南朝）之历", f"{bname(e['book'])}所记为{e['regime']}事；检验以中原一线之历衡之", "据"))
    nb = neighbor_months(key)
    # 邻月
    for side, (nm, nj, nl) in nb.items():
        if nj == e["jdn"]:
            if nl and side == "下月":
                cands.append(("史文讹误", f"脱「閏」字：所书为{nm}之朔", f"所书干支即推步、张表{nm}之朔", "疑"))
            elif l and side == "上月":
                cands.append(("史文讹误", f"衍「閏」字或闰位异：所书为{nm}之朔", f"所书干支即推步、张表{nm}之朔", "疑"))
            else:
                cands.append(("史文讹误", f"月名讹：所书为{nm}之朔", f"所书干支即推步、张表{nm}之朔", "疑"))
    # 年误：前后年同月
    for dy in (-1, 1):
        j = ZHANG.get((y + dy, m, l))
        if j is not None and gz(j) == e["obs"]:
            cands.append(("纪年换算误", f"系年差 {dy:+d} 年：所书为 {y + dy} 年同月之朔", "所书干支合张表前后年同月", "疑"))
    oc = one_char(gz(a), e["obs"])
    if oc and not any(c[0] == "史文讹误" and "一字" in c[1] for c in cands):
        xj = frozenset((oc[1], oc[2])) in XJ
        cands.append(("史文讹误", f"干支一字之异（{oc[0]}「{oc[2]}」/两家「{oc[1]}」{'，形近' if xj else ''}）",
                      f"改{oc[0]}一字即合两家（{gz(a)}）", "疑" if xj else "疑（非形近）"))
    # 实际合朔
    cj = conjunction(a, regime_lon)
    e["conj"] = cj
    if cj:
        cd, ch = cj
        e["conj_txt"] = f"{gz(cd)} {ch:.1f}时（{cd - a:+d}）"
        if cd == e["jdn"] and d != 0:
            if e["eclipse"] == "是" and y < 619:
                cands.append(("记日习惯", "食日书朔（平朔时代日食书于合朔之日）", f"所书即实际合朔日；平朔推步朔在{d:+d}日", "据"))
            elif y < 619:
                cands.append(("实际合朔", "所书合实际合朔（平朔时代：疑后人以定朔改书或观测朔）", f"实际合朔{gz(cd)} {ch:.1f}时", "疑"))
            else:
                cands.append(("实际合朔", "所书合实际合朔（定朔时代：当时历家合朔或进朔与推步不同）", f"实际合朔{gz(cd)} {ch:.1f}时；推步朔{gz(a)}", "疑"))
        if y >= 619 and d == 1 and cd == a and ch >= 15:
            cands.append(("记日习惯", "进朔：合朔在推步朔日之晚，史书记次日", f"实际合朔{gz(cd)} {ch:.1f}时", "疑"))
        if y >= 619 and d == -1 and cd == a and ch <= 3:
            cands.append(("记日习惯", "退朔/不进：合朔近子夜，史书记前一日", f"实际合朔{gz(cd)} {ch:.1f}时", "疑"))
    if 1 <= d <= 29 and not any("「朔」字衍" in c[1] for c in cands):
        cands.append(("史文讹误", f"「朔」字衍或其上脱事：所书为该月第 {d + 1} 日", "所书干支在两家该月内", "疑（弱）" if d <= 2 else "疑"))
    if d in (-1, -2) and not any(c[0] in ("实际合朔",) for c in cands):
        cands.append(("记日习惯", "以晦为朔或先天一二日", f"所书在两家朔前 {-d} 日", "疑"))
    return cands



# ---------------------------------------------------------------- 原文上下文（维基文库缓存，scripts/sky_culture/_cache/）

CN_BOOK = {"hanshu": "漢書", "houhanshu": "後漢書", "sanguozhi": "三國志", "jinshu": "晉書", "songshu": "宋書", "nanqishu": "南齊書", "liangshu": "梁書",
           "chenshu": "陳書", "suishu": "隋書", "beishi": "北史", "jiutangshu": "舊唐書", "xintangshu": "新唐書", "jiuwudaishi": "舊五代史",
           "xinwudaishi": "新五代史", "songshi": "宋史", "yuanshi": "元史", "mingshi": "明史", "qingshigao": "清史稿"}
CACHE = ROOT / "scripts/sky_culture/_cache"
_texts: dict = {}
MON_RE = re.compile(r"(閏)?(正|十一|十二|十|[一二三四五六七八九])月")
HEAD_PREV = set("。 　\n春夏秋冬年>=」")
CNNUM = {"正": 1, "一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9, "十": 10, "十一": 11, "十二": 12}


def book_texts(b):
    if b not in _texts:
        nb = b.replace("_benji", "").split("_")[0]
        _texts[b] = [(f.stem, f.read_text(encoding="utf-8")) for f in sorted(CACHE.glob(f"{CN_BOOK.get(nb, nb)}_卷*.txt"))]
    return _texts[b]


REF_RE = re.compile(r"<ref[^>]*>(.*?)</ref>", re.S)
TPL_RE = re.compile(r"\{\{\*\|.*?\}\}|\{\{[^{}]*\}\}|<[^>]+>", re.S)


def clean(t):
    """去 <ref> 校勘记、{{*|…}} 注文与其余模板、标签（月名判读只看正文）；{{YL|年号}} 之类正文模板保留其字。"""
    t = REF_RE.sub("", t)
    t = re.sub(r"\{\{(?:YL|PUA|lang\|[^|{}]*)\|([^{}|]*)\}\}", r"\1", t)
    for _ in range(3):
        t = TPL_RE.sub("", t)
    return t


def locate(e, n_chars=16):
    t = e["text"][:n_chars]
    if len(t) < 6:
        return None
    hits = []
    for n, s in book_texts(e["book"]):
        p = s.find(t)
        if p >= 0:
            hits.append((n, s, p))
            if s.find(t, p + 1) >= 0:
                hits.append((n, s, -1))
    return hits[0] if len(hits) == 1 else None


def jiaokan(e, hit):
    """该条附近（前 600 字至句后 300 字）点校本校勘记（维基文库录文 <ref>）中提到所书干支者。"""
    if not hit:
        return []
    n, s, p = hit
    out = []
    for mm in REF_RE.finditer(s, max(0, p - 600), min(len(s), p + 300)):
        body = TPL_RE.sub("", mm.group(1)).strip()
        if e["obs"] in body:
            out.append(f"{n}校勘记：{body[:150]}")
    return out


YEAR_RE = re.compile(r"(元|[一二三四五六七八九十]+)年")


def cn2int(t):
    if t == "元":
        return 1
    v = 0
    if "十" in t:
        a, _, b = t.partition("十")
        v = (CNNUM.get(a, 1) if a else 1) * 10 + (CNNUM.get(b, 0) if b else 0)
    else:
        v = CNNUM.get(t, 0)
    return v


def year_heading(win):
    """窗口内最后一个年首（「十八年春二月」「二十五年庚辰春」「元年夏」）：(年数, 片段)。"""
    last = None
    for mm in YEAR_RE.finditer(win):
        after = win[mm.end():mm.end() + 4]
        prev = win[mm.start() - 1] if mm.start() else "。"
        if not (prev in HEAD_PREV or "\u4e00" <= prev <= "\u9fff"):
            continue
        mon = MON_RE.match(win[mm.end():mm.end() + 3])
        if mon and re.match(r"[一二三四五六七八九十廿]+日", win[mm.end() + mon.end():mm.end() + mon.end() + 4]):
            continue  # 文中日期（「十二年四月二十七日敕」）
        if after[:1] in "春夏秋冬" or (after[:2] in GZ and after[2:3] in "春夏秋冬，") or mon:
            last = (cn2int(mm.group(1)), win[mm.start():mm.start() + 10].replace("\n", " "))
    return last


_ERA_MAX = None


def era_max(era):
    """年号最大年数（各政权同名年号取最大；年号表为简体，纪日表为繁体，经 OpenCC 转换）。"""
    global _ERA_MAX
    if _ERA_MAX is None:
        _ERA_MAX = defaultdict(int)
        for r in read_dicts(ROOT / "data/calendar/汇总/era_names_merged.csv"):
            for nm, yy in ((r["era_name"], r["era_year"]), (r["gaiyuan_prev_era_name"], r["gaiyuan_prev_era_year"])):
                try:
                    _ERA_MAX[nm] = max(_ERA_MAX[nm], int(yy))
                except ValueError:
                    pass
    try:
        import opencc
        era = opencc.OpenCC("t2s").convert(era)
    except Exception:
        pass
    return _ERA_MAX.get(era)


def context(e):
    """在原文中找到该条，看其前正文最近的月首：→ (判, 说明, 卷名)。判：抽取误（纪日表所系之月与原文月首不同，多因文中日期如
    「十月二十五日」被当作月首）／原文系此月／未定（改制月名）／未找到。"""
    hit = locate(e)
    e["hit"] = hit
    try:
        ey0 = 1 if e.get("eyear") in ("元", "1") else int(e.get("eyear"))
    except (TypeError, ValueError):
        ey0 = None
    mx = era_max(e.get("era", ""))
    if mx and ey0 and ey0 > mx:
        return ("抽取误", f"纪日表作{e.get('era', '')}{ey0}年，年号表{e.get('era', '')}只 {mx} 年（系年之误）", hit[0] if hit else "")
    if not hit:
        return ("未找到", "原文未见或多处见", "")
    n, s, p = hit
    # 本条若以月首起（如「九月己未，……」），月首即在本条内，故窗口含本条开头
    own = MON_RE.match(e["text"])
    win = clean(s[max(0, p - 5000):p + (own.end() + 2 if own else 0)])
    yr = year_heading(win)
    try:
        ey = 1 if e.get("eyear") in ("元", "1") else int(e.get("eyear"))
    except (TypeError, ValueError):
        ey = None
    # 「明年」：其后接月名或季名、在最后一个年首之后，而纪日表未加一年（改年后该日正在同名月内）
    my = list(re.finditer(r"明年(?=[春夏秋冬正二三四五六七八九十])", win))
    if my and e.get("key") and (not yr or win.rfind(yr[1]) < my[-1].start()):
        k2 = (e["key"][0] + 1,) + tuple(e["key"][1:])
        if k2 in ZHANG and k2 in ZHANG_NEXT:
            a2, b2 = ZHANG[k2], ZHANG_NEXT[k2]
            d2 = a2 + (gzi(e["obs"]) - gzi(gz(a2))) % 60
            if d2 < b2:
                e["year_doubt"] = f"原文其前有「{win[my[-1].start():my[-1].start() + 6]}…」而纪日表作{e.get('era', '')}{ey}年未加一年；移至 {k2[0]} 年同月则在月内"
    if yr and ey and yr[0] != ey and e.get("key"):
        # 只在按原文年首改年后，该干支确在同名之月内时才判系年之误（改元之年新旧年号并行，年数不同不足为据）
        y2 = e["key"][0] + yr[0] - ey
        k2 = (y2,) + tuple(e["key"][1:])
        if k2 in ZHANG and k2 in ZHANG_NEXT:
            a2, b2 = ZHANG[k2], ZHANG_NEXT[k2]
            d2 = a2 + (gzi(e["obs"]) - gzi(gz(a2))) % 60
            if d2 < b2:
                e["year_doubt"] = f"原文其前年首「{yr[1]}…」，纪日表作{e.get('era', '')}{ey}年；移至 {y2} 年同月则在月内"
    toks = []
    for mm in MON_RE.finditer(win):
        prev = win[mm.start() - 1] if mm.start() else "。"
        after = win[mm.end():mm.end() + 4]
        # 月首：前为句界、季名或年，后接干支、「，」或「朔」（如「十月庚寅朔」「冬十月，」）；否则为文中日期或叙述（「十月二十五日」「五月後來」）
        head_after = after[:2] in GZ or after[:1] in "，,、。" or "朔" in after[:3]
        embedded = prev not in HEAD_PREV or not head_after or bool(re.match(r"[一二三四五六七八九十廿]+日|朔日|晦日|望", after))
        toks.append((mm.group(0), bool(mm.group(1)), CNNUM[mm.group(2)], embedded, mm.start()))
    heads = [x for x in toks if not x[3]]
    lab_m = e["mname"]
    if not heads:
        return ("未找到", "前文无月首", n)
    h = heads[-1]
    cur = ("閏" if e.get("leap_lab") else "") + lab_m
    snippet = win[h[4]:h[4] + 12].replace("\n", " ")
    key = lab_m.replace("月", "")
    if key not in CNNUM:
        return ("未定", f"改制月名「{lab_m}」，原文其前月首「{snippet}…」", n)
    if h[2] == CNNUM[key] and h[1] == bool(e.get("leap_lab")):
        return ("原文系此月", f"原文其前月首「{snippet}…」", n)
    emb = [x for x in toks if x[3] and x[4] > h[4] and x[2] == CNNUM[key]]
    if emb:
        x = emb[-1]
        return ("抽取误", f"原文其前月首「{snippet}…」；其后文中「{win[x[4]:x[4] + 10]}…」被当作系月（纪日表作{cur}）", n)
    return ("抽取误", f"原文其前月首「{snippet}…」，纪日表作{cur}", n)


def analyze_jiri(e, key, regime_lon):
    """纪日一条：所需移动（日）与成因候选。"""
    y, m, l = key
    a, nxt, prv = reck_span(*key)
    d = e["day"]
    o = d - a
    L = nxt - a
    cands = []
    e["key"] = key
    cx = context(e)
    e["ctx"] = cx
    if cx[0] == "抽取误":
        sub = "纪日表系年之误（解析之误，非史文）" if "系年之误" in cx[1] else "纪日表系月与原文不同（解析之误，非史文）"
        cands.append(("纪日抽取误", sub, f"{cx[2]}：{cx[1]}", "据"))
    if e.get("year_doubt"):
        cands.append(("纪日抽取误", "纪日表系年疑误（解析之误）", f"{cx[2]}：{e['year_doubt']}", "疑"))
    jk = jiaokan(e, e.get("hit"))
    e["jk"] = jk
    if jk:
        cands.append(("史文讹误", "点校本校勘记涉及此条干支", jk[0], "据"))
    if e["pos"] == "晦":
        sh = d + 1 - nxt  # 下月朔须移动
        e["diff"] = sh
        e["shift"] = ("下朔", sh)
        e["need"] = f"晦在{gz(d)}：下月朔当为{gz(d + 1)}（{sh:+d}）"
    elif e["nth"] is not None:
        imp = d - (e["nth"] - 1)
        sh = wrap(imp - a)
        e["diff"] = sh
        e["shift"] = ("朔", sh)
        e["need"] = f"第 {e['nth']} 日{e['obs']}：朔当为{gz(imp)}（{sh:+d}）"
        exp = gz(a + e["nth"] - 1)
        oc = one_char(exp, e["obs"])
        if oc:
            xj = frozenset((oc[1], oc[2])) in XJ
            cands.append(("史文讹误", f"日干支一字之异（{oc[0]}「{oc[2]}」/应作「{oc[1]}」{'，形近' if xj else ''}）", f"两家第 {e['nth']} 日为{exp}", "疑" if xj else "疑（非形近）"))
        if 1 <= abs(sh) <= 2:
            cands.append(("记日习惯", f"日序差 {sh:+d} 日（朔日不同或日序起算不同）", e["need"], "疑"))
        elif abs(sh) >= 3:
            cands.append(("史文讹误", "日序或干支讹", e["need"], "疑"))
    else:
        after = o - L + 1          # 出月末几日（d 在下月第 after 日）
        before = 60 - o            # 早于月首几日
        # 月外之条落在下月还是上月：先看纪日表行序（同书同月月内之条之后 → 下月，之前 → 上月），无月内之条时取距月界较近者
        ins = IN_ROWS.get((key, e["book"]), [])
        if ins and e["ri"] > max(ins):
            go_after, e["order"] = True, "行序在月内诸条之后"
        elif ins and e["ri"] < min(ins):
            go_after, e["order"] = False, "行序在月内诸条之前"
        else:
            go_after, e["order"] = after <= before, ("行序夹在月内诸条之间" if ins else "同月无月内之条，取距月界较近者")
        e["diff"] = after if go_after else -before
        if go_after:
            e["shift"] = ("下朔", after)
            e["need"] = f"{e['obs']}在两家下月第 {after} 日（月末须后移 {after} 日；{e['order']}）"
        else:
            e["shift"] = ("朔", -before)
            e["need"] = f"{e['obs']}在两家月朔前 {before} 日（朔须前移 {before} 日；{e['order']}）"
        s = after if go_after else before
        nb = neighbor_months(key)
        if s <= 2:
            cands.append(("记日习惯", f"距月界 {s} 日：朔日或月大小与两家不同", e["need"], "疑"))
        elif e["order"].startswith("行序夹"):
            cands.append(("史文讹误", "日干支讹：行序夹在同月月内诸条之间（非系月之误）", e["need"] + "；" + e["order"], "疑"))
        else:
            side = "下月" if go_after else "上月"
            nm, nj, nl = nb.get(side, ("?", None, False))
            if nl:
                cands.append(("史文讹误", f"事在{nm}（闰月）：疑脱「閏」字或闰位异", e["need"], "疑"))
            else:
                cands.append(("史文讹误", f"事在{side}（{nm}）：系月误、脱月名或错简", e["need"], "疑"))
        # 形近一字可入月者
        alts = []
        for g2 in GZ:
            if g2 == e["obs"] or not xingjin(g2, e["obs"]):
                continue
            d2 = a + (gzi(g2) - gzi(gz(a))) % 60
            if d2 < nxt:
                alts.append(f"{g2}（第 {d2 - a + 1} 日）")
        if alts:
            cands.append(("史文讹误", f"日干支形近之讹：作{'、'.join(alts)}即在月内", "形近字表", "疑"))
    # 实际合朔（月界移动方向）
    sh = e["shift"]
    if sh[0] == "朔":
        cj = conjunction(a, regime_lon)
        base = a
    else:
        cj = conjunction(nxt, regime_lon)
        base = nxt
    e["conj"] = cj
    if cj:
        cd, ch = cj
        e["conj_txt"] = f"{gz(cd)} {ch:.1f}时（{'本月' if sh[0] == '朔' else '下月'}，{cd - base:+d}）"
        if sh[0] == "朔" and sh[1] != 0 and abs(sh[1]) <= 2:
            if (sh[1] < 0 and cd <= a + sh[1]) or (sh[1] > 0 and cd >= a + sh[1]) or (e["nth"] is not None and cd == a + sh[1]):
                cands.append(("实际合朔", "所需月朔合实际合朔", f"实际合朔{gz(cd)} {ch:.1f}时", "疑"))
        if sh[0] == "下朔" and abs(sh[1]) <= 2 and sh[1] != 0:
            if (sh[1] > 0 and cd >= nxt + sh[1]) or (sh[1] < 0 and cd <= nxt + sh[1]):
                cands.append(("实际合朔", "所需下月朔合实际合朔", f"实际合朔{gz(cd)} {ch:.1f}时", "疑"))
    return cands


# ---------------------------------------------------------------- 主程序

def mulu_table():
    """《通鉴目录》（刘羲叟《长历》）：(岁, 月, 闰) → 朔干支（只取列出之月；中原或南朝一行）。"""
    sys.path.insert(0, str(ROOT / "scripts/calendar_data"))
    try:
        import tongjian_mulu_check as tm
    except Exception:
        return {}
    out = {}
    for r in tm.load_data():
        try:
            y = int(r["年"])
        except ValueError:
            continue
        guo = r["国"]
        if not (guo == "" or (guo == "魏" and y <= 265) or guo in ("宋", "南齊", "梁", "陳") or (guo == "隋" and y >= 589)):
            continue
        raw = r["校读"] or r["朔闰原文"]
        try:
            toks, _ = tm.parse(raw)
        except Exception:
            continue
        for mm, ll, g in toks:
            out.setdefault((y, mm, ll), g)
    return out


# 人工复核（逐条读原文所得，优先于自动判定）：(年, 月, 所书干支) → (大类, 细目, 依据, 强度)
_SHUNDI = ("纪日抽取误", "纪日表系年之误：事在延光四年（125）十一月",
           "《後漢書·順帝紀》卷首：延光三年太子废，「明年三月，安帝崩……十一月丁巳，京師及郡國十六地震。是夜，中黃門孫程等……"
           "迎濟陰王……即皇帝位」，其后戊午、己未；纪日表未计「明年」，误系 124 年。125 年十一月两家甲寅朔，丁巳、戊午、己未为初四至初六", "据")
MANUAL = {(124, "11", "丁巳"): _SHUNDI, (124, "11", "戊午"): _SHUNDI, (124, "11", "己未"): _SHUNDI}

PRIMARY_ORDER = ["纪日抽取误", "史文讹误", "行用之历不同", "纪年换算误", "记日习惯", "实际合朔", "无从判断"]


def main():
    collect_shuo()
    collect_jiri()
    # 核对：重判与生成器一致
    mism = []
    for r in MONTHS:
        if r["异同"] != "相同":
            continue
        key = (int(r["年"]),) + lab_parse(r["月"])
        if key not in ZHANG:  # 张表以前（秦汉初）不在本页范围
            continue
        c = reproduce_class(key)
        if c != r["分类"]:
            mism.append((key, r["分类"], c))
    print(f"重判与 zhongyuan_reckoning_months.csv 不一致 {len(mism)} 月", Counter((a, b) for _, a, b in mism), [x for x in mism if "同误" in x][:20])
    mulu = mulu_table()
    rows_out, months_out = [], []
    for r in TONGWU:
        y = int(r["年"])
        m, l = lab_parse(r["月"])
        key = (y, m, l)
        name = r["历"]
        a, nxt, prv = reck_span(*key)
        evs = [dict(e) for e in shuo_ev.get(key, [])] + [dict(e) for e in jiri_ev.get(key, [])]
        con = [e for e in evs if not e["ok"]]
        sup = [e for e in evs if e["ok"]]
        regime = (con or evs)[0]["regime"]
        lon = lon_of(regime, y)
        month_c = Counter()
        month_basis = []
        for e in con:
            cands = analyze_shuo(e, key, lon) if e["kind"] == "明书朔" else analyze_jiri(e, key, lon)
            cy = change_year(name, y)
            if cy:
                cands.append(("行用之历不同", f"改历之年：{cy}", "行用年段之界，当时实用之历或与推步所据不同", "疑"))
            if proxied(name):
                cands.append(("行用之历不同", "术不传或代推之段：推步非当时本术", f"历名：{name}", "疑"))
            if (y, lab(m, l), e["obs"]) in MANUAL:
                cands.insert(0, MANUAL[(y, lab(m, l), e["obs"])])
            if not cands:
                cands.append(("无从判断", "无他据", "", "疑"))
            # 主因：有「据」者优先，其次按大类次序
            def rank(c):
                if c in MANUAL.values():
                    return (-2, 0)
                # 北朝政权之条以中原之历衡之，检验口径本身不合，列为主因
                if "行本政权之历" in c[1]:
                    return (-1, 0)
                return ({"据": 0, "疑": 1, "疑（非形近）": 2}.get(c[3], 3), PRIMARY_ORDER.index(c[0]))
            cands.sort(key=rank)
            e["cands"] = cands
            e["primary"] = cands[0]
        # 同向独立来源：所需移动相同（朔差或月界移动同向且合）
        # 「≥2 条独立之证一致」之组：同一月界（本月朔或下月朔）移动 s 日能同时解释两条以上独立之证（不同书或同书不同事，且不同日）。
        # 明书朔、书日序、书晦者为确数，s 取任意；不书日序之纪日只给下限，s 限 ±1、±2。逐组注排除之由，余为改推步候选。
        cand_reck = []
        for bd in ("朔", "下朔"):
            best = None
            for sft in [x for x in range(-30, 31) if x]:
                ok = []
                for e in con:
                    sh = e.get("shift")
                    if not sh or sh[0] != bd:
                        continue
                    exact = e["kind"] == "明书朔" or e.get("nth") is not None or e.get("pos") == "晦"
                    if exact:
                        hit = sh[1] == sft
                    else:
                        hit = abs(sft) <= 2 and ((sh[1] < 0 and sft <= sh[1]) or (sh[1] > 0 and sft >= sh[1]))
                    if hit:
                        ok.append(e)
                # 独立：明书朔按史源（同源之书互见不重计），纪日按（书、日）——同书同日之事只算一条
                evk = {("朔", lineage(e["book"])) if e["kind"] == "明书朔" else (e["book"], e["day"]) for e in ok}
                if len(evk) >= 2 and (best is None or len(evk) > best[2] or (len(evk) == best[2] and abs(sft) < abs(best[0][1]))):
                    best = ((bd, sft), ok, len(evk), len({lineage(e["book"]) for e in ok}))
            if best:
                cand_reck.append(best)
        # 支持之证（与同误冲突）
        conflict = bool(sup)
        kz = KAOZ.get(key, [])
        mg = mulu.get(key)
        for e in con:
            pc = e["primary"]
            others = [c for c in e["cands"][1:]]
            rows_out.append(dict(
                历=name, 年=y, 月=lab(m, l), 推步朔=gz(a), 张表朔=r["张表朔"], 证据类=e["kind"], 证据朔=(e["obs"] if e["kind"] == "明书朔" else ""),
                纪日干支=(e["obs"] if e["kind"] == "纪日" else ""), 位置=(e.get("pos") or ("朔" if e["kind"] == "明书朔" else "")),
                差日=e.get("diff", ""), 所需=(e.get("need") or f"朔{e['obs']}（{e['diff']:+d}）"), 证据原文=e["text"].replace("\n", " ")[:160],
                出处=f"{bname(e['book'])}·{e['label']}", 政权=e["regime"], 等级=e["grade"], 实际合朔=e.get("conj_txt", ""),
                长历=mg or "", 校勘记="｜".join(e.get("jk") or []), 原文上下文=(f"{e['ctx'][0]}（{e['ctx'][2]}）：{e['ctx'][1]}" if e.get("ctx") else ""), 成因=pc[0], 细目=pc[1], 依据=pc[2], 强度=pc[3],
                他解="；".join(f"{c[0]}：{c[1]}（{c[3]}）" for c in others),
                备注="；".join(x for x in [
                    (f"同月另有合两家之证 {len(sup)} 条（" + "、".join(f"{bname(s['book'])}{s['kind']}" for s in sup[:3]) + "）") if conflict else "",
                    "已判史书误（eval）" if e.get("judged") else "",
                    f"他书同记：{e['other']}" if e.get("other") else "",
                    ("已有考证：" + " | ".join(kz)) if kz else "",
                ] if x),
            ))
            month_c[pc[0]] += 1
        # 月级成因：若已判（据）者优先
        # 月级主因：有「据」者取「据」之中条数最多的大类，否则取各条主因中最多的大类（同数按大类次序）
        pool = [e["primary"][0] for e in con if e["primary"][3] == "据"] or [e["primary"][0] for e in con]
        pc_ = Counter(pool)
        mprim = min(pc_, key=lambda k: (-pc_[k], PRIMARY_ORDER.index(k))) if pc_ else "无从判断"
        months_out.append(dict(key=key, name=name, a=a, nxt=nxt, n_con=len(con), n_sup=len(sup), primary=mprim,
                               cats=month_c, cand=cand_reck, con=con, sup=sup, gz=gz(a), grade=r["证据等级"], mulu=mg, kz=kz))
    # 成串落入邻月：同月两条以上原文系于此月而落在同一邻月者
    runs = Counter((r["历"], r["年"], r["月"], r["细目"][:4]) for r in rows_out if r["细目"].startswith(("事在下月", "事在上月")))
    for r in rows_out:
        k = (r["历"], r["年"], r["月"], r["细目"][:4])
        if r["细目"].startswith(("事在下月", "事在上月")) and runs[k] >= 2:
            r["细目"] += f"；同月成串 {runs[k]} 条，" + ("疑脱下月月名" if r["细目"].startswith("事在下月") else "疑月名错位或错简（月首当在其后）")
    write_csv(rows_out)
    write_md(rows_out, months_out, mism)
    print(f"同误 {len(TONGWU)} 月，证据 {len(rows_out)} 条；≥2 条独立之证一致之组 {sum(len(x['cand']) for x in months_out)}")


COLS = ["历", "年", "月", "推步朔", "张表朔", "证据类", "证据朔", "纪日干支", "位置", "差日", "所需", "证据原文", "出处", "政权", "等级", "实际合朔",
        "长历", "校勘记", "原文上下文", "成因", "细目", "依据", "强度", "他解", "备注"]


def write_csv(rows):
    with open(OUT_CSV, "w", encoding="utf-8", newline="") as f:
        f.write("# 中原一线推步与张表相同而证据与两家皆不合（同误）之月逐条成因（scripts/calendar_data/tongwu_causes.py 生成，勿手改）。\n")
        f.write(f"# 所据逐月表 sha256 {MONTHS_SHA}（{len(TONGWU)} 月同误）。\n")
        f.write("# 一行一条不合两家之证据。月取自 data/calendar/zhongyuan_reckoning_months.csv 分类「同误」者；证据按 zhongyuan_reckoning_eval.rs 规则重取："
                "明书朔（benji_shuo_check.csv）、纪日（本纪纪日/*_benji.csv，不在两家该月内，或日序、晦不合）。\n")
        f.write("# 差日：明书朔 = 所书朔 − 两家朔；书日序者 = 所推朔 − 两家朔；书晦者 = 所推下月朔 − 两家下月朔；不书日序者 = 月界须移动之日数"
                "（正 = 在下月第 N 日，月末须后移；负 = 在朔前 N 日，朔须前移）。\n")
        f.write("# 实际合朔：skyfield + DE406（或 DE441），都城地方平时，括注合朔日 − 两家朔（或下月朔）。长历：《资治通鉴目录》所列该月朔（只作旁参）。\n")
        f.write("# 成因：纪日抽取误（纪日表系月、系年与原文不合）／史文讹误／行用之历不同／纪年换算误／记日习惯／实际合朔／无从判断；"
                "强度：据＝已有考证、校勘记或原文明证，疑＝形式上可能、无他证，疑（非形近）＝改一字可通而二字不形近，疑（弱）＝另有更近之解。"
                "校勘记：维基文库录文所附点校本校勘记中提到所书干支者；原文上下文：该条在原文中其前之月首（判纪日抽取误所据）。他解：其余可能（同格式）。证据只由上往下：合张表、合推步不作证据。\n")
        w = csv.DictWriter(f, fieldnames=COLS)
        w.writeheader()
        for r in rows:
            w.writerow(r)


def esc(s):
    return str(s).replace("|", "／").replace("\n", " ")


DYN = [(-300, 220, "汉"), (220, 265, "三国魏"), (265, 420, "晋"), (420, 589, "南朝"), (589, 618, "隋"), (618, 907, "唐"), (907, 960, "五代"),
       (960, 1279, "宋"), (1279, 1368, "元"), (1368, 1644, "明"), (1644, 1912, "清")]


def dyn(y):
    for a, b, n in DYN:
        if a <= y < b:
            return n
    return "?"


def write_md(rows, months, mism):
    L = []
    L.append("# 推步与张表「同误」之月的成因")
    L.append("")
    L.append("由 `scripts/calendar_data/tongwu_causes.py` 生成，**勿手改**；逐条数据 `data/calendar/tongwu_causes.csv`。上游为 "
             "`rust/core/examples/zhongyuan_reckoning_eval.rs` 的逐月表 `data/calendar/zhongyuan_reckoning_months.csv`"
             "（推步与张表逐月相同、互异之分类，见 [rili-sources.md](rili-sources.md)「逐历检验」与 [benji-checks.md](benji-checks.md)）。"
             f"所据逐月表 sha256 `{MONTHS_SHA}`（git fe629cf 所入 LFS 对象；其后推步若改、eval 重跑，同误之月会变，须以新表重跑本脚本）。")
    L.append("")
    L.append("## 一、问题与方法")
    L.append("")
    L.append(f"中原一线逐历检验中，推步与张表朔日相同之月，若所据证据（本纪明书朔、本纪纪日）与两家都不合，分类记作「同误」，共 **{len(months)}** 月"
             f"（明书朔 C/E 级、纪日 D/E 级）。本表按生成器规则重取证据、逐月重判（不取实物），生成器判同误之月全部重判为同误；"
             f"另有 {sum(1 for x in mism if x[2] == '同误')} 月（{'、'.join(f'{k[0]}-{lab(k[1], k[2])}' for k, a, b in mism if b == '同误')}）"
             "本纪不合两家而有实物合两家，生成器判有证，不在本页；"
             f"生成器判有证而本表重判无证者 {sum(1 for x in mism if x[1] == '有证' and x[2] == '无证')} 月（生成器另取实物、他书等证据，本表只取本纪），不涉同误。「同误」只是说证据不合两家，"
             "不等于两家皆误：证据本身可能讹，记日习惯可能与历日之朔不同，也可能当时行用之历与推步所据不同。本页逐条查成因。")
    L.append("")
    L.append("**证据重取**（同 eval 规则）：明书朔取 `benji_shuo_check.csv`（跳过辽、金、魏、北齐、周书本纪与吴、蜀，元 1280 年前；月名经 "
             "`month_label_reforms.csv` 换为建寅表月）；纪日取 `本纪纪日/*_benji.csv` 中原诸史之条，只取能分辨朔日者——干支不在两家该月内，"
             "或书日序（「某日」）、书晦而与两家不合。一行一条不合之证据（同月可多条）。")
    L.append("")
    L.append("**差日**：明书朔为所书朔 − 两家朔；书日序者为所推之朔 − 两家朔；书晦者为所推下月朔 − 两家下月朔；"
             "不书日序之纪日只能给出月界至少须移动之日数（正 = 落在两家下月第 N 日；负 = 落在两家朔前 N 日）。"
             "60 干支一周两月，月外之日在下月或上月两读皆可：按纪日表行序，在同书同月月内诸条之后者读作下月，之前者读作上月；"
             "同月无月内之条者取距月界较近之读。")
    L.append("")
    L.append("**成因分类**（每条列全部可能，按「强度」与大类次序取一主因；余入「他解」）：")
    L.append("")
    L.append("| 大类 | 细目与判法 | 强度 |")
    L.append("|---|---|---|")
    L.append("| 纪日抽取误 | 纪日表（`本纪纪日/*_benji.csv`，`zhi_star_records.py` 机器抽取）所系之月与原文不合：在维基文库缓存原文（`scripts/sky_culture/_cache/`）中找到该条，"
             "去掉校勘记与注文后，其前正文最近的月首不是纪日表所系之月（多因文中日期如「十月二十五日」被当作月首，或录文月首干支讹而未被认作月首）；"
             "或纪日表年号年数超出年号表（如「端拱三年」「義寧七年」）；或原文其前年首与纪日表年数不同而改年后该日正在同名月内 | 前二者「据」，末者「疑」 |")
    L.append("| 史文讹误 | 点校本校勘记（维基文库录文 `<ref>`，该条前 600 字至后 300 字内提到所书干支者）；已考讹误（`benji_shuo_errata.csv`）；「其他」类判语（`benji_qita_kaozheng_notes.csv`：一字误、朔衍、月名误）；"
             "所书即邻月之朔（月名讹、脱「閏」字）；干支只差一字（天干或地支；形近者注明）；所书为月内某日（「朔」字衍）；"
             "纪日落在邻月三日以上——按纪日表行序定在下月（月内诸条之后）还是上月（之前），同月成串者注「疑脱下月月名」或「疑月名错位」；"
             "行序夹在月内诸条之间者判日干支讹 | 前三者为「据」，余为「疑」；「疑（弱）」= 所书为两家朔后一二日而判「朔」字衍，另有进朔等解 |")
    L.append("| 记日习惯 | 平朔时代日食书于合朔之日（食日书朔）；定朔时代所书为推步朔次日而合朔在日晚（进朔）；所书早一二日（以晦为朔/先天）；"
             "纪日距月界一二日（朔日或月大小不同） | 食日书朔合实际合朔者「据」，余「疑」 |")
    L.append("| 行用之历不同 | 北朝政权之条（《北史》）以中原之历衡之；改历之年（行用首末年）；术不传、代推之段 | 北朝之条「据」，余「疑」 |")
    L.append("| 纪年换算误 | 「其他」类判年误；所书合前后年同月之朔 | 判语「据」，余「疑」 |")
    L.append("| 实际合朔 | 所书（或所需之月界）合 DE406/DE441 实际合朔所在日（都城地方平时） | 疑（提示观测朔或后人以定朔改书） |")
    L.append("| 无从判断 | 以上皆不成立 | — |")
    L.append("")
    L.append("「疑」只说明形式上可通，不是考定。按资料可靠性排序，证据只能由上往下检验：张表与推步相合不算证据，"
             "史书讹误须有校勘记、他书或形近之据才算「据」。")
    L.append("")
    # 汇总
    L.append("## 二、按成因汇总")
    L.append("")
    cc = Counter(r["成因"] for r in rows)
    cs = Counter((r["成因"], r["强度"]) for r in rows)
    mc = Counter(x["primary"] for x in months)
    L.append("以条计（主因）与以月计（月内有「据」者取「据」中最多之大类，否则取各条主因中最多之大类）：")
    L.append("")
    SS = ["据", "疑", "疑（非形近）", "疑（弱）"]
    L.append("| 成因 | 条数 | " + " | ".join(f"「{x}」" for x in SS) + " | 月数 |")
    L.append("|---|---|" + "---|" * len(SS) + "---|")
    for k in PRIMARY_ORDER:
        L.append(f"| {k} | {cc[k]} | " + " | ".join(str(cs[(k, x)]) for x in SS) + f" | {mc[k]} |")
    L.append(f"| 合计 | {len(rows)} | " + " | ".join(str(sum(v for (a, b), v in cs.items() if b == x)) for x in SS) + f" | {len(months)} |")
    L.append("")
    sub = Counter(re.sub(r"[（(].*", "", r["细目"]).split("：")[0] for r in rows)
    L.append("主因细目：")
    L.append("")
    L.append("| 细目 | 条数 |")
    L.append("|---|---|")
    for k, v in sub.most_common():
        L.append(f"| {esc(k)} | {v} |")
    L.append("")
    allc = Counter()
    for r in rows:
        allc[r["成因"]] += 1
        for x in r["他解"].split("；"):
            if x:
                allc[x.split("：")[0] + "（他解）"] += 1
    L.append("各条可能成因（含他解，一条可计多类）：" + "、".join(f"{k} {v}" for k, v in sorted(allc.items())) + "。")
    L.append("")
    conflict = sum(1 for x in months if x["n_sup"])
    L.append(f"同月另有合两家之证（证据互相冲突）者 {conflict} 月：这些月份本身已有证据支持两家，不合之条多为纪日系月之误。")
    L.append("")
    nj = sum(1 for r in rows if r["证据类"] == "纪日")
    nchou = sum(1 for r in rows if r["成因"] == "纪日抽取误")
    nrun = sum(1 for r in rows if "同月成串" in r["细目"])
    nbei = sum(1 for r in rows if "行本政权之历" in r["细目"])
    njk = sum(1 for r in rows if r["校勘记"])
    nsmall = sum(1 for r in rows if r["差日"] != "" and abs(int(r["差日"])) <= 2)
    L.append("要点：")
    L.append("")
    nbig = sum(1 for r in rows if r["证据类"] == "纪日" and abs(int(r["差日"])) >= 3)
    L.append(f"- 不合之证 {len(rows)} 条中纪日 {nj} 条、明书朔 {len(rows) - nj} 条。纪日之条差三日以上者 {nbig} 条（{100 * nbig // max(nj, 1)}%），"
             "是整条落在邻月（系月），不是朔日差一二日；能指向月界移动一二日的只有 "
             f"{nsmall} 条（含明书朔），其中没有一组能构成两源以上、同向、无反证的改推步候选（见第四节）。")
    L.append(f"- **纪日抽取误 {nchou} 条**：纪日表系月、系年与原文不合，是本项目纪日表的解析之误，不是史文，也不是推步或张表之误；"
             "宜回到 `scripts/calendar_data/qinhan_eval/zhi_star_records.py` 修正（文中日期不作月首、年号年数越界、录文月首干支讹）后重跑 eval。")
    L.append(f"- 原文本身系于此月而落在邻月、同月成串者 {nrun} 条：多为史文（或维基文库录文）脱下月月名、月名错位，"
             "清《清史稿》本纪尤多（如乾隆十三年三月丁巳以下诸条，录文脱「夏四月」）。")
    L.append(f"- 《北史》《隋书》所记北魏、北周、隋（开皇九年以前）之条 {nbei} 条以南朝之历衡之，不合是检验口径所致，"
             "应在 eval 中剔除或改检分政权推步（`regime_benji_check`）。")
    L.append(f"- 点校本校勘记已指出干支有误者 {njk} 条（晋书、宋书为多，如《晋书·武帝纪》太康元年五月「丙寅」校勘记：「五月丁亥朔，無丙寅……皆在六月內」）。")
    L.append("")
    # 按朝代
    L.append("## 三、按朝代、按历分布")
    L.append("")
    dc = defaultdict(Counter)
    for x in months:
        dc[dyn(x["key"][0])][x["primary"]] += 1
    L.append("以月计（主因）：")
    L.append("")
    L.append("| 朝代 | 月数 | " + " | ".join(PRIMARY_ORDER) + " |")
    L.append("|---|---|" + "---|" * len(PRIMARY_ORDER))
    for _, _, n in DYN:
        if n in dc:
            L.append(f"| {n} | {sum(dc[n].values())} | " + " | ".join(str(dc[n][k]) for k in PRIMARY_ORDER) + " |")
    L.append("")
    lc = defaultdict(Counter)
    for x in months:
        lc[x["name"]][x["primary"]] += 1
    L.append("| 历 | 月数 | " + " | ".join(PRIMARY_ORDER) + " |")
    L.append("|---|---|" + "---|" * len(PRIMARY_ORDER))
    for n, c in sorted(lc.items(), key=lambda t: -sum(t[1].values())):
        L.append(f"| {n} | {sum(c.values())} | " + " | ".join(str(c[k]) for k in PRIMARY_ORDER) + " |")
    L.append("")
    # 候选
    L.append("## 四、≥2 条独立之证一致者与候选改推步清单")
    L.append("")
    L.append("同月不合之证中，两条以上独立之证（明书朔按史源计，同源互见只算一条；纪日按书与日计，同书同日只算一条）同指一个朔"
             "（明书朔、日序、晦为确数，差日不限），或同指月界须同向移动一二日（不书日序之纪日只给下限）者，逐组列出。"
             "这些可能是推步与张表共同之误（进朔限、月大小、闰位、行用年段），也可能是共同的记日习惯或同源传抄之误，故逐组注结论："
             "**排除**（含已判纪日抽取误、北朝政权之条、已考史文讹误，或皆为平朔时代食日书朔，或差逾二日属月名闰位之事）、"
             "**弱候选**（同一史源、同月另有合两家之证或另有不能同时解释之条）、**候选**（两源以上、同向、无反证）。只列候选与理由，不改代码。")
    L.append("")
    L.append("| 历 | 年 | 月 | 两家朔 | 所需 | 独立条数 | 独立史源 | 证据 | 实际合朔 | 同月合两家之证 | 长历 | 已有考证 | 旁注 | 结论 | 理由 |")
    L.append("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")
    ncand = 0
    vcount = Counter()
    for x in months:
        for g, es, n, nl in x["cand"]:
            ncand += 1
            y, m, l = x["key"]
            bd, sft = g
            base = x["a"] if bd == "朔" else x["nxt"]
            need = f"{'本月朔' if bd == '朔' else '下月朔'}移 {sft:+d} 日（{gz(base)}→{gz(base + sft)}）；其余不合 {x['n_con'] - len(es)} 条"
            evt = "；".join(f"{bname(e['book'])}「{esc(e['text'][:24])}」" for e in es)
            cj = sorted({e.get("conj_txt", "") for e in es})
            ev = assess(x, es, g)
            vd, why = verdict(x, es, g)
            vcount[vd] += 1
            L.append(f"| {x['name']} | {y} | {lab(m, l)} | {x['gz']} | {esc(need)} | {n} | {nl} | {evt} | {esc('；'.join(cj))} | {x['n_sup']} | "
                     f"{x['mulu'] or ''} | {esc(' ／ '.join(x['kz'])[:80])} | {ev} | **{vd}** | {esc(why)} |")
    L.append("")
    L.append(f"共 {ncand} 组：" + "、".join(f"{k} {vcount[k]}" for k in ("候选", "弱候选", "排除")) + "。")
    L.append("")
    # 逐条
    L.append("## 五、逐条表")
    L.append("")
    L.append("一行一条不合两家之证据，按年月排列。「所需」为该证据要求的朔（或月界）；「实际合朔」括注与两家朔（或下月朔）之差。")
    L.append("")
    L.append("| 历 | 年 | 月 | 两家朔 | 类 | 所书 | 差日 | 所需 | 出处 | 等级 | 原文 | 实际合朔 | 长历 | 校勘记 | 原文上下文 | 成因 | 细目 | 强度 | 依据 | 他解 | 备注 |")
    L.append("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")
    for r in rows:
        L.append("| " + " | ".join(esc(r[k]) for k in ["历", "年", "月", "推步朔"]) + " | " + esc(r["证据类"]) + " | "
                 + esc(r["证据朔"] or (r["纪日干支"] + (f"（{r['位置']}）" if r["位置"] else ""))) + " | "
                 + " | ".join(esc(r[k]) for k in ["差日", "所需", "出处", "等级"]) + " | " + esc(r["证据原文"][:60]) + " | "
                 + " | ".join(esc(r[k]) for k in ["实际合朔", "长历", "校勘记", "原文上下文", "成因", "细目", "强度", "依据", "他解", "备注"]) + " |")
    L.append("")
    L.append("## 六、限制与待核")
    L.append("")
    nf = sum(1 for r in rows if r["原文上下文"].startswith("未找到"))
    L.append("- 成因「疑」只是形式判定：一字之异、邻月之朔、月内之日在 60 干支中常有多解，须对点校本校勘记、他书（《资治通鉴》《册府元龟》、实录）逐条核读才能定。"
             "本页所用依据：仓库已有考证表（`benji_shuo_errata.csv`、`benji_qita_kaozheng_notes.csv`、`tang_shuo_kaozheng.csv`、`song_shuo_kaozheng.csv`、"
             "`tangsong_cunyi_kaozheng.csv`、`wudai_shuo_kaozheng.csv`、`yuan_shuo_kaozheng.csv`）与维基文库录文所附点校本校勘记（`<ref>`，只取提到所书干支者）；"
             "未取点校本扫描页另核，校勘记录文与原书之异未核。")
    L.append(f"- 「纪日抽取误」靠在原文中定位该条与判读月首：月首判法（句界或季名之后、接干支或「，」「朔」）是启发式；原文中找不到或多处出现者 {nf} 条未判。"
             "系年之误只查了年号越界、「明年」与年首年数三种，其余（如改元之年新旧年号并用）未查；年首年数一法只记「疑」。人工复核只有 124 年十一月三条（`MANUAL`）。")
    L.append("- 原文系于此月、成串落入邻月者判「疑脱下月月名／月名错位」：维基文库录文脱字与点校本原文脱字未分，须对点校本核。")
    L.append("- 纪日之条（占同误证据之大半）不书日序，只知落在两家月外，不能定朔；同书一串条目常因一处月名脱漏而同时落入邻月，eval 以「同源取条数最多之书」计独立条数，"
             "一串实只一误，故 eval 的「纪日 N 条不合两家」高估了证据之独立性。")
    L.append("- 《北史》《隋书》所记北魏、北齐、北周、隋（590 年前）事按中原（南朝）之历检验，不合多因政权历不同，属检验口径问题，宜在 eval 中按政权剔除或改检分政权推步。")
    L.append("- 「纪年换算误」「实际合朔」两类没有一条成为主因：年号、月序换算之误实际都出在纪日表抽取（归「纪日抽取误」）；实际合朔只在明书朔、日序之条差一二日时作旁证，"
             "这些条另有更直接的解释（食日书朔、「朔」字衍、进朔）或无从判断。")
    L.append("- 实际合朔按都城地方平时，ΔT 用 skyfield 内置模型（历表 DE406；有 DE441 时用 DE441）；早期 ΔT 不确定可达半小时以上，近子夜之合朔所在日只作参考。")
    L.append("- 《资治通鉴目录》（刘羲叟《长历》）为宋人推算之表，只作旁参；其所列月份不全，空格为目录未列。")
    L.append("- 各出土历日检验文档（楼兰、出土具注历、汉简、走马楼、敦煌、尹湾等）另有少量同误，不在本页 356 月之内，未逐条查。")
    L.append("")
    OUT_MD.write_text("\n".join(L) + "\n", encoding="utf-8")


def verdict(x, es, g):
    """→ (结论, 理由)。结论：排除／弱候选／候选。"""
    why = []
    if any(e["primary"][0] == "纪日抽取误" for e in es):
        why.append("含已判纪日抽取误之条")
    if any("行本政权之历" in c[1] for e in es for c in e["cands"]):
        why.append("含北朝政权之条（检验口径）")
    if all(e["kind"] == "明书朔" and e.get("eclipse") == "是" for e in es) and x["key"][0] < 619:
        why.append("皆平朔时代食日书朔")
    if any(e["primary"][3] == "据" and e["primary"][0] == "史文讹误" for e in es):
        why.append("含已考史文讹误之条")
    if abs(g[1]) > 2:
        why.append(f"差 {g[1]:+d} 日，非朔日推算之差（月名、闰位或系月）")
    indep = len({lineage(e["book"]) for e in es})
    if why:
        return "排除", "；".join(why)
    if indep < 2 or x["n_sup"] or x["n_con"] > len(es):
        return "弱候选", "；".join(t for t in [
            "同一史源" if indep < 2 else "", f"同月另有 {x['n_sup']} 条合两家" if x["n_sup"] else "",
            f"另有 {x['n_con'] - len(es)} 条不能同时解释" if x["n_con"] > len(es) else ""] if t)
    return "候选", "两源以上独立、同向且无反证"


def assess(x, es, g):
    """候选的评估语。"""
    y = x["key"][0]
    parts = []
    if x["n_sup"]:
        parts.append(f"同月另有 {x['n_sup']} 条合两家，冲突")
    books = {e["book"] for e in es}
    if len({lineage(b) for b in books}) < 2:
        parts.append("同一史源")
    bd, sft = g
    base = x["a"] if bd == "朔" else x["nxt"]
    cjs = {e["conj"][0] for e in es if e.get("conj")}
    if base + sft in cjs:
        parts.append("所移之日合实际合朔")
    elif cjs:
        parts.append("实际合朔不在所移之日")
    if x["n_con"] > len(es):
        parts.append(f"另有 {x['n_con'] - len(es)} 条不能同时解释")
    if y < 619:
        parts.append("平朔时代：改推步须术文依据")
    return "；".join(parts) or "可查"


if __name__ == "__main__":
    main()
