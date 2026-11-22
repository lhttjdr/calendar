"""楚简纪日与实际月相对照：楚历是按术推步（颛顼历平朔），还是看天定月（合朔日或朏）？

包山（data/calendar/baoshan_records.csv，《包山楚墓》系年）与新蔡王徙于鄩郢之岁（data/calendar/xincai_records.csv，去甲三:204 共 19 条）
各条，分别在两种月首下检验干支是否落在所记之月：
  (1) 颛顼历平朔（冬柰 = 夏正十月，与 baoshan_check.py / xincai_check.py 同）；
  (2) 看天：月首 = 都城当地合朔之日 + c，c = 0（合朔日）、1、2（约当朏）。
实际合朔用 DE406（skyfield almanac，视黄经相合），ΔT 用 skyfield 内置（Morrison & Stephenson），换算为栽郢（纪南城，东经 112.2°）地方平时。
楚月与实际朔望月的对应：取离颛顼平朔最近的那次合朔。

新蔡另对前 410—前 375 逐年看「看天」各 c 下合几条（王徙之岁的系年见 xincai_check.py；不靠历日的范围前 402—前 381）。
输出 doc/calendar/chu-sky-checks.md（覆盖写）。
用法（仓库根）：uv run --no-project --with skyfield python -I scripts/calendar_data/chu_sky_check.py
"""
import csv
import math
from fractions import Fraction as Fr
from pathlib import Path

from skyfield import almanac
from skyfield.api import load, load_file

ROOT = Path(__file__).resolve().parents[2]
ts = load.timescale(builtin=True)
eph = load_file(str(ROOT / "vendor/jpl/de406/de406.bsp"))
LON = 112.2
GZ = ["甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12] for i in range(60)]
_L, _S, _E, _Y0 = Fr(27759, 940), Fr(1461, 4), Fr(1587781) - Fr(1, 2), -365
CS = (0, 1, 2)


def ten(Y):
    return math.floor(((Y - _Y0) * _S - 3 * _S / 24) / _L) - 1


def zx_start(Y, n):
    return math.floor(_E + (ten(Y) + n - 1) * _L + Fr(1, 2))


t, y = almanac.find_discrete(ts.ut1_jd(1587781 - 65 * 365.25 - 400), ts.ut1_jd(1587781 + 70 * 365.25), almanac.moon_phases(eph))
CONJ = [tt.ut1 + LON / 360 for tt, yy in zip(t, y) if yy == 0]  # 当地平时 JD


def conj(Y, n):
    m = float(_E + (ten(Y) + n - 1) * _L)
    return min(CONJ, key=lambda x: abs(x - m))


def sky_start(Y, n, c):
    return math.floor(conj(Y, n) + 0.5) + c


def fits(a, b, g):
    return any((d + 49) % 60 == g for d in range(a, b))


def ok_zx(Y, n, g):
    return fits(zx_start(Y, n), zx_start(Y, n + 1), g)


def ok_sky(Y, n, g, c):
    return fits(sky_start(Y, n, c), sky_start(Y, n + 1, c), g)


def rows(p):
    return list(csv.DictReader(l for l in open(ROOT / p, encoding="utf-8") if not l.startswith("#")))


def yname(Y):
    return f"前 {1 - Y}"


B = []
for r in rows("data/calendar/baoshan_records.csv"):
    try:
        Y = int(r["sui_year"])
    except ValueError:
        continue
    B.append((r["sui"], Y, int(r["month_no"]), r["month"], r["gz"], r["jian"]))
X = [(r["岁"], int(r["月序"]), r["月"], r["干支"], r["简号"]) for r in rows("data/calendar/xincai_records.csv")
     if r["岁"] == "王徙于鄩郢之岁" and r["简号"] != "甲三:204"]

md = ["# 楚简纪日与实际月相（自动生成）", "",
      "> 由 `scripts/calendar_data/chu_sky_check.py` 生成，勿手改。方法与结论见 [zhanguo-calendars.md](zhanguo-calendars.md) 四。", "",
      "两种月首：**颛顼**＝颛顼历平朔（冬柰 = 夏正十月）；**看天 c**＝栽郢当地合朔之日 + c 日（c = 0 合朔日，1、2 约当朏）。"
      "实际合朔：DE406，ΔT 用 skyfield 内置，地方平时（东经 112.2°）。✓ 合、✗ 不合。", "",
      "## 各月月首对照", "", "| 岁 | 年 | 月 | 颛顼朔 | 实际合朔（地方时） | 颛顼朔 − 合朔日 |", "|---|---|---|---|---|---|"]
seen = set()
for key, Y, n in sorted({(b[0], b[1], b[2]) for b in B} | {("王徙于鄩郢之岁", -397, x[1]) for x in X}, key=lambda k: (k[1], k[2])):
    if (Y, n) in seen:
        continue
    seen.add((Y, n))
    j = conj(Y, n)
    zs = zx_start(Y, n)
    md.append(f"| {key} | {yname(Y)} | {n} | {GZ[(zs + 49) % 60]} | {GZ[(math.floor(j + 0.5) + 49) % 60]} {((j + 0.5) % 1) * 24:.1f} 时 | {zs - math.floor(j + 0.5):+d} |")

md += ["", "## 包山逐条（《包山楚墓》系年）", "", "| 岁 | 年 | 月 | 干支 | 简 | 颛顼 | 看天 0 | 看天 1 | 看天 2 |", "|---|---|---|---|---|---|---|---|---|"]
tot = {"颛顼": 0, 0: 0, 1: 0, 2: 0}
for s, Y, n, m, g, jian in B:
    gi = GZ.index(g)
    a = ok_zx(Y, n, gi)
    sk = [ok_sky(Y, n, gi, c) for c in CS]
    tot["颛顼"] += a
    for c, v in zip(CS, sk):
        tot[c] += v
    md.append(f"| {s} | {yname(Y)} | {m} | {g} | {jian} | {'✓' if a else '✗'} | " + " | ".join("✓" if v else "✗" for v in sk) + " |")
md += ["", f"合计 {len(B)} 条：颛顼 {tot['颛顼']}，看天 c=0 {tot[0]}、c=1 {tot[1]}、c=2 {tot[2]}。", ""]
print("包山", len(B), tot)

Y98 = -397
md += ["## 新蔡王徙于鄩郢之岁逐条（前 398）", "", "| 月 | 干支 | 简号 | 颛顼 | 看天 0 | 看天 1 | 看天 2 |", "|---|---|---|---|---|---|---|"]
for s, n, m, g, jian in X:
    gi = GZ.index(g)
    md.append(f"| {m} | {g} | {jian} | {'✓' if ok_zx(Y98, n, gi) else '✗'} | " + " | ".join("✓" if ok_sky(Y98, n, gi, c) else "✗" for c in CS) + " |")
md += ["", "## 新蔡王徙之岁逐年（看天，19 条中合几条）", "", "| 年 | 颛顼 | 看天 0 | 看天 1 | 看天 2 |", "|---|---|---|---|---|"]
xi = [(n, GZ.index(g)) for _, n, _, g, _ in X]
for yr in range(410, 374, -1):
    Y = 1 - yr
    z = sum(ok_zx(Y, n, g) for n, g in xi)
    sk = [sum(ok_sky(Y, n, g, c) for n, g in xi) for c in CS]
    if max(sk + [z]) >= 17:
        md.append(f"| 前 {yr} | {z} | " + " | ".join(map(str, sk)) + " |")
        print("新蔡", yr, z, sk)
(ROOT / "doc/calendar/chu-sky-checks.md").write_text("\n".join(md) + "\n", encoding="utf-8")
