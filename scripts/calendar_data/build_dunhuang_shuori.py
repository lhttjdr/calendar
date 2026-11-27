"""由邓文宽附录一比较表（data/calendar/敦煌历日_比较表.csv）生成「敦煌」政权朔日表 data/calendar/敦煌朔日表.csv。

只收敦煌曆一栏有朔干支之月（原件可读之格，与原书括号内「據殘曆本身提供的條件推算而得」之格，分记依据），
吐蕃（808—834）、归义军（858—993）两期；450、451 年北魏历日是北魏行用之历，不入此表（见 push-algorithms.md 3.5）。
敦煌朔只有干支，日期取比较表同行中原曆朔（与张表逐格全合）在 ±3 日内同干支之日；月序、岁取敦煌曆自身
（敦煌月序一栏有字时用之，否则同行中原月序；926 年「上年閏十二月」行为敦煌 926 年正月；「表列次年正月」行记于次年）。
同一月见于两件者（次年正月）须朔、大小不矛盾，合为一行、文书并记。

另并入《俄藏敦煌文獻》补录 data/calendar/dunhuang_ezang.csv 有朔之月（比较表本身是邓书逐格转录，不改）：
只收定年独立于朔者（定年方式 = 缀合等）；「据朔推定」之件（如 Дх.01411）选年用到了它自己的朔，
年份不独立，若并入会把循环定年的一月送进换算，故不收，仍只留在补录表与 dunhuang_ezang_check.csv；
只书闰月无朔者（Дх.01410）无朔可并。月大小推得之朔记「推算」，历首月大小表记「原件」大小。

前 8 列与 data/calendar/推步政权朔日表.csv 同式，供 rust/core/src/calendar/dunhuang.rs 读入。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/build_dunhuang_shuori.py
"""
import csv
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gaiyuan_date as g  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
N = {"正": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9, "十": 10, "十一": 11, "十二": 12}


def rows():
    with open(ROOT / "data/calendar/敦煌历日_比较表.csv", encoding="utf-8") as f:
        return list(csv.DictReader(line for line in f if not line.startswith("#")))


def label(lab):
    m = re.fullmatch(r"(上年)?(閏)?(.+)月", lab)
    return bool(m[1]), N[m[3]], bool(m[2])


def ezang_rows():
    with open(ROOT / "data/calendar/dunhuang_ezang.csv", encoding="utf-8") as f:
        return list(csv.DictReader(line for line in f if not line.startswith("#")))


# 补录件历首月大小表所记各月大小（原件），键 (年, 月)；朔由同件推得
EZANG_SIZE = {(945, 8): 29, (945, 9): 30, (945, 10): 30, (945, 11): 30, (945, 12): 29}
EZANG_ID = {"具注曆日": ("Дх.01454+02418V", "後晉天福十年乙巳歲")}


def merge_ezang(out):
    for r in ezang_rows():
        v = r["朔干支"].strip()
        if v not in g.GANZHI or r["定年方式"] in ("据朔推定", "据闰推定"):
            continue
        y, m, lp = int(r["公元"]), int(r["月"]), r["闰"] == "1"
        doc_id, title = next(val for k, val in EZANG_ID.items() if r["件"].startswith(k))
        z = {(ly, lm, lpp): s for ly, lm, lpp, s in g._months()}[(y, m, lp)]
        cands = [z + k for k in range(-3, 4) if g.ganzhi_of_jdn(z + k) == v]
        assert len(cands) == 1, (r, cands)
        date = g.from_jdn(cands[0])
        size = EZANG_SIZE.get((y, m), "")
        page = "俄敦" + r["俄敦册页"]
        basis = "原件" if r["依据"] == "原件" else "推算"
        key = (y, m, lp)
        if key in out:
            old = out[key]
            assert (old["gregorian_year"], old["gregorian_month"], old["gregorian_day"]) == date, (old, r)
            if old["月日数"] == "" and size != "":
                old["月日数"], old["大小依据"] = size, "原件"
            assert size in ("", old["月日数"]), (old, r)
            if basis == "原件":
                old["朔依据"] = "原件"
            if doc_id not in old["文书"].split("；"):
                old["文书"] += "；" + doc_id
                old["文书纪年"] += "；" + title
                old["文书岁"] = f'{old["文书岁"]}；{y}'
                old["书页"] += "；" + page
            continue
        out[key] = {
            "regime": "敦煌", "calendar": "吐蕃时期敦煌历" if y < 848 else "归义军时期敦煌历",
            "gregorian_year": date[0], "gregorian_month": date[1], "gregorian_day": date[2],
            "lunar_year": y, "lunar_month": m, "is_leap": int(lp),
            "朔干支": v, "朔依据": basis, "月日数": size, "大小依据": "原件" if size != "" else "",
            "文书": doc_id, "文书纪年": title, "文书岁": y, "书页": page,
        }


