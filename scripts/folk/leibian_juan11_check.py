"""类编卷十一逐月日表（data/folk/leibian_juan11_days.csv）三项检验：

1. 自洽：每神在每月所值之日能否归为「依支」「依干」「依干支」的一条起例，列出例外日（刻漏、误刻）。
2. 与类编卷十五「日吉凶神」起例（本项目 tongshu_leibian_vs_xieji.csv 类编栏所录、已对影像）逐日比对。
3. 与协纪月表（data/folk/xieji_yuebiao.csv，卷二十至三十一）逐日比对：同名或已知异名者按日计同异；
   另对每个类编神求协纪月表中逐日重合最高之神（Jaccard），用来发现异名同位。

用法（仓库根）：python3 scripts/folk/leibian_juan11_check.py [--md]
输出 data/folk/leibian_juan11_vs_xieji.csv；--md 打印供 doc/folk/tongshu.md 用的表。
"""
import collections
import csv
import sys

G = "甲乙丙丁戊己庚辛壬癸"
Z = "子丑寅卯辰巳午未申酉戌亥"
GZ = [G[i % 10] + Z[i % 12] for i in range(60)]

DAYS = "data/folk/leibian_juan11_days.csv"
YUEBIAO = "data/folk/xieji_yuebiao.csv"
OUT = "data/folk/leibian_juan11_vs_xieji.csv"

# 异写归一（只用于比对，CSV 照录原刻）
NORM = {"幽微": "幽微星"}


def load_lb():
    d = collections.defaultdict(set)
    rows = list(csv.DictReader(open(DAYS, encoding="utf-8")))
    assert len(rows) == 720
    for r in rows:
        for x in (r["吉"] + " " + r["凶"]).split():
            d[NORM.get(x, x)].add((int(r["月"]), r["干支"]))
    return d, rows


def load_xj():
    d = collections.defaultdict(set)
    for r in csv.DictReader(open(YUEBIAO, encoding="utf-8")):
        for x in (r["吉神"] + " " + r["凶神"]).split():
            d[x].add((int(r["月"]), r["干支"]))
    return d


def zhi_rule(lst):
    """十二月各取一组支（list of str）→ 日集"""
    return {(m + 1, g) for m in range(12) for g in GZ if g[1] in lst[m]}


def gan_rule(lst):
    return {(m + 1, g) for m in range(12) for g in GZ if g[0] in lst[m]}


def seq(s):
    return s.split("|")


def season(a, b, c, d):
    return [a] * 3 + [b] * 3 + [c] * 3 + [d] * 3


