"""辽历之检验：《辽史》本纪明书朔日与三种历比较——主表（宋历定朔）、祖冲之大明历平朔（《辽史·历象志》所载）、金重修大明历定朔
（daming_dingshuo.py，上推至辽代）。

《辽史·历象志》自言「歷書法禁不可得，求《大明曆》元，得祖沖之法於外史……遼歷因是固可補，然弗之補，史貴闕文也」：所载为祖冲之
大明历（《宋书·律历志》），不是统和十二年（994）贾俊所进之历。本脚本以本纪检验之。
祖冲之大明历推朔术：上元甲子至宋大明七年（463）癸卯 51939 年算外；积月 = 年数 × 章月 4836 ÷ 章岁 391，积日 = 积月 × 月法 116321
÷ 日法 3939，命甲子算外为天正十一月朔；累加朔策 29 + 2090/3939。实现以梁陈（510—588，行大明历）主表朔日验证。

输出 data/calendar/liao_calendar_check.csv（逐条）。
用法（仓库根）：uv run --project scripts python scripts/calendar_data/liao_calendar_check.py
"""
import csv
import sys
from fractions import Fraction as Fr
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import daming_dingshuo as dm  # noqa: E402
import discriminating_records as d  # noqa: E402
import gaiyuan_date as g  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
ZU_LUNAR = Fr(29 * 3939 + 2090, 3939)
PERIODS = ((907, 946, "辽初（未有历）"), (947, 993, "乙未元历（调元历）"), (994, 1125, "贾俊大明历"))


def zu_tianzheng(y):
    """祖冲之大明历：冬至在 y−1 年之岁的天正十一月朔（自上元甲子夜半起之日，含小余）。"""
    n = 51939 + (y - 463)
    return Fr(n * 4836 // 391 * 116321, 3939)


def zu_offset():
    """上元日序 → JDN：463 年天正朔干支对齐（462 年 12 月）。"""
    d0 = int(zu_tianzheng(463))
    approx = g.jdn(462, 12, 20)
    return min((approx + k for k in range(-40, 40) if (approx + k + 49) % 60 == d0 % 60), key=lambda j: abs(j - approx)) - d0


def zu_days(y0, y1):
    off, t = zu_offset(), zu_tianzheng(y0)
    out, k = [], 0
    while True:
        j = int(t + k * ZU_LUNAR) + off
        if g.from_jdn(j)[0] > y1:
            return out
        out.append(j)
        k += 1


def main():
    mt = d.tables()["主表"]
    lc = set(zu_days(500, 600))
    lc_main = [v for (y, _, _), v in mt.items() if 510 <= y <= 588]
    print(f"祖冲之大明历验证：梁陈 510—588 主表朔在平朔推步 {sum(v in lc for v in lc_main)}/{len(lc_main)}")
    zu = zu_days(900, 1130)
    off = dm.epoch_offset()
    jin = {}
    for y in range(900, 1135):
        for _, _, jn, day, h, lim, _ in dm.year_moons(y):
            jin[day + off] = (jn + off, h, lim)
    with open(ROOT / "data/calendar/main_table_check.csv", encoding="utf-8") as f:
        rows = [r for r in csv.DictReader(line for line in f if not line.startswith("#"))
                if r["政权"] == "辽" and r["本纪朔 JDN"] and r["类"] not in ("其他", "差一月")]
    out = []
    for r in rows:
        y, rec, s = int(r["年"]), int(r["本纪朔 JDN"]), int(r["主表朔 JDN"])
        z = min(zu, key=lambda x: abs(x - s))
        k = min(jin, key=lambda x: abs(x - s))
        jn, h, lim = jin[k]
        out.append({"年": y, "纪年": r["纪年"], "月": r["月"], "时期": next(p for lo, hi, p in PERIODS if lo <= y <= hi),
                    "本纪朔": r["本纪朔"], "宋历（主表）": g.ganzhi_of_jdn(s), "祖冲之平朔": g.ganzhi_of_jdn(z),
                    "金大明定朔": g.ganzhi_of_jdn(k), "定朔小余（时）": f"{h:.2f}", "进朔限（时）": f"{lim:.2f}",
                    "金大明进朔": g.ganzhi_of_jdn(jn), "合宋": int(rec == s), "合祖冲之": int(rec == z), "合金大明": int(rec == jn),
                    "原文": r["原文"]})
    with open(ROOT / "data/calendar/liao_calendar_check.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 《辽史》本纪明书朔日与宋历（主表）、祖冲之大明历平朔（历象志所载）、金重修大明历定朔上推之比较"
                "（scripts/calendar_data/liao_calendar_check.py）；进朔限为术文原限加经验改正 0.25 时（同 1182 年前代推）。\n")
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)
    for lo, hi, p in PERIODS:
        sub = [o for o in out if lo <= o["年"] <= hi]
        print(f"{p} {lo}—{hi}：{len(sub)} 条，合宋 {sum(o['合宋'] for o in sub)}，合祖冲之平朔 {sum(o['合祖冲之'] for o in sub)}，"
              f"合金大明 {sum(o['合金大明'] for o in sub)}")
    base = [v for (y, _, _), v in mt.items() if 994 <= y <= 1125]
    zs = set(zu)
    print(f"对照：994—1125 主表（宋定朔）朔在祖冲之平朔 {sum(v in zs for v in base)}/{len(base)}（定朔与平朔偶合之率）")


if __name__ == "__main__":
    main()
