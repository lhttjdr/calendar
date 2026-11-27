"""吐鲁番出土文书纪日（北凉、高昌）检验北凉玄始历推步、玄始外推与张表。

录文：data/calendar/tulufan_jiri.csv（《吐鲁番出土文书》图录本，文物出版社 1992—1996，壹—肆册；墓表取王素《吐鲁番出土高昌
文献编年》注中所引原文、荒川正晴与关尾史郎论文所录墓表、《西域考古图谱》下卷史料(26) 墓表书影；逐条注册、页、文书编号）。每条记某月朔干支（明书）或某月某日干支（由日序逆推朔干支）；只记干支不记日序者
检该干支是否落在该月；只记「閏某月」者检该表当年有无此闰月（另列「闰月对照」）。

比较的历表：
- 张表：vendor/calendar/chinese_calendar/实历朔日表.csv（中原/南朝一线，经 discriminating_records.tables()）；
- 推步:高昌（「高昌历推步」，分类的「推步」一家）：data/calendar/推步政权朔日表.csv 之高昌（rust/core/src/calendar/gaochang.rs：
  正光常数 + 两段历元平移，616 年以前 δ = −0.144 日、617 年起 +0.026 日；δ 由本文件的稳健分析反推，属「按术推步 + 据实物定参」，
  不是术文）。本脚本另以 Python 照同一参数重算（gaochang_table），与导出表逐月比对一致才继续，并以此做 δ 与切点的敏感性；
- 推步:北凉：data/calendar/推步政权朔日表.csv 之北凉（玄始历，398—439，430 年二月 +1 日）；
- 推步:北魏：同表北魏/西魏/北周/隋（398—590 北朝诸历分段）；
- 玄始外推：北凉玄始历常数一路外推到 640（push_calendar::year_new_moons 之移植，见 gaochang_630_check.py），
  史家言高昌行北凉旧历；
- 另以北朝诸历常数（景初、正光、兴和、天保、天和、大象、开皇）一路外推作对照，只入汇总；「正光外推」「玄始外推」
  （皆不平移）的逐月分类保留作对照。

另求平朔的「整体平移」：设高昌所用历合朔时刻比玄始（或正光）常数一律晚 δ 日，每条有据之朔给出 δ 的容许区间
[D − 0.5 − m, D + 0.5 − m)（m 为平朔儒略日，D 为所记朔日 JDN），再并入 data/calendar/高昌_卅日.csv 的「某月卅日」（该月须大），
按时期求交集，并试 δ 随月数线性变化（朔策之差），以检视延寿七年（630）十二月残历壬辰与玄始辛卯差一日能否统一解释。

年份：公元年取图录本标题所注（编者据王素等考定；高昌诸王纪年换算依据见生成文档）；月为当时月名（建寅）。

输出 doc/calendar/tulufan-jiri-checks.md。
用法（仓库根）：uv run --project scripts python scripts/calendar_data/tulufan_jiri_check.py
"""
import csv
import sys
from functools import lru_cache
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
- **改历元之年**（推步取此结论，见下条）：以正光为底，按「满足条数最多」求 δ（记录多了以后个别墓表书误使严格可行域为空）：619 年以前 130 条最多满足 124 条，
  δ ≈ −0.144 日（分时段：530—560、561—580、601—613 的最优区间都含或邻近此值；581—600 最优在 −0.28 至 −0.24，41 条满足 37，取 −0.144 时满足 36）；620 年以后 54 条满足 51 条，δ ∈ [+0.016, +0.036)。
  按年切成前后两段，合计满足最多（176/184）的切点是后段始于 617 年（616、618、620、621 年为 175）。逐条看：616 年閏五月要前段之 δ
  （< −0.012），617 年十一月戊申朔（义和四年丁丑，集注 150 号）要 δ ≥ −0.030，即后段；故**改历元在 616 年閏五月之后、617 年十一月之前，
  即义和三年下半至四年**，较原先的 616—627 大为收窄，落在义和政变期间（614—619），不在重光复辟（620）或延寿改元（624）。
  此切点的后端只靠 617 年十一月一条（若除去此条，则至 621 年十二月甲寅朔才须后段），宜视为「616—621 年间，以 617 年最可能」。
  改动形式：正光常数下历元推后约 0.16—0.18 日（约 4 小时），朔策、闰法不变。
