"""《开元占经》三家星官：石氏、甘氏、巫咸中外官的名录、定义句与石氏度数。

来源：维基文库「唐開元占經 (四庫全書本)」卷六十五至七十（石氏中官、石氏外官、甘氏中官、甘氏外官、巫咸中外官）。
各条以「某某占N」为题（序号按节连续，用以核对完整性）；有的题被换成首句锚点（如「石氏曰攝提六星夾大角」），则以下一条
之前的首个定义句补名。定义句取该条首个「某氏曰某某N星……」。石氏条目的度数在其注文中（「入某宿N度去極N度在黄道内/外N度」），
原样保留注文，度数的解析与认星另做。

输出：
1. data/sky_culture/kaiyuan_sanjia.csv：school,section,ordinal,title,definition,stars,note
   - title：条目题中的星官名（原文字形）；definition：定义句；stars：定义句中的星数；note：定义句后紧随的注文（石氏度数在此）。
2. data/sky_culture/sanjia_schools.csv：陈卓体系（Stellarium chinese_chenzhuo）各星官所属之家与判定依据。
   依据：同名（繁简与异体字归一后同名）、家括注（Stellarium 以「(甘氏)」等区分同名星官）、方位（同名者按定义句方位区分）、
   别名、拆分（一条对应数官，如攝提→左右摄提）、附座（随所附之官）、二十八宿（卷六十至六十三，石氏）、
   推定（四库本阙文处据序号推定，非原文明记）。对不上的留空：天库（一星，03L）三家名录无此官，为 Stellarium 复原所加。
3. data/sky_culture/shishi_xingjing.csv：石氏星表度数（中外官 86 条注文 + 卷六十至六十三二十八宿的宿度、距星与去极度），
   解析为数值（古度）。认星与检验在 Rust（astronomy::sky_culture::tests_shishi）。

用法（仓库根）：uv run --project scripts python scripts/sky_culture/sanjia.py
维基文库原文缓存于 scripts/sky_culture/_cache/（不入库）。
"""

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from lunar_mansions import OUT, wikisource  # noqa: E402

NUM = "一二三四五六七八九十百"
CHAPTERS = range(65, 71)

# 四库本原文的订正（卷, 原文, 订正, 理由）。只为让条目切分与序号正确，订正处在输出注释中列出；其余异体字照录。
FIXES = [
    (65, "匏{{SKchar|2201}}星二十七", "匏瓜星占二十七", "题脱「占」；SKchar 2201 据《晋书·天文志》「匏瓜五星在离珠北」作「瓜」"),
    (65, "石氏匏{{SKchar|2201}}星五星", "石氏曰匏瓜五星", "定义句脱「曰」"),
    (69, "{{SK anchor|内五諸侯星四十九}}", "{{SK anchor|内五諸侯星占四十九}}", "题脱「占」"),
    (70, "{{SK anchor|蓋屋星占十二}}", "{{SK anchor|蓋屋星占十三}}", "与上条「泣星占十二」重号，按次序当为十三"),
]
# 四库本原阙的条目：（家, 节, 序号）。卷六十六至「太㣲星占四十六」止，卷六十七自「三台占五十三」起。
LACUNAE = [("石氏", "中官", n) for n in range(47, 53)]


def cn_num(s):
    """一二…九十百 → 整数（够用到三位数）。"""
    d = dict(zip("一二三四五六七八九", range(1, 10)))
    if s == "十":
        return 10
    total, cur = 0, 0
    for ch in s:
        if ch in d:
            cur = d[ch]
        elif ch == "十":
            total += (cur or 1) * 10
            cur = 0
        elif ch == "百":
            total += (cur or 1) * 100
            cur = 0
    return total + cur


