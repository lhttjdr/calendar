"""史书所记入宿度（「在牽牛二度」「同躔壁三度」）与推算入宿度逐条对照。

推算入宿度取各书检验表（data/phenomena/checks/<书>_star_records.csv）「推算入宿」列：形如「壁3.21（史载 壁3.0）」，
宿度体系见 rust/core/src/astronomy/sky_culture/tests_records.rs::lodge_model（清时宪历黄道 360°，余为陈卓距星赤道古度）。
只取日干支在所记之月者。同宿者计度差（推算 − 所记）；不同宿者记「异宿」，不计度差。
输出 data/phenomena/checks/lodge_degrees.csv，由 checks_to_md.py 收入 doc/astronomy/phenomena-checks.md。

用法（仓库根）：uv run --project scripts python scripts/phenomena/lodge_degrees.py
"""
import csv
import re
import statistics
from pathlib import Path

BOOKS = [("hanshu", "《汉书》"), ("houhanshu", "《后汉书》"), ("jinshu", "《晋书》"), ("songshu", "《宋书》"), ("nanqishu", "《南齐书》"),
         ("weishu", "《魏书》"), ("suishu", "《隋书》"), ("xintangshu", "《新唐书》"), ("jiutangshu", "《旧唐书》"),
         ("jiuwudaishi", "《旧五代史》"), ("xinwudaishi", "《新五代史》"), ("songshi", "《宋史》"), ("jinshi", "《金史》"),
         ("yuanshi", "《元史》"), ("mingshi", "《明史》"), ("qingshigao", "《清史稿》")]
CELL = re.compile(r"^(?P<m>[^\d\-]+)(?P<d>-?[\d.]+)（史载 (?P<rm>[^\d]+)(?P<rd>[\d.]+)）$")


def main():
    out = []
    for book, name in BOOKS:
        path = Path(f"data/phenomena/checks/{book}_star_records.csv")
        if not path.exists():
            continue
        rows = list(csv.DictReader([ln for ln in path.read_text(encoding="utf-8").splitlines() if not ln.startswith("#")]))
        if "推算入宿" not in (rows[0] if rows else {}):
            continue
        for r in rows:
            m = CELL.match(r["推算入宿"])
            if not m or r["干支在月"] != "是":
                continue
            same = m["m"] == m["rm"]
            out.append({"书": name, "序": r["序"], "纪年日期": r["纪年日期"], "公历(儒略)": r["公历(儒略)"], "记载": r["记载"],
                        "所记": f"{m['rm']}{m['rd']}", "推算": f"{m['m']}{float(m['d']):.2f}",
                        "度差": f"{float(m['d']) - float(m['rd']):+.2f}" if same else "异宿", "原文": r["原文"][:40]})
    dst = Path("data/phenomena/checks/lodge_degrees.csv")
    with open(dst, "w", encoding="utf-8", newline="") as f:
        f.write("# 史书所记入宿度与推算入宿度（scripts/phenomena/lodge_degrees.py）；只取日干支在月者。度差 = 推算 − 所记（该书宿度体系的度）。\n")
        by = {}
        for o in out:
            by.setdefault(o["书"], []).append(o)
        for name, v in by.items():
            d = [abs(float(o["度差"])) for o in v if o["度差"] != "异宿"]
            if d:
                f.write(f"# {name} {len(v)} 条：同宿 {len(d)}，|度差| 中位 {statistics.median(d):.2f}，≤1 度 {sum(x <= 1 for x in d)}，"
                        f"≤2 度 {sum(x <= 2 for x in d)}\n")
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    print(dst, len(out), "条")


if __name__ == "__main__":
    main()
