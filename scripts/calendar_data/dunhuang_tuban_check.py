"""敦煌历日比较表诸件对原卷图版的逐格核对表。

输入：data/calendar/敦煌历日_比较表.csv（邓文宽 1996 附录一转录）；
      data/calendar/dunhuang_tuban_check.csv（伯希和诸件对《法藏敦煌西域文献》图版，人工读图）；
      data/calendar/dunhuang_tuban_check_stein.csv（斯坦因诸件对 IDP 彩色图版与《英藏敦煌文献》图版，人工读图）。
输出：doc/calendar/dunhuang-tuban-checks.md——每件比较表中敦煌曆有朔之格，标「原件/推算」与图版核对结果
（合 / 大小合 / 大小合、朔误书 / 未见 / 图版无历日 / 未核）。
月以敦煌月序为准（敦煌月序空则同行月序），记作「公元/月名」，与核对表的「未见月」「仅大小月」「误书月」诸列对应。
用法（仓库根）：python3 scripts/calendar_data/dunhuang_tuban_check.py
"""
import csv
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

IDP_COLL = "https://idp.bl.uk/collection/{}/"


def rows(path):
    return list(csv.DictReader(l for l in open(path, encoding="utf-8") if not l.startswith("#")))


def keys(s):
    return {x for x in (s or "").split(";") if x}


def check(table, docs, where, vol_col, tot):
    """逐件逐格核对；返回（总表行、分件节）。where(d) 给出该件图版出处一句，vol_col(d) 给出总表「出处」格。"""
    summary, sections = [], []
    for d in docs:
        y = int(d["公元"])
        unseen, size_only, miswritten = keys(d["未见月"]), keys(d.get("仅大小月")), keys(d.get("误书月"))
        cells = [r for r in table if r["文书"] == d["文书"] and int(r["公元"]) in (y, y + 1) and r["敦煌朔"]]
        seen = f"图版所见：{d['图版所见']}。" if d["图版所见"] else ""
        sec = [f"### {d['文书']}（{y}）", "", where(d) + seen + (f"{d['备注']}。" if d["备注"] else ""), "",
               "| 公元 | 敦煌月 | 大小 | 朔 | 类 | 图版 |", "|---|---|---|---|---|---|"]
        ok = 0
        for r in cells:
            month = r["敦煌月序"] or r["行月序"]
            if month == "上年閏十二月":
                month = "正月"
            key = f"{r['公元']}/{month}"
            kind = "推算" if r["敦煌朔"].startswith("(") else "原件"
            if d["结论"] == "无历日":
                st = "图版无历日"
            elif d["结论"] != "合":
                st = "未核"
            elif key in unseen:
                st = "未见"
            elif key in size_only:
                st = "大小合"
            elif key in miswritten:
                st = "大小合、朔误书"
            else:
                st = "合"
            tot[kind][0] += 1
            tot[kind][1] += st == "合"
            tot[kind][2] += st == "大小合"
            tot[kind][3] += st == "大小合、朔误书"
            ok += st == "合"
            sec.append(f"| {r['公元']} | {month} | {r['敦煌大小']} | {r['敦煌朔'].strip('()')} | {kind} | {st} |")
        sec.append("")
        sections += sec
        def mk(r):
            m = r["敦煌月序"] or r["行月序"]
            return f"{r['公元']}/{'正月' if m == '上年閏十二月' else m}"
        n_size = sum(1 for r in cells if mk(r) in size_only | miswritten) if d["结论"] == "合" else 0
        summary.append(f"| {d['文书']} | {y} | {vol_col(d)} | {len(cells)} | {ok} | {n_size} | {d['结论']} |")
    return summary, sections


def totals_line(tot):
    mis = tot["原件"][3] + tot["推算"][3]
    tail = (f"原件所书朔干支与比较表不同者 {mis} 格（月大小与逐日建除皆合比较表，是原件抄误，见逐件备注），"
            "此外图版与比较表不合者 0。" if mis else "图版与比较表不合者 0。")
    return (f"原件格 {tot['原件'][0]}，图版证合 {tot['原件'][1]}、仅证大小 {tot['原件'][2] + tot['原件'][3]}；"
            f"推算格 {tot['推算'][0]}，图版证合 {tot['推算'][1]}、仅证大小 {tot['推算'][2] + tot['推算'][3]}。" + tail)


def main():
    table = rows(ROOT / "data/calendar/敦煌历日_比较表.csv")
    pel = rows(ROOT / "data/calendar/dunhuang_tuban_check.csv")
    stein = rows(ROOT / "data/calendar/dunhuang_tuban_check_stein.csv")
    head = "| 文书 | 公元 | {} | 有朔之格 | 图版证合 | 仅证大小 | 结论 |"
    sep = "|---|---|---|---|---|---|---|"

    tot_p = {"原件": [0, 0, 0, 0], "推算": [0, 0, 0, 0]}
    sum_p, sec_p = check(table, pel, lambda d: f"法藏第 {d['法藏册']} 册书页 {d['书页']}。", lambda d: d["法藏册"], tot_p)

    tot_s = {"原件": [0, 0, 0, 0], "推算": [0, 0, 0, 0]}

    def where_s(d):
        if d["IDP记录"]:
            return f"{d['IDP编号']}，[IDP]({IDP_COLL.format(d['IDP记录'])}) 图幅 {d['图幅']}。"
        if d.get("来源"):
            return f"{d['IDP编号']}，{d['来源']}。"
        return f"{d['IDP编号']}，图版未得。"

    def src_s(d):
        return "IDP" if d["IDP记录"] else (d.get("来源") or "").split("（")[0] or "—"

    sum_s, sec_s = check(table, stein, where_s, src_s, tot_s)

    md = ["# 敦煌历日图版核对（自动生成）", "",
          "> 由 `scripts/calendar_data/dunhuang_tuban_check.py` 生成，勿手改。底本与方法见 [dunhuang-calendar.md](dunhuang-calendar.md)「图版核对」。", "",
          "比较表（邓文宽 1996 附录一）诸件逐格对原卷图版：伯希和（P.）诸件对《法国国家图书馆藏敦煌西域文献》，"
          "斯坦因（S.）诸件对国际敦煌项目（IDP）彩色图版或《英藏敦煌文献（汉文佛经以外部份）》图版。"
          "「原件」= 比较表直书之格，「推算」= 比较表括号格；「图版」一栏：合 = 图版见朔日干支（或由图版月首、月末日辰及月大小表直接推得）与比较表合，"
          "大小合 = 图版只辨出月名大小、朔日干支字小不能辨，大小合、朔误书 = 原件所书朔干支与比较表不同而月大小、逐日建除皆合（原件抄误），"
          "未见 = 图版未见该月，图版无历日 = 该号图版无历日部分，未核 = 未得图版。", "",
          "## 伯希和诸件（法藏）", "", head.format("法藏册"), sep] + sum_p + ["", totals_line(tot_p), ""]
    md += ["## 斯坦因诸件（IDP、《英藏》）", "", head.format("图版"), sep] + sum_s + ["", totals_line(tot_s), ""]
    md += ["## 逐件（伯希和诸件）", ""] + sec_p + ["## 逐件（斯坦因诸件）", ""] + sec_s
    (ROOT / "doc/calendar/dunhuang-tuban-checks.md").write_text("\n".join(md) + "\n", encoding="utf-8")
    print("伯", tot_p, "斯", tot_s)


if __name__ == "__main__":
    main()
