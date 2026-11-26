"""鳌头卷二至卷五各门逐月吉日表与协纪月表、董公逐日判定逐日对照。

用法（仓库根）：python3 scripts/folk/aotou_men_vs_xieji.py
先跑 scripts/folk/aotou_men_tables.py、scripts/folk/aotou_qili.py 生成各 data/folk/aotou_*.csv。
前录十三表各一文件；续录诸表在 data/folk/aotou_jiri.csv（按「表」列取），不分月之表（月空）推及十二月。
自洽检验所用起例读 data/folk/aotou_rijia_shen.csv：该门有本门凶神栏者先取本门之值，否则依卷五日家凶神 → 卷二凶日月表 → 出行凶神之序。
读 data/folk/xieji_yuebiao.csv（协纪月表）、data/folk/dong_gong_days.csv（董公逐日判定）。
写 data/folk/aotou_men_vs_xieji.csv（一表一月一日一行），并生成 doc/folk/aotou_men_checks.md（逐条表，勿手改）。

同一表同月同干支重出者只计一次（等取先出者，「重出」列记之）。
协纪三态：宜／忌（含「諸事不宜」）／两不注。董公：先取该事明文（无条件者优先，次条件之辞），次「諸事」泛言，皆无为「无」。
"""
import csv
import re
from collections import Counter, defaultdict

GZ = re.compile("[甲乙丙丁戊己庚辛壬癸][子丑寅卯辰巳午未申酉戌亥]")

XIEJI = "data/folk/xieji_yuebiao.csv"
DONG = "data/folk/dong_gong_days.csv"
OUT = "data/folk/aotou_men_vs_xieji.csv"
DOC = "doc/folk/aotou_men_checks.md"
JIRI = "aotou_jiri.csv"
RIJIA = "data/folk/aotou_rijia_shen.csv"
XIONG = "data/folk/aotou_xiongshen.csv"

