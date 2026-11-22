"""三国吴蜀、十六国后秦北凉之史书纪日抽取（分政权推步检验用，rust/core/examples/regime_benji_eval.rs）。

底本：维基文库（zh.wikisource.org，action=raw）——
  吴：《三國志》卷四十七吳書·吳主傳、卷四十八吳書·三嗣主傳；唐許嵩《建康實錄》卷二至四（吳，維基文库整理本，校记
      <ref> 不取）——许嵩据吴国史等旧籍，与《三國志》部分同源，单列一书以便分别统计；
  蜀：《三國志》卷三十二蜀書·先主傳、卷三十三蜀書·後主傳；
  后秦：《晉書》卷一百十六至一百十九載記（姚弋仲、姚襄、姚萇、姚興、姚泓）；
  北凉：《晉書》卷一百二十九載記（沮渠蒙遜）、《魏書》卷九十九（盧水胡沮渠蒙遜傳）、《宋書》卷九十八（氐胡傳）。
裴松之注（{{*|…}}）不取（注引他书，纪年体例不一）。
按段落顺序维持「年号、年、月、闰」状态：「年号 + N年」「N年」（段首或句首，后接季、月或标点）定年，「明年」进一年，
「改元」「改年」而未书新年号者取本政权当年始用之年号；「（春夏秋冬）N月」定月，「閏月」为本年当前月之闰；
月首干支后书「朔」「晦」者、或书「某月朔干支」者记 position。年份换算：年号元年之农历年（各政权年号表见 ERAS）。
只取所属政权行用本国历之年内者（吴 222—280、蜀 221—263、后秦 384—416、北凉 398—439）。
抽取结果经逐条人工核对（年月归属不可靠者在 MANUAL_DROP 中剔除并注明理由，或在 MANUAL_FIX 中改正），写入
data/calendar/本纪纪日/{sanguozhi_wushu,sanguozhi_shushu}.csv（后秦、北凉诸书无可用之日干支，不生成文件），列与其余本纪纪日表一致
（calendar、in_month、jdn、julian 列留空：在月与否由检验程序按各家历日判定）。
原 data/calendar/本纪纪日/sanguozhi.csv 中吴、蜀之行（吴 39、蜀 1）年月多误（如东兴之战建兴元年 252 年十二月记作太元元年
251 年十月），已删去，由本脚本输出取代；魏之行不动。
用法（仓库根）：python3 scripts/calendar_data/sanguo_shiliuguo_jiri.py <维基文库原文目录>
  目录内文件名：三國志_卷47.txt 等（title 中 / 换作 _），可用 curl -A "calendar-research-script/0.1" 取 action=raw。
"""
import csv
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CN = {c: i for i, c in enumerate("〇一二三四五六七八九", 0)}


def cn2int(s):
    if s == "元":
        return 1
    if s == "正":
        return 1
    n = 0
    if "十" in s:
        a, _, b = s.partition("十")
        n = (CN[a] if a else 1) * 10 + (CN[b] if b else 0)
    else:
        n = CN[s]
    return n


# 年号 → 元年之农历年。各书按所述政权列出优先序（同名年号以本国为先）
WU = {"黃武": 222, "黃龍": 229, "嘉禾": 232, "赤烏": 238, "太元": 251, "神鳳": 252, "建興": 252, "五鳳": 254, "太平": 256,
      "永安": 258, "元興": 264, "甘露": 265, "寶鼎": 266, "建衡": 269, "鳳凰": 272, "天冊": 275, "天璽": 276, "天紀": 277}
SHU = {"章武": 221, "建興": 223, "延熙": 238, "景耀": 258, "炎興": 263}
HAN_WEI = {"建安": 196, "延康": 220, "黃初": 220, "太和": 227, "青龍": 233, "景初": 237, "正始": 240, "嘉平": 249,
           "正元": 254, "景元": 260, "咸熙": 264, "泰始": 265, "咸寧": 275, "太康": 280}
