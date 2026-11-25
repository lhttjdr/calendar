"""高昌延寿七年（630）历日：吐鲁番阿斯塔那 387 号墓出土残历（86TAM387:38-4）与张表、玄始历外推逐月比较。

底本：邓文宽《吐鲁番新出〈高昌延寿七年（630）历日〉考》，收入《敦煌吐鲁番天文历法研究》（甘肃教育出版社 2002），
书页 228—240（PDF 页 250—262，书页 = PDF 页 − 22）。残历存十列（三月至十二月）各十日左右，每日记干支与建除；
各列同一日期上下对齐（表格式），由此得各月相对位置与大小；以《高昌赵悦子妻马氏墓表》「延壽七年庚寅歲七月□□朔，
十六日己卯」定七月甲子朔，以《高昌曹妻蘇氏墓表》「延壽八年辛卯歲正月辛酉朔」定十二月小。正月、閏正月、二月三朔
是邓氏取王素《麴氏高昌历法初探》之推拟（残历上部无存），不作证据。邓文表一为复原全年，这里只取各月朔。
录文：data/calendar/gaochang_630.csv（月、闰、朔干支、依据、书页）。

比较：张表（唐贞观四年，戊寅历）；玄始历外推（北凉玄始历常数，push_calendar::xuanshi_params，平朔平气无中气置闰，
本脚本照 push_calendar::year_new_moons 移植；史家言高昌行北凉旧历，见 doc/calendar/calendar-compendium.md「高昌」）。
输出 doc/calendar/gaochang-630-checks.md。
用法（仓库根）：uv run --project scripts python scripts/calendar_data/gaochang_630_check.py
"""
import csv
import math
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import discriminating_records as d  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
GZ = ["甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12] for i in range(60)]
MON = ["正", "二", "三", "四", "五", "六", "七", "八", "九", "十", "十一", "十二"]
# push_calendar::xuanshi_params
XUANSHI = dict(lunar=29.0 + 47251.0 / 89052.0, solar=365.0 + 1759.0 / 7200.0,
               jd_epoch=1871510.5 + 4995.0 / 89052.0 + 1e-6, jdw=1721048.5 + 1189.0 / 1200.0 + 1e-6)


def pingshuo_year(p, n):
    """push_calendar::year_new_moons 的移植：岁 n 各月 (月, 闰, 朔 JDN)"""
    day = lambda jd: math.floor(jd + 0.5)  # noqa: E731
    lunar, solar = p["lunar"], p["solar"]
    z1 = p["jdw"] + (n + 1.0 / 6.0) * solar
    i1 = math.floor((day(z1) + 0.5 - p["jd_epoch"]) / lunar)
    m1 = p["jd_epoch"] + i1 * lunar
    has_leap = m1 + 13.0 * lunar <= day(z1 + solar) + 0.5
    nm = 13 if has_leap else 12
    starts = [day(m1 + j * lunar) for j in range(nm + 1)]
    leap = 0
    if has_leap:
        zq = [day(z1 + k * solar / 12.0) for k in range(13)]
        leap = next(j for j in range(nm) if not any(starts[j] <= q < starts[j + 1] for q in zq))
    out = []
    for k in range(nm):
        if leap and k == leap:
            out.append((leap, True, starts[k]))
        elif leap and k > leap:
            out.append((k, False, starts[k]))
        else:
            out.append((k + 1, False, starts[k]))
    return out


def main():
    rows = [r for r in csv.DictReader(l for l in open(ROOT / "data/calendar/gaochang_630.csv", encoding="utf-8") if not l.startswith("#"))]
    main_t = d.tables()["张表"]
    xs = {(m, lp): j for y in (630,) for m, lp, j in pingshuo_year(XUANSHI, y)}
    xs_next = {(m, lp): j for m, lp, j in pingshuo_year(XUANSHI, 631)}
    md = ["# 高昌延寿七年（630）历日与张表、玄始历外推（自动生成）", "",
          "> 由 `scripts/calendar_data/gaochang_630_check.py` 生成，勿手改。录文 `data/calendar/gaochang_630.csv`，依据见脚本说明。", "",
          "残历（86TAM387:38-4）存三月至十二月；朔日由残历各月对齐与两方墓表推得（邓文宽 2002）。「推拟」三朔不作证据，只列。", "",
          "| 月 | 残历朔 | 依据 | 张表朔 | 合张表 | 玄始外推朔 | 合玄始 |", "|---|---|---|---|---|---|---|"]
    n = dict(ev=0, z=0, x=0)
    for r in rows:
        y, m, lp = int(r["年"]), int(r["月"]), r["闰"] == "1"
        zj = main_t.get((y, m, lp))
        xj = (xs if y == 630 else xs_next).get((m, lp))
        zg = GZ[(zj + 49) % 60] if zj is not None else "（无此月）"
        xg = GZ[(xj + 49) % 60] if xj is not None else "（无此月）"
        ev = r["依据"] != "推拟"
        if ev:
            n["ev"] += 1
            n["z"] += zg == r["朔干支"]
            n["x"] += xg == r["朔干支"]
        lab = ("閏" if lp else "") + MON[m - 1] + "月" + ("（631）" if y == 631 else "")
        md.append(f"| {lab} | {r['朔干支']} | {r['依据']} | {zg} | {'✓' if zg == r['朔干支'] else '✗'} | {xg} | {'✓' if xg == r['朔干支'] else '✗'} |")
    md += ["", f"有据之月 {n['ev']}：合张表 {n['z']}，合玄始外推 {n['x']}。", ""]
    (ROOT / "doc/calendar/gaochang-630-checks.md").write_text("\n".join(md), encoding="utf-8")
    print("\n".join(md))


if __name__ == "__main__":
    main()
