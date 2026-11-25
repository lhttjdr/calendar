"""两汉简牍（居延汉简、居延新简等）公文书所记「某年某月某干支朔（某干支）」：检验张表与两汉按术推步（三统、四分）。

汉代官文书书首例作「年号 N 年 M 月 干支朔 干支」（如「始建國五年九月丙午朔乙亥」），朔干支是当时所行历日（第 1 级实物）。
释文据《中国简牍集成》（敦煌文艺出版社 2001）各册排印释文；OCR（RapidOCR，竖排）候选由外部步骤生成
（JSON：页序 → [[x, y, 文字, 置信]]），本脚本只做匹配与比对。

候选（candidates）：抽「年号 N 年（閏）M 月 干支朔」，年号按两汉、新莽年号表（data/calendar/汇总/era_names_merged.csv，
政权 汉、新）定年；新莽月名经 month_label_reforms.csv 换算为建寅表月。逐条算张表朔与推步朔（太初后三统、元和二年后四分，
同 han_pingshuo_check.py；太初以前不推）。
核对：OCR 所读朔干支与张表或推步相合者，错读恰合的概率约 1/60，按 OCR 采信（核对 = 「OCR合」）；不合者逐条对原书页图
人工核对（核对 = 「图」），读法与 OCR 不同者记于「图读」。只有简文残缺、年号或数字不能辨者不收。
逐条写入 data/calendar/hanjian_jiri.csv（列：书,册,PDF页,集成编号,年号,年,闰,月,朔干支,原文,核对,图读,备注），
check 生成 doc/calendar/hanjian-checks.md。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/hanjian_shuo_check.py candidates <书> <册> <ocr.json>
            uv run --project scripts python scripts/calendar_data/hanjian_shuo_check.py check
"""
import csv
import json
import re
import sys
from collections import defaultdict
from pathlib import Path

import opencc

