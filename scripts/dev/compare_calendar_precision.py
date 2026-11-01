"""比对两种数值后端的朔、节气时刻（rust/core/examples/precision_calendar.rs 的输出）。

用法：python scripts/dev/compare_calendar_precision.py <dd.txt> <f64.txt>
打印逐类最大差，以及按东八区日界（JD + 8/24 + 0.5 取整；时刻取力学时，只看两后端是否分属两天）判定不同的条数。条目不一一对应或日界不同则非零退出。
"""
import math
import sys
from collections import defaultdict


def load(path):
    out = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            y, kind, k, jd = line.split()
            out.append((int(y), kind, k, float(jd)))
    return out


dd, f64 = load(sys.argv[1]), load(sys.argv[2])
if [r[:3] for r in dd] != [r[:3] for r in f64]:
    sys.exit(f"条目不对应：{len(dd)} 与 {len(f64)}")
worst = defaultdict(float)
crossings = 0
for a, b in zip(dd, f64):
    worst[a[1]] = max(worst[a[1]], abs(a[3] - b[3]) * 86400)
    crossings += math.floor(a[3] + 8 / 24 + 0.5) != math.floor(b[3] + 8 / 24 + 0.5)
for kind, seconds in worst.items():
    print(f"{kind}：{sum(r[1] == kind for r in dd)} 条，最大差 {seconds:.4f} s")
print(f"东八区日界不同：{crossings} 条")
sys.exit(1 if crossings else 0)