def chapter_tokens(k):
    """按出现顺序给出（类型, 文本）：anchor 锚点、title 独立成行的题、text 正文行（去模板，保留注文标记）。"""
    s = wikisource(f"唐開元占經 (四庫全書本)/卷{k:03d}")
    for ch, old, new, _ in FIXES:
        if ch == k:
            assert old in s, (k, old)
            s = s.replace(old, new)
    s = re.sub(r"\{\{SKchar\|\d+\}\}", "□", s)
    toks = []
    for line in s.split("\n"):
        line = line.strip()
        if not line or line.startswith("<!--") or line.startswith("{{SKQS"):
            continue
        # 锚点与正文交错：按位置拆开
        pos = 0
        for a in re.finditer(r"\{\{SK anchor\|([^}]*)\}\}", line):
            if a.start() > pos:
                toks.append(("text", line[pos:a.start()]))
            toks.append(("anchor", a.group(1)))
            pos = a.end()
        rest = line[pos:]
        if rest:
            plain = re.sub(r"\{\{[^}]*\}\}", "", rest).strip()
            if re.fullmatch(rf"[^\s]{{1,8}}?[占古][上下]?[{NUM}]+", plain):
                toks.append(("title", plain))
            else:
                toks.append(("text", rest))
    return toks


TITLE = re.compile(rf"(.{{1,8}}?)(?:星)?[占古][上下]?([{NUM}]+)")
SECTION = re.compile(r"(石氏|甘氏|巫咸)(中外官|中官|外官)")
DEFN = re.compile(rf"(石氏|甘氏|巫咸)曰([^曰在，。{{}}\s]{{1,8}}?)([{NUM}]+)星")


def extract():
    entries = []
    for k in CHAPTERS:
        school = section = None
        cur = None
        for kind, x in chapter_tokens(k):
            if kind == "anchor" and SECTION.search(x) and len(x) <= 10:
                m = SECTION.search(x)
                school, section = m.group(1), m.group(2)
                continue
            if school is None:
                continue
            if kind in ("anchor", "title"):
                m = TITLE.fullmatch(x)
                if m:
                    cur = {"school": school, "section": section, "ordinal": cn_num(m.group(2)), "title": m.group(1), "text": ""}
                    entries.append(cur)
                    continue
                if kind == "anchor" and DEFN.match(x):
                    # 题被换成首句锚点：首句即定义句，属于「上一题所开之条」或新条（序号由前后推定）
                    if cur is None or cur["text"]:
                        cur = {"school": school, "section": section, "ordinal": None, "title": DEFN.match(x).group(2), "text": ""}
                        entries.append(cur)
                    cur["text"] += x
                    continue
            if cur is not None:
                cur["text"] += x
    # 补缺失的序号：夹在两个已知序号之间者顺次补
    for i, e in enumerate(entries):
        if e["ordinal"] is None:
            prev = next((p["ordinal"] for p in reversed(entries[:i]) if p["section"] == e["section"] and p["school"] == e["school"] and p["ordinal"]), 0)
            e["ordinal"] = prev + 1
    for e in entries:
        t = e["text"]
        m = DEFN.search(t)
        if not m:
            # 无「某氏曰」的定义句（如巫咸十二国「秦二星在周東南」）：取条目开头的「某某N星」
            m = re.match(rf"\s*()([^\s曰{{}}]{{1,8}}?)([{NUM}]+)星", t)
        e["definition"] = ""
        e["stars"] = ""
        e["note"] = ""
        if m:
            end = t.find("{{SK notes", m.end())
            stop = min([p for p in (t.find(" ", m.end()), end) if p >= 0] or [len(t)])
            e["definition"] = re.sub(r"\{\{[^}]*\}\}", "", t[m.start():stop]).strip()
            e["stars"] = cn_num(m.group(3))
            n = re.match(r"[^{]{0,40}?\{\{SK notes\|([^}]*)\}\}", t[m.end():])
            if n:
                e["note"] = n.group(1)
        # 石氏度数注文（含「去極」）：取整条中第一个，不依赖定义句（如「老人星在弧南」无星数）
        deg = re.search(r"\{\{SK notes\|([^}]*去(?:北)?極[^}]*)\}\}", t)
        if deg and e["school"] == "石氏":
            e["note"] = deg.group(1)
    return entries


