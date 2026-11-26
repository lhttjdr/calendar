"""琉球历书实物（data/calendar/ryukyu_rili.csv）与清时宪历推步、张表逐月比较 → doc/calendar/ryukyu-rili-checks.md。

琉球奉清正朔，每年进贡船在福州领回次年时宪书；船未归之前，琉球「司憲書官」先印《選日通書》权行国中（乾隆二十七年本卷首题识：
「教令印造選日通書權行國中以俟天朝頒賜憲書，憲書頒到日通國皆用憲書」）。所以选日通书是琉球自推之历：看它的朔闰、
节气日与交时合不合当年清廷所用之法，便知琉球在福州所学是哪一法、推的是哪一地之时。

- 推步：1742 年起行用《历象考成后编》（scripts/calendar_data/shixian_houbian.py，用时京师）；另以《历象考成》（康熙甲子元法，
  scripts/calendar_data/shixian_kaocheng.py）作对照——琉球同册所收《推朔望法》以平朔加日月均数求实朔，像考成上编之法，须看所载合哪一法。
- 张表：vendor/calendar/chinese_calendar/实历朔日表.csv。
- 分类：shiwu_fenlei（每月一个；历书实物 A 级）；推步取后编。
- 交时：书载 − 推算（分）。推算时刻取到分（舍秒，「截」）与四舍五入两种都列；那霸在京师东约 11.3°，若按那霸地方时当晚 45 分。

用法（仓库根）：uv run --project scripts python -I scripts/calendar_data/ryukyu_rili_check.py
"""
import csv
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))
import shiwu_fenlei as fl  # noqa: E402
import shixian_houbian as hb  # noqa: E402
import shixian_kaocheng as kc  # noqa: E402

GZ = fl.GZ
Z = "子丑寅卯辰巳午未申酉戌亥"
SRC = ROOT / "data/calendar/ryukyu_rili.csv"
OUT = ROOT / "doc/calendar/ryukyu-rili-checks.md"
QI = ["冬至", "小寒", "大寒", "立春", "雨水", "惊蛰", "春分", "清明", "谷雨", "立夏", "小满", "芒种", "夏至", "小暑", "大暑", "立秋",
      "处暑", "白露", "秋分", "寒露", "霜降", "立冬", "小雪", "大雪"]