LATER_QIN = {"白雀": 384, "建初": 386, "皇初": 394, "弘始": 399, "永和": 416}
NORTHERN_LIANG = {"神璽": 397, "天璽": 399, "永安": 401, "玄始": 412, "承玄": 428, "義和": 431, "承和": 433, "緣禾": 432, "太緣": 435}
JIN_SONG = {"永和": 345, "升平": 357, "昇平": 357, "太元": 376, "隆安": 397, "元興": 402, "義熙": 405, "永初": 420,
            "景平": 423, "元嘉": 424, "天興": 398, "天賜": 404, "永興": 409, "神瑞": 414, "泰常": 416, "始光": 424,
            "神䴥": 428, "延和": 432, "太延": 435}

SOURCES = [
    # (文件, 书, 政权, 年号表（先者优先）, 本国历行用年段, 输出)
    ("三國志_卷47.txt", "《三國志》卷四十七吳主傳", "吴", [WU, HAN_WEI], (222, 280), "sanguozhi_wushu"),
    ("三國志_卷48.txt", "《三國志》卷四十八三嗣主傳", "吴", [WU, HAN_WEI], (222, 280), "sanguozhi_wushu"),
    ("建康實錄_卷02.txt", "《建康實錄》卷二吳中太祖下", "吴", [WU, HAN_WEI], (222, 280), "jiankang_shilu_wu"),
    ("建康實錄_卷03.txt", "《建康實錄》卷三吳中下廢帝景帝", "吴", [WU, HAN_WEI], (222, 280), "jiankang_shilu_wu"),
    ("建康實錄_卷04.txt", "《建康實錄》卷四吳下後主", "吴", [WU, HAN_WEI], (222, 280), "jiankang_shilu_wu"),
    ("三國志_卷32.txt", "《三國志》卷三十二先主傳", "蜀", [SHU, HAN_WEI], (221, 263), "sanguozhi_shushu"),
    ("三國志_卷33.txt", "《三國志》卷三十三後主傳", "蜀", [SHU, HAN_WEI], (221, 263), "sanguozhi_shushu"),
    ("晉書_卷116.txt", "《晉書》卷一百十六載記", "后秦", [LATER_QIN, JIN_SONG], (384, 416), "shiliuguo_zaiji"),
    ("晉書_卷117.txt", "《晉書》卷一百十七載記", "后秦", [LATER_QIN, JIN_SONG], (384, 416), "shiliuguo_zaiji"),
    ("晉書_卷118.txt", "《晉書》卷一百十八載記", "后秦", [LATER_QIN, JIN_SONG], (384, 416), "shiliuguo_zaiji"),
    ("晉書_卷119.txt", "《晉書》卷一百十九載記", "后秦", [LATER_QIN, JIN_SONG], (384, 416), "shiliuguo_zaiji"),
    ("晉書_卷129.txt", "《晉書》卷一百二十九載記", "北凉", [NORTHERN_LIANG, JIN_SONG], (398, 439), "shiliuguo_zaiji"),
    ("魏書_卷99.txt", "《魏書》卷九十九", "北凉", [NORTHERN_LIANG, JIN_SONG], (398, 439), "shiliuguo_zaiji"),
    ("宋書_卷98.txt", "《宋書》卷九十八", "北凉", [NORTHERN_LIANG, JIN_SONG], (398, 439), "shiliuguo_zaiji"),
]

GAN, ZHI = "甲乙丙丁戊己庚辛壬癸", "子丑寅卯辰巳午未申酉戌亥"
NUM = "元一二三四五六七八九十"
MON = r"(?P<mon>正|十一|十二|十|[一二三四五六七八九])月"

