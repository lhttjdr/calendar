"""秦历合朔相位逐条检验：秦王政元年至二世元年（前 246—前 208）出土与传世纪日对「秦历平朔 = 颛顼历平朔 + δ」的约束。

问题（INVENTORY「秦统一前合朔相位变动之年」）：睡虎地秦简秦王政二十年两条要求 δ ∈ [−118, +264] 分（1/940 日），
统一后秦历推步 `calendar::qinhan_shiyong` 两段 δ ≥ 429 分，张家山《奏谳书》「六年八月丙子朔」要求 δ ≥ 553 分。
本脚本把三表所录纪日合读，逐条求 δ 窗口，按年、按出土地排出可行域，并求「分段常数 δ」至少要几段、段界可在哪些年。

底本：
- data/calendar/qin_phase_records.csv：《奏谳书》案例十七、十八、二十二，《吕氏春秋·序意》，《里耶秦簡（壹）索引稿》曆日索引（秦王政二十五年至始皇三十五年）；
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
from collections import OrderedDict, defaultdict
from fractions import Fraction as Fr
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GZ = ["甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12] for i in range(60)]

_L, _S, _E, _Y0 = Fr(27759, 940), Fr(1461, 4), Fr(1587781), -365
D_LO, D_HI = -940, 1880
# 本项目统一后秦历推步两段（rust/core/src/calendar/qinhan_shiyong.rs SEGMENTS）与张培瑜所说 639—649 分
SHIYONG = [(-230, -216, 429, 532, 480), (-215, -202, 639, 649, 644)]
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
                     原文=r["原文"], 注=r["注"]))
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


def group(r):
    if r["来源"] in ("睡虎地", "周家台") or r["地"] == "南郡":
        return "南郡"
    if r["来源"] == "岳麓":
        return "岳麓"
    return "中央与洞庭"


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
groups = OrderedDict((g, [r for r in used if group(r) == g]) for g in ("中央与洞庭", "南郡", "岳麓"))


def all_inter(rows):
    cur = (D_LO, D_HI)
    for r in rows:
        cur = inter(cur, hull(r["iv"]))
    return cur


central = all_inter(groups["中央与洞庭"])
central_pre = all_inter([r for r in groups["中央与洞庭"] if r["年"] <= -221])
central_yuelu = all_inter(groups["中央与洞庭"] + [r for r in groups["岳麓"] if r["年"] >= -212])
assert central == CENTRAL, central


def conflicts(rows, w):
    return [r for r in rows if not any(inter((a, b), w) for a, b in r["iv"])]


# ---------- 输出 CSV ----------
with open(ROOT / "data/calendar/qin_phase_check.csv", "w", encoding="utf-8", newline="") as f:
    f.write("# 秦历合朔相位逐条窗口（scripts/calendar_data/qin_phase_check.py 生成）。δ 单位：分（1/940 日），秦历平朔 = 颛顼历平朔 + δ；"
            f"窗口为合此条之 δ 区间（扫描 {D_LO}—{D_HI}，端点即扫描边界者表示不限）。入交集：依据为书且系年为书或同牍。"
            f"合单段：δ ∈ [{central[0]}, {central[1]}]（中央与洞庭各条之交）内是否相合。\n")
    w = csv.writer(f, lineterminator="\n")
    w.writerow(["来源", "组", "纪年", "年", "月", "闰", "朔", "日", "依据", "系年", "简号", "书页", "颛顼朔", "窗口", "入交集", "合单段", "合颛顼本术"])
    for r in recs:
        i = month_index(r["年"], r["月"], r["闰"])
        w.writerow([r["来源"], group(r), r["纪年"], r["年"], r["月"], int(r["闰"]), r["朔"], r["日"], r["依据"], r["系年"],
                    r["简号"], r["书页"], gz(shuo(i)) if i is not None else "", fmt_iv(r["iv"]), int(r["入交集"]),
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
  "（《奏谳书》、《吕氏春秋·序意》、《里耶秦簡（壹）索引稿》）、`shuihudi_records.csv`（睡虎地）、`qinhan_shiwu_months.csv`（岳麓《质日》、周家台《历谱》）。"
  "「入交集」只取依据为书（无补释字）且系年为原文明书或同牍承上之条。")
P("")
P("## 一、结论数字")
P("")
P(f"- 中央与洞庭（《奏谳书》案例十七、二十二，《序意》，里耶）各条之交：**δ ∈ [{central[0]}, {central[1]}] 分**（{hours(central[0])}—{hours(central[1])} 时），"
  f"共 {len(groups['中央与洞庭'])} 条，自秦王政元年至始皇三十五年无一条不合；其中统一前（年 ≤ −221）{sum(1 for r in groups['中央与洞庭'] if r['年'] <= -221)} 条之交为 "
  f"[{central_pre[0]}, {central_pre[1]}]。")
P(f"- 加入岳麓《质日》三十四、三十五年：[{central_yuelu[0]}, {central_yuelu[1]}]。")
shd = [r for r in groups["南郡"] if r["来源"] == "睡虎地"]
COMBOS = (("中央与洞庭", groups["中央与洞庭"]), ("中央与洞庭 + 睡虎地", groups["中央与洞庭"] + shd),
          ("中央与洞庭 + 睡虎地（去《编年记》廿年七月甲寅）", groups["中央与洞庭"] + [r for r in shd if not (r["年"] == -226 and r["月"] == 7)]),
          ("中央与洞庭 + 岳麓", groups["中央与洞庭"] + groups["岳麓"]), ("中央与洞庭 + 南郡", groups["中央与洞庭"] + groups["南郡"]),
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
P(f"以中央与洞庭之交 [{central[0]}, {central[1]}] 为准，入交集之条中不合者：")
P("")
P("| 来源 | 组 | 纪年 | 月 | 朔 / 日 | 简号 | 书页 | 窗口（分） |")
P("|---|---|---|---|---|---|---|---|")
for r in conflicts(used, central):
    P(f"| {r['来源']} | {group(r)} | {r['纪年']} | {'後' if r['闰'] else ''}{r['月']}月 | {r['朔'] + '朔' if r['朔'] else ''}{r['日']} | {r['简号']} | {r['书页']} | {fmt_iv(r['iv'])} |")
P("")
P("## 三、按年、按组之可行域（入交集之条）")
P("")
P("| 年 | 中央与洞庭 | 南郡 | 岳麓 |")
P("|---|---|---|---|")
yw = {g: year_windows(rows) for g, rows in groups.items()}
for y in sorted(set().union(*[set(v) for v in yw.values()])):
    def cell(g):
        if y not in yw[g]:
            return ""
        w = yw[g][y]
        return "空（自相矛盾）" if w is None else f"[{'−∞' if w[0] == D_LO else w[0]}, {'+∞' if w[1] == D_HI else w[1]}]"
    P(f"| {yname(y)}（{y}） | {cell('中央与洞庭')} | {cell('南郡')} | {cell('岳麓')} |")
P("")
P("## 四、本项目推步段与里耶全部纪日")
P("")
P("`qinhan_shiyong` 现行两段所取 δ 与里耶（索引稿）入交集之条的不合者：")
P("")
P("| 段 | 段内年 | δ | 不合之条 |")
P("|---|---|---|---|")
for a, b, lo, hi, d in SHIYONG:
    bad = [r for r in used if r["来源"] == "里耶" and a <= r["年"] <= b and not in_iv(r["iv"], d)]
    P(f"| [{lo}, {hi + 1}) | {yname(a)}—{yname(b)} | {d} | " + ("；".join(
        f"{r['纪年']}{r['月']}月{r['朔'] + '朔' if r['朔'] else ''}{r['日']}（{r['简号']}，需 {fmt_iv(r['iv'])}）" for r in bad) or "无") + " |")
P("")
P("## 五、逐条（有限窗口者）")
P("")
P("只列窗口不是全域的条目（日干支距月首、月尾都远的条目对 δ 无约束，见 `data/calendar/qin_phase_check.csv`）。合单段：δ 在中央与洞庭之交内；合颛顼：δ = 0。")
P("")
P("| 来源 | 组 | 纪年 | 年 | 月 | 朔 / 日 | 依据 | 系年 | 简号 | 书页 | 颛顼朔 | 窗口（分） | 入交集 | 合单段 | 合颛顼 |")
P("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")
n_lim = 0
for r in recs:
    if not r["有限"] and r["iv"]:
        continue
    n_lim += 1
    i = month_index(r["年"], r["月"], r["闰"])
    P(f"| {r['来源']} | {group(r)} | {r['纪年']} | {r['年']} | {'後' if r['闰'] else ''}{r['月']}月 | {r['朔'] + '朔' if r['朔'] else ''}{r['日']} | "
      f"{r['依据']} | {r['系年']} | {r['简号']} | {r['书页']} | {gz(shuo(i))} | {fmt_iv(r['iv'])} | {'✓' if r['入交集'] else ''} | "
      f"{'✓' if any(inter((a, b), central) for a, b in r['iv']) else '✗'} | {'✓' if in_iv(r['iv'], 0) else '✗'} |")
P("")
P(f"共 {len(recs)} 条（有限窗口 {n_lim} 条；入交集 {len(used)} 条：中央与洞庭 {len(groups['中央与洞庭'])}、南郡 {len(groups['南郡'])}、岳麓 {len(groups['岳麓'])}）。")
P("")
(ROOT / "doc/calendar/qin-phase-checks.md").write_text("\n".join(L), encoding="utf-8")
print(f"{len(recs)} 条；中央与洞庭 {central}，统一前 {central_pre}，加岳麓 34—35 年 {central_yuelu}")
for name, rows in COMBOS:
    print(name, segments(rows))
