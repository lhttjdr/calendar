"""吐鲁番出土文书纪日（北凉、高昌）检验北凉玄始历推步、玄始外推与张表。

录文：data/calendar/tulufan_jiri.csv（《吐鲁番出土文书》图录本，文物出版社 1992—1996，壹—肆册；墓表取王素《吐鲁番出土高昌
文献编年》注中所引原文、荒川正晴与关尾史郎论文所录墓表、《西域考古图谱》下卷史料(26) 墓表书影；逐条注册、页、文书编号）。每条记某月朔干支（明书）或某月某日干支（由日序逆推朔干支）；只记干支不记日序者
检该干支是否落在该月；只记「閏某月」者检该表当年有无此闰月（另列「闰月对照」）。

比较的历表：
- 张表：vendor/calendar/chinese_calendar/实历朔日表.csv（中原/南朝一线，经 discriminating_records.tables()）；
- 推步:北凉：data/calendar/推步政权朔日表.csv 之北凉（玄始历，398—439，430 年二月 +1 日）；
- 推步:北魏：同表北魏/西魏/北周/隋（398—590 北朝诸历分段）；
- 玄始外推：北凉玄始历常数一路外推到 640（push_calendar::year_new_moons 之移植，见 gaochang_630_check.py），
  史家言高昌行北凉旧历；
- 另以北朝诸历常数（景初、正光、兴和、天保、天和、大象、开皇）一路外推作对照，只入汇总。

另求平朔的「整体平移」：设高昌所用历合朔时刻比玄始（或正光）常数一律晚 δ 日，每条有据之朔给出 δ 的容许区间
[D − 0.5 − m, D + 0.5 − m)（m 为平朔儒略日，D 为所记朔日 JDN），再并入 data/calendar/高昌_卅日.csv 的「某月卅日」（该月须大），
按时期求交集，并试 δ 随月数线性变化（朔策之差），以检视延寿七年（630）十二月残历壬辰与玄始辛卯差一日能否统一解释。

年份：公元年取图录本标题所注（编者据王素等考定；高昌诸王纪年换算依据见生成文档）；月为当时月名（建寅）。

输出 doc/calendar/tulufan-jiri-checks.md。
用法（仓库根）：uv run --project scripts python scripts/calendar_data/tulufan_jiri_check.py
"""
import csv
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import discriminating_records as d  # noqa: E402
from gaochang_630_check import GZ, XUANSHI, pingshuo_year  # noqa: E402
import shiwu_fenlei as fl  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
CSV = ROOT / "data/calendar/tulufan_jiri.csv"
OUT = ROOT / "doc/calendar/tulufan-jiri-checks.md"
GZI = {g: i for i, g in enumerate(GZ)}
MON = ["正", "二", "三", "四", "五", "六", "七", "八", "九", "十", "十一", "十二"]


