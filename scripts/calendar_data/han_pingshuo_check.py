"""两汉平朔历（太初/三统、东汉四分）按术文推步月表，与主表、《汉书》《后汉书》本纪纪日比较。

两历上元都是甲子日夜半朔旦冬至，故日序、朔、气同一原点：第 k 朔 = ⌊k × 月法 / 日法⌋，冬至 = 积年 × 岁实。
- 三统历（《汉书·律历志》）：日法 81，月法 2392（朔策 29 + 43/81），岁实 562120 / 1539（统法），上元至太初元年（前 104）143127 岁。
- 四分历（《后汉书·律历志》）：日法 940（蔀月），通数 27759，岁实 1461/4，上元庚辰至熹平三年（174）9455 岁（算上，见 pingshuo_params_check.py）。
置闰：十一月为含冬至之月；岁中 13 月者，以第一个无中气之月为闰（平气）。月以建寅编号，岁 Y 的正月在公历 Y 年初。
上元原点对齐 JDN：取与主表朔日合者最多之偏移（只在 ±60 日内找，验算元日干支为甲子）。
输出 data/calendar/han_pingshuo_check.csv（本纪逐条）。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/han_pingshuo_check.py
"""
import csv
import sys
from fractions import Fraction as Fr
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import discriminating_records as d  # noqa: E402
import gaiyuan_date as g  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
LI = {
    "三统": dict(yue=2392, ri=81, sui=Fr(562120, 1539), base=-104, ji=143127),
    "四分": dict(yue=27759, ri=940, sui=Fr(1461, 4), base=174, ji=9455),
}


def table(name, y0, y1, off):
    p = LI[name]
    out = {}
    for y in range(y0, y1 + 1):
        n = p["ji"] + (y - 1 - p["base"])           # 冬至在 y−1 年之岁的积年
        dz0, dz1 = n * p["sui"], (n + 1) * p["sui"]
        k = int(dz0 * p["ri"] / p["yue"]) - 2
        days = [k_ * p["yue"] // p["ri"] for k_ in range(k, k + 17)]
        zq = [int(dz0 + i * p["sui"] / 12) for i in range(14)]
        i0 = next(i for i in range(16) if days[i] <= int(dz0) < days[i + 1])
        i1 = next(i for i in range(16) if days[i] <= int(dz1) < days[i + 1])
        leap = None
        if i1 - i0 == 13:
            leap = next(i for i in range(i0 + 1, i1) if not any(days[i] <= z < days[i + 1] for z in zq))
        lab, yy = 11, y - 1
        for i in range(i0, i1):
            if i == leap:
                out[(yy_prev, lab_prev, True)] = days[i] + off   # 闰月属前一月所在之岁
                continue
            out[(yy, lab, False)] = days[i] + off
            lab_prev, yy_prev = lab, yy
            lab += 1
            if lab == 13:
                lab, yy = 1, y
    return out


def align(name, main, y0, y1):
    probe = table(name, y0, y1, 0)
    k = next(iter(probe))
    guess = main[k] - probe[k] if k in main else 0
    cands = []
    for o in range(guess - 60, guess + 61):
        cands.append((sum(main.get(kk) == v + o for kk, v in probe.items()), o))
    return max(cands)


def main():
    main_t = d.tables()["主表"]
    res = {}
    for name, y0, y1 in (("三统", -103, 84), ("四分", 85, 220)):
        hit, off = align(name, main_t, y0, y1)
        t = table(name, y0, y1, off)
        same = sum(main_t.get(k) == v for k, v in t.items())
        print(f"{name}：{y0}—{y1} 推步 {len(t)} 月，与主表同月同日 {same}；元日干支 {g.ganzhi_of_jdn(off)}")
        res[name] = (y0, y1, t)
    rows = []
    for book in ("hanshu_benji", "houhanshu_benji"):
        for r in csv.DictReader(open(ROOT / f"data/calendar/本纪纪日/{book}.csv", encoding="utf-8")):
            y = int(r["year"])
            if y < -103 or not r["month"].lstrip("闰").isdigit():
                continue
            m, lp = int(r["month"].lstrip("闰")), r["month"].startswith("闰")
            name = "三统" if y <= 84 else "四分"
            _, _, t = res[name]
            gz = g.GANZHI.index(r["ganzhi"])

            def inm(tab):
                s = tab.get((y, m, lp))
                if s is None:
                    return ""
                e = min((v for v in tab.values() if v > s), default=s + 30)
                return int((gz - (s + 49)) % 60 < e - s)
            rows.append({"书": book, "年": y, "纪年": f"{r['era']}{r['year_in_era']}", "月": r["month"], "干支": r["ganzhi"],
                         "历": name, "主表在月": inm(main_t), "推步在月": inm(t), "朔": r["position"], "原文": r["text"][:40]})
    with open(ROOT / "data/calendar/han_pingshuo_check.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 两汉本纪纪日在主表与按术文推步（三统、四分）月中否（scripts/calendar_data/han_pingshuo_check.py）。\n")
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)
    for name in ("三统", "四分"):
        R = [r for r in rows if r["历"] == name]
        a = sum(r["主表在月"] == 1 for r in R)
        b = sum(r["推步在月"] == 1 for r in R)
        diff = [r for r in R if r["主表在月"] != r["推步在月"]]
        print(f"{name} 本纪 {len(R)} 条：主表在月 {a}，推步在月 {b}；二者判断不同 {len(diff)} 条")
        for r in diff[:30]:
            print("   ", r["年"], r["纪年"], r["月"], r["干支"], "主表", r["主表在月"], "推步", r["推步在月"], r["原文"][:24])


if __name__ == "__main__":
    main()
