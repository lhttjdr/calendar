"""主序列历日可信度：各历推步按所经检验之最高证据等级分档，由检验样本估推步不合率，汇成逐十年、逐朝代与全时段之可信度。

输入（只读，均为本项目生成物）：
- data/calendar/main_series_reckoning_months.csv（main_series_reckoning_eval 逐月表：分类、证据等级、注）
- data/calendar/tongwu_causes.csv（同误逐条成因，scripts/calendar_data/tongwu_causes.py）
- doc/calendar/hanjian-checks.md（汉简公文书逐条检验表，scripts/calendar_data/hanjian_shuo_check.py；B 级当时文字）
- doc/calendar/qinhan-shiwu-checks.md（汉初实物分段拟合：「区间未定」「段界未定」月数）

方法：
1. 检验等级：一部历的推步，取它所碰到的最高一级证据之检验——A 实物、B 当时文书（汉简）、C 史书（本纪明书朔、纪日、后修），
   该级检验月数不足 20 者往下取；三级合计仍不足 20 者为「检验稀少」，不估计。汉初颛顼历的实物是拟合合朔改正 δ 所用，
   不是独立检验，记「实物定参」，区间上界取实物仍定不下来之月（区间未定 + 段界未定）占比。
2. 推步可能不合之月：只计成因指向推步或实历之月——
   - 史书（C 级）：同误之月，除去有「据」之史文讹误或纪日抽取误者、平朔时代食日书朔者；余下之中，有明书朔差一二日者，
     或主因为「行用之历不同」「无从判断」者，计为推步可能不合（差三日以上而无「据」者是系月、月名之事，推步不能差这么多）。
     ②（证据证推步误）之月亦计。
   - 汉简（B 级）：同误之月中判「合前后月（月名、年号抄误或闰位异）」或「待核」者计；差十日以上判抄误、释读误者不计。
   - 实物（A 级）：不合即计。
3. 不合率 p = 可能不合月数 / 检验月数，区间为 Wilson 95%。这是上界：所计之月仍可能是史书之误。
4. 无证之月的估计错月 = 该历 p 之和；检验稀少之历借全部已检验月份的总不合率估计，单列。

输出：doc/calendar/rili-credibility.md（方法、总账、逐朝代、逐历、待核月清单）、data/calendar/rili_credibility_decades.csv（逐十年）。

用法（仓库根）：python3 scripts/calendar_data/rili_credibility.py
"""
from __future__ import annotations

import csv
import math
import re
from collections import Counter, OrderedDict, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MONTHS = ROOT / "data/calendar/main_series_reckoning_months.csv"
TONGWU = ROOT / "data/calendar/tongwu_causes.csv"
HANJIAN = ROOT / "doc/calendar/hanjian-checks.md"
QINHAN = ROOT / "doc/calendar/qinhan-shiwu-checks.md"
OUT_MD = ROOT / "doc/calendar/rili-credibility.md"
OUT_CSV = ROOT / "data/calendar/rili_credibility_decades.csv"
MIN_N = 20
ERAS = [(-221, -104, "秦—汉初"), (-104, 220, "两汉"), (220, 589, "魏晋南北朝"), (589, 960, "隋唐五代"), (960, 1279, "宋"),
        (1279, 1368, "元"), (1368, 1644, "明"), (1644, 1912, "清")]
TIER_NAME = {"A": "实物检验", "fit": "实物定参", "B": "当时文书检验", "C": "史书检验", "few": "检验稀少"}


def wilson(k: int, n: int, z: float = 1.96):
    if n == 0:
        return 0.0, 1.0
    p = k / n
    d = 1 + z * z / n
    c = p + z * z / (2 * n)
    h = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n))
    return max(0.0, (c - h) / d), min(1.0, (c + h) / d)


def dicts(path: Path):
    return list(csv.DictReader(l for l in open(path, encoding="utf-8") if not l.startswith("#")))


def tongwu_residual():
    """同误之月中推步可能不合者：{(历,年,月): 说明}。"""
    by = defaultdict(list)
    for r in dicts(TONGWU):
        by[(r["历"], r["年"], r["月"])].append(r)
    out = {}
    for k, v in by.items():
        if any(r["强度"] == "据" and r["成因"] in ("史文讹误", "纪日抽取误") for r in v):
            continue
        if any("食日书朔" in r["细目"] for r in v):
            continue
        small = [r for r in v if r["证据类"] == "明书朔" and re.fullmatch(r"[+-]?\d+", r["差日"]) and abs(int(r["差日"])) <= 2]
        pool = [r for r in v if r["强度"] == "据"] or v
        top = Counter(r["成因"] for r in pool).most_common(1)[0][0]
        if small or top in ("行用之历不同", "无从判断"):
            r0 = small[0] if small else pool[0]
            out[k] = f"{r0['证据原文'][:16]}（差 {r0['差日']} 日；{r0['成因']}：{r0['细目'][:24]}）"
    return out


