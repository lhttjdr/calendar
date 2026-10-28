"""《清史稿》卷三十七月与五星「犯」「距」：按北京民用日全天求地心最小角距（DE441/DE406，skyfield）。

卷首：钦天监「每年預推月五星入此限者，繕冊進呈」（相距书），所记为推算的地心相距，不论昼夜可见与否；
tests_records.rs 只在天黑且天体在地平以上时求角距，对白天发生者给出数度之差。本脚本取全天（289 点，5 分钟一点）地心最小角距。
输出 data/phenomena/checks/qingshigao_moon_fullday.csv，由 checks_to_md.py 收入 doc/astronomy/phenomena-checks.md。

用法（仓库根）：uv run --no-project --with skyfield python -I scripts/phenomena/qingshigao_moon_fullday.py
"""
import csv, numpy as np, os, statistics
from skyfield.api import load, load_file
bsp = "vendor/jpl/de441/de441.bsp" if os.path.exists("vendor/jpl/de441/de441.bsp") else "vendor/jpl/de406/de406.bsp"
eph = load_file(bsp); ts = load.timescale()
earth = eph["earth"]; names = {"Moon": "moon", "Mars": "mars barycenter", "Venus": "venus barycenter", "Jupiter": "jupiter barycenter", "Saturn": "saturn barycenter", "Mercury": "mercury barycenter"}
recs = list(csv.DictReader(open("data/phenomena/qingshigao_star_records.csv")))
lines = [l for l in open("data/phenomena/checks/qingshigao_star_records.csv") if not l.startswith("#")]
out = []
for r in csv.DictReader(lines):
    rec = recs[int(r["序"]) - 1]
    if rec["juan"] != "037" or not rec["bodies"].startswith("Moon") or rec["target_kind"] != "body" or rec["in_month"] != "1":
        continue
    jd0 = int(rec["jdn_start"]) - 0.5 - 116.39 / 360  # 北京民用日 0 时（UT 近似）
    t = ts.ut1_jd(jd0 + np.linspace(0, 1, 289))
    a = earth.at(t).observe(eph["moon"]).apparent(); b = earth.at(t).observe(eph[names[rec["target_id"]]]).apparent()
    sep = a.separation_from(b).degrees
    night = r["天体间最大角距°"].split("（")[0]
    out.append((float(sep.min()), night, rec["text"][:24], r["序"], r["纪年日期"], r["公历(儒略)"], r["记载"], float(sep.argmin()) * 24 / 288))
mins = [x[0] for x in out]
from pathlib import Path
dst = Path("data/phenomena/checks/qingshigao_moon_fullday.csv")
with open(dst, "w", encoding="utf-8", newline="") as f:
    f.write("# 《清史稿》卷三十七月与五星犯、距：北京民用日全天地心最小角距（scripts/phenomena/qingshigao_moon_fullday.py）；只取干支在月者。\n")
    f.write(f"# {len(out)} 条：全天最小 中位 {statistics.median(mins):.2f}°，≤1° {sum(m <= 1 for m in mins)}，≤2° {sum(m <= 2 for m in mins)}；"
            f"仅夜间可见时段（检验表）中位 {statistics.median(float(x[1]) for x in out if x[1] not in ('—', '')):.2f}°\n")
    w = csv.writer(f)
    w.writerow(["序", "纪年日期", "公历(儒略)", "记载", "全天地心最小角距°", "最近时刻（北京时，时）", "夜间可见时段最小角距°", "原文"])
    for x in out:
        w.writerow([x[3], x[4], x[5], x[6], f"{x[0]:.2f}", f"{x[7]:.1f}", x[1], x[2]])
print(dst, len(out), "条")
