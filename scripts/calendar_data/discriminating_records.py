"""能区分历表版本的史书记录：逐条检验各历表下「日干支是否在所记之月」（记「朔」者另检该历朔日干支），列出各历表判断不一者。

史书天象记录（data/phenomena/<书>_star_records.csv、<书>_{solar,lunar}_eclipses.csv）的日期是当时行用之历的一手证据，独立于
各家复原表。参与比较的历表：
- 主表：vendor/calendar/chinese_calendar/实历朔日表.csv（据张培瑜等复原）；
- 本项目推步：data/calendar/推步政权朔日表.csv（calendar::push_calendar 纯推步，各政权）；
- 上游推步：vendor/calendar/chinese_calendar/分政权朔日表.csv（ytliu0 导出，含照张培瑜书的人工修正）。
主表之外只取与记录政权同系的推步表（FAMILY）；所记月名先按月序改制表（gaiyuan_date.table_month）换算为建寅表月，再在各历表中取该月。
只记「闰月」不记月序者，取各历表当年的闰月（闰位不同即可判别）。

输出 data/calendar/discriminating_records.csv（只列判断不一者，含各历下记录之日的 JDN，供
scripts/phenomena/discriminating_records_sky.py 以天象判定哪个日期为真），并在末尾标出三处各家异说之月
（吴 244 年十二月、吴 247 年九月、430 年二月，见 doc/calendar/push-algorithms.md §3.1）有无记录可证。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/discriminating_records.py
"""
import csv
import glob
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gaiyuan_date as g  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
UPSTREAM = {"Shu": "蜀", "Wu": "吴", "LaterQin": "后秦", "NorthernLiang": "北凉", "WeiZhouSui": "北魏/西魏/北周/隋", "WeiQi": "东魏/北齐"}
DISPUTED = [("吴", 244, 12, False), ("吴", 247, 9, False), ("北凉", 430, 2, False), ("北魏/西魏/北周/隋", 430, 2, False)]


