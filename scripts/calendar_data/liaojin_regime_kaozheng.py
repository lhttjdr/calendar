"""辽、金分政权推步之史书检验逐条考证表 → data/calendar/liaojin_regime_kaozheng.csv（覆盖写）。

收录：(1)《辽史》《金史》本纪纪日中推步不合、或推步与张表（宋历）判断不同之条（取自 rust/core/examples/regime_benji_eval.rs
所出 data/calendar/regime_benji_check.csv）；(2) 人工考证之条（MANUAL，含已入史载例外表、改月之条）；(3) 抽取改字（EXTRACT，
scripts/calendar_data/qinhan_eval/zhi_star_records.py 的 jinshi_benji corrections，改后该年记事归回本年，不再不合，记此备查）。
两家同不合者按日干支距推步月首的日数自动归类：在前一月或后一月 = 本纪略书月份（「是月」后不书次月）或月序讹；书朔差一二日 =
本纪讹或别有所据；其余 = 干支讹。此类两家判断相同，不涉辽金历与宋历之别。书朔之条另接 data/calendar/liao_shuo_kaozheng.csv 的初判。
「处置」：改月 / 不计 由 regime_benji_eval.rs 读入应用；已入例外表 = data/calendar/dingshuo_exceptions.csv 有此条；存 = 只记考证。
推步月首据 data/calendar/推步政权朔日表.csv（examples/export_regime_calendars.rs）。
用法（仓库根）：python3 scripts/calendar_data/liaojin_regime_kaozheng.py（先跑 regime_benji_eval 与 export_regime_calendars）
"""
import collections
import csv
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GZ = ["甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12] for i in range(60)]
BOOK = {"辽": "liaoshi_benji", "金": "jinshi_benji"}

# (政权, 年, 干支, 原文起首) → (类, 处置, 改作, 判)
MANUAL = {}


def man(rg, y, gz_list, months, cls, how, to, why):
    """months：本纪纪日月列（「3」「闰9」「闰」）之集合"""
    for g in gz_list:
        for mo in months.split():
            MANUAL[(rg, y, g, mo)] = (cls, how, to, why)


# —— 辽 ——
man("辽", 985, ["癸酉", "丙子", "庚辰", "辛巳", "丙申", "戊戌", "己亥"], "闰9", "月名从宋历（辽历九月）", "改月", "9",
    "本纪「閏九月癸酉……庚辰，重九，駱駝山登高，賜群臣菊花酒」：庚辰为壬申朔之初九，即重九，此月在辽历为九月。宣明代推是年閏八月、九月壬申朔，"
    "宋历（张表）閏九月壬申朔；点校本校勘记〔五〕（据汪曰桢《辑要》、陈垣表）亦谓辽閏八、宋閏九，此失书辽閏。同月内日序（重九）是独立于历表的证据，"
    "本纪「閏」字当是修史者照宋历月名所加。按九月检验")
man("辽", 1067, ["癸亥", "丁亥", "辛卯", "乙巳"], "3 闰", "闰位存疑", "存", "",
    "本纪「三月癸亥，宋主曙殂……閏月丁亥」，宋历閏三月合；宣明代推閏正月，日本宣明、《高丽史》亦閏正月（「二月己酉朔」合推步）。"
    "宋英宗正月丁巳崩，辽二月得讣亦通。本纪月序或经修史者以宋历改，存疑（liao_shuo_kaozheng.csv）")
man("辽", 1030, ["乙卯"], "1", "临限从本纪", "已入例外表", "",
    "宣明代推定朔甲寅 17.77 时，不及正月进朔限约 0.23 时；本纪「十年春正月乙卯朔」。贾俊大明术不传，差在代推误差内，从本纪（例外表类别「本纪」）")
# —— 金 ——
man("金", 1162, ["甲戌", "戊子", "庚寅", "辛卯", "乙未", "癸亥", "甲辰", "乙巳", "辛亥"], "闰 3", "史载闰位", "已入例外表", "",
    "本纪大定二年二月、閏月、三月相次，即閏二月；重修大明代推閏三月（杨级大明术不传）。《高丽史》卷十八毅宗十六年「二月……丁卯……閏月甲戌……"
    "己丑……三月丙午」亦閏二月（高丽自行宣明，只作旁证）。例外表置閏二月戊辰朔、三月戊戌朔")
man("金", 1174, ["丙戌"], "5", "临限从本纪", "已入例外表", "",
    "代推定朔丙戌 17.64 时，限 17.63 时，超 36 秒进作丁亥；本纪「五月丙戌朔」，张表同。天文志是年只书十一月日食，此月无食")
man("金", 1178, ["庚寅"], "10", "临限从本纪", "已入例外表", "",
    "代推定朔庚寅 18.14 时，限 18.00 时，超 0.14 时进作辛卯（张表亦辛卯）；本纪「十月庚寅朔」。从本纪")
man("金", 1179, ["乙未"], "3", "本纪干支讹", "存", "",
    "本纪大定十九年「正月庚申朔……二月己酉……三月乙未朔，萬春節」，推步、张表三月皆己未朔，乙未距之二十四日；乙、己形近，当为「己未」之讹"
    "（同书大定十八年亦有「三月乙未朔，萬春節」，或涉上年而误）")

# —— 抽取改字（zhi_star_records.py jinshi_benji corrections）——
EXTRACT = [
    ("金", 1116, "", "", "卷二「熙宗天眷二年，以黃龍府爲濟州」为插叙，原被当作纪年，其后收国元年九月至二年之事（十余条）归入天眷二年（1139）；解析时略去其年数"),
    ("金", 1174, "1", "己丑", "卷七「十四年正月已醜朔」，「已醜」為「己丑」之訛，致大定十四年之事归入十三年；改后正月己丑朔合推步、张表"),
    ("金", 1194, "1", "癸亥", "卷十「五年春正月癸朔」脱「亥」，致明昌五年之事归入四年；改后合推步、张表（点校本未核）"),
    ("金", 1219, "1", "庚午", "卷十五「三年春正月庚千」，「千」為「午」之訛，致兴定三年之事（约七十条）归入二年；兴定三年正月戊辰朔，庚午初三"),
    ("金", 1216, "1", "己卯", "卷十七哀宗纪「貞祐初……四年正月己卯，立守禮為皇太子」，「四年」承贞祐，原被当作承安四年（1199）；解析时补年号"),
]


def jd(y, m, d):
    a = (14 - m) // 12
    yy = y + 4800 - a
    mm = m + 12 * a - 3
    return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - 32083


def table(path, regime=None):
    T = collections.defaultdict(list)
    for r in csv.DictReader(open(path, encoding="utf-8")):
        if regime and r["regime"] != regime:
            continue
        T[int(r["lunar_year"])].append((jd(int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"])), int(r["lunar_month"]), r["is_leap"] == "1"))
    return T


def auto(rg, T, r):
    """两家同不合之条：按日干支距推步月首的日数归类。"""
    y = int(r["年"])
    m0 = r["月"].split("→")[0]
    if m0.startswith("闰（"):
        return "閏月纪日不在闰月内（干支或月序讹）", "存", "閏月不书月数，按两家当年闰月判，皆不在月内"
    l = m0.startswith("闰")
    m = int(m0.lstrip("闰"))
    c = [e for e in T.get(y, []) if e[1] == m and e[2] == l]
    if not c:
        return "推步无此月", "存", ""
    a = c[0][0]
    gi = GZ.index(r["干支"])
    k = (gi - (a + 49)) % 60
    k = k if k < 30 else k - 60
    nxt = min([e[0] for e in T.get(y, []) + T.get(y + 1, []) if e[0] > a], default=a + 30)
    if r["位置"] == "朔":
        if abs(k) <= 2:
            return f"书朔差{k:+d}日（两家同）", "存", "本纪讹或别有所据，推步、宋历同不合，不涉两家之别"
        return "朔干支或月序讹", "存", f"本纪朔干支距推步月首 {k:+d} 日"
    if k < 0:
        return "略书月份或月序讹（干支在前一月）", "存", f"距推步月首 {k:+d} 日"
    if a + k >= nxt:
        return "略书月份或月序讹（干支在后一月）", "存", f"距推步月首 +{k} 日，月长 {nxt - a}"
    return "干支讹", "存", ""


def main():
    shuo_kz = {}
    for r in csv.DictReader(l for l in open(ROOT / "data/calendar/liao_shuo_kaozheng.csv", encoding="utf-8") if not l.startswith("#")):
        if r["类"] == "明书朔":
            shuo_kz[(int(r["年"]), r["月"])] = r["初判"]
    out = []
    seen = set()
    for r in csv.DictReader(open(ROOT / "data/calendar/regime_benji_check.csv", encoding="utf-8")):
        rg = r["政权"]
        if rg not in BOOK:
            continue
        y = int(r["年"])
        m0 = r["月"].split("→")[0]
        month_key = "闰" if m0.startswith("闰（") else m0
        key = (rg, y, r["干支"], month_key) if (rg, y, r["干支"], month_key) in MANUAL else None
        if key is None and r["推步合"] == "1" and r["张表合"] == "1":
            continue
        T = table(ROOT / "data/calendar/推步政权朔日表.csv", rg) if rg not in table.cache else table.cache[rg]
        table.cache[rg] = T
        if key:
            cls, how, to, why = MANUAL[key]
            seen.add(key)
        elif r["推步合"] != r["张表合"]:
            ok = r["推步合"] == "1"
            cls = "合推步不合张表" if ok else "合张表不合推步"
            how, to = "存", ""
            why = ("本纪从本国之历（" + r["推步之历"] + "），与宋历异" if ok else "待考")
        else:
            cls, how, why = auto(rg, T, r)
            to = ""
        if r["位置"] == "朔" and rg == "辽" and (y, str(int(month_key.lstrip("闰") or 0))) in shuo_kz:
            why = (why + "；" if why else "") + "liao_shuo_kaozheng 初判：" + shuo_kz[(y, str(int(month_key.lstrip("闰") or 0)))]
        out.append(dict(书=r["书"], 政权=rg, 年=y, 月=month_key, 干支=r["干支"], 位置=r["位置"], 推步之历=r["推步之历"], 推步月首=r["推步月首"],
                        合推步=r["推步合"], 张表月首=r["张表月首"], 合张表=r["张表合"], 类=cls, 处置=how, 改作=to, 判=why, 原文=r["原文"][:80]))
    for rg, y, m, g, why in EXTRACT:
        out.append(dict(书=BOOK[rg], 政权=rg, 年=y, 月=m, 干支=g, 位置="", 推步之历="", 推步月首="", 合推步="", 张表月首="", 合张表="",
                        类="抽取改字（已改）", 处置="存", 改作="", 判=why, 原文=""))
    missing = sorted({k[:3] for k in MANUAL} - {k[:3] for k in seen})
    assert not missing, f"人工考证之条未在检验结果中出现：{missing}"
    path = ROOT / "data/calendar/liaojin_regime_kaozheng.csv"
    with open(path, "w", encoding="utf-8", newline="") as f:
        f.write("# 辽金分政权推步之史书检验逐条考证（scripts/calendar_data/liaojin_regime_kaozheng.py 生成，人工考证写在脚本 MANUAL、EXTRACT 中）。"
                "处置：改月/不计 由 rust/core/examples/regime_benji_eval.rs 读入；已入例外表 = data/calendar/dingshuo_exceptions.csv（历名「政权：历名」）。\n")
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    c = collections.Counter((o["政权"], o["类"]) for o in out)
    for k, v in sorted(c.items()):
        print(k, v)
    print(path, len(out), "条")


table.cache = {}

if __name__ == "__main__":
    main()
