"""新蔡葛陵楚简纪日检验楚历相位：与包山楚简相位窗口（scripts/calendar_data/baoshan_check.py）互证。

底本：data/calendar/xincai_records.csv（《新蔡葛陵楚墓》附录一释文，逐条对书页转录）。
新蔡简岁名都没有公认的绝对年代（整理者定墓葬在楚悼王末年，前 381 年前后；诸家说法不一），所以这里不预设年份：
把楚历看作颛顼历月序（冬至在十一月、年终闰；冬柰 = 夏正十月）而合朔相位偏 δ（同 baoshan_check.py 的 _ten/_ok），
对前 450—前 350 年逐年、δ 从 −1 日到 +2 日（步长 2/940 日）逐格检验。

1. 组内自洽：每个岁名下各条在某一 (年, δ) 同时全合的最大条数，及最大时不合之条（同月干支跨度、相邻月衔接的问题）。
2. 王徙于鄩郢之岁（20 条，三个相连之月）：去掉组内不合之条后，逐年给出 δ 窗口，标出颛顼、殷、鲁、周等历的 δ 落在哪些年，
   以及与包山归胙之岁定出的 δ ∈ [−8.4, +28.3] 时相交的年份。

输出追加到 doc/calendar/baoshan-checks.md 之后的独立文件 doc/calendar/xincai-checks.md（覆盖写）。
用法（仓库根）：python3 scripts/calendar_data/xincai_check.py
"""
import collections
import csv
import math
from fractions import Fraction as Fr
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GZ = ["甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12] for i in range(60)]

recs = list(csv.DictReader(l for l in open(ROOT / "data/calendar/xincai_records.csv", encoding="utf-8") if not l.startswith("#")))
groups = collections.OrderedDict()
for r in recs:
    groups.setdefault(r["岁"], []).append(r)

# 颛顼历平朔（与 baoshan_check.py 相同）
_L, _S, _E, _Y0 = Fr(27759, 940), Fr(1461, 4), Fr(1587781) - Fr(1, 2), -365


def _ten(Y):
    return math.floor(((Y - _Y0) * _S - 3 * _S / 24) / _L) - 1


def month_start(Y, n, k):
    """楚岁 Y（冬柰在夏正 Y−1 年）第 n 月朔日 JDN，合朔偏 k/940 日。"""
    i = _ten(Y) + (n - 1)
    return math.floor(_E + i * _L + Fr(k, 940) + Fr(1, 2))


def ok(Y, n, g, k):
    a, b = month_start(Y, n, k), month_start(Y, n + 1, k)
    return any((d + 49) % 60 == g for d in range(a, b))


YEARS = range(-449, -348)  # 楚岁标号 Y：前 450—前 350
KS = range(-940, 2 * 940, 2)
OFF = {"颛顼历": 0.0, "殷历": 0.323, "鲁历": 0.378, "夏历（雨水历元）": 0.404, "周历": -0.426, "黄帝历": -0.480, "夏历（冬至历元）": -0.535}
BAOSHAN = (-8.4 / 24, 28.3 / 24)  # 包山归胙之岁 79 条定的 δ 窗口（日）


def yname(Y):
    return f"前 {1 - Y}"


def hours(k):
    return k / 940 * 24


md = ["# 新蔡葛陵楚简纪日检验（自动生成）", "",
      "> 由 `scripts/calendar_data/xincai_check.py` 生成，勿手改。底本 `data/calendar/xincai_records.csv`"
      "（《新蔡葛陵楚墓》附录一释文，逐条对书页转录）。方法与依据见 [zhanguo-calendars.md](zhanguo-calendars.md) 四。", "",
      "楚历模型：颛顼历月序（冬柰 = 夏正十月，年终闰），合朔相对颛顼历平朔偏 δ。新蔡岁名无公认绝对年代，"
      "故对前 450—前 350 年逐年、δ ∈ [−24, +48) 时逐格检验。", "",
      "## 逐条", "", "| 岁 | 月 | 干支 | 简号 | 书页 | 注 |", "|---|---|---|---|---|---|"]
for r in recs:
    md.append(f"| {r['岁']} | {r['月']} | {r['干支']} | {r['简号']} | {r['书页']} | {r['注']} |")

# 1. 组内自洽
md += ["", "## 组内自洽", "",
       "每个岁名下各条在同一 (年, δ) 下同时合的最大条数；最大时仍不合之条列出（若不合之条不止一种组合，列出全部）。", "",
       "| 岁 | 条数 | 最多同时合 | 不合之条 |", "|---|---|---|---|"]
keep = {}
for key, rs in groups.items():
    items = [(int(r["月序"]), GZ.index(r["干支"])) for r in rs]
    best, fails = -1, set()
    for Y in YEARS:
        for k in KS:
            hit = tuple(ok(Y, n, g, k) for n, g in items)
            c = sum(hit)
            if c > best:
                best, fails = c, {hit}
            elif c == best:
                fails.add(hit)
    bad = sorted({"、".join(f"{rs[i]['月']}{rs[i]['干支']}（{rs[i]['简号']}）" for i, h in enumerate(f) if not h) or "无" for f in fails})
    md.append(f"| {key} | {len(rs)} | {best} | {'；或 '.join(bad)} |")
    # 唯一不合组合时，去掉不合之条供下一步
    if len(fails) == 1:
        f = next(iter(fails))
        keep[key] = [rs[i] for i, h in enumerate(f) if h]
    else:
        keep[key] = rs
    print(key, len(rs), best, bad)

# 2. 各岁逐年 δ 窗口（去掉组内不合之条后）
md += ["", "## 逐年相位窗口", "",
       "去掉组内不合之条后，各岁在每一候选年全合的 δ 范围（时）。窗口不连续时分段列出。"
       "「包山」列：与包山归胙之岁 δ ∈ [−8.4, +28.3] 时有交集者打 ✓。各历 δ：颛顼 0、殷 +7.8、鲁 +9.1、夏（雨水历元）+9.7、"
       "周 −10.2、黄帝 −11.5、夏（冬至历元）−12.8 时。", ""]
summary = []
for key, rs in keep.items():
    items = [(int(r["月序"]), GZ.index(r["干支"])) for r in rs]
    md += [f"### {key}（{len(rs)} 条）", "", "| 年 | δ 窗口（时） | 窗口内之历 | 包山 |", "|---|---|---|---|"]
    yrs = []
    for Y in YEARS:
        good = [k for k in KS if all(ok(Y, n, g, k) for n, g in items)]
        if not good:
            continue
        segs, s0, p = [], good[0], good[0]
        for k in good[1:]:
            if k != p + 2:
                segs.append((s0, p)); s0 = k
            p = k
        segs.append((s0, p))
        cals = [c for c, v in OFF.items() if any(a / 940 <= v <= b / 940 for a, b in segs)]
        inter = any(a / 940 <= BAOSHAN[1] and b / 940 >= BAOSHAN[0] for a, b in segs)
        yrs.append((Y, segs, cals, inter))
        md.append(f"| {yname(Y)} | {'、'.join(f'[{hours(a):+.1f}, {hours(b):+.1f}]' for a, b in segs)} | {'、'.join(cals) or '—'} | {'✓' if inter else ''} |")
    n_all = len(yrs)
    n_bs = sum(1 for y in yrs if y[3])
    zx = [yname(y[0]) for y in yrs if "颛顼历" in y[2]]
    yin = [yname(y[0]) for y in yrs if "殷历" in y[2]]
    md += ["", f"共 {n_all} 年可合（{len(YEARS)} 年中），与包山窗口相交 {n_bs} 年；颛顼历合于 {'、'.join(zx) or '无'}；殷历合于 {'、'.join(yin) or '无'}。", ""]
    summary.append((key, len(rs), n_all, n_bs, zx, yin))
    print(key, n_all, n_bs, "颛顼", zx, "殷", yin)

(ROOT / "doc/calendar/xincai-checks.md").write_text("\n".join(md) + "\n", encoding="utf-8")

# 3. 独立系年：宋华强《新蔡葛陵楚简初探》（武汉大学出版社 2010）第三章（书页 120—135），不用历日的两条前提：
#    (a) 「大莫敖阳为晋师战于长城之岁」= 前 403 年（李学勤以羌钟、古本《竹书纪年》晋烈公十二年系之于前 404，
#        宋华强据楚简以前一年之事纪年改为前 403），且王徙于鄩郢之岁是九岁中最晚者（整理者据病情记录）→ 不早于前 403；
#    (b) 祭祷先王止于声王、不见悼王，悼王时尚在世（宋华强书页 124—125）→ 不晚于前 381（悼王卒年）。
#    宋氏另据世系与骨骼年龄推前 401—前 395，范围更窄、前提更多，只作参照。
#    宋氏又把无岁名的零 200、323「夏柰之月丙辰之日」归入此岁（书页 122—123），作为附加之条另算。
md2 = ["", "## 独立系年下的王徙于鄩郢之岁", "",
       "前提（不用历日）：宋华强《新蔡葛陵楚简初探》（2010）第三章——(a) 长城之岁 = 前 403 年且王徙之岁为诸岁最晚 → 不早于前 403；"
       "(b) 祭祷先王止于声王、悼王尚在 → 不晚于前 381。宋氏据世系与骨骼年龄更推前 401—前 395，只作参照。"
       "刘信芳、李学勤定前 377、刘彬徽定前 398，都是拿历日查历表所得，此处不用（会循环）。", "",
       "| 年 | 条件 | δ 窗口（时） | 窗口内之历 |", "|---|---|---|---|"]
wx = [(int(r["月序"]), GZ.index(r["干支"])) for r in keep["王徙于鄩郢之岁"]]
extra = (7, GZ.index("丙辰"))
adm = []
for Y in range(1 - 403, 1 - 381 + 1):
    for label, items in (("19 条", wx), ("19 条 + 零 200、323 夏柰丙辰", wx + [extra])):
        good = [k for k in KS if all(ok(Y, n, g, k) for n, g in items)]
        if good:
            cals = [c for c, v in OFF.items() if good[0] / 940 <= v <= good[-1] / 940]
            md2.append(f"| {yname(Y)} | {label} | [{hours(good[0]):+.1f}, {hours(good[-1]):+.1f}] | {'、'.join(cals) or '—'} |")
            if label == "19 条":
                adm.append((Y, good, cals))
# 与包山强层（归胙之岁 δ ∈ [−8.4, +28.3] 时）合
both = [(Y, max(good[0] / 940, BAOSHAN[0]), min(good[-1] / 940, BAOSHAN[1])) for Y, good, _ in adm
        if good[-1] / 940 >= BAOSHAN[0] and good[0] / 940 <= BAOSHAN[1]]
md2 += ["", "前 403—前 381 年间可合者只上表诸年；与包山归胙之岁 79 条定的 δ ∈ [−8.4, +28.3] 时相交者只：" +
        "；".join(f"{yname(Y)} δ ∈ [{a*24:+.1f}, {b*24:+.1f}] 时，在内者 {'、'.join(c for c, v in OFF.items() if a <= v <= b) or '无'}" for Y, a, b in both) + "。", ""]
with open(ROOT / "doc/calendar/xincai-checks.md", "a", encoding="utf-8") as f:
    f.write("\n".join(md2) + "\n")
print("独立系年", [(yname(Y), c) for Y, _, c in adm], both)