- **高昌历推步**（2026-11-27）：把上条结论落成推步——正光常数，616 年以前历元平移 δ = −0.144 日、617 年起 +0.026 日
  （各取满足最多之区间 [−0.14497, −0.14307)、[+0.01548, +0.03556) 之中），年限 523—640，经推步政权表（政权「高昌」）供换算。
  δ 由本页砖志反推，属「按术推步 + 据实物定参」，不是术文。麴氏高昌朔 162 条合 142（正光不平移 128、玄始 123、张表 113），
  闰 7 条合 6，卅日 11 条合 10，630 残历有据之月 11 全合。按月分类（147 月）：与张表**相同 117**（有证 103／同误 13／无证 1），
  **互异 30**（①证张表误 24／②证推步误 0／③无据待核 6；① 须过留一检验，原 25 月中 573 年三月不过改 ③，见「分类」节）。旧以正光外推（不平移）为推步一栏时相同 130（有证 103／同误 27／无证 0）、
  互异 17（①12／②0／③5）；今只记日序干支、不书朔者作纪日（D 级）一条不判，正光外推一栏 567 年七月（第 8 条）由①改③，成①11／③6。
  正光外推 → 高昌推步之变：14 个月由「同误（墓表早张表、正光一日）」变为①（留一检验后 13 个月仍①、1 个月③），567 年七月由③变为无证（推步闰位移到六月后，与张表同）。
  这 14 个月正是 δ 所据，推步合之是拟合的结果；墓表证张表误则不依赖 δ。
  区间两端各重跑一次，逐条结果与取中全同；切点移到 616、618、620 年合计各少 1 条（见「高昌历推步」一节）。
- **未合之条**（高昌历推步，逐条见「未能合之条」）：朔 20 条——月名差一 7（多为墓表书误或解释之异，567 年七月纪日、592 年五月等），
  差 7—20 日 7（抄误或释读误），差一日 4（581 年十二月乙亥、588 年正月庚午、635 年閏五月乙未早推步一日，600 年閏二月辛酉晚一日），
  638 年十月乙巳（与次月丙午朔二方不容，判书误），592 年閏正月丁未（推步是年无閏正月，闰 7 条中唯一不合者）；另卅日 1（622 年十月）。
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
4. **高昌历推步**（后段 δ = +0.026）下十二月朔壬辰、小月，残历有据之月 11 全合（[gaochang-630-checks.md](gaochang-630-checks.md)）。
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


# 高昌历推步：照 rust/core/src/calendar/gaochang.rs SEGMENTS（起年, δ, 满足最多之 δ 区间 [lo, hi)）
GC_SEG = [(523, -0.144, (-0.14497, -0.14307)), (617, 0.026, (0.01548, 0.03556))]
GC_YEARS = (523, 640)


def gaochang_table(d1=GC_SEG[0][1], d2=GC_SEG[1][1], cut=GC_SEG[1][0]):
    """正光常数、历元平移：cut 年以前 d1，cut 年起 d2 → {(年, 月, 闰): 朔 JDN}。"""
    P = OTHERS["正光"]
    return {(y, m, lp): j for y in range(GC_YEARS[0], GC_YEARS[1] + 1)
            for m, lp, j in pingshuo_year(dict(P, jd_epoch=P["jd_epoch"] + (d1 if y < cut else d2)), y)}


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


@lru_cache(maxsize=None)
def _leap_hits(pkey, y, mo, lstep):
    import numpy as np
    P = dict(pkey)
    lgrid = np.round(np.arange(-1.0, 1.0, lstep), 3)
    return np.array([any(m == mo and lp for m, lp, _ in pingshuo_year(dict(P, jd_epoch=P["jd_epoch"] + d), y)) for d in lgrid])


def delta_constraints(delta, P, rows):
    """以历 P 为底的 δ 约束：(朔日与卅日约束, 闰月约束, 闰月网格步)。每条约束 (标签, 年, 距 630 年十二月月数, 判定函数 δ→布尔)；
    标签以「（编号）」结尾（卅日为「（王素 编号）」），供留一检验剔除某条砖志。
    朔日：floor(m + δ + 0.5) = 所记朔日；卅日：该月为大月；闰月：平移 δ 后（气不动）当年有此闰月。只用麴氏高昌。"""
    import numpy as np
    xs = extrap_table(P, 530, 645)
    lunar = P["lunar"]
    ref = mean_conj(P, xs[(630, 12, False)])
    cons = []
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
    lstep = 0.005
    pkey = tuple(sorted(P.items()))
    lcons = []
    for r in rows:
        if r["闰"] != "1" or r.get("计入", "1") == "0" or r["政权"] != "麴氏高昌":
            continue
        y, mo = int(r["公元"]), int(r["月"])
        hit = _leap_hits(pkey, y, mo, lstep)
        n = round((mean_conj(P, xs[(y, mo, False)]) - ref) / lunar)
        lcons.append((f"閏 {y}年閏{MON[mo - 1]}月（{r['编号']}）", y, n,
                      lambda d, hit=hit: hit[np.clip(np.round((np.asarray(d) + 1.0) / lstep).astype(int), 0, len(hit) - 1)]))
    return cons, lcons, lstep


