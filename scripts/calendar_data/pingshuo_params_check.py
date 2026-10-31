"""以史志术文核验平朔诸历的上游参数：按志文通数/日法与积年生成朔日，逐月比对 推步政权朔日表（上游参数算出）。

生成法：上元之日为日序 0，第 k 朔在日序 floor(k × 通数 / 日法)。历表某月朔 J 合术文，当且仅当存在整数 k 使该式等于 J − off。
off 由积年 × 岁实定其大致，在 ±60 日内取相合最多者；相合的月数与元日干支一并输出。朔的相位若有差，几百个月不可能全合。
同时输出 data/calendar/pingshuo_params_check.csv。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/pingshuo_params_check.py
"""
import csv
import sys
from fractions import Fraction as Fr
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gaiyuan_date as g  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
# 历, 出处, 积年说明, 基年, 积年（算外）, 通数, 日法, 岁实分子, 岁实分母, 推步表历名
SPEC = [
    ("四分", "《后汉书·律历志下》", "上元庚辰至熹平三年甲寅 9455 算上（志未明言，干支证之）", 174, 9454, 27759, 940, 1461, 4, "四分历"),
    ("乾象", "《晋书》卷十七", "上元己丑至建安十一年丙戌 7378 算上", 206, 7377, 43026, 1457, 215130, 589, "乾象历"),
    ("景初", "《晋书》卷十八、《宋书》卷十二", "壬辰元至景初元年丁巳 4046 算上", 237, 4045, 134630, 4559, 673150, 1843, "景初历"),
    ("三纪", "《晋书》卷十八", "甲子上元至太元九年甲申 83841 算上；日法志作 6062，由通数证为 6063", 384, 83840, 179044, 6063, 365 * 2451 + 605, 2451, "三纪历"),
    ("正光", "《魏书》卷一〇七上", "壬子元至熙平二年丁酉 167745 算外", 517, 167745, 29 * 74952 + 39769, 74952, 2213377, 6060, "正光历"),
    ("兴和", "《魏书》卷一〇七下", "上元甲子至兴和二年庚申 293997 算上", 540, 293996, 6158017, 208530, 6158017, 16860, "兴和历"),
    ("天保", "《隋书》卷十七", "上元甲子至天保元年庚午 110526 算外（点校本补「二十」）；日法、通数未见于志，取上游", 550, 110526, 29 * 292635 + 155272, 292635, 365 * 23660 + 5787, 23660, "天保历"),
    ("天和", "《隋书》卷十七", "上元甲寅至天和元年丙戌 875792 算外", 566, 875792, 29 * 290160 + 153991, 290160, 365 * 23460 + 5731, 23460, "天和历"),
    ("大象", "《隋书》卷十七", "上元丙寅至大象元年己亥 41554 算上", 579, 41553, 29 * 53563 + 28422, 53563, 365 * 12992 + 3167, 12992, "大象历"),
    ("开皇", "《隋书》卷十七", "上元甲子至开皇四年甲辰 4129001 算上", 584, 4129000, 5372209, 181920, 365 * 102960 + 25063, 102960, "开皇历"),
]


def table():
    t = {}
    with open(ROOT / "data/calendar/推步政权朔日表.csv", encoding="utf-8") as f:
        for r in csv.DictReader(f):
            t.setdefault(r["calendar"], set()).add(g.jdn(int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"])))
    return t


def hits(js, off, tong, ri):
    n = 0
    for j in js:
        d = j - off
        k = -(-d * ri // tong)              # ceil(d × 日法 / 通数)
        n += k * tong // ri == d
    return n


def main():
    tab = table()
    out = []
    for name, src, note, y, ji, tong, ri, sn, sd, cal in SPEC:
        js = sorted(tab[cal])
        y_jdn = g.jdn(y - 1, 12, 20)
        base = y_jdn - int(Fr(ji * sn, sd))
        best = max(range(base - 60, base + 61), key=lambda off: hits(js, off, tong, ri))
        n = hits(js, best, tong, ri)
        yr = Fr(sn, sd)
        out.append({"历": name, "出处": src, "积年": note, "通数": tong, "日法": ri, "朔策": f"{tong / ri:.6f}", "岁实": f"{float(yr):.6f}",
                    "推步表月数": len(js), "合术文": n, "元日干支": g.ganzhi_of_jdn(best), "off 偏离积年估计（日）": best - base})
        print(name, f"{n}/{len(js)}", "元日", g.ganzhi_of_jdn(best), "偏", best - base)
    with open(ROOT / "data/calendar/pingshuo_params_check.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 平朔诸历：术文生成的朔日与上游参数算出的推步表逐月比对（scripts/calendar_data/pingshuo_params_check.py）。\n")
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader()
        w.writerows(out)


if __name__ == "__main__":
    main()