# (文件, aotou_jiri.csv 之表名或 None, 简称, 协纪事（None＝协纪月表无对应事项）, 董公事目, 本门凶神栏（aotou_rijia_shen.csv 出处关键字）, 本门宜忌总说（aotou_xiongshen.csv 表／门）)
TABLES = [
    ("aotou_chuxing.csv", None, "出行", "出行", ["出行"], [], None),
    ("aotou_xingchuan.csv", None, "行船", "乘船渡水", [], [], None),
    ("aotou_kaizhang.csv", None, "開張", "開市", ["開張"], [], None),
    ("aotou_liqi.csv", None, "立契", "立券交易", [], [], None),
    ("aotou_ruxue.csv", None, "入學", "入學", ["上學"], [], None),
    ("aotou_shangguan.csv", None, "上官", "上官赴任", ["上官"], [], None),
    ("aotou_dongtu.csv", None, "動土", "修造動土", ["動土"], ["動土凶日"], None),
    ("aotou_qigong.csv", None, "起工", "修造動土", ["起造", "修造"], [], None),
    ("aotou_shuzhu.csv", None, "豎柱上梁", "豎柱上梁", ["豎造"], [], None),
    ("aotou_gaiwu.csv", None, "蓋屋", "修造動土", ["起造", "修造"], [], None),  # 协纪月表只出「忌苫蓋」、无「宜苫蓋」，改对修造動土
    ("aotou_ruzhai.csv", None, "入宅", "移徙", ["入宅", "移居"], [], None),
    ("aotou_zhancao.csv", None, "斬草", "破土", ["破土"], [], None),
    ("aotou_anzang_zhuyue.csv", None, "安葬", "安葬", ["安葬"], [], None),
    # 以下续录（aotou_qili.py → aotou_jiri.csv）
    (JIRI, "逐月出財放債吉日", "出財放債", "開倉庫出貨財", [], [], None),
    (JIRI, "逐月出兵征討吉日", "出兵", None, [], ["出兵吉凶"], None),
    (JIRI, "逐月襲爵受封吉日", "襲爵受封", "襲爵受封", [], ["襲爵凶日"], ("襲爵凶日", "襲爵受封")),
    (JIRI, "襲爵受封宜日", "襲爵宜日（不分月）", "襲爵受封", [], ["襲爵凶日"], ("襲爵凶日", "襲爵受封")),
    (JIRI, "逐月上冊受封吉日", "上冊受封", "上冊受封", [], ["襲爵凶日"], None),
    (JIRI, "逐月給由考滿吉日", "給由考滿", None, [], [], ("上章宜忌", "給由考滿")),
    (JIRI, "逐月納表上章吉日", "納表上章", "上表章", [], [], ("上章宜忌", "納表上章")),
    (JIRI, "逐月臨政親民吉日", "臨政親民", "臨政親民", [], [], ("上章宜忌", "臨政親民")),
    (JIRI, "逐月檢舉刑獄吉日", "檢舉刑獄", None, [], ["襲爵凶日"], None),
    (JIRI, "逐月小兒剃頭吉日", "剃頭", "剃頭", [], [], ("剃頭宜忌", "小兒剃頭")),
    (JIRI, "經絡吉日", "經絡（不分月）", "經絡", [], ["經絡凶日"], None),
    (JIRI, "逐月經絡吉日", "經絡", "經絡", [], ["經絡凶日"], None),
    (JIRI, "裁衣合帳吉日", "裁衣（不分月）", "裁衣", [], ["裁衣凶日"], ("裁衣凶日", "裁衣合帳")),
    (JIRI, "逐月裁衣合帳吉日", "裁衣", "裁衣", [], ["裁衣凶日"], ("裁衣凶日", "裁衣合帳")),
    (JIRI, "安床設帳吉日", "安床（不分月）", "安牀", [], [], None),
    (JIRI, "逐月造床吉日", "造床", None, [], [], None),  # 协纪月表有安牀、无造床，不强对
    (JIRI, "作竈吉日", "作竈（不分月）", "修造動土", [], ["作竈凶日"], None),  # 协纪月表无作竈，对修造動土（同董公「作灶」入修造）
    (JIRI, "逐月作竈吉日", "作竈", "修造動土", [], ["作竈凶日"], None),
    (JIRI, "倉庫諸吉日", "倉庫（不分月）", "修倉庫", ["作倉庫"], ["倉庫凶日"], None),
    (JIRI, "逐月修作倉庫吉日", "修作倉庫", "修倉庫", ["作倉庫"], ["倉庫凶日"], None),
    (JIRI, "入倉開庫吉日", "入倉開庫（不分月）", "納財", [], ["倉庫凶日"], None),
    (JIRI, "逐月入倉開庫吉日", "入倉開庫", "納財", [], ["倉庫凶日"], None),  # 「五穀入倉、開張坊」，以入倉为主对納財
    (JIRI, "逐月種蒔栽植吉日", "種蒔栽植", "栽種", [], ["種植凶日"], None),
    (JIRI, "逐月收割吉日", "收割", None, [], [], None),
    (JIRI, "逐月祭祀鬼神吉日", "祭祀", "祭祀", [], ["祭祀吉凶"], None),
    (JIRI, "求醫諸吉日", "求醫（不分月）", "求醫療病", [], ["求醫吉凶"], ("求醫吉凶", "求醫療病")),
    (JIRI, "逐月求醫治病吉日", "求醫", "求醫療病", [], ["求醫吉凶"], ("求醫吉凶", "求醫療病")),
    (JIRI, "納采問名吉日", "納采問名（不分月）", "納采問名", ["納采"], ["嫁娶凶日"], None),
    (JIRI, "天牛不守塚日（啟攢）", "啟攢（不分月）", "啟攢", [], [], None),
    (JIRI, "十二月上吉日", "十二月上吉日", None, [], [], None),
]
N_OLD = 13
JC = "建除滿平定執破危成收開閉"