def add_630_delta(delta):
    """延寿七年（630）残历有据之月并入 δ 约束（以玄始外推为参照，记朔日区间）。"""
    xs = extrap_table(XUANSHI, 390, 645)
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


def loo_check(delta, rows, obs, cls, zhang):
    """留一检验（推步参数由砖志反推，推步与某砖志相合可能只是拟合所致）：对高昌推步判 ① 之月，剔除该月诸砖志（及其闰月约束），
    以其余约束重求该段 δ（满足最多之区间中、取离原值最近之一段的中点，步 0.001），重推该月。
    → {月: (留一 δ, 留一推步朔, 是否仍合砖志而异张表)}。"""
    import numpy as np
    P = OTHERS["正光"]
    cons, lcons, _ = delta_constraints(delta, P, rows)
    allc = cons + lcons
    grid = np.round(np.arange(-1.0, 1.0, 0.001), 3)
    (y1, d1, _), (y2, d2, _) = GC_SEG
    out = {}
    for k, (c, _, _) in cls.items():
        if c != "①":
            continue
        ids = {x[2] for x in obs[k]}
        later = k[0] >= y2
        cs = [x for x in allc if (x[1] >= y2) == later and not any(x[0].endswith(f"（{i}）") for i in ids)]
        sat = np.zeros_like(grid, dtype=int)
        for x in cs:
            sat += x[3](grid)
        best = sat == sat.max()
        # 最优点分成连续段，取离原 δ 最近之段的中点
        idx = np.flatnonzero(best)
        runs = np.split(idx, np.flatnonzero(np.diff(idx) > 1) + 1)
        d0 = d2 if later else d1
        run = min(runs, key=lambda r: min(abs(grid[r[0]] - d0), abs(grid[r[-1]] - d0)))
        dn = round((grid[run[0]] + grid[run[-1]] + 0.001) / 2, 3)
        t = gaochang_table(dn if not later else d1, dn if later else d2)
        sg = _g(t, k)
        rec = {x[0] for x in obs[k]}
        out[k] = (dn, sg, sg in rec and sg != _g(zhang, k))
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
    cons, lcons, lstep = delta_constraints(delta, P, rows)
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
    # 闰月并入：平移 δ 后该历当年闰位是否与高昌所记相合（δ 网格步 0.005，取最近格点；约束见 delta_constraints）
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


def month_obs(rows):
    """麴氏高昌有朔（明书或由日序逆推）之月 → [(朔干支, 等级, 编号)]：明书朔者 B（当时文字），只记日序干支者 D（纪日，同月两条才判）。"""
    obs = defaultdict(list)
    for r in rows:
        s = implied_shuo(r)
        if r["政权"] != "麴氏高昌" or r.get("计入", "1") == "0" or s is None:
            continue
        obs[(int(r["公元"]), int(r["月"]), r["闰"] == "1")].append((GZ[s], "B" if r["朔干支"] else "D", r["编号"]))
    return obs


def _g(t, k):
    if k in t:
        return GZ[(t[k] + 49) % 60]
    return "（无此月）" if k[0] in {x[0] for x in t} else ""


def classify_months(obs, t, zhang):
    """{月: (分类, 注, 等级)}；推步一家为 t。"""
    out = {}
    for k, v in obs.items():
        grade = "B" if any(x[1] == "B" for x in v) else "D"
        c, note = fl.classify(_g(t, k), _g(zhang, k), [x[0] for x in v], grade, "墓表" if grade == "B" else "纪日")
        out[k] = (c, note, grade if c not in ("无证",) else "—")
    return out


