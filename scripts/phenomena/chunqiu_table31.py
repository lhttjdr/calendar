"""张培瑜《中国古代历法》表 3-1（页 167）与本项目日食检验对照。
第一步写 data/phenomena/chunqiu_zhang_3_1_input.csv（书给儒略日，historical_eclipses 的输入）；算出 *_check.csv 后第二步写 chunqiu_zhang_3_1.csv。
书的括号食分（宣十、襄十五）照录为数值。书「无日食」「不见」者食分留空。
用法（仓库根）：uv run --project scripts python scripts/phenomena/chunqiu_table31.py；
cd rust && cargo run --release -p lunar-core --example historical_eclipses --no-default-features --features real-f64 -- solar
../data/phenomena/chunqiu_zhang_3_1_input.csv ../data/phenomena/chunqiu_zhang_3_1_check.csv 116.98 35.6；再跑一遍本脚本。
"""
import csv
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "calendar_data"))
import gaiyuan_date as g  # noqa: E402

JD = [1458496, 1462659, 1467857, 1474619, 1477218, 1477750, 1479137, 1482415, 1484837, 1485959, 1492810, 1498008, 1502171, 1502703, 1501670, 1511533, 1512064, 1517262, 1517764,
      1519683, 1520037, 1520067, 1520540, 1521071, 1521100, 1522282, 1526091, 1529044, 1529900, 1531289, 1531820, 1532322, 1535098, 1537018, 1539793, 1540827, 1545847]
MAG = [.47, 1.00, .58, .68, .88, .72, .84, .88, .26, None, .79, .87, .91, .50, .43, .96, .66, .65, .35, None, .69, None, .91, 1.01, None, .94, .35, .93, .82, .62, .59, .58, .53, .44, .87, .51, .82]
NOTE = {3: "书：壬子", 9: "书：无日食；儒略日扫描似作 1485859，与书干支壬子不合，按 1485959", 12: "书：十月甲子", 14: "书：宣七年六月癸卯", 18: "书：七月丁巳", 19: "书：曲阜不见", 21: "书：无日食", 24: "书：无日食", 28: "书：九月晦癸酉"}
P = Path("data/phenomena")


def main():
    with open(P / "chunqiu_solar_eclipses.csv", encoding="utf-8") as f:
        base = list(csv.DictReader(f))
    assert len(base) == len(JD) == len(MAG)
    with open(P / "chunqiu_zhang_3_1_input.csv", "w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(base[0]))
        w.writeheader()
        for r, j in zip(base, JD):
            y, m, d = g.from_jdn(j)
            w.writerow({**r, "jdn": j, "julian": f"{y}-{m:02d}-{d:02d}", "lunar_day": ""})
    chk = P / "chunqiu_zhang_3_1_check.csv"
    if not chk.exists():
        return
    with open(chk, encoding="utf-8") as f:
        c_ = list(csv.DictReader(f))
    with open(P / "chunqiu_solar_eclipses_check.csv", encoding="utf-8") as f:
        mine = list(csv.DictReader(f))
    out, diffs = [], []
    for i, (r, j, mg, c1, m1) in enumerate(zip(base, JD, MAG, c_, mine)):
        v = float(c1["site_magnitude_0"]) if c1["site_magnitude_0"] else None
        d = round(v - mg, 2) if v is not None and mg is not None else ""
        if d != "":
            diffs.append(abs(d))
        out.append({"公": r["emp"], "年": r["year_in_era"], "经月": r["month"], "经干支": r["ganzhi"], "书儒略日": j, "书干支": g.ganzhi_of_jdn(j), "书食分": mg if mg is not None else "",
                    "本项目判定（书之日）": c1["verdict"], "食型": c1["kind"], "日差": c1["day_offset"], "本项目食分": c1["site_magnitude_0"], "差": d,
                    "本项目按春秋历换算之日": m1["jdn"], "同日": int(int(m1["jdn"]) == j), "书注": NOTE.get(i, "")})
    with open(P / "chunqiu_zhang_3_1.csv", "w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(out[0]))
        w.writeheader()
        w.writerows(out)
    print(f"同日 {sum(o['同日'] for o in out)}/37；两边都有食分 {len(diffs)} 条，平均差 {sum(diffs) / len(diffs):.3f}，最大 {max(diffs)}")
    for o in out:
        print(o["公"], o["年"], o["书干支"], o["书食分"], o["本项目判定（书之日）"], o["日差"], o["本项目食分"], o["差"], "同" if o["同日"] else "异", o["书注"])


if __name__ == "__main__":
    main()