def hanjian_months():
    """汉简逐条表按（公元年, 简文日期）归月：{(年, 日期): [分类…]}。"""
    hj = defaultdict(list)
    for line in open(HANJIAN, encoding="utf-8"):
        c = [x.strip() for x in line.split("|")]
        if len(c) < 13:
            continue
        m = re.fullmatch(r"(前 ?)?(\d+)", c[4])
        if not m:
            continue
        y = -int(m.group(2)) + 1 if m.group(1) else int(m.group(2))
        hj[(y, c[3])].append((c[11], c[12][:20]))
    return hj


def hanjian_li(y: int) -> str:
    return "太初历（太初元年，五月改历）" if y == -103 else ("太初历（三统历）" if y <= 84 else "四分历")


def qinhan_undetermined() -> tuple[int, int]:
    t = QINHAN.read_text(encoding="utf-8")
    m = re.search(r"合计 (\d+) 月：.*?区间未定 (\d+)，段界未定 (\d+)", t)
    return int(m.group(2)) + int(m.group(3)), int(m.group(1))


def main():
    rows = dicts(MONTHS)
    resid = tongwu_residual()
    seg = OrderedDict()
    for r in rows:
        li, y = r["历"], int(r["年"])
        s = seg.setdefault(li, dict(y0=y, y1=y, months=0, ev={"A": [0, 0], "B": [0, 0], "C": [0, 0]}, res=[]))
        s["y0"], s["y1"], s["months"] = min(s["y0"], y), max(s["y1"], y), s["months"] + 1
        f, g = r["分类"], r["证据等级"][:1]
        G = "A" if g == "A" else ("C" if g in "CDE" else None)
        if not G:
            continue
        if f == "有证" or f.startswith("①"):
            s["ev"][G][0] += 1
        elif f == "同误" or f.startswith("②"):
            s["ev"][G][0] += 1
            k = (li, r["年"], r["月"])
            if f.startswith("②") or k in resid:
                s["ev"][G][1] += 1
                s["res"].append((f"{r['年']}年{r['月']}月", G, resid.get(k, "证据证推步误")))
    hj_conf = 0
    for (y, lab), cls in hanjian_months().items():
        if not -104 <= y <= 236:
            continue
        li = hanjian_li(y)
        if li not in seg:
            continue
        e = seg[li]["ev"]["B"]
        if any(c == "有证" for c, _ in cls):
            e[0] += 1
            hj_conf += 1
        elif any("合前后月" in c or "待核" in c for c, _ in cls):
            e[0] += 1
            e[1] += 1
            seg[li]["res"].append((f"{y}年{lab}", "B", next(c for c, _ in cls if "合前后月" in c or "待核" in c)[:30]))
        elif any(c.startswith("同误") for c, _ in cls):
            e[0] += 1
    und, qn = qinhan_undetermined()
    for li, s in seg.items():
        if li.startswith("颛顼"):
            s.update(tier="fit", n=s["ev"]["A"][0], k=0, p=0.0, lo=0.0, hi=und / qn)
            continue
        for G in "ABC":
            if s["ev"][G][0] >= MIN_N:
                n, k = s["ev"][G]
                s.update(tier=G, n=n, k=k)
                break
        else:
            n = sum(v[0] for v in s["ev"].values())
            k = sum(v[1] for v in s["ev"].values())
            s.update(tier="few", n=n, k=k)
        s["p"] = s["k"] / s["n"] if s["n"] else 0.0
        s["lo"], s["hi"] = wilson(s["k"], s["n"])
    tested = [s for s in seg.values() if s["tier"] not in ("few", "fit")]
    pool_k, pool_n = sum(s["k"] for s in tested), sum(s["n"] for s in tested)
    pool_p, (pool_lo, pool_hi) = pool_k / pool_n, wilson(pool_k, pool_n)

    # 总账：证实、查明、存疑、无证
    led = Counter()
    exp = dict(p=0.0, lo=0.0, hi=0.0)
    few = dict(n=0)
    for r in rows:
        f, k, s = r["分类"], (r["历"], r["年"], r["月"]), seg[r["历"]]
        if f == "有证" or f.startswith("①"):
            led["证实"] += 1
        elif f.startswith("②"):
            led["证实"] += 1
        elif f == "同误":
            led["存疑·同误待核" if k in resid else "查明·证据之误"] += 1
        elif f.startswith("③"):
            led["存疑·两家互异无据"] += 1
        else:
            led["无证"] += 1
            if s["tier"] == "few":
                few["n"] += 1
            else:
                exp["p"] += s["p"]
                exp["lo"] += s["lo"]
                exp["hi"] += s["hi"]
    total = len(rows)

    # 逐十年
    bins = OrderedDict()
    for r in rows:
        y = int(r["年"])
        s = seg[r["历"]]
        b = bins.setdefault((y // 10) * 10, dict(n=0, tier=Counter(), t=0, e=0.0, lo=0.0, hi=0.0, li=Counter()))
        b["n"] += 1
        b["tier"][s["tier"]] += 1
        b["li"][r["历"]] += 1
        if s["tier"] != "few":
            b["t"] += 1
            b["e"] += s["p"]
            b["lo"] += s["lo"]
            b["hi"] += s["hi"]
    with open(OUT_CSV, "w", encoding="utf-8", newline="") as fh:
        fh.write("# 主序列逐十年历日可信度（scripts/calendar_data/rili_credibility.py 生成，勿手改）。估计不合为推步可能不合月数之期望（上界），区间为各历 Wilson 95% 之和。\n")
        w = csv.writer(fh)
        w.writerow(["十年起", "月数", "实物检验", "实物定参", "当时文书检验", "史书检验", "检验稀少", "已估计月数", "估计不合", "估计不合下限", "估计不合上限", "所行之历"])
        for d, b in bins.items():
            w.writerow([d, b["n"], *(b["tier"].get(t, 0) for t in ("A", "fit", "B", "C", "few")), b["t"], f"{b['e']:.2f}", f"{b['lo']:.2f}", f"{b['hi']:.2f}",
                        "、".join(k for k, _ in b["li"].most_common())])

    pct = lambda x: f"{x * 100:.1f}%"
    L = ["# 主序列历日可信度", "",
         "> 由 `scripts/calendar_data/rili_credibility.py` 生成，勿手改。逐十年数据 `data/calendar/rili_credibility_decades.csv`。", "",
         "## 一、问题与方法", "",
         "主序列（前 221—1911 年）每个月的历日，可信到什么程度？有证据之月看证据；无证据之月只有推步，可信程度取决于这部历的推步经得起多少检验。"
         "本页按资料可靠性排序（实物 > 当时文字 > 史书 > 推步），把各历推步拿到它所碰到的最高一级证据前检验，由检验样本估不合率，再外推到同一部历的无证之月。", "",
         "- **检验等级**：A 实物（整年、单月实物历日）→ B 当时文书（汉简公文书，见 [hanjian-checks.md](hanjian-checks.md)）→ C 史书（本纪明书朔、纪日、后修，见 [benji-checks.md](benji-checks.md)），"
         f"该级检验不足 {MIN_N} 月者往下取，三级合计仍不足者为「检验稀少」。汉初颛顼历的实物是拟合合朔改正 δ 所用（[qin-han-calendar-evaluation.md](qin-han-calendar-evaluation.md)），不是独立检验，记「实物定参」。",
         "- **推步可能不合**：证据与推步不合之月，先按 [tongwu-causes.md](tongwu-causes.md) 逐条查成因。有「据」之史文讹误（校勘记、已考讹误、判语、同书相邻朔距不可能）、纪日抽取误、平朔时代食日书朔，都是证据之误，不计；"
         "余下之中，明书朔与推步差一二日者（进朔、退朔、以晦为朔之类），或主因为改历之年、代推之段、无从判断者，计为推步可能不合。差三日以上而无「据」者是系月、月名之事，推步不能差这么多，不计。汉简之条，判「合前后月」「待核」者计，差十日以上判抄误者不计。",
         "- **不合率**：可能不合月数 / 检验月数，区间 Wilson 95%。所计之月仍可能是史书之误，故为**上界**。",
         "- **外推**：无证之月的估计错月 = 该历不合率之和。外推的前提是有证之月与无证之月同质（有证之月多是本纪记了事的月份），此前提未经检验。", "",
         "## 二、总账", "",
         "| 类别 | 月数 | 占比 | 说明 |", "|---|---|---|---|",
         f"| 证实 | {led['证实']} | {pct(led['证实'] / total)} | 实物、史书明书或两条以上纪日合所取历日（汉简另有 {hj_conf} 月证实，未并入主序列逐月表） |",
         f"| 查明·证据之误 | {led['查明·证据之误']} | {pct(led['查明·证据之误'] / total)} | 同误之月，成因已查明是史文讹误等，推步不受影响 |",
         f"| 存疑·同误待核 | {led['存疑·同误待核']} | {pct(led['存疑·同误待核'] / total)} | 同误之月，成因指向推步或行用之历（见第五节） |",
         f"| 存疑·两家互异无据 | {led['存疑·两家互异无据']} | {pct(led['存疑·两家互异无据'] / total)} | 推步与张表不同而无证据可判；主序列按缺省取法取一家 |",
         f"| 无证 | {led['无证']} | {pct(led['无证'] / total)} | 只有推步 |",
         f"| 合计 | {total} | 100% | |", "",
         f"无证之月中，所属之历已检验者估计错 **{exp['p']:.0f}** 月（区间 {exp['lo']:.0f}—{exp['hi']:.0f}）；检验稀少之历 {few['n']} 月，借全部已检验月份的总不合率 {pct(pool_p)}（{pct(pool_lo)}—{pct(pool_hi)}）估计约 {few['n'] * pool_p:.0f} 月。"
         f"两家互异无据之月按最坏情形一半取错计 {led['存疑·两家互异无据'] / 2:.0f} 月，同误待核之月全计 {led['存疑·同误待核']} 月。", ""]
    pt = exp["p"] + few["n"] * pool_p + led["存疑·两家互异无据"] / 2 + led["存疑·同误待核"]
    hi = exp["hi"] + few["n"] * pool_hi + led["存疑·两家互异无据"] + led["存疑·同误待核"]
    L += [f"合计估计错 **{pt:.0f}** 月，历日可信率点估计 **{pct(1 - pt / total)}**；各项取区间上界、互异全计错，可信率下界 **{pct(1 - hi / total)}**。", "",
          "## 三、逐朝代", "", "| 时段 | 月数 | 检验等级构成 | 估计不合率（上界） | 95% 区间 |", "|---|---|---|---|---|"]
    for a, b_, n in ERAS:
        bs = [b for d, b in bins.items() if a <= d < b_]
        N = sum(b["n"] for b in bs)
        T = sum(b["t"] for b in bs)
        tc = Counter()
        for b in bs:
            tc.update(b["tier"])
        comp = "、".join(f"{TIER_NAME[t]} {tc[t] * 100 // N}%" for t in ("A", "fit", "B", "C", "few") if tc[t])
        e, lo, hi_ = (sum(b[x] for b in bs) for x in ("e", "lo", "hi"))
        L.append(f"| {n} | {N} | {comp} | {pct(e / T) if T else '—'} | {pct(lo / T) + '—' + pct(hi_ / T) if T else '—'} |")
    L += ["", "## 四、逐历", "",
          "| 历 | 行用 | 月数 | 检验等级 | 实物检验/不合 | 汉简检验/不合 | 史书检验/不合 | 不合率 | 95% 区间 |", "|---|---|---|---|---|---|---|---|---|"]
    for li, s in seg.items():
        ev = s["ev"]
        est = "—" if s["tier"] == "few" else pct(s["p"])
        ci = "—" if s["tier"] == "few" else f"{pct(s['lo'])}—{pct(s['hi'])}"
        L.append(f"| {li} | {s['y0']}—{s['y1']} | {s['months']} | {TIER_NAME[s['tier']]} | {ev['A'][0]}/{ev['A'][1]} | {ev['B'][0]}/{ev['B'][1]} | {ev['C'][0]}/{ev['C'][1]} | {est} | {ci} |")
    L += ["", "## 五、推步可能不合之月（待核清单）", "",
          "这些月份是推步本身最值得再查的地方：补证据（墓志、出土历日、他书）可以把它们转为证实或查明。", "",
          "| 历 | 月 | 等级 | 证据与成因 |", "|---|---|---|---|"]
    for li, s in seg.items():
        for lab, g, why in s["res"]:
            L.append(f"| {li} | {lab} | {g} | {why} |")
    L += ["", "## 六、限制", "",
          "- 不合率是上界；同时也只覆盖有证之月。无证之月是否与有证之月同质，未经检验。",
          "- 汉简按「年 + 简文日期」归月，新莽建丑月名未逐条换算，个别月份可能归错；汉简证实之月未并入主序列逐月表。",
          "- 检验稀少之历借总不合率，是权宜之计；补证据后应改用本历之数。",
          "- 两家互异无据之月按一半取错计，没有用推步与张表各自的命中率加权。", ""]
    OUT_MD.write_text("\n".join(L), encoding="utf-8")
    print(f"总 {total} 月；{dict(led)}；无证估计错 {exp['p']:.0f}（{exp['lo']:.0f}—{exp['hi']:.0f}）；合计 {pt:.0f}；可信率 {pct(1 - pt / total)}，下界 {pct(1 - hi / total)}")


if __name__ == "__main__":
    main()