NUM = {"初": 0, "一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9, "十": 10}
NAHA_MIN = round((127.68 - 116.47) * 4)


def g(j):
    return GZ[(j + 49) % 60]


def jdn(s):
    y, m, d = map(int, s.split("-"))
    a = (14 - m) // 12
    yy, mm = y + 4800 - a, m + 12 * a - 3
    return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - yy // 100 + yy // 400 - 32045


def cn(s):
    if s in ("", "初"):
        return 0
    if s.startswith("二十"):
        return 20 + (NUM[s[2]] if len(s) > 2 else 0)
    if s.startswith("十"):
        return 10 + (NUM[s[1]] if len(s) > 1 else 0)
    return NUM[s]


def parse_time(s):
    """「亥初一刻六分」→ 自子正起分钟（子初 = 23 时）；含 □ 者 None"""
    if not s or "□" in s:
        return None
    zhi, half, rest = Z.index(s[0]), s[1], s[2:]
    ke, fen = rest.split("刻")
    h = zhi * 2 - (0 if half == "正" else 1)
    h = 23 if h < 0 else h
    return h * 60 + cn(ke) * 15 + cn(fen.replace("分", ""))


def fmt(mins):
    """自子正起分钟 → 「亥初一刻六分」"""
    h = mins // 60
    half, zhi = ("初", Z[((h + 1) // 2) % 12]) if h % 2 else ("正", Z[h // 2])
    k, f = divmod(mins - h * 60, 15)
    return f"{zhi}{half}{'初一二三'[k]}刻{f}分"


def months(mod, fn, y):
    """农历年 y 之 (月, 闰) → 朔 JDN（用时）"""
    out = {}
    for sy in (y, y + 1):
        for j, m, leap in fn(sy):
            ly = sy if m <= 10 else sy - 1
            if ly == y:
                out[(m, int(leap))] = j
    return out


def qi_time(mod, name, near):
    """推算节气用时：返回 (日 JDN, 自子正起分钟 float)，取最接近 near（JDN）者"""
    k = QI.index(name)
    yr = int((near - 1721426) / 365.2425) + 1
    best = None
    for sy in (yr, yr + 1):
        t = mod.jieqi_times(sy)[k]
        u = t + mod.shi_cha(t)
        d = mod.jdn_of(math.floor(u))
        if best is None or abs(d - near) < abs(best[0] - near):
            best = (d, (u % 1) * 1440)
    return best


def main():
    rows = [r for r in csv.DictReader(l for l in open(SRC, encoding="utf-8") if not l.startswith("#"))]
    years = sorted({int(r["年"]) for r in rows})
    ZH = {}
    for r in csv.DictReader(open(ROOT / "vendor/calendar/chinese_calendar/实历朔日表.csv", encoding="utf-8")):
        if int(r["lunar_year"]) in years:
            ZH[(int(r["lunar_year"]), int(r["lunar_month"]), int(r["is_leap"]))] = jdn(
                f"{r['gregorian_year']}-{r['gregorian_month']}-{r['gregorian_day']}")
    HB, KC = {}, {}
    for y in years:
        for k, v in months(hb, hb.year_months, y).items():
            HB[(y,) + k] = v
        for k, v in months(hb, hb.year_months, y + 1).items():
            if k == (1, 0):
                HB[(y + 1,) + k] = v
        for k, v in months(kc, kc.sui_months, y).items():
            KC[(y,) + k] = v

    mrows, cats, qrows = [], [], []
    for r in rows:
        k = (int(r["年"]), int(r["月"]), int(r["闰"]))
        h, c, z = HB.get(k), KC.get(k), ZH.get(k)
        shu = r["朔干支"]
        cat, note = fl.classify(g(h) if h else "", g(z) if z else "", [shu], "A", "历书")
        cats.append((cat, "A"))
        later = [v for kk, v in HB.items() if v > h] if h else []
        nxt = min(later) if later else None
        size = ("大" if nxt - h == 30 else "小") if (h and nxt) else ""
        mrows.append([f"{k[0]}", ("閏" if k[2] else "") + str(k[1]), r["大小"], shu, g(h) if h else "", size,
                      g(c) if c else "", g(z) if z else "", "✓" if h and shu == g(h) else "✗", "✓" if z and shu == g(z) else "✗",
                      cat + ("；" + note if note else "")])
        if not h:
            continue
        for kind in ("节", "中气"):
            name = r[kind]
            if not name:
                continue
            day = cn(r[kind + "日"])
            gz = r[kind + "日干支"]
            near = h + day - 1
            (dh, mh), (dk, mk) = qi_time(hb, name, near), qi_time(kc, name, near)
            bm = parse_time(r[kind + "交时"])
            okd = "✓" if g(near) == gz else f"✗（{g(near)}）"
            qrows.append([("閏" if k[2] else "") + str(k[1]), name, f"{r[kind + '日']}日{gz}", okd, r[kind + "交时"] or "",
                          f"{g(dh)} {fmt(math.floor(mh))}", "" if bm is None or dh != near else f"{bm - math.floor(mh):+d}",
                          "" if bm is None or dh != near else f"{bm - round(mh):+d}",
                          f"{g(dk)} {fmt(math.floor(mk))}", "" if bm is None or dk != near else f"{bm - math.floor(mk):+d}",
                          "✓" if dh == near else "✗", "✓" if dk == near else "✗"])

    n_h = sum(1 for m in mrows if m[8] == "✓")
    n_z = sum(1 for m in mrows if m[9] == "✓")
    n_k = sum(1 for m in mrows if m[3] == m[6])
    dh = [int(q[6]) for q in qrows if q[6]]
    dr_ = [int(q[7]) for q in qrows if q[7]]
    dk = [int(q[9]) for q in qrows if q[9]]
    from collections import Counter
    md = [
        "# 琉球选日通书与清时宪历（自动生成）", "",
        "> 由 `scripts/calendar_data/ryukyu_rili_check.py` 生成，勿手改。录文 `data/calendar/ryukyu_rili.csv`；书目与背景见 "
        "[ryukyu-sources.md](ryukyu-sources.md)。", "",
        "## 底本与方法", "",
        "- 底本：《大清乾隆二十七年選日通書》一卷（琉球司憲書官印造；日本鈔本，东京大学综合图书馆藏），影印于《琉球王國漢文文獻集成》"
        "（复旦大学出版社 2013）第 23 册第 1—44 页。卷首题识：「琉球國司憲書官謹奉教令印造選日通書權行國中以俟天朝頒賜憲書。"
        "憲書頒到日通國皆用憲書，共得凜遵一王之正朔」。录卷首「都城順天府節氣時刻」一览（第 9—10 页）：月大小、朔干支、节与中气之日及交时。",
        "- 推步：乾隆七年（1742）起清廷行用《历象考成后编》，取后编用时（京师）为「推步」；《历象考成》（康熙甲子元法）作对照。"
        "张表：张培瑜表（经 ChineseCalendar 转录）。",
        "- 分类：" + "两家**相同**之月——有证（实物合两家）、同误（实物与两家皆不合）、无证；两家**互异**之月——①证张表误、②证推步误、③无据待核。"
        "历书实物 A 级，一见即判。",
        f"- 交时：书载 − 推算（分）。「截」= 推算时刻舍去秒，「入」= 四舍五入。那霸（东经约 127.7°）在京师东约 {NAHA_MIN} 分。", "",
        "## 分类（按月）", "",
        f"共 {len(mrows)} 月：" + fl.headline(cats, "后编推步") + "。", ""] + fl.grade_table(cats) + ["",
        f"朔合后编 {n_h}／{len(mrows)}，合张表 {n_z}／{len(mrows)}，合考成 {n_k}／{len(mrows)}。", "",
        "## 逐月", "",
        "| 年 | 月 | 书大小 | 书朔 | 后编朔 | 后编大小 | 考成朔 | 张表朔 | 合后编 | 合张表 | 分类 |",
        "|---|---|---|---|---|---|---|---|---|---|---|"]
    md += ["| " + " | ".join(m) + " |" for m in mrows]
    md += ["", "## 节气日与交时", "",
           "节气日干支由书载朔干支与日数推得，「日合」核书载干支；后编、考成皆为用时京师。", "",
           "| 月 | 气 | 书日 | 日数合干支 | 书交时 | 后编 | 差（截） | 差（入） | 考成 | 差 | 合后编日 | 合考成日 |",
           "|---|---|---|---|---|---|---|---|---|---|---|---|"]
    md += ["| " + " | ".join(q) + " |" for q in qrows]
    md += ["", "## 小结", "",
           f"- 交时差（书 − 后编，截秒）分布：{sorted(Counter(dh).items())}；四舍五入：{sorted(Counter(dr_).items())}。"
           f"书 − 考成：{sorted(Counter(dk).items())}。",
           f"- 气日合后编 {sum(q[10] == '✓' for q in qrows)}／{len(qrows)}，合考成 {sum(q[11] == '✓' for q in qrows)}／{len(qrows)}。",
           "- 结论见 [ryukyu-sources.md](ryukyu-sources.md)「选日通书检验」。"]
    OUT.write_text("\n".join(md) + "\n", encoding="utf-8")
    print(fl.headline(cats, "后编推步"))
    print("交时差 截", sorted(Counter(dh).items()), "入", sorted(Counter(dr_).items()), "考成", sorted(Counter(dk).items()))


if __name__ == "__main__":
    main()
