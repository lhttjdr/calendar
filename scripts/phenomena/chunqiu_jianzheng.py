"""春秋历各年岁首的建正：以 DE441 真冬至（曲阜地方平时）定子月，看春秋历正月是子、丑还是亥。
另算《左传》两条「日南至」（僖公五年正月辛亥朔、昭公二十年二月己丑朔）与真冬至之差。
春秋历是今人排定的（第 4d 级），这里的建正只说明「按此表则如何」。

输出 data/calendar/chunqiu_jianzheng.csv。
用法（仓库根）：uv run --project scripts --with skyfield python scripts/phenomena/chunqiu_jianzheng.py
"""
import csv
import os
import sys
from pathlib import Path

from skyfield import almanac
from skyfield.api import load, load_file

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "calendar_data"))
import chunqiu_check as c  # noqa: E402
import gaiyuan_date as g  # noqa: E402

LON = 116.98
ts = load.timescale(builtin=True)
bsp = "vendor/jpl/de441/de441.bsp" if os.path.exists("vendor/jpl/de441/de441.bsp") else "vendor/jpl/de406/de406.bsp"
eph = load_file(bsp)


def solstices(y0, y1):
    """y 年（天文纪年）末冬至所在地方民用日 JDN：{y: jdn}。儒略历 12 月下旬前后。"""
    t0, t1 = ts.ut1_jd(g.jdn(y0, 1, 1)), ts.ut1_jd(g.jdn(y1, 12, 31))
    times, kinds = almanac.find_discrete(t0, t1, almanac.seasons(eph))
    out = {}
    for t, k in zip(times, kinds):
        if k == 3:
            local = t.ut1 + LON / 360
            day = int(local + 0.5)
            out[g.from_jdn(day)[0] if g.from_jdn(day)[1] > 6 else g.from_jdn(day)[0] - 1] = day
    return out


def main():
    sol = solstices(-725, -470)
    rows = []
    start = c.DUKES
    for y in range(-721, -478):
        ms = c.chunqiu_year(y)
        ws = sol[y - 1]  # 岁首附近的冬至
        # 正月相对子月（含冬至之月）的偏移：0 子、1 丑、-1 亥
        off = next((k for k in range(-4, 5) if ms[0] - k * c.CQ_LUNAR <= ws < ms[0] - (k - 1) * c.CQ_LUNAR), None)
        duke = [n for n, s in start if s <= y][-1]
        rows.append({"年": y, "公": duke, "正月朔": g.ganzhi_of_jdn(ms[0]), "朔 JDN": ms[0], "真冬至 JDN": ws, "建": {0: "子", 1: "丑", -1: "亥", 2: "寅"}.get(off, str(off))})
    with open("data/calendar/chunqiu_jianzheng.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 春秋历正月建（scripts/phenomena/chunqiu_jianzheng.py）；真冬至取 DE441，曲阜地方平时。\n")
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)
    for n, _ in start:
        r = [x for x in rows if x["公"] == n]
        cnt = {k: sum(x["建"] == k for x in r) for k in ("亥", "子", "丑", "寅")}
        print(n, len(r), cnt)
    for name, y, gz in (("僖公五年正月辛亥朔日南至", -654, "辛亥"), ("昭公二十年二月己丑朔日南至", -521, "己丑")):
        ws = sol[y - 1]
        k = g.GANZHI.index(gz)
        j = min((ws + i for i in range(-30, 31) if (ws + i + 49) % 60 == k), key=lambda x: abs(x - ws))
        print(name, "记录日", g.from_jdn(j), "真冬至", g.from_jdn(ws), g.ganzhi_of_jdn(ws), "差", j - ws)


if __name__ == "__main__":
    main()