def check(entries):
    """各节序号应自 1 连续；返回问题列表。"""
    problems = []
    groups = {}
    for e in entries:
        groups.setdefault((e["school"], e["section"]), []).append(e["ordinal"])
    for school, section, n in LACUNAE:
        groups.setdefault((school, section), []).append(n)
    for key, ords in groups.items():
        want = list(range(1, max(ords) + 1))
        if sorted(ords) != want:
            missing = sorted(set(want) - set(ords))
            dup = sorted({o for o in ords if ords.count(o) > 1})
            problems.append(f"{key}: 共 {len(ords)} 条，最大序号 {max(ords)}，缺 {missing}，重 {dup}")
    return problems, {k: len(v) for k, v in groups.items()}


# 异体字 → 通行字（归一后再繁转简）
VARIANTS = str.maketrans({"𤣥": "玄", "㣲": "微", "闗": "关", "隂": "阴", "髙": "高", "葢": "盖", "皷": "鼓", "鼔": "鼓", "罇": "樽",
                          "䜛": "谗", "廏": "厩", "龞": "鳖", "舎": "舍", "牀": "床", "栁": "柳", "觿": "觽", "罸": "罚"})

# 人工对照：（家, 节, 序号）→ [陈卓星官名…], 依据, 说明
MANUAL = {
    ("石氏", "中官", 1): (["左摄提", "右摄提"], "拆分", "攝提六星夹大角，陈卓分左右"),
    ("石氏", "中官", 13): (["天市左垣", "天市右垣"], "拆分", "天市垣二十二星"),
    ("石氏", "中官", 14): (["帝坐(天市)"], "方位", "帝座一星在市中"),
    ("石氏", "中官", 21): (["东咸", "西咸"], "拆分", "東西咸"),
    ("石氏", "中官", 25): (["河鼓", "鼓旗"], "拆分", "河鼓三星旗九星"),
    ("石氏", "中官", 27): (["瓠瓜"], "别名", "匏瓜即瓠瓜"),
    ("石氏", "中官", 33): (["天大将军"], "别名", "天將軍"),
    ("石氏", "中官", 37): (["五车", "柱(五车)"], "拆分", "五車五星三柱九星"),
    ("石氏", "中官", 39): (["南河", "北河"], "拆分", "南河北河六星夹东井"),
    ("石氏", "中官", 41): (["积水(井宿)"], "方位", "積水一星在北河西星北"),
    ("石氏", "中官", 46): (["太微左垣", "太微右垣"], "拆分", "太微十星"),
    ("石氏", "中官", 53): (["三台", "三台(上台)", "三台(中台)", "三台(下台)"], "拆分", "三台六星两两而居（Stellarium 另列合称「三台」）"),
    ("石氏", "中官", 58): (["北斗", "辅(附北斗)"], "拆分", "北斗七星辅一星"),
    ("石氏", "中官", 59): (["紫微东垣", "紫微西垣"], "拆分", "紫微垣十五星西蕃七东蕃八"),
    ("石氏", "中官", 60): (["北极", "钩陈"], "拆分", "北極五星鈎陳六星"),
    ("石氏", "外官", 1): (["库楼", "柱(库楼)", "衡"], "拆分", "庫樓十星五柱十五星衡四星"),
    ("石氏", "外官", 6): (["龟"], "别名", "原文缺字，「五星在尾南」即龟"),
    ("石氏", "外官", 13): (["羽林军", "垒壁阵"], "拆分", "羽林四十五星壘壁陣十二星"),
    ("石氏", "外官", 14): (["北落师门"], "别名", "北落"),
    ("石氏", "外官", 28): (["弧矢"], "别名", "弧九星在狼东南"),
    ("石氏", "外官", 30): (["天稷"], "别名", "稷五星在七星南"),
    ("甘氏", "中官", 7): (["柱史"], "别名", "柱下史"),
    ("甘氏", "中官", 17): (["传舍"], "别名", "原文缺字，「九星在華蓋上近河」即传舍"),
    ("甘氏", "中官", 28): (["败瓜"], "别名", "原文缺字，「五星在匏瓜傍」即败瓜"),
    ("甘氏", "中官", 29): (["左旗"], "别名", "河皷左旗"),
    ("甘氏", "中官", 41): (["帝坐(大角)"], "别名", "帝席三星在大角北"),
    ("甘氏", "中官", 42): (["天田(角宿)"], "方位", "天田二星在右角北"),
    ("甘氏", "中官", 51): (["从官(太微)"], "方位", "從官一星在太子西北"),
    ("甘氏", "中官", 62): (["坐旗"], "别名", "座旗"),
    ("甘氏", "中官", 64): (["砺石"], "别名", "礪石"),
    ("甘氏", "外官", 10): (["天田(牛宿)"], "方位", "天田九星在牽牛南"),
    ("甘氏", "外官", 17): (["霹雳"], "别名", "原文缺字，霹靂"),
    ("甘氏", "外官", 20): (["𫓧锧"], "方位", "鈇鑕五星在天倉西南"),
    ("甘氏", "外官", 28): (["九斿"], "别名", "九游"),
    ("甘氏", "外官", 38): (["天记"], "别名", "天紀一星在外厨南，即天记（非石氏天纪）"),
    ("巫咸", "中外官", 9): (["土司空(轸宿)"], "方位", "土司空四星在軍門南"),
    ("巫咸", "中外官", 12): (["从官(房宿)"], "方位", "從官二星在房星南"),
    ("巫咸", "中外官", 21): (["天钩"], "别名", "鈎九星如鈎狀在造父北"),
    ("巫咸", "中外官", 23): (["天籥"], "别名", "天鑰"),
    ("巫咸", "中外官", 42): (["𫓧钺"], "方位", "鈇鑕三星在八魁西北一曰鈇鉞"),
}
# 四库本阙文（石氏中官四十七至五十二）：陈卓太微垣中余下五官，按序号推定属石氏
INFERRED_SHI = ["五帝坐", "内屏", "郎位", "郎将", "常陈"]


