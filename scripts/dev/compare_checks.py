"""比对两种数值后端（f64 / double-double）写出的逐条校验表：除下列容差外逐格相同（# 说明行除外）。

容差只给「极值时刻」列：极值附近目标函数平坦，f64 构建的儒略日只到约 40 µs，行星方向带约 3e-12 rad 的量化噪声；
极值时刻以五点差分导数之根求（search::extremum_by_derivative_root），噪声折合约 0.2 s。其物理意义更粗：模型误差 0.36″ 时
大距时刻本身只定到数小时。
其余各列（角距、食分、γ、星等、宿度……）须逐格相同。

用法：python scripts/dev/compare_checks.py <dd 目录> <f64 目录>；不一致则非零退出。
"""
import csv
import sys
from pathlib import Path

# (文件, 列名) → (按行判别函数, 容差)。容差单位同列（秒）。
TOLERANCE = {
    # 大距、最接近：五点差分导数求根（search::extremum_by_derivative_root），f64 构建的时间量化噪声使之差约 0.2 s
    # 留：经度变化率二分求根，容差 10 s；合、冲：1 s
    ("planet_events_vs_skyfield.csv", "时刻差 s"): lambda row: {"Elong": 0.5, "Stat": 10.0}.get(row.get("类别"), 1.0),
    ("close_approaches_vs_skyfield.csv", "时刻差 s"): lambda row: 0.5,
}


def rows(path):
    with open(path, encoding="utf-8") as f:
        return list(csv.DictReader(line for line in f if not line.startswith("#")))


def main(dd_dir, f64_dir):
    bad = 0
    for p in sorted(Path(dd_dir).glob("*.csv")):
        q = Path(f64_dir) / p.name
        if not q.exists():
            print(f"缺 f64 输出：{p.name}")
            bad += 1
            continue
        a, b = rows(p), rows(q)
        if len(a) != len(b) or (a and a[0].keys() != b[0].keys()):
            print(f"不一致：{p.name} 行数或列不同（{len(a)} / {len(b)}）")
            bad += 1
            continue
        diffs, tolerated, worst = [], 0, 0.0
        for i, (ra, rb) in enumerate(zip(a, b), 1):
            for col in ra:
                if ra[col] == rb[col]:
                    continue
                tol = TOLERANCE.get((p.name, col))
                if tol is not None:
                    try:
                        d = abs(float(ra[col]) - float(rb[col]))
                    except ValueError:
                        d = None
                    if d is not None and d <= tol(ra):
                        tolerated += 1
                        worst = max(worst, d)
                        continue
                diffs.append(f"  第 {i} 行「{col}」：{ra[col]} / {rb[col]}")
        if diffs:
            print(f"不一致：{p.name}")
            print("\n".join(diffs[:10]))
            bad += 1
        else:
            note = f"；极值时刻在搜索容差内不同 {tolerated} 格，最大 {worst:g} s" if tolerated else ""
            print(f"一致：{p.name}（{len(a)} 行{note}）")
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main(*sys.argv[1:3])