# 类编卷十五「日吉凶神」起例（02094410.cn PDF 页），取自 tongshu_leibian_vs_xieji.csv 类编栏
JUAN15 = {
    "天德": ("干支混", None, "52"),
    "月德": (gan_rule(seq("丙|甲|壬|庚|丙|甲|壬|庚|丙|甲|壬|庚")), "", "52"),
    "月德合": (gan_rule(seq("辛|己|丁|乙|辛|己|丁|乙|辛|己|丁|乙")), "", "52"),
    "月空": (gan_rule(seq("壬|庚|丙|甲|壬|庚|丙|甲|壬|庚|丙|甲")), "", "52"),
    "月恩": (gan_rule(seq("丙|丁|庚|己|戊|辛|壬|癸|庚|乙|甲|辛")), "", "52"),
    "母倉": (zhi_rule(season("亥子", "寅卯", "辰戌丑未", "申酉")), "", "54"),
    "要安": (zhi_rule(seq("寅|申|卯|酉|辰|戌|巳|亥|午|子|未|丑")), "", "53"),
    "玉堂": (zhi_rule(seq("卯|酉|辰|戌|巳|亥|午|子|未|丑|申|寅")), "", "53"),
    "金堂": (zhi_rule(seq("辰|戌|巳|亥|午|子|未|丑|申|寅|酉|卯")), "", "53"),
    "敬心": (zhi_rule(seq("未|丑|申|寅|酉|卯|戌|辰|亥|巳|子|午")), "", "53"),
    "普護": (zhi_rule(seq("申|寅|酉|卯|戌|辰|亥|巳|子|午|丑|未")), "", "53"),
    "福生": (zhi_rule(seq("酉|卯|戌|辰|亥|巳|子|午|丑|未|寅|申")), "", "53"),
    "聖心": (zhi_rule(seq("亥|巳|子|午|丑|未|寅|申|卯|酉|辰|戌")), "", "53"),
    "益後": (zhi_rule(seq("子|午|丑|未|寅|申|卯|酉|辰|戌|巳|亥")), "", "53"),
    "續世": (zhi_rule(seq("丑|未|寅|申|卯|酉|辰|戌|巳|亥|午|子")), "", "53"),
    "六合": (zhi_rule(seq("亥|戌|酉|申|未|午|巳|辰|卯|寅|丑|子")), "", "54"),
    "游禍": (zhi_rule(seq("巳|寅|亥|申|巳|寅|亥|申|巳|寅|亥|申")), "", "59"),
    "月殺": (zhi_rule(seq("丑|戌|未|辰|丑|戌|未|辰|丑|戌|未|辰")), "", "59"),
    "天火": (zhi_rule(seq("子|卯|午|酉|子|卯|午|酉|子|卯|午|酉")), "顺行四仲", "59"),
    "天瘟": (zhi_rule(seq("未|戌|辰|寅|午|子|酉|申|巳|亥|丑|卯")), "", "59"),
    "土瘟": (zhi_rule(seq("辰|巳|午|未|申|酉|戌|亥|子|丑|寅|卯")), "", "59"),
    "焦坎": (zhi_rule(seq("辰|丑|戌|未|卯|子|酉|午|寅|亥|申|巳")), "", "60"),
    "九空": (zhi_rule(seq("辰|丑|戌|未|卯|子|酉|午|寅|亥|申|巳")), "九空焦坎合列", "60"),
    "飛廉": (zhi_rule(seq("戌|巳|午|未|寅|卯|辰|亥|子|丑|申|酉")), "飛廉大殺", "60"),
    "重喪": (gan_rule(seq("甲|乙|己|丙|丁|己|庚|辛|己|壬|癸|己")), "", "60"),
    "歸忌": (zhi_rule(seq("丑|寅|子|丑|寅|子|丑|寅|子|丑|寅|子")), "", "61"),
    "荒蕪": (zhi_rule(seq("巳|酉|丑|辰|申|子|卯|未|亥|寅|戌|午")), "天地荒蕪表", "61"),
    "龍虎": (zhi_rule(seq("巳|亥|午|子|未|丑|申|寅|酉|卯|戌|辰")), "", "61"),
    "罪至": (zhi_rule(seq("午|子|未|丑|申|寅|酉|卯|戌|辰|亥|巳")), "", "61"),
    "受死": (zhi_rule(seq("戌|辰|亥|巳|子|午|丑|未|寅|申|卯|酉")), "", "61"),
    "刀砧": (zhi_rule(season("亥子", "寅卯", "巳午", "申酉")), "刀砧（六畜）", "63"),
    "正四廢": ({(m + 1, g) for m in range(12) for g in season(["庚申", "辛酉"], ["壬子", "癸亥"], ["甲寅", "乙卯"], ["丙午", "丁巳"])[m]}, "正四廢", "64"),
    "火星": ({(m + 1, g) for m in range(12) for g in [
        ["乙丑", "甲戌", "癸未", "壬辰", "辛丑", "庚戌", "己未"],
        ["甲子", "癸酉", "壬午", "辛卯", "庚子", "己酉", "戊午"],
        ["壬申", "辛巳", "庚寅", "己亥", "戊申", "丁巳"]][m % 3]}, "火星凶日", "64"),
}
# 卷十五天德（干支混）
_td = ["丁", "申", "壬", "辛", "亥", "甲", "癸", "寅", "丙", "乙", "巳", "庚"]
_tdh = ["壬", "巳", "丁", "丙", "寅", "己", "戊", "亥", "辛", "庚", "申", "乙"]
JUAN15["天德"] = ({(m + 1, g) for m in range(12) for g in GZ if _td[m] in g}, "仲月申亥寅巳", "52")
JUAN15["天德合"] = ({(m + 1, g) for m in range(12) for g in GZ if _tdh[m] in g}, "仲月巳寅亥申", "52")
JUAN15["傍四廢"] = (gan_rule(season("庚辛", "壬癸", "甲乙", "丙丁")), "傍四廢（卷十一作四廢）", "64")
JUAN15["四廢∪正四廢"] = JUAN15["傍四廢"]
del JUAN15["傍四廢"]

