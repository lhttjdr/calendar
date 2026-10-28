"""出土与石刻纪日（data/calendar/出土纪日.csv）对各历表的核对。

纪日记有日序与日干支（或朔干支）者，可直接检验当时行用之历的朔日，是独立于各复原表（实历朔日表、张培瑜书、汪曰桢书、
本项目推步）的一手证据。对每条记录，列出各历表推得的日干支、朔干支是否与铭文相合。

历表：
- 主表：vendor 实历朔日表；
- 本实现：data/calendar/推步政权朔日表.csv（calendar::push_calendar 纯推步）；
- 上游：vendor/calendar/chinese_calendar/分政权朔日表.csv（ytliu0 推步，含照张培瑜书的人工修正）。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/check_dated_inscriptions.py
"""
import csv
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gaiyuan_date as g  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
UPSTREAM_REGION = {"蜀": "Shu", "吴": "Wu", "后秦": "LaterQin", "北凉": "NorthernLiang", "北魏": "WeiZhouSui", "东魏": "WeiQi"}
OURS_REGIME = {"北魏": "北魏/西魏/北周/隋", "东魏": "东魏/北齐"}


def load(path, col, value):
    out = {}
    for r in csv.DictReader(open(path, encoding="utf-8")):
        if r[col] == value:
            out[(int(r["lunar_year"]), int(r["lunar_month"]), r["is_leap"] == "1")] = g.jdn(
                int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"]))
    return out


def main():
    rows = list(csv.DictReader(open(ROOT / "data/calendar/出土纪日.csv", encoding="utf-8")))
    bad = 0
    for r in rows:
        key = (int(r["lunar_year"]), int(r["month"]), r["is_leap"] == "1")
        day = int(r["day"])
        cands = {}
        try:
            cands["主表"] = g.month_span(*key)[0]
        except KeyError:
            pass
        reg = r["regime"]
        if reg in UPSTREAM_REGION:
            ours = load(ROOT / "data/calendar/推步政权朔日表.csv", "regime", OURS_REGIME.get(reg, reg))
            up = load(ROOT / "vendor/calendar/chinese_calendar/分政权朔日表.csv", "region", UPSTREAM_REGION[reg])
            if key in ours:
                cands["本实现"] = ours[key]
            if key in up:
                cands["上游"] = up[key]
        cells = []
        for name, s in cands.items():
            ok = g.ganzhi_of_jdn(s + day - 1) == r["day_ganzhi"] and (not r["shuo_ganzhi"] or g.ganzhi_of_jdn(s) == r["shuo_ganzhi"])
            bad += (name == r["table"] or name == "本实现") and not ok
            cells.append(f"{name} 朔{g.ganzhi_of_jdn(s)}、{day}日{g.ganzhi_of_jdn(s + day - 1)} {'合' if ok else '不合'}")
        print(f"{r['regime']}「{r['text']}」（{r['object']}）：" + "；".join(cells))
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
