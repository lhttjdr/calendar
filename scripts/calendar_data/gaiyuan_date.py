#!/usr/bin/env python3
"""改元日期换算工具：农历「某年某月某干支日」/「某月某日」-> 农历日序 + 公历日期。

史书本纪记改元多作「九月壬午，改元天授」，只给月份和日干支。本模块用 vendor 实历朔日表（与 rust/core
HistoricalCalendar 同源）查该月朔日，按儒略日推出日干支序号，换算成农历日序与公历日期，避免手算出错。
公历：1582-10-15 之前为儒略历、之后为格里高利历（与朔日表、era_gaiyuan_day.csv 约定一致）。

用法（工作目录为仓库根）：
  uv run --project scripts python scripts/calendar_data/gaiyuan_date.py 690 9 0 壬午
  uv run --project scripts python scripts/calendar_data/gaiyuan_date.py 690 9 0 9      # 直接给日序
"""
from __future__ import annotations

import csv
import sys
from functools import lru_cache
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SHUORI = ROOT / "vendor/calendar/chinese_calendar/实历朔日表.csv"

GAN = "甲乙丙丁戊己庚辛壬癸"
ZHI = "子丑寅卯辰巳午未申酉戌亥"
GANZHI = [GAN[i % 10] + ZHI[i % 12] for i in range(60)]


def jdn(y: int, m: int, d: int) -> int:
    """公历日期 -> 儒略日数（整数）。1582-10-15 之前按儒略历，之后按格里高利历；年份为天文纪年。"""
    a = (14 - m) // 12
    yy = y + 4800 - a
    mm = m + 12 * a - 3
    if (y, m, d) >= (1582, 10, 15):
        return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - yy // 100 + yy // 400 - 32045
    return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - 32083


