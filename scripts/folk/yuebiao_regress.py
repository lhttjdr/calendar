#!/usr/bin/env python3
"""对比新旧两份月表推算，列出已核日子（data/folk/xieji_yuebiao_checks.csv 中为「合」者）里宜忌有变的。
用法（仓库根）：python3 scripts/folk/yuebiao_regress.py 旧.csv"""
import csv
import sys

old = {(r["月"], r["干支"]): r for r in csv.DictReader(open(sys.argv[1], encoding="utf-8"))}
new = {(r["月"], r["干支"]): r for r in csv.DictReader(open("data/folk/xieji_yuebiao.csv", encoding="utf-8"))}
ok = {(r["月"], r["干支"]) for r in csv.DictReader(open("data/folk/xieji_yuebiao_checks.csv", encoding="utf-8"))
      if r["結果"] == "合"}
changed = [k for k in new if (new[k]["宜"], new[k]["忌"]) != (old[k]["宜"], old[k]["忌"])]
bad = [k for k in changed if k in ok]
print(f"变动 {len(changed)} 日，其中已核为合者 {len(bad)} 日")
for k in bad:
    print("  ", "-".join(k), "\n    旧宜", old[k]["宜"], "\n    新宜", new[k]["宜"], "\n    旧忌", old[k]["忌"], "\n    新忌", new[k]["忌"])