QILI = "data/folk/aotou_shensha_qili.csv"
JC_DAY = {"建": "建", "破": "破", "平": "平", "收": "收", "滿": "滿", "執": "執", "閉": "閉"}
# 不分月、按干支列举之凶日（取自前录 aotou_shensha_qili.csv）
FLAT = ("離窠", "人民離", "九土鬼", "九醜", "五不歸")
# 无本门凶神栏时同名之神取用次序（同前录：卷五日家凶神 → 卷二第 41 页凶日月表 → 卷二出行凶神；出行表天賊十月、天罡二月异文因此不取）
DEFAULT_ORDER = ["日家凶神", "凶日月表", "出行凶神"]
# 表尾、总说用语 → 起例表神名
ALIAS = {"刀砧殺": "刀砧煞", "火星": "火星凶日", "天牢": "天牢黑道", "死氣": "死炁", "刀砧": "刀砧煞", "冰消瓦解": "冰消瓦陷",
         "勾絞": "勾絞", "官符": "官符"}
EXPAND = {"魁罡": ["河魁", "天罡"], "陰陽錯": ["陰錯", "陽錯"], "大小耗": ["大耗", "小耗"], "天地賊": ["天賊", "地賊"],
          "天地火": ["天火", "地火"], "罪刑": ["刑獄"], "牢獄": ["牢日", "獄日"]}


OTHER = ("地中白虎", "橫天朱雀", "犯歸忌並忌遠回惟出行不忌", "天翻地覆")


def read_csv(path):
    f = open(path, encoding="utf-8")
    return list(csv.DictReader(l for l in f if not l.startswith("#")))


def load_rijia():
    """aotou_rijia_shen.csv 凶神：[(名, [每月值之集合]×12, 出处)]。"""
    out = []
    for r in read_csv(RIJIA):
        if r["吉凶"] != "凶":
            continue
        vals = [set() if v == "-" else set(v.split("+")) for v in r[list(r)[2]].split("|")]
        if len(vals) != 12:
            continue
        out.append((r["名"], vals, r["出处"]))
    return out


def load_flat():
    flat = {}
    for r in read_csv(QILI):
        if not r["月"] and r["神"] in FLAT and r["神"] not in flat:
            flat[r["神"]] = set(GZ.findall(r["值"]))
    return flat


def gods_for(own, rijia):
    """该门所用起例：神 → 每月值集合。本门凶神栏（出处含 own 关键字）优先，否则依 DEFAULT_ORDER；只见于他门凶神栏者，诸栏值同则借用，不同则不取。"""
    by = {}
    names = []
    for n, _, _ in rijia:
        if n not in names:
            names.append(n)
    for n in names:
        cands = [(v, src) for nn, v, src in rijia if nn == n]
        pick = None
        for key in list(own) + DEFAULT_ORDER:
            pick = next((v for v, src in cands if key in src), None)
            if pick:
                break
        if pick:
            by[n] = pick
        elif all(v == cands[0][0] for v, _ in cands):  # 只见于他门凶神栏、且诸栏值同者借用；诸栏不一者不借
            by[n] = cands[0][0]
    for n in ("月建", "月破", "建日", "破日", "閉日"):  # 与建除同值，由建除检验
        by.pop(n, None)
    return by


def own_names(own, rijia):
    """本门凶神栏所列之神（出处含 own 关键字者）。"""
    return {n for n, _, src in rijia if any(k in src for k in own)} - {"月建", "月破", "建日", "破日", "閉日", "除日（男忌）", "破日（女忌）"}


# 所取凶神栏不属本门者（借邻门之栏起例），不把该栏诸神并入所避
NOT_OWN_MEN = {"上冊受封", "檢舉刑獄", "納采問名（不分月）", "入倉開庫", "入倉開庫（不分月）"}


def aotou_hits(m, gz, jc, by, flat):
    """鳌头自家起例下该日所值凶神（含建除之日）。值为一字者：干比日干、支比日支；两字比干支。"""
    hit = []
    for god, mv in by.items():
        for v in mv[m - 1]:
            if (len(v) == 2 and v == gz) or (len(v) == 1 and v in gz):
                hit.append(god)
                break
    for god, ss in flat.items():
        if gz in ss:
            hit.append(god)
    hit.append(jc + "日")
    return hit


