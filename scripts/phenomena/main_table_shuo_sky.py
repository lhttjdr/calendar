"""本纪明书朔日与主表差一二日者（data/calendar/main_table_check.csv 类 = 朔差一二日），以实际合朔定是非：
求所书朔前后的日月合朔（DE441/DE406，视黄经相等），换算为都城地方平时（data/phenomena/capitals.csv 经度），
看合朔落在本纪之日还是主表之日。另对相合的明书朔日统计「朔日 = 合朔所在日」之比，作为该时期历法（平朔/定朔）的基线。

平朔历（南北朝以前）朔日与实际合朔本可差一二日，此判定只说明哪一个与天合，不等于哪一个是当时行用之历；
定朔历（唐麟德历以后）朔日原则上即合朔之日，但有进朔（合朔在日内较晚则朔进一日，大衍历以后为日之 3/4 以后），
故另注合朔地方时：晚于 18 时而主表朔为次日者记「主表为进朔」；本纪朔为合朔次日而合朔在 15 时以后者记「本纪或为进朔」
（历家推算的合朔与实际可差一二时）。进朔只就定朔（619 年唐戊寅历起）而言。
输出 data/calendar/main_table_shuo_sky.csv。

用法（仓库根）：uv run --no-project --with skyfield python -I scripts/phenomena/main_table_shuo_sky.py
"""
import csv
import os
from collections import Counter, defaultdict

from skyfield import almanac
from skyfield.api import load, load_file

ts = load.timescale(builtin=True)
bsp = "vendor/jpl/de441/de441.bsp" if os.path.exists("vendor/jpl/de441/de441.bsp") else "vendor/jpl/de406/de406.bsp"
eph = load_file(bsp)


def rows(path):
    with open(path, encoding="utf-8") as f:
        return list(csv.DictReader(line for line in f if not line.startswith("#")))


CAPITALS = rows("data/phenomena/capitals.csv")


def lon_of(regime, year):
    for r in CAPITALS:
        if r["regime"] == regime and (not r["from_year"] or int(r["from_year"]) <= year) and (not r["to_year"] or year <= int(r["to_year"])):
            return float(r["lon"])
    return 108.94


def conjunction(jdn, lon):
    """JDN 附近（±3 日）的合朔：(地方平时所在民用日 JDN, 地方平时时数)。"""
    t0, t1 = ts.ut1_jd(jdn - 4), ts.ut1_jd(jdn + 4)
    times, phases = almanac.find_discrete(t0, t1, almanac.moon_phases(eph))
    best = None
    for t, p in zip(times, phases):
        if p != 0:
            continue
        local = t.ut1 + lon / 360  # 地方平时（儒略日，日界在正午）
        day = int(local + 0.5)
        hour = (local + 0.5 - day) * 24
        if best is None or abs(day - jdn) < abs(best[0] - jdn):
            best = (day, hour)
    return best


def main():
    out, base, late = [], defaultdict(Counter), defaultdict(list)
    for r in rows("data/calendar/main_table_check.csv"):
        if not r["主表朔 JDN"]:
            continue
        y = int(r["年"])
        main_j, rec_j = int(r["主表朔 JDN"]), int(r["本纪朔 JDN"])
        if r["类"] == "":
            # 相合者：统计朔日与合朔同日之比（时期基线），按朝代
            day, hour = conjunction(rec_j, lon_of(r["政权"], y))
            base[r["朝代"]][day - rec_j] += 1
            if day - rec_j == -1:
                late[r["朝代"]].append(hour)
            continue
        if r["类"] != "朔差一二日":
            continue
        day, hour = conjunction(rec_j, lon_of(r["政权"], y))
        dingshuo = y >= 619  # 唐戊寅历起行定朔；此前平朔，无进朔之说
        if day == rec_j:
            v = "合朔在本纪日"
        elif day == main_j:
            # 本纪朔在合朔次日而合朔已晚：历家推算的合朔可晚于实际一二时，本纪或为进朔
            v = "合朔在主表日；本纪或为进朔" if dingshuo and rec_j == day + 1 and hour >= 15 else "合朔在主表日"
        elif dingshuo and main_j == day + 1 and hour >= 18:
            v = "主表为进朔"
        else:
            v = "合朔皆不在二日"
        out.append({k: r[k] for k in ("书", "朝代", "政权", "年", "纪年", "月", "本纪朔", "主表", "日食", "他书同记")} |
                   {"合朔地方时日差（合朔日−本纪朔）": day - rec_j, "合朔地方时": f"{hour:.1f}时", "判定": v, "原文": r["原文"]})
    with open("data/calendar/main_table_shuo_sky.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 本纪明书朔日与主表差一二日者之实际合朔（scripts/phenomena/main_table_shuo_sky.py，DE441/DE406，都城地方平时）。\n")
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    with open("data/calendar/main_table_shuo_baseline.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 本纪明书朔日与主表相合者：合朔（都城地方平时）所在日 − 朔日 的次数（scripts/phenomena/main_table_shuo_sky.py）；"
                "唐代 −1 者为进朔，合朔时刻见 jinshuo_hours。\n")
        w = csv.writer(f)
        w.writerow(["朝代", "差", "次数", "jinshuo_hours"])
        for dyn, c in base.items():
            for k in sorted(c):
                w.writerow([dyn, k, c[k], " ".join(f"{h:.1f}" for h in sorted(late[dyn])) if k == -1 else ""])
    for o in out:
        print(o["书"], o["年"], o["月"], o["本纪朔"], o["主表"], o["合朔地方时日差（合朔日−本纪朔）"], o["合朔地方时"], o["判定"], o["原文"][:16])
    print("相合者之朔日与合朔日差（合朔日 − 朔日：次数）")
    for dyn, c in base.items():
        print(" ", dyn, dict(sorted(c.items())))


if __name__ == "__main__":
    main()
