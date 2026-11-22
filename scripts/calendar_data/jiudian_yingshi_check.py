"""九店楚简 56 号墓简 96「□𡰥朔於營（室）」：荆𡰥之月合朔时太阳在营室——检验楚月建是亥正还是丑正。

简 96（《江陵九店东周墓》科学出版社 1995，附录二李家浩释文书页 510）：「☑□𡰥朔於營，夏𡰥□，享月□，夏柰□，八月□，九月□徙，十月□☑」，
逐月记朔日所在之宿，首月当为荆𡰥（其后接夏𡰥、享月，与楚月序合）。
两种假设：
  亥正：冬柰 = 夏正十月，荆𡰥为楚四月 = 夏正正月（与睡虎地《日书》秦楚月名对照同）；
  丑正：冬柰 = 夏正十二月，荆𡰥 = 夏正三月（九店同篇建除「[荆]𡰥：建於辰」、王红星《包山楚墓》附录二〇之说）。
月份取颛顼历平朔（亥正时为荆𡰥；丑正时同一楚月在亥正月序上后移两月），求该时刻太阳视赤经（DE406，赤道 of date），
与营室（室宿距星 α Peg，HIP 113963 → 东壁距星 γ Peg，HIP 1067，Hipparcos I/311 位置自行）比。宿度为赤道度。
九店 M56 为战国晚期早段墓，逐年列前 350—前 250。
另验「朔日所在之宿」这种读法：睡虎地《日书》甲种「除」（楚除，《睡虎地秦墓竹简》文物 1990 释文书页 180，简 1—13 正壹）以秦月列宿：
十一月斗、十二月須（女）、正月營（室）、二月奎、三月胃、四月畢、五月東（井）、六月柳、七月張、八月角、九月氐、十月心。
秦月即夏正月序（无月建之争），逐年求各月颛顼平朔时太阳所在之宿（陈卓距星，data/sky_culture/chinese_lunar_mansion_determinatives.csv，
Hipparcos 位置自行，赤道 of date），统计落在所列之宿及其前一宿的年数——若此表确是朔日太阳所在，各月都应集中在所列之宿附近。
输出 doc/calendar/jiudian-yingshi-checks.md（覆盖写）。
用法（仓库根）：uv run --no-project --with skyfield python -I scripts/calendar_data/jiudian_yingshi_check.py
"""
import math
from fractions import Fraction as Fr
from pathlib import Path

from skyfield.api import Star, load, load_file

ROOT = Path(__file__).resolve().parents[2]
ts = load.timescale(builtin=True)
eph = load_file(str(ROOT / "vendor/jpl/de406/de406.bsp"))
earth, sun = eph["earth"], eph["sun"]
EP = ts.tt_jd(2448349.0625)  # J1991.25
A_PEG = Star(ra_hours=346.190070561 / 15, dec_degrees=15.205367531, ra_mas_per_year=60.40, dec_mas_per_year=-41.30, epoch=EP)
G_PEG = Star(ra_hours=3.308958475 / 15, dec_degrees=15.183616100, ra_mas_per_year=1.98, dec_mas_per_year=-9.28, epoch=EP)
_L, _S, _E, _Y0 = Fr(27759, 940), Fr(1461, 4), Fr(1587781) - Fr(1, 2), -365


def ten(Y):
    return math.floor(((Y - _Y0) * _S - 3 * _S / 24) / _L) - 1


def ra(body, t):
    r, _, _ = earth.at(t).observe(body).apparent().radec(epoch="date")
    return r.hours * 15


md = ["# 九店简 96「荆𡰥朔於营室」检验（自动生成）", "",
      "> 由 `scripts/calendar_data/jiudian_yingshi_check.py` 生成，勿手改。方法见 [zhanguo-calendars.md](zhanguo-calendars.md) 四。", "",
      "荆𡰥之月颛顼平朔时刻的太阳视赤经（of date），与营室（α Peg → γ Peg）比。「距室」= 太阳赤经 − 室宿距星赤经（负为尚在室宿之前）。", "",
      "| 年 | 室宿 [α Peg, γ Peg] | 亥正：太阳赤经 | 距室 | 在营室 | 丑正：太阳赤经 | 距室 | 在营室 |", "|---|---|---|---|---|---|---|---|"]
