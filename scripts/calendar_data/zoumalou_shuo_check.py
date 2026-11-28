"""长沙走马楼三国吴简（《长沙走马楼三国吴简·竹简》释文册）中带日序与干支、或书「某干支朔」的纪日：检验吴历（乾象历推步）与魏历。

吴简是孙吴长沙郡临湘侯国的当时官文书（第 1 级实物），所书日辰即当时吴国行用之历日。释文册编者注常据陈垣《魏蜀吴朔闰异同表》
推日（今人推算，不作证据）；这里只用简文本身：
  A 「某年某月某日 + 干支」：朔 = 干支 − (日 − 1)；
  B 「某月某干支朔」。
OCR（RapidOCR，竖排）候选由 scripts 外部步骤生成（JSON：页 → [(x, y, 文字, 置信)]），本脚本只做匹配与比对；每条候选须对原书页图人工核对后
写入 data/calendar/zoumalou_jiri.csv（列：册,书页,简号,年号,年,月,闰,日,干支,朔干支,原文,核对），检验只用核对过的行
（核对=1 本项目对原书页图核过；核对=2 本项目无原书，据学者论文所引正式释文并附简号——引者据图版改释之处随之，备注注明出处）。
吴历：data/calendar/推步政权朔日表.csv「吴」（乾象历推步）；魏历：张培瑜历表（魏行黄初/景初，与主序列推步同）。
输出 doc/calendar/zoumalou-checks.md 与 data/calendar/zoumalou_months.csv（逐月所据之简与分类，regime_benji_eval 读入并入分政权逐月表）。
用法（仓库根）：python3 scripts/calendar_data/zoumalou_shuo_check.py candidates <ocr.json>...  （列出候选）
            python3 scripts/calendar_data/zoumalou_shuo_check.py check                       （检验已核对之行）
"""
import csv
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))
import shiwu_fenlei as fl  # noqa: E402
GZ = ["甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12] for i in range(60)]
# 年号以末字认：简文年号首字常残（「☐禾元年」），OCR 亦常误首字；末字四号互不相同
ERA = {"安": 196, "武": 222, "龍": 229, "龙": 229, "禾": 232, "烏": 238, "乌": 238}
ERA_NAME = {196: "建安", 222: "黄武", 229: "黄龍", 232: "嘉禾", 238: "赤烏"}
NUM = {"元": 1, "正": 1, "一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9, "十": 10}


def cn(s):
    s = s.replace("廿", "二十").replace("卅", "三十")
    if s in NUM:
        return NUM[s]
    if s.startswith("十"):
        return 10 + (NUM[s[1:]] if s[1:] else 0)
    if "十" in s:
        a, b = s.split("十")
        return NUM[a] * 10 + (NUM[b] if b else 0)
    return None


G = "[甲乙丙丁戊己庚辛壬癸][子丑寅卯辰巳午未申酉戌亥]"
N = "[元正一二三四五六七八九十廿卅]+"
E = "[" + "".join(ERA) + "]"
PA = re.compile(rf"({E})({N})年(閏|闰)?({N})月({N})日({G})")
PB = re.compile(rf"(?:({E})({N})年)?(閏|闰)?({N})月({G})朔")


def candidates(paths):
    for p in paths:
        data = json.load(open(p))
        vol = Path(p).stem
        for pg, boxes in sorted(data.items(), key=lambda kv: int(kv[0])):
            for x, y, t, c in boxes:
                for m in PA.finditer(t):
                    print(f"{vol}\tPDF页{int(pg) + 1}\tA\t{m.group(0)}\t{t}")
                for m in PB.finditer(t):
                    print(f"{vol}\tPDF页{int(pg) + 1}\tB\t{m.group(0)}\t{t}")


def jd(y, m, d):
    a = (14 - m) // 12
    yy, mm = y + 4800 - a, m + 12 * a - 3
    return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - 32083


def months(rows):
    out = {}
    for y, m, l, j in rows:
        out[(y, m, l)] = j
    return out, sorted(out.values())


def check():
    wu = months((int(r["lunar_year"]), int(r["lunar_month"]), r["is_leap"] == "1", jd(int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"])))
                for r in csv.DictReader(open(ROOT / "data/calendar/推步政权朔日表.csv", encoding="utf-8")) if r["regime"] == "吴")
    wei = months((int(r["lunar_year"]), int(r["lunar_month"]), r["is_leap"] == "1", jd(int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"])))
                 for r in csv.DictReader(open(ROOT / "vendor/calendar/chinese_calendar/实历朔日表.csv", encoding="utf-8")) if 200 <= int(r["lunar_year"]) <= 290)
    rows = [r for r in csv.DictReader(l for l in open(ROOT / "data/calendar/zoumalou_jiri.csv", encoding="utf-8") if not l.startswith("#")) if r["核对"] in ("1", "2")]
    md = ["# 走马楼吴简纪日与吴、魏历（自动生成）", "", "> 由 `scripts/calendar_data/zoumalou_shuo_check.py check` 生成，勿手改。底本与方法见 [historical-calendar.md](historical-calendar.md)「三国十六国」。", "",
          "简文所书朔日（书「某干支朔」，或由日序与干支推得）与吴乾象历推步、魏历（张表）之朔比较。", "",
          "| 册 | 书页 | 简号 | 简文日期 | 所示朔 | 吴推步朔 | 合吴 | 魏历朔 | 合魏 | 分类 | 原文 | 备注 |", "|---|---|---|---|---|---|---|---|---|---|---|---|"]
    tot = {"n": 0, "wu": 0, "wei": 0, "only_wu": 0, "only_wei": 0}
    # 每月一个分类：推步 = 吴乾象推步，张表 = 魏历；吴简是当时官文书（B 级），一简即可判，同月简文冲突归③
    obs = {}
    for r in rows:
        obs.setdefault((int(r["年"]), int(r["月"]), r["闰"] == "1"), []).append(r["朔干支"])
    g_ = lambda M, k: GZ[(M[0][k] + 49) % 60] if k in M[0] else ""
    cls = {k: fl.classify(g_(wu, k), g_(wei, k), v, "B", "简文") for k, v in obs.items()}
    cl = lambda k: (cls[k][0] + (f"（{cls[k][1]}）" if cls[k][1] else "")) if cls[k][0] else ""
    # 逐月所据之简与分类写回 data/calendar/zoumalou_months.csv：rust/core/examples/regime_benji_eval.rs 读入，并入分政权逐月表
    # （吴月之 B 级证据；同一批简、同一分类规则，分政权表不另判简文）
    slips = {}
    for r in rows:
        slips.setdefault((int(r["年"]), int(r["月"]), r["闰"] == "1"), []).append(f"{r['册']} {r['简号'] or r['书页']}")
    with open(ROOT / "data/calendar/zoumalou_months.csv", "w", encoding="utf-8", newline="") as f:
        f.write("# 走马楼吴简逐月（scripts/calendar_data/zoumalou_shuo_check.py check 生成，勿手改）：核对过之简按年月归并；吴推步朔＝推步政权朔日表「吴」，"
                "张表朔＝实历朔日表同名之月；简文朔、简以「；」分隔，一一对应；分类同 doc/calendar/zoumalou-checks.md。\n")
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["年", "月", "闰", "吴推步朔", "张表朔", "简文朔", "简", "分类", "注"])
        for k in sorted(obs):
            w.writerow([k[0], k[1], int(k[2]), g_(wu, k), g_(wei, k), "；".join(obs[k]), "；".join(slips[k]), cls[k][0] or "", cls[k][1]])
    for r in rows:
        y, m, leap = int(r["年"]), int(r["月"]), r["闰"] == "1"
        res = []
        for M, _ in (wu, wei):
            j = M.get((y, m, leap))
            res.append((GZ[(j + 49) % 60] if j else "—", j is not None and GZ[(j + 49) % 60] == r["朔干支"]))
        tot["n"] += 1
        tot["wu"] += res[0][1]
        tot["wei"] += res[1][1]
        tot["only_wu"] += res[0][1] and not res[1][1]
        tot["only_wei"] += res[1][1] and not res[0][1]
        ey = y - {v: k for k, v in ERA_NAME.items()}[r["年号"]] + 1
        date = f"{r['年号']}{'元' if ey == 1 else ey}年{'閏' if leap else ''}{m}月" + (f"{r['日']}日" if r["日"] else "") + r["干支"]
        md.append(f"| {r['册']} | {r['书页']} | {r['简号']} | {date} | {r['朔干支']} | {res[0][0]} | {'✓' if res[0][1] else '✗'} | {res[1][0]} | {'✓' if res[1][1] else '✗'} | {cl((y, m, leap))} | {r['原文']} | {r.get('备注') or ''} |")
    md += ["", f"共 {tot['n']} 条：合吴 {tot['wu']}，合魏 {tot['wei']}；只合吴 {tot['only_wu']}，只合魏 {tot['only_wei']}。", ""]
    # 按月归并：只列吴、魏朔不同之月（能判别之月）
    gz = lambda j: GZ[(j + 49) % 60]
    by = {}
    for r in rows:
        k = (int(r["年"]), int(r["月"]), r["闰"] == "1")
        if k not in wu[0] or k not in wei[0] or wu[0][k] == wei[0][k]:
            continue
        c = by.setdefault(k, [0, 0, 0])
        c[0 if r["朔干支"] == gz(wu[0][k]) else 1 if r["朔干支"] == gz(wei[0][k]) else 2] += 1
    md += ["## 按月归并（吴、魏朔不同之月）", "", "| 年 | 月 | 吴推步朔 | 魏历朔 | 合吴 | 合魏 | 皆不合 |", "|---|---|---|---|---|---|---|"]
    for (y, m, leap), c in sorted(by.items()):
        md.append(f"| {y} | {'閏' if leap else ''}{m} | {gz(wu[0][(y, m, leap)])} | {gz(wei[0][(y, m, leap)])} | {c[0]} | {c[1]} | {c[2]} |")
    md += ["", f"能判别之月 {len(by)} 个：合吴 {sum(c[0] for c in by.values())} 简，合魏 {sum(c[1] for c in by.values())} 简。", ""]
    cats = [(c, "B" if c != "无证" else "—") for c, _ in cls.values() if c]
    md += ["## 分类（按月）", "",
           "分类对象是吴乾象历推步（「推步」）与张表（此段张表即魏历，不代表吴历）：①= 简文合吴推步、不合张表（证张表不能代表吴历），"
           "②= 简文合张表（魏历）、不合吴推步（证吴推步误）。吴简是当时官文书（B 级），一简即可判，同月简文冲突归③；"
           "同月诸简各算一条。" + fl.RULE, "",
           f"有简之月 {len(cats)}：" + fl.headline(cats, "吴推步") + "。", ""] + fl.grade_table(cats) + [""]
    (ROOT / "doc/calendar/zoumalou-checks.md").write_text("\n".join(md) + "\n", encoding="utf-8")
    print(tot)


if __name__ == "__main__":
    candidates(sys.argv[2:]) if sys.argv[1] == "candidates" else check()
