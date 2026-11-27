"""琉球当时文字纪日（《琉球國碑文記》落款、蔡大鼎《北上雜記》日记）与中原历日比较 → data/calendar/ryukyu_jiri_check.csv、
doc/calendar/ryukyu-jiri-checks.md。

- 张表：vendor/calendar/chinese_calendar/实历朔日表.csv（张培瑜表，经 ChineseCalendar 转录；复原，不作证据）。
- 推步（按年取当时行用之法）：明 1368—1644 大统（scripts/calendar_data/datong_tonggui_dingshuo.py，照《大统曆日通軌》立成）；
  清 1645—1725 西洋新法（data/calendar/shixian_xinfa_check.csv 用时朔日；康熙五至八年复用大统，取大统）；1726—1741《历象考成》
  （shixian_kaocheng_check.csv 平时朔日，本项目该段缺省）；1742 年起《历象考成后编》（shixian_houbian_check.csv 用时朔日）。
  节气：后编用时京师（shixian_houbian.py）。
- 验项（录文「验项」列）：
  - 日干支：书「某月某日干支」，朔 = 日干支 − (日 − 1)，与两家该月朔比；
  - 朔干支：书「某月某干支朔」；
  - 只书日干支不书日者（验项「日干支」而日空）：看该干支在两家该月第几日，不能定朔；
  - 月大：书「某月三十日」，与两家该月大小比；
  - 闰月：书「闰某月」，与两家该年是否闰该月比；
  - 节气：书「某日交某气」（日记），只与后编用时京师比较，张表不载节气，不入分类；
  - 无可验：只书年月（或有日无干支）。
- 分类（scripts/calendar_data/shiwu_fenlei.py，每月、每项一类）：两家相同——有证（与当时文字合）／同误／无证；两家互异——①证张表误／
  ②证推步误／③无据待核。碑文、日记为当时文字（B 级）；两家互异之月，单条纪日不足以定误（按 D 级，同月独立两条以上才判），归 ③。

用法（仓库根）：uv run --project scripts python -I scripts/calendar_data/ryukyu_jiri_check.py
"""
import csv
import math
import sys
from collections import defaultdict
from functools import lru_cache
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))
import gaiyuan_date as g  # noqa: E402
import shiwu_fenlei as fl  # noqa: E402

GZ = fl.GZ
SOURCES = [("碑文", ROOT / "data/calendar/ryukyu_beiwen.csv", "《琉球國碑文記》（第 17 册）"),
           ("北上雜記", ROOT / "data/calendar/ryukyu_beishang_zaji.csv", "蔡大鼎《北上雜記》卷一、二（第 29 册）")]
OUT_CSV = ROOT / "data/calendar/ryukyu_jiri_check.csv"
OUT_MD = ROOT / "doc/calendar/ryukyu-jiri-checks.md"
QI = ["冬至", "小寒", "大寒", "立春", "雨水", "惊蛰", "春分", "清明", "谷雨", "立夏", "小满", "芒种", "夏至", "小暑", "大暑", "立秋",
      "处暑", "白露", "秋分", "寒露", "霜降", "立冬", "小雪", "大雪"]
QI_ALIAS = {"驚蟄": "惊蛰", "穀雨": "谷雨", "小滿": "小满", "處暑": "处暑", "芒種": "芒种"}


def rows(p):
    if not p.exists():
        return []
    return list(csv.DictReader(l for l in open(p, encoding="utf-8") if not l.startswith("#")))


def ymd(j):
    return "%d-%02d-%02d" % g.from_jdn(j)


# ---------------------------------------------------------------- 张表与推步
@lru_cache(maxsize=1)
def zhang():
    out = {}
    for r in csv.DictReader(open(ROOT / "vendor/calendar/chinese_calendar/实历朔日表.csv", encoding="utf-8")):
        y = int(r["lunar_year"])
        if 1300 <= y <= 1920:
            out[(y, int(r["lunar_month"]), int(r["is_leap"]))] = g.jdn(int(r["gregorian_year"]), int(r["gregorian_month"]),
                                                                    int(r["gregorian_day"]))
    return out