def fit_counts(t, rows):
    """某高昌历表对全部麴氏高昌证据的相合条数：朔（逐条）、闰（该表有此闰月）、卅日（该月大）、630 残历有据之月。"""
    out = {}
    q = [r for r in rows if r["政权"] == "麴氏高昌" and r.get("计入", "1") != "0"]
    sh = [r for r in q if implied_shuo(r) is not None]
    out["朔"] = (sum(judge(r, t)[1] == "✓" for r in sh), len(sh))
    lp = [r for r in q if r["闰"] == "1"]
    out["闰"] = (sum((int(r["公元"]), int(r["月"]), True) in t for r in lp), len(lp))
    starts = sorted(t.values())
    nxt = {j: starts[i + 1] for i, j in enumerate(starts[:-1])}
    ss = [r for r in csv.DictReader(l for l in open(ROOT / "data/calendar/高昌_卅日.csv", encoding="utf-8") if not l.startswith("#"))]
    out["卅日"] = (sum(nxt.get(t.get((int(r["公元"]), int(r["月"]), False)), 0) - t.get((int(r["公元"]), int(r["月"]), False), 0) == 30
                     for r in ss), len(ss))
    cl = [r for r in csv.DictReader(l for l in open(ROOT / "data/calendar/gaochang_630.csv", encoding="utf-8") if not l.startswith("#"))
          if r["依据"] != "推拟"]
    out["630 残历"] = (sum(_g(t, (int(r["年"]), int(r["月"]), r["闰"] == "1")) == r["朔干支"] for r in cl), len(cl))
    return out


MISMATCH_NOTES = {
    "8": "延昌七年「建於實沈，下旬二日，日維辛卯」：实沈作建申（七月）是王素之说（《砖集》作四月）；推步是年閏六月庚午朔，"
         "所推「七月庚午」即推步之閏六月，月名差一（正光不平移则閏七月，七月庚午合）。纪日一条、月份由解释而来，不判",
    "33": "600 年閏月：閏下月数漫漶，同年七月戊子朔只容閏二月；閏位合，墓表朔辛酉比推步庚申晚一日（须 δ ≥ +0.17，与其余前段之条不容）",
    "80": "581 年十二月乙亥朔：推步丙子，墓表早一日；同年无他方可参，待核",
    "91": "588 年正月庚午朔（只书廿五日、无日干支）：推步辛未，墓表早一日，待核",
    "99": "592 年閏正月丁未朔：推步是年无閏正月（正光闰法下须 δ ∈ [+0.23, +0.70)，与前段不容）；原砖今已漫漶，录文据罗振玉以来诸家",
    "101": "592 年五月乙亥朔：集注引王素当为乙巳（差一月），墓表书误",
    "106": "597 年閏六月：朔字下缺，「丙午」由十三日戊午逆推",
    "172": "635 年閏月乙未朔：月数未书，閏五月是推补；推步閏五月丙申，墓表早一日（须 δ < −0.169，同前段）",
    "176": "638 年十月乙巳朔：同年十一月丙午朔二方（177、178），十月乙巳与之不容（须十月 31 日），集注引王素推十月丙子；判墓表书误",
}