def from_jdn(n: int) -> tuple[int, int, int]:
    if n >= 2299161:  # 1582-10-15
        a = n + 32044
        b = (4 * a + 3) // 146097
        c = a - 146097 * b // 4
    else:
        b = 0
        c = n + 32082
    d = (4 * c + 3) // 1461
    e = c - 1461 * d // 4
    m = (5 * e + 2) // 153
    day = e - (153 * m + 2) // 5 + 1
    month = m + 3 - 12 * (m // 10)
    year = 100 * b + d - 4800 + m // 10
    return year, month, day


def ganzhi_of_jdn(n: int) -> str:
    return GANZHI[(n + 49) % 60]  # 2000-01-01（JDN 2451545）为戊午


@lru_cache(maxsize=1)
def _months() -> list[tuple[int, int, bool, int]]:
    """按时间排序的 (lunar_year, lunar_month, is_leap, 朔日 JDN)。"""
    out = []
    with open(SHUORI, encoding="utf-8") as f:
        for r in csv.DictReader(f):
            n = jdn(int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"]))
            out.append((int(r["lunar_year"]), int(r["lunar_month"]), r["is_leap"] == "1", n))
    out.sort(key=lambda t: t[3])
    return out


def month_span(lunar_year: int, month: int, leap: bool) -> tuple[int, int]:
    """该农历月的 [朔日 JDN, 下月朔日 JDN)。"""
    ms = _months()
    for i, (ly, lm, lp, n) in enumerate(ms):
        if (ly, lm, lp) == (lunar_year, month, leap):
            return n, ms[i + 1][3]
    raise KeyError(f"朔日表无 {lunar_year} 年{'闰' if leap else ''}{month}月")


REGIME_TABLE = ROOT / "data/calendar/推步政权朔日表.csv"


@lru_cache(maxsize=None)
def _regime_months(regime: str) -> list[tuple[int, int, bool, int]]:
    """推步政权（蜀、吴、后秦、北凉、北魏/西魏/北周/隋、东魏/北齐）按时间排序的 (岁, 月, 闰, 朔日 JDN)。
    数据由 rust/core examples/export_regime_calendars.rs 从 calendar::push_calendar 导出。"""
    out = []
    with open(REGIME_TABLE, encoding="utf-8") as f:
        for r in csv.DictReader(f):
            if r["regime"] == regime:
                n = jdn(int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"]))
                out.append((int(r["lunar_year"]), int(r["lunar_month"]), r["is_leap"] == "1", n))
    if not out:
        raise KeyError(f"推步政权朔日表无政权 {regime}")
    out.sort(key=lambda t: t[3])
    return out


def regime_month_span(regime: str, lunar_year: int, month: int, leap: bool) -> tuple[int, int]:
    """推步政权某月的 [朔日 JDN, 下月朔日 JDN)。"""
    ms = _regime_months(regime)
    for i, (ly, lm, lp, n) in enumerate(ms):
        if (ly, lm, lp) == (lunar_year, month, leap) and i + 1 < len(ms):
            return n, ms[i + 1][3]
    raise KeyError(f"{regime} 无 {lunar_year} 年{'闰' if leap else ''}{month}月")


def calendar_month_span(table: str, lunar_year: int, month: int, leap: bool) -> tuple[int, int]:
    """table 为「主表」（实历朔日表）或推步政权名。"""
    return month_span(lunar_year, month, leap) if table == "主表" else regime_month_span(table, lunar_year, month, leap)


REFORMS = ROOT / "data/calendar/month_label_reforms.csv"
MONTH_NAMES = ["正月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "十一月", "十二月"]


@lru_cache(maxsize=1)
def _reforms() -> tuple[dict, set]:
    """月序改制表（与 rust/core calendar::month_labels 同源）：(政权, 名义岁, 月名, 闰) → (表岁, 表月, 闰)；及被改名的表月集合。"""
    by_label, relabeled = {}, set()
    with open(REFORMS, encoding="utf-8") as f:
        for r in csv.DictReader(f):
            t = (int(r["table_lunar_year"]), int(r["table_lunar_month"]), r["table_is_leap"] == "1")
            by_label[(r["regime"], int(r["label_year"]), r["label_month"], r["label_is_leap"] == "1")] = t
            relabeled.add((r["regime"],) + t)
    return by_label, relabeled


def table_month(regime: str, label_year: int, month_name: str, leap: bool) -> tuple[int, int, bool] | None:
    """当时政权的月名（「正月」「腊月」「一月」「后十二月」「建子月」……）→ 实历朔日表的建寅月 (岁, 月, 闰)。
    不在改制期按建寅月名；当时并无此月名（如景初元年三月、武周十一月）返回 None。"""
    by_label, relabeled = _reforms()
    t = by_label.get((regime, label_year, month_name, leap))
    if t is not None:
        return t
    if month_name not in MONTH_NAMES:
        return None
    t = (label_year, MONTH_NAMES.index(month_name) + 1, leap)
    return None if (regime,) + t in relabeled else t


def resolve(lunar_year: int, month: int, leap: bool, day: str | int) -> dict:
    """day 为干支（如「壬午」）或日序（1-30）。干支不在该月内时抛 ValueError（多为史书月份/干支有误，需人工判断）。"""
    start, end = month_span(lunar_year, month, leap)
    if isinstance(day, str) and not day.isdigit():
        target = GANZHI.index(day)
        off = (target - (start + 49) % 60) % 60
        if start + off >= end:
            raise ValueError(
                f"{lunar_year} 年{'闰' if leap else ''}{month}月（朔 {ganzhi_of_jdn(start)}，{end - start} 日）无 {day} 日"
            )
    else:
        off = int(day) - 1
        if not 0 <= off < end - start:
            raise ValueError(f"{lunar_year} 年{'闰' if leap else ''}{month}月只有 {end - start} 日")
    n = start + off
    return {
        "lunar_day": off + 1,
        "ganzhi": ganzhi_of_jdn(n),
        "gregorian": from_jdn(n),
        "month_len": end - start,
        "shuo_ganzhi": ganzhi_of_jdn(start),
    }


if __name__ == "__main__":
    y, m, lp, d = int(sys.argv[1]), int(sys.argv[2]), sys.argv[3] == "1", sys.argv[4]
    print(resolve(y, m, lp, d))
