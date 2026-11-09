"""鳌头卷三婚姻诸表与协纪月表、董公、类编逐日对照。

用法（仓库根）：python3 scripts/folk/aotou_vs_xieji.py
先跑 scripts/folk/aotou_tables.py 生成 data/folk/aotou_jiaqu_days.csv。
读 data/folk/xieji_yuebiao.csv（协纪月表）、data/folk/dong_gong_days.csv（董公逐日判定），
类编卷七三表（求婚、下定、嫁娶）为书影转录，写在本脚本 LEIBIAN（02094406.cn PDF 第 42—45 页）。
写 data/folk/aotou_jiaqu_vs_xieji.csv（一行一表一日），并在标准输出打印 doc 用的统计表。

事项对应：求婚 → 协纪「納采問名」；下定結婚 → 「結婚姻」；嫁娶、備急 → 「嫁娶」。
协纪三态：宜／忌（含諸事不宜）／两不注。董公取「婚姻」明文（无条件者优先，次条件之辞，次「諸事」泛言），皆无则「无」。
"""
import csv
import re
from collections import Counter, defaultdict

GAN = "甲乙丙丁戊己庚辛壬癸"
ZHI = "子丑寅卯辰巳午未申酉戌亥"
GZ = re.compile(f"[{GAN}][{ZHI}]")

AOTOU = "data/folk/aotou_jiaqu_days.csv"
XIEJI = "data/folk/xieji_yuebiao.csv"
DONG = "data/folk/dong_gong_days.csv"
OUT = "data/folk/aotou_jiaqu_vs_xieji.csv"

EVENT = {"逐月求婚吉日": "納采問名", "逐月下定結婚吉日": "結婚姻", "逐月嫁娶吉日": "嫁娶", "逐月嫁娶結婚吉日備急用": "嫁娶"}
SHORT = {"逐月求婚吉日": "求婚", "逐月下定結婚吉日": "下定", "逐月嫁娶吉日": "嫁娶", "逐月嫁娶結婚吉日備急用": "備急"}

# 类编卷七（北大藏本 02094406.cn.pdf）；「|」后为「外」日
LEIBIAN = {
    "求婚": [  # 第 42—43 页
        "丙子戊子壬子庚寅辛卯丁丑癸卯庚午丙午壬午辛未丁未",
        "丙寅丁卯己巳乙巳庚寅甲寅己卯辛卯癸卯丙戌|甲戌壬戌",
        "庚子丁卯丙子戊子壬子癸卯|癸酉己酉",
        "丙子戊子庚子乙丑丁丑庚午丁卯辛卯癸丑乙卯",
        "丙寅戊寅庚寅甲寅壬辰辛未己未甲辰|戊辰丙辰",
        "乙卯戊寅己卯庚寅甲寅丁卯己卯辛卯癸卯",
        "丙子戊子壬辰|庚子壬午丙午甲子庚午癸未丁未壬子戊辰丙辰",
        "乙丑丁丑癸丑丙寅庚寅壬辰乙巳|乙亥丙辰",
        "丙子丁卯己卯辛卯癸卯庚午壬午丙午|辛酉",
        "丙子庚午壬午辛未癸未乙未丁未己未",
        "丙寅庚寅戊寅甲寅己巳",
        "乙丑丙寅戊寅庚寅甲寅己卯辛卯乙卯",
    ],
    "下定": [  # 第 43—45 页；第 44 页首栏月名字形近「正」，按栏序当为二月（未核）；五、十月末「巳未」读作己未
        "丁卯己卯辛卯乙卯庚午辛未丙午丙子戊子壬子",
        "丙寅戊寅甲寅庚寅丁未己未辛未己亥乙未",
        "丙子戊子壬子丁卯乙卯庚午丙午庚子",
        "丁卯己卯辛卯癸卯庚午乙丑戊子癸丑",
        "乙丑丙寅戊寅甲寅庚寅壬辰辛未癸未丁未己未",
        "丙寅丁卯戊寅己卯庚寅辛卯癸卯甲寅乙卯",
        "丙午庚午辛未丙子戊子壬子丁未癸未己未",
        "丙寅戊寅壬辰庚寅",
        "丙子庚午丙午丁卯辛卯癸卯",
        "丁卯己卯戊子癸卯庚午辛未丁未辛卯乙卯己未",
        "丙寅辛未戊寅甲寅壬辰丁未庚寅己未",
        "丙寅丁卯戊寅己卯庚寅辛卯癸卯甲寅乙卯",
    ],
    "嫁娶": [  # 第 45 页起，只核了正—六月，与鳌头逐字相同（注语偶有「上吉」之差）
        "丙子丁卯辛卯壬子壬午乙未己卯",
        "乙丑己丑丁丑乙未癸丑",
        "己卯",
        "丁卯癸卯乙卯己卯",
        "乙丑己丑丁丑癸丑",
        "癸卯丁卯乙卯辛卯庚寅戊寅己卯",
    ] + [None] * 6,
}


