"""敦煌无据之月退回何表：敦煌朔日表（data/calendar/敦煌朔日表.csv）原件之月，与张表朔、实际合朔所在日逐月比较。

实际合朔：DE441（无则 DE406）日月视黄经相等之时，换为敦煌地方平时（东经 94.66°）所在民用日；另记合朔地方时，
看敦煌历是否有进朔之迹。张表朔取与敦煌朔相距 3 日内的张表月首。只用原件之月（括号推算格不计）。
输出 data/calendar/dunhuang_fallback_check.csv，汇总印出，结论写在 doc/calendar/dunhuang-calendar.md「无据之月」。

用法（仓库根）：uv run --no-project --with skyfield python -I scripts/phenomena/dunhuang_fallback_check.py
"""
import csv
import os
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/calendar_data"))
import gaiyuan_date as g  # noqa: E402
from skyfield import almanac  # noqa: E402
from skyfield.api import load, load_file  # noqa: E402

LON = 94.66
ts = load.timescale(builtin=True)
bsp = ROOT / ("vendor/jpl/de441/de441.bsp" if os.path.exists(ROOT / "vendor/jpl/de441/de441.bsp") else "vendor/jpl/de406/de406.bsp")
eph = load_file(str(bsp))


def conjunction(jdn):
    t0, t1 = ts.ut1_jd(jdn - 4), ts.ut1_jd(jdn + 4)
    times, phases = almanac.find_discrete(t0, t1, almanac.moon_phases(eph))
    best = None
    for t, p in zip(times, phases):
        if p != 0:
            continue
        local = t.ut1 + LON / 360
        day = int(local + 0.5)
        hour = (local + 0.5 - day) * 24
        if best is None or abs(day - jdn) < abs(best[0] - jdn):
            best = (day, hour)
    return best


def main():
    starts = sorted(s for _, _, _, s in g._months())
    with open(ROOT / "data/calendar/敦煌朔日表.csv", encoding="utf-8") as f:
        recs = list(csv.DictReader(line for line in f if not line.startswith("#")))
    out, stat = [], defaultdict(lambda: {"张表": Counter(), "合朔": Counter()})
    for r in recs:
        if r["朔依据"] != "原件":
            continue
        dh = g.jdn(int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"]))
        main = min(starts, key=lambda s: abs(s - dh))
        day, hour = conjunction(dh)
        period = "吐蕃" if int(r["lunar_year"]) < 848 else "归义军"
        stat[period]["张表"][dh - main] += 1
        stat[period]["合朔"][dh - day] += 1
        out.append({"公元": r["lunar_year"], "敦煌月": ("閏" if r["is_leap"] == "1" else "") + str(r["lunar_month"]),
                    "敦煌朔": r["朔干支"], "敦煌−张表": dh - main, "合朔日": g.ganzhi_of_jdn(day),
                    "敦煌−合朔日": dh - day, "合朔地方时": f"{hour:.1f}", "文书": r["文书"]})
    with open(ROOT / "data/calendar/dunhuang_fallback_check.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 敦煌朔日表原件之月与张表朔、实际合朔（DE441，敦煌地方平时 94.66°E）逐月比较；"
                "scripts/phenomena/dunhuang_fallback_check.py 生成。\n")
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    for p, s in stat.items():
        for k, c in s.items():
            n = sum(c.values())
            print(f"{p} 敦煌−{k}：n={n} 同日 {c[0]}（{c[0] / n:.0%}） 分布 {dict(sorted(c.items()))}")
    # 合朔在敦煌朔前一日者，合朔地方时分布（进朔之迹）
    late = sorted(float(o["合朔地方时"]) for o in out if o["敦煌−合朔日"] == 1)
    same = sorted(float(o["合朔地方时"]) for o in out if o["敦煌−合朔日"] == 0)
    print("敦煌朔为合朔次日者合朔地方时：", late)
    print("敦煌朔即合朔日者合朔地方时最晚：", same[-5:])


if __name__ == "__main__":
    main()
