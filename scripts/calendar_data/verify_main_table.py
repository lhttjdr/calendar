"""以本纪的一手日期核验主表（实历朔日表，据张培瑜等复原）：

1. 明书朔日：本纪「某月某干支朔」（data/calendar/本纪纪日/<书>.csv 中 position = 朔）与主表该月朔日逐条比对，不合者记日差；
   有同系推步表者（北朝、吴蜀等，discriminating_records.FAMILY）一并比对；
2. 闰位：本纪「閏月」所接之月（<书>_閏月.csv）与主表当年闰位比对（本纪常略书月份，閏月只保证在所记最后一月之后，主表闰位更晚者为略书相容）。
本纪月份更替常不书，故不用「在月率」评主表；明书朔日与闰位不受此影响。
输出 data/calendar/main_table_check.csv（逐条）与 main_table_leaps.csv，并按朝代汇总。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/verify_main_table.py
"""
import csv
import glob
import re
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import discriminating_records as d  # noqa: E402
import gaiyuan_date as g  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
DYN = {"西汉": "汉", "东汉": "汉", "魏": "三国", "蜀": "三国", "吴": "三国", "晋": "晋", "宋": "南朝", "齐": "南朝", "梁": "南朝", "陈": "南朝",
       "北魏": "北朝", "东魏": "北朝", "西魏": "北朝", "北齐": "北朝", "北周": "北朝", "隋": "隋", "唐": "唐", "武周": "唐",
       "后梁": "五代", "后唐": "五代", "后晋": "五代", "后汉": "五代", "后周": "五代", "宋": "宋", "辽": "辽", "金": "金", "元": "元", "明": "明", "清": "清"}


def dyn_of(regime, year):
    """政权 → 汇总朝代；刘宋与赵宋同名「宋」，按年分（960 年前为南朝）。"""
    return "南朝" if regime == "宋" and year < 960 else DYN.get(regime, regime)


ORDER = ("汉", "三国", "晋", "南朝", "北朝", "隋", "唐", "五代", "宋", "辽", "金", "元", "明", "清")


def kind(main, t, gz, diff):
    """不合之类：朔差一二日（朔日推算之差，主表或本纪有一误）；差一月（干支为主表邻月或同年闰月之朔，闰位或月序不同）；
    其他（多为干支传写之误或解析误定年月，不作主表证据）。"""
    if abs(diff) <= 2:
        return "朔差一二日"
    y, m, lp = t
    near = [(y, m - 1, False), (y, m + 1, False), (y, m, not lp), (y, m - 1, True), (y + 1, 1, False), (y - 1, 12, False)]
    if any(k in main and g.ganzhi_of_jdn(main[k]) == gz for k in near):
        return "差一月"
    return "其他"


