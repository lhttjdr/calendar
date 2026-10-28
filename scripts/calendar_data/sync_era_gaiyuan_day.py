#!/usr/bin/env python3
"""从汇总表 era_names_merged.csv 的主线改元年生成/补齐 era_gaiyuan_day.csv 骨架。

数据源是 data/calendar/汇总/era_names_merged.csv（is_primary=1 且 has_gaiyuan=1 的行），而不是
vendor/年号表.csv 原文件：Rust 侧 GaiyuanDays::resolve 只在 EraNames（同样加载汇总表）的条目
has_gaiyuan=1 时才查本表，两者必须同源，否则汇总表里订正过的改元（如 697 年神功）在本表里没有骨架行。
年数列用中文数字文本（与 rust/core/src/calendar/era_names.rs 的 year_to_cn 同一格式）。

已填的改元日、来源、备注会保留；只追加尚未出现的改元年行。
日期列留空，由 fill_era_gaiyuan_day.py 据 era_gaiyuan_sources.csv（逐条查证的史料记录）填入。
"""

from __future__ import annotations

import csv
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
ERA_NAMES = ROOT / "data" / "calendar" / "汇总" / "era_names_merged.csv"
OUT = ROOT / "data" / "calendar" / "era_gaiyuan_day.csv"

FIELDS = [
    "lunar_year",
    "regime_simp",
    "emperor_simp",
    "from_era_name_simp",
    "from_era_year_simp",
    "to_era_name_simp",
    "to_era_year_simp",
    "lunar_month",
    "lunar_is_leap",
    "lunar_day",
    "gregorian_year",
    "gregorian_month",
    "gregorian_day",
    "precision",
    "source",
    "note",
]

DATE_FIELDS = {
    "lunar_month",
    "lunar_is_leap",
    "lunar_day",
    "gregorian_year",
    "gregorian_month",
    "gregorian_day",
    "precision",
    "source",
    "note",
}


CN_DIGITS = "零一二三四五六七八九"


def year_to_cn(s: str) -> str:
    """整数年数文本 -> 中文数字年份，与 rust/core/src/calendar/era_names.rs::year_to_cn 一致。"""
    s = s.strip()
    if not s:
        return ""
    n = int(s)
    if n == 1:
        return "元年"
    if n == 0 or n > 99:
        return f"{n}年"
    if n < 10:
        return f"{CN_DIGITS[n]}年"
    tens, ones = divmod(n, 10)
    return f"{'' if tens == 1 else CN_DIGITS[tens]}十{CN_DIGITS[ones] if ones else ''}年"


def row_key(row: dict[str, str]) -> tuple[str, str]:
    """骨架行按 (年份, 改元后年号) 配对：一年多次改元时，查证后的骨架行改元前年号会被 fill_era_gaiyuan_day.py
    改成实际的上一个年号（如 465 年「景和→泰始」而非年初的「永光」），不能再拿改元前年号配对。"""
    return (row["lunar_year"].strip(), row["to_era_name_simp"].strip())


def has_filled_data(row: dict[str, str]) -> bool:
    return any(row.get(k) for k in DATE_FIELDS if k != "precision") or row.get("precision") not in ("", "unknown")


def main() -> None:
    existing: dict[tuple[str, str], dict[str, str]] = {}
    extras: list[dict[str, str]] = []
    if OUT.exists():
        with OUT.open(newline="", encoding="utf-8") as f:
            reader = csv.DictReader(f)
            for raw in reader:
                row = {k: (raw.get(k) or "").strip() for k in FIELDS}
                if not row["lunar_year"]:
                    continue
                existing[row_key(row)] = row
                extras.append(row)

    stubs: list[dict[str, str]] = []
    with ERA_NAMES.open(newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for r in reader:
            if (r.get("is_primary") or "").strip() != "1" or (r.get("has_gaiyuan") or "").strip() != "1":
                continue
            stub = {k: "" for k in FIELDS}
            stub["lunar_year"] = (r.get("lunar_year") or "").strip()
            stub["regime_simp"] = (r.get("regime") or "").strip()
            stub["emperor_simp"] = (r.get("emperor") or "").strip()
            stub["from_era_name_simp"] = (r.get("gaiyuan_prev_era_name") or "").strip()
            stub["from_era_year_simp"] = year_to_cn(r.get("gaiyuan_prev_era_year") or "")
            stub["to_era_name_simp"] = (r.get("era_name") or "").strip()
            stub["to_era_year_simp"] = year_to_cn(r.get("era_year") or "")
            stub["precision"] = "unknown"
            stubs.append(stub)

    seen_from_era = {row_key(s) for s in stubs}
    merged: list[dict[str, str]] = []
    for stub in stubs:
        key = row_key(stub)
        if key in existing:
            old = existing[key]
            for field in DATE_FIELDS:
                if old.get(field):
                    stub[field] = old[field]
            if not stub["precision"]:
                stub["precision"] = "unknown"
        merged.append(stub)

    for extra in extras:
        if row_key(extra) not in seen_from_era and has_filled_data(extra):
            # 同年多次改元：年表只有一对前后号，额外事件行予以保留。
            # 没填任何日期/来源的旧骨架行（如汇总表订正年号名后键变了）不含信息，丢弃，避免重复骨架。
            merged.append(extra)

    merged.sort(key=lambda r: int(r["lunar_year"]))

    with OUT.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=FIELDS, lineterminator="\n")
        writer.writeheader()
        writer.writerows(merged)

    filled = sum(
        1
        for r in merged
        if r.get("precision") in ("day", "month")
        and (r.get("gregorian_day") or r.get("lunar_day"))
    )
    print(f"wrote {OUT} rows={len(merged)} dated={filled} unknown={len(merged) - filled}")


if __name__ == "__main__":
    main()
