"""《汉书》《后汉书》日食所记日躔（「在某宿某度」）与三统历、四分历平行日躔逐条对照：判断这些宿度是不是当时或后人按历推算的。

两历日躔都是「日行一度」，冬至时刻按平气，宿度用汉代赤道宿度（《汉书·律历志》，与《元史》「周天列宿度」汉栏同），斗宿带斗分：
- 三统历：岁实 = 周天 = 365 又 385/1539 日（度），朔策 29 又 43/81 日；历元为太初元年前十一月甲子朔旦冬至（前 105 年 12 月 25 日，
  JDN 1683431）；冬至日在牵牛初度。
- 后汉四分历：岁实 = 周天 = 365¼，朔策 29 又 499/940 日；蔀首取文帝后元三年前十一月甲子朔旦冬至（前 162 年 12 月 25 日，
  JDN 1662611）；冬至日在斗二十一度四分度之一。

每条按两种日子推算日躔，均取该日夜半：
- 记载日：数据表 jdn（按实历朔日表换算，即当时行用历的日子）。
- 同名日：按该历自己的平朔重排月日后，与记载同名（朔、晦、先晦一日、二日）的那一天。若宿度是后人按某历追算，追算的是该历里的「某月晦」，
  不一定是当时行用历的同一天。
差 = 推算 − 记载（度），以所记之宿的宿首起量。记载只到宿、不到度者不计。

输出 data/phenomena/checks/han_eclipse_sun_by_calendar.csv（逐条）与 _summary.csv（按时期），收入 doc/astronomy/phenomena-checks.md。
用法（仓库根）：python3 scripts/calendar_data/qinhan_eval/han_eclipse_sun_by_calendar.py
"""
import csv
import statistics as st
import sys
from fractions import Fraction as F
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE.parent))
import gaiyuan_date as g  # noqa: E402

# 汉代赤道宿度（《汉书·律历志》），自角起
WIDTHS = dict(zip("角亢氐房心尾箕斗牛女虚危室壁奎娄胃昴毕觜参井鬼柳星张翼轸",
                  [12, 9, 15, 5, 5, 18, 11, 26, 8, 12, 10, 17, 16, 9, 16, 12, 14, 11, 16, 2, 9, 33, 4, 15, 7, 18, 18, 17]))
ORDER = "牛女虚危室壁奎娄胃昴毕觜参井鬼柳星张翼轸角亢氐房心尾箕斗"  # 自牵牛初起
# 名：(历元 JDN（甲子朔旦冬至夜半）, 岁实, 朔策, 冬至日躔距牵牛初度数)
CALENDARS = {
    "三统": (1683431, F(562120, 1539), F(2392, 81), F(0)),
    "四分": (1662611, F(1461, 4), F(27759, 940), F(-5)),  # 斗 26¼ 度，斗 21¼ 在牛初前 5 度
}
NAMES = [("南斗", "斗"), ("東井", "井"), ("輿鬼", "鬼"), ("牽牛", "牛"), ("婺女", "女"), ("營室", "室"), ("東壁", "壁"), ("七星", "星"),
         ("虛", "虚"), ("婁", "娄"), ("畢", "毕"), ("參", "参"), ("張", "张"), ("軫", "轸")]
DIGITS = "〇一二三四五六七八九"
# 记载的日位 → 相对下月朔（晦）或本月朔的偏移
POSITION = {"朔": ("朔", 0), "二日": ("朔", 1), "2日": ("朔", 1), "晦": ("次朔", -1), "先晦一日": ("次朔", -2)}
PERIODS = [(-300, -105, "太初以前（颛顼历行用）", "三统"), (-104, 0, "太初至哀平（太初历行用）", "三统"),
           (1, 84, "光武至章帝（太初历行用）", "四分"), (85, 300, "元和以后（四分历行用）", "四分")]


def parse_lodge(s):
    """「斗二十度」「營室九度」「七星初」→（宿, 度）；只有宿名者返回 None。"""
    for a, b in NAMES:
        s = s.replace(a, b)
    lodge, rest = s[0], s[1:]
    if lodge not in WIDTHS:
        return None
    if rest.startswith("初"):
        return lodge, F(0)
    num, sep, tail = rest.partition("度")
    if not sep or not num:
        return None
    v = cur = 0
    for c in num:
        if c in "十百":
            v += (cur or 1) * (10 if c == "十" else 100)
            cur = 0
        elif c in DIGITS:
            cur = DIGITS.index(c)
        else:
            return None
    v += cur
    return lodge, F(v) + {"少": F(1, 4), "半": F(1, 2), "太": F(3, 4)}.get(tail[:1], F(0))


