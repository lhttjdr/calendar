"""秦历合朔相位逐条检验：秦王政元年至二世元年（前 246—前 208）出土与传世纪日对「秦历平朔 = 颛顼历平朔 + δ」的约束。

问题（INVENTORY「秦统一前合朔相位变动之年」）：睡虎地秦简秦王政二十年两条要求 δ ∈ [−118, +264] 分（1/940 日），
统一后秦历推步 `calendar::qinhan_shiyong` 两段 δ ≥ 429 分，张家山《奏谳书》「六年八月丙子朔」要求 δ ≥ 553 分。
本脚本把三表所录纪日合读，逐条求 δ 窗口，按文书性质分两组（官文书 / 私记与购藏，见 group()）、按年按地排出可行域，并求「分段常数 δ」至少要几段、段界可在哪些年。

底本：
- data/calendar/qin_phase_records.csv：《奏谳书》案例十七、十八、二十二，《吕氏春秋·序意》，《里耶秦簡（壹）索引稿》曆日索引（秦王政二十五年至始皇三十五年；
  逐条对《里耶秦简牍校释》第一卷核过，「核对」列：校释合 / 校释异 / 未核 / 校释补收），
  《嶽麓書院藏秦簡（叁）》《為獄等狀四種》（秦王政十八年至始皇二十八年）；
- data/calendar/shuihudi_records.csv：睡虎地《编年记》《语书》（国 = 秦）；
- data/calendar/qinhan_shiwu_months.csv：岳麓《质日》、周家台《历谱》（年 ≤ −208；里耶 12 条已含于索引稿，不重读）。

推步：颛顼历（与 rust/core/src/calendar/zhuanxu.rs、shuihudi_check.py 同法：历元前 366 年正月甲寅朔旦立春 JDN 1587781 夜半，
朔策 27759/940，冬至在十一月、年终置后九月）；δ 作用于合朔时刻，月序、置闰不变（同 `qinhan_shiyong`）。δ 扫描 −940 至 +1880 分。
某条之窗口：书朔者，该月朔日干支相合；书日干支者，该日落在该月之内（月首至下月朔前一日）。

输出 data/calendar/qin_phase_check.csv（逐条）与 doc/calendar/qin-phase-checks.md（覆盖写）。
用法（仓库根）：python3 scripts/calendar_data/qin_phase_check.py
"""
import csv
import math
import sys
from collections import OrderedDict, defaultdict
from fractions import Fraction as Fr
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))
import shiwu_fenlei as fl  # noqa: E402
GZ = ["甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12] for i in range(60)]

_L, _S, _E, _Y0 = Fr(27759, 940), Fr(1461, 4), Fr(1587781), -365
D_LO, D_HI = -940, 1880
# 本项目秦历推步段（rust/core/src/calendar/qinhan_shiyong.rs SEGMENTS 第一段，2026-11-26 由两段合一）：(起年, 止年, lo, hi, δ)；
# 旧两段（−230—−216 [429, 533) 取 480；−215 起 [639, 650) 取 644）留作对照，见文档第四节
SHIYONG = [(-230, -202, 639, 649, 644)]
SHIYONG_OLD = [(-230, -216, 429, 532, 480), (-215, -202, 639, 649, 644)]
CENTRAL = (638, 649)  # 下文求得的单段可行域，写进文档时由计算结果覆盖校验


def gz(j):
    return GZ[(j + 49) % 60]


def shiyue_index(Q):
    return math.floor(((Q - _Y0) * _S - 3 * _S / 24) / _L) - 1


def shuo(i, k=0):
    return math.floor(_E + i * _L + Fr(k, 940))


def month_index(Q, m, leap):
    a, b = shiyue_index(Q), shiyue_index(Q + 1)
    if leap:
        return a + 12 if b - a == 13 else None
    return a + (m - 10) % 12


def window(Q, m, leap, sh, day):
    """合此条之 δ 集合（分），返回区间列表。"""
    i = month_index(Q, m, leap)
    if i is None:
        return []
    ok = []
    for k in range(D_LO, D_HI + 1):
        s, n = shuo(i, k), shuo(i + 1, k)
        if sh and gz(s) != sh:
            continue
        if day and not any(gz(d) == day for d in range(s, n)):
            continue
        ok.append(k)
    iv = []
    for k in ok:
        if iv and k == iv[-1][1] + 1:
            iv[-1][1] = k
        else:
            iv.append([k, k])
    return iv


def hours(k):
    return f"{k * 24 / 940:+.1f}"


def fmt_iv(iv):
    if not iv:
        return "空"
    return "、".join(
        f"[{'−∞' if a == D_LO else a}, {'+∞' if b == D_HI else b}]" for a, b in iv
    )


def hull(iv):
    return (iv[0][0], iv[-1][1]) if iv else None


def inter(a, b):
    if a is None or b is None:
        return None
    lo, hi = max(a[0], b[0]), min(a[1], b[1])
    return (lo, hi) if lo <= hi else None


def in_iv(iv, k):
    return any(a <= k <= b for a, b in iv)


def yname(Q):
    return f"前 {1 - Q}"


# ---------- 读三表 ----------
def read(path):
    with open(path, encoding="utf-8") as f:
        return list(csv.DictReader(l for l in f if not l.startswith("#")))


recs = []
for r in read(ROOT / "data/calendar/qin_phase_records.csv"):
    recs.append(dict(来源=r["来源"], 地=r["地"], 纪年=r["纪年"], 年=int(r["年"]), 月=int(r["月"]), 闰=r["闰"] == "1",
                     朔=r["朔"], 日=r["日"], 依据=r["依据"], 系年=r["系年"], 简号=r["简号"], 书页=r["书页"],
                     原文=r["原文"], 注=r["注"], 核对=r["核对"]))
for r in read(ROOT / "data/calendar/shuihudi_records.csv"):
    if r["国"] != "秦":
        continue
    recs.append(dict(来源="睡虎地", 地="南郡", 纪年=r["纪年"], 年=int(r["年"]), 月=int(r["月"]), 闰=r["闰"] == "1",
                     朔=r["朔"], 日=r["干支"], 依据="书", 系年="书", 简号=f"{r['篇']} {r['简号']}", 书页=r["书页"],
                     原文=f"「{r['原文']}」", 注=r["注"]))
for r in read(ROOT / "data/calendar/qinhan_shiwu_months.csv"):
    if r["来源"] not in ("岳麓", "周家台") or int(r["年"]) > -208:
        continue
    recs.append(dict(来源=r["来源"], 地="南郡" if r["来源"] == "周家台" else "不明（购藏）", 纪年=r["纪年"], 年=int(r["年"]),
                     月=int(r["月"]), 闰=r["闰"] == "1", 朔=r["朔干支"], 日="", 依据=r["依据"] if r["依据"] != "推" else "书",
                     系年="书", 简号=r["简号"], 书页=r["书页"], 原文=r["注"].split("；")[0], 注=""))
recs.sort(key=lambda r: (r["年"], (r["月"] - 10) % 12 + (0.5 if r["闰"] else 0)))

for r in recs:
    r["iv"] = window(r["年"], r["月"], r["闰"], r["朔"] or None, r["日"] or None)
    r["入交集"] = r["依据"] == "书" and r["系年"] in ("书", "同牍")
    r["有限"] = bool(r["iv"]) and hull(r["iv"]) != (D_LO, D_HI)


GROUPS = ("官文书", "私记与购藏")


def group(r):
    """按文书性质分组（地点另列，不作分组依据）：
    官文书 = 各级官府行用、经手之文书（里耶、《奏谳书》、《为狱等状》各郡县奏谳与郡报、睡虎地《语书》），
    《吕氏春秋·序意》为秦廷所作、记中央历日，亦归此组；
    私记与购藏 = 个人记事与私人历谱（睡虎地《编年记》、岳麓《质日》（购藏，出土地不明）、周家台《历谱》）。"""
    if r["来源"] in ("岳麓", "周家台"):
        return "私记与购藏"
    if r["来源"] == "睡虎地" and r["简号"].startswith("编年记"):
        return "私记与购藏"
    return "官文书"


# ---------- 按年、按组之交集 ----------
def year_windows(rows):
    out = OrderedDict()
    for r in rows:
        h = hull(r["iv"])
        key = r["年"]
        out[key] = inter(out[key], h) if key in out else h
    return out


def segments(rows):
    """分段常数 δ：按年序贪心，求最少段数及每个段界可在的年份范围（前向贪心给最晚段界，后向给最早）。"""
    by_year = year_windows(rows)
    years = sorted(by_year)
    if any(by_year[y] is None for y in years):
        return None, [y for y in years if by_year[y] is None]

    def greedy(seq):
        segs, cur, start = [], (D_LO, D_HI), None
        for y in seq:
            nxt = inter(cur, by_year[y])
            if nxt is None:
                segs.append((start, prev, cur))
                cur, start = by_year[y], y
            else:
                cur = nxt
                start = y if start is None else start
            prev = y
        segs.append((start, prev, cur))
        return segs

    fw = greedy(years)
    bw = greedy(list(reversed(years)))
    return fw, bw


used = [r for r in recs if r["入交集"]]
groups = OrderedDict((g, [r for r in used if group(r) == g]) for g in GROUPS)
OFF = groups["官文书"]
PRIV = groups["私记与购藏"]


def all_inter(rows):
    cur = (D_LO, D_HI)
    for r in rows:
        cur = inter(cur, hull(r["iv"]))
    return cur


central = all_inter(OFF)
central_pre = all_inter([r for r in OFF if r["年"] <= -221])
central_yuelu = all_inter(OFF + [r for r in PRIV if r["来源"] == "岳麓" and r["年"] >= -212])
OFF_JS = [r for r in OFF if r["来源"] != "里耶" or r.get("核对") == "校释合"]
central_js = all_inter(OFF_JS)
LIYE_K = OrderedDict((k, sum(1 for r in recs if r["来源"] == "里耶" and r.get("核对") == k)) for k in ("校释合", "校释异", "未核", "校释补收"))
assert central == CENTRAL, central


def wcell(w):
    return "空（自相矛盾）" if w is None else f"[{'−∞' if w[0] == D_LO else w[0]}, {'+∞' if w[1] == D_HI else w[1]}]"


def by_place(rows):
    out = OrderedDict()
    for r in sorted(rows, key=lambda r: r["地"]):
        out.setdefault(r["地"], []).append(r)
    return out


def conflicts(rows, w):
    return [r for r in rows if not any(inter((a, b), w) for a, b in r["iv"])]


# ---------- 输出 CSV ----------
with open(ROOT / "data/calendar/qin_phase_check.csv", "w", encoding="utf-8", newline="") as f:
    f.write("# 秦历合朔相位逐条窗口（scripts/calendar_data/qin_phase_check.py 生成）。δ 单位：分（1/940 日），秦历平朔 = 颛顼历平朔 + δ；"
            f"窗口为合此条之 δ 区间（扫描 {D_LO}—{D_HI}，端点即扫描边界者表示不限）。入交集：依据为书且系年为书或同牍。"
            f"合单段：δ ∈ [{central[0]}, {central[1]}]（官文书各条之交）内是否相合。\n")
    w = csv.writer(f, lineterminator="\n")
    w.writerow(["来源", "组", "地", "纪年", "年", "月", "闰", "朔", "日", "依据", "系年", "简号", "书页", "核对", "颛顼朔", "窗口", "入交集", "合单段", "合颛顼本术"])
    for r in recs:
        i = month_index(r["年"], r["月"], r["闰"])
        w.writerow([r["来源"], group(r), r["地"], r["纪年"], r["年"], r["月"], int(r["闰"]), r["朔"], r["日"], r["依据"], r["系年"],
                    r["简号"], r["书页"], r.get("核对", ""), gz(shuo(i)) if i is not None else "", fmt_iv(r["iv"]), int(r["入交集"]),
                    "✓" if any(inter((a, b), central) for a, b in r["iv"]) else "✗", "✓" if in_iv(r["iv"], 0) else "✗"])

# ---------- 输出文档 ----------
L = []
P = L.append
P("# 秦历合朔相位逐条检验（自动生成）")
P("")
P("由 `scripts/calendar_data/qin_phase_check.py` 生成，勿手改。说明见 [zhanguo-calendars.md](./zhanguo-calendars.md)「秦王政至二世的合朔相位」、"
  "[qin-han-calendar-evaluation.md](./qin-han-calendar-evaluation.md) 第七节。")
P("")
P("设秦历平朔 = 颛顼历平朔 + δ（δ 以 1/940 日即「分」计，月序、置闰同颛顼历），逐条求合的 δ 区间。底本：`data/calendar/qin_phase_records.csv`"
  "（《奏谳书》、《吕氏春秋·序意》、《里耶秦簡（壹）索引稿》、《嶽麓書院藏秦簡（叁）·為獄等狀四種》）、`shuihudi_records.csv`（睡虎地）、`qinhan_shiwu_months.csv`（岳麓《质日》、周家台《历谱》）。"
  "「入交集」只取依据为书（无补释字）且系年为原文明书或同牍承上之条。")
P("")
P("## 一、结论数字")
P("")
P(f"- 官文书（《奏谳书》案例十七、十八、二十二，《序意》，里耶，《为狱等状》各郡县文书，睡虎地《语书》）各条之交："
  f"**δ ∈ [{central[0]}, {central[1]}] 分**（{hours(central[0])}—{hours(central[1])} 时），"
  f"共 {len(OFF)} 条，自秦王政元年至始皇三十五年无一条不合；其中统一前（年 ≤ −221）{sum(1 for r in OFF if r['年'] <= -221)} 条之交为 "
  f"[{central_pre[0]}, {central_pre[1]}]。按地：" + "；".join(
      f"{d} {len(rs)} 条 {wcell(all_inter(rs))}" for d, rs in by_place(OFF).items()) + "。")
P(f"- 加入岳麓《质日》三十四、三十五年：[{central_yuelu[0]}, {central_yuelu[1]}]。")
P(f"- 里耶对《里耶秦简牍校释》第一卷逐条核对：" + "、".join(f"{k} {v}" for k, v in LIYE_K.items()) + " 条；"
  f"官文书中里耶只取「校释合」之条（{len(OFF_JS)} 条）之交：[{central_js[0]}, {central_js[1]}]。")
bnj = [r for r in PRIV if r["来源"] == "睡虎地"]
COMBOS = (("官文书", OFF), ("官文书 + 睡虎地《编年记》", OFF + bnj),
          ("官文书 + 睡虎地《编年记》（去廿年七月甲寅）", OFF + [r for r in bnj if not (r["年"] == -226 and r["月"] == 7)]),
          ("官文书 + 岳麓《质日》", OFF + [r for r in PRIV if r["来源"] == "岳麓"]),
          ("官文书 + 周家台《历谱》", OFF + [r for r in PRIV if r["来源"] == "周家台"]),
          ("全部入交集之条", used))
P("")
P("按年序分段、段内 δ 为常数，最少几段（前向贪心给出各段界的最晚位置，后向贪心给出最早位置；同一年内不同来源已互相矛盾者无解）：")
P("")
for name, rows in COMBOS:
    fw, bw = segments(rows)
    if fw is None:
        P(f"- {name}：同一年内已互相矛盾（{('、'.join(yname(y) for y in bw))}），分段常数 δ 无解。")
        continue
    P(f"- {name}：至少 {len(fw)} 段。前向贪心各段：" + "；".join(f"{yname(a)}—{yname(b)} [{w[0]}, {w[1]}]" for a, b, w in fw)
      + "。后向贪心各段：" + "；".join(f"{yname(b)}—{yname(a)} [{w[0]}, {w[1]}]" for a, b, w in reversed(bw)) + "。")
P("")
P("## 二、与单段 δ 不合之条")
P("")
P(f"以官文书之交 [{central[0]}, {central[1]}] 为准，入交集之条中不合者：")
P("")
P("| 来源 | 组 | 地 | 纪年 | 月 | 朔 / 日 | 简号 | 书页 | 窗口（分） |")
P("|---|---|---|---|---|---|---|---|---|")
for r in conflicts(used, central):
    P(f"| {r['来源']} | {group(r)} | {r['地']} | {r['纪年']} | {'後' if r['闰'] else ''}{r['月']}月 | {r['朔'] + '朔' if r['朔'] else ''}{r['日']} | {r['简号']} | {r['书页']} | {fmt_iv(r['iv'])} |")
P("")
P("## 三、按年、按组、按地之可行域（入交集之条）")
P("")
P("每年先列该年两组之交，再按地分行（地为文书所出或所涉之地）。")
P("")
P("| 年 | 地 | 官文书 | 私记与购藏 |")
P("|---|---|---|---|")


for y in sorted({r["年"] for r in used}):
    ry = [r for r in used if r["年"] == y]
    cells = {g: [r for r in ry if group(r) == g] for g in GROUPS}
    P(f"| {yname(y)}（{y}） | （合计） | " + " | ".join(wcell(all_inter(cells[g])) if cells[g] else "" for g in GROUPS) + " |")
    for d, rs in by_place(ry).items():
        P(f"| | {d} | " + " | ".join(
            wcell(all_inter([r for r in rs if group(r) == g])) if any(group(r) == g for r in rs) else "" for g in GROUPS) + " |")
P("")
P("## 四、本项目推步段与里耶全部纪日")
P("")
P("`qinhan_shiyong` 秦段所取 δ 与里耶入交集之条的不合者（现行一段；旧两段在 2026-11-26 合一，列出以见其由）：")
P("")
P("| | 段 | 段内年 | δ | 不合之条 |")
P("|---|---|---|---|---|")
for tag, a, b, lo, hi, d in [("现行", *x) for x in SHIYONG] + [("旧", *x) for x in SHIYONG_OLD]:
    bad = [r for r in used if r["来源"] == "里耶" and a <= r["年"] <= b and not in_iv(r["iv"], d)]
    P(f"| {tag} | [{lo}, {hi + 1}) | {yname(a)}—{yname(b)} | {d} | " + ("；".join(
        f"{r['纪年']}{r['月']}月{r['朔'] + '朔' if r['朔'] else ''}{r['日']}（{r['简号']}，需 {fmt_iv(r['iv'])}）" for r in bad) or "无") + " |")
P("")
P("## 五、逐条（有限窗口者）")
P("")
P("只列窗口不是全域的条目（日干支距月首、月尾都远的条目对 δ 无约束，见 `data/calendar/qin_phase_check.csv`）。合单段：δ 在官文书之交内；合颛顼：δ = 0。")
P("")
P("| 来源 | 组 | 地 | 纪年 | 年 | 月 | 朔 / 日 | 依据 | 系年 | 简号 | 书页 | 核对 | 颛顼朔 | 窗口（分） | 入交集 | 合单段 | 合颛顼 |")
P("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")
n_lim = 0
for r in recs:
    if not r["有限"] and r["iv"]:
        continue
    n_lim += 1
    i = month_index(r["年"], r["月"], r["闰"])
    P(f"| {r['来源']} | {group(r)} | {r['地']} | {r['纪年']} | {r['年']} | {'後' if r['闰'] else ''}{r['月']}月 | {r['朔'] + '朔' if r['朔'] else ''}{r['日']} | "
      f"{r['依据']} | {r['系年']} | {r['简号']} | {r['书页']} | {r.get('核对', '')} | {gz(shuo(i))} | {fmt_iv(r['iv'])} | {'✓' if r['入交集'] else ''} | "
      f"{'✓' if any(inter((a, b), central) for a, b in r['iv']) else '✗'} | {'✓' if in_iv(r['iv'], 0) else '✗'} |")
P("")
P(f"共 {len(recs)} 条（有限窗口 {n_lim} 条；入交集 {len(used)} 条：官文书 {len(OFF)}、私记与购藏 {len(PRIV)}）。")
P("")
# ---------- 分类（按月） ----------
D_SHI = SHIYONG[0][4]


def grade(r):
    """A 历谱、质日；B 官文书与睡虎地《编年记》；《吕氏春秋·序意》为传世书，记 C。"""
    if r["来源"] in ("岳麓", "周家台"):
        return "A"
    return "C" if r["来源"] == "吕氏春秋" else "B"


months = defaultdict(list)
for r in used:
    months[(r["年"], r["月"], r["闰"])].append(r)
cats, odd = [], []
for k, rs in sorted(months.items(), key=lambda kv: (kv[0][0], (kv[0][1] - 10) % 12 + (0.5 if kv[0][2] else 0))):
    i = month_index(*k)
    if i is None:
        continue
    same = shuo(i, D_SHI) == shuo(i)
    p = [r for r in rs if in_iv(r["iv"], D_SHI) and not in_iv(r["iv"], 0)]
    z = [r for r in rs if in_iv(r["iv"], 0) and not in_iv(r["iv"], D_SHI)]
    b = [r for r in rs if in_iv(r["iv"], 0) and in_iv(r["iv"], D_SHI)]
    x = [r for r in rs if not in_iv(r["iv"], 0) and not in_iv(r["iv"], D_SHI)]
    g = min(grade(r) for r in rs)
    if same:  # 朔同：只合一家之纪日判的是次月朔，于本月不作证
        if b:
            c, note, g = "有证", (f"另 {len(x)} 条两家皆不合" if x else ""), min(grade(r) for r in b)
        elif x:
            c, note, g = "同误", "两家皆不合，待核", min(grade(r) for r in x)
        else:
            c, note, g = "无证", "纪日只判次月朔", "—"
    elif p and z:
        c, note = "③", f"证据冲突：只合推步 {len(p)}、只合颛顼 {len(z)}"
    elif p:
        c, note, g = "①", (f"另 {len(x)} 条两家皆不合" if x else ""), min(grade(r) for r in p)
    elif z:
        c, note, g = "②", (f"另 {len(x)} 条两家皆不合" if x else ""), min(grade(r) for r in z)
    else:
        c, note, g = "③", ("两家皆不合" if x else "纪日落在两家月内重合之日，不能判"), "—"
    cats.append((c, g))
    if c not in ("有证", "①", "无证"):
        y, m, lp = k
        P_ = rs[0]
        odd.append(f"| {P_['纪年']} | {'後' if lp else ''}{m}月 | {gz(shuo(i, D_SHI))} | {gz(shuo(i))} | "
                   + "；".join(f"{r['来源']} {r['朔'] + '朔' if r['朔'] else ''}{r['日']}" for r in rs[:4]) + ("……" if len(rs) > 4 else "")
                   + f" | {c}{f'（{note}）' if note else ''} | {g} |")
P("## 六、分类（按月）")
P("")
P(f"此段无张表；分类对象是本项目实用历推步（δ = {D_SHI}，「推步」）与颛顼本术（δ = 0，「张表」位），限入交集之条。"
  "某月两家朔日同者为相同，否则为互异。书朔之条看朔干支；书日干支之条看是否落在该家此月之内，落在两家此月重合之日者不能判"
  "（朔同之月，只合一家的纪日判的是次月之朔，于本月不作证）。"
  "等级：历谱、质日 A；官文书与睡虎地《编年记》B；《吕氏春秋·序意》C。当时文字一条即可判，同月冲突归③。"
  "注意实用推步的 δ 本由官文书定出，「①」对颛顼本术是检验，对实用推步不是独立检验。" + fl.RULE.replace("张表误", "颛顼本术误"))
P("")
P(f"有入交集之条之月 {len(cats)}：" + fl.headline(cats, "实用推步").replace("与张表", "与颛顼本术").replace("证张表误", "证颛顼本术误") + "。")
P("")
for line in fl.grade_table(cats):
    P(line.replace("①证张表误", "①证颛顼误"))
P("")
P("②、③与同误之月（③中「不能判」者是纪日落在两家此月重合之日）：")
P("")
P("| 纪年 | 月 | 推步朔 | 颛顼朔 | 所记 | 分类 | 等级 |")
P("|---|---|---|---|---|---|---|")
for line in odd:
    P(line)
P("")
(ROOT / "doc/calendar/qin-phase-checks.md").write_text("\n".join(L), encoding="utf-8")
print(f"{len(recs)} 条；官文书 {central}，统一前 {central_pre}，加岳麓 34—35 年 {central_yuelu}，里耶只取校释合 {central_js}；{dict(LIYE_K)}")
for name, rows in COMBOS:
    print(name, segments(rows))
