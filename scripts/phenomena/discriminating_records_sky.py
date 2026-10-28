"""以天象判定历表：data/calendar/discriminating_records.csv 中主表与同系推步表判断不一的记录，按各历下的记录之日
（两日相差约一月）求当日（北京时 0—24 时，地心）天体与目标的最小角距；日食记录求日月最小角距。角距小者即天象所支持之历。

月一月绕天一周，凌犯记录换一个月日期，角距即差数十度，判定不依赖任何复原表；行星行动慢，差一月角距差可能不大，判定较弱。
目标为星官者取其陈卓体系成员星中最近者；为天体者取天体。日干支在某历不在所记之月者，该历候选日取离该月最近的同名干支日。
两历候选日为同一日者（日干支所指之日固定，两历只是月序不同），此日角距 < 3° 则支持日干支在月的那一历；
候选日不同者，较小角距 < 3° 且两日相差 > 2° 为强，否则记「弱」。
输出 data/calendar/discriminating_records_sky.csv，由 doc/calendar/push-algorithms.md 引用。

用法（仓库根）：uv run --no-project --with skyfield python -I scripts/phenomena/discriminating_records_sky.py
"""
import csv
import os

import numpy as np
from skyfield.api import Star, load, load_file

ts = load.timescale(builtin=True)
bsp = "vendor/jpl/de441/de441.bsp" if os.path.exists("vendor/jpl/de441/de441.bsp") else "vendor/jpl/de406/de406.bsp"
eph = load_file(bsp)
earth = eph["earth"]
BODY = {"Moon": "moon", "Sun": "sun", "Mercury": "mercury barycenter", "Venus": "venus barycenter", "Mars": "mars barycenter",
        "Jupiter": "jupiter barycenter", "Saturn": "saturn barycenter"}


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


def min_sep(jdn, a, targets):
    """JDN 所指民用日（东八区近似，UT −8h 起 24 小时，5 分钟一点）内天体 a 与 targets 的最小地心角距（度）。"""
    t = ts.ut1_jd(jdn - 0.5 - 8 / 24 + np.linspace(0, 1, 289))
    pa = earth.at(t).observe(eph[BODY[a]]).apparent()
    best = 1e9
    for tg in targets:
        pb = earth.at(t).observe(tg).apparent()
        best = min(best, float(pa.separation_from(pb).degrees.min()))
    return best


def main():
    out = []
    for r in rows("data/calendar/discriminating_records.csv"):
        if r["类"] == "日食":
            a, targets = "Sun", [eph["moon"]]
        else:
            a = r["bodies"].split()[0]
            tid = r["target_id"]
            if tid in BODY:
                targets = [eph[BODY[tid]]]
            else:
                hips = [int(h) for i in tid.split("+") for h in members.get(i, []) if int(h) in stars]
                targets = [star(h) for h in hips]
        if not targets:
            continue
        res = {}
        for col, lab in (("主表日 JDN", "主表"), ("推步日 JDN", "推步")):
            if r[col]:
                res[lab] = min_sep(int(r[col]), a, targets)
        in_main, in_push = r["主表"].startswith("在月"), r["本项目推步"].startswith("在月")
        if len(res) == 2 and r["主表日 JDN"] != r["推步日 JDN"]:
            # 两历候选日不同（相差约两月）：角距小者为天象所支持
            lo, hi = sorted(res.values())
            win = ("主表" if res["主表"] < res["推步"] else "推步") + ("" if hi - lo > 2 and lo < 3 else "（弱）")
        elif res:
            # 同一日（日干支所指之日固定，两历只是月序不同）：此日天象相符，则支持把它排在所记之月的那一历
            sep = next(iter(res.values()))
            if sep < 3:
                win = "主表" if in_main else ("推步" if in_push else "")
            else:
                win = "此日天象不符（记录有误），不能判定"
        else:
            win = ""
        out.append({**{k: r[k] for k in ("书", "类", "政权", "年", "月", "日干支")},
                    "主表日角距°": f"{res['主表']:.2f}" if "主表" in res else "", "推步日角距°": f"{res['推步']:.2f}" if "推步" in res else "",
                    "天象所支持": win, "原文": r["原文"]})
    with open("data/calendar/discriminating_records_sky.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 以天象判定主表与推步表（scripts/phenomena/discriminating_records_sky.py，DE441/DE406 地心，当日最小角距）；"
                "日食为日月角距（<约 0.5° 方可有食）。\n")
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    for o in out:
        print(o["书"], o["年"], o["月"], o["日干支"], "主表", o["主表日角距°"], "推步", o["推步日角距°"], "→", o["天象所支持"], "|", o["原文"][:24])


if __name__ == "__main__":
    main()
