"""包山楚简纪日检验楚历：汉传六历、颛顼历在「楚正月（冬柰）当夏正某月」各假设下，简文干支是否落在所记之月。

底本：维基文库《包山楚簡》录文（缓存 hanshu_cache/lizhi/baoshan.txt，不入库；缺时自动下载）。录文楚月名多用替代字与私用区字：
冬/屈/遠/夏 + U+E12E = 冬柰、屈柰、远柰、夏柰（柰即「夕」）；「峣昺」= 荆𡰥（刑夷）；「夏昺」= 夏𡰥（夏杘）；「亯月」= 享月（纺月）；
U+E268 + 月 = 爨月。楚月序依睡虎地秦简《日书》甲种「岁」篇秦楚月名对照：冬夕、屈夕、援夕、刑夷、夏杘、纺月、七月（包山作夏柰）、
八月、九月、十月、爨月、献马，冬夕当秦十月（即夏正十月）。

只取「月名 + 干支之日」明书相连的纪日；「受期」后之限期日（月份不明）不取。月名前 60 字内有「某某之歲」者记入该岁。
各岁年份按《包山楚墓》系年（转引，未核原书）：卲阳败晋师于襄陵之岁前 322，齐客陈豫贺王之岁前 321，鲁阳公以楚师后城郑之岁前 320，
宋客盛公边聘于楚之岁前 318，东周之客许缊归胙于栽郢之岁前 317，大司马悼滑救郙之岁前 316；另以整体平移 −2—+2 年检验系年。
楚岁以冬柰为首（与秦十月岁首同），岁 Y 之冬柰在夏正 Y−1 年。

六历月表由 rust/core/examples/guliuli_months_export.rs 导出（data/calendar/guliuli_months_350_300.csv）。
输出 data/calendar/baoshan_records.csv（抽出的纪日）、data/calendar/baoshan_check.csv（各假设合数）。
用法（仓库根）：python3 scripts/calendar_data/baoshan_check.py
"""
import collections
import csv
import re
import time
import urllib.parse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CACHE = ROOT / "hanshu_cache/lizhi/baoshan.txt"
if not CACHE.exists():
    CACHE.parent.mkdir(parents=True, exist_ok=True)
    url = "https://zh.wikisource.org/w/index.php?title=" + urllib.parse.quote("包山楚簡") + "&action=raw"
    req = urllib.request.Request(url, headers={"User-Agent": "calendar-research-script/0.1"})
    CACHE.write_bytes(urllib.request.urlopen(req, timeout=60).read())
    time.sleep(1)