cnt = {"亥": [0, []], "丑": [0, []]}
for yr in range(350, 249, -1):
    Y = 1 - yr
    cells = []
    for tag, n in (("亥", 4), ("丑", 6)):
        t = ts.tt_jd(float(_E + (ten(Y) + n - 1) * _L))
        s, a, g = ra(sun, t), ra(A_PEG, t), ra(G_PEG, t)
        span = (g - a) % 360
        rel = (s - a + 180) % 360 - 180
        inside = 0 <= rel <= span
        cnt[tag][0] += inside
        cnt[tag][1].append(rel)
        cells.append((s, rel, inside))
        if tag == "亥":
            room = f"[{a:.1f}, {g:.1f}]"
    md.append(f"| 前 {yr} | {room} | " + " | ".join(f"{s:.1f} | {rel:+.1f} | {'✓' if ins else '✗'}" for s, rel, ins in cells) + " |")
n = 101
md += ["", f"亥正：{cnt['亥'][0]}/{n} 年荆𡰥朔在营室内，距室 {min(cnt['亥'][1]):+.1f}—{max(cnt['亥'][1]):+.1f} 度（室宿宽约 16.5 度）；"
           f"丑正：{cnt['丑'][0]}/{n} 年，距室 {min(cnt['丑'][1]):+.1f}—{max(cnt['丑'][1]):+.1f} 度。", ""]
# —— 楚除十二月列宿 ——
import csv as _csv
HIP = {}
for line in open(ROOT / "data/stars/hipparcos_bright.csv", encoding="utf-8"):
    if line.startswith("#") or line.startswith("hip"):
        continue
    f = line.split(",")
    HIP[int(f[0])] = tuple(float(x) for x in f[1:3]) + tuple(float(x) for x in f[4:6])
MAN = [(r["mansion"], int(r["hip"])) for r in _csv.DictReader(l for l in open(ROOT / "data/sky_culture/chinese_lunar_mansion_determinatives.csv", encoding="utf-8") if not l.startswith("#")) if r["system"] == "chenzhuo"]
STARS = [(m, Star(ra_hours=HIP[h][0] / 15, dec_degrees=HIP[h][1], ra_mas_per_year=HIP[h][2], dec_mas_per_year=HIP[h][3], epoch=EP)) for m, h in MAN]
LIST = {11: "斗", 12: "女", 1: "室", 2: "奎", 3: "胃", 4: "毕", 5: "井", 6: "柳", 7: "张", 8: "角", 9: "氐", 10: "心"}
NAMES = [m for m, _ in MAN]


def lodge(t):
    s_ = ra(sun, t)
    edges = [(m, ra(st, t)) for m, st in STARS]
    best = None
    for i, (m, a) in enumerate(edges):
        rel = (s_ - a) % 360
        if best is None or rel < best[1]:
            best = (m, rel)
    return best[0]


md += ["## 楚除十二月列宿与朔日太阳所在之宿", "",
       "秦月（夏正）各月颛顼平朔时太阳所在之宿（陈卓距星），前 350—前 250 共 101 年的分布。「所列」= 楚除该月所书之宿。", "",
       "| 秦月 | 所列 | 在所列之宿 | 在其前一宿 | 其他 |", "|---|---|---|---|---|"]
for k in (11, 12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10):
    n = k + 3 if k <= 9 else k - 9
    hits = {}
    for yr in range(350, 249, -1):
        Y = 1 - yr
        t = ts.tt_jd(float(_E + (ten(Y) + n - 1) * _L))
        hits[lodge(t)] = hits.get(lodge(t), 0) + 1
    tgt = LIST[k]
    prev = NAMES[(NAMES.index(tgt) - 1) % 28]
    other = {m: c for m, c in hits.items() if m not in (tgt, prev)}
    md.append(f"| {k} 月 | {tgt} | {hits.get(tgt, 0)} | {prev} {hits.get(prev, 0)} | {'、'.join(f'{m} {c}' for m, c in other.items()) or '—'} |")
    print(k, tgt, hits)
md.append("")
(ROOT / "doc/calendar/jiudian-yingshi-checks.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print(md[-2])
