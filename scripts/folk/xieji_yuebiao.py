#!/usr/bin/env python3
"""
《欽定協紀辨方書》月表（卷二十至三十一）的参考推算：由「月 × 日干支」推出当日吉神、凶神与宜、忌。

依据：
  - 月神取值：data/folk/shensha_month.csv（卷五、卷六起例）与本文件补充的建除异名、叢辰（卷四）
  - 各神宜忌：data/folk/yiji_shi.csv（卷十一，以事为纲）
  - 相併相抵：卷十各条与宜忌等第表 data/folk/yiji_dengdi.csv
  - 铺注：卷十「鋪註條例」
输出：data/folk/xieji_yuebiao.csv（720 行）。
用法（仓库根）：python3 scripts/folk/xieji_yuebiao.py [OCR JSON]
  给出 OCR JSON（scripts/folk/yuebiao_ocr.py 的输出）时，逐日与月表 OCR 比对并报告差异。
"""

import csv
import json
import re
import sys

GAN = "甲乙丙丁戊己庚辛壬癸"
ZHI = "子丑寅卯辰巳午未申酉戌亥"
JC = "建除滿平定執破危成收開閉"
NAYIN = ["海中金", "鑪中火", "大林木", "路傍土", "劍鋒金", "山頭火", "澗下水", "城頭土", "白鑞金", "楊柳木",
         "井泉水", "屋上土", "霹靂火", "松柏木", "長流水", "砂石金", "山下火", "平地木", "壁上土", "金箔金",
         "覆燈火", "天河水", "大驛土", "釵釧金", "桑柘木", "大溪水", "沙中土", "天上火", "石榴木", "大海水"]
VAR = str.maketrans({"刼": "劫", "夘": "卯", "隂": "陰", "徳": "德", "虚": "虛", "摇": "搖", "絶": "絕"})


def gz_name(k):
    return GAN[k % 10] + ZHI[k % 12]


# ---------------- 月神 ----------------
MONTH = {}  # 名 -> (吉, 依, [12 个集合])
for r in csv.DictReader(open("data/folk/shensha_month.csv", encoding="utf-8")):
    name = r["名"].translate(VAR)
    vals = [set() if v == "-" else set(v.split("+")) for v in r["正月至十二月（|分隔，+並列，-無）"].split("|")]
    MONTH[name] = (r["吉凶"] == "吉", r["依"], vals)


def add_month(name, good, kind, vals):
    MONTH[name] = (good, kind, [set(v) if isinstance(v, (set, list)) else {v} for v in vals])


# 卷四：月厭正月起戌逆行，厭對正月起辰逆行（即六儀），天賊正月起丑逆行（卷六）
add_month("月厭", False, "支", [ZHI[(10 - m) % 12] for m in range(12)])
add_month("厭對", False, "支", [ZHI[(4 - m) % 12] for m in range(12)])
add_month("六儀", True, "支", [ZHI[(4 - m) % 12] for m in range(12)])
add_month("天賊", False, "支", [ZHI[(1 - m) % 12] for m in range(12)])
# 月刑：月表卷首所载（与卷三歲刑同）
add_month("月刑", False, "支", list("巳子辰申午丑寅酉未亥卯戌"))
# 天狗：卷四按「天狗者戌日值滿也」，即申月滿日
add_month("天狗", False, "支", [set(), set(), set(), set(), set(), set(), {"戌"}, set(), set(), set(), set(), set()])
# 不將：卷四引歷例（八月「癸己」作癸巳）
BUJIANG = """辛亥辛丑辛卯庚子庚寅己亥己丑己卯丁亥丁丑丁卯丙子丙寅
庚戌庚子庚寅己亥己丑丁亥丁丑丙戌丙子丙寅乙亥乙丑
己酉己亥己丑丁酉丁亥丁丑丙戌丙子乙酉乙亥乙丑甲戌甲子
丁酉丁亥丙申丙戌丙子乙酉乙亥甲申甲戌甲子戊申戊戌戊子
丙申丙戌乙未乙酉乙亥甲申甲戌戊申戊戌癸未癸酉癸亥
乙未乙酉甲午甲申甲戌戊午戊申戊戌癸未癸酉壬午壬申壬戌
乙巳乙未乙酉甲午甲申戊午戊申癸巳癸未癸酉壬午壬申
甲辰甲午甲申戊辰戊午戊申癸巳癸未壬辰壬午壬申辛巳辛未
戊辰戊午癸卯癸巳癸未壬辰壬午辛卯辛巳辛未庚辰庚午
癸卯癸巳壬寅壬辰壬午辛卯辛巳庚寅庚辰庚午己卯己巳
壬寅壬辰辛丑辛卯辛巳庚寅庚辰己丑己卯己巳丁丑丁卯丁巳
辛丑辛卯庚子庚寅庚辰己丑己卯丁丑丁卯丙子丙寅丙辰""".split("\n")
add_month("不將", True, "干支", [{l[i:i + 2] for i in range(0, len(l), 2)} for l in BUJIANG])

