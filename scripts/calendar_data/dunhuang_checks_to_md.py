"""把敦煌历日换算的逐条检验写进 doc/calendar/dunhuang-calendar.md 末尾（标记行以下整段重写）。

输入：data/calendar/dunhuang_shuori_check.csv、data/calendar/dunhuang_years.csv（rust/core/examples/dunhuang_check.rs）、
data/calendar/dunhuang_fallback_check.csv（scripts/phenomena/dunhuang_fallback_check.py）。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/dunhuang_checks_to_md.py
"""
import csv
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOC = ROOT / "doc/calendar/dunhuang-calendar.md"
MARK = "<!-- 以下由 scripts/calendar_data/dunhuang_checks_to_md.py 生成，勿手改 -->"


def rows(name):
    with open(ROOT / "data/calendar" / name, encoding="utf-8") as f:
        return list(csv.DictReader(line for line in f if not line.startswith("#")))


def table(head, body):
    out = ["| " + " | ".join(head) + " |", "|" + "---|" * len(head)]
    out += ["| " + " | ".join(str(c) for c in r) + " |" for r in body]
    return "\n".join(out)


def main():
    chk, yrs, fb = rows("dunhuang_shuori_check.csv"), rows("dunhuang_years.csv"), rows("dunhuang_fallback_check.csv")
    n = len(chk)
    ok = sum(r["朔合"] == "合" for r in chk)
    nsz = sum(r["大小合"] != "" for r in chk)
    oksz = sum(r["大小合"] == "合" for r in chk)
    strict = sum(r["只认有据亦合"] == "合" for r in chk)
    orig = sum(r["朔依据"] == "原件" for r in chk)
    parts = [MARK, "", "## 逐条检验：比较表每一格的换算重现", "",
             f"比较表敦煌曆一栏有朔之格 {n}（原件 {orig}、原书推算 {n - orig}；含次年正月重出于两件者 3 格）。按敦煌岁月查月首"
             f"（`lifa::resolve_dunhuang_month`，退回主表）：**朔干支合 {ok}/{n}**；原书月大小可验者 {nsz}，自月首逐日换算到次月初一"
             f"（`lifa::resolve_dunhuang_day`）所得月长**合 {oksz}/{nsz}**；只认有据之月（`EvidenceOnly`）月首亦合 {strict}/{n}。"
             "测试 `calendar::lifa::tests::dunhuang_comparison_table_reproduced` 对两种取舍逐格断言。", "",
             table(["文书", "原书定名", "中原月序", "敦煌岁", "敦煌月", "原书朔", "依据", "原书大小", "换算月首（儒略历）", "换算朔",
                    "朔", "换算月长", "大小", "书页"],
                   [[r["文书"], r["原书定名"], r["中原月序"], r["敦煌岁"], r["敦煌月"], r["原书朔"], r["朔依据"], r["原书大小"],
                     r["换算月首"], r["换算朔"], r["朔合"], r["换算月长"], r["大小合"] or "—", r["书页"]] for r in chk]), "",
             "## 各岁纪年与闰月", "",
             "「通行纪年」：吐蕃时期地支纪年，归义军时期为年号表主线的年末年号（`era_names::dunhuang_year_name`）；"
             "「出土历日定名」为邓文宽对该岁历日的定名，多照原件历首。闰月一栏为拼合后敦煌历的闰月（有据者从敦煌，否则从主表）。", "",
             table(["岁", "干支", "时期", "通行纪年", "出土历日定名", "原件之月", "推算之月", "敦煌闰月", "主表闰月"],
                   [[r["岁"], r["干支"], r["时期"], r["通行纪年"], r["出土历日定名"] or "—", r["原件之月"], r["推算之月"],
                     r["敦煌闰月"] or "—", r["主表闰月"] or "—"] for r in yrs]), ""]
    stat = defaultdict(lambda: (Counter(), Counter()))
    for r in fb:
        p = "吐蕃" if int(r["公元"]) < 848 else "归义军"
        stat[p][0][int(r["敦煌−主表"])] += 1
        stat[p][1][int(r["敦煌−合朔日"])] += 1
    body = []
    for p, (a, b) in stat.items():
        for name, c in (("主表朔", a), ("实际合朔日", b)):
            m = sum(c.values())
            body.append([p, name, m, f"{c[0]}（{c[0] / m:.0%}）", f"{sum(v for k, v in c.items() if abs(k) <= 1)}（{sum(v for k, v in c.items() if abs(k) <= 1) / m:.0%}）",
                         " ".join(f"{k:+d}:{v}" for k, v in sorted(c.items()))])
    parts += ["## 无据之月退回何表：原件之月与主表、实际合朔", "",
              "原件之月的敦煌朔与主表朔、与实际合朔（DE441，敦煌地方平时，东经 94.66°）所在日之差（敦煌 − 对方，日）。"
              "`scripts/phenomena/dunhuang_fallback_check.py` → `data/calendar/dunhuang_fallback_check.csv`。", "",
              table(["时期", "比较对象", "月数", "同日", "差一日以内", "分布"], body), "",
              table(["公元", "敦煌月", "敦煌朔", "敦煌−主表", "合朔日", "敦煌−合朔日", "合朔地方时", "文书"],
                    [[r["公元"], r["敦煌月"], r["敦煌朔"], r["敦煌−主表"], r["合朔日"], r["敦煌−合朔日"], r["合朔地方时"], r["文书"]] for r in fb]), ""]
    text = DOC.read_text(encoding="utf-8")
    head = text.split(MARK)[0].rstrip() + "\n\n"
    DOC.write_text(head + "\n".join(parts), encoding="utf-8")
    print(f"写 {DOC.relative_to(ROOT)}：逐条 {n} 格，{len(yrs)} 岁，原件比较 {len(fb)} 月")


if __name__ == "__main__":
    main()
