"""历史月序改制：把实历朔日表（建寅月序）的月份映射为当时政权实际使用的月名，写出 data/calendar/month_label_reforms.csv。

vendor/calendar/chinese_calendar/实历朔日表.csv 的 lunar_year/lunar_month 一律按建寅（夏正）编号。以下四段，
当时政权所用的岁首、月名与之不同（史料原文见 doc/calendar/historical-calendar.md「月序改制」一节）：

- 新莽（始建国元年—地皇四年，表 8 年十二月—23 年十一月）建丑：「以十二月朔癸酉為建國元年正月之朔」
  （《汉书·王莽传上》）。月名 = 表月 + 1，表十二月为次年正月。
- 魏景初（表 237 年三月—239 年十二月）建丑：「三月，定曆改年為孟夏四月」（《三国志·明帝纪》）；景初元年无三月；
  「其以建寅之月為正始元年正月，以建丑月為後十二月」（《三国志·齐王芳纪》），景初三年末月称「后十二月」。
- 武周（表 689 年十一月—700 年十月）建子：「依周制建子月為正月，改永昌元年十一月為載初元年正月，十二月為臘月，
  改舊正月為一月」（《旧唐书·则天皇后纪》）；「（久视元年）冬十月甲寅，復舊正朔，改一月為正月，仍以為歲首，
  正月依舊為十一月」。表十一月 → 次年「正月」，十二月 → 次年「腊月」，正月 → 「一月」，余月同名。
- 唐肃宗—代宗（表 761 年十一月—762 年四月）建子、以斗建为月名：「去上元號，稱元年，以十一月為歲首，月以斗所建辰為名」
  （《新唐书·肃宗纪》）；「改元年為寶應元年，復以正月為歲首，建巳月為四月」（《新唐书·代宗纪》）。
  表十一月 → 次年（元年 = 宝应元年）「建子月」，依次建丑、建寅、建卯、建辰、建巳月。

只列与建寅标准月名不同的月份（年份或月名或闰否有别）；其余月份按表的建寅月名。每行注明政权，因同一年可有并存政权
用不同月序（如地皇四年新莽建丑、更始政权建寅；景初年间吴、蜀仍建寅）。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/build_month_label_reforms.py
"""
import csv
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TABLE = ROOT / "vendor/calendar/chinese_calendar/实历朔日表.csv"
OUT = ROOT / "data/calendar/month_label_reforms.csv"

CN = ["", "正月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "十一月", "十二月"]
DOU = {11: "建子月", 12: "建丑月", 1: "建寅月", 2: "建卯月", 3: "建辰月", 4: "建巳月"}

# (政权, 时期名, 首月, 末月（含）, 规则)；月以 (表年, 表月) 表示，闰月随其前月一并处理
PERIODS = [
    ("新", "新莽建丑", (8, 12), (23, 11), "jianchou"),
    ("魏", "魏景初建丑", (237, 3), (239, 12), "jingchu"),
    ("武周", "武周建子", (689, 11), (700, 10), "wuzhou"),
    ("唐", "唐肃宗建子斗建月名", (761, 11), (762, 4), "suzong"),
]


def months():
    rows = list(csv.DictReader(open(TABLE, encoding="utf-8")))
    out = [(int(r["lunar_year"]), int(r["lunar_month"]), r["is_leap"] == "1") for r in rows]
    return sorted(out, key=lambda t: (t[0], t[1], t[2]))


def label(rule, y, m, leap):
    """(名义岁, 月名, 闰否)。"""
    if rule == "jianchou":
        return (y + 1, "正月", leap) if m == 12 else (y, CN[m + 1], leap)
    if rule == "jingchu":
        if (y, m) == (239, 12):
            return (239, "后十二月", False)
        return (y + 1, "正月", leap) if m == 12 else (y, CN[m + 1], leap)
    if rule == "wuzhou":
        return {11: (y + 1, "正月", leap), 12: (y + 1, "腊月", leap), 1: (y, "一月", leap)}.get(m, (y, CN[m], leap))
    if rule == "suzong":
        return (y + 1 if m >= 11 else y, DOU[m], leap)
    raise ValueError(rule)


def main():
    out = []
    for regime, period, first, last, rule in PERIODS:
        for y, m, leap in months():
            if not first <= (y, m) <= last:
                continue
            ly, name, lleap = label(rule, y, m, leap)
            if (ly, name, lleap) == (y, CN[m], leap):
                continue
            out.append(dict(regime=regime, period=period, table_lunar_year=y, table_lunar_month=m, table_is_leap=int(leap),
                            label_year=ly, label_month=name, label_is_leap=int(lleap)))
    with open(OUT, "w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    print("写出", OUT, len(out), "行")


if __name__ == "__main__":
    main()