@lru_cache(maxsize=1)
def tui_tables():
    """名 → {(农历年, 月, 闰): 朔 JDN}；大统无月名，另给朔日集。"""
    lab = {}

    def load(name, col, year_col):
        f = open(ROOT / "data/calendar" / name, encoding="utf-8")
        next(f)
        d = {}
        for r in csv.DictReader(f):
            y, m, dd = map(int, r[col].split("-"))
            ly = int(r[year_col]) if year_col == "农历年" else (int(r["冬至岁"]) - 1 if int(r["月"]) >= 11 else int(r["冬至岁"]))
            d[(ly, int(r["月"]), int(r["闰"]))] = g.jdn(y, m, dd)
        return d
    lab["新法"] = load("shixian_xinfa_check.csv", "用时朔日", "农历年")
    lab["考成"] = load("shixian_kaocheng_check.csv", "平时朔日", "冬至岁")
    lab["后编"] = load("shixian_houbian_check.csv", "用时朔日", "冬至岁")
    import datong_tonggui_dingshuo as tg
    import dingshuo_check as c
    li = tg.Tonggui(*tg.load_tables(False), True)
    off = c.align_offset(550600 // 10000, g.jdn(1280, 12, 14))
    days = sorted({int(t // 10000) + off for t in li.moons(1367, 1646)} | {int(t // 10000) + off for t in li.moons(1665, 1670)})
    # 大统按张表月名对齐（±3 日内最近之朔）
    zt = zhang()
    dt = {}
    for k, z in zt.items():
        if 1368 <= k[0] <= 1645 or 1665 <= k[0] <= 1670:
            near = [d for d in days if abs(d - z) <= 3]
            if near:
                dt[k] = min(near, key=lambda d: abs(d - z))
    # 康熙五至八年（1666—1669）复用大统：月序、闰位取大统本身推步（推步政权朔日表中明郑所用「大统历（明郑）」，子初进朔；
    # 1669 年与中原大统 reckon_year_any 逐月相同，已核），
    # 不借张表月名——张表此四年随时宪，闰位不同（如 1669 大统閏十二月、时宪次年閏二月）
    ming = defaultdict(dict)
    for r in rows(ROOT / "data/calendar/推步政权朔日表.csv"):
        if r["calendar"].startswith("大统历") and 1666 <= int(r["lunar_year"]) <= 1670:
            ming[r["calendar"]][(int(r["lunar_year"]), int(r["lunar_month"]), int(r["is_leap"]))] = g.jdn(
                int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"]))
    if ming:
        src = ming.get("大统历", {}) or next(iter(ming.values()))
        for k in [k for k in dt if 1666 <= k[0] <= 1669]:
            del dt[k]
        dt.update({k: v for k, v in src.items() if 1666 <= k[0] <= 1669})
    lab["大统"] = dt
    return lab


def method_of(y):
    if y <= 1644 or 1666 <= y <= 1669:
        return "大统"
    if y <= 1725:
        return "新法"
    if y <= 1741:
        return "考成"
    return "后编"


def month_info(table, k):
    """(朔 JDN, 大小) 或 (None, None)"""
    s = table.get(k)
    if s is None:
        return None, None
    later = [v for v in table.values() if s < v < s + 40]
    if not later:
        return s, None
    return s, ("大" if min(later) - s == 30 else "小")


def leap_of(table, y):
    """农历年 y 之闰月（月数），无闰为 0"""
    ls = [m for (yy, m, l) in table if yy == y and l]
    return ls[0] if ls else 0


@lru_cache(maxsize=None)
def jieqi_days(sy):
    """后编：冬至岁 sy 二十四气（用时京师）→ JDN"""
    import shixian_houbian as hb
    ts = hb.jieqi_times(sy)
    out = {}
    for k in range(24):
        t = ts[k]
        u = t + hb.shi_cha(t)
        out[QI[k]] = hb.jdn_of(math.floor(u))
    return out


def day_num(s):
    return int(s) if s and s.strip().isdigit() else None


# ---------------------------------------------------------------- 主
def check_rows(R, src, grade_same="B", grade_diff="D", what="当时文字"):
    """grade_same / grade_diff：两家相同 / 互异之月按何级判（碑文、日记为当时文字 B；互异时单条不足以判，按 D 需两条）。"""
    ZH = zhang()
    TT = tui_tables()
    lines = []
    for r in R:
        y = int(r["农历年"]) if r["农历年"] else None
        m = int(r["月"]) if r["月"] else None
        leap = int(r["闰"] or 0)
        meth = method_of(y) if y else ""
        T = TT.get(meth, {})
        k = (y, m, leap)
        z, zsize = month_info(ZH, k) if y and m else (None, None)
        t, tsize = month_info(T, k) if y and m else (None, None)
        d = day_num(r["日"])
        item = r["验项"]
        tui_v = zh_v = obs = ""
        inmonth = greg = qi = ""
        if item in ("日干支", "朔干支"):
            if r["朔干支"]:
                obs = r["朔干支"]
            elif d and r["日干支"]:
                obs = GZ[(GZ.index(r["日干支"]) - (d - 1)) % 60]
            elif r["日干支"]:
                obs = ""
                if z is not None:
                    inmonth = f"张表第{(GZ.index(r['日干支']) - (z + 49)) % 60 + 1}日"
                if t is not None:
                    inmonth += f"；推步第{(GZ.index(r['日干支']) - (t + 49)) % 60 + 1}日"
            tui_v = GZ[(t + 49) % 60] if t is not None else ""
            zh_v = GZ[(z + 49) % 60] if z is not None else ""
        elif item == "月大":
            obs, tui_v, zh_v = r["书值"], tsize or "", zsize or ""
        elif item == "闰月":
            obs = r["书值"]
            lt, lz = leap_of(T, y), leap_of(ZH, y)
            tui_v = f"闰{lt}" if lt else "无闰"
            zh_v = f"闰{lz}" if lz else "无闰"
        if z is not None and d:
            greg = ymd(z + d - 1)
        if r.get("节气") and z is not None and d and y >= 1742:
            name = QI_ALIAS.get(r["节气"], r["节气"])
            jd = z + d - 1
            cands = [jieqi_days(y)[name], jieqi_days(y + 1)[name]]
            near = min(cands, key=lambda x: abs(x - jd))
            qi = f"{name}：后编 {ymd(near)}，" + ("合" if near == jd else f"不合（书 − 推 {jd - near:+d} 日，待核）")
        if item == "冬至距日" and z is not None and d and y >= 1742:
            jd = z + d - 1
            dz = jieqi_days(y)["冬至"]          # 冬至岁 y 之天正冬至（在 y−1 年十一月）
            n = jd - dz + 1
            qi = f"后编冬至 {ymd(dz)} 至此连首尾 {n} 日，书 {r['书值']}，" + ("合" if str(n) == r["书值"] else "不合（待核）")
        lines.append(dict(来源=src, 编号=r["编号"], 篇名=r.get("篇名", ""), 纪年=f"{r['年号']}{r['年号年']}年" if r["年号"] else "",
                          干支年=r["干支年"], 农历年=y or "", 月=("闰" if leap else "") + (str(m) if m else ""), 日=r["日"],
                          日干支=r["日干支"], 验项=item, 书=obs, 推步法=meth, 推步=tui_v, 张表=zh_v, 月内=inmonth, 公历=greg,
                          节气=qi, 原文=r["原文"], 出处=f"第{r['册']}册 PDF {r['PDF页']}／书页 {r['书页']}", 备注=r["备注"], 分类="",
                          史源=r.get("史源", "")))
    # 分类：同年同月同验项之独立条合为一组
    groups = defaultdict(list)
    for x in lines:
        if x["验项"] in ("日干支", "朔干支", "月大", "闰月") and x["书"]:
            key = (x["农历年"], x["月"] if x["验项"] != "闰月" else "", x["验项"] if x["验项"] in ("月大", "闰月") else "朔")
            groups[key].append(x)
    cats = []
    for key, xs in groups.items():
        tv, zv = xs[0]["推步"], xs[0]["张表"]
        # 同一史源（如《中山世譜》蔡鐸本与蔡温本同文）只算一条独立证据
        seen, obs = set(), []
        for x in xs:
            src_key = x.get("史源") or x["编号"]
            if src_key not in seen:
                seen.add(src_key)
                obs.append(x["书"])
        grade = grade_same if tv == zv else grade_diff
        cat, note = fl.classify(tv, zv, obs, grade, what)
        if cat is None:
            cat, note = "无张表或推步", ""
        else:
            cats.append((cat, grade_same))
        for x in xs:
            x["分类"] = cat + ("；" + note if note else "")
    for x in lines:
        if x["分类"]:
            continue
        if x["验项"] in ("日干支", "朔干支") and x["日干支"] and not x["书"]:
            same = x["推步"] == x["张表"]
            x["分类"] = "无证（只书日干支，不能定朔）" if same else "③（只书日干支）"
            cats.append(("无证" if same else "③", grade_same))
        elif x["验项"] in ("节气", "冬至距日"):
            x["分类"] = "节气另检（只比后编）"
        else:
            x["分类"] = "无可验"
    return lines, cats


def main():
    out_rows, md_secs, summary = [], [], []
    for src, path, title in SOURCES:
        R = rows(path)
        if not R:
            continue
        lines, cats = check_rows(R, src)
        out_rows += lines
        n_none = sum(1 for x in lines if x["分类"] == "无可验")
        n_qi = sum(1 for x in lines if x["节气"])  # 节气与冬至距日
        qi_ok = sum(1 for x in lines if x["节气"].endswith("合"))
        summary.append((src, len(lines), len(lines) - n_none, n_none, cats, n_qi, qi_ok))
        cols = ["编号", "篇名", "纪年", "干支年", "农历年", "月", "日", "日干支", "验项", "书", "推步法", "推步", "张表", "月内", "公历",
                "节气", "分类", "出处", "原文", "备注"]
        sec = [f"## {title}", "",
               f"共 {len(lines)} 条：可验 {len(lines) - n_none} 条，无可验 {n_none} 条。按项分类（同月同项数条合为一类）："
               + fl.headline(cats, "推步", "项") + "。" + (f"节气与冬至距日数 {n_qi} 条（只比后编），合 {qi_ok} 条。" if n_qi else ""), ""]
        sec += fl.grade_table(cats) + ["", "| " + " | ".join(cols) + " |", "|" + "---|" * len(cols)]
        sec += ["| " + " | ".join(str(x[c]).replace("|", "／") for c in cols) + " |" for x in lines]
        md_secs += sec + [""]
    with open(OUT_CSV, "w", encoding="utf-8", newline="") as fh:
        fh.write("# 琉球当时文字纪日与张表、推步比较（scripts/calendar_data/ryukyu_jiri_check.py 生成，勿手改）。录文见 "
                 "data/calendar/ryukyu_beiwen.csv、ryukyu_beishang_zaji.csv。\n")
        w = csv.DictWriter(fh, fieldnames=list(out_rows[0].keys()))
        w.writeheader()
        w.writerows(out_rows)
    md = ["# 琉球碑文与日记纪日检验（自动生成）", "",
          "> 由 `scripts/calendar_data/ryukyu_jiri_check.py` 生成，勿手改。录文 `data/calendar/ryukyu_beiwen.csv`、"
          "`data/calendar/ryukyu_beishang_zaji.csv`，逐条结果 `data/calendar/ryukyu_jiri_check.csv`；书目与结论见 "
          "[ryukyu-sources.md](ryukyu-sources.md)。", "",
          "## 方法", "",
          "- 张表：张培瑜表（经 ChineseCalendar 转录，`vendor/calendar/chinese_calendar/实历朔日表.csv`）。推步：明大统（《大统曆日通軌》立成）、"
          "清西洋新法（用时）、《历象考成》（平时）、《历象考成后编》（用时京师），按年取当时行用之法；康熙五至八年取大统。",
          "- 验项：书「某月某日干支」者朔 = 日干支 − (日 − 1)；书「某月某干支朔」者即朔；只书日干支者看其在两家该月第几日；"
          "书「某月三十日」者验月大；书「闰某月」者验该年闰月；只书年月或有日无干支者无可验（仍逐条列出）。",
          "- " + fl.RULE + " 碑文、日记为当时文字（B 级）；两家互异之项，同月独立之条不足两条者不判张表或推步之误，归 ③ 并注倾向。",
          "- 节气（日记）只与后编用时京师比较（张表不载节气），合否另列，不入分类。琉球奉清（明）正朔，此处只问合不合中原颁历。", "",
          "## 汇总", ""]
    for src, n, nc, nn, cats, nq, qok in summary:
        md.append(f"- {src}：{n} 条，可验 {nc}，无可验 {nn}；" + fl.headline(cats, "推步", "项") + "。"
                  + (f"节气与冬至距日数 {nq} 条，合后编 {qok}。" if nq else ""))
    md += [""] + md_secs
    OUT_MD.write_text("\n".join(md) + "\n", encoding="utf-8")
    for src, n, nc, nn, cats, nq, qok in summary:
        print(src, n, nc, nn, fl.headline(cats, "推步", "项"), nq, qok)
    for x in out_rows:
        if x["分类"] not in ("无可验",) and not x["分类"].startswith(("有证", "无证")) or (x["节气"] and not x["节气"].endswith("合")):
            print("  ", x["来源"], x["编号"], x["纪年"], x["月"], x["日"], x["日干支"], x["验项"], "书", x["书"], "推", x["推步"], "张",
                  x["张表"], x["分类"], x["节气"])


if __name__ == "__main__":
    main()