NOTES = """
## 取材

- **《吐鲁番出土文书》图录本**壹、贰两册（北凉至高昌、唐初）全册逐页识读（竖排释文，RapidOCR 按字切片识读后检索「朔」及日干支）；
  因竖排识读漏检不少，又把壹册的随葬衣物疏（约三十件，以有纪年者为主）逐件看释文（衣物疏是高昌文书中惯写「某年某歲某月某朔某日」的一类）。
  叁册只查目录（全为唐），录得唐显庆三年具注历一件；肆册（唐）未查。凡计入者皆对图版或释文核过。
- 文书纪日绝大多数只记「某年某某歲某月某日」，记朔或日干支者很少：北凉文书一条也没有（衣物疏、券契只记月日）；麴氏高昌
  章和至延昌初（543—562）、重光（620—621）的衣物疏与墓表有「某月某某朔某日某某」。延和以后衣物疏格式改为「某年某歲某月某日」，不再记朔。
- **墓表**不在图录本，取王素《吐鲁番出土高昌文献编年》（新文丰 1997，本地扫描本）注中所引墓表原文（只在有疑义时才引原文，故所得甚少），
  及图录本衣物疏注中所引同墓墓表；王素书 535—640 年部分（书页 139—335）以 RapidOCR 识读检索「朔」「閏」，用到者皆对书影，北凉至 534 年部分（书页 1—138）未逐页识读。
- 闰月：图录本两件衣物疏与王素书所录墓表标题中的「閏某月」，作闰位证据单列（「闰」=1、无干支者）。
- **2026-11-26 增补墓表**（第 20—36 条）：侯灿、吴美琳《吐鲁番出土砖志集注》（2003）与黄文弼《高昌砖集》（1931/1951）未能取得
  （百度网盘无此二书，东洋文库 NII「ディジタル・シルクロード」只有黄文弼《高昌第一分本》（1931），其「墓专目录」只记年月日、不录朔）。改取：
  ① 香川默识编《西域考古图谱》下卷（国华社 1915）史料(26)「高昌国人墓表六种」原砖照片（东洋文库 DSR 书影第 0225 页），逐字识读，得朔五方
  （564、580、589、600 閏、634）；② 荒川正晴 2000（《シルクロード学研究》10，交河沟西新出墓表四方及所引黄文弼《高昌塼集》录文三方）、
  荒川 1983（《史观》109，延和六年张武忠墓表、贞观廿年张延衡墓表），只据论文录文；③ 关尾史郎编《吐鲁番出土文物研究会会报》
  （大阪大学 OUKA 公开，第 1—108 号中能取得者 106 号全文检索「朔」「閏」）：12 号（侯灿 1987 所论章和八年朱阿定墓表「二月朔庚寅……七日丙申」）、78 号（章和八年宋阿虎墓表，「庚申朔」系推补，不计）、87—88 号（墓志索引，
  定墓主名与月份）、91 号（重光元年氾法济墓表原文）；④ 据会报 64 号线索复查图录本贰册，补闰月两条（义和三年閏五月、延寿四年閏四月，皆对图；
  前次 OCR 把「閏」识作「潤」或漏字而失收）。会报 28、29 号所录敦煌前凉至西凉镇墓瓶纪日（多记朔）不在吐鲁番，未收。王素书北凉至 534 年部分（书页 1—138）仍未逐页识读。

- **2026-11-26 再补：侯灿、吴美琳《吐鲁番出土砖志集注》**（巴蜀书社 2003，上下册，无文字层）全书以 RapidOCR 识读（图版页跳过），
  据目录定各方起讫页，凡录文书朔、书闰者逐方录入（第 38—209 条，麴氏高昌 142 条、唐西州 30 条，至龙朔二年 662）。朔、日干支经自洽检验，
  不自洽者对照集注注文（集注多已注明「墓表少推一日」之类）；人名为 OCR 所得，或有讹字。第 1—37 条中与集注同方者，核对列加注集注页码，
  异读记于备注（564 年徐宁周妻孔氏墓表「十七日乙亥」、646 年张延衡墓表「廿一日己卯」皆据集注改正前录）。大凉（北凉）四方墓表无朔。

## 纪年与公元年

- 公元年取图录本标题所注。北凉诸年号（神璽、天璽、永安、玄始、承玄、義和、緣禾等）与公元的对应据《晋书·载记》《魏书·沮渠蒙逊传》等正史。
  阚氏、张氏、马氏、麴氏高昌的年号不见或少见于正史，今人据文书、墓表所记岁干支（「某年某某歲」）排定：同一年号内年数与岁干支
  一一对应，相邻年号首尾以干支衔接，下限延壽十七年（640）唐平高昌（《旧唐书·高昌传》贞观十四年）。王素《吐鲁番出土高昌文献编年》
  （1997）是这种年表的汇总，与图录本互有出入：如图录本「西凉建初二年（406）」功曹书佐奏、「北凉承平五年（447）」道人法安券，
  王素据文中岁干支（庚午、丙戌）改繫张氏高昌建初二年（490）、麴氏高昌承平五年（506）。本表凡原文记岁干支者，检其与所取公元年是否相合（全部相合）。
- 月为当时月名。高昌闰位与中原诸历不同（见「闰月对照」），月名差一而朔干支合于前后月者单独标出（「合前月/合后月」），
  这类不合反映闰位之异，不反映朔日推步之异。

## 结果要点

（2026-11-26 据《砖志集注》补录后重写；下列条数只计麴氏高昌、「计入」者。）

- **朔日**：麴氏高昌 162 条（集注补录前 20 条），合正光外推 128、玄始 123、张表 113、北魏系推步 43/73、大象 65、开皇 87。
  正光、玄始二历朔策只差 7×10⁻⁶ 日，逐月朔日多同，差在闰位；张表在 620 年以后屡晚一日（唐戊寅历定朔）。高昌不行中原历、
  北周隋历，朔日一线属玄始/正光系，可定。不合者多为墓表本身日序与干支差一日（集注注明者十余方）或月名差一（闰位之异）。
- **闰位**（7 条：592 閏正月丁未朔、597 閏六月丙[午]朔、600 閏二月辛酉朔、605 閏七月己未朔、616 閏五月、627 閏四月、635 閏[五]月乙未朔）：
  正光外推合 5（597、600、605、627、635），大象 3，张表 2，玄始 1（635），开皇 1。正光历元平移后（见下），前段 δ ≈ −0.14 日下
  597、600、605、616 皆合，后段 δ ≈ +0.02 日下 627、635 皆合，**七条中六条合「正光闰法 + 历元平移」**，只 592 閏正月（原砖今已漫漶，
  录文据罗振玉以来诸家）须 δ ∈ [+0.23, +0.70)，不合。玄始无论平移多少都要 δ ≥ +0.18 才合任一闰位，与朔日要求不容。**闰法合正光不合玄始**。
- **600 年閏月**：集注录文作「閏三月辛酉朔」（关尾索引同），王素推閏二月；两种图版（集注、《西域考古图谱》）閏下月数皆漫漶。
  同年傅子友墓表「七月朔戊子」：閏二月辛酉朔则七月戊子，閏三月辛酉朔则七月己未，只閏二月相容，本表从閏二月。此条朔辛酉在正光前段 δ 下
  仍晚一日（须 δ ≥ +0.17），是前段四条不合者之一。
- **615 年六月卅日**：集注录文「义和二年乙亥，六月朔辛酉，卅日庚寅」（唐幼谦妻麴氏墓表，黄文弼《砖集》75），王素卅日条得原文印证，
  且明书朔辛酉；正光前段 δ 下合。
- **改历元之年**：以正光为底，按「满足条数最多」求 δ（记录多了以后个别墓表书误使严格可行域为空）：619 年以前 130 条最多满足 124 条，
  δ ≈ −0.144 日（分时段：530—560、561—580、601—613 的最优区间都含或邻近此值；581—600 最优在 −0.28 至 −0.24，41 条满足 37，取 −0.144 时满足 36）；620 年以后 54 条满足 51 条，δ ∈ [+0.016, +0.036)。
  按年切成前后两段，合计满足最多（176/184）的切点是后段始于 617 年（616、618、620、621 年为 175）。逐条看：616 年閏五月要前段之 δ
  （< −0.012），617 年十一月戊申朔（义和四年丁丑，集注 150 号）要 δ ≥ −0.030，即后段；故**改历元在 616 年閏五月之后、617 年十一月之前，
  即义和三年下半至四年**，较原先的 616—627 大为收窄，落在义和政变期间（614—619），不在重光复辟（620）或延寿改元（624）。
  此切点的后端只靠 617 年十一月一条（若除去此条，则至 621 年十二月甲寅朔才须后段），宜视为「616—621 年间，以 617 年最可能」。
  改动形式：正光常数下历元推后约 0.16—0.18 日（约 4 小时），朔策、闰法不变。
- **不合之条**（以最优切点、前段 δ = −0.144、后段 +0.016 计）：前段 581 年十二月乙亥朔、588 年正月庚午朔、592 年五月乙亥朔
  （集注引王素当为乙巳，墓表书误）、600 年閏二月辛酉朔、592 年閏正月；后段 635 年閏月乙未朔、638 年十月乙巳朔（同年十一月丙午朔二方，
  十月乙巳与之不容，集注引王素推十月丙子）、622 年十月卅日。
- **唐西州**（640—662）30 条，合张表 27。
- **西凉建初四年（408）秀才对策文**：同件两处皆作「正月戊申朔一日戊申」，图版清楚。408 年正月诸表皆丙申（原注引《二十史朔闰表》亦丙申），
  差十二日，不能以闰位或进朔解释。正月戊申朔在 380—530 年间只见于 411、504 年（张表、玄始同）。若照王素把「建初」改繫张氏高昌（489 起），
  建初四年为 492，正月朔戊午，亦不合（午、申一字之差）。此条存疑，计入不合。
- **建昌四年（558）张孝章衣物疏**：月字残，图录本释文补「三」，王素作二月（同墓张遁墓表亦二月九日，集注 24 号「二月甲子朔，九日壬申」）；从二月。

## 延壽七年（630）十二月差一日

630 年残历十二月朔壬辰，玄始外推辛卯。玄始平朔小余（合朔时刻在当日的位置，0 为夜半）十二月为 0.982，即平朔在辛卯日将尽、
距壬辰夜半只 0.018 日（约 26 分钟）；同年十月小余 0.921 而残历十月壬辰合玄始（不进）。
1. **不是进朔**：若高昌行进朔，进朔限须在 0.921 与 0.982 之间，远高于诸历常见的四分之三，且八月（0.860）、六月（0.799）皆未进。
2. **620 年以后可由历元微差统一**：集注补录后 620 年以后朔日、卅日、闰月 54 条，满足最多者 51 条，δ ∈ [+0.019, +0.040)（玄始为底，
   约 27—58 分钟；以正光为底则 [+0.016, +0.036)），不合者为 622 年十月卅日、635 年閏月乙未朔、638 年十月乙巳朔三条。
3. **与 619 年以前不能统一**：607 年五月戊申朔要求 δ < −0.047、543 年十二月丙辰朔要求 δ < −0.056，541、602、615 年三条小月卅日要求 δ < −0.107，
   而 630 年十二月要求 δ ≥ +0.018：只用朔日（不用卅日）也已不能以一个常数 δ 统一，线性漂移（朔策另取）亦无解。故须一次改动（见上「改历元之年」）。
"""


