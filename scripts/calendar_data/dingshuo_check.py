"""定朔历复原的共用比较：与主表逐月、与本纪明书朔日逐条，写 data/calendar/<csv>，印不合之条与进朔分界。"""
import csv
from pathlib import Path

import discriminating_records as d
import gaiyuan_date as g

ROOT = Path(__file__).resolve().parents[2]


def align_offset(day_index, approx_jdn):
    """日序（甲子为 0）→ JDN 之差：取 approx_jdn 附近干支相合之日。"""
    return min((approx_jdn + k for k in range(-40, 40) if (approx_jdn + k + 49) % 60 == day_index % 60),
               key=lambda j: abs(j - approx_jdn)) - day_index


def compare(name, f, off, moons, y0, y1, csv_name, note, limit=lambda t: 18.0, books=None, skip=()):
    """moons：定朔分（升序）；f：日法；limit(t)：定朔 t 的进朔限（时）；books：只比这些书的本纪（缺省全部）；skip：区间内不比的年（他历行用之年）。name 如「大衍」，列名「大衍定朔」「大衍进朔」。"""
    by = {v: k for k, v in d.tables()["主表"].items()}
    same, n, late, early = 0, 0, [], []
    for t in moons:
        j = int(t // f) + off
        near_ = [k for k in range(j - 2, j + 3) if k in by and y0 <= by[k][0] <= y1 and by[k][0] not in skip]
        if not near_:
            continue
        k = min(near_, key=lambda x: abs(x - j))
        n += 1
        same += k == j
        h = float((t % f) / f * 24)
        (late if k == j + 1 else early if k == j - 1 else []).append(h)
    print(f"{y0}—{y1} 年：{name}定朔与主表同日 {same}/{n}；主表晚一日 {len(late)} 月"
          + (f" 小余 {min(late):.1f}—{max(late):.1f} 时" if late else "") + f"；早一日 {len(early)} 月")

    def nearest(hint):
        t = min(moons, key=lambda x: abs(int(x // f) + off - hint))
        return int(t // f) + off, float((t % f) / f * 24), limit(t)
    with open(ROOT / "data/calendar/main_table_check.csv", encoding="utf-8") as fh:
        rows = list(csv.DictReader(line for line in fh if not line.startswith("#")))
    out = []
    for r in rows:
        y = int(r["年"])
        if not y0 <= y <= y1 or y in skip or not r["主表朔 JDN"] or r["类"] in ("其他", "差一月") or (books and r["书"] not in books):
            continue
        rec, mt = int(r["本纪朔 JDN"]), int(r["主表朔 JDN"])
        k, h, lim = nearest(mt)
        jin = k + (h >= lim)
        out.append({"书": r["书"], "年": y, "纪年": r["纪年"], "月": r["月"], "本纪朔": r["本纪朔"], "主表朔": g.ganzhi_of_jdn(mt),
                    f"{name}定朔": g.ganzhi_of_jdn(k), "定朔小余（时）": f"{h:.2f}", "进朔限（时）": f"{lim:.2f}", f"{name}进朔": g.ganzhi_of_jdn(jin),
                    "本纪合主表": int(rec == mt), "本纪合不进朔": int(rec == k), "本纪合进朔": int(rec == jin),
                    "日食": r["日食"], "他书同记": r["他书同记"], "原文": r["原文"]})
    with open(ROOT / "data/calendar" / csv_name, "w", encoding="utf-8", newline="") as fh:
        fh.write(f"# {note}\n")
        w = csv.DictWriter(fh, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    print(f"本纪明书朔日 {len(out)} 条：合主表 {sum(o['本纪合主表'] for o in out)}，合不进朔 {sum(o['本纪合不进朔'] for o in out)}，"
          f"合进朔 {sum(o['本纪合进朔'] for o in out)}")
    for o in out:
        if not (o["本纪合主表"] and o["本纪合进朔"]):
            print("  ", o["书"], o["年"], o["月"], o["本纪朔"], "主表", o["主表朔"], name, o[f"{name}定朔"], o["定朔小余（时）"], "限", o["进朔限（时）"], o["原文"][:16])
    print("本纪进朔者小余（最低十）", sorted(float(o["定朔小余（时）"]) for o in out if not o["本纪合不进朔"] and o["本纪合进朔"])[:10])
    print("本纪不进、小余≥15（最高十）", sorted((float(o["定朔小余（时）"]), o["年"], o["月"], o["日食"]) for o in out
                                       if o["本纪合不进朔"] and float(o["定朔小余（时）"]) >= 15)[-10:])
