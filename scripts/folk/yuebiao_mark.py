#!/usr/bin/env python3
"""
把对照影印页核过的日子记入 data/folk/xieji_yuebiao_checks.csv。
用法（仓库根）：python3 scripts/folk/yuebiao_mark.py 1-壬申,1-癸酉,...   （结果记为「合」）
          python3 scripts/folk/yuebiao_mark.py --todo [月]                （列出未核的日子）
"""
import csv
import sys

P = "data/folk/xieji_yuebiao_checks.csv"
G, Z = "甲乙丙丁戊己庚辛壬癸", "子丑寅卯辰巳午未申酉戌亥"
GZ = [G[k % 10] + Z[k % 12] for k in range(60)]


def page(m, k):
    if m == 1 and k < 8:
        return f"06056517.cn PDF 第 {110 + k} 頁"
    return f"060676{92 + (m - 1) // 2:02d}.cn PDF 第 {(6 if m % 2 else 73) + k + 1} 頁"


rows = {(int(r["月"]), r["干支"]): r for r in csv.DictReader(open(P, encoding="utf-8"))}
if sys.argv[1] == "--todo":
    ms = [int(sys.argv[2])] if len(sys.argv) > 2 else range(1, 13)
    print(",".join(f"{m}-{g}" for m in ms for g in GZ if (m, g) not in rows))
    sys.exit()
for x in sys.argv[1].split(","):
    m, g = int(x.split("-")[0]), x.split("-")[1]
    rows.setdefault((m, g), {"月": m, "干支": g, "影印本": page(m, GZ.index(g)), "結果": "合", "說明": ""})
with open(P, "w", newline="", encoding="utf-8") as f:
    w = csv.DictWriter(f, fieldnames=["月", "干支", "影印本", "結果", "說明"])
    w.writeheader()
    for (m, g) in sorted(rows, key=lambda x: (x[0], GZ.index(x[1]))):
        w.writerow(rows[(m, g)])
print(len(rows), "日已核")
