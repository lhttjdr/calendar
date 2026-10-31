"""《宝祐四年会天历》（1256，当时历日的钞本）核验：自校，主表比较，开禧历上推比较（会天历术不传）。

自校：气节干支 = 月一日干支 + 日 − 1；月大小与下月一日相合；全年「凡三百五十四日」。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/check_huitian_1256.py
"""
import csv
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import discriminating_records as d  # noqa: E402
import gaiyuan_date as g  # noqa: E402
import song_dingshuo as s  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]


def main():
    with open(ROOT / "data/calendar/会天历_1256.csv", encoding="utf-8") as f:
        rows = list(csv.DictReader(line for line in f if not line.startswith("#")))
    months = [r for r in rows if r["类"] == "月"]
    idx = [g.GANZHI.index(r["干支"]) for r in months]
    bad = 0
    for i, r in enumerate(months[:-1]):
        n = 30 if r["大小"] == "大" else 29
        if (idx[i] + n) % 60 != idx[i + 1]:
            print("月大小不合", r["名"])
            bad += 1
    total = sum(30 if r["大小"] == "大" else 29 for r in months)
    print("全年日数", total, "（原文三百五十四）")
    qi = [r for r in rows if r["类"] == "气" and r["干支"]]
    for r in qi:
        if (idx[int(r["月"]) - 1] + int(r["日"]) - 1) % 60 != g.GANZHI.index(r["干支"]):
            print("气节干支不合", r["名"])
            bad += 1
    print(f"自校：月 {len(months)}、气 {len(qi)}，不合 {bad}")
    main_t = d.tables()["主表"]
    li = s.KAIXI
    li.off = g.jdn(1206, 12, 15)
    dz = li.tong(li.y_ji) // li.f
    li.off = s.c.align_offset(dz, g.jdn(1206, 12, 15))
    moons = li.moons(1254, 1258)
    out = []
    for r, i in zip(months, idx):
        s0 = main_t[(1256, int(r["月"]), False)]
        t = min(moons, key=lambda x: abs(int(x // li.f) + li.off - s0))
        day = int(t // li.f) + li.off
        h = float((t % li.f) / li.f * 24)
        lim = li.limit(t)
        jin = day + (h >= lim)
        out.append({"月": r["名"], "会天历一日": r["干支"], "主表": g.ganzhi_of_jdn(s0), "开禧定朔（上推）": g.ganzhi_of_jdn(day),
                    "小余（时）": f"{h:.2f}", "进朔限（时）": f"{lim:.2f}", "开禧进朔": g.ganzhi_of_jdn(jin),
                    "合主表": int(g.ganzhi_of_jdn(s0) == r["干支"]), "合开禧": int(g.ganzhi_of_jdn(jin) == r["干支"]),
                    "合开禧不进朔": int(g.ganzhi_of_jdn(day) == r["干支"])})
    with open(ROOT / "data/calendar/huitian_1256_check.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 《宝祐四年会天历》各月一日与主表、开禧历上推比较（scripts/calendar_data/check_huitian_1256.py）。会天历术不传，开禧为上一部历，只作对照。\n")
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    for o in out:
        print(o)
    print("合主表", sum(o["合主表"] for o in out), "合开禧进朔", sum(o["合开禧"] for o in out), "合开禧不进朔", sum(o["合开禧不进朔"] for o in out), "/", len(out))


if __name__ == "__main__":
    main()