# 卷十月厭條所列特定日（「諸事皆忌」等），按月：名 -> {月序: {干支}}
SPECIAL = {
    "大會": {0: "甲戌", 1: "乙酉", 4: "丙午", 5: "丁巳", 6: "庚辰", 7: "辛卯", 10: "壬子", 11: "癸亥"},
    "小會": {1: "己卯", 2: "戊辰", 3: "己巳", 4: "戊午", 7: "己酉", 8: "戊戌", 9: "己亥", 10: "戊子"},
    "單陰": {2: "戊辰"}, "純陽": {3: "己巳"}, "孤陽": {8: "戊戌"}, "純陰": {9: "己亥"},
    "陰位": {2: "庚辰", 8: "甲戌"},
    "陰陽交破": {3: "癸亥", 9: "丁巳"}, "陰陽擊衝": {4: "壬子", 10: "丙午"},
    "陽破陰衝": {5: "癸丑", 11: "丁未"}, "陰道衝陽": {1: "己酉", 7: "己卯"},
    "歲薄": {3: "丙午戊午", 9: "壬子戊子"}, "逐陣": {5: "丙午戊午", 11: "壬子戊子"},
    "三陰": {0: "辛酉", 6: "乙卯"},
    "陽錯": {0: "甲寅", 1: "乙卯", 2: "甲辰", 3: "丁巳己巳", 5: "丁未己未", 6: "庚申", 7: "辛酉", 8: "庚戌",
           9: "癸亥", 11: "癸丑"},
    "陰錯": {0: "庚戌", 1: "辛酉", 2: "庚申", 3: "丁未己未", 5: "丁巳己巳", 6: "甲辰", 7: "乙卯", 8: "甲寅",
           9: "癸丑", 11: "癸亥"},
    "陰陽俱錯": {4: "丙午", 10: "壬子"}, "絕陰": {3: "戊辰"}, "絕陽": {9: "戊戌"},
    # 卷四：行狠、了戾、孤辰
    "行狠": {2: "甲申", 3: "乙未", 8: "庚寅", 9: "辛丑"},
    "了戾": {2: "丙申", 3: "丁未", 8: "壬寅", 9: "癸丑"},
    "孤辰": {2: "戊申庚申壬申", 3: "己未辛未癸未", 8: "甲寅丙寅戊寅", 9: "乙丑丁丑己丑"},
}
for name, d in SPECIAL.items():
    add_month(name, False, "干支", [{d[m][i:i + 2] for i in range(0, len(d[m]), 2)} if m in d else set()
                                    for m in range(12)])

# 与月份无关、按干支固定者（卷五、卷九立成）
FIXED = {
    "天恩": (True, "甲子乙丑丙寅丁卯戊辰己卯庚辰辛巳壬午癸未己酉庚戌辛亥壬子癸丑"),
    "五合": (True, "甲寅乙卯丙寅丁卯戊寅己卯庚寅辛卯壬寅癸卯"),
    "鳴吠": (True, "甲午丙午庚午壬午甲申丙申庚申壬申乙酉丁酉己酉辛酉癸酉"),
    "鳴吠對": (True, "丙子庚子壬子甲寅丙寅庚寅壬寅乙卯丁卯辛卯癸卯"),
    "除神": (True, "甲申丙申戊申庚申壬申乙酉丁酉己酉辛酉癸酉"),
    "五離": (False, "甲申丙申戊申庚申壬申乙酉丁酉己酉辛酉癸酉"),
    "重日": (False, "乙巳丁巳己巳辛巳癸巳乙亥丁亥己亥辛亥癸亥"),
    "八專": (False, "甲寅丁未己未庚申癸丑"),
    "觸水龍": (False, "丙子癸未癸丑"),
    "無祿": (False, "甲辰乙巳丙申丁亥戊戌己丑庚辰辛巳壬申癸亥"),
}
FIXED = {k: (g, {v[i:i + 2] for i in range(0, len(v), 2)}) for k, (g, v) in FIXED.items()}

# 建除十二神同位异名（卷四）；平、收之魁罡按陽建、陰建月分
JC_ALIAS = {"建": ["建日", "月建", "小時", "土府"], "除": ["除日", "吉期"], "滿": ["滿日", "天巫", "福德"],
            "平": ["平日", "死神"], "定": ["定日", "時陰", "死氣"], "執": ["執日", "小耗"], "破": ["月破", "大耗"],
            "危": ["危日"], "成": ["成日", "天喜", "天醫"], "收": ["收日"], "開": ["開日", "時陽", "生氣"],
            "閉": ["閉日", "血支"]}
JIXIONG = {  # 建除异名的吉凶
    "建日": True, "月建": False, "小時": False, "土府": False, "除日": True, "吉期": True, "滿日": True, "天巫": True,
    "福德": True, "平日": False, "死神": False, "定日": True, "時陰": True, "死氣": False, "執日": True,
    "小耗": False, "月破": False, "大耗": False, "危日": True, "成日": True, "天喜": True, "天醫": True,
    "收日": False, "開日": True, "時陽": True, "生氣": True, "閉日": False, "血支": False, "天罡": False,
    "河魁": False,
}
HH = ["青龍", "明堂", "天刑", "朱雀", "金匱", "寶光", "白虎", "玉堂", "天牢", "元武", "司命", "勾陳"]
HUANGDAO = {"青龍", "明堂", "金匱", "寶光", "玉堂", "司命"}
# 异名（卷五、卷六标题〔〕内）：正名 -> 异名
ALIAS = {"災煞": ["天火"], "月煞": ["月虛"], "大時": ["大敗", "咸池"], "天吏": ["致死"], "驛馬": ["天后"],
         "九坎": ["九焦"], "續世": ["血忌"], "月厭": ["地火"], "厭對": ["招搖"]}
