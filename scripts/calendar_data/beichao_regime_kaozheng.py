"""北朝诸国（北魏/西魏/北周/隋 598 年以前、东魏/北齐）分政权推步之史书逐条考证表。

输入：data/calendar/regime_benji_check.csv（rust/core/examples/regime_benji_eval.rs 逐条结果，推步含史载修正）、
data/calendar/本纪纪日/*.csv（叙事顺序）、data/calendar/推步政权朔日表.csv（纯推步月首）。
输出：data/calendar/beichao_regime_kaozheng.csv——推步不合之条、只合推步与只合张表之条，各附类别与考证；
另附抽取校改与史载修正之条（类「抽取校改」「史载修正」）。

类别（自动）：
- 只合推步：本国史书合本国推步、不合中原张表——本国另行一历之证；
- 只合张表：反之；
- 两家皆不合·略书次月：按叙事顺序（自上一条已定之日起找最近的同干支日），此日落在所记之月的下一月，即本纪月份更替不书；
  推步与张表此月月首相同者不判历法；
- 两家皆不合·月首前一二日：干支在所记之月月首前 1—2 日（历日问题之候选，逐条考）；
- 两家皆不合·其他：干支或月份传写之误、追叙等，两家月首相同者不判历法。
人工考证见 NOTES（据《魏书》《北史》《周书》《隋书》原文及天象志互校）。

用法（worktree/仓库根）：python3 scripts/calendar_data/beichao_regime_kaozheng.py
"""
import bisect
import csv
import glob
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
G = ["甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12] for i in range(60)]
KEY = {"北魏": "北魏/西魏/北周/隋", "西魏": "北魏/西魏/北周/隋", "北周": "北魏/西魏/北周/隋", "隋": "北魏/西魏/北周/隋",
       "东魏": "东魏/北齐", "北齐": "东魏/北齐"}
BOOK = {"weishu_benji": "魏书", "beishi_benji": "北史", "beiqishu_benji": "北齐书", "zhoushu_benji": "周书", "suishu_benji": "隋书"}

# (书, 年, 月, 干支) → 考证
NOTES = {
    ("zhoushu_benji", "567", "7", "辛丑"): "天和二年闰六：《隋书·天文志下》「閏六月丁酉」、本纪「閏月庚午」在六月后，七月四日皆要求七月己亥朔；天和历推步闰八，史载例外表置闰（他书）",
    ("zhoushu_benji", "567", "7", "甲辰"): "同上",
    ("zhoushu_benji", "567", "7", "庚戌"): "同上；《隋书·天文志》同日「太白犯軒轅」",
    ("zhoushu_benji", "567", "7", "壬子"): "同上",
    ("zhoushu_benji", "571", "1", "己酉"): "天和六年「正月己酉朔，廢朝」明书（非日食之条），推步庚戌；真合朔己酉 14 时（长安）；天和二年正月癸酉朔、六年四月戊寅朔（《隋书·天文志》）推步皆合；史载例外表退一日（本纪）",
    ("zhoushu_benji", "580", "1", "丁亥"): "大象二年正月丁亥在推步戊子朔前一日；真合朔丁亥 13 时；史载例外表退一日（本纪）",
    ("zhoushu_benji", "580", "7", "甲申"): "大象二年七月甲申在推步乙酉朔前一日；真合朔甲申 21 时；史载例外表退一日（本纪）",
    ("suishu_benji", "583", "2", "己巳"): "开皇三年「二月己巳朔，日有蝕之」，推步庚午、张表己巳；真合朔庚午 7 时，所记反早一日。日食之条，平朔时代史家常以食日书朔，「朔」不作历日证据（beichao_shuo_kaozheng.csv 2026-11-18 考），不改",
    ("suishu_benji", "583", "7", "丁卯"): "「秋七月……丁卯，日有蝕之。八月丁丑」：丁卯书于七月，若作历日则七月须戊戌朔（推步、张表皆丁酉，丁卯为八月朔）；维基文库本注「隋曆後陳曆一日，致以八月朔為七月晦」。同属日食之条，不作历日证据，存疑不改",
    ("beishi_benji", "583", "7", "丁卯"): "同《隋书》开皇三年七月丁卯日食，存疑不改",
    ("beishi_benji", "583", "2", "己巳"): "同《隋书》开皇三年二月己巳朔日食，不作历日证据",
    ("beishi_benji", "583", "12", "乙卯"): "《北史》脱「閏」：《隋书·高祖纪上》作「閏十二月乙卯，遣兼散騎常侍曹令則」，合推步（大象历闰十二）；《北史》书「十二月」，遂只合张表（闰十一）",
    ("beishi_benji", "583", "12", "戊午"): "同上（《隋书》闰十二月戊午）",
    ("beishi_benji", "485", "7", "丙午"): "《北史》讹：《魏书·高祖纪上》太和九年「秋七月丙寅朔，新作諸門」，合推步；「午」「寅」形近",
    ("beishi_benji", "498", "10", "乙酉"): "《北史》讹：《魏书·高祖纪下》太和二十二年「冬十月己酉朔」，合推步；「乙」「己」形近",
    ("beishi_benji", "408", "7", "戊戌"): "日食之条（《魏书·天象志一》同作天赐五年七月戊戌朔）：戊戌距推步、张表朔日（癸巳）五日，非历日之差，系年或干支有误，不判历法",
    ("beishi_benji", "494", "7", "甲辰"): "《北史》略书月份：《魏书》此事在「八月甲辰」",
    ("beishi_benji", "494", "7", "丁未"): "同上（《魏书》八月丁未）",
    ("beishi_benji", "497", "2", "乙未"): "《北史》略书月份：《魏书》作「三月乙未，車駕南巡」",
    ("beishi_benji", "497", "2", "丙辰"): "同上（《魏书》三月丙辰次平陽）",
    ("beishi_benji", "480", "8", "乙酉"): "《北史》讹：《魏书·高祖纪上》太和四年作「八月乙卯，詔諸州置冰室」，合推步（闰七）；「卯」「酉」形近，合张表（闰九）是偶合",
    ("suishu_benji", "583", "11", "甲午"): "略书十二月：「十一月己酉……庚辰……甲午，罷天下諸郡。閏十二月乙卯」，庚辰、甲午按推步在十二月（乙丑朔），下接閏十二月，本纪不书「十二月」",
    ("beishi_benji", "583", "11", "甲午"): "同《隋书》，略书十二月",
    ("beishi_benji", "442", "8", "甲戌"): "「秋八月甲戌晦，日有蝕之」：推步、张表皆八月乙亥朔，甲戌为七月晦，月份有误，不判历法",
    ("beishi_benji", "472", "6", "戊午"): "《北史》略书「閏月」：《魏书》此「戊午，行幸陰山」在閏月",
    ("beiqishu_benji", "575", "8", "己丑"): "（閏月表）武平六年「閏月己丑」：推步、张表皆闰八而己丑不在其中，存疑",
}
EXTRA = [
    ("抽取校改", "魏书", "428—431", "", "", "《魏书·世祖纪上》年号作「神䴥」，年号表作「神麚」，异体未对上，神䴥元年至四年 51 条承始光作 427—430；补异体（zhi_star_records.EXTRA_VARIANTS），北魏合推步 2397→2434"),
    ("抽取校改", "魏书", "531", "", "", "《魏书·前废帝纪》卷首传记末书「建義初」，纪文「春二月己巳，曄進至邙南」不书年，其后普泰元年诸日承「建義」作 528 年；校改补「建明二年」"),
    ("抽取校改", "北史", "428—431", "", "", "《北史·魏本纪二》「辛未，京兆王黎薨。二月，改元」不书年月、年号，其后诸年承始光；据《魏书》补「神䴥元年春正月」"),
    ("抽取校改", "北史", "554—556", "", "", "《北史·周本纪上》书「恭帝元年四月」（无「魏」字），认作西魏恭帝纪年（原承大统作 537）"),
    ("史载修正", "周书、隋书天文志", "562", "闰正", "辛未", "保定二年「閏月己丑……癸巳……己亥……二月壬寅……癸丑」要求闰在正月，《隋书·天文志》「二月壬寅，熒惑犯太微上相」同；正光历代推（明克让周历术不传）闰二，张表亦闰二；例外表置闰（他书）"),
    ("史载修正", "周书", "564", "闰九", "丙戌", "保定四年「九月丁巳……閏月己亥……冬十月癸亥……甲子……丁卯……癸酉」要求闰在九月；代推闰十，张表亦闰十；例外表置闰（本纪）"),
    ("史载修正", "隋书天文志", "567", "闰六", "庚午", "天和二年「閏六月丁酉」（《隋书·天文志下》）；例外表置闰（他书）"),
]


def jj(y, m, d):
    a = (14 - m) // 12
    yy, mm = y + 4800 - a, m + 12 * a - 3
    return d + (153 * mm + 2) // 5 + 365 * yy + yy // 4 - 32083


def main():
    reg = {}
    for r in csv.DictReader(open(ROOT / "data/calendar/推步政权朔日表.csv", encoding="utf-8")):
        if r["regime"] in KEY.values():
            reg.setdefault(r["regime"], []).append((jj(int(r["gregorian_year"]), int(r["gregorian_month"]), int(r["gregorian_day"])),
                                                    int(r["lunar_year"]), int(r["lunar_month"]), r["is_leap"] == "1"))
    for v in reg.values():
        v.sort()
    starts = {k: [e[0] for e in v] for k, v in reg.items()}
    check = {}
    for r in csv.DictReader(open(ROOT / "data/calendar/regime_benji_check.csv", encoding="utf-8"), quoting=csv.QUOTE_NONE):
        if r["政权"] in KEY and r["书"] in BOOK:
            check.setdefault((r["书"], r["年"], r["月"], r["干支"], r["原文"][:20]), r)
    # 叙事顺序分类（纯推步月首）
    narr = {}
    for path in sorted(glob.glob(str(ROOT / "data/calendar/本纪纪日/*.csv"))):
        book = Path(path).stem
        if "閏月" in path or book not in BOOK:
            continue
        prev = None
        for r in csv.DictReader(open(path, encoding="utf-8"), quoting=csv.QUOTE_NONE):
            if r["regime"] not in KEY or not r["month"].lstrip("闰").isdigit() or r["ganzhi"] not in G:
                continue
            L, S = reg[KEY[r["regime"]]], starts[KEY[r["regime"]]]
            y, m, lp = int(r["year"]), int(r["month"].lstrip("闰")), r["leap"] == "1"
            idx = [i for i, e in enumerate(L) if e[1] == y and e[2] == m and e[3] == lp]
            if not idx or idx[0] + 1 >= len(L):
                continue
            i = idx[0]
            a, b = L[i][0], L[i + 1][0]
            gi = G.index(r["ganzhi"])
            d_in = a + (gi - (a + 49)) % 60
            key = (book, r["year"], ("闰" if lp else "") + str(m), r["ganzhi"], (r["text"] or "").replace(",", "，")[:20])
            if d_in < b:
                prev = d_in
                continue
            base = prev if prev is not None and prev > a - 60 else a
            d = base + (gi - (base + 49)) % 60
            j = bisect.bisect_right(S, d) - 1
            early = a - (d_in - 60)
            cat = "月首前一二日" if 1 <= early <= 2 else "略书次月" if j == i + 1 else "其他"
            narr[key] = cat
            if j >= i:
                prev = d
    out = [["类", "书", "政权", "年", "月", "干支", "位置", "推步月首", "合推步", "张表月首", "合张表", "考证", "原文"]]
    for key, r in check.items():
        okr, okt = r["推步合"] == "1", r["张表合"] == "1"
        if okr and okt:
            continue
        cat = "只合推步" if okr else "只合张表" if okt else "两家皆不合·" + narr.get(key, "其他")
        note = NOTES.get((r["书"], r["年"], r["月"], r["干支"]), "")
        if not note and cat.startswith("两家皆不合") and r["推步月首"] == r["张表月首"]:
            note = "推步与张表此月月首相同，不判历法"
        out.append([cat, BOOK[r["书"]], r["政权"], r["年"], r["月"], r["干支"], r["位置"], r["推步月首"], r["推步合"],
                    r["张表月首"], r["张表合"], note, r["原文"]])
    for typ, book, y, m, gz, note in EXTRA:
        out.append([typ, book, "", y, m, gz, "", "", "", "", "", note, ""])
    order = {"只合张表": 0, "两家皆不合·月首前一二日": 1, "两家皆不合·其他": 2, "两家皆不合·略书次月": 3, "只合推步": 4, "抽取校改": 5, "史载修正": 6}
    body = sorted(out[1:], key=lambda x: (order.get(x[0], 9), x[1], x[3]))
    dst = ROOT / "data/calendar/beichao_regime_kaozheng.csv"
    with open(dst, "w", encoding="utf-8", newline="") as f:
        f.write("# 北朝诸国分政权推步之史书逐条考证（scripts/calendar_data/beichao_regime_kaozheng.py 生成）。推步含史载修正（dingshuo_exceptions.csv 历名「政权：历名」）。\n")
        csv.writer(f).writerows([out[0]] + body)
    from collections import Counter
    print(dst.name, len(body), Counter((x[0], x[1]) for x in body))


if __name__ == "__main__":
    main()