def main():
    main_t = {(ly, lm, lp): s for ly, lm, lp, s in g._months()}
    out = {}
    for r in rows():
        y = int(r["公元"])
        if y < 800:
            continue
        cell = r["敦煌朔"].strip()
        v = cell.strip("()")
        if v not in g.GANZHI:
            continue
        prev, m, lp = label(r["行月序"])
        z = main_t[(y - 1 if prev else y, m, lp)]
        cands = [z + k for k in range(-3, 4) if g.ganzhi_of_jdn(z + k) == v]
        assert len(cands) == 1, (r, cands)
        jdn = cands[0]
        if r["敦煌月序"]:
            _, dm, dlp = label(r["敦煌月序"])
        else:
            dm, dlp = m, lp
            assert not prev
        size_cell = r["敦煌大小"].strip()
        size = size_cell.strip("()")
        size = {"大": 30, "小": 29}.get(size, "")
        size_basis = "" if size == "" else ("推算" if size_cell.startswith("(") else "原件")
        key = (y, dm, dlp)
        rec = {
            "regime": "敦煌",
            "calendar": "吐蕃时期敦煌历" if y < 848 else "归义军时期敦煌历",
            "gregorian_year": 0, "gregorian_month": 0, "gregorian_day": 0,
            "lunar_year": y, "lunar_month": dm, "is_leap": int(dlp),
            "朔干支": v, "朔依据": "推算" if cell.startswith("(") else "原件",
            "月日数": size, "大小依据": size_basis,
            "文书": r["文书"], "文书纪年": r["纪年"], "文书岁": y - 1 if "表列次年正月" in r["备注"] else y, "书页": r["书页"],
        }
        rec["gregorian_year"], rec["gregorian_month"], rec["gregorian_day"] = g.from_jdn(jdn)
        if key in out:
            old = out[key]
            assert (old["gregorian_year"], old["gregorian_month"], old["gregorian_day"]) == g.from_jdn(jdn), (old, rec)
            if old["月日数"] == "":
                old["月日数"], old["大小依据"] = rec["月日数"], rec["大小依据"]
            assert rec["月日数"] in ("", old["月日数"]), (old, rec)
            if rec["朔依据"] == "原件":
                old["朔依据"] = "原件"
            if rec["文书"] not in old["文书"].split("；"):
                old["文书"] += "；" + rec["文书"]
                old["文书纪年"] += "；" + rec["文书纪年"]
                old["文书岁"] = f'{old["文书岁"]}；{rec["文书岁"]}'
                old["书页"] += "；" + rec["书页"]
            continue
        out[key] = rec
    merge_ezang(out)
    recs = sorted(out.values(), key=lambda r: g.jdn(r["gregorian_year"], r["gregorian_month"], r["gregorian_day"]))
    path = ROOT / "data/calendar/敦煌朔日表.csv"
    with open(path, "w", encoding="utf-8", newline="") as f:
        f.write("# 敦煌政权朔日表：邓文宽《敦煌天文历法文献辑校》（1996）附录一比较表敦煌曆一栏有朔之月（吐蕃 808—834、归义军 858—993），"
                "由 scripts/calendar_data/build_dunhuang_shuori.py 自 data/calendar/敦煌历日_比较表.csv 生成；"
                "另并入《俄藏敦煌文獻》补录 dunhuang_ezang.csv 定年独立于朔之月（书页记「俄敦册/页」，余为邓书书页；据朔推定之件不收）。\n")
        f.write("# 依据：原件 = 出土具注历日可读（第 1 级证据，实物历日）；推算 = 原书括号格，据残历本身条件推得（同件内推演，仍属该历）。"
                "日期为民用历（儒略历）；月日数空 = 原书「?」或未列；"
                "文书纪年 = 原书定名（多照原件历首，如贞明八年、天复五年为敦煌延用旧号），文书岁 = 该件历日本岁（「表列次年正月」行记前一岁）；"
                "一月见于两件者以「；」并列。\n")
        w = csv.DictWriter(f, fieldnames=list(recs[0].keys()))
        w.writeheader()
        w.writerows(recs)
    n_orig = sum(r["朔依据"] == "原件" for r in recs)
    print(f"写 {path.relative_to(ROOT)}：{len(recs)} 月（原件 {n_orig}，推算 {len(recs) - n_orig}），"
          f"{len({r['lunar_year'] for r in recs})} 岁")


if __name__ == "__main__":
    main()