def attribute(entries):
    import csv as _csv
    from opencc import OpenCC

    t2s = OpenCC("t2s").convert
    norm = lambda x: t2s(x.translate(VARIANTS))  # noqa: E731
    rows = [r for r in _csv.reader(l for l in open(OUT / "chinese_asterisms.csv", encoding="utf-8") if not l.startswith("#"))][1:]
    cz = [r for r in rows if r[0] == "chenzhuo"]
    by_name = {}
    for r in cz:
        by_name.setdefault(r[2], []).append(r)
    base = {}
    for r in cz:
        base.setdefault(re.sub(r"[(（].*[)）]", "", r[2]), []).append(r[2])
    result = {}  # 陈卓星官名 → (家, 依据, 出处)

    def put(name, school, basis, ref):
        assert name in by_name, f"陈卓无此星官：{name}"
        assert name not in result, f"{name} 重复归属：{result[name]} / {school}{ref}"
        result[name] = (school, basis, ref)

    for e in entries:
        key = (e["school"], e["section"], e["ordinal"])
        ref = f"{e['school']}{e['section']}{e['ordinal']}"
        if key in MANUAL:
            names, basis, _ = MANUAL[key]
            for n in names:
                put(n, e["school"], basis, ref)
            continue
        n = norm(e["title"])
        if n in by_name:
            put(n, e["school"], "同名", ref)
            continue
        opts = base.get(n, [])
        tagged = [o for o in opts if f"({e['school']})" in o]
        if len(tagged) == 1:
            put(tagged[0], e["school"], "家括注", ref)
            continue
        d = norm(e["definition"])
        loc = [o for o in opts if re.search(r"[(（](.)", o) and re.search(r"[(（](.)", o).group(1) in d]
        if len(loc) == 1:
            put(loc[0], e["school"], "方位", ref)
            continue
        print("  未对上：", ref, e["title"], opts)
    # 二十八宿与附座（附座随主官；主官已归属者从之）
    for r in cz:
        name = r[2]
        if re.fullmatch(r"\d\dA", r[1]) and name not in result:
            result[name] = ("石氏", "二十八宿", "卷六十至六十三")
    for r in cz:
        m = re.search(r"[(（]附(.+?)[)）]", r[2])
        if m and r[2] not in result:
            host = next((h for h in result if re.sub(r"[(（].*", "", h) == m.group(1) or h == m.group(1)), None)
            if host:
                result[r[2]] = (result[host][0], "附座", f"附{host}")
    if "积尸（舆鬼）" in by_name and "积尸（舆鬼）" not in result and "舆鬼" in result:
        result["积尸（舆鬼）"] = (result["舆鬼"][0], "附座", "附舆鬼（积尸气）")
    for n in INFERRED_SHI:
        if n not in result:
            result[n] = ("石氏", "推定", "石氏中官四十七至五十二（四库本阙）")
    with open(OUT / "sanjia_schools.csv", "w", encoding="utf-8") as f:
        f.write("# 陈卓体系（Stellarium chinese_chenzhuo，CC BY-SA 4.0）各星官所属之家（石氏、甘氏、巫咸），据《开元占经》卷六十至七十\n")
        f.write("# （data/sky_culture/kaiyuan_sanjia.csv）。basis：同名、家括注、方位、别名、拆分、附座、二十八宿、推定（四库本阙文处据序号推定）；\n")
        f.write("# source：所据条目（家+节+序号）。对不上者 school 为空。生成：scripts/sky_culture/sanjia.py。\n")
        f.write("id,name,school,basis,source\n")
        for r in cz:
            sch, basis, ref = result.get(r[2], ("", "", ""))
            f.write(f"{r[1]},{r[2]},{sch},{basis},{ref}\n")
    missing = [r[2] for r in cz if r[2] not in result]
    from collections import Counter

    print("归属：", Counter(v[0] for v in result.values()), "依据：", Counter(v[1] for v in result.values()))
    print("陈卓星官未归属：", len(missing), missing)


