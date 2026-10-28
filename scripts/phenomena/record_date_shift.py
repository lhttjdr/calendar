"""史书月、五星凌犯记录的「日期偏移」检验：记载日期附近天象不合时，是否换一个日期就合。逐条写出
data/phenomena/checks/<book>_date_shift.csv，由 scripts/phenomena/checks_to_md.py 收入 doc/astronomy/phenomena-checks.md。

两部分：
1. 行星记录（不含月；月的 ±3 日见 rust/core sky_culture::tests_records）：在记载日期前后 4 年内逐日求行星与目标（星官取
   陈卓体系各成员中最近者；目标为行星时取两行星）地心角距，列出角距 < 1° 的时段（距记载日的日数），以及距记载日最近的一段。
   若记载只是年、月有误（如年号归属错一年），会在 ±1 年、±1 月附近出现一段；若四年内都没有或只有零散几段，换日期也救不回来。
   用地心天体测量位置（不计光行差、站心视差、可见性），只看度级，与 tests_records 的站心可见性推算互为补充。
2. 日干支不在所记之月的记录（只用历法）：同一干支在前一月、后一月、前一年同月、后一年同月是否在月内。

历表 DE441（缺则 DE406）；恒星 data/stars/hipparcos_bright.csv（I/311，历元 J1991.25，计自行）；星官 data/sky_culture/chinese_asterisms.csv。
用法（仓库根）：uv run --no-project --with skyfield python -I scripts/phenomena/record_date_shift.py
"""
import csv
import os
import sys
from pathlib import Path

import numpy as np
from skyfield.api import Star, load, load_file

sys.path.insert(0, str(Path("scripts/calendar_data").resolve()))
import gaiyuan_date as g  # noqa: E402

BOOKS = [("hanshu", "data/phenomena/hanshu_star_records.csv"), ("houhanshu", "data/phenomena/houhanshu_star_records.csv"),
         ("jinshu", "data/phenomena/jinshu_star_records.csv"), ("songshu", "data/phenomena/songshu_star_records.csv"),
         ("nanqishu", "data/phenomena/nanqishu_star_records.csv"),
         ("weishu", "data/phenomena/weishu_star_records.csv"), ("suishu", "data/phenomena/suishu_star_records.csv"),
         ("xintangshu", "data/phenomena/xintangshu_star_records.csv"), ("jiutangshu", "data/phenomena/jiutangshu_star_records.csv"),
         ("jiuwudaishi", "data/phenomena/jiuwudaishi_star_records.csv"),
         ("xinwudaishi", "data/phenomena/xinwudaishi_star_records.csv"),
         ("songshi", "data/phenomena/songshi_star_records.csv"),
         ("jinshi", "data/phenomena/jinshi_star_records.csv"),
         ("yuanshi", "data/phenomena/yuanshi_star_records.csv"),
         ("mingshi", "data/phenomena/mingshi_star_records.csv"),
         ("qingshigao", "data/phenomena/qingshigao_star_records.csv")]
SPAN = 4 * 365
NEAR_DEG = 1.0

ts = load.timescale(builtin=True)
bsp = "vendor/jpl/de441/de441.bsp" if os.path.exists("vendor/jpl/de441/de441.bsp") else "vendor/jpl/de406/de406.bsp"
eph = load_file(bsp)
earth = eph["earth"]
PLANET = {"Mercury": 1, "Venus": 2, "Mars": 4, "Jupiter": 5, "Saturn": 6}


def rows(path):
    with open(path, encoding="utf-8") as f:
        return list(csv.DictReader(line for line in f if not line.startswith("#")))


stars = {int(r["hip"]): r for r in rows("data/stars/hipparcos_bright.csv")}
members = {r["id"]: r["members"].split() for r in rows("data/sky_culture/chinese_asterisms.csv") if r["system"] == "chenzhuo"}


def star(hip):
    r = stars[hip]
    return Star(ra_hours=float(r["ra_deg"]) / 15.0, dec_degrees=float(r["dec_deg"]), ra_mas_per_year=float(r["pmra_mas_yr"] or 0),
                dec_mas_per_year=float(r["pmdec_mas_yr"] or 0), parallax_mas=max(float(r["plx_mas"] or 0), 1e-3),
                epoch=ts.tt_jd(2448349.0625))


def windows(days, near):
    out = []
    for d in days[near]:
        if out and d - out[-1][1] <= 3:
            out[-1][1] = d
        else:
            out.append([d, d])
    return out