def avoid_seg(text):
    """表尾「不犯……」或总说「忌……」所举之段。"""
    m = re.search("凶神[^：；]{0,12}：", text)
    if "不犯" in text:
        seg = text.split("不犯", 1)[1]
    elif m:  # 无表尾注而记表前凶神栏者（如動土、起工、立契）
        seg = text[m.end():]
    elif re.search("(?<![土月歸血])忌", text):
        seg = re.split("(?<![土月歸血])忌", text, 1)[1]
        seg = re.split("又宜|宜|吉日", seg)[0]  # 忌条止于下一宜条（如求醫总说「……忌……合藥吉日……宜除破開日」）
    else:
        return ""
    return seg.split("（")[0]


def tail_gods(text, names):
    """所避之神中能按本书起例检验者。建除以「破平收日」「建破日」「執破」等形式出现。"""
    seg = avoid_seg(text)
    if not seg:
        return []
    seg = seg.replace("月建轉煞", "月建轉殺")
    # 名中含他神之名而实为另一神者，先抹去（地中白虎非白虎、橫天朱雀非朱雀黑道；出行表「歸忌……惟出行不忌」）
    for ph in OTHER:
        seg = seg.replace(ph, "＿")
    got = []
    for k, v in ALIAS.items():
        if k in seg and v in names:
            got.append(v)
    for k, v in EXPAND.items():  # 合称先展开并抹去（「陰陽錯」「牢獄」中再取「陽錯」「獄日」会漏）
        if k in seg:
            got += [x for x in v if x in names]
            seg = seg.replace(k, "＿")
    if re.search("(?<![天地])轉[煞殺]", seg) and "天地轉煞" in names and "月建轉" not in seg:
        got.append("天地轉煞")
    if re.search("月建(?!轉)", seg):
        got.append("建日")
    if "月破" in seg:
        got.append("破日")
    for n in sorted(names, key=len, reverse=True):  # 长名先取并抹去，免「天地破敗」中再取「地破」
        if n in seg:
            got.append(n)
            seg = seg.replace(n, "＿")
    lead = re.match("[建除滿平定執破危成收開閉]+", seg)
    runs = [lead.group(0)] if lead else []
    runs += re.findall("([建除滿平定執破危成收開閉]+)日", seg)
    runs += re.findall("(?<![不天地月])([建除滿平定執破危成收開閉]{2,})", seg)
    for run in runs:
        if len(run) == 1 and run + "日" not in seg:
            continue
        for ch in run:
            if ch in JC_DAY:
                got.append(ch + "日")
    return sorted(set(got))


def load_xieji():
    return {(int(r["月"]), r["干支"]): r for r in csv.DictReader(open(XIEJI, encoding="utf-8"))}


def xieji_state(r, ev):
    yi, ji = r["宜"].split(), r["忌"].split()
    if "諸事不宜" in yi or "諸事不宜" in ji:
        return "忌"
    if ev is None:
        return "无对应"
    if ev in yi:
        return "宜"
    if ev in ji:
        return "忌"
    return "两不注"


def load_dong():
    d = defaultdict(list)
    for r in csv.DictReader(open(DONG, encoding="utf-8")):
        d[(int(r["月"]), r["干支"])].append(r)
    return d


def dong_state(rows, evs):
    hy = [r for r in rows if r["事"] in evs]
    plain = [r for r in hy if not r["条件"]]
    if plain:
        return plain[-1]["宜忌"]
    if hy:
        return "条件" + hy[-1]["宜忌"]
    zs = [r for r in rows if r["事"] == "諸事"]
    zplain = [r for r in zs if not r["条件"]]
    if zplain:
        return zplain[-1]["宜忌"] + "（泛言）"
    if zs:
        return "条件" + zs[-1]["宜忌"] + "（泛言）"
    return "无"


def pct(a, b):
    return f"{a}（{round(100 * a / b)}%）" if b else "0"


def load_table(fn, title):
    if title is None:
        return list(csv.DictReader(open("data/folk/" + fn, encoding="utf-8")))
    return [r for r in read_csv("data/folk/" + fn) if r["表"] == title]


def load_zongshuo():
    """(表, 门) → 宜忌总说原文（aotou_xiongshen.csv）。"""
    return {(r["表"], r["门"]): r["原文"] for r in read_csv(XIONG) if r["神"] == "（宜忌总说）"}


