"""楼兰、尼雅出土汉文简纸文书（魏晋前凉，兼收罗布淖尔土垠西汉简）中的历日：检验主序列推步与张表。

录文：data/calendar/loulan_jiri.csv。主底本侯灿、杨代欣《楼兰汉文简纸文书集成》（天地出版社 1999）；对校《中国简牍集成·二编》
第 20 册新疆维吾尔自治区卷（尼雅简只据此本）。两本整本 RapidOCR 识读后用 candidates 子命令抽候选，逐条对页图核对后入表。

每条：A「某月某干支朔」；B「某月某日 + 干支」（朔 = 干支 − (日 − 1)，表中朔干支留空由脚本求）；C「某月 + 干支」（只检是否在月内）。
定年为「推」（据历日推定年或据他简补出）者列出而不作证据。
比较：data/calendar/main_series_reckoning_months.csv 的推步朔与张表朔（魏晋为景初历段，土垠西汉为太初/三统历段）。
前凉奉晋正朔、沿用建兴年号，史无自造历法之明文，所用当即晋景初历，亦与景初推步、张表比较。
分类按 shiwu_fenlei（每月一个分类）；文书等级 B，历注习书 A。输出 doc/calendar/loulan-checks.md。
用法（仓库根）：uv run --project scripts python -I scripts/calendar_data/loulan_jiri_check.py candidates <ocr.json>...
            uv run --project scripts python -I scripts/calendar_data/loulan_jiri_check.py
"""
import csv
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))
import shiwu_fenlei as fl  # noqa: E402

GZ = fl.GZ
CSV = ROOT / "data/calendar/loulan_jiri.csv"
OUT = ROOT / "doc/calendar/loulan-checks.md"
G = "[甲乙丙丁戊己庚辛壬癸][子丑寅卯辰巳午未申酉戌亥]"
N = "[元正一二三四五六七八九十廿卅]+"
PA = re.compile(rf"(?:({N})年)?(閏|闰)?({N})月({G})朔")
PB = re.compile(rf"(?:({N})年)?(閏|闰)?({N})月({N})日({G})")
PC = re.compile(rf"(?:({N})年)?(閏|闰)?({N})月({G})")


def candidates(paths):
    """OCR JSON（页 → [(x, y, 文字, 置信)]）中的历日候选：页、类、匹配、行文。"""
    for p in paths:
        data = json.load(open(p))
        for pg, boxes in sorted(data.items(), key=lambda kv: int(kv[0])):
            for _, _, t, _ in boxes:
                seen = set()
                for kind, pat in (("A", PA), ("B", PB), ("C", PC)):
                    for m in pat.finditer(t):
                        if any(m.start() < e and s < m.end() for s, e in seen):
                            continue
                        seen.add((m.start(), m.end()))
                        print(f"PDF{int(pg) + 1}\t{kind}\t{m.group(0)}\t{t}")


def reckoning():
    """{(年, 月, 闰): (推步朔, 张表朔, 历)}：主序列推步（土垠西汉简为太初/三统历段，魏晋为景初历段）。"""
    out = {}
    for r in csv.DictReader(open(ROOT / "data/calendar/main_series_reckoning_months.csv", encoding="utf-8")):
        leap = r["月"].startswith("闰")
        out[(int(r["年"]), int(r["月"].lstrip("闰")), leap)] = (r["推步朔"], r["张表朔"], r["历"])
    return out


def month_days(tab, k):
    """该月所含干支（按下月朔求月长）。"""
    keys = sorted(tab, key=lambda x: (x[0], x[1] + (0.5 if x[2] else 0)))
    i = keys.index(k)
    a = GZ.index(tab[k][0])
    n = (GZ.index(tab[keys[i + 1]][0]) - a) % 60 if i + 1 < len(keys) else 30
    return {GZ[(a + j) % 60] for j in range(n)}


