"""《汉书·天文志》（卷二十六）月与五星凌犯、相合记录：逐条解析纪年、月、日干支，按历法换算日期范围，目标匹配陈卓体系星官。
写出 data/phenomena/hanshu_star_records.csv，供 rust/core 的 sky_culture::tests_records 检验。

与后汉脚本（houhanshu_star_records.py）共用星官匹配、占辞剔除、宿度解析；另处理：
- 多天体相合：「水、木、火三合於東井」「金、木俱在東井」→ 一条记录，bodies 列空格分隔，verb「合」或「在」；
- 只记到月、不记日干支者也收，日期范围取整月（jdn_start—jdn_end）；记日干支者范围为当日；
- 纪年：汉初诸帝以帝号纪年（孝文後二年、孝景中三年……），太初以前月份按秦汉初复原历（hzx.py，十月岁首），
  以后按 vendor 实历朔日表；「其N年」承上文帝号与中、後元；
- 日期须与事件同句或与上一条记录同句（以「。」为界），否则不收；每段开始时日期上下文恢复为上一段段首纪年
  （占辞后的应验叙述会改写日期，不带入下段）；
- 只按历法换算日期，不用天象。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/qinhan_eval/hanshu_star_records.py
"""
import csv
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
sys.path.insert(0, str(HERE))
import gaiyuan_date as g  # noqa: E402
import hzx  # noqa: E402
import houhanshu_star_records as hh  # noqa: E402

ROOT = HERE.parents[2]
OUT = ROOT / "data/phenomena/hanshu_star_records.csv"

# 汉初帝号纪年：元年的天文纪年（汉元年 = 前 206 = −205）
REIGN = {"高帝": -205, "惠帝": -193, "高后": -186, "文帝": -178, "景帝": -155}
SUB = {("文帝", "後"): -162, ("景帝", "中"): -148, ("景帝", "後"): -142}
EMP_ALIAS = {"高皇帝": "高帝", "漢": "高帝", "孝惠": "惠帝", "高后": "高后", "孝文": "文帝", "孝景": "景帝",
             "孝武": "武帝", "孝昭": "昭帝", "孝宣": "宣帝", "孝元": "元帝", "孝成": "成帝", "孝哀": "哀帝", "孝平": "平帝"}
ERA = {"建元": -139, "元光": -133, "元朔": -127, "元狩": -121, "元鼎": -115, "元封": -109, "太初": -103, "天漢": -99,
       "太始": -95, "征和": -91, "始元": -85, "元鳳": -79, "元平": -73, "本始": -72, "地節": -68, "元康": -64,
       "神爵": -60, "五鳳": -56, "甘露": -52, "黃龍": -48, "初元": -47, "永光": -42, "建昭": -37, "竟寧": -32,
       "建始": -31, "河平": -27, "陽朔": -23, "鴻嘉": -19, "永始": -15, "元延": -11, "綏和": -7, "建平": -5,
       "元壽": -1, "元始": 1, "居攝": 6, "初始": 8}
GZ = "[甲乙丙丁戊己庚辛壬癸][子丑寅卯辰巳午未申酉戌亥]"
NUM = "[元一二三四五六七八九十]+"
DATE = re.compile(
    rf"^(?:至)?(?:其)?(?P<emp>{'|'.join(EMP_ALIAS)})?(?P<era>{'|'.join(ERA)})?(?P<sub>中|後)?(?:(?P<yr>{NUM})年)?[春夏秋冬]?"
    rf"(?:(?P<mon>閏|正|十二|十一|十|[二三四五六七八九])月)?(?P<xun>[上中下]旬)?(?P<gz>{GZ})?(?:[甲乙丙丁戊]夜|夕|晨|旦|昏)?$"
)
NAMES = {"月": "Moon", "熒惑": "Mars", "火": "Mars", "太白": "Venus", "金": "Venus", "歲星": "Jupiter", "木": "Jupiter",
         "填星": "Saturn", "鎮星": "Saturn", "土": "Saturn", "辰星": "Mercury", "水": "Mercury"}
BODY_RE = "|".join(sorted(NAMES, key=len, reverse=True))
# 多天体：「水、木、火三合於東井」「金、木俱在東井」「火與水晨出東方」不收（出东方无星官）
MULTI = re.compile(rf"^(?P<bodies>(?:(?:{BODY_RE})[、與与]?)+?)(?:[二三四五]|俱)?(?P<verb>合|在)[於于]?(?P<target>.+)$")
SINGLE = re.compile(rf"^(?P<body>{BODY_RE})?(?:星)?(?P<verb>逆行守|犯|入|乘|守|掩|食|在|貫|觸)(?P<target>.+)$")


def month_span(n, month, leap):
    """(朔 JDN, 下月朔 JDN, 历法)。太初元年（−103）以前用复原历（十月岁首，闰为「后九」），以后用实历朔日表。"""
    if n <= -104:
        labs = hzx.year(n)
        keys = [lab for lab, _ in labs]
        lab = "后9" if leap else str(month)
        i = keys.index(lab)
        nxt = labs[i + 1][1] if i + 1 < len(labs) else hzx.year(n + 1)[0][1]
        return labs[i][1], nxt, "复原历"
    if leap:
        lm = next(lm for ly, lm, lp, _ in g._months() if ly == n and lp)
        s, e = g.month_span(n, lm, True)
    else:
        s, e = g.month_span(n, month, False)
    return s, e, "实历朔日表"


