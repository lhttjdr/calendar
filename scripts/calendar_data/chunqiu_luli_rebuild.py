"""春秋鲁历由《春秋》经文历日直接排定（不取今人拟合表）。

春秋历无术文传世。今人诸表（张培瑜《中国古代历法》表 3-7、王韬《春秋朔闰表》等）都是据经传历日拟合，表中某年为何闰、
正月为何在某月，表内不注依据。这里把拟合过程摊开：每年正月取哪个朔望月、是否闰（闰在岁末，经文文公六年、哀公五年「閏月」
皆在岁末），逐年由经文纪日定，并注明是哪几条经文定的；经文定不下来的年份标「未定」并列可取之数。

月首两种模型，各自逐年排定后比经文合数：
  平朔 φ：月首 = ⌊J0 + k·30328/1027 + φ⌋（朔策同张表；φ 为日界偏移，扫描取合数最多者）；
  看天 c：月首 = 曲阜（东经 117.0°）当地真合朔之日 + c（DE406，ΔT 用 skyfield 内置），c = 0、1、2。
排定：动态规划。年 Y 正月 = 第 s_Y 个月，正月之建 k_Y = s_Y − 含冬至之月（−1 亥、0 子、1 丑、2 寅），s_{Y+1} − s_Y ∈ {12, 13}，
13 即岁末闰。目标依次为：(1) 经文纪日在月（书朔者须为朔日；书日食者其日须在有日食之真合朔前后一日内，合朔时月黄纬 |β| < 1.55°）条数最多；(2) 文公六年（前 621）、哀公五年（前 490）有闰；
(3) 先验：置闰节奏（距上一闰年第几年置闰的机会）与正月多建子、丑；参数按边际似然选。见下「概率模型」注。
每年的标注：「经文定」= 只看 (1)(2) 该年正月与闰已唯一；「建不变推定」= 加 (3)(4) 才唯一；「未定」= 仍有几种。
输出 data/calendar/chunqiu_luli_rebuild.csv（逐年）、chunqiu_luli_months.csv（逐月）、doc/calendar/chunqiu-luli-checks.md。
用法（仓库根）：uv run --no-project --with skyfield python -I scripts/calendar_data/chunqiu_luli_rebuild.py
"""
import csv
import math
import sys
from fractions import Fraction as Fr
from pathlib import Path

