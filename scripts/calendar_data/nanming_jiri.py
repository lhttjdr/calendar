"""南明史料纪日抽取：hanshu_cache/nanming_ws/（nanming_fetch.py 所取维基文库原文）→ data/calendar/本纪纪日/nanming_*.csv。

每书逐段（行）读，段首前缀「年（干支年或年号）／季／月（含閏）／日序／干支／朔晦」记为一条纪日；段中「某月某干支朔」也记一条。
年份：优先取括注或句首的干支年（﹝丁亥﹞、「甲申，我大清……」），否则按年号换算（弘光、隆武、紹武、永曆、監國、順治、康熙）；
无年标而由十一、十二月退到正、二月者视为入下一年，其余月序倒退视为追叙、整段不取。录文者后加的括注（小腆紀年「壬寅（十五日）」之类、{{*|(一六四九)}}）一律删去，只用原书文字。
攷曰、按语等括注内的日期是他书异文，不取。日序：「十八日庚申」记 position = 日18（干支须为月首起第 18 日）。
政权列：海東逸史（鲁监国事）1645—1653 记「鲁监国」；1662 年起记「明郑」；余记「南明」（绍武并入南明）。
《小腆紀年》《小腆紀傳》《明季南略》《臺灣外紀》是清人后编，记日所据之历待检验（看它们合大统还是合清时宪）；
《先王實錄》（杨英，延平王户官，永历三至十六年随军）、《行在陽秋》（王夫之）、《所知錄》（钱澄之）、《思文大紀》、
《海上見聞錄》（阮旻锡）是当时人所记，书中「當時人」列标 1。
用法（仓库根）：python3 -I scripts/calendar_data/nanming_jiri.py
"""
import csv
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "hanshu_cache/nanming_ws"
OUT = ROOT / "data/calendar/本纪纪日"
GAN, ZHI = "甲乙丙丁戊己庚辛壬癸", "子丑寅卯辰巳午未申酉戌亥"
GZ = [GAN[i % 10] + ZHI[i % 12] for i in range(60)]
GZR = f"[{GAN}][{ZHI}]"
NUM = {"元": 1, "正": 1, "一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9, "十": 10, "十一": 11, "十二": 12, "冬": 11, "臘": 12}
# 年号元年之公历年（农历年）
ERA = {"弘光": 1645, "隆武": 1645, "紹武": 1646, "永曆": 1647, "永历": 1647, "監國": 1646, "监国": 1646, "順治": 1644, "顺治": 1644, "康熙": 1662, "崇禎": 1628}
# (文件名, 书名, 卷页通配, 是否当时人, 书首年：无年标时的起始农历年)
BOOKS = [
    ("nanming_xiaotian_jinian", "小腆紀年", "小腆紀年__卷*", 0, 1644),
    ("nanming_xingzai_yangqiu", "行在陽秋", "行在陽秋", 1, 1646),
    ("nanming_suozhilu", "所知錄", "所知錄__卷*", 1, None),
    ("nanming_siwen_daji", "思文大紀", "思文大紀__卷*", 1, 1645),
    ("nanming_hongguang_shilu", "弘光實錄鈔", "弘光實錄鈔__卷*", 1, 1644),
    ("nanming_haidong_yishi", "海東逸史", "海東逸史__*", 0, None),
    ("nanming_minhai_jiyao", "閩海紀要", "閩海紀要__卷*", 0, None),
    ("nanming_haishang_jianwen", "海上見聞錄", "海上見聞錄__卷*", 1, None),
    ("nanming_taiwan_waiji", "臺灣外紀", "臺灣外紀__*", 0, None),
    ("nanming_mingji_nanlue", "明季南略", "明季南略__卷*", 0, None),
    ("nanming_xianwang_shilu", "先王實錄", "先王實錄", 1, None),
]


# 夹有人物履历、追叙（「己未，授工部營繕司主事」之类以干支纪年）的书：段首单个干支不记，只记带月标、朔晦、日序者
import os

DEBUG = bool(os.environ.get("NANMING_DEBUG"))
HEADING_ONLY = {"小腆紀年"}
HEAD_RE = re.compile(rf"^(?:{GZR}[，、])?我大清[^，。]*?(?P<era>順治|顺治|康熙)(?P<yn>元|[一二三四五六七八九十]+)年(?:{GZR})?")
STRICT = {"明季南略", "臺灣外紀", "海東逸史"}


def num(s):
    s = s.replace("廿", "二十").replace("卅", "三十")
    if s in NUM:
        return NUM[s]
    if s.startswith("初"):
        return num(s[1:])
    if s.startswith("十"):
        return 10 + num(s[1:])
    if "十" in s:
        a, b = s.split("十", 1)
        return NUM[a] * 10 + (num(b) if b else 0)
    return NUM[s]


def year_of_gz(g, near=None):
    """干支年 → 1644（甲申）—1703 之间唯一一年：南明诸书所纪不出此范围，追叙前朝之干支年不致带偏上下文"""
    return 1644 + (GZ.index(g) - GZ.index("甲申")) % 60


def clean(t):
    t = re.sub(r"^\s*\d+\s*\t", "", t)  # 部分卷录文每行前有行号
    t = re.sub(r"\.mw-parser-output[^{}]*\{[^{}]*\}", "", t)  # 解析页混入的样式表
    for _ in range(4):  # 模板可嵌套（{{*|{{YL|1651}}}}），由内向外删
        t = re.sub(r"\{\{[^{}]*\}\}", "", t)
    t = re.sub(r"\[\[(?:[^|\]]*\|)?([^\]]*)\]\]", r"\1", t)
    for _ in range(3):  # 录文者括注、攷曰等（可嵌套，如〈（……）〉）
        t = re.sub(r"（[^（）]*）|\([^()]*\)|\[[^\[\]]*\]|〈[^〈〉]*〉", "", t)
    t = t.replace("﹝", "").replace("﹞", "")
    return t


PRE = re.compile(
    rf"^(?P<y>(?:(?P<era>弘光|隆武|紹武|永曆|永历|監國|监国|順治|顺治|康熙)?(?P<yn>元|[一二三四五六七八九十]+)年)?(?P<yg>{GZR})?(?P<ygn>年)?[，,、]?(?:我大清[^，。]*?年)?)"
    rf"(?P<season>[春夏秋冬])?[，,]?(?P<m>(?P<leap>閏|闰)?(?P<mn>正|十一|十二|十|[一二三四五六七八九]|冬|臘)月|(?P<yd>元旦))?[，,]?"
    rf"(?P<d>(?:初|二十|廿|三十|卅|十)?[一二三四五六七八九十]?日)?[，,]?(?P<g>{GZR})?[，,]?(?P<pos>朔|晦)?"
)
INLINE = re.compile(rf"(?P<leap>閏|闰)?(?P<mn>正|十一|十二|十|[一二三四五六七八九])月(?P<g>{GZR})朔")


def joined_lines(text):
    """括注跨行者（〈（……）〉 之类）并为一段，免得括注里的干支被当成纪日、年标被截断"""
    buf, depth = "", 0
    for ln in text.split("\n"):
        buf = buf + ln if buf else ln
        depth = sum(buf.count(c) for c in "（(〈") - sum(buf.count(c) for c in "）)〉")
        if (depth <= 0 and not buf.rstrip().endswith("我大清")) or len(buf) > 4000:
            yield buf
            buf = ""
    if buf:
        yield buf


def extract(files, book, contemp, out, y0=None, strict=False):
    y, m, leap = y0, None, False
    for f in files:
        for raw in joined_lines(f.read_text()):
            line = clean(raw).strip().lstrip("　 ").replace("<br>", "")
            if not line or line.startswith(("|", "{", "=", "<")):
                # 节标题中的干支年（「==永曆三年 己丑==」「== 乙酉，順治二年、南京弘光元年 ==」）
                hd = re.search(GZR, line) if line.startswith("=") else None
                if hd:
                    y, m, leap = year_of_gz(hd.group(0), y or 1650), None, False
                continue
            mt = PRE.match(line)
            if not mt or not mt.group(0):
                continue
            gy = mt.group("yg")
            # 干支年：须带「年」或后接「，我大清」/季/月，才当年标（防段首纪日干支被误读）
            if book in HEADING_ONLY:
                # 只认「甲申，我大清世祖章皇帝順治元年……」式年标；段首其余年号、干支年皆追叙（传记、前事），整段不取
                hm = HEAD_RE.match(line)
                if hm:
                    y, m, leap = ERA[hm.group("era")] + num(hm.group("yn")) - 1, None, False
                    mt = PRE.match(line[hm.end():])
                    if DEBUG:
                        print("年标", y, line[:50])
                elif mt.group("yn") or (gy and (mt.group("ygn") or mt.group("season") or mt.group("m"))):
                    continue
                gy = mt.group("yg")
                is_year_gz = False
            else:
                is_year_gz = gy and (mt.group("yn") or mt.group("ygn") or mt.group("season") or mt.group("m"))
            if mt.group("yn") and not is_year_gz and mt.group("era") in (None, "崇禎"):
                continue  # 无年号无干支之「某年」、崇祯某年：段首追叙前事，整段不取、不改上下文
            if mt.group("yn") or is_year_gz:
                y = year_of_gz(gy, y or 1650) if is_year_gz else ERA[mt.group("era")] + num(mt.group("yn")) - 1
                if DEBUG:
                    print("年标", y, line[:50])
                m, leap = None, False
            if mt.group("m"):
                mn, lp = (1, False) if mt.group("yd") else (num(mt.group("mn")), bool(mt.group("leap")))
                if y is not None and m is not None and not (mt.group("yn") or is_year_gz) and (mn, lp) < (m, leap):
                    if m >= 11 and mn <= 2 and book not in HEADING_ONLY:
                        y += 1  # 十一、十二月之后见正、二月：入下一年（书缺年标）
                    else:
                        continue  # 其余月序倒退：段首追叙前事（解说、传记），整段不取、不改上下文
                m, leap = mn, lp
            g = mt.group("g") or (gy if gy and not is_year_gz else None)
            # 严格模式（夹有人物履历、追叙的书）：段首须有月标，或书朔晦、日序，才记
            if strict and not (mt.group("m") or mt.group("pos") or (mt.group("d") and mt.group("d") != "日")):
                g = None
            if g and y is not None and m is not None:
                pos = mt.group("pos") or ""
                if mt.group("d") and mt.group("d") != "日":
                    pos = f"日{num(mt.group('d')[:-1])}"
                out.append((y, m, leap, g, pos, book, contemp, line[:60]))
            # 段中「某月某干支朔」
            for im in INLINE.finditer(line[mt.end():]):
                if y is None:
                    break
                mn, lp = num(im.group("mn")), bool(im.group("leap"))
                if m is not None and mn < m:
                    break  # 段中倒叙，不取
                m, leap = mn, lp
                out.append((y, m, leap, im.group("g"), "朔", book, contemp, line[:60]))


def regime(book, y):
    if y >= 1662:
        return "明郑"
    if book == "海東逸史" and y <= 1653:
        return "鲁监国"
    return "南明"


def drops():
    """考证表（data/calendar/nanming_xixia_regime_kaozheng.csv）处置为「不计」「编次」之条：抽取误读或原书编次，剔除"""
    f = ROOT / "data/calendar/nanming_xixia_regime_kaozheng.csv"
    rows = csv.DictReader(l for l in open(f, encoding="utf-8") if not l.startswith("#"))
    return [(r["书"], int(r["年"]), int(r["月"]), r["干支"], r["原文前缀"]) for r in rows if r["处置"] in ("不计", "编次")]


def main():
    dr = drops()
    for stem, book, pat, contemp, y0 in BOOKS:
        files = sorted(f for f in SRC.glob(pat + ".txt") if not f.name.endswith(".html.txt"))
        files = [f.with_name(f.stem + ".html.txt") if (f.with_name(f.stem + ".html.txt")).exists() else f for f in files]
        files = [f for f in files if not f.name.endswith(("序.txt", "跋.txt", "自序.txt"))]
        if not files:
            print(book, "无原文")
            continue
        recs = []
        extract(files, book, contemp, recs, y0, strict=book in STRICT)
        n0 = len(recs)
        keep = [r for r in recs if not any(d[0] == stem and d[1] == r[0] and d[2] == r[1] and d[3] == r[3] and r[7].startswith(d[4]) for d in dr)]
        print(book, "剔除（考证表不计、编次）", len(recs) - len(keep))
        recs = keep
        recs = [r for r in recs if 1645 <= r[0] <= 1683]  # 分政权南明一路起于 1645
        if not recs:
            (OUT / f"{stem}.csv").unlink(missing_ok=True)
            print(book, "无可用纪日（多书日序不书干支），不出文件")
            continue
        with open(OUT / f"{stem}.csv", "w", encoding="utf-8", newline="") as fh:
            fh.write(f"# {book}（维基文库，scripts/calendar_data/nanming_jiri.py 抽取）；當時人={contemp}\n")
            w = csv.writer(fh)
            w.writerow(["regime", "era", "year_in_era", "year", "month", "month_name", "leap", "ganzhi", "calendar", "in_month", "jdn", "julian", "position", "buzi", "text"])
            for y, mm, lp, g, pos, b, c, text in recs:
                w.writerow([regime(b, y), "", "", y, mm, ("閏" if lp else "") + f"{mm}月", int(lp), g, "", "", "", "", pos, 0, text.replace(",", "，")])
        print(book, n0, "条，1645—1683 年", len(recs))


if __name__ == "__main__":
    main()