LEIBIAN_OTHER = "data/folk/leibian_jiaqu_jiri.csv"  # 类编卷七吉日另一录本（另行转录）；有则补嫁娶七—十二月与備急表


def merge_other():
    import os
    if not os.path.exists(LEIBIAN_OTHER):
        return
    o = defaultdict(list)
    for r in csv.DictReader(open(LEIBIAN_OTHER, encoding="utf-8")):
        o[(r["表"], int(r["月"]))].append(r["干支"])
    LEIBIAN.setdefault("嫁娶備急", [None] * 12)
    for key in ("嫁娶", "嫁娶備急"):
        for m in range(1, 13):
            if LEIBIAN[key][m - 1] is None and o.get((key, m)):
                LEIBIAN[key][m - 1] = "".join(o[(key, m)])


def load_xieji():
    t = {}
    for r in csv.DictReader(open(XIEJI, encoding="utf-8")):
        t[(int(r["月"]), r["干支"])] = r
    return t


def xieji_state(r, ev):
    yi = r["宜"].split()
    ji = r["忌"].split()
    if "諸事不宜" in yi or "諸事不宜" in ji:
        return "忌"
    if ev in yi:
        return "宜"
    if ev in ji:
        return "忌"
    return "两不注"


def load_dong():
    d = defaultdict(list)
    for r in csv.DictReader(open(DONG, encoding="utf-8")):
        d[(int(r["月"]), r["干支"])].append(r)
    return d


def dong_state(rows):
    hy = [r for r in rows if r["事"] == "婚姻"]
    plain = [r for r in hy if not r["条件"]]
    if plain:
        return plain[-1]["宜忌"]
    if hy:
        return "条件" + hy[-1]["宜忌"]
    zs = [r for r in rows if r["事"] == "諸事"]
    zplain = [r for r in zs if not r["条件"]]
    if zplain:
        return zplain[-1]["宜忌"] + "（泛言）"
    if zs:
        return "条件" + zs[-1]["宜忌"] + "（泛言）"
    return "无"


def lb_set(key, m):
    s = LEIBIAN.get(key, [None] * 12)[m - 1]
    if s is None:
        return None
    main, _, wai = s.partition("|")
    return {g: "正" for g in GZ.findall(main)} | {g: "外" for g in GZ.findall(wai)}


