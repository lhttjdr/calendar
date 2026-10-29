"""邓文宽《敦煌天文历法文献辑校》附录一比较表（data/calendar/敦煌历日_比较表.csv）的校验与敦煌历—主表逐月比较。

1. 转录自校：同一表内相邻两月朔日之差须合前月大小（大 30、小 29 日），中原曆一栏须与主表（实历朔日表）逐格相合
   （原书中原曆据陈垣《廿史朔闰表》，与主表同源）；
2. 敦煌曆一栏（不含括号推算之格）与主表逐月比较：日差（敦煌朔 − 主表朔）、闰位异同，按时期（吐蕃、归义军）汇总。
敦煌历是当地自造之历（吐蕃占领与归义军时期与中原隔绝），为独立历表；本表只作比较，不据以改主表。
输出 data/calendar/dunhuang_vs_main.csv（敦煌曆有据之月逐条），由 main_table_checks_to_md.py 汇入 doc/calendar/main-table-checks.md。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/check_dunhuang_comparison.py
"""
import csv
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import discriminating_records as d  # noqa: E402
import gaiyuan_date as g  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
N = {"正": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9, "十": 10, "十一": 11, "十二": 12}


def rows():
    with open(ROOT / "data/calendar/敦煌历日_比较表.csv", encoding="utf-8") as f:
        return list(csv.DictReader(line for line in f if not line.startswith("#")))


def gz(s):
    """「(壬子)」「水(癸)未」→ 干支；空或无法识读返回 None。"""
    s = s.replace("水(癸)", "癸").strip("()")
    return s if s in g.GANZHI else None


def table_key(r):
    """行月序（中原曆月序）→ 主表键 (年, 月, 闰)。"""
    y, lab = int(r["公元"]), r["行月序"]
    if lab.startswith("上年"):
        y, lab = y - 1, lab[2:]
    m = re.fullmatch(r"(閏)?(.+)月", lab)
    return y, N[m[2]], bool(m[1])


def period(y):
    return "北魏" if y < 500 else ("吐蕃" if y < 848 else "归义军")


def main():
    rs, main_t = rows(), d.tables()["主表"]
    # 1. 自校
    bad = []
    by_page = defaultdict(list)
    for r in rs:
        by_page[r["书页"]].append(r)
    for page, prs in by_page.items():
        for col_s, col_g in (("中原大小", "中原朔"), ("敦煌大小", "敦煌朔")):
            for a, b in zip(prs, prs[1:]):
                sa, ga, gb = a[col_s].strip("()"), gz(a[col_g]), gz(b[col_g])
                if sa in ("大", "小") and ga and gb and (g.GANZHI.index(gb) - g.GANZHI.index(ga)) % 60 != (30 if sa == "大" else 29):
                    bad.append((page, col_g, a["公元"], a["行月序"]))
    zy = sum(gz(r["中原朔"]) == g.ganzhi_of_jdn(main_t[table_key(r)]) for r in rs)
    print(f"转录自校：月大小与朔差不合 {len(bad)} 处 {bad}；中原曆与主表合 {zy}/{len(rs)}")
    # 2. 敦煌曆有据之月与主表
    out, stat, years = [], defaultdict(Counter), defaultdict(Counter)
    for r in rs:
        v = gz(r["敦煌朔"])
        if not v:
            continue
        inferred = r["敦煌朔"].startswith("(")
        k = table_key(r)
        s = main_t[k]
        diff = (g.GANZHI.index(v) - (s + 49)) % 60
        diff = diff - 60 if diff > 30 else diff
        y = int(r["公元"])
        dh_label = r["敦煌月序"] or r["行月序"].replace("上年閏十二月", "正月")
        out.append({"文书": r["文书"], "纪年": r["纪年"], "公元": y, "时期": period(y), "主表月": r["行月序"], "敦煌月": dh_label,
                    "敦煌朔": v, "推算": "是" if inferred else "", "主表朔": g.ganzhi_of_jdn(s), "日差": diff,
                    "敦煌朔 JDN": s + diff, "书页": r["书页"]})
        if not inferred:
            stat[period(y)]["n"] += 1
            stat[period(y)][diff] += 1
            years[(y, r["文书"], r["书页"])][diff] += 1
    with open(ROOT / "data/calendar/dunhuang_vs_main.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 敦煌历日（邓文宽 1996 附录一）有据之月与主表比较（scripts/calendar_data/check_dunhuang_comparison.py）；"
                "日差 = 敦煌朔 − 主表朔；推算 = 原书括号格（据残历条件推得），不计入统计。\n")
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    for p, c in stat.items():
        n = c.pop("n")
        print(f"{p}：有据之月 {n}，日差分布 {dict(sorted(c.items()))}")
    # 闰位：敦煌月序带「閏」者与主表闰位
    for r in rs:
        if "閏" in (r["敦煌月序"] or "") or ("閏" in r["行月序"] and r["敦煌月序"] and "閏" not in r["敦煌月序"]):
            print("闰位异同：", r["公元"], "主表", r["行月序"], "↔ 敦煌", r["敦煌月序"])


if __name__ == "__main__":
    main()