from skyfield import almanac
from skyfield.api import load, load_file

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))
import shiwu_fenlei as fl  # noqa: E402
GZ = ["甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12] for i in range(60)]
Y0, Y1 = -721, -479
LEAP_YEARS = {-620, -489}  # 文公六年、哀公五年经书「閏月」（岁末）
LON = 117.0
J0 = Fr(1457727761054236, 10**9)
L = Fr(30328, 1027)

ts = load.timescale(builtin=True)
eph = load_file(str(ROOT / "vendor/jpl/de406/de406.bsp"))
t0, t1 = ts.ut1_jd(1457727 - 400), ts.ut1_jd(1457727 + 245 * 365.25 + 400)
tt, yy = almanac.find_discrete(t0, t1, almanac.moon_phases(eph))
CONJ = [t.ut1 + LON / 360 for t, y in zip(tt, yy) if y == 0]
_CD = [math.floor(x + 0.5) for x in CONJ]
# 有日食之合朔（合朔时月黄纬 |β| < 1.55°，地球上某处可见日食）
_moon, _earth = eph["moon"], eph["earth"]
_ECL = set()
for t, y in zip(tt, yy):
    if y == 0:
        lat, _, _ = _earth.at(t).observe(_moon).apparent().ecliptic_latlon()
        if abs(lat.degrees) < 1.55:
            _ECL.add(math.floor(t.ut1 + LON / 360 + 0.5))


def CONJ_DAY_NEAR(d):
    import bisect
    i = bisect.bisect_left(_CD, d)
    return _CD[max(0, i - 1): i + 1]
ts_, ys_ = almanac.find_discrete(t0, t1, almanac.seasons(eph))
DZ = [t.ut1 + LON / 360 for t, y in zip(ts_, ys_) if y == 3]  # 冬至


def rows(p):
    return list(csv.DictReader(l for l in open(ROOT / p, encoding="utf-8") if not l.startswith("#")))


REC = []
for r in rows("data/calendar/chunqiu_jingwen.csv"):
    if not r["月"] or r["月"] == "0" or r["干支"] not in GZ:
        continue
    REC.append((int(r["年"]), int(r["月"]), r["闰"] == "1", GZ.index(r["干支"]), r["朔"] == "1", r["日食"] == "1", r))


def starts_mean(phi):
    k0 = math.floor((Fr(CONJ[0]) - J0) / L) - 1
    return [math.floor(J0 + (k0 + i) * L + phi) for i in range(len(CONJ) + 4)]


def starts_sky(c):
    return [math.floor(x + 0.5) + c for x in CONJ]


# 概率模型：经文纪日在真实历日下相合的机会 1 − eps（eps 为传写之误率），在错误排法下碰巧相合约 0.5。
# 先验只管置闰节奏与岁首：距上一闰年第 g 年（g = 1、2、3、≥4）置闰的机会 h1、h2、h3、h4（十九年七闰即多隔二、三年）；
# 正月建子、丑以外（亥、寅）每年 podd。各参数与月首模型一并按边际似然（经验贝叶斯）选取。
def lse(xs):
    m = max(xs)
    return m + math.log(sum(math.exp(x - m) for x in xs)) if m > -1e300 else m


def solve(S, eps, hz, podd):
    """S：各月首 JDN；hz = (h1, h2, h3, h4)。返回 (最大后验路径之经文合数, 逐年后验 {s: p}, 路径, w, score, by_year, ln Z)。"""
    import bisect
    W_OK, W_BAD = math.log((1 - eps) / 0.5), math.log(eps / 0.5)
    P_ODD, P_EVEN = math.log(podd), math.log(1 - podd)
    w = {Y: bisect.bisect_right(S, math.floor(DZ_Y(Y) + 0.5)) - 1 for Y in range(Y0, Y1 + 2)}
    by_year = {}
    for rec in REC:
        by_year.setdefault(rec[0], []).append(rec)

    def score(Y, s, n):
        ok_n = bad_n = 0
        for _, m, leap, g, shuo, eclipse, _ in by_year.get(Y, []):
            i = s + (12 if leap else m - 1)
            if leap and n == 12 or i + 1 >= len(S):
                bad_n += 1
                continue
            a, b = S[i], S[i + 1]
            d = a + (g - (a + 49)) % 60
            ok = d < b and (not shuo or d == a)
            # 日食：记录之日须在有日食之真合朔前后一日内（日期由天象定死）
            if ok and eclipse and not any(abs(d - c) <= 1 for c in CONJ_DAY_NEAR(d) if c in _ECL):
                ok = False
            ok_n += ok
            bad_n += not ok
        return ok_n, bad_n

    cand = {Y: [w[Y] + k for k in (-1, 0, 1, 2)] for Y in range(Y0, Y1 + 2)}
    prior = lambda Y, s: P_EVEN if (s - w[Y]) in (0, 1) else P_ODD
    sc = {}
    for Y in range(Y0, Y1 + 1):
        for s in cand[Y]:
            for n in (12, 13):
                sc[(Y, s, n)] = score(Y, s, n)
    NEG = -1e308
    # 状态 (s, g)：g = 本年距上一闰年之年数（1..4，4 即 ≥4）
    G = (1, 2, 3, 4)
    def steps(Y, s, g):
        for n in (12, 13):
            if Y in LEAP_YEARS and n != 13:
                continue
            s2 = s + n
            if s2 not in cand[Y + 1]:
                continue
            h = hz[g - 1]
            o, bd = sc[(Y, s, n)]
            pr = math.log(h) if n == 13 else math.log(1 - h)
            g2 = 1 if n == 13 else min(g + 1, 4)
            yield (s2, g2), o * W_OK + bd * W_BAD + pr + prior(Y + 1, s2), o
    fw = {Y0: {(s, g): prior(Y0, s) - math.log(4) for s in cand[Y0] for g in G}}
    vit = {Y0: {k: (v, None) for k, v in fw[Y0].items()}}
    for Y in range(Y0, Y1 + 1):
        acc, vacc = {}, {}
        for st, v in fw[Y].items():
            for st2, wgt, _ in steps(Y, *st):
                acc.setdefault(st2, []).append(v + wgt)
                cand_v = (vit[Y][st][0] + wgt, st)
                if st2 not in vacc or cand_v > vacc[st2]:
                    vacc[st2] = cand_v
        fw[Y + 1] = {k: lse(v) for k, v in acc.items()}
        vit[Y + 1] = vacc
    bw = {Y1 + 1: {k: 0.0 for k in fw[Y1 + 1]}}
    for Y in range(Y1, Y0 - 1, -1):
        bw[Y] = {}
        for st in fw[Y]:
            t = [wgt + bw[Y + 1][st2] for st2, wgt, _ in steps(Y, *st) if st2 in bw[Y + 1]]
            bw[Y][st] = lse(t) if t else NEG
    Z = lse(list(fw[Y1 + 1].values()))
    post = {}
    for Y in range(Y0, Y1 + 2):
        d = {}
        for (s, g), v in fw[Y].items():
            if bw[Y].get((s, g), NEG) > NEG:
                d[s] = d.get(s, 0) + math.exp(v + bw[Y][(s, g)] - Z)
        post[Y] = d
    st = max(vit[Y1 + 1], key=lambda x: vit[Y1 + 1][x][0])
    path = {Y1 + 1: st[0]}
    for Y in range(Y1 + 1, Y0, -1):
        st = vit[Y][st][1]
        path[Y - 1] = st[0]
    hits = sum(sc[(Y, path[Y], path[Y + 1] - path[Y])][0] for Y in range(Y0, Y1 + 1))
    return hits, post, path, w, score, by_year, Z


def DZ_Y(Y):
    """天文纪年 Y 之岁首前的冬至（Y−1 年 12 月）。"""
    j = 1721058 + (Y - 1) * 365.25 + 355  # 约略
    return min(DZ, key=lambda x: abs(x - j))


import itertools
import time
models = [(f"平朔 φ={float(phi):.3f}", starts_mean(phi)) for phi in [Fr(i, 40) for i in range(16, 23, 2)]] + [(f"看天 c={c}", starts_sky(c)) for c in (0,)]
GRID = [(e, (h1, h2, 0.8, h4), p) for e in (0.15, 0.2, 0.25, 0.3) for h1 in (0.02, 0.08) for h2 in (0.25, 0.45)
        for h4 in (0.5, 0.85) for p in (0.003, 0.01, 0.05)]
results = []
t_ = time.time()
for name, S_ in models:
    for eps, pchg, podd in GRID:
        r = solve(S_, eps, pchg, podd)
        results.append((r[6], r[0], name, (eps, pchg, podd), S_))
    print(name, max(x[0] for x in results if x[2] == name), round(time.time() - t_), flush=True)
results.sort(key=lambda r: -r[0])
for r in results[:8]:
    print(f"lnZ {r[0]:.1f} 合 {r[1]} {r[2]} eps={r[3][0]} h={r[3][1]} podd={r[3][2]}")
bestZ, _, bname, (EPS_, HZ_, PODD_), S = results[0]
best = (None, f"{bname}，eps={EPS_}、置闰 h1—h4={'/'.join(map(str, HZ_))}、podd={PODD_}")
top1, post, path, w, score, by_year, _ = solve(S, EPS_, HZ_, PODD_)
MODEL_ROWS = []
for name, _ in models:
    r = max((x for x in results if x[2] == name), key=lambda x: x[0])
    MODEL_ROWS.append((name, r[0], r[1], r[3]))

# 张培瑜表 3-7（对照）：闰年表照 rust/core/src/calendar/guliuli.rs 的 CQ_LEAP，朔日 = ⌊J0 + k·朔策 + 0.5⌋（与 Rust 同）
import re as _re
_cq = _re.search(r'const CQ_LEAP: &str = "([01]+)"', (ROOT / "rust/core/src/calendar/guliuli.rs").read_text(encoding="utf-8")).group(1)
zhang, zl = {}, {}
_k = 0
for _i, _c in enumerate(_cq):
    zhang[Y0 + _i] = math.floor(J0 + _k * L + Fr(1, 2))
    zl[Y0 + _i] = _c == "1"
    _k += 13 if _c == "1" else 12
JIAN = {-1: "亥", 0: "子", 1: "丑", 2: "寅"}
yr = ["年,正月朔JDN,正月朔干支,建,闰,标注,经文条数,经文合数,后验,其他可取（正月朔干支建:后验）,张表正月朔,张表闰,与张表同"]
mo = ["年,月,闰,朔日JDN,朔干支,标注"]
same = 0
tags = {"经文定": 0, "无经文按建推定": 0, "未定": 0}
for Y in range(Y0, Y1 + 1):
    s, s2 = path[Y], path[Y + 1]
    n = s2 - s
    # 该年正月与次年正月（即本年是否闰）的联合后验，取两端后验之小者近似
    p = min(post[Y].get(s, 0), post[Y + 1].get(s2, 0))
    nrec = len(by_year.get(Y, []))
    tag = "未定" if p < 0.95 else ("经文定" if nrec else "无经文按建推定")
    tags[tag] += 1
    zj = zhang.get(Y)
    eq = zj == S[s] and zl.get(Y, False) == (n == 13)
    same += eq
    alts = " ".join(f"{GZ[(S[x] + 49) % 60]}{JIAN[x - w[Y]]}:{q:.2f}" for x, q in sorted(post[Y].items(), key=lambda kv: -kv[1]) if x != s and q >= 0.01)
    yr.append(f"{Y},{S[s]},{GZ[(S[s] + 49) % 60]},{JIAN[s - w[Y]]},{int(n == 13)},{tag},{nrec},{score(Y, s, n)[0]},{p:.3f},{alts},{GZ[(zj + 49) % 60] if zj else ''},{int(zl.get(Y, False))},{int(eq)}")
    for i in range(n):
        mo.append(f"{Y},{12 if i == 12 else i + 1},{int(i == 12)},{S[s + i]},{GZ[(S[s + i] + 49) % 60]},{tag}")
(ROOT / "data/calendar/chunqiu_luli_rebuild.csv").write_text("# 由 scripts/calendar_data/chunqiu_luli_rebuild.py 生成。模型：" + best[1] + "\n" + "\n".join(yr) + "\n", encoding="utf-8")
(ROOT / "data/calendar/chunqiu_luli_months.csv").write_text("# 由 scripts/calendar_data/chunqiu_luli_rebuild.py 生成。模型：" + best[1] + "\n" + "\n".join(mo) + "\n", encoding="utf-8")
print("best", best[1], "合", top1, "/", len(REC), tags, "与张表同年", same)
md = ["# 春秋鲁历由经文直接排定（自动生成）", "", "> 由 `scripts/calendar_data/chunqiu_luli_rebuild.py` 生成，勿手改。方法见脚本说明与 [zhanguo-calendars.md](zhanguo-calendars.md)「春秋鲁历」。", "",
      "## 月首模型比较", "", "| 模型 | 经文纪日合（最优排法下） |", "|---|---|"]
md[-2:] = ["| 月首模型 | 最大边际似然 ln Z | 其最大后验排法经文合 | 取得最大之参数 eps / h1—h4 / podd |", "|---|---|---|---|"]
md += [f"| {n} | {z:.1f} | {h}/{len(REC)} | {p[0]} / {'、'.join(map(str, p[1]))} / {p[2]} |" for n, z, h, p in MODEL_ROWS]
md += ["", f"取 **{best[1]}**（最大后验排法下经文合 {top1}/{len(REC)}）。逐年标注：经文定 {tags['经文定']}、无经文按建推定 {tags['无经文按建推定']}、未定 {tags['未定']}（后验 < 0.95）；与张培瑜表 3-7 正月朔与闰皆同者 {same}/{Y1 - Y0 + 1} 年。", "",
       "## 逐年", "", "| 年 | 正月朔 | 建 | 闰 | 标注 | 经文 | 合 | 后验 | 其他可取 | 张表正月朔 | 张表闰 | 同 |", "|---|---|---|---|---|---|---|---|---|---|---|---|"]
for line in yr[1:]:
    c = line.split(",")
    md.append(f"| 前 {1 - int(c[0])} | {c[2]} | {c[3]} | {'闰' if c[4] == '1' else ''} | {c[5]} | {c[6]} | {c[7]} | {c[8]} | {c[9]} | {c[10]} | {'闰' if c[11] == '1' else ''} | {'✓' if c[12] == '1' else '✗'} |")
(ROOT / "doc/calendar/chunqiu-luli-checks.md").write_text("\n".join(md) + "\n", encoding="utf-8")

# —— 分类：本排法（「推步」位）与张表 3-7 逐岁相同、互异，经文为证 ——
def rec_ok(Y, s, n):
    """该岁经文各条在排法 (s, n) 下合否（与 solve.score 同一判法）。"""
    out = []
    for _, m, leap, g, shuo, eclipse, r in by_year.get(Y, []):
        i = s + (12 if leap else m - 1)
        if leap and n == 12 or i + 1 >= len(S):
            out.append(False)
            continue
        a, b = S[i], S[i + 1]
        d = a + (g - (a + 49)) % 60
        ok = d < b and (not shuo or d == a)
        if ok and eclipse and not any(abs(d - c) <= 1 for c in CONJ_DAY_NEAR(d) if c in _ECL):
            ok = False
        out.append(ok)
    return out


def fenlei_section(ZP):
    """逐岁一个分类。经文书朔、日食之条为 C（史书明书），余为纪日 D（同岁两条以上独立才判）。"""
    cats, odd = [], []
    for Y in range(Y0, Y1 + 1):
        recs = by_year.get(Y, [])
        lv = ["C" if (r[4] or r[5]) else "D" for r in recs]
        a = rec_ok(Y, path[Y], path[Y + 1] - path[Y])
        z = rec_ok(Y, ZP[Y], ZP[Y + 1] - ZP[Y])
        enough = lambda idx: idx and (any(lv[i] == "C" for i in idx) or len(idx) >= 2)
        best = lambda idx: min(lv[i] for i in idx) if idx else "—"
        same = S[path[Y]] == S[ZP[Y]] and path[Y + 1] - path[Y] == ZP[Y + 1] - ZP[Y]
        both = [i for i in range(len(recs)) if a[i] and z[i]]
        none = [i for i in range(len(recs)) if not a[i] and not z[i]]
        pa = [i for i in range(len(recs)) if a[i] and not z[i]]
        pz = [i for i in range(len(recs)) if z[i] and not a[i]]
        if same:
            if enough(both):
                c, note, g = "有证", (f"另 {len(none)} 条两家皆不合" if none else ""), best(both)
            elif enough(none) and not both:
                c, note, g = "同误", "经文与两家皆不合，待核（或经文月误）", best(none)
            else:
                c, note, g = "无证", ("纪日一见" if recs else ""), "—"
        elif pa and pz:
            c, note, g = "③", f"证据冲突：只合本排法 {len(pa)}、只合张表 {len(pz)}", "—"
        elif enough(pa):
            c, note, g = "①", "", best(pa)
        elif enough(pz):
            c, note, g = "②", "", best(pz)
        else:
            c, g = "③", "—"
            note = "纪日一见，倾向" + ("本排法" if pa else "张表") if (pa or pz) else ("经文两家皆合，不能判" if both else ("经文两家皆不合" if none else ""))
        cats.append((c, g))
        if c not in ("有证", "无证"):
            odd.append(f"| 前 {1 - Y} | {GZ[(S[path[Y]] + 49) % 60]}{'闰' if path[Y + 1] - path[Y] == 13 else ''} | "
                       f"{GZ[(S[ZP[Y]] + 49) % 60]}{'闰' if ZP[Y + 1] - ZP[Y] == 13 else ''} | {len(recs)} | {len(pa)} | {len(pz)} | {c}{f'（{note}）' if note else ''} | {g} |")
    return ["## 分类（按岁）", "",
            "此段张表即张培瑜表 3-7（今人复原），「推步」位为本排法（今人复原式的经文排定，亦非术文推步）；两家正月朔与闰皆同为相同，否则为互异。"
            "证据为《春秋》经文：书朔、日食之条记 C（史书明书），其余纪日记 D（同岁两条以上独立才判，一见者注「纪日一见」）。"
            "本排法本由经文排出，「有证」「①」对本排法不是独立检验。" + fl.RULE.replace("实物", "经文"), "",
            fl.headline(cats, "本排法", "岁") + "。", ""] + fl.grade_table(cats) + ["",
            "①②③与同误之岁（「只合本」「只合张」为只合一家之经文条数）：", "",
            "| 年 | 本排法正月朔 | 张表正月朔 | 经文 | 只合本 | 只合张 | 分类 | 等级 |", "|---|---|---|---|---|---|---|---|"] + odd + [""]


# —— 对照：张培瑜表 3-7 的排法在同一评分下 ——
def zhang_path(S):
    import bisect
    zs = {}
    for Y, j in zhang.items():
        if j in S:
            zs[Y] = S.index(j)
    return zs


ZP = zhang_path(S)
if len(ZP) >= Y1 - Y0 + 1:
    ZP[Y1 + 1] = ZP[Y1] + 12 + zl.get(Y1, False)
    zh = sum(score(Y, ZP[Y], ZP[Y + 1] - ZP[Y])[0] for Y in range(Y0, Y1 + 1))
    zleaps = [Y for Y in range(Y0, Y1 + 1) if ZP[Y + 1] - ZP[Y] == 13]
    consec = sum(1 for a, b in zip(zleaps, zleaps[1:]) if b - a == 1)
    odd = sum(1 for Y in range(Y0, Y1 + 1) if ZP[Y] - w[Y] not in (0, 1))
    gaps = [b - a for a, b in zip(zleaps, zleaps[1:])]
    import collections as _c
    print("张表排法：经文合", zh, "闰", len(zleaps), "连闰", consec, "亥寅", odd, "闰距分布", sorted(_c.Counter(gaps).items()))
    ml = [Y for Y in range(Y0, Y1 + 1) if path[Y + 1] - path[Y] == 13]
    print("本排法：闰", len(ml), "闰距分布", sorted(_c.Counter(b - a for a, b in zip(ml, ml[1:])).items()))
    _mg = sorted(_c.Counter(b - a for a, b in zip(ml, ml[1:])).items())
    _note = ["## 与张培瑜表 3-7 的排法对照（同一评分）", "",
             f"张表排法：经文合 {zh}/{len(REC)}，闰 {len(zleaps)}，其中连年置闰 {consec} 次，正月建亥、寅 {odd} 年，闰距分布 {sorted(_c.Counter(gaps).items())}。",
             f"本排法（最大后验）：经文合 {top1}/{len(REC)}，闰 {len(ml)}，闰距分布 {_mg}。",
             "张表多合之条绝大多数为普通纪日（书朔两家同，日食相近），要靠连闰、亥寅岁首与长闰距换得；按边际似然所选之先验，这种不规则不比规整之排法更有据，张表亦不注各年依据。", ""]
    _note += fenlei_section(ZP)
    with open(ROOT / "doc/calendar/chunqiu-luli-checks.md", "a", encoding="utf-8") as _f:
        _f.write("\n".join(_note) + "\n")
else:
    print("张表正月不在本月首序列中", len(ZP))
