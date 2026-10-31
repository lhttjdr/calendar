"""由 verify_main_table.py、scripts/phenomena/main_table_shuo_sky.py、daming_dingshuo.py、liao_calendar_check.py 等的输出生成 doc/calendar/main-table-checks.md（逐条，勿手改）。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/main_table_checks_to_md.py
"""
import csv
import re
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DYNS = ("三国", "晋", "南朝", "北朝", "隋", "唐", "五代", "宋", "辽", "金", "元", "明")
BOOK = {"sanguozhi": "三国志", "jinshu_benji": "晋书", "songshu_benji": "宋书", "nanqishu_benji": "南齐书", "liangshu_benji": "梁书",
        "chenshu_benji": "陈书", "weishu_benji": "魏书", "beiqishu_benji": "北齐书", "zhoushu_benji": "周书", "suishu_benji": "隋书",
        "jiutangshu_benji": "旧唐书", "xintangshu_benji": "新唐书",
        "jiuwudaishi_benji": "旧五代史", "xinwudaishi_benji": "新五代史", "songshi_benji": "宋史", "liaoshi_benji": "辽史",
        "jinshi_benji": "金史", "yuanshi_benji": "元史", "mingshi_benji": "明史"}


def rows(name):
    p = ROOT / "data/calendar" / name
    if not p.exists():
        return []
    with open(p, encoding="utf-8") as f:
        return list(csv.DictReader(line for line in f if not line.startswith("#")))


def table(head, body):
    out = ["| " + " | ".join(head) + " |", "|" + "---|" * len(head)]
    # 原文偶带维基模板残片（「{{Textquality|50」），留着 VitePress 会当 Vue 表达式解析
    out += ["| " + " | ".join(re.sub(r"\{\{[^}]*\}*", "", str(c)).replace("|", "／") for c in r) + " |" for r in body]
    return out


def book(b):
    return "、".join(BOOK.get(x, x) for x in b.split("、")) if b else ""