def load_table(path, key_col, conv=lambda x: x):
    """历表 CSV → {名: {(农历年, 月, 闰): 朔 JDN}}。"""
    out = defaultdict(dict)
    for r in csv.DictReader(open(path, encoding="utf-8")):
        out[conv(r[key_col])][(int(r["lunar_year"]), int(r["lunar_month"]), r["is_leap"] == "1")] = g.jdn(
            int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"]))
    return out


def tables():
    """所有历表：{历表名: {(年, 月, 闰): 朔 JDN}}（按朔日排序后可求月长）。"""
    t = {}
    main = {}
    for ly, lm, lp, s in g._months():
        main[(ly, lm, lp)] = s
    t["主表"] = main
    for k, v in load_table(ROOT / "data/calendar/推步政权朔日表.csv", "regime").items():
        t[f"推步:{k}"] = v
    for k, v in load_table(ROOT / "vendor/calendar/chinese_calendar/分政权朔日表.csv", "region", lambda x: UPSTREAM.get(x, x)).items():
        t[f"上游:{k}"] = v
    return t


def spans(table):
    """{(年, 月, 闰): (朔, 下月朔)}。"""
    items = sorted(table.items(), key=lambda kv: kv[1])
    return {k: (s, items[i + 1][1]) for i, (k, s) in enumerate(items[:-1])}


def records():
    for path in sorted(glob.glob(str(ROOT / "data/phenomena/*_star_records.csv"))) + \
            sorted(glob.glob(str(ROOT / "data/phenomena/*_eclipses.csv"))):
        if path.endswith("_check.csv"):
            continue
        book = Path(path).name.split("_")[0]
        kind = "凌犯" if "star_records" in path else ("日食" if "solar" in path else "月食")
        for r in csv.DictReader(open(path, encoding="utf-8")):
            if r.get("ganzhi") not in g.GANZHI or not r.get("year"):
                continue
            yield book, kind, r
    # 本纪记事日期（zhi_star_records.py 的 date_sections 输出）
    for path in sorted(glob.glob(str(ROOT / "data/calendar/本纪纪日/*.csv"))):
        book = Path(path).stem
        for r in csv.DictReader(open(path, encoding="utf-8")):
            if r.get("ganzhi") in g.GANZHI and r.get("year"):
                yield book, "纪事", r


# 政权 → 推步/上游表中的政权名（同一系）；主表外只取同系历表，不拿吴历判刘宋之类
FAMILY = {"北魏": "北魏/西魏/北周/隋", "西魏": "北魏/西魏/北周/隋", "北周": "北魏/西魏/北周/隋", "隋": "北魏/西魏/北周/隋",
          "东魏": "东魏/北齐", "北齐": "东魏/北齐", "吴": "吴", "蜀": "蜀", "北凉": "北凉", "后秦": "后秦", "金": "金"}


def main():
    tabs = {k: spans(v) for k, v in tables().items()}
    out, seen = [], set()
    disputed_hits = defaultdict(list)
    for book, kind, r in records():
        year, label, regime = int(r["year"]), r["month"], r.get("regime", "")
        fam = FAMILY.get(regime)
        if not fam:
            continue
        leap = label.startswith("闰")
        num = label.lstrip("闰")
        if not num.isdigit():
            continue
        key_text = (book, r["text"][:40], r["ganzhi"], year, label)
        if key_text in seen:
            continue
        seen.add(key_text)
        t = g.table_month(regime, year, g.MONTH_NAMES[int(num) - 1], leap) or (year, int(num), leap)
        verdicts = {}
        for name in ("主表", f"推步:{fam}", f"上游:{fam}"):
            sp = tabs.get(name, {})
            if t not in sp:
                continue
            s0, e0 = sp[t]
            off = (g.GANZHI.index(r["ganzhi"]) - (s0 + 49) % 60) % 60
            ok = s0 + off < e0
            v = f"{'在月' if ok else '不在月'}（朔{g.ganzhi_of_jdn(s0)}）"
            if kind != "凌犯" and r.get("position") == "朔":
                v += "朔合" if off == 0 else "朔不合"
            # 记录之日：在月取当月该干支日；不在月取离该月最近的同名干支日（落在邻月），供天象比较两历的候选日期
            jd = s0 + off if ok else min((s0 + off - 60, s0 + off), key=lambda x: abs(x - (s0 + e0) // 2))
            verdicts[name.split(":")[0]] = (v, jd)
            for reg, y, m, lp in DISPUTED:
                if fam == reg and t == (y, m, lp):
                    disputed_hits[(reg, y, m)].append((book, kind, r["text"][:40], name, v))
        if len(verdicts) >= 2 and len({v[0].split("（")[0] for v in verdicts.values()}) > 1:
            out.append({"书": book, "类": kind, "政权": regime, "年": year, "月": label, "日干支": r["ganzhi"],
                        "主表": verdicts.get("主表", ("", ""))[0], "本项目推步": verdicts.get("推步", ("", ""))[0],
                        "上游推步": verdicts.get("上游", ("", ""))[0],
                        "主表日 JDN": verdicts.get("主表", ("", ""))[1], "推步日 JDN": verdicts.get("推步", ("", ""))[1],
                        "bodies": r.get("bodies", ""), "target_id": r.get("target_id", ""), "原文": r["text"][:50]})
    # 各政权纪日/天象记录在各历表下的在月率（同系比较）
    stat = defaultdict(lambda: [0, 0])
    for book, kind, r in records():
        fam = FAMILY.get(r.get("regime", ""))
        label = r["month"]
        if not fam or not label.lstrip("闰").isdigit():
            continue
        y, leap, num = int(r["year"]), label.startswith("闰"), int(label.lstrip("闰"))
        t = g.table_month(r["regime"], y, g.MONTH_NAMES[num - 1], leap) or (y, num, leap)
        names = ("主表", f"推步:{fam}", f"上游:{fam}")
        if not all(t in tabs.get(n, {}) for n in names):
            continue
        for n in names:
            s0, e0 = tabs[n][t]
            off = (g.GANZHI.index(r["ganzhi"]) - (s0 + 49) % 60) % 60
            stat[(fam, n.split(":")[0])][0] += 1
            stat[(fam, n.split(":")[0])][1] += s0 + off < e0
    for (fam, n), (tot, ok) in sorted(stat.items()):
        print(f"{fam} {n}: {ok}/{tot} 在月（{100 * ok / tot:.1f}%）")
    dst = ROOT / "data/calendar/discriminating_records.csv"
    with open(dst, "w", encoding="utf-8", newline="") as f:
        f.write("# 史书天象记录中主表与同系推步表判断不一者（scripts/calendar_data/discriminating_records.py）：「在月」指日干支落在所记之月；")
        f.write("日 JDN 为该历下记录之日（不在月者取离该月最近的同名干支日）；天象判定见 data/calendar/discriminating_records_sky.csv。\n")
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    print(dst, len(out), "条判断不一")
    for reg, y, m, _ in DISPUTED:
        print(f"异说之月 {reg} {y} 年 {m} 月：", disputed_hits.get((reg, y, m)) or "无记录")


def leap_evidence():
    """本纪「閏月」所接之月（zhi_star_records.py 纪日模式输出的 *_閏月.csv）与各历表当年闰位逐条比对。"""
    tabs = tables()
    leaps = {name: {k[0]: k[1] for k in tab if k[2]} for name, tab in tabs.items()}
    out = []
    for path in sorted(glob.glob(str(ROOT / "data/calendar/本纪纪日/*_閏月.csv"))):
        book = Path(path).stem.replace("_閏月", "")
        for r in csv.DictReader(open(path, encoding="utf-8")):
            y, after, regime = int(r["year"]), int(r["leap_after"]), r["regime"]
            fam = FAMILY.get(regime)
            row = {"书": book, "政权": regime, "年": y, "纪年": f"{r['era']}{r['year_in_era']}年", "本纪闰位": f"闰{after}"}
            for col, name in (("主表", "主表"), ("本项目推步", f"推步:{fam}"), ("上游推步", f"上游:{fam}")):
                if name not in leaps:
                    row[col] = ""
                    continue
                lp = leaps[name].get(y)
                covered = any(k[0] == y for k in tabs[name])
                row[col] = "" if not covered else (f"闰{lp}" + ("（合）" if lp == after else "（不合）") if lp else "无闰（不合）")
            row["原文"] = r["text"][:40]
            out.append(row)
    dst = ROOT / "data/calendar/leap_evidence.csv"
    with open(dst, "w", encoding="utf-8", newline="") as f:
        f.write("# 本纪「閏月」所接之月与各历表闰位（scripts/calendar_data/discriminating_records.py）；空格为该表当年无数据。\n")
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    summ = defaultdict(lambda: [0, 0])
    for o in out:
        for col in ("主表", "本项目推步", "上游推步"):
            if o[col]:
                summ[(o["政权"], col)][0] += 1
                summ[(o["政权"], col)][1] += "（合）" in o[col]
    print(dst, len(out), "条")
    for (reg, col), (n, ok) in sorted(summ.items()):
        print(f"  {reg} {col}：{ok}/{n} 合")


if __name__ == "__main__":
    main()
    leap_evidence()