SIJI_ALIAS = {"甲子": "八龍", "乙亥": "八龍", "丙子": "七鳥", "丁亥": "七鳥", "庚子": "九虎", "辛亥": "九虎",
              "壬子": "六蛇", "癸亥": "六蛇"}


def shen_of(m, k):
    """第 m 月（寅月 = 0）干支序 k 之日所值诸神：{名: 吉}"""
    gz = gz_name(k)
    g, z = gz[0], gz[1]
    out = {}
    for name, (good, kind, vals) in MONTH.items():
        key = {"干": g, "支": z, "干支": gz}[kind]
        if key in vals[m]:
            out[name] = good
    for name, (good, days) in FIXED.items():
        if name == "無祿":  # 卷十無祿日條：「月表與萬年書俱不論無祿日」
            continue
        if gz in days:
            out[name] = good
    jc = JC[(ZHI.index(z) - (m + 2)) % 12]
    for a in JC_ALIAS[jc]:
        out[a] = JIXIONG[a]
    yang_month = (m + 2) % 2 == 0  # 寅辰午申戌子为陽建
    if jc == "平":
        out["天罡" if yang_month else "河魁"] = False
    if jc == "收":
        out["河魁" if yang_month else "天罡"] = False
    out[HH[(ZHI.index(z) - (2 * (m + 2) - 4)) % 12]] = HH[(ZHI.index(z) - (2 * (m + 2) - 4)) % 12] in HUANGDAO
    for main, al in ALIAS.items():
        if main in out:
            for a in al:
                out[a] = out[main]
    if "血忌" in out:  # 續世之异名，为凶神（月表列於凶神）
        out["血忌"] = False
    if "四忌" in out or "四窮" in out:
        out[SIJI_ALIAS[gz]] = False
    # 亥子日、午申日（卷十所宜）
    if z in "亥子":
        out["亥子日"] = True
    return out, jc


# ---------------- 事目与宜忌 ----------------
# 月表所用通书六十事（卷十一），合并显示单位：卷十一条目名
SHI60 = [("祭祀", "祭祀"), ("祈福", "祈福"), ("求嗣", "求嗣"), ("上冊受封", "上冊進表章"), ("上表章", "上表章"),
         ("襲爵受封", "施恩封拜"), ("會親友", "宴會"), ("入學", "入學"), ("冠帶", "冠帶"), ("出行", "行幸遣使"),
         ("上官赴任", "上官赴任"), ("臨政親民", "臨政親民"), ("結婚姻", "結婚姻"), ("納采問名", "納采問名"),
         ("嫁娶", "嫁娶"), ("進人口", "進人口"), ("移徙", "般移"), ("遠迴", "遠廻"), ("安牀", "安牀"),
         ("解除", "解除"), ("沐浴", "沐浴"), ("剃頭", "整容剃頭"), ("整手足甲", "整手足甲"),
         ("求醫療病", "求醫療病"), ("療目", "療目"), ("針刺", "針刺"), ("裁衣", "裁製"), ("築隄防", "築隄防"),
         ("修造動土", "興造動土"), ("豎柱上梁", "豎柱上梁"), ("修倉庫", "修倉庫"), ("鼓鑄", "鼓鑄"),
         ("苫蓋", "苫蓋"), ("經絡", "經絡"), ("醞釀", "醞釀"), ("開市", "開市"), ("立券交易", "立券交易"),
         ("納財", "納財"), ("開倉庫出貨財", "開倉庫出貨財"), ("修置產室", "修置產室"), ("開渠穿井", "開渠穿井"),
         ("安碓磑", "安碓磑"), ("補垣塞穴", "補垣塞穴"), ("掃舍宇", "掃舍宇"), ("修飾垣墻", "修飾垣墻"),
         ("平治道塗", "平治道塗"), ("破屋壞垣", "破屋壞垣"), ("伐木", "伐木"), ("捕捉", "捕捉"), ("畋獵", "畋獵"),
         ("取魚", "取魚"), ("乘船渡水", "乘船渡水"), ("栽種", "栽種"), ("牧養", "牧養"), ("納畜", "納畜"),
         ("破土", "破土"), ("安葬", "安葬"), ("啟攢", "啟攢")]
YI, JI = {}, {}  # 卷十一条目 -> 神集合
for r in csv.DictReader(open("data/folk/yiji_shi.csv", encoding="utf-8")):
    if r["類"] in ("宜", "忌") and r["神"]:
        d = YI if r["類"] == "宜" else JI
        d.setdefault(r["事"].translate(VAR), set()).update(x.translate(VAR) for x in r["神"].split("|"))


def row_key(name):
    for k in YI.keys() | JI.keys():
        if k.replace("劵", "券") == name or k == name:
            return k
    return name


