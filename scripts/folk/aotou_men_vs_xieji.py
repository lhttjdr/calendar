"""鳌头卷二至卷五各门逐月吉日表与协纪月表、董公逐日判定逐日对照。

用法（仓库根）：python3 scripts/folk/aotou_men_vs_xieji.py
先跑 scripts/folk/aotou_men_tables.py 生成各 data/folk/aotou_*.csv。
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

# 文件 → (简称, 协纪事, 董公事目)
TABLES = [
    ("aotou_chuxing.csv", "出行", "出行", ["出行"]),
    ("aotou_xingchuan.csv", "行船", "乘船渡水", []),
    ("aotou_kaizhang.csv", "開張", "開市", ["開張"]),
    ("aotou_liqi.csv", "立契", "立券交易", []),
    ("aotou_ruxue.csv", "入學", "入學", ["上學"]),
    ("aotou_shangguan.csv", "上官", "上官赴任", ["上官"]),
    ("aotou_dongtu.csv", "動土", "修造動土", ["動土"]),
    ("aotou_qigong.csv", "起工", "修造動土", ["起造", "修造"]),
    ("aotou_shuzhu.csv", "豎柱上梁", "豎柱上梁", ["豎造"]),
    ("aotou_gaiwu.csv", "蓋屋", "修造動土", ["起造", "修造"]),  # 协纪月表只出「忌苫蓋」、无「宜苫蓋」，改对修造動土
    ("aotou_ruzhai.csv", "入宅", "移徙", ["入宅", "移居"]),
    ("aotou_zhancao.csv", "斬草", "破土", ["破土"]),
    ("aotou_anzang_zhuyue.csv", "安葬", "安葬", ["安葬"]),
]
JC = "建除滿平定執破危成收開閉"


QILI = "data/folk/aotou_shensha_qili.csv"
# 起例取用次序：卷五日家凶神 → 卷二第 41 页凶日月表 → 卷二出行凶神（同名先出者为准；出行表天賊十月、天罡二月异文因此不取）
QILI_ORDER = ["日家凶神", "凶日月表", "出行凶神"]
JC_DAY = {"建": "建", "破": "破", "平": "平", "收": "收", "滿": "滿", "執": "執", "閉": "閉"}


def load_qili():
    """神 → {月: 值}（只取按月起例者）；另返回不分月之干支串（離窠、人民離、九土鬼、九醜、五不歸）。"""
    by = defaultdict(dict)
    flat = {}
    rows = list(csv.DictReader(open(QILI, encoding="utf-8")))
    for t in QILI_ORDER:
        for r in rows:
            if r["表"] != t:
                continue
            if r["月"]:
                if r["神"] not in by or t == by[r["神"]].get("_t"):
                    by[r["神"]]["_t"] = t
                    by[r["神"]][int(r["月"])] = r["值"]
            elif r["神"] in ("離窠", "人民離", "九土鬼", "九醜", "五不歸") and r["神"] not in flat:
                flat[r["神"]] = set(GZ.findall(r["值"]))
    return by, flat


def aotou_hits(m, gz, jc, by, flat):
    """鳌头自家起例下该日所值凶神（含建除之日）。值为一字者：干比日干、支比日支；两字比干支。"""
    hit = []
    for god, mv in by.items():
        if god in ("天德", "天德合"):
            continue
        v = mv.get(m)
        if not v:
            continue
        if (len(v) == 2 and v == gz) or (len(v) == 1 and v in gz):
            hit.append(god)
    for god, ss in flat.items():
        if gz in ss:
            hit.append(god)
    hit.append(jc + "日")
    return hit


def tail_gods(tail, names):
    """表尾注「不犯……」所举之神中，能按本书起例检验者。建除以「破平收日」「建破日」等形式出现。"""
    seg = tail.split("不犯", 1)[1] if "不犯" in tail else ""
    seg = seg.split("（")[0]
    got = [n for n in sorted(names, key=len, reverse=True) if n in seg]
    # 「魁罡」即河魁、天罡；「陰陽錯」即陰錯、陽錯；「破日」等建除
    if "魁罡" in seg:
        got += ["河魁", "天罡"]
    if "陰陽錯" in seg:
        got += ["陰錯", "陽錯"]
    lead = re.match("[建除滿平定執破危成收開閉]+", seg)
    runs = [lead.group(0)] if lead else []
    runs += re.findall("([建除滿平定執破危成收開閉]+)日", seg)
    for run in runs:
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


def main():
    xj = load_xieji()
    dg = load_dong()
    by, flat = load_qili()
    out = []
    meta = {}
    for fn, short, ev, devs in TABLES:
        rows = [r for r in csv.DictReader(open("data/folk/" + fn, encoding="utf-8"))]
        tail = next((r["注"] for r in rows if r["月"] == "0"), "")
        meta[short] = (rows[0]["表"], rows[0]["卷"], rows[0]["门"], rows[0]["册"], ev, devs, tail, fn)
        seen = {}
        for r in rows:
            if not r["干支"] or r["月"] in ("", "0"):
                continue
            m = int(r["月"])
            k = (m, r["干支"])
            if k in seen:
                seen[k]["重出"] = "是"
                continue
            x = xj[k]
            rec = {
                "表": short, "鳌头表": r["表"], "册": r["册"], "页": r["页"], "月": m, "干支": r["干支"], "等": r["等"],
                "建除": x["建除"], "协纪事": ev, "协纪": xieji_state(x, ev), "协纪等第": x["等第"],
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
    write_doc(out, xj, meta, by, flat)
    print(f"{len(out)} 行 → {OUT}；{DOC}")


def write_doc(out, xj, meta, by, flat):
    L = []
    p = L.append
    p("# 鳌头卷二至卷五各门吉日表逐日对照（生成文件，勿手改）")
    p("")
    p("由 `scripts/folk/aotou_men_vs_xieji.py` 生成；录文见 `scripts/folk/aotou_men_tables.py` 与 `data/folk/aotou_*.csv`，"
      "说明与结论见 [tongshu_yiji.md](tongshu_yiji.md) 第十节第 5 小节。对照规则：同表同月同干支只计一次；协纪三态宜／忌（含諸事不宜）／两不注；"
      "董公先取该事明文，无则「諸事」泛言（记「（泛言）」），带条件者记「条件」。"
      "协纪月表只出「忌乘船渡水」「忌苫蓋」、不出其宜，故行船表协纪栏只有忌与两不注，蓋屋表改对「修造動土」。")
    p("")
    p("## 总表")
    p("")
    p("| 表 | 鳌头出处 | 协纪事 | 日数（去重） | 正／外 | 协纪宜 | 协纪忌 | 协纪两不注 | 协纪宜该事之日 | 鳌头收其中 | 董公宜 | 董公忌 |")
    p("|---|---|---|---|---|---|---|---|---|---|---|---|")
    for short, (title, juan, men, ce, ev, devs, tail, fn) in meta.items():
        rs = [r for r in out if r["表"] == short]
        n = len(rs)
        c = Counter(r["协纪"] for r in rs)
        zheng = sum(r["等"] == "正" for r in rs)
        tot = sum(1 for x in xj.values() if xieji_state(x, ev) == "宜")
        dyi = sum(r["董公"].startswith("宜") for r in rs)
        dji = sum(r["董公"].startswith("忌") for r in rs)
        pages = sorted({int(r["页"]) for r in rs})
        pg = f"{pages[0]}—{pages[-1]}" if len(pages) > 1 else str(pages[0])
        p(f"| {short}（{title}） | {juan}，第 {ce} 册第 {pg} 页 | {ev} | {n} | {zheng}／{n - zheng} | {pct(c['宜'], n)} | {pct(c['忌'], n)} | "
          f"{pct(c['两不注'], n)} | {tot} | {c['宜']} | {pct(dyi, n)} | {pct(dji, n)} |")
    all_n = len(out)
    c = Counter(r["协纪"] for r in out)
    p(f"| 合计 | | | {all_n} | | {pct(c['宜'], all_n)} | {pct(c['忌'], all_n)} | {pct(c['两不注'], all_n)} | | | | |")
    p("")
    p("## 正选与外日分开")
    p("")
    p("| 表 | 正：日数 | 正：协纪宜 | 正：协纪忌 | 外：日数 | 外：协纪宜 | 外：协纪忌 |")
    p("|---|---|---|---|---|---|---|")
    for short in meta:
        rs = [r for r in out if r["表"] == short]
        z = [r for r in rs if r["等"] == "正"]
        w = [r for r in rs if r["等"] == "外"]
        cz, cw = Counter(r["协纪"] for r in z), Counter(r["协纪"] for r in w)
        p(f"| {short} | {len(z)} | {pct(cz['宜'], len(z))} | {pct(cz['忌'], len(z))} | {len(w)} | {pct(cw['宜'], len(w))} | {pct(cw['忌'], len(w))} |")
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
        cc = Counter(g for r in rs for g in r["协纪凶神"].split())
        p(f"- {short}（{len(rs)} 日）：" + "、".join(f"{g} {k}" for g, k in cc.most_common(10)))
    p("")
    p("## 表尾注（所避之神）")
    p("")
    for short, (title, juan, men, ce, ev, devs, tail, fn) in meta.items():
        p(f"- **{short}**（`data/folk/{fn}`）：{tail}")
    p("")
    p("## 自洽检验：吉日表是否避开表尾所举之神（按本书起例）")
    p("")
    p("表尾「右吉日不犯……」所举诸神中，能按本书凶神起例表（`data/folk/aotou_shensha_qili.csv`：卷五日家凶神、卷二第 41 页凶日月表、卷二出行凶神，同名取先出者）"
      "或建除逐日检验的，逐表逐神计其犯日。只检本书有起例者；伏斷（值宿）、滅沒（弦望晦朔）、空亡等未立起例者不检。")
    p("")
    p("| 表 | 可检之神 | 犯日数 | 犯者（神：月干支，外日加＊） |")
    p("|---|---|---|---|")
    names = set(by) | set(flat)
    for short, (title, juan, men, ce, ev, devs, tail, fn) in meta.items():
        tg = tail_gods(tail, names)
        if not tg:
            p(f"| {short} | （表尾无可检之神） | — | — |")
            continue
        rs = [r for r in out if r["表"] == short]
        bad = []
        for r in rs:
            hs = set(r["鳌头起例所值"].split())
            for g in tg:
                if g in hs:
                    bad.append(f"{g}：{r['月']}{r['干支']}{'＊' if r['等'] == '外' else ''}")
        p(f"| {short} | {'、'.join(tg)} | {len(bad)} | {'；'.join(bad) if bad else '无'} |")
    p("")
    p("## 两家起例对比：表尾所避之神按鳌头（旧通书）起例与按协纪起例各犯几日")
    p("")
    p("表尾举天賊、天火、重喪（百忌歌「正七連庚甲……辰戌丑未月戊己是重喪」，协纪改称復日并改定）者，分别按本书起例（天賊辰酉寅未……、天火顺行四仲、"
      "重喪百忌歌两干）与协纪月表（`xieji_yuebiao.csv` 凶神栏的天賊、天火、復日）计犯日。若表依协纪改过，协纪起例下应少犯。")
    p("")
    p("| 表 | 神 | 按鳌头起例犯 | 按协纪起例犯 | 按协纪犯者（月干支，外日加＊） |")
    p("|---|---|---|---|---|")
    chong = {1: "庚甲", 7: "庚甲", 2: "乙辛", 8: "乙辛", 4: "丙壬", 10: "丙壬", 5: "丁癸", 11: "丁癸", 3: "戊己", 6: "戊己", 9: "戊己", 12: "戊己"}
    for short, (title, juan, men, ce, ev, devs, tail, fn) in meta.items():
        seg = tail.split("不犯", 1)[1] if "不犯" in tail else ""
        rs = [r for r in out if r["表"] == short]
        for god, xg in (("天賊", "天賊"), ("天火", "天火"), ("重喪", "復日")):
            if god not in seg:
                continue
            b = [r for r in rs if xg in r["协纪凶神"].split()]
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
    for short, (title, juan, men, ce, ev, devs, tail, fn) in meta.items():
        p(f"### {short}：{title}（{juan}，协纪「{ev}」，董公「{'、'.join(devs) or '諸事'}」）")
        p("")
        p("| 月 | 干支 | 鳌头 | 页 | 建除 | 协纪 | 协纪等第 | 董公 | 协纪凶神 | 鳌头起例所值 | 备注 |")
        p("|---|---|---|---|---|---|---|---|---|---|---|")
        for r in [r for r in out if r["表"] == short]:
            note = "；".join(x for x in (("重出" if r["重出"] else ""), r["鳌头注"]) if x)
            p(f"| {r['月']} | {r['干支']} | {r['等']} | {r['页']} | {r['建除']} | {r['协纪']} | {r['协纪等第']} | {r['董公']} | {r['协纪凶神']} | {r['鳌头起例所值']} | {note} |")
        p("")
    with open(DOC, "w", encoding="utf-8") as f:
        f.write("\n".join(L))
    # 供文档引用的摘要
    print("\n".join(L[:60]))


if __name__ == "__main__":
    main()