GZ = ["甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12] for i in range(60)]
t = CACHE.read_text(encoding="utf-8")
s = re.sub(r"\{\{PUA\|(.)\}\}", lambda m: "⟨%04X⟩" % ord(m.group(1)), t)
s = re.sub(r"\{\{--\|\(([^)]*)\)\}\}", r"(\1)", s)
s = re.sub(r"\{\{參\|[^|}]*\|([^}]*)\}\}", r"\1", s)
s = re.sub(r"\{\{\*\|(\d+[反]?)\}\}", r"#\1#", s)
s = s.replace("癸梄(酉)", "癸酉")

# 楚月名 → 楚月序（冬柰 = 1）
MONTHS = [
    (r"冬⟨E12E⟩之月", 1, "冬柰"), (r"屈⟨E12E⟩之月", 2, "屈柰"), (r"遠⟨E12E⟩之月", 3, "远柰"),
    (r"峣昺之月", 4, "荆𡰥"), (r"夏昺之月", 5, "夏𡰥"), (r"亯月", 6, "享月"), (r"夏⟨E12E⟩之月", 7, "夏柰"),
    (r"八月", 8, "八月"), (r"九月", 9, "九月"), (r"十月", 10, "十月"), (r"⟨E268⟩月", 11, "爨月"), (r"獻馬之月", 12, "献马"),
]
SUI = [
    ("敗晉帀(師)於襄陵之⟨E1FF⟩(歲)", "卲阳败晋师于襄陵之岁", -321),
    ("陳豫𧦤王之⟨E1FF⟩(歲)", "齐客陈豫贺王之岁", -320),
    ("後城奠(鄭)之⟨E1FF⟩(歲)", "鲁阳公以楚师后城郑之岁", -319),
    ("盛公臱", "宋客盛公边聘于楚之岁", -317),
    ("於栽郢之⟨E1FF⟩(歲)", "东周之客许缊归胙于栽郢之岁", -316),
    ("於𦳦(栽)郢之⟨E1FF⟩(歲)", "东周之客许缊归胙于栽郢之岁", -316),
    ("郙之⟨E1FF⟩(歲)", "大司马悼滑救郙之岁", -315),
    # 系年不明之岁：单列，不计入任何一组
    ("監𠤳逅楚之⟨E1FF⟩(歲)", "囗客监𠤳逅楚之岁", None),
    ("甘𠤳之⟨E1FF⟩(歲)", "甘𠤳之岁", None),
]
mon_re = re.compile("|".join("(" + p + ")" for p, _, _ in MONTHS))
day_re = re.compile(r"\s*[，、]?\s*(?:#[^#]+#)?\s*(" + "|".join(GZ) + r")\s*(?:#[^#]+#)?\s*之日")

recs = []
for m in mon_re.finditer(s):
    k = next(i for i, g in enumerate(m.groups()) if g)
    d = day_re.match(s, m.end())
    if not d:
        continue
    pre = s[max(0, m.start() - 60):m.start()]
    sui = None
    for key, name, y in SUI:
        if key in pre:
            sui = (name, y)
    jian = re.search(r"#([^#]+)#", s[m.start():m.start() + 200])
    ctx = s[max(0, m.start() - 20):d.end() + 10].replace("\n", "").replace(",", "，")
    recs.append(dict(sui=sui[0] if sui else "", sui_year=(sui[1] if sui[1] is not None else "?") if sui else "", month_no=MONTHS[k][1], month=MONTHS[k][2],
                     gz=d.group(1), jian=jian.group(1) if jian else "", ctx=ctx))
# 同岁同月同干支重出者（卜筮祭祷简多次贞问同一日）只计一次
seen, uniq = set(), []
for r in recs:
    key = (r["sui"], r["month_no"], r["gz"])
    if key in seen:
        continue
    seen.add(key)
    uniq.append(r)

with open(ROOT / "data/calendar/baoshan_records.csv", "w", encoding="utf-8", newline="") as f:
    f.write("# 包山楚简明书「月名 + 干支之日」纪日（scripts/calendar_data/baoshan_check.py 自维基文库录文抽出）；同岁同月同干支只留一条。\n")
    w = csv.DictWriter(f, fieldnames=["sui", "sui_year", "month_no", "month", "gz", "jian", "ctx"])
    w.writeheader()
    w.writerows(uniq)

# 历表
cal = collections.defaultdict(list)
for r in csv.DictReader(open(ROOT / "data/calendar/guliuli_months_350_300.csv", encoding="utf-8")):
    cal[(r["历"], r["闰法"])].append((int(r["朔日JDN"]), int(r["下月朔JDN"]), int(r["夏正年"]), int(r["夏正月"]), r["闰"] == "1"))


def hit(months, y_xia, h, gzi):
    """夏正 y 年 h 月（含同号闰月）内是否有干支 gzi 之日。"""
    for a, b, y, m, _ in months:
        if y == y_xia and m == h and any((d + 49) % 60 == gzi for d in range(a, b)):
            return True
    return False


rows = []
known = [r for r in uniq if r["sui_year"] not in ("", "?")]
unknown = [r for r in uniq if r["sui_year"] == ""]
other = [r for r in uniq if r["sui_year"] == "?"]
for (name, rf), months in cal.items():
    for m0 in (9, 10, 11, 12, 1):
        for off in (-2, -1, 0, 1, 2):
            n = ok = 0
            for r in known:
                Y = r["sui_year"] + off
                h = (m0 - 1 + r["month_no"] - 1) % 12 + 1
                y_xia = Y - 1 if (m0 > 1 and h >= m0) else Y
                n += 1
                ok += hit(months, y_xia, h, GZ.index(r["gz"]))
            # 岁不明之条：同一岁（受期多在一岁之内），取合数最多之年
            best = (0, None)
            for Y in range(-325, -309):
                c = 0
                for r in unknown:
                    h = (m0 - 1 + r["month_no"] - 1) % 12 + 1
                    y_xia = Y - 1 if (m0 > 1 and h >= m0) else Y
                    c += hit(months, y_xia, h, GZ.index(r["gz"]))
                best = max(best, (c, Y))
            rows.append(dict(历=name, 闰法=rf, 冬柰当夏正=m0, 系年平移=off, 有岁条数=n, 有岁合=ok,
                             岁不明条数=len(unknown), 岁不明最佳年合=best[0], 岁不明最佳年=best[1]))
rows.sort(key=lambda r: (-(r["有岁合"] + r["岁不明最佳年合"]), r["历"]))
with open(ROOT / "data/calendar/baoshan_check.csv", "w", encoding="utf-8", newline="") as f:
    f.write("# 包山楚简纪日检验（scripts/calendar_data/baoshan_check.py）：各历×楚正月当夏正某月×系年平移，干支落在所记之月的条数。"
            "基线：任一干支落入一月约 49%。岁不明之条（多为受期）按同一年计，取合数最多之年。\n")
    w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
    w.writeheader()
    w.writerows(rows)
print("纪日", len(recs), "去重", len(uniq), "有岁", len(known), "岁不明", len(unknown), "系年不明之岁（不计）", len(other))
for r in rows[:25]:
    print(r)

# ---- 逐条：主假设（冬柰当夏正十月、系年不平移、岁不明之条取前 317 年）下各历该日为月中第几日（0 = 不在所记之月），写入知识库文档
H = [("颛顼历", "年终"), ("殷历", "年终"), ("鲁历", "年终"), ("周历", "年终"), ("黄帝历", "年终"), ("夏历", "年终"), ("夏历（雨水历元）", "年终")]


def day_in(months, y, h, g):
    for a, b, yy, m, _ in months:
        if yy == y and m == h:
            for dd in range(a, b):
                if (dd + 49) % 60 == g:
                    return dd - a + 1
    return 0


tot = [0] * len(H)
lines = []
for r in known + unknown:
    Y = r["sui_year"] if r["sui_year"] != "" else -316
    h = (9 + r["month_no"] - 1) % 12 + 1
    y_xia = Y - 1 if h >= 10 else Y
    p = [day_in(cal[k], y_xia, h, GZ.index(r["gz"])) for k in H]
    for i, x in enumerate(p):
        tot[i] += x > 0
    lines.append("| " + " | ".join([r["sui"] or "（岁不明，按前 317 年）", r["month"], r["gz"], r["jian"]] + [str(x) if x else "—" for x in p]) + " |")
md = ["# 包山楚简纪日检验（生成文件）", "",
      "> 由 `scripts/calendar_data/baoshan_check.py` 生成，勿手改。方法与结论见 [zhanguo-calendars.md](./zhanguo-calendars.md)「楚：包山楚简检验」。", "",
      "主假设：楚正月（冬柰）当夏正十月，楚岁以冬柰为首；各岁系年依《包山楚墓》（转引），岁不明之条（多为受期简）按前 317 年。"
      "表中数字为该干支是该历所记之月的第几日，「—」为不在此月。", "",
      "| 岁 | 楚月 | 干支 | 简 | " + " | ".join(k[0] for k in H) + " |", "|" + "---|" * (4 + len(H))]
md += lines
md += ["| 合计 |  |  | " + str(len(known) + len(unknown)) + " | " + " | ".join(str(x) for x in tot) + " |", "", "## 各假设合数", "",
       "| 历 | 闰法 | 冬柰当夏正 | 系年平移 | 有岁（合/条） | 岁不明最佳年（合/条） | 最佳年 |", "|---|---|---|---|---|---|---|"]
for r in rows[:40]:
    md.append(f"| {r['历']} | {r['闰法']} | {r['冬柰当夏正']} 月 | {r['系年平移']:+d} | {r['有岁合']}/{r['有岁条数']} | {r['岁不明最佳年合']}/{r['岁不明条数']} | {r['岁不明最佳年']} |")
(ROOT / "doc/calendar/baoshan-checks.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print("合计", dict(zip([k[0] for k in H], tot)))
