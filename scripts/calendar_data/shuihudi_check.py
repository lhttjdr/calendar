"""睡虎地秦简纪日检验秦（颛顼历）与魏（夏历）用历，并给出秦历合朔相位窗口。

底本：data/calendar/shuihudi_records.csv（《睡虎地秦墓竹简》文物出版社 1990，《编年记》《语书》《为吏之道》附魏律，逐条对书页转录）。

1. 秦 8 条（昭王四十五年至秦王政二十七年，前 262—前 220）：
   - 颛顼历按术推步（十月岁首、冬至在十一月、年终置后九月；与 rust/core/src/calendar/zhuanxu.rs 同法，脚本内以分数重算，
     并与 data/calendar/推步政权朔日表.csv「秦」逐月比对以自检）：干支是否落在所记之月；书朔者朔日是否相同；后九月是否有闰。
   - 汉传六历各按本历置闰（年终、无中气两法），月序一律换成夏正编号（秦月名即夏正月名）：月表取
     data/calendar/guliuli_months_350_300.csv（rust/core/examples/guliuli_months_export.rs 导出，前 341—前 300），
     四分术一蔀 76 年 = 940 月 = 27759 日，逐月整体平移一蔀（年 +76、JDN +27759）即得前 265—前 224 年；秦王政二十七年（−219）不在其内，记「—」。
   - 相位窗口：设秦历平朔 = 颛顼历平朔 + δ（月序、置闰同颛顼历），δ 从 −24 到 +48 时（步长 1/940 日），求每条合的 δ 范围及交集，
     标出各历 δ（同 baoshan_check.py：颛顼 0、殷 +7.8、鲁 +9.1、夏雨水历元 +9.7、周 −10.2、黄帝 −11.5、夏冬至历元 −12.8 时），
     及张培瑜等《中国古代历法》书页 249 所说「秦历朔小余比汉传颛顼历大 639—649 分」（+16.3—+16.6 时）。
2. 魏 2 条（安釐王二十五年閏再十二月丙午朔辛亥，同日两律）：各历在前 253—前 251 年的闰月及其朔日。

输出 data/calendar/shuihudi_check.csv（逐条 × 各历）与 doc/calendar/shuihudi-checks.md（覆盖写）。
用法（仓库根）：python3 scripts/calendar_data/shuihudi_check.py
"""
import csv
import math
from fractions import Fraction as Fr
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GZ = ["甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12] for i in range(60)]
GZI = {g: i for i, g in enumerate(GZ)}


def gz(j):
    return GZ[(j + 49) % 60]


def julian_date(j):
    """JDN → 儒略历日期（天文纪年）。"""
    c = j + 32082
    d = (4 * c + 3) // 1461
    e = c - 1461 * d // 4
    m = (5 * e + 2) // 153
    return d - 4800 + (m + 3) // 12, m + 3 - 12 * ((m + 3) // 12), e - (153 * m + 2) // 5 + 1


def julian_to_jdn(y, m, d):
    a = (14 - m) // 12
    yy, mm = y + 4800 - a, m + 12 * a - 3
    return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - 32083


def gregorian_to_jdn(y, m, d):
    a = (14 - m) // 12
    yy, mm = y + 4800 - a, m + 12 * a - 3
    return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - yy // 100 + yy // 400 - 32045


def yname(y):
    return f"前 {1 - y}"


# ---------- 颛顼历（与 zhuanxu.rs 同：历元前 366 年正月甲寅朔旦立春，JDN 1587781 夜半） ----------
_L, _S, _E, _Y0 = Fr(27759, 940), Fr(1461, 4), Fr(1587781), -365


def shiyue_index(Q):
    """建亥年 Q（以其正月所在之年记）十月朔的平朔序号：冬至所在之月往前一月。"""
    return math.floor(((Q - _Y0) * _S - 3 * _S / 24) / _L) - 1


def shuo(i, k=0):
    return math.floor(_E + i * _L + Fr(k, 940))


def qin_month(Q, m, leap):
    """秦岁 Q 的某月在颛顼月序中的平朔序号；后九月不存在则 None。"""
    a, b = shiyue_index(Q), shiyue_index(Q + 1)
    if leap:
        return a + 12 if b - a == 13 else None
    return a + (m - 10) % 12


# 自检：与推步政权朔日表「秦」逐月比对
n_chk = 0
for r in csv.DictReader(open(ROOT / "data/calendar/推步政权朔日表.csv", encoding="utf-8")):
    # 秦王政元年（−245）起推步政权「秦」取秦实用相位（calendar「颛顼历（秦实用相位）」，δ = 644），不是颛顼本术，不比
    if r["regime"] != "秦" or r["calendar"] != "颛顼历":
        continue
    y, gm, gd = int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"])
    Q, m, lp = int(r["lunar_year"]), int(r["lunar_month"]), r["is_leap"] == "1"
    i = qin_month(Q, m, lp)
    j = shuo(i)
    assert j in (gregorian_to_jdn(y, gm, gd), julian_to_jdn(y, gm, gd)), (r, j)
    n_chk += 1

# ---------- 汉传六历月表（导出表平移一蔀） ----------
LIU = {}
for r in csv.DictReader(open(ROOT / "data/calendar/guliuli_months_350_300.csv", encoding="utf-8")):
    key = (r["历"], r["闰法"])
    for s in (76,):
        LIU.setdefault(key, []).append((int(r["夏正年"]) + s, int(r["夏正月"]), r["闰"] == "1",
                                        int(r["朔日JDN"]) + 27759 * s // 76, int(r["下月朔JDN"]) + 27759 * s // 76))
for v in LIU.values():
    v.sort(key=lambda t: t[3])
LIU_KEYS = [k for k in LIU if k[0] != "颛顼历"]
Y_LO, Y_HI = -340 + 76, -299 + 76


def liu_month(key, xy, xm, leap=False):
    for t in LIU[key]:
        if t[0] == xy and t[1] == xm and t[2] == leap:
            return t
    return None


def liu_has_leap_in_qin_year(key, Q):
    """秦岁 Q（夏正 Q−1 年十月至 Q 年九月）内是否有闰月；返回闰月 (夏正年, 月, 朔干支) 或 None。"""
    a = liu_month(key, Q - 1, 10)
    b = liu_month(key, Q, 10)
    if not a or not b:
        return "—"
    for t in LIU[key]:
        if a[3] <= t[3] < b[3] and t[2]:
            return t
    return None


recs = [r for r in csv.DictReader(l for l in open(ROOT / "data/calendar/shuihudi_records.csv", encoding="utf-8")
                                  if not l.startswith("#"))]
qin = [r for r in recs if r["国"] == "秦"]
wei = [r for r in recs if r["国"] == "魏"]

OFF = {"颛顼历": 0.0, "殷历": 0.323, "鲁历": 0.378, "夏历（雨水历元）": 0.404, "周历": -0.426, "黄帝历": -0.480, "夏历（冬至历元）": -0.535}
ZHANG_QIN = (639 / 940, 649 / 940)
KS = range(-940, 2 * 940 + 1)


def qin_ok(r, k):
    Q, m, lp = int(r["年"]), int(r["月"]), r["闰"] == "1"
    i = qin_month(Q, m, lp)
    if i is None:
        return False
    a, b = shuo(i, k), shuo(i + 1, k)
    if r["朔"] and gz(a) != r["朔"]:
        return False
    if r["干支"]:
        return any(gz(d) == r["干支"] for d in range(a, b))
    return True


def segs(ks):
    out = []
    for k in ks:
        if out and k == out[-1][1] + 1:
            out[-1][1] = k
        else:
            out.append([k, k])
    return out


def fmt_segs(ss):
    if not ss:
        return "无"
    return "；".join(f"[{a / 940 * 24:+.1f}, {b / 940 * 24:+.1f}]（{a:+d}—{b:+d} 分）" for a, b in ss)


rows = []
md = ["# 睡虎地秦简纪日检验（自动生成）", "",
      "由 `scripts/calendar_data/shuihudi_check.py` 生成，勿手改。说明见 [zhanguo-calendars.md](./zhanguo-calendars.md)「秦、魏：睡虎地秦简检验」。", "",
      f"底本 `data/calendar/shuihudi_records.csv`（《睡虎地秦墓竹简》文物出版社 1990，书页见表）。颛顼历按术推步已与 `推步政权朔日表.csv`「秦」昭王至庄襄王 {n_chk} 个月逐月比对一致（秦王政起该表取秦实用相位，见 zhanguo-calendars.md「秦王政至二世的合朔相位」）。", "",
      "## 一、秦：颛顼历按术推步", "",
      "| 纪年 | 年 | 月 | 干支 | 简 | 书页 | 颛顼历该月朔（儒略历） | 该月日干支范围 | 干支在月中第几日 | 合 |",
      "|---|---|---|---|---|---|---|---|---|---|"]
for r in qin:
    Q, m, lp = int(r["年"]), int(r["月"]), r["闰"] == "1"
    i = qin_month(Q, m, lp)
    mname = ("后九月" if lp else f"{m}月")
    if i is None:
        md.append(f"| {r['纪年']} | {yname(Q)} | {mname} | {r['干支'] or '—'} | {r['篇']} {r['简号']} | {r['书页']} | 无后九月 | — | — | ✗ |")
        rows.append(dict(r, 历="颛顼历", 闰法="年终", 朔日=None, 结果="✗"))
        continue
    a, b = shuo(i), shuo(i + 1)
    y, mo, d = julian_date(a)
    day = ""
    if r["干支"]:
        dd = [x for x in range(a, b) if gz(x) == r["干支"]]
        day = f"{dd[0] - a + 1}" if dd else "不在月内"
    ok = qin_ok(r, 0)
    extra = f"（书朔{r['朔']}）" if r["朔"] else ""
    md.append(f"| {r['纪年']} | {yname(Q)} | {mname} | {r['干支'] or '（闰）'}{extra} | {r['篇']} {r['简号']} | {r['书页']} | "
              f"{gz(a)}（{y}-{mo:02d}-{d:02d}），{'大' if b - a == 30 else '小'}月 | {gz(a)}—{gz(b - 1)} | {day or '—'} | {'✓' if ok else '✗'} |")
    rows.append(dict(r, 历="颛顼历", 闰法="年终", 朔日=gz(a), 结果="✓" if ok else "✗"))
nq = sum(qin_ok(r, 0) for r in qin)
md += ["", f"颛顼历合 {nq}/{len(qin)}。", ""]

# 六历
md += ["## 二、秦：汉传六历各按本历置闰", "",
       f"月序换成夏正编号（秦月名即夏正月名）；后九月一条看秦岁（夏正前一年十月至本年九月）内有无闰月。月表覆盖 {yname(Y_LO)}—{yname(Y_HI)}，"
       "秦王政二十七年（前 220）不在其内（「—」）。", ""]
hdr = "| 纪年 | 月 | 干支 | " + " | ".join(f"{k[0]}（{k[1]}）" for k in LIU_KEYS) + " |"
md += [hdr, "|" + "---|" * (3 + len(LIU_KEYS))]
tot = {k: [0, 0] for k in LIU_KEYS}
for r in qin:
    Q, m, lp = int(r["年"]), int(r["月"]), r["闰"] == "1"
    cells = []
    for k in LIU_KEYS:
        sd = ""
        if lp:
            t = liu_has_leap_in_qin_year(k, Q)
            if t == "—":
                cells.append("—"); res = None
            elif t is None:
                cells.append("✗ 无闰"); res = False
            else:
                cells.append(f"✓ 有闰（{t[1]}月后）"); res = True
        else:
            xy = Q - 1 if m >= 10 else Q
            t = liu_month(k, xy, m)
            if t is None or not (Y_LO <= xy <= Y_HI):
                cells.append("—"); res = None
            else:
                a, b = t[3], t[4]
                sd = gz(a)
                res = (not r["朔"] or gz(a) == r["朔"]) and any(gz(d) == r["干支"] for d in range(a, b))
                cells.append(f"{'✓' if res else '✗'} {gz(a)}朔")
        if res is not None:
            tot[k][1] += 1
            tot[k][0] += res
        rows.append(dict(r, 历=k[0], 闰法=k[1], 朔日=sd, 结果={True: "✓", False: "✗", None: "—"}[res]))
    md.append(f"| {r['纪年']} | {'后九月' if lp else str(m) + '月'} | {r['干支'] or '—'}{('（朔' + r['朔'] + '）') if r['朔'] else ''} | " + " | ".join(cells) + " |")
md.append("| **合计** | | | " + " | ".join(f"**{a}/{b}**" for a, b in tot.values()) + " |")
md.append("")

# 相位窗口
md += ["## 三、秦：合朔相位窗口", "",
       "设秦历平朔 = 颛顼历平朔 + δ（月序、置闰同颛顼历），逐条求合的 δ 范围（时；括注日法 940 之分，即朔小余之差，闭区间）。", "",
       "| 纪年 | 月 | 干支 | δ 窗口（时） |", "|---|---|---|---|"]
allk = set(KS)
for r in qin:
    ks = [k for k in KS if qin_ok(r, k)]
    allk &= set(ks)
    md.append(f"| {r['纪年']} | {'后九月' if r['闰'] == '1' else r['月'] + '月'} | {r['干支'] or '—'}{('（朔' + r['朔'] + '）') if r['朔'] else ''} | {fmt_segs(segs(ks))} |")
inter = segs(sorted(allk))
inside = [c for c, v in OFF.items() if any(a / 940 <= v <= b / 940 for a, b in inter)]
zq = any(a / 940 <= ZHANG_QIN[0] and ZHANG_QIN[1] <= b / 940 for a, b in inter)
md += ["", f"交集：δ ∈ {fmt_segs(inter)} 时。汉传各历 δ 在内者：{'、'.join(inside) or '无'}；"
       f"张培瑜所说秦历朔小余大 639—649 分（δ = +{ZHANG_QIN[0] * 24:.1f}—+{ZHANG_QIN[1] * 24:.1f} 时）{'在内' if zq else '不在内'}。", ""]
nosuo = [r for r in qin if not r["朔"]]
allk2 = set(KS)
for r in nosuo:
    allk2 &= {k for k in KS if qin_ok(r, k)}
md += [f"去掉《语书》书朔一条（只用 {len(nosuo)} 条纪日）：δ ∈ {fmt_segs(segs(sorted(allk2)))} 时。", ""]
md += ["比对分类：此段无张表；秦八条已并入 [qin-phase-checks.md](./qin-phase-checks.md)「六、分类（按月）」，按实用历推步（δ = 644）与颛顼本术分类"
       "（《语书》秦王政二十年四月、《编年记》同年七月甲寅为 B 级当时文字；其中七月甲寅合颛顼、不合实用推步，归②）。汉传六历只作比对，不另分类。", ""]

# 魏
md += ["## 四、魏：安釐王二十五年閏再十二月丙午朔", "",
       "各历在夏正前 254—前 251 年的闰月（夏正编号，闰在某月之后）及其朔日；书「閏再十二月」即闰在十二月之后（夏历寅正年终闰）。", "",
       "| 历（闰法） | 闰月（夏正年、在几月后） | 闰月朔 | 合「闰十二月丙午朔」 |", "|---|---|---|---|"]
for k in LIU_KEYS:
    hits = [t for t in LIU[k] if t[2] and -253 <= t[0] <= -250]
    for t in hits:
        good = t[1] == 12 and t[0] == -251 and gz(t[3]) == "丙午"
        md.append(f"| {k[0]}（{k[1]}） | {yname(t[0])}，{t[1]}月后 | {gz(t[3])} | {'✓' if good else '✗'} |")
        if t[0] == -251:
            for r in wei:
                rows.append(dict(r, 历=k[0], 闰法=k[1], 朔日=gz(t[3]), 结果="✓" if good else "✗"))
    if not hits:
        md.append(f"| {k[0]}（{k[1]}） | 无 | — | ✗ |")
for Q in (-252, -251, -250):
    a, b = shiyue_index(Q), shiyue_index(Q + 1)
    if b - a == 13:
        md.append(f"| 颛顼历（后九月） | 秦岁{yname(Q)}，后九月 | {gz(shuo(a + 12))} | ✗（闰名后九月） |")
md += ["", "说明：同日两律（户律简 16 伍、奔命律简 22 伍）只算一事。", ""]

with open(ROOT / "data/calendar/shuihudi_check.csv", "w", encoding="utf-8", newline="") as f:
    f.write("# 睡虎地秦简纪日逐条 × 各历检验（scripts/calendar_data/shuihudi_check.py 生成）。结果：✓ 合、✗ 不合、— 月表不覆盖。"
            "魏律两条只列前 252 年闰月或合者。\n")
    w = csv.writer(f)
    w.writerow(["国", "纪年", "年", "月", "闰", "干支", "朔", "简号", "书页", "历", "闰法", "该月朔干支", "结果"])
    for r in rows:
        w.writerow([r["国"], r["纪年"], r["年"], r["月"], r["闰"], r["干支"], r["朔"], r["简号"], r["书页"], r["历"], r["闰法"], r["朔日"] or "", r["结果"]])
(ROOT / "doc/calendar/shuihudi-checks.md").write_text("\n".join(md), encoding="utf-8")
print("\n".join(md))
