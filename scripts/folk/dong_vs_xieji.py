"""《董公選擇日要覽》逐月逐建除之宜忌与协纪月表逐日对照。

用法（仓库根）：python3 scripts/folk/dong_vs_xieji.py
读 vendor/folk/wikisource/董公選擇日要覽/全文.wikitext（嘉庆二十二年蒋云序本录文）与 data/folk/xieji_yuebiao.csv，
写 data/folk/tongshu_dong_vs_xieji.csv：每行一日（月 × 干支）一事。

董公体例：每月十二条，以建除为纲（条首「建寅日」等即该月某支之日），条内或泛论该支诸日，或分举某干支。
解析为规则式（见 doc/folk/tongshu_yiji.md「对照方法」）：
- 只取宜忌，删去应验之语（「犯之……」「……日內」「……年內」「主……」等）；
- 范围：句中举具体干支者只及该干支；「餘某日」「其餘某日」及各干支（不论举否）；泛论者及全支；同一干支后出之说覆盖先出者；
- 宜忌：「宜／利／可用」及「某事大吉、上吉、次吉、吉」为宜；「不宜／不利／忌／不可用」为忌；「百事不宜、諸事不宜、不宜用事、
  百事皆忌、均不可用、不可用、大凶、更凶」等无事项者为诸事皆忌；以人命为条件者（「惟水土生人用之」）不计；分句（以、，；分）含「犯之」等应验语者删去；连举干支（「丁丑、癸丑」）及先举干支而无宜忌、后举干支有宜忌者（「壬午月德、丙午、戊午三日利……」）合为一范围；
  「只宜作生基，如婚姻修造等事用之」之「如」所举作忌；
- 事项映射到协纪六十事：起造修造造作動土營建修整→修造動土，婚姻嫁娶娶親→嫁娶，結婚姻→結婚姻，會親→會親友，安葬埋葬葬→安葬，
  破土斬草→破土，入宅移居移徙→移徙，開張→開市，出行→出行，上官赴任參官→上官赴任，上學→入學，定磉拴架造架上樑→豎柱上梁。
「泛言」列：未举事项而推及十二事者（「百事不宜」、条内只有凶神与应验之语等）记「是」，统计另出一组除泛言者。
对照：协纪该日该事「宜」「忌」（含諸事不宜）「两不注」三态；董宜协宜、董忌协忌为合，董忌协两不注为「半合」，董宜协忌、董忌协宜为异。
"""
import csv
import re

SRC = "vendor/folk/wikisource/董公選擇日要覽/全文.wikitext"
XIEJI = "data/folk/xieji_yuebiao.csv"
OUT = "data/folk/tongshu_dong_vs_xieji.csv"

GAN = "甲乙丙丁戊己庚辛壬癸"
ZHI = "子丑寅卯辰巳午未申酉戌亥"
CN_MONTH = ["正", "二", "三", "四", "五", "六", "七", "八", "九", "十", "十一", "十二"]
EVENTS = [
    ("修造動土", r"起造|修造|造作|動土|營建|營為|修整|修理|修作|裝修|作灶"),
    ("嫁娶", r"婚姻|嫁娶|娶親|成親"),
    ("會親友", r"會親"),
    ("安葬", r"安葬|埋葬|葬"),
    ("破土", r"破土|斬草"),
    ("移徙", r"入宅|移居|移徙"),
    ("開市", r"開張"),
    ("出行", r"出行"),
    ("上官赴任", r"上官|赴任|參官"),
    ("納采問名", r"納采"),
    ("入學", r"上學|入學"),
    ("豎柱上梁", r"定磉|拴架|造架|上樑|上梁|豎柱"),
]
ALL_EVENTS = [e for e, _ in EVENTS]
NEG_ALL = r"百事不宜|諸事不宜|諸事不吉|百事皆忌|百事俱忌|百事忌|不宜用事|不宜用|不可用|均不可用|不用為妙|大凶|更凶|更加凶險|俱不利|具不利|諸事不利|百事不利|不利$|諸事忌|均不利|皆不利|不吉|陰忌之日|凶"
YING = r"犯之|用之主|用之招|日內|年內|^主|^必主|進橫財|大旺|大發|之應"
ALL_GOOD = r"百事皆吉|百事全吉|百事俱吉|諸事順遂|百事順遂|百福駢臻|諸事皆吉"


def gz_name(k):
    return GAN[k % 10] + ZHI[k % 12]


def gz_of_zhi(z):
    return [k for k in range(60) if k % 12 == z]


def events_in(t):
    return [e for e, pat in EVENTS if re.search(pat, t)]


