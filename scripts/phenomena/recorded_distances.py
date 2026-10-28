"""史书所记距离（「去東井一尺」「相去七寸」）与推算角距对照：估计当时「一尺」合多少度。

只取日干支在所记之月、记有尺寸、且推算角距有值的记录。推算角距依次取具体星角距、天体间角距、距星官最小角距
（与 checks_to_md.py 的主角距同序）。尺寸由记录表 qualifier 列（无则原文）解析：「一尺」「五寸」「二尺五寸」「九寸」。
输出 data/phenomena/checks/recorded_distances.csv（逐条），由 checks_to_md.py 收入 doc/astronomy/phenomena-checks.md。

用法（仓库根）：uv run --project scripts python scripts/phenomena/recorded_distances.py
"""
import csv
import re
import statistics
from pathlib import Path

BOOKS = [("jinshu", "《晋书》"), ("songshu", "《宋书》"), ("nanqishu", "《南齐书》"), ("weishu", "《魏书》"), ("suishu", "《隋书》"),
         ("xintangshu", "《新唐书》"), ("jiutangshu", "《旧唐书》"),
         ("jiuwudaishi", "《旧五代史》"), ("xinwudaishi", "《新五代史》"), ("songshi", "《宋史》"), ("jinshi", "《金史》"),
         ("yuanshi", "《元史》"), ("mingshi", "《明史》"), ("qingshigao", "《清史稿》")]
CN = {c: i for i, c in enumerate("〇一二三四五六七八九")}
DIST = re.compile(r"(?:([一二三四五六七八九十]+)尺)?(?:([一二三四五六七八九]+)寸)?")


def cn(s):
    if not s:
        return 0
    if s == "十":
        return 10
    if "十" in s:
        a, b = s.split("十")
        return (CN[a] if a else 1) * 10 + (CN[b] if b else 0)
    return CN[s]


def chi(text):
    """文中第一个尺寸（尺）；「所」「許」「餘」等后缀忽略。无尺寸返回 None。"""
    for m in DIST.finditer(text):
        if m.group(1) or m.group(2):
            return cn(m.group(1)) + cn(m.group(2)) / 10
    return None


def num(s):
    try:
        return float(s.split("（")[0].replace("+", ""))
    except ValueError:
        return None


def main():
    out = []
    for book, name in BOOKS:
        recs = list(csv.DictReader(open(f"data/phenomena/{book}_star_records.csv", encoding="utf-8")))
        path = Path(f"data/phenomena/checks/{book}_star_records.csv")
        if not path.exists():
            continue
        lines = [ln for ln in path.read_text(encoding="utf-8").splitlines() if not ln.startswith("#")]
        rows = list(csv.DictReader(lines))
        for r in rows:
            rec = recs[int(r["序"]) - 1]
            d = chi(rec["qualifier"]) if rec["qualifier"] else None
            if d is None:
                # qualifier 只截前 12 字；距离写在同一分句之后（「相去七寸」）者从原文该分句取
                m = re.search(r"(?:相)?去([一二三四五六七八九十尺寸]+)", rec["text"])
                d = chi(m.group(1)) if m else None
            if d is None or r["干支在月"] != "是":
                continue
            deg = None
            for col in ("具体星角距°", "天体间最大角距°", "距星官最小角距°"):
                deg = num(r.get(col, ""))
                if deg is not None:
                    break
            if deg is None:
                continue
            out.append({"书": name, "序": r["序"], "纪年日期": r["纪年日期"], "公历(儒略)": r["公历(儒略)"], "记载": r["记载"], "天体": rec["bodies"],
                        "所记尺": f"{d:.1f}", "推算角距°": f"{deg:.2f}", "度/尺": f"{deg / d:.2f}", "原文": rec["text"][:40]})
    dst = Path("data/phenomena/checks/recorded_distances.csv")
    ratios = [float(o["度/尺"]) for o in out]
    # 月一夜内移动约 6°，只记到夜的月记录推算角距含时刻误差；行星一夜内移动 <1°，以行星记录估计尺度
    planet = [float(o["度/尺"]) for o in out if "Moon" not in o["天体"] and float(o["所记尺"]) >= 0.5]
    with open(dst, "w", encoding="utf-8", newline="") as f:
        f.write("# 史书所记距离（尺、寸）与推算角距对照（scripts/phenomena/recorded_distances.py）；只取日干支在月者。\n")
        if ratios:
            f.write(f"# {len(ratios)} 条；度/尺 中位 {statistics.median(ratios):.2f}，四分位 "
                    f"{statistics.quantiles(ratios, n=4)[0]:.2f}—{statistics.quantiles(ratios, n=4)[2]:.2f}\n")
        if len(planet) >= 4:
            q = statistics.quantiles(planet, n=4)
            f.write(f"# 行星记录（不含月）且所记 ≥5 寸者 {len(planet)} 条：度/尺 中位 {statistics.median(planet):.2f}，四分位 {q[0]:.2f}—{q[2]:.2f}\n")
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    print(dst, len(out), "条", f"度/尺 中位 {statistics.median(ratios):.2f}" if ratios else "")


if __name__ == "__main__":
    main()