# 逐条人工核对后剔除或改正之条：键 (文件, 干支, 原文前 8 字)
MANUAL_DROP = {
    ("三國志_卷48.txt", "丁亥", "皓舉家西遷，以太"): "太康元年五月丁亥集于京邑：晋纪年，属晋历",
    ("三國志_卷48.txt", "甲申", "四月甲申，詔曰："): "天纪四年（280）三月吴亡后晋武帝诏，属晋历",
    ("建康實錄_卷04.txt", "丁亥", "夏四月，遣使送後"): "太康元年五月丁亥集於洛陽：晋纪年，属晋历",
    ("建康實錄_卷04.txt", "甲午", "甲午，晉帝使詔慰"): "同上，晋事",
    ("建康實錄_卷04.txt", "癸亥", "癸亥，晉瑯琊王伷"): "三月壬寅吴主降后晋军入城之事，非吴历日",
    ("建康實錄_卷04.txt", "乙亥", "乙亥，置酒大會，"): "同上，晋将置酒",
}
MANUAL_FIX = {
    # 《孙休传》开篇追叙太元二年（252）事，「孫亮廢，己未」承上篇太平三年（258）九月戊午废亮，下接十月戊寅、己卯
    ("三國志_卷48.txt", "己未", "孫亮廢，己未，孫"): {"year": 258, "month": 9, "month_name": "9月"},
    ("三國志_卷48.txt", "戊寅", "十月戊寅，行至曲"): {"year": 258},
    ("三國志_卷48.txt", "己卯", "己卯，休至，望便"): {"year": 258},
    # 《建康实录》卷三废帝纪：「十二月丙申，大風雷雹」承建兴元年（252）冬十月城东兴，即《吴志》「十二月朔丙申，大風雷電」
    ("建康實錄_卷03.txt", "丙申", "十二月丙申，大風"): {"year": 252},
}