def verdict_of(clause):
    """一段同范围之辞的宜忌：({事: 宜/忌}, 是否泛言)。泛言＝未举事项、只言「百事不宜」「吉」等，推及十二事"""
    v = {}
    # 「只宜作生基，如婚姻、修造等事用之」「宜合板作生基、如修造會親婚姻」：「如」所举为反例，作忌
    if "生基" in clause:
        for m in re.finditer(r"如([^宜]*)", clause):
            for e in events_in(m.group(1)):
                v[e] = "忌"
        clause = re.sub(r"如[^宜]*", "", clause)
    # 「餘午日埋葬亦不利」：事在「不利」之前
    for piece in clause.split("，"):
        if re.search(r"(不利|不吉)$", piece):
            for e in events_in(piece):
                v[e] = "忌"
    for m in re.finditer(r"(不宜|不利|忌|不可|不載)([^宜利]*)", clause):
        for e in events_in(m.group(2)):
            v[e] = "忌"
    # 「若起造大工及婚姻入宅開張、立見凶禍」：「若」所举而后言凶者作忌
    for m in re.finditer(r"若(.*?)(凶|不利|不可)", clause):
        for e in events_in(m.group(1)):
            v[e] = "忌"
    for m in re.finditer(r"(?<!不)(宜|利於|利)([^不忌若]*)", clause):
        for e in events_in(m.group(2)):
            v.setdefault(e, "宜")
    # 「某事某事、次吉」：吉字以前（自分句起或上一吉字起）所举之事为宜
    for m in re.finditer(r"(.*?)(大吉|上吉|次吉|吉)(?!星|神|曜|日|時)", clause):
        if re.search(r"諸吉|眾吉", m.group(1) + m.group(2)):
            continue
        for e in events_in(m.group(1)):
            v.setdefault(e, "宜")
    # 「宜起造營為、百事皆吉」：举事之外又泛言百事吉者，余事亦宜（泛言）
    if v and re.search(ALL_GOOD, clause) and all(x == "宜" for x in v.values()):
        return {e: v.get(e, "宜") for e in ALL_EVENTS}, True
    if not v:
        clause = re.sub(r"無凶星?|無凶神", "", clause)
        if re.search(ALL_GOOD, clause):
            return {e: "宜" for e in ALL_EVENTS}, True
        if re.search(NEG_ALL, clause):
            return {e: "忌" for e in ALL_EVENTS}, True
        if re.search(r"(大吉|上吉|次吉|卻吉|用之吉|吉|(?<!不)可用)(?!星|神|曜|日|時)", re.sub(r"諸吉|眾吉|不吉|雖云吉", "", clause)):
            return {e: "宜" for e in ALL_EVENTS}, True
    return v, False