sys.path.insert(0, str(Path(__file__).resolve().parent))
import discriminating_records as d  # noqa: E402
import gaiyuan_date as g  # noqa: E402
import han_pingshuo_check as hp  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
T2S = opencc.OpenCC("t2s")
GZ = g.GANZHI if hasattr(g, "GANZHI") else ["甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12] for i in range(60)]
MON = ["正月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "十一月", "十二月"]
CSV = ROOT / "data/calendar/hanjian_jiri.csv"
COLS = ["书", "册", "PDF页", "集成编号", "年号", "年", "闰", "月", "朔干支", "原文", "核对", "图读", "备注"]


def cn(s):
    s = s.replace("廿", "二十").replace("卅", "三十").replace("卌", "四十")
    v = {"元": 1, "正": 1, "一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}
    if s in v:
        return v[s]
    if s == "十":
        return 10
    if "十" in s:
        a, b = s.split("十", 1)
        return (v[a] if a else 1) * 10 + (v[b] if b else 0)
    return None


def eras():
    """年号（简体）→ [(政权, 元年)]：汉、新；同名年号（如东西汉皆有之「建始」等）全列，按候选年份取。"""
    out = defaultdict(set)
    for r in csv.DictReader(open(ROOT / "data/calendar/汇总/era_names_merged.csv", encoding="utf-8")):
        if r["regime"] in ("汉", "新") and r["era_year"] == "1" and -140 <= int(r["lunar_year"]) <= 220:
            out[r["era_name"]].add((r["regime"], int(r["lunar_year"])))
    return {k: sorted(v) for k, v in out.items()}


def reckon_tables(main):
    """三统（前 103—84）、四分（85—220）推步表 (岁, 月, 闰) → 朔 JDN（偏移同 han_pingshuo_check：与张表对齐）"""
    out = {}
    for name, y0, y1 in (("三统", -103, 84), ("四分", 85, 220)):
        _, off = hp.align(name, main, y0, y1)
        for k, v in hp.table(name, y0, y1, off).items():
            if y0 <= k[0] <= y1 or (k[0] == y0 - 1 and k[1] >= 11):
                out[k] = v
    return out


def judge(regime, year, month, leap, gz, main, reck):
    """→ (表月, 张表朔干支, 合张表, 推步朔干支, 合推步)；月名不存在返回 None"""
    t = g.table_month(regime, year, MON[month - 1], leap)
    if t is None:
        return None
    zs = main.get(t)
    rs = reck.get(t)
    zg = GZ[(zs + 49) % 60] if zs is not None else ""
    rg = GZ[(rs + 49) % 60] if rs is not None else ""
    return t, zg, (zg == gz) if zg else None, rg, (rg == gz) if rg else None


def candidates(book, vol, path):
    ev = eras()
    main = d.tables()["张表"]
    reck = reckon_tables(main)
    era_pat = "|".join(sorted({k for k in ev} | {T2S.convert(k) for k in ev}, key=len, reverse=True))
    N = "[元正一二三四五六七八九十廿卅卌]+"
    G = "[甲乙丙丁戊己庚辛壬癸][子丑寅卯辰巳午未申酉戌亥]"
    # 月数可缺：「建武六年閏月甲子朔」只书閏月，取该年有闰之月
    pat = re.compile(rf"({era_pat})({N})年(閏|闰)?({N})?月({G})朔")
    data = json.load(open(path))
    rows, seen = [], set()
    for pg, boxes in sorted(data.items(), key=lambda kv: int(kv[0])):
        # 竖排一列常被 OCR 断成数框：x（框左上角）相差 30 像素以内者视为同列，按 y 接成整列再匹配；单框与整列皆匹配，按（页, 匹配文）去重
        cols = []
        for b in sorted(boxes, key=lambda b: -b[0]):
            if cols and abs(cols[-1][0][0] - b[0]) <= 30:
                cols[-1].append(b)
            else:
                cols.append([b])
        joined = [[c[0][0], min(b[1] for b in c), "".join(b[2] for b in sorted(c, key=lambda b: b[1])), min(b[3] for b in c)] for c in cols if len(c) > 1]
        for x, y, text, conf in boxes + joined:
            s = T2S.convert(text)
            for m in pat.finditer(s):
                if (pg, m.group(0)) in seen:
                    continue
                seen.add((pg, m.group(0)))
                era, yr, leap, gz = m.group(1), cn(m.group(2)), bool(m.group(3)), m.group(5)
                mo = cn(m.group(4)) if m.group(4) else None
                if yr is None or (mo is None and not leap) or (mo is not None and not 1 <= mo <= 12):
                    continue
                for regime, y0 in ev.get(era, []):
                    year = y0 + yr - 1
                    if mo is None:  # 只书閏月：张表该年之闰（无闰则以推步之闰）
                        mo = next((k for k in range(1, 13) if (t := g.table_month(regime, year, MON[k - 1], True)) and (t in main or t in reck)), None)
                        if mo is None:
                            continue
                    j = judge(regime, year, mo, leap, gz, main, reck)
                    rows.append(dict(书=book, 册=vol, PDF页=int(pg) + 1, 年号=era, 年=year, 闰=int(leap), 月=mo, 朔干支=gz,
                                     张表=j[1] if j else "无此月", 合张表=j[2] if j else "", 推步=j[3] if j else "",
                                     合推步=j[4] if j else "", 置信=conf, 原文=text))
    w = csv.DictWriter(sys.stdout, fieldnames=list(rows[0].keys()) if rows else ["无"])
    w.writeheader()
    w.writerows(rows)
    print(f"# 候选 {len(rows)}，合张表 {sum(r['合张表'] is True for r in rows)}，合推步 {sum(r['合推步'] is True for r in rows)}", file=sys.stderr)


def lipu():
    """出土历谱整年月朔（data/calendar/yinwan_lipu.csv）与张表、三统推步逐月比较：朔干支与月大小。"""
    main = d.tables()["张表"]
    reck = reckon_tables(main)
    rows = [r for r in csv.DictReader(l for l in open(ROOT / "data/calendar/yinwan_lipu.csv", encoding="utf-8") if not l.startswith("#"))]

    def size(t, k):
        s = t.get(k)
        nxt = min((v for v in t.values() if s is not None and v > s), default=None)
        return "" if s is None or nxt is None else ("大" if nxt - s == 30 else "小")
    md = ["# 尹湾汉墓历谱与张表、三统推步（自动生成）", "",
          "> 由 `scripts/calendar_data/hanjian_shuo_check.py lipu` 生成，勿手改。录文 `data/calendar/yinwan_lipu.csv`（《尹湾汉墓简牍》释文）。", "",
          "| 件 | 年 | 月 | 历谱朔 | 大小 | 张表朔 | 大小 | 三统推步朔 | 大小 |", "|---|---|---|---|---|---|---|---|---|"]
    n = dict(n=0, z=0, r=0)
    for r in rows:
        k = (int(r["年"]), int(r["月"]), r["闰"] == "1")
        zg = GZ[(main[k] + 49) % 60] if k in main else "（无）"
        rg = GZ[(reck[k] + 49) % 60] if k in reck else "（无）"
        zs, rs = size(main, k), size(reck, k)
        n["n"] += 1
        n["z"] += zg == r["朔干支"] and zs == r["大小"]
        n["r"] += rg == r["朔干支"] and rs == r["大小"]
        lab = ("閏" if k[2] else "") + MON[k[1] - 1]
        md.append(f"| {r['件']} | {k[0]} | {lab} | {r['朔干支']} | {r['大小']} | {zg}{' ✓' if zg == r['朔干支'] else ' ✗'} | {zs}{' ✓' if zs == r['大小'] else ' ✗'} | "
                  f"{rg}{' ✓' if rg == r['朔干支'] else ' ✗'} | {rs}{' ✓' if rs == r['大小'] else ' ✗'} |")
    md += ["", f"共 {n['n']} 月：朔与大小皆合张表 {n['z']}，皆合三统推步 {n['r']}。", ""]
    (ROOT / "doc/calendar/yinwan-lipu-checks.md").write_text("\n".join(md), encoding="utf-8")
    print("\n".join(md))


def check():
    """逐条表（data/calendar/hanjian_jiri.csv）与张表、推步比较，生成 doc/calendar/hanjian-checks.md。"""
    ev = eras()
    main = d.tables()["张表"]
    reck = reckon_tables(main)
    rows = list(csv.DictReader(l for l in open(CSV, encoding="utf-8") if not l.startswith("#")))
    md = ["# 汉简公文书朔日与张表、两汉推步（自动生成）", "",
          "> 由 `scripts/calendar_data/hanjian_shuo_check.py check` 生成，勿手改。逐条表 `data/calendar/hanjian_jiri.csv`，方法见脚本说明与 "
          "[calendar-compendium.md](calendar-compendium.md)「两汉」。", "",
          "| 书 | PDF页 | 简文日期 | 公元 | 简文朔 | 张表朔 | 合张表 | 推步朔 | 合推步 | 核对 | 原文 |", "|---|---|---|---|---|---|---|---|---|---|---|"]
    n = defaultdict(int)
    for r in rows:
        regime, y0 = next(((rg, y0) for rg, y0 in ev.get(r["年号"], []) if y0 <= int(r["年"]) < y0 + 30), ("汉", int(r["年"])))
        j = judge(regime, int(r["年"]), int(r["月"]), r["闰"] == "1", r["朔干支"], main, reck)
        zg, zok, rg_, rok = (j[1], j[2], j[3], j[4]) if j else ("（无此月）", False, "", None)
        n["n"] += 1
        n["z"] += bool(zok)
        n["r"] += bool(rok)
        n["rn"] += rok is not None
        y = int(r["年"])
        yr = f"前 {1 - y}" if y <= 0 else str(y)
        ny = int(r["年"]) - y0 + 1
        lab = f"{r['年号']}{'元' if ny == 1 else ny}年{('閏' if r['闰'] == '1' else '')}{MON[int(r['月']) - 1]}"
        md.append(f"| {r['书']} | {r['PDF页']} | {lab} | {yr} | {r['朔干支']} | {zg} | {'✓' if zok else '✗'} | {rg_} | "
                  f"{'' if rok is None else ('✓' if rok else '✗')} | {r['核对']} | {r['原文'][:40]} |")
    md += ["", f"共 {n['n']} 条：合张表 {n['z']}；推步可比 {n['rn']} 条，合 {n['r']}。", ""]
    (ROOT / "doc/calendar/hanjian-checks.md").write_text("\n".join(md), encoding="utf-8")
    print(md[-2])


if __name__ == "__main__":
    if sys.argv[1] == "candidates":
        candidates(*sys.argv[2:5])
    elif sys.argv[1] == "lipu":
        lipu()
    elif sys.argv[1] == "check":
        check()
