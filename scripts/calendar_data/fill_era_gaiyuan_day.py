#!/usr/bin/env python3
"""把逐条查证的改元日期（data/calendar/era_gaiyuan_sources.csv）换算后填入 data/calendar/era_gaiyuan_day.csv。

分工：sync_era_gaiyuan_day.py 从汇总表生成骨架（哪些年有改元），本脚本只负责填日期——两者都是确定性的，
重新生成时先 sync 再 fill。

era_gaiyuan_sources.csv 每行一次改元事件，记录史书原文（quote）与出处（source），日期只写史书给出的
「月 + 日干支」或「月」，由 gaiyuan_date.py 查实历朔日表换算农历日序与公历日期；干支不在该月内时直接报错
（多为史书干支讹误，需在 sources 里改成只记月份并注明，见已有的 105、363 年两条）。

精度：
- 有日干支/日序 -> precision=day，填 lunar_month/lunar_day/gregorian_*；
- 只有月份      -> precision=month，只填 lunar_month；公历留空——若填该月朔日，GaiyuanDays::resolve 会在
  月初就切换年号，是虚假精度，宁可让该年按日解析返回「未定」。

一年多次改元（304、465、494、555、684、692、712 等）：骨架只有「年初年号 -> 年末年号」一行，本脚本按
(年份, 改元后年号, 改元前年号) 匹配；匹配不上而该年恰有一条同名改元后年号的骨架行时更新它（含改元前年号），
否则作为额外事件行追加。周正时期（689-700）史书「正月」「腊月」在夏正上一年 11、12 月，sources 里已按
朔日表的夏正月份记录，额外事件因此可能落在前一夏正年（如 697 年十一月改元圣历）。

最后校验：每年按日期排序后第一个事件的改元前年号，应等于汇总表该年主线行的 gaiyuan_prev_era_name
（即年初年号），不一致时报错——说明汇总表的年初年号需要订正（build_era_names_merged.py 的 MANUAL_PRIMARY_FIXES）。

用法（工作目录为仓库根）：
  uv run --project scripts python scripts/calendar_data/sync_era_gaiyuan_day.py
  uv run --project scripts python scripts/calendar_data/fill_era_gaiyuan_day.py
"""
from __future__ import annotations

import csv
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gaiyuan_date as gd  # noqa: E402
from sync_era_gaiyuan_day import FIELDS, OUT  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
SOURCES = ROOT / "data/calendar/era_gaiyuan_sources.csv"
MERGED = ROOT / "data/calendar/汇总/era_names_merged.csv"