SHI_ROW = {disp: row_key(r) for disp, r in SHI60}
# 卷十一伐木、畋獵、取魚的宜神附节气条件（立冬後立春前等），改由下面按月处理
for r_ in ("伐木", "畋獵", "取魚"):
    YI[r_] = YI.get(r_, set()) - {"危日", "執日", "收日", "午日", "申日"}
assert all(SHI_ROW[d] in YI or SHI_ROW[d] in JI for d, _ in SHI60), [d for d, _ in SHI60 if SHI_ROW[d] not in YI and SHI_ROW[d] not in JI]

DE = {"天德", "月德", "天德合", "月德合", "天赦", "天願"}
DE4 = {"天德", "月德", "天德合", "月德合"}
MAJOR = ["平日", "收日", "閉日", "劫煞", "災煞", "月煞", "月刑", "月害", "月厭", "大時", "天吏"]
# 主凶神的异名，宜忌并入主凶神一组
MAJOR_ALIAS = {"死神", "天火", "月虛", "大敗", "咸池", "致死", "地火", "血支", "河魁", "天罡"}
GRADE_ORDER = ["上", "上次", "中", "中次", "下", "下下"]
DENGDI = {}
for r in csv.DictReader(open("data/folk/yiji_dengdi.csv", encoding="utf-8")):
    for mz in r["月"]:
        DENGDI[(r["凶神"].translate(VAR), mz)] = r["等第"]
RESIDUAL = {"求醫療病"}  # 「止忌安撫邊境選將訓兵出師求醫療病」中属六十事者
RESIDUAL_YAN = {"出行", "上官赴任", "臨政親民", "結婚姻", "納采問名", "嫁娶", "求醫療病", "移徙", "遠迴", "栽種"}
TUGONG = {"築隄防", "修造動土", "修倉庫", "修置產室", "開渠穿井", "安碓磑", "補垣塞穴", "修飾垣墻", "平治道塗",
          "破屋壞垣", "伐木", "栽種", "破土"}  # 月建與德併「止忌」者（卷十月建條）中属六十事者
# 百忌日（卷十、卷十一）：作为「專忌」
BAIJI = {"甲": {"開倉庫出貨財"}, "乙": {"栽種"}, "丁": {"剃頭"}, "庚": {"經絡"}, "辛": {"醞釀"},
         "丑": {"冠帶"}, "寅": {"祭祀"}, "巳": {"出行"}, "午": {"苫蓋"}, "未": {"求醫療病"}, "申": {"安牀"},
         "酉": {"會親友"}, "亥": {"嫁娶"}, "卯": {"開渠穿井"}, "壬": {"開渠穿井"}}
# 卷十：與德等併則不忌者（神 -> 解神集合）；「猶忌」者不列
# 卷十八風、復日重日條并列「六合」，但月表實例（四月甲申八風、正月己亥癸亥重日，皆值六合）仍忌，
# 而同日五離、五虛之忌則解；今從月表：六合只解五虛、五離；復日重日由天願解（夾注「天願必是六合」）
#（見 doc/folk/shensha.md）
JIE = {"厭對": DE4 | {"天赦"}, "招搖": DE4 | {"天赦"}, "小耗": DE4 | {"天願"}, "五虛": DE4 | {"六合"},
       "八風": DE4, "復日": DE4 | {"天赦", "天願"}, "重日": DE4 | {"天赦", "天願"},  # 「天願必是六合故不言而已在其中」
       "五離": DE4 | {"天赦", "三合", "六合"}}
# 與某併「止忌軍事」（六十事中即全不忌）者：八專與天願併（卷十八專條）
JIE["八專"] = {"天願"}
# 與德併「止忌軍事療病」者
ZHIJI = {"死氣": DE4}
# 卷十一：「每月某日」等非干支所定者，月表不載
NOT_IN_YUEBIAO = {"每月十五日", "每月十二日十五日", "每月一日六日十五日十九日二十一日二十三日", "朔望日",
                  "土王用事後", "伏社日", "立冬後立春前", "霜降後立春前", "雨水後立夏前"}