# 说明（人工判读）
NOTE = {
    "四廢": "卷十一凡傍四廢日注「四廢」，其中正四廢（庚申辛酉等）改注「正四廢」，故此栏「缺」即正四廢二十四日；两栏合并见「四廢∪正四廢」",
    "四廢∪正四廢": "即卷十五傍四廢（春庚辛、夏壬癸、秋甲乙、冬丙丁）；协纪月表四廢只取正四廢",
    "黃道": "协纪对应为青龍明堂金匱寶光（天德）玉堂司命六黄道之并",
    "黑道": "协纪对应为天刑朱雀白虎天牢元武勾陳六黑道之并",
    "重喪": "同阴阳备用季月只忌己之复日；协纪复日三、六月取戊",
    "荒蕪": "一月一支，即卷十五表序；协纪卷三十四谓荒蕪即五虚（按季三支），故协纪多",
    "天火": "类编顺行四仲，协纪逆行",
    "九空": "类编九空与焦坎同位；协纪九空正月辰逆行四季",
    "天德": "四仲月依支（申亥寅巳）；协纪四仲无",
    "天德合": "四仲月依支（巳寅亥申）；协纪四仲无",
    "天賊": "类编正月辰、每月进五辰（辰酉寅未子巳戌卯申丑午亥）；协纪卷六引李鼎祚「正月在丑逆行十二辰」，无一日同",
    "大耗": "即月破之位；8月丁卯破日原刻「小耗」",
    "小耗": "8月丁卯原刻「小耗」当作大耗",
    "龍德": "所值即月建（建日）",
    "明星": "所值即协纪天牢（黑道）之位",
    "黑星": "所值即协纪青龍（黄道）之位；11月壬申原刻黑道",
    "官符": "与死炁同位，即协纪死氣（时阴同位）",
    "生炁": "即协纪生氣（时阳同位）",
    "約絞": "月取冲对二支：正月巳亥、二月子午……；协纪无此神，死神只取其一支",
    "天瑞": "不分月：戊寅、己卯、辛巳、庚寅、壬子五日",
    "天福": "不分月：己卯辛巳庚寅辛卯壬辰癸巳己亥庚子辛丑乙巳丁巳庚申（2月庚子刻作「天地」，7月癸未亦注）",
    "天恩": "不分月十五日，与协纪同",
    "顯星": "三月一周：孟月乙丑起每九日一位、仲月丁卯起、季月丙寅起（见 CSV 逐日）",
    "曲星": "三月一周，与顯星相错",
    "傳星": "三月一周，每九日一位",
    "上吉": "等第语，不依起例",
    "次吉": "等第语，不依起例",
    "土王後母倉": "季月巳午日；协纪母倉条亦有「土王用事後巳午」，月表未单列",
    "刀砧": "即卷十五「刀砧（六畜）」春亥子夏寅卯秋巳午冬申酉；3月丁亥失注",
    "土瘟": "9月庚子失注",
    "火星": "即卷十五「火星凶日」，三月一周",
    "天地轉殺": "仲月：二月卯、五月午、八月酉、十一月子（即月建）",
}

# 卷十一名 → 协纪月表名（同位异名）
ALIAS = {
    "玉堂": ["玉宇"], "敬心": ["敬安"], "生炁": ["生氣"], "游禍": ["遊禍"], "月殺": ["月煞"],
    "焦坎": ["九坎", "九焦"], "飛廉": ["大煞"], "死炁": ["死氣"], "正四廢": ["四廢"],
    "重喪": ["復日"], "荒蕪": ["五虛"], "四廢": [], "四廢∪正四廢": [],
    "黃道": ["青龍", "明堂", "金匱", "寶光", "玉堂", "司命"],
    "黑道": ["天刑", "朱雀", "白虎", "天牢", "元武", "勾陳"],
}


def gz_list(s):
    return " ".join(f"{m}月{g}" for m, g in sorted(s, key=lambda x: (x[0], GZ.index(x[1]))))


def infer(days):
    """把一神的日集按月归纳为依支/依干/依干支，返回 (起例描述, 例外日集合)"""
    desc, exc_miss, exc_extra = [], set(), set()
    kind = None
    for m in range(1, 13):
        s = {g for mm, g in days if mm == m}
        if not s:
            desc.append("-")
            continue
        zc = collections.Counter(g[1] for g in s)
        gc = collections.Counter(g[0] for g in s)
        zs = [z for z in Z if zc[z] >= 3]
        gs = [x for x in G if gc[x] >= 4]
        if zs and sum(zc[z] for z in zs) >= len(s) - 2 and kind != "干":
            kind = kind or "支"
            full = {g for g in GZ if g[1] in zs}
            desc.append("".join(zs))
        elif gs and sum(gc[x] for x in gs) >= len(s) - 2 and kind != "支":
            kind = kind or "干"
            full = {g for g in GZ if g[0] in gs}
            desc.append("".join(gs))
        else:
            desc.append("+".join(sorted(s, key=GZ.index)))
            full = s
        exc_miss |= {(m, g) for g in full - s}
        exc_extra |= {(m, g) for g in s - full}
    return kind or "干支", "|".join(desc), exc_miss, exc_extra


