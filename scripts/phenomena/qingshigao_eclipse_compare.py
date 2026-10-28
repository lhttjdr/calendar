"""《清史稿》日食所记时辰、食分与北京推算之比较（输入 data/phenomena/qingshigao_solar_eclipses_check.csv，
由 rust/core/examples/historical_eclipses.rs 生成）。

- 时辰：推算本地初亏、食甚、复圆的地方平太阳时加均时差（Spencer 1971 公式，误差约 1 分钟）得真太阳时，与所记时辰
  （子 23—1 时、丑 1—3 时……）比较；「时辰差」以食甚计，另列初亏、复圆是否落在所记时辰，并按 1742 年前后分段统计，
  以判别所记时辰指初亏还是食甚。
- 食分：所记食分（分，直径十分之一；少半太为 1/4、1/2、3/4，强弱按 ±1/12）与推算几何食分 ×10 之差。
所记多为钦天监按时宪历预推之数，比较所得为时宪历推步精度。
输出 data/phenomena/checks/qingshigao_eclipses.csv，由 checks_to_md.py 收入 doc/astronomy/phenomena-checks.md。

用法（仓库根）：uv run --project scripts python scripts/phenomena/qingshigao_eclipse_compare.py
"""
import csv
import datetime
import math
import statistics
from pathlib import Path

SHI = "子丑寅卯辰巳午未申酉戌亥"


def eot_minutes(y, m, d):
    n = datetime.date(y, m, d).timetuple().tm_yday
    b = 2 * math.pi * (n - 1) / 365
    return 229.18 * (0.000075 + 0.001868 * math.cos(b) - 0.032077 * math.sin(b) - 0.014615 * math.cos(2 * b)
                     - 0.040849 * math.sin(2 * b))


def shi_of(h):
    return SHI[int(((h + 1) % 24) // 2)]


def main():
    rows = list(csv.DictReader(open("data/phenomena/qingshigao_solar_eclipses_check.csv", encoding="utf-8")))
    out = []
    for r in rows:
        y, m, d = map(int, r["julian"].split("-"))
        lmt = float(r["local_greatest_lmt_h"]) if r["local_greatest_lmt_h"] not in ("", "NaN", "nan") else None
        if lmt is None:
            out.append({"纪年": f"{r['era']}{r['year_in_era']}年{r['month']}月", "公历": r["julian"], "所记": f"{r['shi']}時 {r['extent']}",
                        "推算初亏": "", "推算食甚真太阳时": "北京无食", "推算复圆": "", "推算时辰": "", "时辰差": "",
                        "初亏在所记时辰": "", "复圆在所记时辰": "", "所记食分": r["fen"], "推算食分": "", "食分差": ""})
            continue
        eot = eot_minutes(y, m, d) / 60
        last = (lmt + eot) % 24
        s = shi_of(last)
        first = (float(r["local_first_lmt_h"]) + eot) % 24
        fin = (float(r["local_last_lmt_h"]) + eot) % 24
        diff = (SHI.index(s) - SHI.index(r["shi"]) + 6) % 12 - 6
        geo = float(r["site_magnitude_geometric"]) * 10
        out.append({"纪年": f"{r['era']}{r['year_in_era']}年{r['month']}月", "公历": r["julian"], "所记": f"{r['shi']}時 {r['extent']}",
                    "推算初亏": f"{int(first):02d}:{int(first % 1 * 60):02d}", "推算食甚真太阳时": f"{int(last):02d}:{int(last % 1 * 60):02d}",
                    "推算复圆": f"{int(fin):02d}:{int(fin % 1 * 60):02d}", "推算时辰": s, "时辰差": f"{diff:+d}",
                    "初亏在所记时辰": "是" if shi_of(first) == r["shi"] else "否", "复圆在所记时辰": "是" if shi_of(fin) == r["shi"] else "否",
                    "所记食分": r["fen"], "推算食分": f"{geo:.2f}", "食分差": f"{geo - float(r['fen']):+.2f}"})
    ok = [o for o in out if o["时辰差"]]
    dst = Path("data/phenomena/checks/qingshigao_eclipses.csv")
    with open(dst, "w", encoding="utf-8", newline="") as f:
        f.write("# 《清史稿》卷三十七日食在北京：所记时辰、食分与推算比较（scripts/phenomena/qingshigao_eclipse_compare.py）。食分以分计（直径十分之一）。\n")
        if ok:
            fd = [abs(float(o["食分差"])) for o in ok]
            f.write(f"# {len(out)} 条，北京有食 {len(ok)} 条：时辰相合 {sum(o['时辰差'] == '+0' for o in ok)}，差一时辰 "
                    f"{sum(o['时辰差'] in ('+1', '-1') for o in ok)}；|食分差| 中位 {statistics.median(fd):.2f} 分，≤1 分 {sum(x <= 1 for x in fd)}\n")
        for lab, sel in (("1742 年前", lambda o: int(o["公历"][:4]) < 1742), ("1742 年起", lambda o: int(o["公历"][:4]) >= 1742)):
            v = [o for o in ok if sel(o)]
            f.write(f"# {lab} {len(v)} 条：所记时辰含初亏 {sum(o['初亏在所记时辰'] == '是' for o in v)}、含食甚 {sum(o['时辰差'] == '+0' for o in v)}、"
                    f"含复圆 {sum(o['复圆在所记时辰'] == '是' for o in v)}；|食分差| 中位 {statistics.median(abs(float(o['食分差'])) for o in v):.2f} 分\n")
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    print(dst, len(out), "条")


if __name__ == "__main__":
    main()