def yiji(m, k):
    shen, jc = shen_of(m, k)
    gz = gz_name(k)
    g, z = gz[0], gz[1]
    names = set(shen)
    mz = ZHI[(m + 2) % 12]
    de = names & DE
    # 陽錯陰錯、陰陽俱錯、大會：德作建論不作德論（卷十月厭條）
    if names & {"陽錯", "陰錯", "陰陽俱錯"} or ("大會" in names and m in (4, 10)):
        de = de & {"天赦"}
    # 歲薄、逐陣：德作建論；「四月丙午六月戊午亦不從德願所宜」（卷十月厭條）
    if names & {"歲薄", "逐陣"}:
        de = set()
    # 鋪註條例：二月甲戌四月丙申六月甲子七月戊申八月庚辰九月辛卯十月甲子十二月甲子，德合與赦願所會之辰，諸事不忌
    # 條例原作「六月甲子」（四庫本、影印本同），甲子不值赦願；月表六月甲午（天德月德與天赦會）諸事不忌，
    # 而六月甲子照常注忌——條例「甲子」為「甲午」之誤，從月表
    ZHUSHI_BUJI = {1: "甲戌", 3: "丙申", 5: "甲午", 6: "戊申", 7: "庚辰", 8: "辛卯", 9: "甲子", 11: "甲子"}
    src = {}  # 事 -> 忌之来源神集合
    FUZANG_NEUTRAL.clear()
    released = set()  # 按卷十各條「併則不忌」整神解除者

    def yi_of(n):
        return {d for d, _ in SHI60 if n in YI.get(SHI_ROW[d], set())}

    yi = set()
    for n in names:
        yi |= yi_of(n)
    if names & {"官日", "守日", "相日"}:
        yi.add("襲爵受封")
    if "三合" in names:
        if z in "寅申巳亥":
            yi |= yi_of("母倉")
        if z in "子午卯酉":
            yi |= yi_of("王日")
    if names & {"收日", "母倉", "六合", "五富"} and names & {"月恩", "四相", "時德", "開日"}:
        yi.add("修倉庫")
    if "天倉" in names and names & {"月恩", "四相", "開日"}:
        yi.add("修倉庫")
    if names & {"執日", "危日", "收日"}:
        if m >= 9:
            yi.add("畋獵")
        if 0 <= m <= 2:
            yi.add("取魚")
    if m >= 9 and ("危日" in names or z in "午申"):
        yi.add("伐木")

    for d, _ in SHI60:
        for n in JI.get(SHI_ROW[d], set()) & names:
            src.setdefault(d, set()).add(n)
    for key in (g, z):
        for d in BAIJI.get(key, ()):
            src.setdefault(d, set()).add(key + "日")
    if "天狗" in names:
        src.setdefault("祭祀", set()).add("天狗")
    # 卷十月建條「又為土府，專忌營建宮室……破土」：土功之忌遇德不解，相抵亦留（三月壬辰月表）
    if "土府" in names:
        for d in TUGONG | {"築隄防"}:
            if d in src:
                src[d].add("土府專忌")
    # 卷十一補垣塞穴條：土府土符地囊土王用事後止忌補垣
    for n in names & {"土府", "土符", "地囊"}:
        src.setdefault("補垣塞穴", set()).add(n)

    # 次要凶神的解：與德等併則不忌（德作建論之日，德不算）
    jnames = names - (DE - de)
    # 卷十小耗條：子午卯酉月值劫煞，雖與德合併猶忌
    xiaohao_youji = mz in "子午卯酉" and "劫煞" in names
    for d in list(src):
        for n in list(src[d]):
            jie = JIE.get(n, set())
            if n == "小耗" and xiaohao_youji:
                jie = set()
            if jie and jnames & jie:
                src[d].discard(n)
                released.add(n)
                if n in ("復日", "重日"):  # 卷十：「則不忌亦不註宜」
                    yi.discard(d)
                    FUZANG_NEUTRAL.add(d)
            if n in ZHIJI and names & ZHIJI[n] and d not in RESIDUAL:
                src[d].discard(n)
        if not src[d]:
            del src[d]
    # 四耗：辰月與天德月德、寅申月與天德合、巳月與月德合、辰戌丑未月與三合併，止忌軍事
    if "四耗" in names and ((mz == "辰" and names & {"天德", "月德"}) or (mz in "寅申" and "天德合" in names)
                           or (mz == "巳" and "月德合" in names) or (mz in "辰戌丑未" and "三合" in names)):
        for d in list(src):
            src[d].discard("四耗")
            if not src[d]:
                del src[d]
    # 四忌四窮：惟正月乙亥與天願併，止忌軍事，餘皆不忌（卷十四忌四窮條）
    if m == 0 and gz == "乙亥" and "天願" in names:
        for d in list(src):
            src[d] -= {"四窮", "四忌", "八龍"}
            if not src[d]:
                del src[d]
    # 五墓：五月十一月與月德併則不忌
    if "五墓" in names and m in (4, 10) and "月德" in names:
        for d in list(src):
            src[d].discard("五墓")
            if not src[d]:
                del src[d]
    # 九空：寅申月值滿日、子午卯酉月值開日，與月德天德合月德合併則不忌
    if "九空" in names and names & {"月德", "天德合", "月德合"} and (
            (mz in "寅申" and "滿日" in names) or (mz in "子午卯酉" and "開日" in names)):
        for d in list(src):
            src[d].discard("九空")
            if not src[d]:
                del src[d]
    # 月建：與天德月德天德合月德合天赦月恩四相併，止忌土功（卷十月建條）
    if "月建" in names and names & (DE4 | {"天赦", "月恩", "四相"}) and not (names & set(MAJOR)):
        for d in list(src):
            if d not in TUGONG:
                src[d] -= {"月建", "小時"}
            if not src[d]:
                del src[d]
    # 滿日：寅申月值守日、子午卯酉月值相日，與月德天德合月德合月恩四相併則不忌
    if "滿日" in names and mz in "寅申子午卯酉" and names & {"月德", "天德合", "月德合", "月恩", "四相"}:
        released.add("滿日")
        for d in list(src):
            src[d].discard("滿日")
            if not src[d]:
                del src[d]

    major_set = set(MAJOR) | MAJOR_ALIAS
    present = [n for n in MAJOR if n in names]
    # 鋪註條例：凡天吏大時不以死敗論者，遇四廢歲薄逐陣仍以死敗論（不用等第，從常例）
    if names & {"四廢", "歲薄", "逐陣"}:
        present_g = [n for n in present if n not in ("天吏", "大時")]
    else:
        present_g = present
    grades = [DENGDI.get((n, mz)) for n in present_g]
    grades = [x for x in grades if x]
    grade = max(grades, key=GRADE_ORDER.index) if grades else None
    # 有主凶神时，月建、月破、滿日并入主凶神一组（「從刑厭論」「從災煞論」等）
    if grade:
        major_set |= {"月建", "小時", "土府", "月破", "大耗", "滿日"}
        # 卷十九空條：巳亥月值月厭從月厭論，辰戌丑未月值破日從月破論
        if "九空" in names and names & {"月厭", "月破"}:
            major_set |= {"九空"}
    ji_major = {d for d, ns in src.items() if ns & major_set}
    ji_minor = {d for d, ns in src.items() if ns - major_set}
    note, mode = "", "從忌"
    # 特定日：諸事皆忌
    ALLBAD = {"小會", "單陰", "純陽", "孤陽", "純陰", "陰位", "陰陽交破", "陰陽擊衝", "陽破陰衝", "陰道衝陽",
              "三陰", "陰陽俱錯", "絕陰", "絕陽"}
    if names & ALLBAD or ("大會" in names and not (m in (4, 10) and "天赦" in names)):
        return shen, jc, None, None, "特定日→皆忌"
    if "四廢" in names and "月破" in names:
        return shen, jc, None, None, "四廢併月破→皆忌"
    if grade:
        has_de = bool(de)
        if "月厭" in present and mz in "子午":
            has_de = "天赦" in names
        if grade == "上" or (grade in ("上次", "中") and has_de):
            mode = "從宜"
        elif (grade == "上次" and not has_de) or (grade == "中次" and has_de):
            mode = "相抵"
        elif (grade in ("中", "中次") and not has_de) or (grade == "下" and has_de):
            mode = "從忌"
        else:
            mode = "皆忌"
        note = f"{grade}{'遇德' if has_de else ''}→{mode}"
        if mode == "皆忌":
            return shen, jc, None, None, note
        # 月厭遇德「止忌行幸……結婚姻……栽種」，凡從宜、相抵皆留；大時只忌軍事
        residual = RESIDUAL_YAN if "月厭" in present else (set() if present == ["大時"] else RESIDUAL)
        if mode == "從宜":
            ji_major = ji_major & residual
        elif mode == "相抵":
            both = ji_major & yi
            yi -= both
            ji_major = (ji_major - both) | (ji_major & residual)
            # 鋪註條例：吉凶相抵不註忌甲亦不註忌乙
            freed = both - residual - ji_minor
            # 上冊受封、上表章同为卷十「上冊進表章」一事
            if freed & {"上冊受封", "上表章"}:
                freed |= {"上冊受封", "上表章"}
                ji_major -= {"上冊受封", "上表章"} - {d for d, ns in src.items() if ns - major_set}
            # 「遇不將而不註忌嫁娶者亦仍註忌」：嫁娶只由不將放开时不连带
            if "嫁娶" in freed and not (names - {"不將"}) & YI.get(SHI_ROW["嫁娶"], set()):
                freed = freed - {"嫁娶"}
            for a, bs in LINK:
                if a & freed:
                    ji_major -= bs
            # 鋪註條例：德合赦願月恩四相時德等日不註忌進人口……
            if names & (DE | {"月恩", "四相", "時德"}):
                ji_major -= {"進人口", "安牀", "經絡", "醞釀", "開市", "立券交易", "納財", "開倉庫出貨財"}
        else:
            yi -= ji_major
    else:
        yi -= ji_major
    yi -= ji_minor
    ji = ji_major | ji_minor
    # 鋪註條例：凡吉凶相抵不註忌開市立券交易，亦不註忌開倉庫出貨財，如遇專忌之日則仍註忌
    #（月表實例：甲日百忌「甲不開倉」作專忌論，五虛等不在其列；二月甲寅例外，存疑）
    if mode == "相抵" and "開倉庫出貨財" in ji and not {"開市", "立券交易"} & ji:
        if not {"甲日", "天賊"} & src.get("開倉庫出貨財", set()):  # 天賊與德合併猶忌（五月辛酉月表）
            ji.discard("開倉庫出貨財")
    if ZHUSHI_BUJI.get(m) == gz:  # 只是不注忌；本应忌者亦不注宜
        yi -= ji
        ji = set()
    # 復日重日遇德：葬事不忌亦不註宜——只免復日重日与主凶神之忌，他神猶忌者照忌
    ji -= {d for d in FUZANG_NEUTRAL if not (src.get(d, set()) - major_set)}
    yi -= FUZANG_NEUTRAL
    yi -= ji
    # 鋪註條例：凡歲薄逐陣日所宜事照月厭所忌刪，所忌仍從本日
    if names & {"歲薄", "逐陣"}:
        yi -= {d for d, _ in SHI60 if "月厭" in JI.get(SHI_ROW[d], set())}
    # 卷十無祿日條：「惟癸亥為干支俱盡日，雖值天月二德……仍以無祿論」，止註祭祀解除沐浴等，餘事不註宜
    #（其余無祿日視年而定，「月表與萬年書俱不論」）
    if gz == "癸亥":
        yi &= {"祭祀", "解除", "沐浴", "剃頭", "整手足甲", "掃舍宇", "修飾垣墻", "平治道塗", "破屋壞垣", "伐木"}
    # 鋪註條例：凡開日不註宜破土安葬啟攢（他神相抵照算，最后不注宜）
    if "開日" in names:
        yi -= {"破土", "安葬", "啟攢"}
    # 鋪註條例：天狗寅日忌祭祀不註宜祈福求嗣
    if "祭祀" in ji:
        yi -= {"祈福", "求嗣"}
    # 鋪註條例：凡月厭忌行幸上官，不註宜頒詔施恩封拜詔命公卿招賢舉正直
    if "月厭" in names and {"出行", "上官赴任"} & ji:
        yi.discard("襲爵受封")
    # 鋪註條例：凡忌詔命公卿招賢不註宜施恩封拜舉正直襲爵受封
    zm = (JI.get("詔命公卿招賢", set()) & names) - released
    if (zm - major_set) or (zm & major_set and mode in ("從忌",)):
        yi.discard("襲爵受封")
    # 土府土符地囊止註忌補垣，亦不註宜塞穴；壬日止忌開渠，卯日止忌穿井
    disp = {}
    eff = src.get("補垣塞穴", set()) if "補垣塞穴" in ji_major else src.get("補垣塞穴", set()) - major_set
    if "補垣塞穴" in ji and eff and eff <= {"土府", "土府專忌", "土符", "地囊"}:
        disp["補垣塞穴"] = "補垣"
    eff = src.get("開渠穿井", set()) if "開渠穿井" in ji_major else src.get("開渠穿井", set()) - major_set
    if "開渠穿井" in ji and eff and eff <= {"卯日", "壬日"}:
        disp["開渠穿井"] = "穿井" if eff == {"卯日"} else ("開渠" if eff == {"壬日"} else "開渠穿井")
    YIJI_DISP.clear()
    YIJI_DISP.update(disp)
    return shen, jc, yi, ji, note


