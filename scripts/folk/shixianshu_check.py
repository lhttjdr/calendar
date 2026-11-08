"""乾隆三十年时宪书逐日宜与推算比对。

用法（仓库根）：
  rust/target/release/examples/puzhu_year 1765 csv > /tmp/x.csv   # 推算（月表 + 铺注，民用三十七事）
  python3 scripts/folk/shixianshu_check.py /tmp/x.csv
读 data/folk/shixianshu_qianlong30.csv（时宪书逐日转录），按（月, 日, 干支）对上推算，写
data/folk/shixianshu_qianlong30_checks.csv：结果「合」「異」，异者列推算多注、少注之事。
时宪书只印宜与少数「不宜」，故只比宜；「—」表示该日无宜（推算的諸事不宜同此）。
"""
import csv
import sys

SRC = "data/folk/shixianshu_qianlong30.csv"
OUT = "data/folk/shixianshu_qianlong30_checks.csv"


def items(s):
    s = s.strip()
    return [] if s in ("", "—", "諸事不宜") else s.split()


def main():
    calc = {(r["月"], r["日"], r["干支"]): r for r in csv.DictReader(open(sys.argv[1], encoding="utf-8"))}
    old = {}
    try:
        for r in csv.DictReader(open(OUT, encoding="utf-8")):
            old[(r["月"], r["日"])] = r
    except FileNotFoundError:
        pass
    rows, n_ok = [], 0
    for r in csv.DictReader(open(SRC, encoding="utf-8")):
        c = calc.get((r["月"], r["日"], r["干支"]))
        if c is None:
            print("推算无此日：", r["月"], r["日"], r["干支"])
            continue
        book, got = items(r["宜"]), items(c["宜"])
        more = [x for x in got if x not in book]
        less = [x for x in book if x not in got]
        ok = not more and not less
        n_ok += ok
        note = old.get((r["月"], r["日"]), {}).get("說明", "")
        rows.append({
            "月": r["月"], "日": r["日"], "干支": r["干支"], "書頁": r["書頁"],
            "時憲書宜": r["宜"], "推算宜": c["宜"], "月表宜": c["月表宜"], "鋪註": c["铺注"],
            "結果": "合" if ok else "異", "推算多注": " ".join(more), "推算少注": " ".join(less), "說明": note,
        })
    with open(OUT, "w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()), lineterminator="\n")
        w.writeheader()
        w.writerows(rows)
    print(f"{len(rows)} 日，合 {n_ok}，異 {len(rows) - n_ok}")
    for r in rows:
        if r["結果"] == "異":
            print(f"  {r['月']}月{r['日']} {r['干支']} 多注[{r['推算多注']}] 少注[{r['推算少注']}] 鋪註[{r['鋪註']}]")


if __name__ == "__main__":
    main()