def main():
    lb, _ = load_lb()
    lb["四廢∪正四廢"] = lb["四廢"] | lb["正四廢"]
    xj = load_xj()
    out = []
    md = []
    for name in sorted(lb, key=lambda n: -len(lb[n])):
        S = lb[name]
        if len(S) < 12:
            continue
        kind, desc, miss, extra = infer(S)
        # 卷十五
        j15 = JUAN15.get(name)
        if j15:
            R = j15[0]
            j15s = f"同{len(S & R)}；卷十一多{len(S - R)}；卷十五多{len(R - S)}"
            j15d = ("卷十一多：" + gz_list(S - R) + "；" if S - R else "") + ("卷十五多：" + gz_list(R - S) if R - S else "")
        else:
            j15s, j15d = "", ""
        # 协纪
        names = ALIAS[name] if name in ALIAS else ([name] if name in xj else [])
        if names:
            X = set().union(*(xj[n] for n in names))
            xjs = f"同{len(S & X)}；類編多{len(S - X)}；協紀多{len(X - S)}"
        else:
            X = set()
            xjs = "協紀月表無"
        # 最近似
        best = max(xj, key=lambda n: len(S & xj[n]) / len(S | xj[n]))
        jac = len(S & xj[best]) / len(S | xj[best])
        out.append([name, kind, len(S), desc, gz_list(miss), gz_list(extra), j15s, j15d,
                    "+".join(names), xjs, best, f"{jac:.2f}", NOTE.get(name, "")])
    with open(OUT, "w", encoding="utf-8", newline="") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["類編名", "依", "日數", "卷十一歸納起例（正月至十二月）", "依起例當有而未注", "起例以外而注",
                    "與卷十五", "與卷十五異日", "協紀月表對應", "與協紀月表", "協紀月表最近似之神", "Jaccard", "說明"])
        w.writerows(out)
    if "--md" in sys.argv:
        print("| 类编名 | 日数 | 卷十一起例 | 例外 | 与卷十五 | 协纪对应 | 与协纪 | 最近似（Jaccard） |")
        print("|---|---|---|---|---|---|---|---|")
        for r in out:
            exc = "；".join(x for x in [("缺 " + r[4]) if r[4] else "", ("衍 " + r[5]) if r[5] else ""] if x)
            print(f"| {r[0]} | {r[2]} | {r[3] if len(r[3]) < 60 else '（逐日，见 CSV）'} | {exc} | {r[6]} | {r[8]} | {r[9]} | {r[10]}（{r[11]}） |")
    if "--md" in sys.argv:
        dengdi()
    print(f"写出 {OUT}：{len(out)} 神", file=sys.stderr)


def dengdi():
    """卷十一「上吉」「次吉」与协纪月表宜事多少、「諸事不宜」对照"""
    xj = {(r["月"], r["干支"]): r for r in csv.DictReader(open(YUEBIAO, encoding="utf-8"))}
    st = collections.defaultdict(lambda: [0, 0, 0])
    exc = []
    for r in csv.DictReader(open(DAYS, encoding="utf-8")):
        j = r["吉"].split()
        lv = "上吉" if "上吉" in j else ("次吉" if "次吉" in j else "不注")
        x = xj[(r["月"], r["干支"])]
        bad = "諸事不宜" in x["宜"]
        st[lv][0] += 1
        st[lv][1] += bad
        st[lv][2] += 0 if bad else len(x["宜"].split())
        if lv != "不注" and bad:
            exc.append(f"{r['月']}月{r['干支']}（{lv}；协纪{x['等第']}）")
    print()
    print("| 卷十一等第 | 日数 | 协纪「諸事不宜」 | 协纪平均宜事数 |")
    print("|---|---|---|---|")
    for lv in ["上吉", "次吉", "不注"]:
        n, b, y = st[lv]
        print(f"| {lv} | {n} | {b} | {y / n:.1f} |")
    print()
    print("卷十一注吉而协纪諸事不宜：" + "、".join(exc))


if __name__ == "__main__":
    main()