def parse_entry(zhi, text):
    """返回 {干支序: {事: (宜忌, 原句, 泛言)}}。逐分句定范围：分句举干支则切到该干支，「餘某日」切到未举者，
    以「只、但、亦、又、惟獨」起之句承上句之范围；条件之辞（以人命、某某者）不计；后出之说覆盖先出者，但泛论全支之句不覆盖分举干支者，泛言之宜不覆盖明文。"""
    text = re.sub(r"\s+", "", text)
    days = gz_of_zhi(zhi)
    names = {gz_name(k): k for k in days}
    out = {k: {} for k in days}
    mentioned = set()
    excluded = set()
    ying_scopes = []  # 删去应验语时所在之范围
    specific = set()  # 已由分举干支之句得宜忌者；「餘某日」指其外诸日
    prev_scope = days
    pat_names = "|".join(names)
    for sent in re.split(r"[。！]", text):
        if not sent:
            continue
        scope = prev_scope if re.match(r"只|但|亦|又(?!云)|惟獨", sent) else days
        groups = []  # [(scope, [frags])]
        name_only = False
        for f in re.split(r"[，,；;：:「」、]", sent):
            # 应验之语整段删去（「不利造作裝修婚姻會親、犯之主田蚕不收……、丁丑、癸丑煞入中宮更凶」只删「犯之……」一段）
            if not f:
                continue
            if re.search(YING, f):
                ying_scopes.append(scope)
                continue
            hit = [names[n] for n in re.findall(pat_names, f)]
            if re.search(r"[正二三四五六七八九十]+月之", f):  # 「一年四季惟二月之辛亥吉」言他月，不计
                continue
            if re.search(r"餘.?" + ZHI[zhi] + r"|其餘", f):
                scope = "REST"  # 用时再按已分举者求其余
            elif hit:
                # 「甲辰雖有氣，卻與戊辰同」「丁丑、癸丑煞入中宮」：「與某同」及连举干支并入上一范围
                # 上一组只举干支、神煞而无宜忌（「壬午月德、丙午、戊午三日利……」「壬子……，甲子……，不宜用也」）亦并入
                bare = bool(groups) and groups[-1][0] == scope and not verdict_of("，".join(groups[-1][1]))[0]
                union = (re.search(r"與(" + pat_names + r")同", f) or name_only or bare) and scope not in (days, "REST")
                scope = sorted(set(scope) | set(hit)) if union else hit
                mentioned.update(hit)
            name_only = bool(hit) and re.search("(" + pat_names + r")$", f) is not None
            if groups and groups[-1][0] == scope:
                groups[-1][1].append(f)
            else:
                groups.append((scope, [f]))
        for sc, frags in groups:
            clause = "，".join(frags)
            if sc == "REST":
                # 「餘某日」：未举之干支（条内未举任何干支，则「餘」指余事而非余日，不计）
                sc = [k for k in days if k not in specific and k not in mentioned] if (specific or mentioned) else []
            if re.search(r"生人用之|命宮相合方可|人用之|者.{0,4}吉|者次吉", clause):
                excluded.update(sc if sc != days else [])
                continue
            v, blanket = verdict_of(clause)
            if v and sc != days:
                specific.update(sc)
            for k in sc:
                for e, x in v.items():
                    # 泛言之宜不覆盖已有明文（「宜婚姻入宅……。壬寅、戊寅上吉」），泛言之忌覆盖（「惟庚申日乃春正四廢百事忌」）
                    # 例外以「惟」引起者（「惟癸未一日……大吉」）可覆盖
                    if blanket and x == "宜" and e in out[k] and "惟" not in sent:
                        continue
                    # 泛论全支之句不覆盖已分举干支之说（「惟乙卯一日次吉。……破敗大凶」）
                    if sc == days and k in specific and e in out[k]:
                        continue
                    out[k][e] = (x, clause, blanket)
        prev_scope = scope
    # 条内只列凶神与应验（「有朱雀勾絞……犯之……」）而无明文宜忌之日：作诸事皆忌，记为泛言
    # 只及删去应验语时所在范围之日（「丙戌、壬戌朱雀勾絞、白虎入中宮、用之主……」不及甲戌等）
    lead = re.split(r"[。！，,]", text)[0]
    for sc in ying_scopes:
        for k in (sc if sc != "REST" else []):
            if not out[k] and k not in excluded:
                out[k] = {e: ("忌", lead + "……犯之……", True) for e in ALL_EVENTS}
    return out


def main():
    s = open(SRC, encoding="utf-8").read()
    parts = re.split(r"==([正二三四五六七八九十]+[一二]?月)[　 ](.)月==", s)
    xie = {(int(r["月"]), r["干支"]): r for r in csv.DictReader(open(XIEJI, encoding="utf-8"))}
    rows = []
    for i in range(1, len(parts), 3):
        mname, mzhi, body = parts[i], parts[i + 1], parts[i + 2]
        month = CN_MONTH.index(mname[:-1]) + 1
        ents = re.findall(r"\n([建除滿平定執破危成收開閉])(.)日[：:](.*?)(?=\n[建除滿平定執破危成收開閉].日[：:]|\n==|\Z)", body, re.S)
        for jc, z, text in ents:
            zi = ZHI.index(z)
            res = parse_entry(zi, text)
            for k, evs in res.items():
                x = xie[(month, gz_name(k))]
                yi, ji = x["宜"].split(), x["忌"].split()
                for e, (v, clause, blanket) in evs.items():
                    xv = "宜" if e in yi else ("忌" if (e in ji or "諸事不宜" in yi or x["宜"].strip() == "諸事不宜") else "两不注")
                    if v == xv:
                        res_ = "合"
                    elif v == "忌" and xv == "两不注":
                        res_ = "半合"
                    elif v == "宜" and xv == "两不注":
                        res_ = "董宜协未注"
                    else:
                        res_ = "異"
                    rows.append({"月": month, "建除": jc, "干支": gz_name(k), "事": e, "董公": v, "协纪": xv, "結果": res_,
                                 "泛言": "是" if blanket else "", "协纪等第": x["等第"], "董公原句": clause})
    with open(OUT, "w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()), lineterminator="\n")
        w.writeheader()
        w.writerows(rows)
    from collections import Counter
    c = Counter(r["結果"] for r in rows)
    print(len(rows), dict(c))
    print(" 除泛言", dict(Counter(r["結果"] for r in rows if not r["泛言"])))
    for e in ALL_EVENTS:
        ce = Counter(r["結果"] for r in rows if r["事"] == e)
        if ce:
            print(" ", e, dict(ce))


if __name__ == "__main__":
    main()