def _p(lunar, solar, epoch, jdw, dtc, dts):
    return dict(lunar=lunar, solar=solar, jd_epoch=epoch + dtc, jdw=jdw + dts)


# push_calendar::*_params（含 dtc_round、dts_round），见 doc/calendar/push-algorithms.md 第四节
OTHERS = {
    "景初": _p(29 + 2419 / 4559, 365 + 455 / 1843, 1811120.5, 1721050.5 + 220 / 1843, 1e-4, 1e-5),
    "正光": _p(29 + 39769 / 74952, 365 + 1477 / 6060, 1911671.5 + 6427 / 9369, 1721048.5 + 569 / 1515, 1e-6, 1e-6),
    "兴和": _p(29 + 110647 / 208530, 365 + 4117 / 16860, 1918286.5 + 111983 / 208530, 1721048.5 + 1118 / 4215, 1e-6, 1e-6),
    "天保": _p(29 + 155272 / 292635, 365 + 5787 / 23660, 1921918.5 + 231721 / 292635, 1721049.5 + 193 / 5915, 1e-6, 1e-6),
    "天和": _p(29 + 153991 / 290160, 365 + 5731 / 23460, 1928976.5 + 246737 / 290160, 1721047.5 + 1331 / 3910, 1e-6, 1e-6),
    "大象": _p(29 + 28422 / 53563, 365 + 3167 / 12992, 1932520.5 + 36950 / 53563, 1721048.5 + 281 / 6496, 1e-5, 1e-6),
    "开皇": _p(29 + 96529 / 181920, 365 + 25063 / 102960, 1934971.5 + 19841 / 36384, 1721048.5 + 908 / 6435, 1e-6, 1e-7),
}


def extrap_table(p, y0, y1):
    """{(年, 月, 闰): 朔 JDN}"""
    return {(y, m, lp): j for y in range(y0, y1 + 1) for m, lp, j in pingshuo_year(p, y)}


def mean_conj(p, jdn):
    """距 jdn 最近的平朔时刻（儒略日，日始于半夜 = x.5）。"""
    i = round((jdn - p["jd_epoch"]) / p["lunar"])
    return p["jd_epoch"] + i * p["lunar"]


def load():
    rows = []
    for r in csv.DictReader(l for l in open(CSV, encoding="utf-8") if not l.startswith("#")):
        rows.append(r)
    return rows


def implied_shuo(r):
    """所记之朔干支序号；无日序时返回 None。"""
    if r["朔干支"]:
        return GZI[r["朔干支"]]
    if r["日"] and r["干支"]:
        return (GZI[r["干支"]] - (int(r["日"]) - 1)) % 60
    return None