def lodge_start(lodge, dou):
    x = F(0)
    for c in ORDER:
        if c == lodge:
            return x
        x += WIDTHS[c] + (dou if c == "斗" else 0)


def sun_degree(cal, day):
    """该历在 day 日夜半的平行日躔：自牵牛初起的度数。"""
    epoch, year, _, start = CALENDARS[cal]
    t = F(day) - epoch
    return (t - (t // year) * year + start) % year


def residual(cal, day, lodge, deg):
    dou = CALENDARS[cal][1] - 365
    d = sun_degree(cal, day) - lodge_start(lodge, dou)
    d = (d + CALENDARS[cal][1] / 2) % CALENDARS[cal][1] - CALENDARS[cal][1] / 2
    return float(d - deg)


def same_named_day(cal, jdn, position):
    """按该历平朔重排后与记载同名的日子；记载日位不明者返回 None。"""
    if position not in POSITION:
        return None
    ref, off = POSITION[position]
    epoch, _, month, _ = CALENDARS[cal]
    target = jdn - off  # 当时行用历的（次）朔日
    k = round((F(target) - epoch) / month)
    return int(epoch + k * month) + off


def main():
    rows = []
    for book, title in (("hanshu", "《汉书》"), ("houhanshu", "《后汉书》")):
        with open(ROOT / f"data/phenomena/{book}_solar_eclipses.csv", encoding="utf-8") as f:
            for r in csv.DictReader(line for line in f if not line.startswith("#")):
                if not r["lodge"] or not r["jdn"]:
                    continue
                p = parse_lodge(r["lodge"])
                if p is None:
                    continue
                lodge, deg = p
                jdn, year = int(r["jdn"]), int(r["year"])
                period, cal = next((name, c) for lo, hi, name, c in PERIODS if lo <= year <= hi)
                o = {"书": title, "年": year, "公历": r["julian"], "日位": r["position"], "记载": r["lodge"], "时期": period}
                for c in CALENDARS:
                    o[f"{c}·记载日"] = residual(c, jdn, lodge, deg)
                    s = same_named_day(c, jdn, r["position"])
                    o[f"{c}·同名日"] = residual(c, s, lodge, deg) if s is not None else None
                    o[f"{c}同名日−记载日"] = (s - jdn) if s is not None else None
                rows.append(o)
    cols = ["三统·记载日", "三统·同名日", "四分·记载日", "四分·同名日"]
    out_dir = ROOT / "data/phenomena/checks"
    with open(out_dir / "han_eclipse_sun_by_calendar.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 《汉书》《后汉书》日食所记日躔与三统历、四分历平行日躔逐条对照（scripts/calendar_data/qinhan_eval/han_eclipse_sun_by_calendar.py）\n")
        f.write("# 差 = 推算 − 记载（度，汉代赤道宿度）；记载日为当时行用历之日，同名日为按该历平朔重排后与记载同名（朔、晦等）之日\n")
        keys = ["书", "年", "公历", "日位", "记载", "时期"] + cols + ["三统同名日−记载日", "四分同名日−记载日"]
        w = csv.writer(f)
        w.writerow(keys)
        for o in rows:
            w.writerow([f"{o[k]:+.2f}" if isinstance(o[k], float) else ("" if o[k] is None else o[k]) for k in keys])
    with open(out_dir / "han_eclipse_sun_by_calendar_summary.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 按时期汇总：每格为「中位差 / |差|≤1 度条数」；条数为记载到度且日位明确者\n")
        w = csv.writer(f)
        w.writerow(["时期", "条数"] + cols)
        for _, _, name, _ in PERIODS:
            sub = [o for o in rows if o["时期"] == name]
            cells = []
            for c in cols:
                v = [o[c] for o in sub if o[c] is not None]
                cells.append(f"{st.median(v):+.2f} / {sum(abs(x) <= 1 for x in v)}" if v else "")
            w.writerow([name, len(sub)] + cells)
            print(name, len(sub), cells)


if __name__ == "__main__":
    main()