def fenlei_section(rows, zhang, gc, xs, zg, delta):
    """麴氏高昌有朔之月按月分类：高昌历推步（正光 + δ）与张表互异、相同；玄始、正光外推（不平移）作对照。
    高昌推步之 ① 须过留一检验（loo_check）：剔除该月砖志重求 δ 后推步仍合，才算推步与砖志两路独立；否则改 ③。"""
    obs = month_obs(rows)
    tang = Counter()
    for r in rows:
        if r.get("计入", "1") != "0" and implied_shuo(r) is not None and r["政权"] == "唐":
            tang[judge(r, zhang)[1][0]] += 1
    tabs = {"高昌推步": classify_months(obs, gc, zhang), "正光外推": classify_months(obs, zg, zhang),
            "玄始外推": classify_months(obs, xs, zhang)}
    loo = loo_check(delta, rows, obs, tabs["高昌推步"], zhang)
    raw_one = sum(1 for v in tabs["高昌推步"].values() if v[0] == "①")
    for k, (dn, sg, ok) in loo.items():
        c, note, g = tabs["高昌推步"][k]
        if ok:
            tabs["高昌推步"][k] = ("①", f"留一通过：剔除本月砖志重求 δ = {dn:+.3f}，推步仍{sg}" + (f"；{note}" if note else ""), g)
        else:
            tabs["高昌推步"][k] = ("③", f"留一不过：剔除本月砖志重求 δ = {dn:+.3f}，推步作{sg}，与砖志之合系 δ 拟合所致；倾向砖志", g)
    n_pass = sum(1 for v in loo.values() if v[2])
    out = ["", "## 分类（按月）", "",
           "分类对象是**高昌历推步**（正光常数 + 两段历元平移，见下「高昌历推步」）与张表，限麴氏高昌有朔（明书或由日序逆推）之月。"
           "墓表、衣物疏明书朔者是当时文字（B 级），一条即可判，同月诸条冲突归③；只记日序与日干支、不书朔者作纪日（D 级），"
           "同月两条以上独立才判，一条者相同归无证、互异归③。同月诸方各算一条。一家无此月（闰位异）者亦算互异。"
           "张表在此段是中原历，不代表高昌历日，①即「张表不合高昌所行之历」（不是说张表作为中原历表有误）。δ 是由这些砖志反推的参数，"
           "推步与某砖志相合可能只是拟合所致，故**①须过留一检验**：剔除该月砖志（朔、闰约束）后以其余约束重求该段 δ"
           "（满足最多之区间、取离原值最近一段之中点），重推该月仍合砖志而异张表，才算砖志与推步两路独立、判 ①；否则改 ③（倾向砖志）。"
           "中原各朝推步出自术文、与砖志墓志无关，不需此检验。「有证」同理不全是独立检验（见下「高昌历推步」一节之说明）。" + fl.RULE, "",
           f"留一检验：原判 ① 之月 {raw_one}，过者 {n_pass}，改 ③ {raw_one - n_pass}（逐月见下表「分类（高昌推步）」括注）。", ""]
    for name in ("高昌推步", "正光外推", "玄始外推"):
        cats = [(c, g) for c, _, g in tabs[name].values() if c]
        lab = {"高昌推步": "**高昌历推步（正光 + δ）**", "正光外推": "正光外推（不平移，对照）", "玄始外推": "玄始外推（不平移，对照）"}[name]
        out.append(f"- {lab}：有朔之月 {len(cats)}，" + fl.headline(cats, name) + "。")
    out += ["", "高昌历推步按等级：", ""] + fl.grade_table([(c, g) for c, _, g in tabs["高昌推步"].values() if c])
    # 新旧分类对照
    cross = Counter((tabs["正光外推"][k][0], tabs["高昌推步"][k][0]) for k in obs)
    out += ["", "旧分类（正光外推，不平移）→ 新分类（高昌历推步）逐月对照（只列有变动者）：", "",
            "| 正光外推 | 高昌历推步 | 月数 |", "|---|---|---|"]
    for (a, b), n in sorted(cross.items(), key=lambda kv: (fl.CATS.index(kv[0][0]), fl.CATS.index(kv[0][1]))):
        if a != b:
            out.append(f"| {a} | {b} | {n} |")
    out += ["", f"唐西州（640 以后）行唐历，张表即唐历，无另一家可比：有朔之条合张表 {tang['✓']}、不合 {tang['✗']}，不入分类。", "",
            "逐月（全列；「所记朔」后括注砖志编号，等级 B 明书朔、D 只记日序干支）：", "",
            "| 公元 | 月 | 所记朔 | 等级 | 张表 | 高昌推步 | 分类（高昌推步） | 正光外推 | 分类（正光，对照） | 玄始外推 | 分类（玄始，对照） |",
            "|---|---|---|---|---|---|---|---|---|---|---|"]
    f = lambda c: (c[0] or "—") + (f"（{c[1]}）" if c[1] else "")
    for k in sorted(obs):
        lab = ("閏" if k[2] else "") + MON[k[1] - 1] + "月"
        rec = "、".join(f"{x[0]}（{x[2]}）" for x in obs[k])
        grade = "B" if any(x[1] == "B" for x in obs[k]) else "D"
        out.append(f"| {k[0]} | {lab} | {rec} | {grade} | {_g(zhang, k) or '—'} | {_g(gc, k) or '—'} | {f(tabs['高昌推步'][k])} | "
                   f"{_g(zg, k) or '—'} | {f(tabs['正光外推'][k])} | {_g(xs, k) or '—'} | {f(tabs['玄始外推'][k])} |")
    return out