def judge(r, table):
    """(朔干支或 None, 结论 ✓/✗/—)"""
    key = (int(r["公元"]), int(r["月"]), r["闰"] == "1")
    j = table.get(key)
    if j is None:
        return "（无此月）", "✗" if key[0] in {k[0] for k in table} else "—"
    sg = GZ[(j + 49) % 60]
    s = implied_shuo(r)
    if s is not None:
        if GZI[sg] == s:
            return sg, "✓"
        # 闰位不同则月名差一：看前后月
        prv = max((v for v in table.values() if v < j), default=None)
        nxt = min((v for v in table.values() if v > j), default=None)
        if prv is not None and (prv + 49) % 60 == s:
            return sg, "✗（合前月）"
        if nxt is not None and (nxt + 49) % 60 == s:
            return sg, "✗（合后月）"
        return sg, "✗"
    if not r["干支"]:  # 只记月（如闰月）：该表有此月即合
        return sg, "✓"
    # 只记干支：检是否在本月内
    nxt = min((v for v in table.values() if v > j), default=j + 30)
    ok = any((jj + 49) % 60 == GZI[r["干支"]] for jj in range(j, nxt))
    return sg, "✓" if ok else "✗"


def _intervals(mask, grid):
    """布尔网格 → 连续区间文字。"""
    out, st = [], None
    for i, v in enumerate(list(mask) + [False]):
        if v and st is None:
            st = i
        if not v and st is not None:
            out.append(f"[{grid[st]:+.3f}, {grid[i - 1] + (grid[1] - grid[0]):+.3f})")
            st = None
    return "、".join(out) or "空"


def leap_section(rows, zhang):
    """高昌闰月证据与各历外推闰位对照（前后一年内的闰月都列出，月份前加年份后两位以免歧义）。"""
    cals = [("张表", None), ("玄始", XUANSHI)] + [(k, OTHERS[k]) for k in ("正光", "兴和", "天保", "大象", "开皇")]

    def leaps(p, y):
        if p is None:
            return [(k[0], k[1]) for k in zhang if k[2] and y - 1 <= k[0] <= y]
        return [(yy, m) for yy in (y - 1, y) for m, lp, j in pingshuo_year(p, yy) if lp]

    out = ["## 闰月对照", "",
           "麴氏高昌闰月证据（本表「闰」=1 各条，唐不列）与各历外推之闰位。格中列该年及前一年的闰月（「12/閏4」= 某年閏四月，"
           "取年份末两位）；与高昌同年同月者标 ✓。", "",
           "| # | 公元 | 高昌闰 | " + " | ".join(c for c, _ in cals) + " |", "|---|---|---|" + "---|" * len(cals)]
    hit = Counter()
    n = 0
    for r in rows:
        if r["闰"] != "1" or r["政权"] != "麴氏高昌":
            continue
        y, m = int(r["公元"]), int(r["月"])
        use = r.get("计入", "1") != "0"
        n += use
        cells = []
        for c, p in cals:
            ls = leaps(p, y)
            ok = (y, m) in ls
            hit[c] += ok and use
            cells.append("、".join(f"{yy % 100:02d}/閏{mm}" for yy, mm in ls) + (" ✓" if ok else "") or "—")
        out.append(f"| {r['编号']}{'' if use else '（不计）'} | {y} | 閏{MON[m - 1]}月 | " + " | ".join(cells) + " |")
    out += ["", "计入者 " + str(n) + " 条，同年同月相合：" + "，".join(f"{c} {hit[c]}" for c, _ in cals) + "。", ""]
    return out