# ---------------------------------------------------------------------------------------------------------------------
# 石氏度数

LODGE_ALIASES = [
    ("角", "角"), ("亢", "亢"), ("氐", "氐"), ("房", "房"), ("心", "心"), ("尾", "尾"), ("箕", "箕"),
    ("南斗", "斗"), ("斗", "斗"), ("牽牛", "牛"), ("牛", "牛"), ("須女", "女"), ("湏女", "女"), ("婺女", "女"), ("女", "女"), ("虚", "虚"),
    ("危", "危"), ("營室", "室"), ("室", "室"), ("東壁", "壁"), ("壁", "壁"), ("奎", "奎"), ("婁", "娄"), ("胃", "胃"),
    ("昴", "昴"), ("畢", "毕"), ("觜觿", "觜"), ("觜", "觜"), ("參", "参"), ("東井", "井"), ("井", "井"), ("輿鬼", "鬼"),
    ("鬼", "鬼"), ("栁", "柳"), ("桞", "柳"), ("柳", "柳"), ("七星", "星"), ("星", "星"), ("張", "张"), ("翼", "翼"), ("軫", "轸"),
]
LODGE_RE = "|".join(re.escape(a) for a, _ in LODGE_ALIASES)
LODGE_NORM = dict(LODGE_ALIASES)
CN = "一二三四五六七八九十百"