def gaochang_section(rows, gc, zhang):
    """高昌历推步：δ 取值、与证据相合条数、敏感性（δ 区间两端、切点）与未合之条。"""
    base = fit_counts(gc, rows)
    tot = lambda c: (sum(v[0] for v in c.values()), sum(v[1] for v in c.values()))
    obs = month_obs(rows)

    def head(t):
        cats = [(c, g) for c, _, g in classify_months(obs, t, zhang).values() if c]
        return fl.headline(cats, "推步")
    (y1, d1, iv1), (y2, d2, iv2) = GC_SEG
    out = ["", "## 高昌历推步（正光常数 + 两段历元平移）", "",
           "**性质**：按术推步 + 据实物定参。朔策、岁实、闰法用北魏正光历术文常数（`push_calendar::zhengguang_params`）；"
           "历元平移 δ（平朔时刻一律加 δ 日，冬至历元不动）不见于任何术文或史书，是由本页砖志、卅日、闰月反推的参数。"
           "史书不载高昌所行之历；「行正光历而历元另定」是由比对得出的解释，不是记载。",
           "",
           f"**取值**：下文「正光平朔整体平移 δ」一节的稳健分析（184 条约束，按「满足条数最多」求 δ，步长 10⁻⁵ 复算）："
           f"{y2 - 1} 年以前满足最多之区间 [{iv1[0]:+.5f}, {iv1[1]:+.5f})（121/126；下端由 565 年四月朔（60）、上端由 573 年三月朔（71）定），"
           f"{y2} 年起 [{iv2[0]:+.5f}, {iv2[1]:+.5f})（55/58；下端由 638 年十一月丙午朔（177、178）、上端由 637 年七月卅日（王素 934）定）。"
           f"区间内各点满足条数相同，**取区间之中**（舍入到 0.001 日）：前段 δ = {d1:+.3f} 日，后段 δ = {d2:+.3f} 日（推后约 {(d2 - d1) * 24:.1f} 小时）。"
           f"**切点**取前后两段合计满足最多（176/184）之「后段始于 {y2} 年」：{y2 - 1} 年（含）以前用前段，{y2} 年起用后段。"
           "逐条看，616 年閏五月须前段、617 年十一月戊申朔须后段，改历元在二者之间；后端只靠 617 年一方，若除去则可晚至 621 年。",
           "",
           f"**年限** {GC_YEARS[0]}—{GC_YEARS[1]}：正光历北魏 523 年行用，高昌得之不能更早（起年是推定，不是记载；有朔之砖志最早 537 年，"
           "523—536 为外推、未经检验）；640 年（延寿十七年）八月唐平高昌置西州，此后行唐历，归中原一线（唐西州 640—662 年 30 条合张表 27）。"
           "此表经推步政权表供换算（政权名「高昌」），换算可给出高昌历日。",
           "",
           "**与证据相合**（朔逐条；闰 = 该表当年有此闰月；卅日 = 该月大；630 残历 = 有据之月）：",
           "", "| 方案 | 朔 | 闰 | 卅日 | 630 残历 | 合计 | 分类 |", "|---|---|---|---|---|---|---|"]

    def row(lab, t):
        c = fit_counts(t, rows)
        a, n = tot(c)
        return f"| {lab} | " + " | ".join(f"{c[k][0]}/{c[k][1]}" for k in ("朔", "闰", "卅日", "630 残历")) + f" | {a}/{n} | {head(t)} |"
    out.append(row(f"**取值**（前 {d1:+.3f}，后 {d2:+.3f}，切 {y2}）", gc))
    out.append(row("正光外推（不平移）", {k: v for k, v in extrap_table(OTHERS["正光"], *GC_YEARS).items()}))
    out.append(row("玄始外推（不平移）", {k: v for k, v in extrap_table(XUANSHI, *GC_YEARS).items()}))
    out += ["", "**敏感性**（每次只改一项，余同取值）：", "", "| 方案 | 朔 | 闰 | 卅日 | 630 残历 | 合计 | 分类 |", "|---|---|---|---|---|---|---|"]
    for lab, kw in ((f"前段 δ 取区间下端 {iv1[0]:+.5f}", dict(d1=iv1[0])), (f"前段 δ 取区间上端 {iv1[1] - 1e-5:+.5f}", dict(d1=iv1[1] - 1e-5)),
                    (f"前段 δ 刚出下端 {iv1[0] - 1e-5:+.5f}", dict(d1=iv1[0] - 1e-5)), (f"前段 δ 刚出上端 {iv1[1]:+.5f}", dict(d1=iv1[1])),
                    (f"后段 δ 取区间下端 {iv2[0]:+.5f}", dict(d2=iv2[0])), (f"后段 δ 取区间上端 {iv2[1] - 1e-5:+.5f}", dict(d2=iv2[1] - 1e-5)),
                    (f"后段 δ 刚出下端 {iv2[0] - 1e-5:+.5f}", dict(d2=iv2[0] - 1e-5)), (f"后段 δ 刚出上端 {iv2[1]:+.5f}", dict(d2=iv2[1])),
                    ("前段 δ 出区间：−0.150", dict(d1=-0.150)), ("前段 δ 出区间：−0.135", dict(d1=-0.135)),
                    ("后段 δ 出区间：+0.010", dict(d2=0.010)), ("后段 δ 出区间：+0.040", dict(d2=0.040)),
                    ("切点：后段始于 616", dict(cut=616)), ("切点：后段始于 618", dict(cut=618)),
                    ("切点：后段始于 620", dict(cut=620)), ("切点：后段始于 622", dict(cut=622)),
                    ("不分段：一律 −0.144", dict(d2=-0.144)), ("不分段：一律 +0.026", dict(d1=0.026))):
        out.append(row(lab, gaochang_table(**kw)))
    out += ["", "区间两端与取中所得逐条全同（区间即「满足条数最多」之集合，区间内各点的朔日、闰位无一变动）；刚出区间（10⁻⁵ 日）即失一二条："
            "前段出下端失 565 年四月朔（60），出上端失 573 年三月朔（71）；后段出下端失 638 年十一月丙午朔二方（177、178），出上端失 637 年七月卅日。"
            "切点移到 616 年失 616 年閏五月，移到 618、620 年失 617 年十一月戊申朔，移到 622 年再失 621 年之条；不分段则合计少 9 条以上。", ""]
    # 未合之条
    out += ["### 未能合之条（高昌历推步）", "", "| 编号 | 公元 | 月 | 所记 | 高昌推步 | 张表 | 说明 |", "|---|---|---|---|---|---|---|"]
    for r in rows:
        if r["政权"] != "麴氏高昌" or r.get("计入", "1") == "0":
            continue
        y, mo, lp = int(r["公元"]), int(r["月"]), r["闰"] == "1"
        lab = ("閏" if lp else "") + MON[mo - 1] + "月"
        s = implied_shuo(r)
        if s is not None:
            sg, v = judge(r, gc)
            if v == "✓":
                continue
            zg_, _ = judge(r, zhang)
            out.append(f"| {r['编号']} | {y} | {lab} | 朔{GZ[s]} | {sg} {v} | {zg_} | {MISMATCH_NOTES.get(r['编号'], fl.off_note(GZ[s], sg) if sg in GZ else '闰位异')} |")
        elif lp and (y, mo, True) not in gc:
            ls = [f"閏{MON[m - 1]}" for (yy, m, l) in gc if yy == y and l]
            out.append(f"| {r['编号']} | {y} | {lab} | 有此闰月 | {'、'.join(ls) or '无闰'} | — | {MISMATCH_NOTES.get(r['编号'], '闰位异')} |")
    starts = sorted(gc.values())
    nxt = {j: starts[i + 1] for i, j in enumerate(starts[:-1])}
    for r in csv.DictReader(l for l in open(ROOT / "data/calendar/高昌_卅日.csv", encoding="utf-8") if not l.startswith("#")):
        k = (int(r["公元"]), int(r["月"]), False)
        if k in gc and nxt[gc[k]] - gc[k] != 30:
            out.append(f"| 王素 {r['编号']} | {k[0]} | {MON[k[1] - 1]}月 | 卅日（月大） | 小月（{_g(gc, k)}朔） | 张表 {r['张表日数']} 日 | "
                       f"{'重光三年十月卅日：须 δ ∈ [+0.073, +0.604)，与 637 年七月卅日（须 δ < +0.036）不容；王素书只录卅日、未见原文，待核' if k[0] == 622 else '待核'} |")
    for r in csv.DictReader(l for l in open(ROOT / "data/calendar/gaochang_630.csv", encoding="utf-8") if not l.startswith("#")):
        if r["依据"] == "推拟":
            continue
        k = (int(r["年"]), int(r["月"]), r["闰"] == "1")
        if _g(gc, k) != r["朔干支"]:
            out.append(f"| 630 残历 | {k[0]} | {MON[k[1] - 1]}月 | 朔{r['朔干支']} | {_g(gc, k)} | {_g(zhang, k)} | 见 gaochang-630-checks.md |")
    out.append("")
    return out


