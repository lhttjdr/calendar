"""乾隆三十年时宪书逐日宜与推算比对。

用法（仓库根）：
  rust/target/release/examples/puzhu_year 1765 csv > /tmp/x.csv   # 推算（月表 + 铺注，民用三十七事）
  python3 scripts/folk/shixianshu_check.py /tmp/x.csv [转录.csv [检验.csv]]
他年：python3 scripts/folk/shixianshu_check.py /tmp/x1756.csv data/folk/shixianshu_qianlong21.csv
（检验表缺省为转录文件名加 _checks）。缺省读 data/folk/shixianshu_qianlong30.csv（时宪书逐日转录），按（月, 日, 干支）对上推算，写
data/folk/shixianshu_qianlong30_checks.csv：结果「合」「異」，异者列推算多注、少注之事。
时宪书只印宜与少数「不宜」，故只比宜；「—」表示该日无宜（推算的諸事不宜同此）；
「?」表示书影漫漶未能辨认，记「未辨」，不计合异。
朱字另比：书印之神应在推算「朱字」（应值全集）之内；推算有而书未印者多因宜事长、栏底无空，只记不判。
「不宜」同此：书印之事应在推算「不宜」（应值全集，puzhu::bu_yi）之内；推算有而书未印者只记不判。
"""
import csv
import sys

SRC = "data/folk/shixianshu_qianlong30.csv"
OUT = "data/folk/shixianshu_qianlong30_checks.csv"


def items(s):
    s = s.strip()
    return [] if s in ("", "—", "諸事不宜") else s.split()


def main():
    global SRC, OUT
    if len(sys.argv) > 2:
        SRC = sys.argv[2]
        OUT = sys.argv[3] if len(sys.argv) > 3 else SRC[: -len(".csv")] + "_checks.csv"
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
        bb, bc = r["不宜"].split(), c.get("不宜", "").split()
        by_extra = [x for x in bb if x not in bc]
        by_miss = [x for x in bc if x not in bb]
        zb, zc = r["朱字"].split(), c.get("朱字", "").split()
        z_extra = [x for x in zb if x not in zc]
        z_miss = [x for x in zc if x not in zb]
        if r["宜"].strip() == "?":
            rows.append({
                "月": r["月"], "日": r["日"], "干支": r["干支"], "書頁": r["書頁"],
                "時憲書宜": "?", "推算宜": c["宜"], "月表宜": c["月表宜"], "鋪註": c["铺注"],
                "結果": "未辨", "推算多注": "", "推算少注": "", "說明": old.get((r["月"], r["日"]), {}).get("說明", ""),
                "時憲書朱字": r["朱字"], "推算朱字": c.get("朱字", ""), "朱字書有推算無": " ".join(z_extra), "朱字推算有書未印": " ".join(z_miss),
                "時憲書不宜": r["不宜"], "推算不宜": c.get("不宜", ""), "不宜書有推算無": " ".join(by_extra), "不宜推算有書未印": " ".join(by_miss),
            })
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
            "時憲書朱字": r["朱字"], "推算朱字": c.get("朱字", ""), "朱字書有推算無": " ".join(z_extra), "朱字推算有書未印": " ".join(z_miss),
            "時憲書不宜": r["不宜"], "推算不宜": c.get("不宜", ""), "不宜書有推算無": " ".join(by_extra), "不宜推算有書未印": " ".join(by_miss),
        })
    with open(OUT, "w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()), lineterminator="\n")
        w.writeheader()
        w.writerows(rows)
    n_un = sum(r["結果"] == "未辨" for r in rows)
    print(f"{len(rows)} 日，合 {n_ok}，異 {len(rows) - n_ok - n_un}，未辨 {n_un}")
    zh = sum(len(r["時憲書朱字"].split()) for r in rows)
    zx = [(r["月"], r["日"], r["朱字書有推算無"]) for r in rows if r["朱字書有推算無"]]
    zm = sum(len(r["朱字推算有書未印"].split()) for r in rows)
    print(f"朱字：书印 {zh} 位，在推算之内 {zh - sum(len(x[2].split()) for x in zx)}，书有推算无 {sum(len(x[2].split()) for x in zx)}，推算有书未印 {zm}")
    for m, d, x in zx:
        print(f"  朱字 {m}月{d} 书有推算无 [{x}]")
    ok = [r for r in rows if r["結果"] != "未辨"]
    bh = sum(len(r["時憲書不宜"].split()) for r in ok)
    bx = [(r["月"], r["日"], r["不宜書有推算無"]) for r in ok if r["不宜書有推算無"]]
    bm = sum(len(r["不宜推算有書未印"].split()) for r in ok)
    nx = sum(len(x[2].split()) for x in bx)
    print(f"不宜：书印 {bh} 处，在推算之内 {bh - nx}，书有推算无 {nx}，推算有书未印 {bm}（不计未辨之日）")
    for m, d, x in bx:
        print(f"  不宜 {m}月{d} 书有推算无 [{x}]")
    for r in rows:
        if r["結果"] == "異":
            print(f"  {r['月']}月{r['日']} {r['干支']} 多注[{r['推算多注']}] 少注[{r['推算少注']}] 鋪註[{r['鋪註']}]")


if __name__ == "__main__":
    main()