def main():
    ast = hh.load_asterisms()
    text = hh.fetch("漢書/卷026")
    start = text.index("漢元年十月")
    end = text.index("==校勘記==") if "==校勘記==" in text else len(text)
    text = text[start:end]
    rows = []
    emp = era = sub = None
    yr = mon = gz = xun = None
    leap = False
    last_bodies, last_sentence, fresh_sentence = None, -1, -1
    sent_no = 0
    # 段落：每段开始时日期上下文恢复为上一段段首纪年（段内「占曰」之后的应验叙述如「至河平元年三月，旱」会改写上下文，不应带入下段）
    lead_ctx = None
    pieces = []
    for para in text.split("\n\n"):
        pieces.append(("PARA", False))
        pieces.extend(hh.segments(para))
    para_first_date = False
    for seg, note in pieces:
        if seg == "PARA":
            if lead_ctx is not None:
                emp, era, sub, yr, mon, gz, xun, leap = lead_ctx
            para_first_date = True
            continue
        if note:
            continue  # 《汉书》注文为颜师古等训诂，不含观测
        seg = hh.drop_other_quotes(seg)
        for sent in re.split(r"[。！？]", seg):
            sent_no += 1
            for clause in re.split(r"[，；：「」『』\s]", sent):
                clause = clause.strip("　 ")
                if not clause:
                    continue
                m = DATE.match(clause)
                if m and any(m.group(k) for k in ("emp", "era", "sub", "yr", "mon", "gz", "xun")):
                    if m["emp"]:
                        emp, era, sub = EMP_ALIAS[m["emp"]], None, None
                    if m["era"]:
                        # 武帝「後元」为年号；景帝「後元」为帝号纪年分段
                        era, sub = m["era"], None
                    if m["sub"]:
                        if m["sub"] == "後" and emp == "武帝":
                            era, sub = "後元", None
                        else:
                            sub = m["sub"]
                    if m["yr"]:
                        yr, mon, gz = hh.cnnum(m["yr"]), None, None
                    if m["mon"]:
                        leap = m["mon"] == "閏"
                        mon = None if leap else (1 if m["mon"] == "正" else hh.cnnum(m["mon"]))
                        gz, xun = None, None
                        fresh_sentence = sent_no
                    if m["xun"]:
                        xun = "上中下".index(m["xun"][0])
                    if m["gz"]:
                        gz = m["gz"]
                        fresh_sentence = sent_no
                    if para_first_date:
                        lead_ctx = (emp, era, sub, yr, mon, gz, xun, leap)
                        para_first_date = False
                    continue
                mm = MULTI.match(clause)
                ms = SINGLE.match(clause)
                if mm and len(re.findall(BODY_RE, mm["bodies"])) >= 2:
                    bodies = [NAMES[b] for b in re.findall(BODY_RE, mm["bodies"])]
                    verb, target = mm["verb"], mm["target"]
                elif ms:
                    if ms["body"]:
                        bodies = [NAMES[ms["body"]]]
                    elif sent_no == last_sentence and last_bodies:
                        bodies = last_bodies
                    else:
                        continue
                    verb, target = ms["verb"], ms["target"]
                else:
                    continue
                if not (fresh_sentence == sent_no or sent_no == last_sentence):
                    continue
                if yr is None or (mon is None and not leap):
                    continue
                if era == "後元":
                    base = -87
                elif era:
                    base = ERA[era]
                elif emp in REIGN:
                    base = SUB.get((emp, sub), REIGN[emp]) if sub else REIGN[emp]
                else:
                    continue
                year = base + yr - 1
                try:
                    s0, e0, cal = month_span(year, mon, leap)
                except Exception:
                    continue
                if gz:
                    off = (g.GANZHI.index(gz) - (s0 + 49) % 60) % 60
                    in_month = s0 + off < e0
                    jdn = s0 + off if in_month else min((s0 + off - 60, s0 + off), key=lambda x: abs(x - (s0 + e0) // 2))
                    j0 = j1 = jdn
                elif xun is not None:
                    in_month, j0, j1 = True, s0 + 10 * xun, (s0 + 10 * xun + 9 if xun < 2 else e0 - 1)
                else:
                    in_month, j0, j1 = True, s0, e0 - 1
                kind, tid, tname, qual = hh.match_target(target, ast)
                ld = hh.lodge_degree(target) if verb == "在" else None
                if kind == "unknown" and not ld:
                    continue
                y0, m0, d0 = g.from_jdn(j0)
                rows.append(dict(
                    juan="026", reign=emp or "", era=era or (sub or ""), year_in_era=yr, year=year,
                    month=("闰" if leap else "") + (str(mon) if mon else "") + ("" if xun is None else "上中下"[xun] + "旬"), ganzhi=gz or "", calendar=cal,
                    jdn_start=j0, jdn_end=j1, julian=f"{y0}-{m0:02d}-{d0:02d}", in_month=int(in_month),
                    bodies=" ".join(bodies), verb=verb.translate(hh.T2S), target_kind=kind, target_id=tid, target_name=tname,
                    qualifier=qual[:12], lodge=ld[0] if ld else "", lodge_deg=ld[1] if ld else "", text=sent.strip()[:120],
                ))
                last_bodies, last_sentence = bodies, sent_no
    for r in rows:
        for k, v in r.items():
            if isinstance(v, str):
                r[k] = v.replace(",", "，").replace('"', "＂").replace("\n", " ").replace("\r", "")
    with open(OUT, "w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)
    from collections import Counter
    print(len(rows), "条", Counter(r["verb"] for r in rows), Counter(r["target_kind"] for r in rows),
          "多天体", sum(" " in r["bodies"] for r in rows), "只记到月", sum(not r["ganzhi"] for r in rows),
          "干支不在月", sum(not r["in_month"] for r in rows))


if __name__ == "__main__":
    main()
