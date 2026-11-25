"""本纪明书朔日「其他」类（干支与张表该月朔差 3 日以上、又非邻月或闰月之朔）逐条排查致误之由。

对 data/calendar/benji_shuo_check.csv 中「类 = 其他」之条，机械列出各种可能的解释（只列可能，不下结论）：
1. 一字误：本纪干支改天干或地支一字即得张表该月朔（天干差 2、4 字 → 日差 ±12、±24、±36、±48；地支差 → ±10、±20……）；
2. 年月误：本纪干支恰为张表前后 18 个月内某月之朔（解析误定年、月，或本纪错简、月名脱误）；
3. 月内日：本纪干支落在张表该月之内，或为该月某日之干支而「朔」字衍（或「朔」上脱日事）；
4. 同书上下文：本纪纪日表中该条前后同月之条的干支，在张表该月内否、在以本纪所书朔起算之 30 日内否；
5. 日食之条另据 data/calendar/benji_shuo_sky.csv 不合（此类只在朔差一二日中计算，「其他」中的日食之条看合朔所在月）。
人工按语（何者成立、依据）录于 data/calendar/benji_qita_kaozheng_notes.csv（书、年、月、本纪朔、判、按语），合并输出。
输出 data/calendar/benji_qita_kaozheng.csv 与 doc/calendar/benji-qita-kaozheng.md。

用法（仓库根）：python3 scripts/calendar_data/benji_qita_kaozheng.py
"""
import csv
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import discriminating_records as d  # noqa: E402
import gaiyuan_date as g  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
BOOK = {"hanshu_benji": "汉书", "houhanshu_benji": "后汉书", "jinshu_benji": "晋书", "jiutangshu_benji": "旧唐书", "xintangshu_benji": "新唐书",
        "jiuwudaishi_benji": "旧五代史", "xinwudaishi_benji": "新五代史", "songshi_benji": "宋史", "liaoshi_benji": "辽史", "jinshi_benji": "金史",
        "yuanshi_benji": "元史", "mingshi_benji": "明史", "qingshigao_benji": "清史稿", "beishi_benji": "北史", "weishu_benji": "魏书"}
ORDER = ("汉", "三国", "晋", "南朝", "北朝", "隋", "唐", "五代", "宋", "辽", "金", "元", "明", "清")
# 排查中查出的纪日表定年、判朔之误，已在抽取器 scripts/calendar_data/qinhan_eval/zhi_star_records.py 改正（BOOKS 的 corrections 与判朔逻辑），
# 改正后这些条不再落入「其他」（多数与张表相合）
FIXED = [
    ("新唐书", "661 年正月戊午", "年号「龍朔」的「朔」被当作月朔（原文「龍朔元年正月戊午」无朔）；同一误使天文志「龍朔元年五月甲子晦」日食记作朔"),
    ("旧唐书", "806 年八、九月丁酉、丁卯", "宪宗纪首「八月丁酉朔，受內禪」承顺宗纪卷末「元和元年」，实为永贞元年（805），合张表"),
    ("旧唐书", "761 年建辰、建巳月庚辰、庚戌", "上元二年九月去年号、但称元年，其后建辰、建巳月为 762 年，合张表"),
    ("旧五代史", "907 年四月丁亥", "明宗纪一「二年」「三年」「四年」为同光纪年而不书年号，承天祐作 907 年；实为同光四年（926），合张表；同书庄宗纪是月作丁丑，因此得判为误"),
    ("旧五代史", "941 年七、八、十月癸未、壬子、辛亥", "少帝纪上文脱「七年」（点校本注），高祖之丧诸条实为天福七年（942），合张表"),
    ("宋史", "1199 年八月己丑", "宁宗纪「紹熙五年」只作节标题，纪文「五年」配作庆元五年；实为绍熙五年（1194），合张表（同时消去该年闰位矛盾一条）"),
    ("宋史", "1293 年二月戊寅", "二王纪「十六年正月」承祥兴纪年；实为至元十六年（1279），合张表"),
    ("清史稿", "1916 年十月辛酉", "世祖纪首「八年」为崇德八年（1643），年号表缺后金诸元而配作宣统八年；借崇祯纪年定年后合张表"),
    ("清史稿", "1662 年十一月丙子", "圣祖纪首「帝即位……改元康熙」言来年改元，抽取时当即换作康熙，顺治十八年二月至十二月三十余条误系 1662 年；实为 1661 年十一月，合张表（2026-11-25 对点校本第 2 册书页 167 查出：此条在「康熙元年壬寅春正月乙亥朔」之前）"),
]
NUM = ["", "正", "二", "三", "四", "五", "六", "七", "八", "九", "十", "十一", "十二"]


def rows(path):
    return list(csv.DictReader(l for l in open(path, encoding="utf-8") if not l.startswith("#")))