def main():
    chk, leaps, sky, base = rows("main_table_check.csv"), rows("main_table_leaps.csv"), rows("main_table_shuo_sky.csv"), \
        rows("main_table_shuo_baseline.csv")
    sky_by = {(s["书"], s["年"], s["月"], s["本纪朔"]): s for s in sky}
    L = ["# 以本纪核验主表：逐条结果", "",
         "由 `scripts/calendar_data/verify_main_table.py`、`scripts/phenomena/main_table_shuo_sky.py` 计算，"
         "`scripts/calendar_data/main_table_checks_to_md.py` 生成，**勿手改**。方法与结论见 "
         "[historical-calendar.md](./historical-calendar.md)「以本纪核验主表」；逐条数据见 `data/calendar/main_table_*.csv`，"
         "本纪纪日由 `scripts/calendar_data/qinhan_eval/zhi_star_records.py` 的纪日模式（`*_benji` 配置）抽取，见 `data/calendar/本纪纪日/`。", ""]
    # 一、明书朔日
    st = defaultdict(lambda: defaultdict(int))
    for r in chk:
        s = st[r["朝代"]]
        s["n"] += 1
        s["ok"] += r["主表"] == "合"
        s[r["类"]] += 1
        if r["本项目推步"]:
            s["pn"] += 1
            s["pok"] += r["本项目推步"] == "合"
    L += ["## 一、明书朔日（本纪「某月某干支朔」）", "",
          "不合者分三类：**朔差一二日**（朔日推算之差，主表或本纪有一误）、**差一月**（所书干支为主表邻月或闰月之朔，闰位或月序不同）、"
          "**其他**（多为干支传写之误或解析误定年月，不作主表证据）。北朝、隋另列本项目推步表。", ""]
    L += table(["朝代", "条数", "与主表合", "朔差一二日", "差一月", "其他", "本项目推步合"],
               [[d, st[d]["n"], st[d]["ok"], st[d]["朔差一二日"], st[d]["差一月"], st[d]["其他"],
                 f"{st[d]['pok']}/{st[d]['pn']}" if st[d]["pn"] else ""] for d in DYNS if st[d]["n"]])
    L += ["", "### 朔差一二日：实际合朔（第 6 级，只作旁证）", "",
          "合朔为 DE441 视合朔换算为都城地方平时；「日差」= 合朔所在日 − 本纪朔日。日食记录在平朔时代常把食日（即合朔日）书为朔，"
          "不作历日证据（见 historical-calendar.md）。", ""]
    body = []
    for r in chk:
        if r["类"] != "朔差一二日":
            continue
        s = sky_by.get((r["书"], r["年"], r["月"], r["本纪朔"]), {})
        body.append([BOOK.get(r["书"], r["书"]), r["政权"], r["年"], r["纪年"], r["月"], r["本纪朔"], r["主表"], r["本项目推步"], r["日食"],
                     s.get("合朔地方时日差（合朔日−本纪朔）", ""), s.get("合朔地方时", ""), s.get("判定", ""), book(r["他书同记"]),
                     r["旁证（主表/本纪朔）"], r["原文"]])
    L += table(["书", "政权", "年", "纪年", "月", "本纪朔", "主表", "本项目推步", "日食", "日差", "合朔地方时", "合朔所在（旁证）", "他书同记",
                "旁证（主表/本纪朔）", "原文"], body)
    for kind in ("差一月", "其他"):
        L += ["", f"### {kind}", "",
              "「旁证」为他书同月记日干支落在主表该月内者 / 落在以本纪所书朔起算 30 日内者（共几条）。", ""]
        L += table(["书", "政权", "年", "纪年", "月", "本纪朔", "主表", "本项目推步", "日食", "他书同记", "旁证（主表/本纪朔）", "原文"],
                   [[BOOK.get(r["书"], r["书"]), r["政权"], r["年"], r["纪年"], r["月"], r["本纪朔"], r["主表"], r["本项目推步"], r["日食"],
                     book(r["他书同记"]), r["旁证（主表/本纪朔）"], r["原文"]] for r in chk if r["类"] == kind])
    if base:
        L += ["", "### 相合者之朔日与实际合朔", "",
              "本纪明书朔日与主表相合者，合朔（都城地方平时）所在日 − 朔日之分布。−1 即朔在合朔次日；唐代此类合朔时刻几乎都在 16 时以后，为进朔。", ""]
        by = defaultdict(dict)
        for b in base:
            by[b["朝代"]][int(b["差"])] = int(b["次数"])
        L += table(["朝代", "−1", "0", "+1"], [[d, by[d].get(-1, 0), by[d].get(0, 0), by[d].get(1, 0)] for d in DYNS if d in by])
    # 二、闰位
    ls = defaultdict(lambda: defaultdict(int))
    for r in leaps:
        ls[r["朝代"]]["n"] += 1
        ls[r["朝代"]][r["主表"] if r["主表"] in ("合", "略书相容") else "矛盾"] += 1
    L += ["", "## 二、本纪「閏月」位置", "",
          "本纪「閏月」只保证在所记最后一月之后：主表闰位更晚为**略书相容**，更早或主表当年无闰为**矛盾**。"
          "北朝主表本非当时行用之历（北朝推步表闰位见 `data/calendar/leap_evidence.csv` 与 push-algorithms.md §3.4）。", ""]
    L += table(["朝代", "条数", "合", "略书相容", "矛盾"],
               [[d, ls[d]["n"], ls[d]["合"], ls[d]["略书相容"], ls[d]["矛盾"]] for d in DYNS if ls[d]["n"]])
    L += ["", "### 矛盾者", ""]
    L += table(["书", "朝代", "年", "纪年", "本纪闰位（不早于）", "主表", "原文"],
               [[BOOK.get(r["书"], r["书"]), r["朝代"], r["年"], r["纪年"], r["本纪闰位"], r["主表"], r["原文"]]
                for r in leaps if r["主表"] not in ("合", "略书相容")])
    # 三、敦煌历日（邓文宽 1996 附录一；check_dunhuang_comparison.py）
    dh = rows("dunhuang_vs_main.csv")
    if dh:
        L += ["", "## 三、敦煌历日与主表（邓文宽 1996 附录一）", "",
              "敦煌出土具注历日为当地自造之历（吐蕃、归义军时期），是独立历表，不据以改主表。转录 `data/calendar/敦煌历日_比较表.csv`"
              "（中原曆一栏据陈垣，与主表 447 格全合），逐月比较见 `data/calendar/dunhuang_vs_main.csv`。下表只计原文有据之月"
              "（不含原书括号推算之格）；「日差」= 敦煌朔 − 主表朔。", ""]
        per = defaultdict(lambda: defaultdict(int))
        for r in dh:
            if r["推算"]:
                continue
            k = (r["公元"], r["纪年"], r["文书"], r["时期"])
            per[k]["n"] += 1
            per[k][int(r["日差"])] += 1
        body = []
        for (y, era, doc, p), c in sorted(per.items(), key=lambda kv: int(kv[0][0])):
            body.append([y, era, doc, p, c["n"], c[0], c[-2] + c[-1], c[1] + c[2]])
        L += table(["公元", "纪年", "文书", "时期", "有据之月", "与主表同朔", "敦早（1—2 日）", "敦迟（1—2 日）"], body)
        L += ["", "### 敦煌朔与主表不同之月（有据者）", ""]
        L += table(["公元", "纪年", "文书", "主表月", "敦煌月", "敦煌朔", "主表朔", "日差", "书页"],
                   [[r["公元"], r["纪年"], r["文书"], r["主表月"], r["敦煌月"], r["敦煌朔"], r["主表朔"], r["日差"], r["书页"]]
                    for r in dh if not r["推算"] and r["日差"] != "0"])
    # 四、金大明历（daming_dingshuo.py）
    jn = rows("daming_dingshuo_check.csv")
    if jn:
        L += ["", "## 四、金大明历定朔与本纪明书朔日", "",
              "重修大明历定朔、进朔（`calendar::daming_li`）；1182 年起用术文原限，1182 年前杨级历以之代推，进朔限加经验改正 0.25 时"
              "（见 historical-calendar.md「金大明历」）。逐条见 `data/calendar/daming_dingshuo_check.csv`；下表列本纪不合主表、"
              "或不合金历、或临限（1182 年前超术文限 0—0.5 时）之条。", ""]
        n = len(jn)
        L += [f"共 {n} 条：合主表 {sum(int(r['本纪合主表']) for r in jn)}，合金历不进朔 {sum(int(r['本纪合大明不进朔']) for r in jn)}，"
              f"合术文进朔 {sum(int(r['本纪合术文进朔']) for r in jn)}，合缺省进朔 {sum(int(r['本纪合大明进朔']) for r in jn)}。", ""]
        L += table(["年", "纪年", "月", "本纪朔", "主表朔", "金定朔", "定朔小余（时）", "术文限（时）", "术文进朔", "缺省进朔", "日食", "原文"],
                   [[r["年"], r["纪年"], r["月"], r["本纪朔"], r["主表朔"], r["大明定朔"], r["定朔小余（时）"], r["术文进朔限（时）"],
                     r["术文进朔"], r["大明进朔（校准）"], r["日食"], r["原文"]]
                    for r in jn if r["临限"] or r["本纪合主表"] == "0" or r["本纪合大明进朔"] == "0"])
    # 五、辽历（liao_calendar_check.py）
    lc = rows("liao_calendar_check.csv")
    if lc:
        L += ["", "## 五、辽历：本纪明书朔日与宋历、祖冲之大明历、金大明历", "",
              "《辽史·历象志》所载「大明历」为祖冲之法（志自言「歷書法禁不可得……得祖沖之法於外史」），以本纪检之。"
              "祖冲之平朔与任何定朔历偶合约 68%（994—1125 年主表 1110/1632），辽本纪合平朔之率与之相当，合定朔则在 95% 以上："
              "辽行定朔之历，非祖冲之法。逐条见 `data/calendar/liao_calendar_check.csv`。", ""]
        per = defaultdict(lambda: [0, 0, 0, 0])
        for r in lc:
            c = per[r["时期"]]
            c[0] += 1
            c[1] += int(r["合宋"])
            c[2] += int(r["合祖冲之"])
            c[3] += int(r["合金大明"])
        L += table(["时期", "明书朔日", "合宋历（主表）", "合祖冲之平朔", "合金大明历上推"], [[p, *c] for p, c in per.items()])
        L += ["", "### 本纪不合宋历之条", ""]
        L += table(["年", "纪年", "月", "本纪朔", "宋历", "祖冲之平朔", "金定朔", "定朔小余（时）", "进朔限（时）", "金进朔", "原文"],
                   [[r["年"], r["纪年"], r["月"], r["本纪朔"], r["宋历（主表）"], r["祖冲之平朔"], r["金大明定朔"], r["定朔小余（时）"],
                     r["进朔限（时）"], r["金大明进朔"], r["原文"]] for r in lc if r["合宋"] == "0"])
    # 六、唐定朔诸历（linde_dingshuo.py、dayan_dingshuo.py、chongxuan_dingshuo.py）
    tang = [("戊寅历定朔 627—645", "wuyin_ding_check.csv", "戊寅定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("戊寅历平朔 646—664", "wuyin_ping_check.csv", "戊寅定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("麟德历 665—728", "linde_dingshuo_check.csv", "麟德定朔", "本纪合麟德进朔", "本纪合麟德无进朔", "麟德定朔小余（时）"),
            ("大衍历 729—761", "dayan_dingshuo_check.csv", "大衍定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("五纪历 762—783", "wuji_dingshuo_check.csv", "五纪定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("正元历 784—806", "zhengyuan_dingshuo_check.csv", "正元定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("宣明历 822—892", "xuanming_dingshuo_check.csv", "宣明定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("崇玄历 893—938", "chongxuan_dingshuo_check.csv", "崇玄定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("崇天历 1024—1064（《宋史》）", "song_崇天_check.csv", "崇天定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("纪元历 1106—1135（《宋史》）", "song_纪元_check.csv", "纪元定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("明天历 1065—1074（《宋史》）", "song_明天_check.csv", "明天定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("观天历 1094—1102（《宋史》）", "song_观天_check.csv", "观天定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("统元历 1136—1167（《宋史》）", "song_统元_check.csv", "统元定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("乾道历 1168—1176（《宋史》）", "song_乾道_check.csv", "乾道定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("淳熙历 1177—1190（《宋史》）", "song_淳熙_check.csv", "淳熙定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("会元历 1191—1198（《宋史》）", "song_会元_check.csv", "会元定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("统天历 1199—1207（《宋史》）", "song_统天_check.csv", "统天定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("开禧历 1208—1251（《宋史》）", "song_开禧_check.csv", "开禧定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）"),
            ("成天历 1271—1276（《宋史》）", "song_成天_check.csv", "成天定朔", "本纪合进朔", "本纪合不进朔", "定朔小余（时）")]
    L += ["", "## 六、唐宋定朔诸历与本纪明书朔日", "", "术数见 [calendar-compendium.md](./calendar-compendium.md)。下表列本纪不合主表、或不合进朔限之条（限：麟德试用 18 时，崇玄经验 18 时（术文限不合本纪），大衍 19 时，宣明、五纪、正元据志文，崇天日入分，纪元 3/4 日法并春分后减限）。"]
    for title, name, col, jin_, bu, hcol in tang:
        rs = rows(name)
        L += ["", f"### {title}", "", f"共 {len(rs)} 条：合主表 {sum(int(r['本纪合主表']) for r in rs)}，合进朔 {sum(int(r[jin_]) for r in rs)}，"
              f"合不进朔 {sum(int(r[bu]) for r in rs)}。逐条见 `data/calendar/{name}`。", ""]
        L += table(["书", "年", "纪年", "月", "本纪朔", "主表朔", "术推朔", "小余（时）", "日食", "原文"],
                   [[BOOK.get(r["书"], r["书"]), r["年"], r["纪年"], r["月"], r["本纪朔"], r["主表朔"], r[col], r[hcol], r["日食"], r["原文"]]
                    for r in rs if r["本纪合主表"] == "0" or r[jin_] == "0"])
    # 七、宝祐四年会天历（check_huitian_1256.py）
    ht = rows("huitian_1256_check.csv")
    if ht:
        L += ["", "## 七、《宝祐四年会天历》（1256）", "",
              "第 1 级资料（当时历日的清钞本，转录 `data/calendar/会天历_1256.csv`）。会天历术不传，开禧为上一部历，上推只作对照。", ""]
        L += table(["月", "会天历一日", "主表", "开禧定朔（上推）", "小余（时）", "进朔限（时）", "开禧进朔"],
                   [[r["月"], r["会天历一日"], r["主表"], r["开禧定朔（上推）"], r["小余（时）"], r["进朔限（时）"], r["开禧进朔"]] for r in ht])
    (ROOT / "doc/calendar/main-table-checks.md").write_text("\n".join(L) + "\n", encoding="utf-8")
    print("doc/calendar/main-table-checks.md", len(L), "行")


if __name__ == "__main__":
    main()