def shift_section(delta, P, pname, rows=()):
    """把朔日记录与「某月卅日」（data/calendar/高昌_卅日.csv）合起来，求某历（P）平朔整体平移 δ（及随时间线性变化的 δ）的可行域。

    朔日记录：floor(m + δ + 0.5) = 所记朔日（m 取所记朔日最近的该历平朔，不受闰位影响）；
    卅日记录：该月为大月，即 floor(m₂ + δ + 0.5) − floor(m₁ + δ + 0.5) = 30，m₁、m₂ 取该历外推中同名之月及下月
    （高昌闰位若与该历不同，月名所指可差一月）。只用麴氏高昌。"""
    import numpy as np
    xs = extrap_table(P, 530, 645)
    lunar = P["lunar"]
    ref = mean_conj(P, xs[(630, 12, False)])
    cons = []  # (标签, 年, 月数 n, 判定函数(δ数组)→布尔数组)
    for r, lo, hi, m in delta:
        if r["政权"] != "麴氏高昌":
            continue
        D = round(m + lo + 0.5)
        mm = mean_conj(P, D)
        lo, hi = D - 0.5 - mm, D + 0.5 - mm
        n = round((mm - ref) / lunar)
        cons.append((f"朔 {r['公元']}年{MON[int(r['月']) - 1]}月（{r['编号']}）", int(r["公元"]), n,
                     lambda d, lo=lo, hi=hi: (d >= lo) & (d < hi)))
    for r in csv.DictReader(l for l in open(ROOT / "data/calendar/高昌_卅日.csv", encoding="utf-8") if not l.startswith("#")):
        y, mo = int(r["公元"]), int(r["月"])
        if (y, mo, False) not in xs:
            continue
        m1 = mean_conj(P, xs[(y, mo, False)])
        m2 = m1 + lunar
        n = round((m1 - ref) / lunar)
        cons.append((f"卅日 {y}年{MON[mo - 1]}月（王素 {r['编号']}）", y, n,
                     lambda d, m1=m1, m2=m2: (np.floor(m2 + d + 0.5) - np.floor(m1 + d + 0.5)) == 30))
    grid = np.round(np.arange(-1.0, 1.0, 0.001), 3)
    out = [f"## {pname}平朔整体平移 δ 的可行域（并入「某月卅日」）", "",
           f"把上表朔日与 `data/calendar/高昌_卅日.csv` 的「某月卅日」（该月须大）合起来，以{pname}外推为底，逐条求 δ 的可行区间（步长 0.001 日），"
           f"再按时期求交集。卅日条的月名按{pname}外推同名之月取，高昌闰位不同时会错配。", "",
           "| 约束 | 公元 | 距 630 年十二月月数 | δ 可行区间 |", "|---|---|---|---|"]
    for lab, y, n, f in sorted(cons, key=lambda c: c[2]):
        out.append(f"| {lab} | {y} | {n} | {_intervals(f(grid), grid)} |")
    out.append("")
    for name, sel in (("全部", lambda y: True), ("619 年以前（义和政变以前）", lambda y: y <= 619), ("620 年以后（重光复辟以后）", lambda y: y >= 620),
                      ("620 年以后，除 622 年十月卅日", "x622"),
                      ("只用朔日（不用卅日）", None)):
        if sel == "x622":
            cs = [c for c in cons if c[1] >= 620 and "622年十月" not in c[0]]
        else:
            cs = [c for c in cons if (c[0].startswith("朔") if sel is None else sel(c[1]))]
        mask = np.ones_like(grid, dtype=bool)
        for c in cs:
            mask &= c[3](grid)
        out.append(f"- {name}（{len(cs)} 条）：常数 δ ∈ {_intervals(mask, grid)}")
    # 按年排出：同年诸约束之交
    out += ["", f"按年排出（同年诸约束求交；「前段」「后段」= 该年 δ 区间与 619 年以前（除 600 年閏月辛酉朔）/ 620 年以后（除 622 年十月卅日）之常数解是否相交）：", "",
            "| 公元 | 约束数 | 该年 δ 可行域 | 容于前段 | 容于后段 |", "|---|---|---|---|---|"]
    def _mask(cs):
        mk = np.ones_like(grid, dtype=bool)
        for c in cs:
            mk &= c[3](grid)
        return mk
    pre = _mask([c for c in cons if c[1] <= 619 and "600年" not in c[0]])
    post = _mask([c for c in cons if c[1] >= 620 and "622年十月" not in c[0]])
    for y in sorted({c[1] for c in cons}):
        mk = _mask([c for c in cons if c[1] == y])
        out.append(f"| {y} | {sum(c[1] == y for c in cons)} | {_intervals(mk, grid)} | {'✓' if (mk & pre).any() else '✗'} | {'✓' if (mk & post).any() else '✗'} |")
    # 分段（改历元）之年：按时间把约束切成前后两段，两段各自有常数解者
    out += ["", "改历元之年（把约束按时间切成前后两段，各段须各有常数 δ 解；列出所有可行切点，「切点」= 前段最后一条与后段第一条之间）：", ""]
    for name, cs in (("全部约束", cons), ("除 622 年十月卅日", [c for c in cons if "622年十月" not in c[0]]),
                     ("除 622 年十月卅日与 600 年閏月辛酉朔", [c for c in cons if "622年十月" not in c[0] and "600年" not in c[0]])):
        cs = sorted(cs, key=lambda c: c[2])
        ok = []
        for k in range(1, len(cs)):
            if cs[k][2] == cs[k - 1][2]:
                continue
            a, b = _mask(cs[:k]), _mask(cs[k:])
            if a.any() and b.any():
                ok.append((cs[k - 1][0], cs[k][0], _intervals(a, grid), _intervals(b, grid)))
        if not ok:
            out.append(f"- {name}：任何切点都不能使前后两段各有常数解（须两次以上改动或个别记录有误）。")
        else:
            out.append(f"- {name}：可行切点 {len(ok)} 处，最早在「{ok[0][0]}」之后（前段 δ ∈ {ok[0][2]}，后段 {ok[0][3]}），"
                       f"最晚在「{ok[-1][1]}」之前（前段 {ok[-1][2]}，后段 {ok[-1][3]}）。")
    # 闰月并入：平移 δ 后该历当年闰位是否与高昌所记相合（δ 网格步 0.005，取最近格点）
    lstep = 0.005
    lgrid = np.round(np.arange(-1.0, 1.0, lstep), 3)
    lcons = []
    for r in rows:
        if r["闰"] != "1" or r.get("计入", "1") == "0" or r["政权"] != "麴氏高昌":
            continue
        y, mo = int(r["公元"]), int(r["月"])
        hit = np.array([any(m == mo and lp for m, lp, _ in pingshuo_year(dict(P, jd_epoch=P["jd_epoch"] + d), y)) for d in lgrid])
        n = round((mean_conj(P, xs[(y, mo, False)]) - ref) / lunar)
        lcons.append((f"閏 {y}年閏{MON[mo - 1]}月（{r['编号']}）", y, n,
                      lambda d, hit=hit: hit[np.clip(np.round((np.asarray(d) + 1.0) / lstep).astype(int), 0, len(hit) - 1)]))
    out += ["", f"闰月并入：{pname}平朔平移 δ 后（气不动），当年闰位与高昌所记相合的 δ 区间（网格步 {lstep} 日）：", "",
            "| 约束 | 公元 | δ 可行区间 | 容于前段 | 容于后段 |", "|---|---|---|---|---|"]
    for c in lcons:
        mk = c[3](grid)
        out.append(f"| {c[0]} | {c[1]} | {_intervals(mk, grid)} | {'✓' if (mk & pre).any() else '✗'} | {'✓' if (mk & post).any() else '✗'} |")
    usable = [c for c in lcons if (c[3](grid) & (pre | post)).any()]
    base = [c for c in cons if "622年十月" not in c[0] and "600年" not in c[0]]
    out.append("")
    for nm, cs in ((f"朔日、卅日（除 622 年十月卅日与 600 年閏月辛酉朔）并入与前段或后段相容的闰月 {len(usable)} 条", base + usable),
                   (f"只用朔日（除 600 年閏月辛酉朔）与上述闰月，不用卅日", [c for c in base if c[0].startswith("朔")] + usable)):
        cs = sorted(cs, key=lambda c: c[2])
        ok = []
        for k in range(1, len(cs)):
            if cs[k][2] == cs[k - 1][2]:
                continue
            a, b = _mask(cs[:k]), _mask(cs[k:])
            if a.any() and b.any():
                ok.append((cs[k - 1][0], cs[k][0], _intervals(a, grid), _intervals(b, grid)))
        if ok:
            out.append(f"- {nm}（{len(cs)} 条）：可行切点 {len(ok)} 处，最早在「{ok[0][0]}」之后（前段 δ ∈ {ok[0][2]}，后段 {ok[0][3]}），"
                       f"最晚在「{ok[-1][1]}」之前（前段 {ok[-1][2]}，后段 {ok[-1][3]}）。")
        else:
            out.append(f"- {nm}（{len(cs)} 条）：任何切点都不能使前后两段各有常数解。")
    # 稳健分析：记录多了以后个别墓表（日序差一日、月名误书等）使常数解为空，改求「满足条数最多」的 δ
    def _best(cs):
        if not cs:
            return 0, ""
        sat = np.zeros_like(grid, dtype=int)
        for c in cs:
            sat += c[3](grid)
        mx = sat.max()
        return int(mx), _intervals(sat == mx, grid)
    allc = cons + lcons
    out += ["", f"**稳健分析**（朔日、卅日、闰月合计 {len(allc)} 条；取满足条数最多的 δ，不要求全部满足）：", "",
            "| 时段 | 条数 | 最多满足 | 最优 δ 区间 |", "|---|---|---|---|"]
    for a, b in ((530, 560), (561, 580), (581, 600), (601, 613), (614, 619), (620, 629), (630, 640), (530, 619), (620, 640)):
        cs = [c for c in allc if a <= c[1] <= b]
        k, iv = _best(cs)
        out.append(f"| {a}—{b} | {len(cs)} | {k} | {iv} |")
    ys = sorted({c[1] for c in allc})
    sc = []
    for y in ys[1:]:
        k1, i1 = _best([c for c in allc if c[1] < y]); k2, i2 = _best([c for c in allc if c[1] >= y])
        sc.append((k1 + k2, y, k1, i1, k2, i2))
    k0, i0 = _best(allc)
    top = max(s[0] for s in sc)
    out += ["", f"不分段（一个常数 δ）最多满足 {k0}/{len(allc)} 条，δ ∈ {i0}。按年切成前后两段、各取最优 δ，合计满足条数：", "",
            "| 后段始于 | 合计满足 | 前段满足 | 前段 δ | 后段满足 | 后段 δ |", "|---|---|---|---|---|---|"]
    for s in sorted(sc, key=lambda s: -s[0])[:8]:
        out.append(f"| {s[1]} | {s[0]}{' ★' if s[0] == top else ''} | {s[2]} | {s[3]} | {s[4]} | {s[5]} |")
    best_y = [s[1] for s in sc if s[0] == top]
    out += ["", f"合计满足最多（{top}/{len(allc)}）的切点：后段始于 {'、'.join(map(str, best_y))} 年。", ""]
    # 以最优切点列出不合之条
    y0 = best_y[0]
    pre_mask = None
    for seg, cs in (("前段", [c for c in allc if c[1] < y0]), ("后段", [c for c in allc if c[1] >= y0])):
        sat = np.zeros_like(grid, dtype=int)
        for c in cs:
            sat += c[3](grid)
        d0 = grid[np.argmax(sat)]
        bad = [c[0] for c in cs if not c[3](np.array([d0]))[0]]
        out.append(f"- {seg}取 δ = {d0:+.3f} 时不合者 {len(bad)} 条：{'；'.join(bad) or '无'}")
    out.append("")
    # 线性漂移
    A = np.round(np.arange(-0.6, 0.6, 0.002), 3)
    B = np.arange(-4e-4, 4e-4 + 1e-12, 2e-6)
    out += ["", "线性漂移 δ(n) = a + b·n（n 为距 630 年十二月月数；b 即朔策之差，日/月；网格 a 步 0.002、b 步 2×10⁻⁶）：", ""]
    for name, cs in (("全部约束", cons), ("除 622 年十月卅日", [c for c in cons if "622年十月" not in c[0]]),
                     ("只用朔日", [c for c in cons if c[0].startswith("朔")])):
        ns = np.array([c[2] for c in cs])
        feas = []
        for bb in B:
            d = A[:, None] + bb * ns[None, :]
            ok = np.ones(len(A), dtype=bool)
            for k, c in enumerate(cs):
                ok &= c[3](d[:, k])
            if ok.any():
                feas.append((bb, A[ok].min(), A[ok].max()))
        if feas:
            out.append(f"- {name}（{len(cs)} 条）：b ∈ [{feas[0][0]:+.6f}, {feas[-1][0]:+.6f}]，朔策 {lunar + feas[0][0]:.6f}—{lunar + feas[-1][0]:.6f} 日"
                       f"（{pname} {lunar:.6f}，今测平朔望月 29.530589）。")
        else:
            out.append(f"- {name}（{len(cs)} 条）：无可行解。")
    out.append("")
    return out