def strip_markup(s):
    # 去裴注 {{*|…}}（可嵌套）
    out, i, depth = [], 0, 0
    while i < len(s):
        if s.startswith("{{*|", i):
            depth, j = 1, i + 4
            while j < len(s) and depth:
                if s.startswith("{{", j):
                    depth, j = depth + 1, j + 2
                elif s.startswith("}}", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            i = j
            continue
        out.append(s[i])
        i += 1
    s = "".join(out)
    s = re.sub(r"\{\{YL\|([^|}]*)(\|[^}]*)?\}\}", r"\1", s)
    s = re.sub(r"\{\{!\|([^}]*)\}\}", r"\1", s)
    s = re.sub(r"\[\[(?:[^|\]]*\|)?([^\]]*)\]\]", r"\1", s)
    s = re.sub(r"-\{([^}]*)\}-", r"\1", s)
    s = re.sub(r"<ref[^>]*>.*?</ref>|<[^>]+>", "", s, flags=re.S)
    s = re.sub(r"\{\{[^{}]*\}\}", "", s)
    return s


def extract(fname, book, regime, eras, span, src_dir):
    text = strip_markup((src_dir / fname).read_text(encoding="utf-8"))
    allera = {}
    for t in reversed(eras):
        allera.update(t)
    own = eras[0]
    era_pat = "|".join(sorted(allera, key=len, reverse=True))
    tok = re.compile(
        rf"(?P<era>{era_pat})?(?P<yr>[{NUM}]{{1,3}})年(?=[春夏秋冬，、。正一二三四五六七八九十閏是])"
        rf"|(?P<ming>明年)|(?P<gai>改元|改年|改(?=。))(?:為|曰)?(?P<newera>{era_pat})?"
        rf"|(?P<run>閏月)|{MON}"
        rf"|(?P<gz>[{GAN}][{ZHI}])(?P<pos>朔|晦)?"
    )
    rows, year, month, leap, cur_n = [], None, None, False, 1
    for para in text.split("\n"):
        if para.startswith("=="):  # 传主更替：年月状态清空，待下一个年号或年出现
            year, month, leap = None, None, False
            continue
        for m in tok.finditer(para):
            st = m.start()
            prev = para[max(0, st - 3):st]
            if m.group("yr"):
                # 只认段首、句首（前一字为。，或段首、或季节字）与「年号 N 年」
                if m.group("era"):
                    base = allera[m.group("era")]
                elif not (st == 0 or para[st - 1] in "。　 ，；" or prev.endswith(("其", "至"))):
                    continue
                else:
                    base = year - (cur_n - 1) if year is not None else None
                    if base is None:
                        continue
                n = cn2int(m.group("yr"))
                if m.group("era"):
                    cur_era_base = base
                year = base + n - 1
                cur_n = n
                month, leap = None, False
            elif m.group("ming") and year is not None:
                year += 1
                cur_n += 1
                month, leap = None, False
            elif m.group("gai") and year is not None:
                ne = m.group("newera")
                base = allera[ne] if ne else next((b for e, b in own.items() if b == year), None)
                if base is not None:
                    cur_n = year - base + 1
            elif m.group("run"):
                leap = True
            elif m.group("mon"):
                month, leap = cn2int(m.group("mon")), False
            elif m.group("gz"):
                if prev.endswith(("歲在", "歲次", "太歲", "在", "問", "實")) or para[m.end():m.end() + 1] in ("年", "歲", "遠") or prev.endswith("元年"):
                    continue
                if year is None or month is None:
                    continue
                if (GAN.index(m.group("gz")[0]) - ZHI.index(m.group("gz")[1])) % 2:
                    continue  # 非干支（如「甲未」）
                if not (span[0] <= year <= span[1]):
                    continue
                # 句子：自上一个句号至下一个句号
                a = max(para.rfind("。", 0, st), -1) + 1
                b = para.find("。", m.end())
                sent = para[a:b if b >= 0 else len(para)]
                rows.append({"regime": regime, "era": "", "year_in_era": "", "year": year, "month": month,
                             "month_name": ("閏" if leap else "") + f"{month}月", "leap": int(leap), "ganzhi": m.group("gz"),
                             "calendar": "", "in_month": "", "jdn": "", "julian": "", "position": m.group("pos") or ("朔" if prev.endswith("月朔") else ""),
                             "buzi": 0, "text": sent.replace(",", "，"), "book": book, "file": fname})
    return rows


def main():
    src_dir = Path(sys.argv[1])
    out = {}
    for fname, book, regime, eras, span, dst in SOURCES:
        if not (src_dir / fname).exists():
            print("缺", fname)
            continue
        for r in extract(fname, book, regime, eras, span, src_dir):
            key = (fname, r["ganzhi"], r["text"][:8])
            if key in MANUAL_DROP:
                continue
            r.update(MANUAL_FIX.get(key, {}))
            if r["month_name"] != ("閏" if r["leap"] else "") + f"{r['month']}月":
                r["month_name"] = ("閏" if r["leap"] else "") + f"{r['month']}月"
            # 同书同年同月同干支只留一条（如《建康实录》卷二、卷三重出大帝崩日）
            if any((x["year"], x["month"], x["leap"], x["ganzhi"]) == (r["year"], r["month"], r["leap"], r["ganzhi"]) for x in out.get(dst, [])):
                continue
            out.setdefault(dst, []).append(r)
    cols = ["regime", "era", "year_in_era", "year", "month", "month_name", "leap", "ganzhi", "calendar", "in_month", "jdn",
            "julian", "position", "buzi", "text"]
    for dst, rows in out.items():
        p = ROOT / "data/calendar/本纪纪日" / f"{dst}.csv"
        with open(p, "w", encoding="utf-8", newline="") as f:
            w = csv.DictWriter(f, fieldnames=cols, extrasaction="ignore")
            w.writeheader()
            w.writerows(rows)
        print(dst, len(rows))
        for r in rows:
            print(" ", r["file"][:9], r["year"], r["month_name"], r["ganzhi"], r["position"], r["text"][:50])


if __name__ == "__main__":
    main()