def main():
    merge_other()
    xj = load_xieji()
    dg = load_dong()
    rows = [r for r in csv.DictReader(open(AOTOU, encoding="utf-8")) if r["干支"]]
    out = []
    seen = set()
    for r in rows:
        m, g, t = int(r["月"]), r["干支"], r["表"]
        if (t, m, g) in seen:  # 原表重出之日只计一次
            continue
        seen.add((t, m, g))
        x = xj[(m, g)]
        ev = EVENT[t]
        key = SHORT[t]
        lbk = {"備急": "嫁娶備急"}.get(key, key)
        lb = lb_set(lbk, m)
        lbv = "未核" if lb is None else lb.get(g, "不收")
        out.append({
            "表": key, "月": m, "干支": g, "鳌头等": r["等"], "鳌头注": r["注"], "建除": x["建除"],
            "协纪事": ev, "协纪": xieji_state(x, ev),
            "协纪結婚姻": xieji_state(x, "結婚姻"), "协纪嫁娶": xieji_state(x, "嫁娶"),
            "董公婚姻": dong_state(dg.get((m, g), [])),
            "类编": lbv, "协纪等第": x["等第"], "协纪凶神": x["凶神"], "协纪吉神": x["吉神"],
        })
    cols = list(out[0].keys())
    with open(OUT, "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=cols)
        w.writeheader()
        w.writerows(out)
    report(out, xj)


def pct(a, b):
    return f"{a}（{round(100 * a / b)}%）" if b else "0"


def report(out, xj):
    print("## 总表")
    print("| 表 | 日数 | 协纪宜 | 协纪忌 | 协纪两不注 | 董公宜 | 董公忌 | 董公无/条件 | 类编同收 | 类编不收 |")
    print("|---|---|---|---|---|---|---|---|---|---|")
    for key in ["求婚", "下定", "嫁娶", "備急"]:
        rs = [r for r in out if r["表"] == key]
        n = len(rs)
        c = Counter(r["协纪"] for r in rs)
        dyi = sum(r["董公婚姻"].startswith("宜") for r in rs)
        dji = sum(r["董公婚姻"].startswith("忌") for r in rs)
        lb = [r for r in rs if r["类编"] != "未核"]
        lbin = sum(r["类编"] in ("正", "外") for r in lb)
        print(f"| {key} | {n} | {pct(c['宜'], n)} | {pct(c['忌'], n)} | {pct(c['两不注'], n)} | {pct(dyi, n)} | {pct(dji, n)} | {n - dyi - dji} | {lbin}/{len(lb)} | {len(lb) - lbin} |")

    print("\n## 下定：正选与外日分开")
    rs = [r for r in out if r["表"] == "下定"]
    for k in ["正", "外"]:
        sub = [r for r in rs if r["鳌头等"] == k]
        c = Counter(r["协纪"] for r in sub)
        print(f"- {k}：{len(sub)} 日，协纪結婚姻宜 {c['宜']}、忌 {c['忌']}、两不注 {c['两不注']}")

    print("\n## 协纪覆盖：协纪宜該事之日被鳌头收入的比例")
    for key, ev in [("求婚", "納采問名"), ("下定", "結婚姻"), ("嫁娶", "嫁娶")]:
        tot = sum(1 for (m, g), x in xj.items() if xieji_state(x, ev) == "宜")
        got = sum(1 for r in out if r["表"] == key and r["协纪"] == "宜")
        print(f"- {key}（协纪{ev}）：协纪宜 {tot} 日，鳌头收 {got} 日")

    print("\n## 类编同名表自身与协纪（作参照）")
    print("| 表 | 日数 | 协纪宜 | 协纪忌 | 协纪两不注 |")
    print("|---|---|---|---|---|")
    for key, ev in [("求婚", "納采問名"), ("下定", "結婚姻")]:
        c = Counter()
        for m in range(1, 13):
            for g in lb_set(key, m):
                c[xieji_state(xj[(m, g)], ev)] += 1
        n = sum(c.values())
        print(f"| 类编{key} | {n} | {pct(c['宜'], n)} | {pct(c['忌'], n)} | {pct(c['两不注'], n)} |")
    for key, ev in [("求婚", "納采問名"), ("下定", "結婚姻")]:
        rs = [r for r in out if r["表"] == key and r["类编"] == "不收"]
        c = Counter(r["协纪"] for r in rs)
        print(f"- 鳌头{key}有而类编无者 {len(rs)} 日：协纪宜 {c['宜']}、忌 {c['忌']}、两不注 {c['两不注']}")
        drop = []
        for m in range(1, 13):
            have = {r["干支"] for r in out if r["表"] == key and r["月"] == m}
            for g in lb_set(key, m):
                if g not in have:
                    drop.append(xieji_state(xj[(m, g)], ev))
        c = Counter(drop)
        print(f"- 类编{key}有而鳌头删者 {len(drop)} 日：协纪宜 {c['宜']}、忌 {c['忌']}、两不注 {c['两不注']}")

    print("\n## 建除分布")
    print("| 表 | " + " | ".join("建除滿平定執破危成收開閉") + " |")
    print("|---|" + "---|" * 12)
    for key in ["求婚", "下定", "嫁娶", "備急"]:
        c = Counter(r["建除"] for r in out if r["表"] == key)
        print(f"| {key} | " + " | ".join(str(c[j]) for j in "建除滿平定執破危成收開閉") + " |")

    print("\n## 鳌头收而协纪忌之日，协纪凶神出现次数（前 12）")
    for key in ["求婚", "下定", "嫁娶", "備急"]:
        c = Counter()
        n = 0
        for r in out:
            if r["表"] == key and r["协纪"] == "忌":
                n += 1
                c.update(r["协纪凶神"].split())
        print(f"- {key}（{n} 日）：" + "、".join(f"{k} {v}" for k, v in c.most_common(12)))

    print("\n## 下定逐日")
    print("| 月 | 干支 | 鳌头 | 建除 | 类编下定 | 协纪結婚姻 | 协纪嫁娶 | 董公婚姻 |")
    print("|---|---|---|---|---|---|---|---|")
    for r in out:
        if r["表"] == "下定":
            print(f"| {r['月']} | {r['干支']} | {r['鳌头等']} | {r['建除']} | {r['类编']} | {r['协纪結婚姻']} | {r['协纪嫁娶']} | {r['董公婚姻']} |")


if __name__ == "__main__":
    main()
