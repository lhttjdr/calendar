"""敦煌出土北魏太平真君十一年、十二年（450、451）历日（data/calendar/敦煌历日.csv，邓文宽 1996 录文）与各历表逐月比对。

- 朔日与闰位：主表、本项目推步（data/calendar/推步政权朔日表.csv）、上游推步（vendor/.../分政权朔日表.csv）；
- 历术：以景初历、玄始历（push_calendar.rs 的 jingchu_params、xuanshi_params）分别推两年朔闰，并以平气推历注节气之日。
历日是当时行用之历的实物，为一手证据（与本纪日期、天象记录同类，而远较其直接）。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/check_dunhuang_calendar.py
"""
import csv
import math
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import discriminating_records as d  # noqa: E402
import gaiyuan_date as g  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
TERMS = "冬至 小寒 大寒 立春 雨水 驚蟄 春分 清明 穀雨 立夏 小滿 芒種 夏至 小暑 大暑 立秋 處暑 白露 秋分 寒露 霜降 立冬 小雪 大雪".split()
NUM = {"一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}
# (lunar, solar, jd_epoch, jdw, dtc_round, dts_round)，同 push_calendar.rs
LI = {"景初": (29 + 2419 / 4559, 365 + 455 / 1843, 1811120.5, 1721050.5 + 220 / 1843, 1e-4, 1e-5),
      "玄始": (29 + 47251 / 89052, 365 + 1759 / 7200, 1871510.5 + 4995 / 89052, 1721048.5 + 1189 / 1200, 1e-6, 1e-6)}


def day_num(s):
    """「廿五」「十一」「卅」「九」→ 日序。"""
    s = s.replace("廿", "二十").replace("卅", "三十")
    if "十" in s:
        a, b = s.split("十")
        return (NUM.get(a, 1) if a else 1) * 10 + (NUM.get(b, 0) if b else 0)
    return NUM[s]


def push_year(p, n):
    """平朔平气、无中气置闰（同 push_calendar.rs::year_new_moons）：(闰在第几月后, 各月朔 JDN)。"""
    lunar, solar, jde, jdw, dtc, dts = p
    jde, jdw = jde + dtc, jdw + dts
    day = lambda jd: math.floor(jd + 0.5)  # noqa: E731
    z1 = jdw + (n + 1 / 6) * solar
    m1 = jde + math.floor((day(z1) + 0.5 - jde) / lunar) * lunar
    has_leap = m1 + 13 * lunar <= day(z1 + solar) + 0.5
    k = 13 if has_leap else 12
    st = [day(m1 + j * lunar) for j in range(k + 1)]
    lm = 0
    if has_leap:
        zq = [day(z1 + q * solar / 12) for q in range(13)]
        lm = next(j for j in range(k) if not any(st[j] <= x < st[j + 1] for x in zq))
    return lm, st[:k]


def main():
    rows = list(csv.DictReader(ln for ln in open(ROOT / "data/calendar/敦煌历日.csv", encoding="utf-8") if not ln.startswith("#")))
    tabs = d.tables()
    print("一、朔日与闰位（逐月）")
    for name in ("主表", "推步:北魏/西魏/北周/隋", "上游:北魏/西魏/北周/隋"):
        ok = sum(g.ganzhi_of_jdn(tabs[name].get((int(r["lunar_year"]), int(r["month"]), r["is_leap"] == "1"), 0)) == r["new_moon_ganzhi"]
                 for r in rows)
        leaps = sorted((k[0], k[1]) for k in tabs[name] if k[2] and k[0] in (450, 451))
        print(f"  {name}：{ok}/{len(rows)} 月合，闰 {leaps}")
    print("二、历术（景初、玄始）：朔闰与平气节气日")
    for li, p in LI.items():
        month_ok = term_ok = term_n = 0
        leaps = []
        for y in (450, 451):
            lm, st = push_year(p, y)
            leaps.append((y, lm))
            labels = [(m, False) for m in range(1, 13)]
            if lm:
                labels.insert(lm, (lm, True))
            by = dict(zip(labels, st))
            solar, jdw = p[1], p[3] + p[5]
            for r in rows:
                if int(r["lunar_year"]) != y:
                    continue
                s = by.get((int(r["month"]), r["is_leap"] == "1"))
                month_ok += s is not None and g.ganzhi_of_jdn(s) == r["new_moon_ganzhi"]
                if s is None:
                    continue
                for dd, term in re.findall(r"([一二三四五六七八九十廿卅]+)日(" + "|".join(TERMS) + ")", r["notes"]):
                    rec = s + day_num(dd) - 1
                    k = TERMS.index(term)
                    pred = min((math.floor(jdw + n * solar + k * solar / 24 + 0.5) for n in (y - 1, y, y + 1)), key=lambda c: abs(c - rec))
                    term_n += 1
                    term_ok += pred == rec
        print(f"  {li}历：朔闰 {month_ok}/{len(rows)} 月合，闰 {leaps}；节气 {term_ok}/{term_n} 日合")


if __name__ == "__main__":
    main()