def gudu(s):
    """「八度少」「二度半」「太」「初」「半度」「五十九度半」「百一十六度」「九十四度半少」「一度弱」→ 古度。
    少 ¼、半 ½、太 ¾；強 +1/12、弱 −1/12（《元史》例）；初 0。「半少」照字面作 ½+¼，并在 flags 中注明。"""
    s = s.strip()
    if s in ("初", "初度", ""):
        return 0.0, ""
    m = re.fullmatch(rf"([{CN}]*)(度)?(少|半|太)?(少|半|太)?(強|强|弱)?(度)?", s)
    if not m:
        raise ValueError(s)
    n, _, f1, f2, qr, _ = m.groups()
    v = float(cn_num(n)) if n else 0.0
    flags = ""
    for f in (f1, f2):
        if f:
            v += {"少": 0.25, "半": 0.5, "太": 0.75}[f]
    if f1 and f2:
        flags = f"「{f1}{f2}」照字面相加"
    if qr in ("強", "强"):
        v += 1 / 12
    elif qr == "弱":
        v -= 1 / 12
    return v, flags


NOTE_RE = re.compile(
    rf"(?P<qual>[^入]{{0,8}}?)入(?P<lodge>{LODGE_RE})(?P<ru>[{CN}]*度?[少半太]?[強强弱]?度?|初度?)"
    rf"去(?:北)?極(?P<qj>[{CN}]+度[少半太]?[少半太]?[強强弱]?)"
    rf"(?:(?:在|直)黄道(?P<io>[内外])?(?P<hd>[{CN}]*度?[少半太]?[強强弱]?))?"
)


def parse_shishi(entries):
    rows = []
    for e in entries:
        if e["school"] != "石氏" or not e["note"]:
            continue
        note = e["note"]
        m = NOTE_RE.search(note)
        ref = f"石氏{e['section']}{e['ordinal']}"
        if not m:
            print("  度数未解析：", ref, note[:60])
            continue
        ru, f1 = gudu(m.group("ru"))
        qj, f2 = gudu(m.group("qj").replace("度", "", 1) if m.group("qj").count("度") > 1 else m.group("qj"))
        hd = ""
        extra = ""
        if m.group("hd"):
            try:
                v, _ = gudu(m.group("hd"))
                if m.group("io"):
                    hd = f"{'+' if m.group('io') == '内' else '-'}{v:.4f}"
                else:
                    extra = f"黄道 {v:g} 度未言内外"
            except ValueError:
                hd = ""
        qual = re.sub(r".*[。，]", "", m.group("qual")).strip()
        rows.append(dict(ref=ref, title=e["title"], qualifier=qual, lodge=LODGE_NORM[m.group("lodge")], ru_du=f"{ru:.4f}",
                         qu_ji=f"{qj:.4f}", huangdao=hd, flags="；".join(x for x in (f1, f2, extra) if x), note=m.group(0)))
    return rows


def parse_lodges():
    """卷六十至六十三：二十八宿宿度、距星描述、去极度、黄道内外度。"""
    rows = []
    for k in range(60, 64):
        s = re.sub(r"\s+", "", wikisource(f"唐開元占經 (四庫全書本)/卷{k:03d}"))
        s = re.sub(r"\{\{SKanchor\|([^}]*)\}\}", r"\1", s)
        s = s.replace("{{SK notes|", "〔").replace("{{SKnotes|", "〔").replace("}}", "〕")
        for m in re.finditer(
            rf"石氏曰(?P<name>[^曰〔〕]{{1,12}}?)(?:〔[^〕]*〕)?(?P<w>[{CN}]+度(?:四分度之一)?)(?:甘氏同)?(?:〔[^〕]*〕)?度?距(?P<det>[^去〔]{{0,12}}?)先?至?"
            rf"去(?:北)?極(?P<qj>[{CN}]+度[少半太]?[少半太]?[強强弱]?)(?:〔(?:在|直)黄道(?P<io>[内外])(?P<hd>[^〕]*)〕)?",
            s,
        ):
            name = m.group("name")
            lodge = next((LODGE_NORM[a] for a, _ in LODGE_ALIASES if name.startswith(a)), None)
            if lodge is None:
                print("  宿名未识：", name)
                continue
            w = m.group("w")
            width = float(cn_num(w.split("度")[0])) + (0.25 if "四分度之一" in w else 0.0)
            qj, fl = gudu(m.group("qj"))
            hd = ""
            if m.group("io"):
                hs = re.sub(r"[也]$", "", m.group("hd"))
                try:
                    v, _ = gudu(hs if hs else "")
                    hd = f"{'+' if m.group('io') == '内' else '-'}{v:.4f}" if hs.strip("度") else ""
                except ValueError:
                    hd = ""
            rows.append(dict(ref=f"石氏二十八宿{lodge}", title=lodge, qualifier="距" + m.group("det"), lodge=lodge, ru_du="0.0000",
                             qu_ji=f"{qj:.4f}", huangdao=hd, flags=fl, note=m.group(0)[:80], width=f"{width:.4f}"))
    return rows