def fenlei_section(rows, zhang, xs, zg):
    """麴氏高昌有朔之月按月分类：外推（玄始、正光各一套）与张表互异、相同，墓表、衣物疏为当时文字（B 级）。"""
    obs = defaultdict(list)
    tang = Counter()
    for r in rows:
        s = implied_shuo(r)
        if r.get("计入", "1") == "0" or s is None:
            continue
        key = (int(r["公元"]), int(r["月"]), r["闰"] == "1")
        if r["政权"] == "麴氏高昌":
            obs[key].append(GZ[s])
        elif r["政权"] == "唐":
            tang[judge(r, zhang)[1][0]] += 1

    def g(t, k):
        if k in t:
            return GZ[(t[k] + 49) % 60]
        return "（无此月）" if k[0] in {x[0] for x in t} else ""

    out = ["", "## 分类（按月）", "",
           "分类对象是外推历（「推步」：玄始、正光常数一路外推，各分一套，皆不作历元平移）与张表，限麴氏高昌有朔（明书或由日序逆推）之月；"
           "墓表、衣物疏、券契是当时文字（B 级），一条即可判，同月诸条冲突归③；同月诸方各算一条。"
           "一家无此月（闰位异）者亦算互异。张表在此段是中原历，不代表高昌历日，①即「张表不能代表高昌」。"
           "同误中「差 −1 日」「差 +1 日」者多可由下文「历元平移 δ」（前段 δ<0、后段 δ>0）解释，分类按未平移之外推，不据 δ 改判。" + fl.RULE, ""]
    tabs = {}
    for name, t in (("玄始", xs), ("正光", zg)):
        cls = {k: fl.classify(g(t, k), g(zhang, k), v, "B", "墓表") for k, v in obs.items()}
        tabs[name] = cls
        cats = [(c, "B" if c != "无证" else "—") for c, _ in cls.values() if c]
        out += [f"- **{name}外推**：有朔之月 {len(cats)}，" + fl.headline(cats, f"{name}外推") + "。"]
    cats = [(c, "B" if c != "无证" else "—") for c, _ in tabs["正光"].values() if c]
    out += ["", "正光外推按等级：", ""] + fl.grade_table(cats)
    out += ["", f"唐西州（640 以后）行唐历，张表即唐历，无另一家可比：有朔之条合张表 {tang['✓']}、不合 {tang['✗']}，不入分类。", "",
            "逐月（只列两家互异或同误之月；余为有证）：", "",
            "| 公元 | 月 | 所记朔 | 张表 | 玄始外推 | 分类（玄始） | 正光外推 | 分类（正光） |", "|---|---|---|---|---|---|---|---|"]
    for k in sorted(obs):
        a, b = tabs["玄始"][k], tabs["正光"][k]
        if a[0] == "有证" and b[0] == "有证":
            continue
        f = lambda c: (c[0] or "—") + (f"（{c[1]}）" if c[1] else "")
        lab = ("閏" if k[2] else "") + MON[k[1] - 1] + "月"
        out.append(f"| {k[0]} | {lab} | {'、'.join(obs[k])} | {g(zhang, k) or '—'} | {g(xs, k) or '—'} | {f(a)} | {g(zg, k) or '—'} | {f(b)} |")
    return out