def main():
    tabs = d.tables()
    out, stat = [], defaultdict(lambda: defaultdict(lambda: [0, 0, 0, 0, 0]))
    for path in sorted(glob.glob(str(ROOT / "data/calendar/本纪纪日/*.csv"))):
        if path.endswith("_閏月.csv"):
            continue
        book = Path(path).stem
        seen = set()
        for r in csv.DictReader(open(path, encoding="utf-8")):
            if r.get("position") != "朔" or not r["month"].lstrip("闰").isdigit():
                continue
            y, lab, regime = int(r["year"]), r["month"], r["regime"]
            key = (y, lab, r["ganzhi"])
            if key in seen:
                continue
            seen.add(key)
            leap, num = lab.startswith("闰"), int(lab.lstrip("闰"))
            t = g.table_month(regime, y, g.MONTH_NAMES[num - 1], leap) or (y, num, leap)
            row = {"书": book, "朝代": dyn_of(regime, y), "政权": regime, "年": y, "纪年": f"{r['era']}{r['year_in_era']}年",
                   "月": lab, "本纪朔": r["ganzhi"]}
            fam = d.FAMILY.get(regime)
            for col, name in (("主表", "主表"), ("本项目推步", f"推步:{fam}" if fam else None), ("上游推步", f"上游:{fam}" if fam else None)):
                s = tabs.get(name, {}).get(t) if name else None
                if s is None:
                    row[col] = ""
                    continue
                diff = (g.GANZHI.index(r["ganzhi"]) - (s + 49) % 60 + 30) % 60 - 30
                row[col] = "合" if diff == 0 else f"差{diff:+d}日"
                st = stat[row["朝代"]][col]
                st[0] += 1
                st[1] += diff == 0
                if col == "主表":
                    row["主表朔 JDN"], row["本纪朔 JDN"] = s, s + diff
                if col == "主表" and diff:
                    row["类"] = kind(tabs["主表"], t, r["ganzhi"], diff)
                    st[{"朔差一二日": 2, "差一月": 3}.get(row["类"], 4)] += 1
            row.setdefault("类", "")
            row["日食"] = "是" if re.search(r"日有?[蝕食]|日蝕|奏言日蝕", r["text"]) else ""
            row.setdefault("主表朔 JDN", "")
            row.setdefault("本纪朔 JDN", "")
            row["_t"] = t
            row["原文"] = r["text"][:40]
            out.append(row)
    # 旁证：他书同月记日干支，在主表该月内者几条、在「以本纪所书朔起算的 30 日」内者几条（只对不合者）
    others = defaultdict(list)
    for path in glob.glob(str(ROOT / "data/calendar/本纪纪日/*.csv")):
        if path.endswith("_閏月.csv"):
            continue
        for r in csv.DictReader(open(path, encoding="utf-8")):
            lab = r["month"]
            if not lab.lstrip("闰").isdigit():
                continue
            y, leap, num = int(r["year"]), lab.startswith("闰"), int(lab.lstrip("闰"))
            t = g.table_month(r["regime"], y, g.MONTH_NAMES[num - 1], leap) or (y, num, leap)
            others[(dyn_of(r["regime"], y), t)].append((Path(path).stem, g.GANZHI.index(r["ganzhi"])))
    sp = d.spans(tabs["主表"])
    for o in out:
        o["旁证（主表/本纪朔）"] = ""
        if not o["类"] or o["_t"] not in sp:
            continue
        s0, e0 = sp[o["_t"]]
        rec = s0 + int(o["主表"][1:-1])
        gzs = [x for b, x in others[(o["朝代"], o["_t"])] if b != o["书"]]
        if gzs:
            in_main = sum((x - (s0 + 49)) % 60 < e0 - s0 for x in gzs)
            in_rec = sum((x - (rec + 49)) % 60 < 30 for x in gzs)
            o["旁证（主表/本纪朔）"] = f"{in_main}/{in_rec}（共{len(gzs)}）"
    for o in out:
        del o["_t"]
    # 他书同记：同年同月同朔干支见于另一书（如两唐书），两条独立记录相合，较单条可信
    by = defaultdict(set)
    for o in out:
        by[(o["年"], o["月"], o["本纪朔"])].add(o["书"])
    for o in out:
        o["他书同记"] = "、".join(sorted(by[(o["年"], o["月"], o["本纪朔"])] - {o["书"]}))
    with open(ROOT / "data/calendar/main_table_check.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 本纪明书朔日与主表（及同系推步表）比对（scripts/calendar_data/verify_main_table.py）；差 = 本纪朔 − 该表朔（日）。\n")
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    print("一、明书朔日")
    for dyn in ORDER:
        cells = "；".join(f"{col} {ok}/{n}" + (f"（不合者：朔差一二日 {a}、差一月 {b}、其他 {c}）" if col == "主表" else "")
                         for col, (n, ok, a, b, c) in stat[dyn].items())
        print(f"  {dyn}：{cells}")
    # 闰位
    leaps = {name: {k[0]: k[1] for k in tab if k[2]} for name, tab in tabs.items()}
    lout, lstat = [], defaultdict(lambda: [0, 0, 0])
    for path in sorted(glob.glob(str(ROOT / "data/calendar/本纪纪日/*_閏月.csv"))):
        book = Path(path).stem.replace("_閏月", "")
        for r in csv.DictReader(open(path, encoding="utf-8")):
            y, after, regime = int(r["year"]), int(r["leap_after"]), r["regime"]
            lp = leaps["主表"].get(y)
            # 本纪只保证闰月在所记最后一月之后：主表闰位更晚为「略书相容」（宋元嘉二十七年「冬閏月」承七月），更早或无闰才是矛盾
            v = "合" if lp == after else ("略书相容" if lp and lp > after else ("主表无闰" if not lp else f"主表闰{lp}（早于所记之月）"))
            dyn = dyn_of(regime, y)
            lstat[dyn][0] += 1
            lstat[dyn][1] += v == "合"
            lstat[dyn][2] += v == "略书相容"
            lout.append({"书": book, "朝代": dyn, "年": y, "纪年": f"{r['era']}{r['year_in_era']}年", "本纪闰位": f"闰{after}",
                         "主表": v, "原文": r["text"][:40]})
    with open(ROOT / "data/calendar/main_table_leaps.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 本纪「閏月」所接之月与主表闰位（scripts/calendar_data/verify_main_table.py）；「略书相容」= 主表闰位晚于所记之月（本纪不书其间月份）；「早于所记之月」或「主表无闰」为矛盾。\n")
        w = csv.DictWriter(f, fieldnames=list(lout[0].keys()))
        w.writeheader()
        w.writerows(lout)
    print("二、閏月位置（主表）")
    for dyn in ORDER:
        n, ok, maybe = lstat[dyn]
        print(f"  {dyn}：{n} 条，合 {ok}，略书相容 {maybe}，矛盾 {n - ok - maybe}")


if __name__ == "__main__":
    main()