def main() -> None:
    with OUT.open(encoding="utf-8", newline="") as f:
        day_rows = [{k: (r.get(k) or "").strip() for k in FIELDS} for r in csv.DictReader(f)]
    with SOURCES.open(encoding="utf-8", newline="") as f:
        sources = list(csv.DictReader(f))
    primary: dict[int, dict] = {}
    with MERGED.open(encoding="utf-8", newline="") as f:
        for r in csv.DictReader(f):
            if r["is_primary"] == "1":
                primary[int(r["lunar_year"])] = r

    errors: list[str] = []
    touched: list[dict] = []  # 按 sources 顺序记录被填写的行
    events_by_year: dict[int, list[tuple[int, dict]]] = {}
    for s in sources:
        y = int(s["lunar_year"])
        month, leap, day = int(s["month"]), s["leap"] == "1", s["day"].strip()
        try:
            start, end = gd.month_span(y, month, leap)
            res = gd.resolve(y, month, leap, day) if day else None
        except (KeyError, ValueError) as e:
            errors.append(f"{y} {s['to_era']}: {e}")
            continue
        # 日期区间：有日为单日，只记到月为整月 [朔, 晦]
        span = (gd.jdn(*res["gregorian"]),) * 2 if res else (start, end - 1)
        events_by_year.setdefault(y, []).append((span, s))

        # 只在尚未被本次填写过的行里配对（304 年两次改元为「永安」，第二次不能覆盖第一次刚追加的行）
        taken = {id(r) for r in touched}
        stub_candidates = [
            r for r in day_rows
            if r["lunar_year"] == str(y) and r["to_era_name_simp"] == s["to_era"] and id(r) not in taken
        ]
        exact = [r for r in stub_candidates if r["from_era_name_simp"] == s["from_era"]]
        if exact:
            row = exact[0]
        elif len(stub_candidates) == 1:
            row = stub_candidates[0]
        else:
            # 额外事件：政权/帝号取该年骨架行
            base = next((r for r in day_rows if r["lunar_year"] == str(y)), None)
            row = {k: "" for k in FIELDS}
            row["lunar_year"] = str(y)
            row["regime_simp"] = base["regime_simp"] if base else ""
            row["emperor_simp"] = base["emperor_simp"] if base else ""
            row["to_era_name_simp"] = s["to_era"]
            day_rows.append(row)
        touched.append(row)
        row["from_era_name_simp"] = s["from_era"]
        if s["from_year"]:
            row["from_era_year_simp"] = s["from_year"]
        if s["to_year"] or not row["to_era_year_simp"]:
            row["to_era_year_simp"] = s["to_year"] or "元年"
        row["lunar_month"] = str(month)
        row["lunar_is_leap"] = "1" if leap else "0"
        if res:
            gy, gm, gday = res["gregorian"]
            row.update(lunar_day=str(res["lunar_day"]), gregorian_year=str(gy), gregorian_month=str(gm),
                       gregorian_day=str(gday), precision="day")
        else:
            row.update(lunar_day="", gregorian_year="", gregorian_month="", gregorian_day="", precision="month")
        row["source"] = s["source"]
        row["note"] = f"原文：{s['quote']}" + (f"；{s['note']}" if s["note"] else "")

    # 同年多事件以 sources 文件中的先后为准（只记到月的事件无法与同月有日的事件比先后，如 402 年三月壬申复隆安、
    # 其后改元大亨），并校验与可算出的日期不矛盾（不倒序）。
    # 年初年号校验：该年第一个事件的改元前年号 == 汇总表主线行的 gaiyuan_prev_era_name（即年初年号）
    for y, evs in sorted(events_by_year.items()):
        spans = [k for k, _ in evs]
        if any(a[0] > b[1] for a, b in zip(spans, spans[1:])):
            errors.append(f"{y}: sources 中本年事件的先后与日期不一致")
        first = evs[0][1]
        p = primary.get(y)
        prev = p["gaiyuan_prev_era_name"] if p else None
        if p is None or p["has_gaiyuan"] != "1":
            errors.append(f"{y}: 汇总表该年主线未标改元（has_gaiyuan≠1），但 sources 有改元事件")
        elif prev != first["from_era"]:
            errors.append(f"{y}: 汇总表年初年号「{prev}」≠ 最早改元事件的改元前年号「{first['from_era']}」")

    # 骨架行（汇总表每个改元年一行，改元后年号取年级标签）未被任何查证记录匹配时：若该年已有查证事件，说明年级标签
    # 与年末实际年号不同（如新莽建丑正月、武周周正使夏正 13 年年末已是天凤，而年级标签仍记始建国五年），丢弃该骨架行；
    # 该年完全没有查证记录才报错。
    touched_ids = {id(r) for r in touched}
    years_with_events = {str(y) for y in events_by_year}
    kept = []
    for r in day_rows:
        if id(r) in touched_ids:
            kept.append(r)
        elif r["lunar_year"] in years_with_events:
            continue
        else:
            errors.append(f"{r['lunar_year']} {r['to_era_name_simp']}: 骨架行没有查证记录")
    day_rows = kept

    if errors:
        print("\n".join(errors))
        sys.exit(f"{len(errors)} 处问题，未写入")

    # 同年按 sources 中的先后输出：GaiyuanDays::resolve 依表中顺序逐个推进，顺序即时间顺序
    seq = {id(r): i for i, r in enumerate(touched)}
    day_rows.sort(key=lambda r: (int(r["lunar_year"]), seq[id(r)]))
    with OUT.open("w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=FIELDS, lineterminator="\n")
        w.writeheader()
        w.writerows(day_rows)
    n_day = sum(r["precision"] == "day" for r in day_rows)
    print(f"wrote {OUT} rows={len(day_rows)} day={n_day} month={len(day_rows) - n_day}")


if __name__ == "__main__":
    main()