def check():
    tab = reckoning()
    rows = [r for r in csv.DictReader(l for l in open(CSV, encoding="utf-8") if not l.startswith("#")) if r["核对"] == "1"]
    obs, grade = {}, {}
    for r in rows:
        if r["类"] == "B" and not r["朔干支"]:
            r["朔干支"] = GZ[(GZ.index(r["干支"]) - int(r["日"]) + 1) % 60]
        k = (int(r["年"]), int(r["月"]), r["闰"] == "1")
        if r["类"] not in "AB":
            continue
        obs.setdefault(k, [])
        if r["定年"] != "推":
            obs[k].append(r["朔干支"])
            grade[k] = min(grade.get(k, "B"), r["等级"] or "B")
    cls = {k: fl.classify(tab.get(k, ("", ""))[0], tab.get(k, ("", ""))[1], v, grade.get(k, "B"), "文书") for k, v in obs.items()}
    md = ["# 楼兰、尼雅简纸文书历日与中原历（自动生成）", "",
          "> 由 `scripts/calendar_data/loulan_jiri_check.py` 生成，勿手改。录文 `data/calendar/loulan_jiri.csv`。", "",
          "## 底本与方法", "",
          "- 主底本：侯灿、杨代欣《楼兰汉文简纸文书集成》（天地出版社 1999），按出土地点（L.A.、L.E.、土垠等）编排，"
          "释文以发掘整理者（孔好古、沙畹、马伯乐、黄文弼、1980 年新疆楼兰考古队）为本、汇校王国维、林梅村、孟凡人诸家；"
          "对校本《中国简牍集成·二编》第 20 册新疆维吾尔自治区卷（敦煌文艺出版社），尼雅简只见于后者。"
          "两本整本 RapidOCR 识读后抽「某月某干支朔」（A）、「某月某日 + 干支」（B，朔 = 干支 −（日 − 1））、「某月 + 干支」"
          "（C，只检是否在月内）候选，逐条对页图核对；同简异本取发掘整理者释文，异文记备注，只算一条。",
          "- 定年：纪年书于简者为证；年号首字残、整理者据文例补者亦作证；年或年月由历日推定（马伯乐、黄文弼据长历）、"
          "或据他简补出者，列出而不作证据（避免以历证历）。",
          "- 比较：主序列推步（`data/calendar/main_series_reckoning_months.csv`，即 lifa 主序列缺省所取）与张培瑜表。"
          "魏晋简为景初历段（237—444；两家此段只 278 年三月、314 年正月与三月、430 年二月四月不同，楼兰简所及 264—330 前后皆两家相同之月），"
          "土垠西汉简为太初（三统）历段（两家全同）。",
          "- 前凉：张氏奉晋正朔，愍帝亡（316）后仍用建兴年号（楼兰沙木886「建兴十八年」= 330，无日干支，不入表），"
          "史无前凉自造历法之明文（河西自造之历始于北凉赵𢾺玄始历，412 年行用），所用当即晋景初历，故前凉纪年亦与景初推步、张表比较。"
          "沙木887「丙申朔七日壬寅大将军右长史关」无年月（侯灿据张骏官号定 324—346 年间，丙申朔之月有 325 年四月、330 年七月、338 年八月、"
          "340 年十一月、345 年十二月），不能定月，不入表。",
          "- 分类：" + fl.RULE + "楼兰、尼雅、土垠文书为当时官私文书，证据等级 B（一条即可判）；历注习书为 A。", "",
          "## 逐条", "",
          "| # | 出处 | 简号 | 纪年 | 公元 | 月 | 类 | 定年 | 原文 | 记朔 | 推步朔 | 张表朔 | 合推步 | 合张表 | 分类（该月） | 备注 |",
          "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|"]
    ok = {"n": 0, "t": 0, "z": 0}
    for r in rows:
        k = (int(r["年"]), int(r["月"]), r["闰"] == "1")
        t, z, li = tab.get(k, ("—", "—", "—"))
        mo = ("閏" if k[2] else "") + str(k[1])
        if r["类"] in "AB":
            a, b = r["朔干支"] == t, r["朔干支"] == z
            c = (cls[k][0] or "") + (f"（{cls[k][1]}）" if cls[k][1] else "")
            if r["定年"] == "推":
                c += "；本条推定年，不计"
            rec = r["朔干支"] + ("朔" if r["类"] == "A" else f"（{r['日']}日{r['干支']}）")
        else:
            a = k in tab and r["干支"] in month_days({kk: v[:1] for kk, v in tab.items()}, k)
            b = k in tab and r["干支"] in month_days({kk: v[1:2] for kk, v in tab.items()}, k)
            c, rec = "不作朔证", r["干支"] + "（在月内否）"
        ok["n"] += 1
        ok["t"] += a
        ok["z"] += b
        src = f"{r['出处']} {r['书页']}" + (f"；{r['对校']}" if r["对校"] else "")
        md.append(f"| {r['编号']} | {src} | {r['简号']} | {(r['年号'] + r['年序'] + '年') if r['年号'] else '（无纪年）'} | {k[0]} | {mo} | {r['类']} | {r['定年']} | {r['原文']} | {rec} | "
                  f"{t} | {z} | {'✓' if a else '✗'} | {'✓' if b else '✗'} | {c} | {r['备注']} |")
    cats = [(c, "—" if c == "无证" or (c == "③" and not obs[k]) else grade.get(k, "B")) for k, (c, _) in cls.items() if c]
    md += ["", f"共 {ok['n']} 条：合推步 {ok['t']}，合张表 {ok['z']}（推步与张表在诸条所在之月全同）。", "",
           "## 分类（按月）", "", f"有朔之月 {len(cats)}：" + fl.headline(cats, "推步") + "。", ""] + fl.grade_table(cats) + ["",
           "## 讨论", "",
           "- 楼兰魏晋简所书朔日、日辰与景初推步、张表（两家此段全同）合者为多：咸熙二年七月癸丑朔、泰始二年八月十日丙辰、"
           "泰始三年二月廿八日辛未、泰始四年四月十六日壬子、泰始五年五月一日辛卯、六月庚申朔、十月戊午朔（尼雅）、永嘉四年八月十九日己酉，"
           "说明西域长史府所用即中原颁行之景初历。",
           "- 不合三条：80LBT:017「泰始四年四月六日壬戌」（推得丁巳朔，差 20 日，同月马木221 合两家，当是日序或干支书误；"
           "侯灿以为楼兰未及时得中原告朔、自行推算之误，然同月他简合中原历，此说难立）；沙木725「泰始四年闰月六日己巳」"
           "（推得甲子朔，两家闰八月乙丑朔，差一日，或六日为五日之误）；沙木727「泰始五年正月（或释四月）廿日壬申」（两读皆不合）。",
           "- 土垠西汉简（黄文弼所获）永光五年七月、河平四年十一月之朔合三统历推步与张表；元延五年（即绥和元年）二月甲辰朔，"
           "两家是年二月乙巳朔、四月甲辰朔，差一日，黄文弼已疑「必有一误」，本条一见，判同误待核。", ""]
    OUT.write_text("\n".join(md) + "\n", encoding="utf-8")
    print(ok, fl.headline(cats, "推步"))


if __name__ == "__main__":
    candidates(sys.argv[2:]) if len(sys.argv) > 1 and sys.argv[1] == "candidates" else check()