def planet_shift(r):
    bodies = r["bodies"].split()
    if "Moon" in bodies or not all(b in PLANET for b in bodies):
        return None
    if r["target_kind"] == "body" and r["target_id"] in PLANET:
        targets = [eph[PLANET[r["target_id"]]]]
    elif r["target_kind"] == "asterism":
        ids = [x for x in r["target_id"].split("+") for x in members.get(x, [])]
        hips = [int(x) for x in ids if x.isdigit() and int(x) in stars]
        if not hips:
            return None
        targets = [star(h) for h in hips]
    elif len(bodies) >= 2:  # 多天体相合：天体两两最大角距
        targets = None
    else:
        return None
    j0 = (int(r["jdn_start"]) + int(r["jdn_end"])) // 2
    days = np.arange(-SPAN, SPAN + 1)
    t = ts.tt_jd(j0 + days)
    e = earth.at(t)
    pos = [e.observe(eph[PLANET[b]]) for b in bodies]
    if targets is None:
        sep = np.max([pos[i].separation_from(pos[k]).degrees for i in range(len(pos)) for k in range(i + 1, len(pos))], axis=0)
    else:
        tp = [e.observe(x) for x in targets]
        sep = np.max([np.min([p.separation_from(q).degrees for q in tp], axis=0) for p in pos], axis=0)
    w = windows(days, sep < NEAR_DEG)
    best = min(w, key=lambda a: min(abs(a[0]), abs(a[1])), default=None)
    i0 = SPAN
    return {
        "记载日角距°": f"{sep[i0]:.2f}",
        "±30 日内最小角距°": f"{sep[i0 - 30:i0 + 31].min():.2f}",
        "±4 年内 <1° 时段数": str(len(w)),
        "最近一段（距记载日，日）": "" if best is None else f"{best[0]:+d}…{best[1]:+d}",
        "全部时段": "；".join(f"{a:+d}…{b:+d}" for a, b in w),
    }


def table_of(r):
    """该记录记日所用历表：取记录表 calendar 列（zhi_star_records.py 按政权写入；「推步:政权」→ 推步政权朔日表）。"""
    c = r.get("calendar") or ""
    return c.split(":", 1)[1] if c.startswith("推步:") else "主表"


def ganzhi_fit(r):
    """干支不在月：同一干支落在哪些邻近的月（表月，建寅）。"""
    if r["ganzhi"] not in g.GANZHI or r["in_month"] == "1" or not r["month"].isdigit():  # 以日序记日者（「26日」）不适用
        return None
    y, m = int(r["year"]), int(r["month"])
    fits = []
    for label, yy, mm in (("前一月", y, m - 1), ("后一月", y, m + 1), ("前一年同月", y - 1, m), ("后一年同月", y + 1, m)):
        if mm < 1:
            yy, mm = yy - 1, 12
        if mm > 12:
            yy, mm = yy + 1, 1
        try:
            s, e = g.calendar_month_span(table_of(r), yy, mm, False)
        except KeyError:
            continue
        off = (g.GANZHI.index(r["ganzhi"]) - (s + 49) % 60) % 60
        if s + off < e:
            fits.append(label)
    return "、".join(fits) or "均不在"


def main():
    for book, path in BOOKS:
        out = []
        for i, r in enumerate(rows(path), 1):
            ps, gf = planet_shift(r), ganzhi_fit(r)
            if ps is None and gf is None:
                continue
            rec = {"序": str(i), "纪年日期": f"{r['era'] or r.get('reign', '')}{r['year_in_era']}年{r['month']}月{r['ganzhi']}",
                   "公历(儒略)": r["julian"], "天体": r["bodies"], "记载": f"{r['verb']}{r['target_name']}", "干支可落之月": gf or ""}
            rec.update(ps or {k: "" for k in ("记载日角距°", "±30 日内最小角距°", "±4 年内 <1° 时段数", "最近一段（距记载日，日）", "全部时段")})
            rec["原文"] = r["text"][:60]
            out.append(rec)
        dst = Path(f"data/phenomena/checks/{book}_date_shift.csv")
        with open(dst, "w", encoding="utf-8", newline="") as f:
            f.write(f"# 记载日期偏移检验（scripts/phenomena/record_date_shift.py，{bsp.split('/')[-1]}）：行星记录在记载日 ±4 年内与目标地心角距 <1° 的时段；\n")
            f.write("# 日干支不在所记之月者，同一干支可落在的邻近月份（实历朔日表或该政权推步历，建寅）。序号同 data/phenomena/ 下的记录表行序。\n")
            w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
            w.writeheader()
            w.writerows(out)
        print(dst, len(out), "条")


if __name__ == "__main__":
    main()