def main():
    rows = load()
    tabs = d.tables()
    zhang = tabs["张表"]
    beiliang = tabs.get("推步:北凉", {})
    beiwei = tabs.get("推步:北魏/西魏/北周/隋", {})
    xs = extrap_table(XUANSHI, 390, 645)
    others = {k: extrap_table(p, 390, 645) for k, p in OTHERS.items()}

    md = ["# 吐鲁番出土文书纪日与玄始历、张表（自动生成）", "",
          "> 由 `scripts/calendar_data/tulufan_jiri_check.py` 生成，勿手改。录文 `data/calendar/tulufan_jiri.csv`"
          "（《吐鲁番出土文书》图录本壹—肆册、王素书所引墓表、荒川与关尾所录墓表、《西域考古图谱》墓表书影），依据与列义见该文件注释。", ""]
    md += NOTES.strip("\n").splitlines()
    md += ["", "## 逐条", "",
           "「记朔」为所记或由日序逆推之朔干支；「—」为该表无此年。推步北凉限 398—439，推步北魏限 398—590。"
           "「玄始小余」为与所记朔日最近的玄始平朔的日内小数（0 为夜半；无朔者取同名之月）。「（不计）」者只列，不入汇总与 δ。", "",
           "| # | 政权 | 纪年 | 公元 | 月 | 原文（节） | 记朔 | 张表 | 推步北凉 | 推步北魏 | 玄始外推 | 玄始小余 | 合之外推历 | 岁干支 | 出处 | 核对 |",
           "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|"]
    cnt = defaultdict(Counter)
    delta = []
    for r in rows:
        use = r.get("计入", "1") != "0"
        kind = "朔" if implied_shuo(r) is not None else ("闰" if r["闰"] == "1" and not r["干支"] else "日")
        s = implied_shuo(r)
        rec = GZ[s] if s is not None else (f"（{r['干支']}在月内）" if r["干支"] else "（有此闰月）")
        res = {}
        for name, t in (("张表", zhang), ("推步北凉", beiliang), ("推步北魏", beiwei), ("玄始外推", xs)):
            res[name] = judge(r, t)
        okothers = [k for k, t in others.items() if judge(r, t)[1] == "✓"]
        for k, t in others.items():
            v = judge(r, t)[1]
            if v != "—" and use:
                cnt[f"{kind}|{k}"][v[0]] += 1
                cnt[f"{kind}|{k}|{r['政权']}"][v[0]] += 1
                if "月" in v:
                    cnt[f"{kind}|{k}"]["邻"] += 1
                    cnt[f"{kind}|{k}|{r['政权']}"]["邻"] += 1
        for name, (sg, v) in res.items():
            if v != "—" and use:
                cnt[f"{kind}|{name}"][v[0]] += 1
                cnt[f"{kind}|{name}|{r['政权']}"][v[0]] += 1
                if "月" in v:
                    cnt[f"{kind}|{name}"]["邻"] += 1
                    cnt[f"{kind}|{name}|{r['政权']}"]["邻"] += 1
        # 玄始小余与 δ 区间
        frac = ""
        key = (int(r["公元"]), int(r["月"]), r["闰"] == "1")
        if key not in xs and key[2] and (key[0], key[1], False) in xs:
            key = (key[0], key[1], False)  # 玄始无此闰月：以同名月为底，前后月内找所记之朔
        if key in xs:
            m = mean_conj(XUANSHI, xs[key])
            frac = f"{(m + 0.5) % 1:.3f}"
            if s is not None:
                # 所记朔 D：取玄始朔日前后 2 日内干支相符者
                # （闰位异而月名差一者取前后月之平朔）
                for base in (xs[key], xs[key] - 30, xs[key] + 30):
                    cands = [base + k for k in range(-2, 3) if (base + k + 49) % 60 == s]
                    if cands:
                        D = cands[0]
                        mm = mean_conj(XUANSHI, D)
                        frac = f"{(mm + 0.5) % 1:.3f}"
                        if use:
                            delta.append((r, D - 0.5 - mm, D + 0.5 - mm, mm))
                        break
        lab = ("閏" if r["闰"] == "1" else "") + MON[int(r["月"]) - 1] + "月"
        src = f"{r['册']}/{r['页']}"
        sui = ""
        if r["岁干支"]:
            sui = r["岁干支"] + (" ✓" if GZ[(int(r["公元"]) - 4) % 60] == r["岁干支"] else f" ✗（{r['公元']} 为{GZ[(int(r['公元']) - 4) % 60]}）")
        md.append(f"| {r['编号']}{'' if use else '（不计）'} | {r['政权']} | {r['年号']}{r['年'] + '年' if r['年'] else ''} | {r['公元']} | {lab} | {r['原文']} | {rec} | "
                  + " | ".join(f"{sg} {v}" if v != "—" else "—" for sg, v in res.values())
                  + f" | {frac} | {'、'.join(okothers) or '—'} | {sui or '—'} | {src} | {r['核对']} |")

    md += ["", "## 汇总", ""]
    regs = []
    for r in rows:
        if r["政权"] not in regs:
            regs.append(r["政权"])
    for kind, title in (("朔", "朔日（明书或由日序逆推）"), ("闰", "闰月（该表当年是否有此闰月）")):
        md += [f"**{title}**（只计「计入」者）：", "", "| 历表 | 合 | 不合 | 其中合前后月（闰位异） |", "|---|---|---|---|"]
        for name in ["张表", "推步北凉", "推步北魏", "玄始外推"] + list(OTHERS):
            c = cnt[f"{kind}|{name}"]
            md.append(f"| {name}{'（外推）' if name in OTHERS else ''} | {c['✓']} | {c['✗']} | {c['邻']} |")
        md += ["", f"其中麴氏高昌（{title[:2]}）：", "", "| 历表 | 合 | 不合 | 其中合前后月 |", "|---|---|---|---|"]
        for name in ["张表", "推步北魏", "玄始外推"] + list(OTHERS):
            c = cnt[f"{kind}|{name}|麴氏高昌"]
            md.append(f"| {name}{'（外推）' if name in OTHERS else ''} | {c['✓']} | {c['✗']} | {c['邻']} |")
        md += ["", "| 政权 | 张表 合/不合 | 推步北凉 | 推步北魏 | 玄始外推 |", "|---|---|---|---|---|"]
        for g in regs:
            cells = []
            for name in ["张表", "推步北凉", "推步北魏", "玄始外推"]:
                c = cnt[f"{kind}|{name}|{g}"]
                cells.append(f"{c['✓']}/{c['✗']}" if c else "—")
            if any(x != "—" for x in cells):
                md.append(f"| {g} | " + " | ".join(cells) + " |")
        md.append("")

    md += fenlei_section(rows, zhang, xs, others["正光"])
    # δ 区间
    md += ["", "## 玄始外推的常数偏移 δ", "",
           "设高昌实用之历的平朔比本项目玄始常数一律晚 δ 日（δ>0 即合朔时刻推后），各条有据之朔容许的 δ 区间如下"
           "（含 630 年残历诸月，录自 data/calendar/gaochang_630.csv 之有据者）。", "",
           "| 来源 | 公元 | 月 | 记朔 | 玄始小余 | δ 下限 | δ 上限 |", "|---|---|---|---|---|---|---|"]
    for r in csv.DictReader(l for l in open(ROOT / "data/calendar/gaochang_630.csv", encoding="utf-8") if not l.startswith("#")):
        if r["依据"] == "推拟":
            continue
        key = (int(r["年"]), int(r["月"]), r["闰"] == "1")
        if key not in xs:
            continue
        s = GZI[r["朔干支"]]
        m = mean_conj(XUANSHI, xs[key])
        D = next(xs[key] + k for k in range(-2, 3) if (xs[key] + k + 49) % 60 == s)
        delta.append(({"编号": "630残历", "公元": r["年"], "月": r["月"], "闰": r["闰"], "政权": "麴氏高昌"}, D - 0.5 - m, D + 0.5 - m, m))
    for r, lo, hi, m in sorted(delta, key=lambda x: x[3]):
        lab = ("閏" if r["闰"] == "1" else "") + MON[int(r["月"]) - 1] + "月"
        sg = GZ[(round(lo + m + 0.5) + 49) % 60]
        md.append(f"| {r['编号']} | {r['公元']} | {lab} | {sg} | {(m + 0.5) % 1:.3f} | {lo:+.3f} | {hi:+.3f} |")
    md += leap_section(rows, zhang)
    md += [""] + shift_section(delta, XUANSHI, "玄始", rows) + shift_section(delta, OTHERS["正光"], "正光", rows)
    OUT.write_text("\n".join(md), encoding="utf-8")
    print("\n".join(md))


if __name__ == "__main__":
    main()