def main():
    rows = load()
    tabs = d.tables()
    zhang = tabs["张表"]
    beiliang = tabs.get("推步:北凉", {})
    beiwei = tabs.get("推步:北魏/西魏/北周/隋", {})
    gc = tabs["推步:高昌"]  # Rust 导出（rust/core/src/calendar/gaochang.rs）
    mine = gaochang_table()
    assert gc == mine, "推步政权朔日表之高昌与本脚本 GC_SEG 重算不一致（先跑 export_regime_calendars，或同步 GC_SEG）：" + \
        str(sorted(set(gc.items()) ^ set(mine.items()))[:6])
    xs = extrap_table(XUANSHI, 390, 645)
    others = {k: extrap_table(p, 390, 645) for k, p in OTHERS.items()}

    md = ["# 吐鲁番出土文书纪日与高昌历推步、张表（自动生成）", "",
          "> 由 `scripts/calendar_data/tulufan_jiri_check.py` 生成，勿手改。录文 `data/calendar/tulufan_jiri.csv`"
          "（《吐鲁番出土文书》图录本壹—肆册、王素书所引墓表、荒川与关尾所录墓表、《西域考古图谱》墓表书影），依据与列义见该文件注释。", ""]
    md += NOTES.strip("\n").splitlines()
    md += ["", "## 逐条", "",
           "「记朔」为所记或由日序逆推之朔干支；「—」为该表无此年。推步北凉限 398—439，推步北魏限 398—590。"
           "「高昌推步」为高昌历推步（正光 + δ，523—640）。「玄始小余」为与所记朔日最近的玄始平朔的日内小数（0 为夜半；无朔者取同名之月）。「（不计）」者只列，不入汇总与 δ。", "",
           "| # | 政权 | 纪年 | 公元 | 月 | 原文（节） | 记朔 | 张表 | 高昌推步 | 推步北凉 | 推步北魏 | 玄始外推 | 玄始小余 | 合之外推历 | 岁干支 | 出处 | 核对 |",
           "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|"]
    cnt = defaultdict(Counter)
    delta = []
    for r in rows:
        use = r.get("计入", "1") != "0"
        kind = "朔" if implied_shuo(r) is not None else ("闰" if r["闰"] == "1" and not r["干支"] else "日")
        s = implied_shuo(r)
        rec = GZ[s] if s is not None else (f"（{r['干支']}在月内）" if r["干支"] else "（有此闰月）")
        res = {}
        for name, t in (("张表", zhang), ("高昌推步", gc), ("推步北凉", beiliang), ("推步北魏", beiwei), ("玄始外推", xs)):
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
        for name in ["张表", "高昌推步", "推步北凉", "推步北魏", "玄始外推"] + list(OTHERS):
            c = cnt[f"{kind}|{name}"]
            md.append(f"| {name}{'（外推）' if name in OTHERS else ''} | {c['✓']} | {c['✗']} | {c['邻']} |")
        md += ["", f"其中麴氏高昌（{title[:2]}）：", "", "| 历表 | 合 | 不合 | 其中合前后月 |", "|---|---|---|---|"]
        for name in ["张表", "高昌推步", "推步北魏", "玄始外推"] + list(OTHERS):
            c = cnt[f"{kind}|{name}|麴氏高昌"]
            md.append(f"| {name}{'（外推）' if name in OTHERS else ''} | {c['✓']} | {c['✗']} | {c['邻']} |")
        md += ["", "| 政权 | 张表 合/不合 | 高昌推步 | 推步北凉 | 推步北魏 | 玄始外推 |", "|---|---|---|---|---|---|"]
        for g in regs:
            cells = []
            for name in ["张表", "高昌推步", "推步北凉", "推步北魏", "玄始外推"]:
                c = cnt[f"{kind}|{name}|{g}"]
                cells.append(f"{c['✓']}/{c['✗']}" if c else "—")
            if any(x != "—" for x in cells):
                md.append(f"| {g} | " + " | ".join(cells) + " |")
        md.append("")

    add_630_delta(delta)
    md += fenlei_section(rows, zhang, gc, xs, others["正光"], delta)
    md += gaochang_section(rows, gc, zhang)
    # δ 区间
    md += ["", "## 玄始外推的常数偏移 δ", "",
           "设高昌实用之历的平朔比本项目玄始常数一律晚 δ 日（δ>0 即合朔时刻推后），各条有据之朔容许的 δ 区间如下"
           "（含 630 年残历诸月，录自 data/calendar/gaochang_630.csv 之有据者）。", "",
           "| 来源 | 公元 | 月 | 记朔 | 玄始小余 | δ 下限 | δ 上限 |", "|---|---|---|---|---|---|---|"]
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