YIJI_DISP = {}
FUZANG_NEUTRAL = set()


# 鋪註條例「凡吉凶相抵不註忌甲亦不註忌乙」
LINK = [({"祈福"}, {"求嗣"}),
        ({"結婚姻"}, {"冠帶", "納采問名", "嫁娶", "進人口"}),
        ({"嫁娶"}, {"冠帶", "結婚姻", "納采問名", "進人口", "移徙", "安牀"}),
        ({"移徙"}, {"安牀"}), ({"安牀"}, {"移徙"}),
        ({"解除"}, {"剃頭", "整手足甲"}),
        ({"修造動土", "豎柱上梁"}, {"築隄防", "修倉庫", "鼓鑄", "苫蓋", "修置產室", "開渠穿井", "安碓磑", "補垣塞穴",
                                "修飾垣墻", "平治道塗", "破屋壞垣"}),
        ({"開市"}, {"立券交易", "納財"}), ({"納財"}, {"開市", "立券交易"}), ({"立券交易"}, {"開市", "納財"}),
        ({"開市", "立券交易"}, {"開倉庫出貨財"}),
        # 條例「不註忌牧養亦不註忌納畜，不註忌納畜亦不註忌牧養」；月表實例只見前一向（二月戊戌納畜放而牧養仍忌）
        ({"牧養"}, {"納畜"}),
        # 「有宜安葬不註忌啟攢，有宜啟攢不註忌安葬」；月表實例（二月丙戌）破土亦隨安葬不注
        ({"安葬"}, {"啟攢", "破土"}), ({"啟攢"}, {"安葬"})]