def write_shishi(entries):
    rows = parse_lodges() + parse_shishi(entries)
    with open(OUT / "shishi_xingjing.csv", "w", encoding="utf-8") as f:
        f.write("# 石氏星表度数（《开元占经》卷六十至六十八，四库本）。ref：二十八宿或石氏中官/外官序号；qualifier：注文中所测之星（如「西星」）\n")
        f.write("# 或距星描述；lodge：所入之宿；ru_du：入宿度（古度，二十八宿距星为 0）；qu_ji：去极度（古度）；huangdao：黄道内（+）外（−）度；\n")
        f.write("# width：宿度（仅二十八宿）；flags：解析说明；note：所据原文。少 ¼、半 ½、太 ¾、強 +1/12、弱 −1/12、初 0。\n")
        f.write("# 卷六十亢宿条无去极度，故二十八宿 27 条。生成：scripts/sky_culture/sanjia.py。\n")
        f.write("ref,title,qualifier,lodge,ru_du,qu_ji,huangdao,width,flags,note\n")
        for r in rows:
            f.write(",".join(str(r.get(c, "")).replace(",", "，") for c in ("ref", "title", "qualifier", "lodge", "ru_du", "qu_ji", "huangdao", "width", "flags", "note")) + "\n")
    print("石氏度数：二十八宿", sum(1 for r in rows if "二十八宿" in r["ref"]), "条，中外官", sum(1 for r in rows if "二十八宿" not in r["ref"]), "条")


if __name__ == "__main__":
    entries = extract()
    problems, counts = check(entries)
    print(counts)
    for p in problems:
        print("  问题：", p)
    with open(OUT / "kaiyuan_sanjia.csv", "w", encoding="utf-8") as f:
        f.write("# 《开元占经》三家星官（石氏、甘氏、巫咸中外官）：条目题名、定义句、星数与注文。来源：维基文库「唐開元占經 (四庫全書本)」\n")
        f.write("# 卷六十五至七十。title、definition、note 为原文字形（四库本异体字照录，SKchar 缺字作 □）；石氏度数在 note 中。\n")
        def clean(x):
            x = re.sub(r"\{\{SKchar\|(\d+)\}\}", r"□(SKchar \1)", x)
            return re.sub(r"\{\{SK anchor\|([^}]*)\}\}", r"\1", x)

        for k, old, new, why in FIXES:
            f.write(f"# 订正 卷{k}：「{clean(old)}」→「{clean(new)}」（{why}）\n")
        f.write(f"# 四库本原阙：{'、'.join(f'{a}{b}第{n}条' for a, b, n in LACUNAE)}（卷六十六止于太㣲四十六，卷六十七起于三台五十三）。\n")
        f.write("# 生成：scripts/sky_culture/sanjia.py。\n")
        f.write("school,section,ordinal,title,definition,stars,note\n")
        for e in entries:
            row = [e["school"], e["section"], e["ordinal"], e["title"], e["definition"], e["stars"], e["note"]]
            f.write(",".join(str(x).replace(",", "，") for x in row) + "\n")
    print(len(entries), "条")
    attribute(entries)
    write_shishi(entries)
