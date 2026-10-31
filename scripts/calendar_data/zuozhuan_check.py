"""《左传》传文纪日检验：传文「某月干支」与春秋历（张培瑜表 3-7）的月序关系。

只取月与干支紧邻者（「三月甲寅」）；传中引述前事、对话的日期无法逐条排除，属噪声。与同年经文同月同干支者记「同经」，不算独立。
对每条试月序偏移 k = -1, 0, +1, +2：传文 m 月当春秋历 m+k 月时，干支是否在该月内；+2 相当于传文用建寅（夏正），经用建子。
任意干支落在一个月内的基线约 49%。春秋历是由经文拟合的，所以这里看的是传文相对经文的月序差，不是证明哪部历对。

输出 data/calendar/zuozhuan_check.csv。
用法（仓库根）：uv run --project scripts python scripts/calendar_data/zuozhuan_check.py <缓存目录>
"""
import csv
import re
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import chunqiu_check as c  # noqa: E402
import gaiyuan_date as g  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
PAT = re.compile(r"(?:(閏)月|(十有[一二]|十[一二]|十|正|[一二三四五六七八九])月)[，．、。]?(" + "|".join(g.GANZHI) + r")[，．、。]?(朔)?")
OFFS = (-1, 0, 1, 2)


def span(y, m, k):
    """春秋历 y 年第 m 月偏移 k 月后的 (朔, 下月朔)；闰月在岁末，跨年按实际月数。超出闰表返回 None。"""
    i = m - 1 + k
    while True:
        if not 0 <= y + 721 < len(c.CQ_LEAP):
            return None
        ms = c.chunqiu_year(y)
        n = len(ms) - 1
        if i < 0:
            y -= 1
            if y + 721 < 0:
                return None
            i += len(c.chunqiu_year(y)) - 1
        elif i >= n:
            i -= n
            y += 1
        else:
            return ms[i], ms[i + 1]


def parse(cache):
    start = dict(c.DUKES)
    out = []
    for duke, text in c.source(Path(cache)):
        text = re.sub(r"\{\{[^{}]*\}\}|<ref.*?</ref>", "", text, flags=re.S)
        for sec in re.split(r"(?m)^(?===[^=])", text):
            m = re.match(rf"=={duke}([元一二三四五六七八九十]+)年", sec)
            if not m:
                continue
            n = c.g_num(m[1])
            y = start[duke] + n - 1
            for part in re.split(r"(?m)^(?====)", sec):
                if not re.match(r"===\s*傳", part):
                    continue
                for t in PAT.finditer(part):
                    if t[1]:
                        continue
                    a = max(0, t.start() - 12)
                    out.append({"公": duke, "公年": n, "年": y, "月": c.NUM[t[2]], "干支": t[3], "朔": int(bool(t[4])), "原文": part[a:t.end() + 14].replace("\n", " ")})
    return out

# 经传同年同干支而月不同、人工核对为同一事件者：(公, 公年, 干支, 传月, 经月, 事)。解析只取月干支紧邻，「壬戌，战于韩原」这类漏掉
SAME_EVENT = [("僖公", 17, "乙亥", 10, 12, "齐桓公卒"), ("成公", 10, "丙午", 6, 5, "晋景公卒"), ("襄公", 2, "庚辰", 7, 6, "郑伯睔卒"),
              ("襄公", 9, "己亥", 11, 12, "同盟于戏"), ("襄公", 25, "己巳", 7, 8, "同盟于重丘"), ("昭公", 8, "壬午", 11, 10, "楚灭陈")]


def clusters(rows):
    """同一年独立条目中偏 0 不合 ≥2 条的，看偏 ±1 能否整年解释。"""
    by = defaultdict(list)
    for r in rows:
        if not r["同经"]:
            by[(r["公"], r["公年"], r["年"])].append(r)
    out = []
    for (d, n, y), rs in by.items():
        bad = [r for r in rs if r["偏+0"] == "外"]
        if len(bad) >= 2:
            out.append({"公": d, "公年": n, "年": y, "条数": len(rs), "偏0不合": len(bad), "偏-1合": sum(r["偏-1"] != "外" for r in rs),
                        "偏+1合": sum(r["偏+1"] != "外" for r in rs), "偏+0合": len(rs) - len(bad), "偏+2合": sum(r["偏+2"] != "外" for r in rs)})
    with open(ROOT / "data/calendar/zuozhuan_year_clusters.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 传文不合春秋历集中的年份：全年条目在各月序偏移下的合数（zuozhuan_check.py）。\n")
        w = csv.DictWriter(f, fieldnames=list(out[0]))
        w.writeheader()
        w.writerows(out)
    for o in out:
        print(o)


def main():
    recs = [r for r in parse(sys.argv[1] if len(sys.argv) > 1 else "chunqiu_cache") if 0 <= r["年"] + 721 < len(c.CQ_LEAP)]
    with open(ROOT / "data/calendar/chunqiu_check.csv", encoding="utf-8") as f:
        jing = {(r["公"], r["公年"], r["月"], r["干支"]) for r in csv.DictReader(line for line in f if not line.startswith("#"))}
    rows, tal = [], defaultdict(lambda: defaultdict(lambda: [0, 0, 0, 0]))
    for r in recs:
        k = g.GANZHI.index(r["干支"])
        row = {x: r[x] for x in ("公", "公年", "年", "月", "干支", "朔")}
        row["同经"] = int((r["公"], str(r["公年"]), str(r["月"]), r["干支"]) in jing)
        best = []
        for o in OFFS:
            s = span(r["年"], r["月"], o)
            hit = sh = False
            if s:
                hit, sh = any((d + 49) % 60 == k for d in range(*s)), (s[0] + 49) % 60 == k
            row[f"偏{o:+d}"] = ("朔" if sh else "内") if hit else "外"
            if hit:
                best.append(o)
        row["原文"] = r["原文"]
        rows.append(row)
        if row["同经"]:
            continue
        for grp in (r["公"], "全"):
            for o in OFFS:
                t = tal[grp][o]
                t[0] += row[f"偏{o:+d}"] != "外"
                t[1] += 1
                t[2] += row[f"偏{o:+d}"] == "朔" if r["朔"] else 0
                t[3] += r["朔"]
    with open(ROOT / "data/calendar/zuozhuan_check.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 《左传》传文「某月干支」与春秋历月序偏移（zuozhuan_check.py）；同经=同年经文有同月同干支。\n")
        w = csv.DictWriter(f, fieldnames=list(rows[0]))
        w.writeheader()
        w.writerows(rows)
    clusters(rows)
    print(f"传文 {len(rows)} 条，同经 {sum(x['同经'] for x in rows)}，独立 {tal['全'][0][1]}")
    print("公 条数 " + " ".join(f"偏{o:+d}" for o in OFFS))
    for grp in [d for d, _ in c.DUKES] + ["全"]:
        t = tal[grp]
        if t[0][1]:
            print(grp, t[0][1], " ".join(f"{t[o][0]}({t[o][0] / t[o][1]:.0%})" for o in OFFS), "书朔 " + " ".join(f"{t[o][2]}/{t[o][3]}" for o in OFFS))


if __name__ == "__main__":
    main()