# 月表不印的神：建除本名（已見標題）、军事神（六十事无军事）、天獄（卷六：與災煞同位，不另列）、亥子日
NO_PRINT = {"無祿", "建日", "除日", "滿日", "平日", "定日", "執日", "危日", "成日", "收日", "開日", "閉日", "兵吉", "兵福", "兵寶",
            "兵禁", "天獄", "亥子日"}


def order(evs):
    return [d for d, _ in SHI60 if d in evs]


def main():
    rows = []
    for m in range(12):
        for k in range(60):
            shen, jc, yi, ji, note = yiji(m, k)
            ji_s = [n for n, g in shen.items() if g and n not in NO_PRINT]
            xiong = [n for n, g in shen.items() if not g and n not in NO_PRINT]
            rows.append({"月": m + 1, "干支": gz_name(k), "建除": jc, "吉神": " ".join(ji_s), "凶神": " ".join(xiong),
                         "宜": "諸事不宜" if yi is None else " ".join(order(yi)),
                         "忌": "" if ji is None else " ".join(YIJI_DISP.get(e, e) for e in order(ji)), "等第": note})
    with open("data/folk/xieji_yuebiao.csv", "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0]))
        w.writeheader()
        w.writerows(rows)
    if len(sys.argv) > 1:
        compare(rows, json.load(open(sys.argv[1], encoding="utf-8")))


# ---------------- 与 OCR 比对 ----------------
OCR_FIX = str.maketrans({"杞": "祀", "化": "祀", "犯": "祀", "他": "祀", "舟": "冊", "術": "冊", "辦": "冊", "將": "冊",
                         "姿": "娶", "嬖": "娶", "要": "娶", "鞍": "娶", "嬰": "娶", "嫁": "嫁", "林": "牀", "狀": "牀",
                         "株": "牀", "斛": "解", "鮮": "解", "收": "牧", "巷": "養", "餐": "養", "番": "養", "奠": "葬",
                         "爽": "葬", "棒": "攢", "捕": "捕", "憤": "攢", "改": "畋", "敗": "畋", "肌": "取", "耿": "取",
                         "叔": "取", "醒": "醞", "醜": "醞", "醢": "醞", "卷": "券", "春": "券", "泰": "券", "舂": "券",
                         "芬": "券", "奏": "券", "眷": "券", "徒": "徙", "從": "徙", "陵": "隄", "提": "隄", "彼": "捉",
                         "提": "捉", "挺": "捉", "汁": "井", "升": "井", "雄": "碓", "雎": "碓", "睢": "碓", "堆": "碓",
                         "惟": "碓", "樵": "碓", "植": "磑", "驢": "磑", "燈": "磑", "禮": "磑", "驥": "磑", "壇": "垣",
                         "填": "垣", "坦": "垣", "迫": "垣", "壤": "壞", "祟": "築", "穴": "穴", "央": "穴", "坎": "穴",
                         "給": "絡", "格": "絡", "錦": "鑄", "故": "鼓", "書": "苫", "載": "栽", "裡": "種", "我": "栽"})


def find_ev(t, v):
    """事目 v 在 t 中的位置：三字以上容一字之差，二字须全同"""
    n = len(v)
    if n <= 2:
        return t.find(v)
    for i in range(len(t) - n + 1):
        if sum(a != b for a, b in zip(t[i:i + n], v)) <= 1:
            return i
    return -1


def compare(rows, ocr):
    vocab = [d for d, _ in SHI60]
    shen_vocab = sorted({n for m in range(12) for k in range(60) for n in shen_of(m, k)[0]} | {"吉神", "凶神"},
                        key=len, reverse=True)
    nbad = nday = 0
    for r in rows:
        key = f"{r['月']}-{r['干支']}"
        t = ocr.get(key)
        if t is None:
            continue
        nday += 1
        # 下一日标题漏认时，本日文字会接上后几日：在第二个「……日吉神」处截断
        k0 = [gz_name(i) for i in range(60)].index(r["干支"])
        heads = [gz_name((k0 + j) % 60) + NAYIN[((k0 + j) % 60) // 2] for j in (1, 2, 3)]
        cut_at = len(t)
        for h in heads:
            for i in range(8, len(t) - 5):
                if sum(a != b for a, b in zip(t[i:i + 5], h)) <= 2:
                    cut_at = min(cut_at, i)
                    break
        mm = [x.start() for x in re.finditer(r"[日目回匡圓曰][吉書苫占富言音胃百盲羈蘆賣害舌][神仲沖]", t)]
        if len(mm) >= 2:
            cut_at = min(cut_at, mm[1] - 6)
        t = t[:cut_at]
        t = t[8:].translate(OCR_FIX)
        for n in shen_vocab:
            t = t.replace(n, "")
        bad_all = any(x in t[:40] for x in ("諸事不宜", "諸事稀", "諸事不"))
        want_all = r["宜"] == "諸事不宜"
        if bad_all or want_all:
            if bad_all != want_all:
                nbad += 1
                print(key, r["等第"], "推" + ("諸事不宜" if want_all else "有宜忌"), "月表" + ("諸事不宜" if bad_all else "有宜忌"), sep="\t")
            continue
        want_yi, want_ji = set(r["宜"].split()), set(r["忌"].split())
        # 「忌」字常被 OCR 认错（言、忘、己……），所以在各事目出现位置之间都试切，取与推算最一致者
        pos = {}
        for v in vocab:
            n = len(v)
            ps = [i for i in range(len(t) - n + 1)
                  if (t[i:i + n] == v if n <= 2 else sum(a != b for a, b in zip(t[i:i + n], v)) <= 1)]
            if ps:
                pos[v] = ps
        cuts = sorted({p for ps in pos.values() for p in ps} | {len(t)})
        best = None
        for cut in cuts:
            gy = {v for v, ps in pos.items() if any(p < cut for p in ps)}
            gj = {v for v, ps in pos.items() if any(p >= cut for p in ps)}
            sc = len((want_yi & gj) | (want_ji & gy)) + len(gy & gj)
            if best is None or sc < best[0]:
                best = (sc, gy - gj, gj - gy)
        _, got_yi, got_ji = best
        contra = (want_yi & got_ji) | (want_ji & got_yi)
        extra = (got_yi - want_yi - want_ji) | (got_ji - want_ji - want_yi)
        if contra or extra:
            nbad += 1
            print(key, r["等第"], "宜忌相反:" + " ".join(order(contra)),
                  "月表宜而推未注:" + " ".join(order(got_yi - want_yi - want_ji)),
                  "月表忌而推未注:" + " ".join(order(got_ji - want_ji - want_yi)), sep="\t")
    print(f"比对 {nday} 日，有冲突或多出者 {nbad} 日", file=sys.stderr)


if __name__ == "__main__":
    main()