def label(k):
    y, m, lp = k
    return f"{y} 年{'閏' if lp else ''}{NUM[m]}月"


def one_char(a, b):
    """a、b 两干支只差一字时返回「X作Y」（本纪 a，当作 b），否则 None。"""
    if a == b:
        return None
    if a[1] == b[1]:
        return f"本纪天干「{a[0]}」当作「{b[0]}」"
    if a[0] == b[0]:
        return f"本纪地支「{a[1]}」当作「{b[1]}」"
    return None


def main():
    zhang = d.tables()["张表"]
    order = sorted(zhang, key=zhang.get)
    idx = {k: i for i, k in enumerate(order)}
    sp = d.spans(zhang)
    notes = {}
    p = ROOT / "data/calendar/benji_qita_kaozheng_notes.csv"
    if p.exists():
        for n in rows(p):
            notes[(n["书"], n["年"], n["月"], n["本纪朔"])] = n
    # 同书纪日上下文
    ctx = {}
    for book in BOOK:
        path = ROOT / f"data/calendar/本纪纪日/{book}.csv"
        if not path.exists():
            continue
        recs = list(csv.DictReader(open(path, encoding="utf-8")))
        ctx[book] = recs
    out = []
    for r in rows(ROOT / "data/calendar/benji_shuo_check.csv"):
        if r["类"] != "其他":
            continue
        book, y, lab, gz = r["书"], int(r["年"]), r["月"], r["本纪朔"]
        leap, num = lab.startswith("闰"), int(lab.lstrip("闰"))
        t = g.table_month(r["政权"], y, g.MONTH_NAMES[num - 1], leap) or (y, num, leap)
        s0, e0 = sp[t]
        true = g.ganzhi_of_jdn(s0)
        gi = g.GANZHI.index(gz)
        cand = []
        oc = one_char(gz, true)
        if oc:
            cand.append(f"一字误：{oc}（张表{label(t)}朔{true}）")
        i = idx[t]
        for k in order[max(0, i - 18): i + 19]:
            if k != t and g.ganzhi_of_jdn(zhang[k]) == gz:
                cand.append(f"年月误：张表{label(k)}朔{gz}（距 {idx[k] - i:+d} 月）")
            elif abs(idx[k] - i) <= 1 and k != t:
                oc2 = one_char(gz, g.ganzhi_of_jdn(zhang[k]))
                if oc2:
                    cand.append(f"邻月且一字误：{oc2}（张表{label(k)}朔{g.ganzhi_of_jdn(zhang[k])}）")
        dd = (gi - (s0 + 49)) % 60
        if dd < e0 - s0:
            cand.append(f"月内日：{gz}为张表该月第 {dd + 1} 日，「朔」或衍")
        # 同书上下文：同年同月他条
        same = [c for c in ctx.get(book, []) if c["year"] == str(y) and c["month"] == lab and c.get("position") != "朔"]
        in_z = sum((g.GANZHI.index(c["ganzhi"]) - (s0 + 49)) % 60 < e0 - s0 for c in same)
        in_b = sum((g.GANZHI.index(c["ganzhi"]) - gi) % 60 < 30 for c in same)
        # 若干支为前后一年同名月之朔（年系误之候选），看同月他条落在该邻年月内几条
        alt = []
        for dy in (-1, 1):
            k2 = g.table_month(r["政权"], y + dy, g.MONTH_NAMES[num - 1], leap) or (y + dy, num, leap)
            if k2 in sp and g.ganzhi_of_jdn(sp[k2][0]) == gz:
                a0, a1 = sp[k2]
                alt.append(f"{y + dy}：{sum((g.GANZHI.index(c['ganzhi']) - (a0 + 49)) % 60 < a1 - a0 for c in same)}/{len(same)}")
        key = (book, r["年"], lab, gz)
        n = notes.get(key, {})
        out.append({"书": BOOK.get(book, book), "书键": book, "朝代": r["朝代"], "年": y, "纪年": r["纪年"], "月": lab, "本纪朔": gz,
                    "张表朔": true, "差": r["张表"], "日食": r["日食"],
                    "同书同月他条（合张表/合本纪朔/共）": f"{in_z}/{in_b}/{len(same)}" if same else "",
                    "同月他条合邻年": "；".join(alt),
                    "可能之解": "；".join(cand), "判": n.get("判", ""), "按语": n.get("按语", ""), "原文": r["原文"]})
    out.sort(key=lambda o: (ORDER.index(o["朝代"]), o["书"], o["年"]))
    cols = list(out[0])
    with open(ROOT / "data/calendar/benji_qita_kaozheng.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 本纪明书朔日「其他」类逐条排查（scripts/calendar_data/benji_qita_kaozheng.py 生成，勿手改；判与按语录于 benji_qita_kaozheng_notes.csv）\n")
        w = csv.DictWriter(f, cols)
        w.writeheader()
        w.writerows(out)
    def cls(v):
        """判语归类：去括注（当作之字、存疑），「一字误兼朔衍」「月名误兼一字误」等合并为「兼两误」。"""
        v = v.split("（")[0]
        return "兼两误" if "兼" in v else (v or "未判")
    stat = defaultdict(lambda: defaultdict(int))
    for o in out:
        stat[o["朝代"]][cls(o["判"])] += 1
        stat[o["朝代"]]["存疑"] += "存疑" in o["判"]
    md = ["# 本纪明书朔日「其他」类逐条排查（自动生成）", "",
          "> 由 `scripts/calendar_data/benji_qita_kaozheng.py` 生成，勿手改；判与按语在 `data/calendar/benji_qita_kaozheng_notes.csv`。方法见 [benji-checks.md](benji-checks.md)「其他」、[historical-calendar.md](historical-calendar.md)「以本纪核验历日」。", "",
          "「其他」= 本纪所书朔干支与张表该月朔差 3 日以上，又不是邻月或闰月之朔（[benji-checks.md](benji-checks.md) 第一节）。这类条不作历日证据，这里逐条找致误之由：",
          "一字误（改天干或地支一字即合张表）、年月误（干支为张表前后 18 个月内某月之朔）、月内日（干支在张表该月之内，「朔」字或衍）；",
          "「同书同月他条」为本纪同年同月其余纪日落在张表该月内 / 落在以本纪所书朔起算 30 日内 / 共几条；「合邻年」为干支恰是前后一年同名月之朔时，同月他条落在该邻年月内几条。机械所列只是可能，「判」为逐条读原文、上下文、他书与天象后所定；「存疑」为只有机械可能而无他证者。", "",
          "## 总表", ""]
    keys = [k for k in ("一字误", "朔衍", "一字误或朔衍", "兼两误", "年误", "食日书朔", "未明", "未判") if any(k in st for st in stat.values())]
    md.append("| 朝代 | 条数 | " + " | ".join(keys) + " | 其中存疑 |")
    md.append("|---|---|" + "---|" * (len(keys) + 1))
    tot = defaultdict(int)
    for dyn in sorted(stat, key=ORDER.index):
        n = sum(v for k, v in stat[dyn].items() if k != "存疑")
        tot["n"] += n
        for k in keys + ["存疑"]:
            tot[k] += stat[dyn].get(k, 0)
        md.append(f"| {dyn} | {n} | " + " | ".join(str(stat[dyn].get(k, 0)) for k in keys) + f" | {stat[dyn].get('存疑', 0)} |")
    md.append(f"| 合计 | {tot['n']} | " + " | ".join(str(tot[k]) for k in keys) + f" | {tot['存疑']} |")
    # 一字误所涉字对：本纪之字 → 当作之字
    pairs = defaultdict(int)
    for o in out:
        if o["判"].startswith("一字误") and "当作" in o["判"]:
            want = o["判"].split("当作")[1][:2]
            if len(want) == 2 and want[0] in g.GAN and want[1] in g.ZHI:
                a, b = o["本纪朔"], want
                x, y = (a[0], b[0]) if a[0] != b[0] else (a[1], b[1])
                pairs["↔".join(sorted((x, y), key=(g.GAN + g.ZHI).index))] += 1
    md += ["", "一字误所涉之字（不分方向）：" + "、".join(f"{k} {v}" for k, v in sorted(pairs.items(), key=lambda kv: -kv[1])) + "。"
           "「乙」「己」形近，相混最多，干支差 24 日（或 36 日）的条几乎都是此误。", "",
           "## 纪日表定年之误（已改正）", "",
           "排查中查出以下各条落入「其他」是纪日抽取之误而非本纪之误，已在抽取器改正，不再列入下表：", "",
           "| 书 | 原列 | 致误与改正 |", "|---|---|---|"] + [f"| {a} | {b} | {c} |" for a, b, c in FIXED]
    md += ["", "## 逐条", "", "| 书 | 年 | 纪年 | 月 | 本纪朔 | 张表朔 | 差 | 日食 | 同书同月他条 | 合邻年 | 可能之解 | 判 | 按语 | 原文 |",
           "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|"]
    for o in out:
        md.append("| " + " | ".join(str(o[c]).replace("|", "／") for c in ("书", "年", "纪年", "月", "本纪朔", "张表朔", "差", "日食",
                                                                            "同书同月他条（合张表/合本纪朔/共）", "同月他条合邻年", "可能之解", "判", "按语", "原文")) + " |")
    (ROOT / "doc/calendar/benji-qita-kaozheng.md").write_text("\n".join(md) + "\n", encoding="utf-8")
    print(len(out), dict(tot))


if __name__ == "__main__":
    main()