def main():
    xj = load_xieji()
    dg = load_dong()
    rijia = load_rijia()
    flat = load_flat()
    zs = load_zongshuo()
    out = []
    meta = {}
    for fn, title, short, ev, devs, own, zkey in TABLES:
        rows = load_table(fn, title)
        tail = next((r["注"] for r in rows if r["月"] == "0"), "")
        by = gods_for(own, rijia)
        ztext = zs.get(zkey, "") if zkey else ""
        old = len(meta) < N_OLD
        if old:  # 前录各表中不分月之附表（如斬草表所附「斬草吉日」「破土鳴吠對日」）不入逐月对照，同前
            rows = [r for r in rows if r["月"] != ""]
        nomonth = all(r["月"] in ("", "0") for r in rows)
        meta[short] = dict(title=rows[0]["表"], juan=rows[0]["卷"], men=rows[0]["门"], ce=rows[0]["册"], ev=ev, devs=devs,
                           tail=tail, fn=fn, by=by, own=own, ownnames=own_names(own, rijia), ztext=ztext, nomonth=nomonth, old=old)
        seen = {}
        for r in rows:
            if not r["干支"] or r["月"] == "0":
                continue
            months = range(1, 13) if r["月"] == "" else [int(r["月"])]
            for m in months:
                k = (m, r["干支"])
                if k in seen:
                    seen[k]["重出"] = "是"
                    continue
                x = xj[k]
                rec = {
                    "表": short, "鳌头表": r["表"], "册": r["册"], "页": r["页"], "月": m, "干支": r["干支"], "等": r["等"] or "正",
                    "建除": x["建除"], "协纪事": ev or "", "协纪": xieji_state(x, ev), "协纪等第": x["等第"],
                    "董公事": "、".join(devs) or "（諸事）", "董公": dong_state(dg.get(k, []), devs),
                    "协纪凶神": x["凶神"], "协纪吉神": x["吉神"], "重出": "", "鳌头注": r["注"],
                    "鳌头起例所值": " ".join(aotou_hits(m, r["干支"], x["建除"], by, flat)),
                }
                seen[k] = rec
                out.append(rec)
    cols = list(out[0].keys())
    with open(OUT, "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=cols)
        w.writeheader()
        w.writerows(out)
    write_doc(out, xj, meta, flat)
    print(f"{len(out)} 行 → {OUT}；{DOC}")


def write_doc(out, xj, meta, flat):
    L = []
    p = L.append
    p("# 鳌头卷二至卷五各门吉日表逐日对照（生成文件，勿手改）")
    p("")
    p("由 `scripts/folk/aotou_men_vs_xieji.py` 生成；录文见 `scripts/folk/aotou_men_tables.py`（前录十三表）、`scripts/folk/aotou_qili.py`（续录诸表，`data/folk/aotou_jiri.csv`）"
      "与 `data/folk/aotou_*.csv`，说明与结论见 [tongshu_yiji.md](tongshu_yiji.md) 第十节第 5 小节。对照规则：同表同月同干支只计一次；协纪三态宜／忌（含諸事不宜）／两不注；"
      "董公先取该事明文，无则「諸事」泛言（记「（泛言）」），带条件者记「条件」。"
      "协纪月表只出「忌乘船渡水」「忌苫蓋」、不出其宜，故行船表协纪栏只有忌与两不注，蓋屋表改对「修造動土」。"
      "续录诸表中不分月者（原书只列干支、不系月，表名加「（不分月）」）推及十二月，一干支计十二日；"
      "协纪月表无对应事项之表（出兵、給由考滿、檢舉刑獄、造床、收割、十二月上吉日）只计协纪「諸事不宜」之日（记「忌」），余记「无对应」。"
      "作竈对修造動土（协纪月表无作竈），入倉開庫对納財，出財放債对開倉庫出貨財，納表上章对上表章。")
    p("")
    p("## 总表")
    p("")

    def head():
        p("| 表 | 鳌头出处 | 协纪事 | 日数（去重） | 正／外 | 协纪宜 | 协纪忌 | 协纪两不注 | 协纪宜该事之日 | 鳌头收其中 | 董公宜 | 董公忌 |")
        p("|---|---|---|---|---|---|---|---|---|---|---|---|")

    def row(short, mt):
        rs = [r for r in out if r["表"] == short]
        n = len(rs)
        c = Counter(r["协纪"] for r in rs)
        zheng = sum(r["等"] == "正" for r in rs)
        ev = mt["ev"]
        tot = sum(1 for x in xj.values() if xieji_state(x, ev) == "宜") if ev else "—"
        dyi = sum(r["董公"].startswith("宜") for r in rs)
        dji = sum(r["董公"].startswith("忌") for r in rs)
        pages = sorted({int(r["页"]) for r in rs})
        pg = f"{pages[0]}—{pages[-1]}" if len(pages) > 1 else str(pages[0])
        yi = pct(c["宜"], n) if ev else "—"
        nz = pct(c["两不注"], n) if ev else "—"
        ji = pct(c["忌"], n) + ("" if ev else "（諸事不宜）")
        p(f"| {short}（{mt['title']}） | {mt['juan']}，第 {mt['ce']} 册第 {pg} 页 | {ev or '（无）'} | {n} | {zheng}／{n - zheng} | {yi} | {ji} | "
          f"{nz} | {tot} | {c['宜'] if ev else '—'} | {pct(dyi, n)} | {pct(dji, n)} |")

    def total(keys, label):
        rs = [r for r in out if r["表"] in keys and meta[r["表"]]["ev"]]
        n = len(rs)
        c = Counter(r["协纪"] for r in rs)
        p(f"| {label} | | | {n} | | {pct(c['宜'], n)} | {pct(c['忌'], n)} | {pct(c['两不注'], n)} | | | | |")

    old = [k for k, v in meta.items() if v["old"]]
    new = [k for k, v in meta.items() if not v["old"]]
    p("### 前录十三表（卷二至卷四，`aotou_men_tables.py`）")
    p("")
    head()
    for k in old:
        row(k, meta[k])
    total(old, "合计（有对应事项者）")
    p("")
    p("### 续录诸表（卷二至卷五，`aotou_qili.py` → `aotou_jiri.csv`）")
    p("")
    head()
    for k in new:
        row(k, meta[k])
    total([k for k in new if not meta[k]["nomonth"]], "合计：分月之表（有对应事项者）")
    total([k for k in new if meta[k]["nomonth"]], "合计：不分月之表（有对应事项者）")
    p("")
    p("## 正选与外日分开")
    p("")
    p("| 表 | 正：日数 | 正：协纪宜 | 正：协纪忌 | 外：日数 | 外：协纪宜 | 外：协纪忌 |")
    p("|---|---|---|---|---|---|---|")
    for short in meta:
        rs = [r for r in out if r["表"] == short]
        z = [r for r in rs if r["等"] == "正"]
        w = [r for r in rs if r["等"] == "外"]
        if not w:
            continue
        cz, cw = Counter(r["协纪"] for r in z), Counter(r["协纪"] for r in w)
        p(f"| {short} | {len(z)} | {pct(cz['宜'], len(z))} | {pct(cz['忌'], len(z))} | {len(w)} | {pct(cw['宜'], len(w))} | {pct(cw['忌'], len(w))} |")
    p("")
    p("（无外日之表不列。）")
    p("")
    p("## 建除分布")
    p("")
    p("| 表 | " + " | ".join(JC) + " |")
    p("|---|" + "---|" * 12)
    for short in meta:
        cc = Counter(r["建除"] for r in out if r["表"] == short)
        p(f"| {short} | " + " | ".join(str(cc[j]) for j in JC) + " |")
    p("")
    p("## 鳌头收而协纪忌之日：协纪凶神出现次数（前 10）")
    p("")
    for short in meta:
        rs = [r for r in out if r["表"] == short and r["协纪"] == "忌"]
        if not rs:
            continue
        cc = Counter(g for r in rs for g in r["协纪凶神"].split())
        p(f"- {short}（{len(rs)} 日）：" + "、".join(f"{g} {k}" for g, k in cc.most_common(10)))
    p("")
    p("## 表尾注与本门宜忌总说（所避之神）")
    p("")
    for short, mt in meta.items():
        extra = f"；本门宜忌总说（`aotou_xiongshen.csv`）：{mt['ztext']}" if mt["ztext"] else ""
        p(f"- **{short}**（`data/folk/{mt['fn']}`{'' if mt['old'] else '「' + mt['title'] + '」'}）：{mt['tail'] or '（无表尾注）'}{extra}")
    p("")
    p("## 自洽检验：吉日表是否避开表尾所举之神（按本书起例）")
    p("")
    p("表尾「右吉日不犯……」与本门宜忌总说「忌……」所举诸神中，能按本书起例（`data/folk/aotou_rijia_shen.csv`：该门有本门凶神栏者取本门之值，"
      "否则依卷五日家凶神、卷二第 41 页凶日月表、卷二出行凶神之序；只见于他门凶神栏之神，诸栏值同者借用、不同者不检；月建、月破与建、破日同值，并入建除；離窠、人民離、九土鬼、九醜、五不歸取 `aotou_shensha_qili.csv` 不分月之串）"
      "或建除逐日检验的，逐表逐神计其犯日。「大小耗」作大耗、小耗，「天地賊」作天賊、地賊，「魁罡」作河魁、天罡，「火星」作火星凶日。"
      "伏斷（值宿）、滅沒（弦望晦朔）、空亡、長星短星（农历日）等未立按月起例者不检。「本门栏」列该表所取本门凶神栏；"
      "注「并入所避」者，该门凶神栏所列诸神（多为表前「凶星宜忌見上層」之栏）一并作所避之神检验，借邻门之栏者（上冊、檢舉刑獄借襲爵凶日，入倉開庫借倉庫凶日，納采借嫁娶凶日）只取其值、不并入。"
      "不分月之表原书不系月，本不按月避神，犯日只示其与按月凶神的交叠，不作自洽判断。")
    p("")
    p("| 表 | 本门栏 | 可检之神 | 犯日数 | 犯者（神：月干支，外日加＊） |")
    p("|---|---|---|---|---|")
    for short, mt in meta.items():
        names = set(mt["by"]) | set(flat)
        tg = set(tail_gods(mt["tail"], names)) | set(tail_gods(mt["ztext"], names))
        own = "、".join(mt["own"]) or "—"
        if mt["own"] and short not in NOT_OWN_MEN:
            tg |= mt["ownnames"]
            own += "（并入所避）"
        tg = sorted(tg)
        if not tg:
            p(f"| {short} | {own} | （无可检之神） | — | — |")
            continue
        rs = [r for r in out if r["表"] == short]
        bad = []
        for r in rs:
            hs = set(r["鳌头起例所值"].split())
            for g in tg:
                if g in hs:
                    bad.append(f"{g}：{r['月']}{r['干支']}{'＊' if r['等'] == '外' else ''}")
        if mt["nomonth"]:
            # 不分月之表推及十二月，犯日只列计数，按神合计
            cc = Counter(b.split("：")[0] for b in bad)
            lst = "、".join(f"{g} {k}" for g, k in cc.most_common()) or "无"
            p(f"| {short} | {own} | {'、'.join(tg)} | {len(bad)} | （不分月，按神计）{lst} |")
        else:
            p(f"| {short} | {own} | {'、'.join(tg)} | {len(bad)} | {'；'.join(bad) if bad else '无'} |")
    p("")
    p("## 两家起例对比：表尾所避之神按鳌头（旧通书）起例与按协纪起例各犯几日")
    p("")
    p("表尾举天賊、天火、重喪（百忌歌「正七連庚甲……辰戌丑未月戊己是重喪」，协纪改称復日并改定）者，分别按本书起例（天賊辰酉寅未……、天火顺行四仲、"
      "重喪百忌歌两干；有本门凶神栏者取本门之值）与协纪月表（`xieji_yuebiao.csv` 凶神栏的天賊、天火、復日）计犯日。若表依协纪改过，协纪起例下应少犯。")
    p("")
    p("| 表 | 神 | 按鳌头起例犯 | 按协纪起例犯 | 按协纪犯者（月干支，外日加＊） |")
    p("|---|---|---|---|---|")
    chong = {1: "庚甲", 7: "庚甲", 2: "乙辛", 8: "乙辛", 4: "丙壬", 10: "丙壬", 5: "丁癸", 11: "丁癸", 3: "戊己", 6: "戊己", 9: "戊己", 12: "戊己"}
    for short, mt in meta.items():
        seg = avoid_seg(mt["tail"]) + avoid_seg(mt["ztext"])
        rs = [r for r in out if r["表"] == short]
        for god, xg in (("天賊", "天賊"), ("天火", "天火"), ("重喪", "復日")):
            if god not in seg and not (god == "天賊" and "天地賊" in seg) and not (god == "天火" and "天地火" in seg):
                continue
            b = [r for r in rs if xg in r["协纪凶神"].split()]
            if mt["nomonth"]:
                lst = f"（不分月）{len(b)} 日次"
            else:
                lst = "；".join(f"{r['月']}{r['干支']}{'＊' if r['等'] == '外' else ''}" for r in b) or "无"
            if god == "重喪":
                a = [r for r in rs if r["干支"][0] in chong[int(r["月"])]]
                a2 = [r for r in rs if "復日" in r["鳌头起例所值"].split()]
                p(f"| {short} | 重喪（协纪復日） | 百忌歌两干 {len(a)}；本书第 41 页復日（同《集正》）{len(a2)} | {len(b)} | {lst} |")
            else:
                a = [r for r in rs if god in r["鳌头起例所值"].split()]
                p(f"| {short} | {god}（协纪{xg}） | {len(a)} | {len(b)} | {lst} |")
    p("")
    p("## 逐日")
    p("")
    for short, mt in meta.items():
        p(f"### {short}：{mt['title']}（{mt['juan']}，协纪「{mt['ev'] or '无对应'}」，董公「{'、'.join(mt['devs']) or '諸事'}」）")
        p("")
        rs = [r for r in out if r["表"] == short]
        if mt["nomonth"]:
            p("不分月，一干支一行，十二月之协纪、董公结果并列（月序正至十二，「宜」「忌」「—」＝两不注或无对应；董公「无」记「·」）。")
            p("")
            p("| 干支 | 页 | 协纪宜 | 协纪忌 | 协纪十二月 | 董公十二月 | 备注 |")
            p("|---|---|---|---|---|---|---|")
            by_gz = defaultdict(list)
            for r in rs:
                by_gz[r["干支"]].append(r)
            for gz, rr in by_gz.items():
                rr.sort(key=lambda r: r["月"])
                xs = "".join({"宜": "宜", "忌": "忌"}.get(r["协纪"], "—") for r in rr)
                ds = "".join("宜" if r["董公"].startswith("宜") else "忌" if r["董公"].startswith("忌") else "·" for r in rr)
                note = rr[0]["鳌头注"]
                p(f"| {gz} | {rr[0]['页']} | {xs.count('宜')} | {xs.count('忌')} | {xs} | {ds} | {note} |")
            p("")
            continue
        p("| 月 | 干支 | 鳌头 | 页 | 建除 | 协纪 | 协纪等第 | 董公 | 协纪凶神 | 鳌头起例所值 | 备注 |")
        p("|---|---|---|---|---|---|---|---|---|---|---|")
        for r in rs:
            note = "；".join(x for x in (("重出" if r["重出"] else ""), r["鳌头注"]) if x)
            p(f"| {r['月']} | {r['干支']} | {r['等']} | {r['页']} | {r['建除']} | {r['协纪']} | {r['协纪等第']} | {r['董公']} | {r['协纪凶神']} | {r['鳌头起例所值']} | {note} |")
        p("")
    with open(DOC, "w", encoding="utf-8") as f:
        f.write("\n".join(L) + "\n")
    # 供文档引用的摘要
    print("\n".join(L[:90]))


if __name__ == "__main__":
    main()
