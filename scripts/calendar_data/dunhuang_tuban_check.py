"""敦煌历日比较表伯希和诸件对《法藏敦煌西域文献》图版的逐格核对表。

输入：data/calendar/敦煌历日_比较表.csv（邓文宽 1996 附录一转录）、data/calendar/dunhuang_tuban_check.csv（逐件图版所见，人工读图）。
输出：doc/calendar/dunhuang-tuban-checks.md——每件比较表中敦煌曆有朔之格，标「原件/推算」与图版核对结果（合 / 未见 / 未核）。
月以敦煌月序为准（敦煌月序空则同行月序），记作「公元/月名」，与 dunhuang_tuban_check.csv 的「未见月」一列对应。
用法（仓库根）：python3 scripts/calendar_data/dunhuang_tuban_check.py
"""
import csv
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def rows(path):
    return list(csv.DictReader(l for l in open(path, encoding="utf-8") if not l.startswith("#")))


def main():
    table = rows(ROOT / "data/calendar/敦煌历日_比较表.csv")
    docs = rows(ROOT / "data/calendar/dunhuang_tuban_check.csv")
    md = ["# 敦煌历日图版核对（自动生成）", "",
          "> 由 `scripts/calendar_data/dunhuang_tuban_check.py` 生成，勿手改。底本与方法见 [dunhuang-calendar.md](dunhuang-calendar.md)「图版核对」。", "",
          "比较表（邓文宽 1996 附录一）伯希和诸件，逐格对《法国国家图书馆藏敦煌西域文献》图版。「原件」= 比较表直书之格，「推算」= 比较表括号格。", ""]
    tot = {"原件": [0, 0], "推算": [0, 0]}
    summary = []
    for d in docs:
        y = int(d["公元"])
        unseen = {s for s in d["未见月"].split(";") if s}
        cells = [r for r in table if r["文书"] == d["文书"] and int(r["公元"]) in (y, y + 1) and r["敦煌朔"]]
        md += [f"## {d['文书']}（{y}）", "", f"法藏第 {d['法藏册']} 册书页 {d['书页']}。图版所见：{d['图版所见']}。" + (f"{d['备注']}。" if d["备注"] else ""), "",
               "| 公元 | 敦煌月 | 大小 | 朔 | 类 | 图版 |", "|---|---|---|---|---|---|"]
        ok = 0
        for r in cells:
            month = r["敦煌月序"] or r["行月序"]
            if month == "上年閏十二月":
                month = "正月"
            key = f"{r['公元']}/{month}"
            kind = "推算" if r["敦煌朔"].startswith("(") else "原件"
            if d["结论"] != "合":
                st = "未核"
            elif key in unseen:
                st = "未见"
            else:
                st = "合"
            tot[kind][0] += 1
            tot[kind][1] += st == "合"
            ok += st == "合"
            md.append(f"| {r['公元']} | {month} | {r['敦煌大小']} | {r['敦煌朔'].strip('()')} | {kind} | {st} |")
        md.append("")
        summary.append(f"| {d['文书']} | {y} | {d['法藏册']} | {len(cells)} | {ok} | {d['结论']} |")
    head = ["## 总表", "", "| 文书 | 公元 | 法藏册 | 有朔之格 | 图版证合 | 结论 |", "|---|---|---|---|---|---|"] + summary + [
        "", f"原件格 {tot['原件'][0]}，图版证合 {tot['原件'][1]}；推算格 {tot['推算'][0]}，图版证合 {tot['推算'][1]}。图版与比较表不合者 0。", ""]
    md[6:6] = head
    (ROOT / "doc/calendar/dunhuang-tuban-checks.md").write_text("\n".join(md) + "\n", encoding="utf-8")
    print(tot)


if __name__ == "__main__":
    main()
