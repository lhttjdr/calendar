"""两汉简牍（居延汉简、居延新简等）公文书所记「某年某月某干支朔（某干支）」：检验张表与两汉按术推步（三统、四分）。

汉代官文书书首例作「年号 N 年 M 月 干支朔 干支」（如「始建國五年九月丙午朔乙亥」），朔干支是当时所行历日（第 1 级实物）。
释文据《中国简牍集成》（敦煌文艺出版社 2001）各册排印释文；OCR（RapidOCR，竖排）候选由外部步骤生成
（JSON：页序 → [[x, y, 文字, 置信]]），本脚本只做匹配与比对；有文字的今人释文（如张俊民《肩水金关汉简（壹）释文》Word 录文）
转成纯文本后用 candidates-text 抽候选，同一正则与判定。

候选（candidates）：抽「年号 N 年（閏）M 月 干支朔」，年号按两汉、新莽年号表（data/calendar/汇总/era_names_merged.csv，
政权 汉、新）定年；新莽月名经 month_label_reforms.csv 换算为建寅表月。逐条算张表朔与推步朔（太初后三统、元和二年后四分，
同 han_pingshuo_check.py；太初以前不推）。
核对：OCR 所读朔干支与张表或推步相合者，错读恰合的概率约 1/60，按 OCR 采信（核对 = 「OCR合」）；不合者逐条对原书页图
人工核对（核对 = 「图」），读法与 OCR 不同者记于「图读」。只有简文残缺、年号或数字不能辨者不收。
同一简见于两本释文而读法不同者，以所从之本为准，另一本之行核对记「重出」，列出而不计数（悬泉汉简以发掘整理者《敦煌悬泉汉简释粹》为准）。
逐条写入 data/calendar/hanjian_jiri.csv（列：书,册,PDF页,集成编号,年号,年,闰,月,朔干支,原文,核对,图读,备注），
check 生成 doc/calendar/hanjian-checks.md。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/hanjian_shuo_check.py candidates <书> <册> <ocr.json>
            uv run --project scripts python scripts/calendar_data/hanjian_shuo_check.py candidates-text <书> <册> <释文.txt>
            uv run --project scripts python scripts/calendar_data/hanjian_shuo_check.py check
"""
import csv
import json
import re
import sys
from collections import defaultdict
from pathlib import Path

import opencc

sys.path.insert(0, str(Path(__file__).resolve().parent))
import discriminating_records as d  # noqa: E402
import gaiyuan_date as g  # noqa: E402
import han_pingshuo_check as hp  # noqa: E402
import shiwu_fenlei as fl  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
T2S = opencc.OpenCC("t2s")
GZ = g.GANZHI if hasattr(g, "GANZHI") else ["甲乙丙丁戊己庚辛壬癸"[i % 10] + "子丑寅卯辰巳午未申酉戌亥"[i % 12] for i in range(60)]
MON = ["正月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "十一月", "十二月"]
CSV = ROOT / "data/calendar/hanjian_jiri.csv"
ALIAS = {}  # 年号别写 → 正名（eras() 填）
COLS = ["书", "册", "PDF页", "集成编号", "年号", "年", "闰", "月", "朔干支", "原文", "核对", "图读", "备注"]


def cn(s):
    s = s.replace("廿", "二十").replace("卅", "三十").replace("卌", "四十").replace("亖", "四").replace("桼", "七").replace("泰", "七")  # 新莽书「四」作「亖」、「七」作「桼」（释文或作「泰」）
    v = {"元": 1, "正": 1, "一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}
    if s in v:
        return v[s]
    if s == "十":
        return 10
    if "十" in s:
        a, b = s.split("十", 1)
        return (v[a] if a else 1) * 10 + (v[b] if b else 0)
    return None


def eras():
    """年号（简体）→ [(政权, 元年)]：汉、新；同名年号（如东西汉皆有之「建始」等）全列，按候选年份取。"""
    out = defaultdict(set)
    for r in csv.DictReader(open(ROOT / "data/calendar/汇总/era_names_merged.csv", encoding="utf-8")):
        if r["regime"] in ("汉", "新") and r["era_year"] == "1" and -140 <= int(r["lunar_year"]) <= 220:
            out[r["era_name"]].add((r["regime"], int(r["lunar_year"])))
    # 年号表未收、而河西简所行者：建世（刘盆子，25—27；居延新简 EPF22 有「建世二年」），历日按汉历
    out.setdefault("建世", set()).add(("汉", 25))
    # 简文「太」常作「大」（「大始元年」「大初」）：别名同指一号，抽出后归正名（ALIAS）
    for k in list(out):
        if k.startswith("太"):
            ALIAS["大" + k[1:]] = k
            out["大" + k[1:]] = out[k]
    return {k: sorted(v) for k, v in out.items()}


def reckon_tables(main):
    """三统（前 103—84）、四分（85—220）推步表 (岁, 月, 闰) → 朔 JDN（偏移同 han_pingshuo_check：与张表对齐）"""
    out = {}
    for name, y0, y1 in (("三统", -103, 84), ("四分", 85, 220)):
        _, off = hp.align(name, main, y0, y1)
        for k, v in hp.table(name, y0, y1, off).items():
            if y0 <= k[0] <= y1 or (k[0] == y0 - 1 and k[1] >= 11):
                out[k] = v
    return out


def judge(regime, year, month, leap, gz, main, reck):
    """→ (表月, 张表朔干支, 合张表, 推步朔干支, 合推步)；月名不存在返回 None"""
    t = g.table_month(regime, year, MON[month - 1], leap)
    if t is None:
        return None
    zs = main.get(t)
    rs = reck.get(t)
    zg = GZ[(zs + 49) % 60] if zs is not None else ""
    rg = GZ[(rs + 49) % 60] if rs is not None else ""
    return t, zg, (zg == gz) if zg else None, rg, (rg == gz) if rg else None


def _pattern(ev):
    era_pat = "|".join(sorted({k for k in ev} | {T2S.convert(k) for k in ev}, key=len, reverse=True))
    N = "[元正一二三亖四五六七桼泰八九十廿卅卌]+"
    G = "[甲乙丙丁戊己庚辛壬癸][子丑寅卯辰巳午未申酉戌亥]"
    # 月数可缺：「建武六年閏月甲子朔」只书閏月，取该年有闰之月
    # 新莽地皇简文多作「（新始建国）地皇上戊 N 年」，年号与年数之间容「上戊」
    return re.compile(rf"({era_pat})(?:上戊)?({N})年(閏|闰)?({N})?月({G})朔")


def _judge_match(m, ev, main, reck):
    """一处匹配 → [(年号, 年, 闰, 月, 朔干支, judge 结果)]（同名年号各出一条）；数字不能解者返回 []"""
    era, yr, leap, gz = T2S.convert(m.group(1)), cn(m.group(2)), bool(m.group(3)), m.group(5)
    era = ALIAS.get(era, era)
    mo = cn(m.group(4)) if m.group(4) else None
    if yr is None or (mo is None and not leap) or (mo is not None and not 1 <= mo <= 12):
        return []
    out = []
    for regime, y0 in ev.get(era, []):
        year = y0 + yr - 1
        mo_ = mo
        if mo_ is None:  # 只书閏月：张表该年之闰（无闰则以推步之闰）
            mo_ = next((k for k in range(1, 13) if (t := g.table_month(regime, year, MON[k - 1], True)) and (t in main or t in reck)), None)
            if mo_ is None:
                continue
        out.append((era, year, int(leap), mo_, gz, judge(regime, year, mo_, leap, gz, main, reck)))
    return out


def _emit(rows):
    w = csv.DictWriter(sys.stdout, fieldnames=list(rows[0].keys()) if rows else ["无"])
    w.writeheader()
    w.writerows(rows)
    print(f"# 候选 {len(rows)}，合张表 {sum(r['合张表'] is True for r in rows)}，合推步 {sum(r['合推步'] is True for r in rows)}", file=sys.stderr)


def _row(book, vol, page, hit, conf, text):
    era, year, leap, mo, gz, j = hit
    return dict(书=book, 册=vol, PDF页=page, 年号=era, 年=year, 闰=leap, 月=mo, 朔干支=gz,
                张表=j[1] if j else "无此月", 合张表=j[2] if j else "", 推步=j[3] if j else "",
                合推步=j[4] if j else "", 置信=conf, 原文=text)


def candidates(book, vol, path):
    ev = eras()
    main = d.tables()["张表"]
    reck = reckon_tables(main)
    pat = _pattern(ev)
    data = json.load(open(path))
    rows, seen = [], set()
    for pg, boxes in sorted(data.items(), key=lambda kv: int(kv[0])):
        # 竖排一列常被 OCR 断成数框：x（框左上角）相差 30 像素以内者视为同列，按 y 接成整列再匹配；单框与整列皆匹配，按（页, 匹配文）去重
        cols = []
        for b in sorted(boxes, key=lambda b: -b[0]):
            if cols and abs(cols[-1][0][0] - b[0]) <= 30:
                cols[-1].append(b)
            else:
                cols.append([b])
        joined = [[c[0][0], min(b[1] for b in c), "".join(b[2] for b in sorted(c, key=lambda b: b[1])), min(b[3] for b in c)] for c in cols if len(c) > 1]
        for x, y, text, conf in boxes + joined:
            s = T2S.convert(text)
            for m in pat.finditer(s):
                if (pg, m.group(0)) in seen:
                    continue
                seen.add((pg, m.group(0)))
                rows += [_row(book, vol, int(pg) + 1, h, conf, text) for h in _judge_match(m, ev, main, reck)]
    _emit(rows)


def candidates_text(book, vol, path):
    """纯文本释文（如 Word 录文转出之 txt）抽候选：去空白后整篇匹配（日期跨行亦可接上），
    图片占位「EMBED …」作「□」；「PDF页」记匹配处之后最近的简号（如 73EJT1:125A，释文例于简末书简号），
    其后 400 字内无简号者记「段 N」（N 为原文行号）。原文取匹配起至简号（或其后 40 字）。"""
    ev = eras()
    main = d.tables()["张表"]
    reck = reckon_tables(main)
    pat = _pattern(ev)
    raw = open(path, encoding="utf-8").read()
    raw = re.sub(r"EMBED\s+[A-Za-z.]+\d*", "□", raw)
    s, line_of, ln = [], [], 1
    for ch in raw:
        if ch == "\n":
            ln += 1
        if not ch.isspace():
            s.append(ch)
            line_of.append(ln)
    s = "".join(s)
    # 繁转简后匹配（年号表为简体）；逐字转，保证位置与原文一一对应，原文栏仍取繁体
    ss = "".join(c if len(t := T2S.convert(c)) != 1 else t for c in s)
    jian = re.compile(r"\d+EJ[A-Z]*\d+[:：]\d+[A-Z]?(\+\d+[A-Z]?)*")
    rows = []
    for m in pat.finditer(ss):
        k = jian.search(s, m.end(), m.end() + 400)
        page = k.group(0) if k else f"段 {line_of[m.start()]}"
        text = (s[m.start():k.start()] if k else s[m.start():m.end() + 40])[:60]
        rows += [_row(book, vol, page, h, "", text) for h in _judge_match(m, ev, main, reck)]
    _emit(rows)


def lipu():
    """出土历谱整年月朔（data/calendar/yinwan_lipu.csv）与张表、三统推步逐月比较：朔干支与月大小。"""
    main = d.tables()["张表"]
    reck = reckon_tables(main)
    rows = [r for r in csv.DictReader(l for l in open(ROOT / "data/calendar/yinwan_lipu.csv", encoding="utf-8") if not l.startswith("#"))]

    def size(t, k):
        s = t.get(k)
        nxt = min((v for v in t.values() if s is not None and v > s), default=None)
        return "" if s is None or nxt is None else ("大" if nxt - s == 30 else "小")
    md = ["# 尹湾汉墓历谱与张表、三统推步（自动生成）", "",
          "> 由 `scripts/calendar_data/hanjian_shuo_check.py lipu` 生成，勿手改。录文 `data/calendar/yinwan_lipu.csv`（《尹湾汉墓简牍》释文）。", "",
          "| 件 | 年 | 月 | 历谱朔 | 大小 | 张表朔 | 大小 | 三统推步朔 | 大小 | 分类 |", "|---|---|---|---|---|---|---|---|---|---|"]
    n = dict(n=0, z=0, r=0)
    cats = []
    for r in rows:
        k = (int(r["年"]), int(r["月"]), r["闰"] == "1")
        zg = GZ[(main[k] + 49) % 60] if k in main else "（无）"
        rg = GZ[(reck[k] + 49) % 60] if k in reck else "（无）"
        zs, rs = size(main, k), size(reck, k)
        n["n"] += 1
        n["z"] += zg == r["朔干支"] and zs == r["大小"]
        n["r"] += rg == r["朔干支"] and rs == r["大小"]
        lab = ("閏" if k[2] else "") + MON[k[1] - 1]
        c, note = fl.classify(rg if k in reck else "", zg if k in main else "", [r["朔干支"]], "A", "历谱")
        if c:
            cats.append((c, "A" if c != "无证" else "—"))
        md.append(f"| {r['件']} | {k[0]} | {lab} | {r['朔干支']} | {r['大小']} | {zg}{' ✓' if zg == r['朔干支'] else ' ✗'} | {zs}{' ✓' if zs == r['大小'] else ' ✗'} | "
                  f"{rg}{' ✓' if rg == r['朔干支'] else ' ✗'} | {rs}{' ✓' if rs == r['大小'] else ' ✗'} | {(c or '') + (f'（{note}）' if note else '')} |")
    md += ["", f"共 {n['n']} 月：朔与大小皆合张表 {n['z']}，皆合三统推步 {n['r']}。", "",
           "分类对象是三统推步与张表（按朔日；月大小由次月朔定，已含在相邻月中）；历谱是实物历日（A 级）。" + fl.RULE, "",
           fl.headline(cats, "三统推步") + "。", ""] + fl.grade_table(cats) + [""]
    (ROOT / "doc/calendar/yinwan-lipu-checks.md").write_text("\n".join(md), encoding="utf-8")
    print("\n".join(md))


NOTES = """## 闰位线索、新莽纪年与他本异读

以下据逐条表与所补诸书考查（2026-11-27）。简号、页码为所引书之编号；「张表」与三统推步在两汉逐月相同（见上），故下文合称「两家」。

### 闰位线索

- **神爵元年（前 61）**：两家皆閏四月（四月壬子、閏四月壬午）。居延汉简「神爵元年四月壬午朔己酉」（集成第 5 册 PDF 298）四月朔即两家之閏四月朔。
  陈梦家《汉简年历表叙》（《汉简缀述》，中华书局 1980，第一节之三「汉简年历表」（一）闰月第 3 例，页 233）据居延汉简元康五年诏书册
  （10.27、5.10、332.26、10.33、10.30、10.32、10.29、10.31 八简相联）下达日期「閏月丁巳」「閏月庚申」皆在閏三月壬子朔内，定是年閏三月壬子、四月壬午。
  同年他简：元康五年正月癸未、二月癸丑、三月癸未（悬泉、居延）与神爵元年十月己卯（悬泉）两说皆合（三月以前、閏月以后两说同朔），
  诏书册之閏月丁巳不在两家閏四月（壬午朔）之内，而四月壬午朔一简合閏三月之说。实物两证一致：**实行之历閏三月，两家闰位同误**（只移閏月，月朔不差）。
  集成第 7 册 PDF 236「神爵元年四月癸未朔乙酉張掖肩水」，劳榦《居延汉简考释·释文》卷一（页 20—21）读作「神爵元年正月癸未朔乙酉張掖肩水」
  （同页三栏皆作正月，简号 306.4、5.9 等），正月癸未合两家；四月癸未则与两说皆不合（差一日）。两本异读未见图版，暂从集成计数，劳榦本列为重出。
- **元寿元年（前 2）**：两家閏十一月（十一月丙申、閏十一月丙寅、十二月乙未）。居延汉简「建平五年十二月丙寅朔」（集成第 6 册 PDF 86；劳榦释文同）
  十二月朔即两家之閏十一月朔，陈梦家同页第 5 例据此定当閏十二月乙未，与刘羲叟《长历》同。同年他简（建平五年正月辛丑、八月戊辰、十月丁卯）皆在閏前，两说同朔，不能分；
  所补诸书未见是年十一月、閏月之简；劳榦 495.12「建平五年十二月辛卯朔庚寅」朔与日自相矛盾（庚寅不在辛卯朔之月），不能为证。
  悬泉汉简（肆）Ⅱ90DXT0115②:244「入穬麥四斗　建平五年閏月己巳縣泉置嗇夫博受遮要置護」（伍楚嘉 2025-10-20 简帛网文引，未见原书）：己巳为两家閏十一月丙寅朔之第四日；
  若如陈说閏在十二月丙寅之后（閏十二月乙未），閏月无己巳。两枚当时之简各指一边，两歧，閏位仍不能定（同文据 Ⅱ90DXT0115②:171A 复原之《建平五年曆日》只存正至三月朔，余按陈垣表推，不作证）。
  仍只一证，推定閏十二月（两家闰位同误），待他简。
- **太始元年（前 96）**：疏勒河流域出土汉简 413「大始元年十二月辛丑朔」，辛丑为两家十一月朔。陈梦家同页第 1 例：两家置閏于上年（前 97）十二月丁丑，
  而《汉书·五行志》太始元年「正月乙巳晦」日食，则是年正月丁丑朔，与此简合，是年閏在岁终（閏十二月辛未）。若是，太始元年全年月名比两家晚一月，两家同误。
  同书 386「大始三年閏月辛酉朔」合两家閏九月。
- **鸿嘉三年（前 18）**：两家閏九月（九月庚午、閏九月庚子）。劳榦释文 32.17「鴻嘉三年閏月庚午朔癸酉」（释文卷一，PDF 45，对图），庚午为两家九月朔，
  即陈梦家同页第 4 例所据之简，合閏八月庚午之说；本表按「只书閏月取两家閏月」记于閏九月，归同误（合前后月）。只此一简，同年他简未见，推定閏八月（两家闰位同误）。
- 陈梦家所举其余闰位、朔日异例（始元七年閏月甲辰三简、始建国二年十一月丙子、元兴元年历谱閏十月辛亥等），简文为不书朔之纪日或历谱，不入本表，列此备核：
  始元七年两家閏三月壬申，閏月甲辰不在其内（陈氏以为閏二月或四月）；
  始建国二年两家閏寅正九月（丑正十月）癸亥，「十一月丙子」不在丑正十一月（壬辰朔）内，陈氏以为閏在丑正十一月（刘表）；
  元兴元年两家七月壬子、九月辛亥、閏九月辛巳、十月庚戌，金关历谱作七月癸丑、九月壬子、閏十月辛亥（差一日且闰位异）；始元元年正月两家戊寅，简作己卯。
  以上皆两家相同之月而简与之异，若属实则为两家同误，不是张表与推步之分歧。

### 他本异读

- 劳榦释文与集成（合校系）同简异读，凡月名「十」「七」之异四见：91.1 本始元年七月庚寅、534.3 始元三年七月甲午、534.2 征和四年七月癸亥（534.2、534.3 集成作七月，91.1《人名编年》引作七月，皆合两家；91.1 七月读法未回集成核，暂不计），
  劳榦皆作「十月」；23.13 竟宁元年「十月戊辰朔」，戊辰为两家七月朔，当同。汉简「七」字作「十」形，劳榦本此类当从合校系读「七」。四简劳榦本皆列为重出不计。
- 神爵元年 306.4 等劳榦作正月、集成作四月，见上。
- 劳榦本与集成不合而未见他本者（初元三年十月壬子、建平二年二月壬子、阳朔二年四月乙未、建武三年十月己酉、建平五年十二月辛卯朔庚寅、永元四年閏月丙子、本始二年二月辛巳朔己未）照劳榦计数，皆同误或另有合者，待图版。

### 《居延汉简人名编年》线索未能回核者

编年所引带朔之简与本表已收者重出 155 处（读法同）；未收者回集成对图补录（逐条表核对 = 图，备注「人名编年 PDF N 引」）。下列未能回核，不入表：
EPT51:228「建始元年四月甲午朔乙未」、EPT53:63「元康二年五月己巳朔辛卯」（集成同号为他简，编年简号或有误）；甲附 36「永光元年二月丙申朔」（两家戊戌，差二日，未见原释文）；EPT59:57「始建国天凤二年七月丙申朔戊戌」（合两家，集成第 11 册未回检）。
编年中据陈垣表推算之朔（元康二年閏七月、建武二十六年十一月等）及作者拟补之年（如「河平四年正月甲寅朔」）不收。

### 新莽「差一年」（已解）

前次所收新莽简 22 条，18 条按建丑改制（始建国元年正月 = 初始元年十二月癸酉朔，`month_label_reforms.csv`）换算后合两家；
不合者 4 条 3 月：始建国三年正月丙戌、始建国三年九月壬午（居延新简，集成第 11 册 PDF 125、162）、天凤三年正月丁巳（敦煌马圈湾，甘肃省卷 PDF 26、29 两简），
所记朔恰为下一年同名之月。复核图版（2026-11-27）：四简集成原文皆作「**亖**」（四横，新莽「四」字），不是「三」；前次 OCR 识作「三」，图核时亦未辨。
改读「始建国四年」「天凤四年」后四条皆合两家，逐条表已改（备注注明）。

佐证：集成第 12 册 EPT68:193「始建國天鳳亖年六月甲申朔丁酉」（《人名编年》引作「四年」）、EPF22 地皇诸简「新始建國地皇上戊亖年」十一月丁丑、桼月己卯、五月庚辰、八月己酉（集成注皆云公元二十三年），
所记朔皆合地皇四年；而作「三」之简（始建国三年二月辛酉、十二月丙辰，天凤三年八月己丑、十二月丁亥）皆合三年。

结论：**不是改制换算之误，也不是当时纪年之异**；是识读把「亖」当成「三」。新莽简凡「三年」而合下一年者，先查原书是否「亖」。
本项目建丑换算（与陈梦家《汉简年历表叙》页 231 所述新改以建丑之月为正月、初始元年十二月癸酉朔即始建国元年正月癸酉朔同）由 29 条新莽简（含补检天凤一条、地皇六条）全数验证。
抽候选脚本已加：「亖」→四、「桼／泰」→七、年号与年数间容「上戊」、年号表外补「建世」。第 3—11 册于 2026-11-28 全部重新 OCR（RapidOCR，200 dpi）、按此重抽，与已收逐条比对（键：册、页、月、朔干支），新出 15 条皆看图或按两家合采信：
新收 13 条——「上戊」简 3（天凤上戊六年五月癸酉、地皇上戊二年十一月己丑、上戊三年正月戊子，第 9 册）、「桼／泰」月 1（天凤二年七月丙申，第 11 册）、
「建世」2（二年二月甲午、三月癸亥，第 9、11 册）、前次漏抽 5（元康四年二月己未左後部简、阳朔元年九月乙巳、地节三年十一月癸未、建平三年五月庚戌、建始元年四月甲午），
皆合两家；不合两家 1、改读后合 1：第 3 册马圈湾 MC.191「始建国三年三月乙酉朔」，集成排印作「三」，而中研院史语所「文物图象研究室」敦煌汉简释文作「亖」（网页以「鑻」字代亖：同页 MC.70、MC.99 天凤亖年诸简与 TH.1806 注「《沙釋》作“四”，《大釋》作“鑻”」同例），乙酉正合始建国四年三月，从之、按四年计；
第 9 册 E.P.T14:14 建武八年桼月丁亥朔（两家皆六月丁巳、闰六月丁亥）；循此检出第 12 册 EPT68:123「建武八年六月戊午朔」（前次 OCR 作「二六月」漏抽，图核），
二简相接：六月戊午小、七月丁亥，中无閏六月，是年实行之历閏在七月后，张表、三统推步两家同误（许名瑲简帛网文亦据此二简推閏七月、「置閏在常例後一月」）。重出 2 条（本始六年三月、太始元年十二月，同《疏勒河》本）。
编者注释中据表推算之朔（如「鸿嘉二年六月丁未朔，無辛卯」）不收。RapidOCR 仍把「亖」识作「三」，此类只能由「三年」而合下一年之不合者看图查出。

### 陈梦家《汉简年历表叙》要点

据《汉简缀述》（中华书局 1980）「汉简年历表叙」第一节「汉简年历」（页 229—238）：

- 一、年表与朔闰表（页 229—230）：汉简为当时官文书，未经传抄，记年月日较《汉书》完整；记日法五类（「年月干支朔干支」「年月干支」「年月日」「年月干支朔日序」「年月干支朔日序干支」，后者东汉之制），月日之间介以朔干支自汉初至东汉为官文书固有形式。
- 二、两汉历术（页 231—232）：两汉、新所用历分四期：古四分（殷或颛顼，高祖元年至太初元年）、太初（太初元年至孺子婴初始元年，更始元年至章帝元和二年）、三统（始建国元年至地皇四年）、后汉四分（元和二年至延康元年）。
  新莽改建丑，初始元年十二月癸酉朔即始建国元年正月癸酉朔；更始元年二月复寅正，故是年有两十月（地皇四年十月戊申朔、更始元年十月丁丑朔）。
  太初以后闰在无中气之月称「閏月」。汉简称閏不书几月，由所记朔日可推。汉简所记历日据当时颁历，天官或临时改动朔闰，故与后人推定之朔闰表（汪曰桢、张其翮、陈垣）不免出入。
- 三、汉简年历表（页 232—234）：1962—1963 年据居延、酒泉、敦煌、罗布淖尔简与两《汉书》纪、志、表编排。所举不合诸表之闰月七例（太始元年、始元七年、神爵元年、鸿嘉三年、元寿元年、始建国二年、元兴元年，见上）
  与朔日差一日二例（始元元年正月、元兴元年七月至十月）；又据新旧年号并见推改元之月（如本始六年三月仍称本始，河平五年四月改阳朔，建平五年即元寿元年）。
- 四、汉简历谱（页 234—238）：可推定年之历谱十五种（本始二年、本始四年、元康三年、神爵元年、神爵三年、五凤元年、永光五年、鸿嘉四年、永始四年、建平二年、居摄元年、居摄三年、永元六年、永元十七年、永兴元年），
  形制五式；历谱八节合太初术平气所推，伏、腊、建除、反支、血忌、八魁诸注。
- 陈氏历表系据简排之今人整理（E 级），不作证据；其所引简号与原文据释文回核：本表已收者见上，其余为纪日或历谱，列「闰位线索」备核。

### 未能读之书

- 网盘《敦煌汉简释文》（吴礽骧等）：`敦煌汉简释文.rar` 内为《论语集注》PDF 与一可执行文件，非是书；`敦煌汉简释文` 目录为超星 PDG（HH 02 加密 CCITT 页），解不出。
  是书即《中国简牍集成》甘肃省卷敦煌汉简释文所据，已收之甘肃省卷诸条与之同源。
"""

NOTES = NOTES.split("\n")


def check():
    """逐条表（data/calendar/hanjian_jiri.csv）与张表、推步比较，生成 doc/calendar/hanjian-checks.md。"""
    ev = eras()
    main = d.tables()["张表"]
    reck = reckon_tables(main)
    rows = list(csv.DictReader(l for l in open(CSV, encoding="utf-8") if not l.startswith("#")))
    md = ["# 汉简公文书朔日与张表、两汉推步（自动生成）", "",
          "> 由 `scripts/calendar_data/hanjian_shuo_check.py check` 生成，勿手改。逐条表 `data/calendar/hanjian_jiri.csv`，方法见脚本说明与 "
          "[calendar-compendium.md](calendar-compendium.md)「两汉」。", "",
          "| 书 | PDF页 | 简文日期 | 公元 | 简文朔 | 张表朔 | 合张表 | 推步朔 | 合推步 | 核对 | 分类 | 原文 |", "|---|---|---|---|---|---|---|---|---|---|---|---|"]
    n = defaultdict(int)
    # 先按表月聚合（同简异本「重出」不计），每月一个分类；简牍公文书为当时文字（B 级），一条即可判，与他条冲突归③
    keyed = []
    for r in rows:
        regime, y0 = next(((rg, y0) for rg, y0 in ev.get(r["年号"], []) if y0 <= int(r["年"]) < y0 + 30), ("汉", int(r["年"])))
        keyed.append((r, regime, y0, judge(regime, int(r["年"]), int(r["月"]), r["闰"] == "1", r["朔干支"], main, reck)))
    by_month = defaultdict(list)
    for r, _, _, j in keyed:
        if j and r["核对"] != "重出":
            by_month[j[0]].append((r["朔干支"], j))
    cls = {}
    for t, obs in by_month.items():
        j = obs[0][1]
        cls[t] = fl.classify(j[3], j[1], [o for o, _ in obs], "B", "简文")
    for r, regime, y0, j in keyed:
        zg, zok, rg_, rok = (j[1], j[2], j[3], j[4]) if j else ("（无此月）", False, "", None)
        if r["核对"] == "重出":  # 同一简两本释文异读，以所从之本为准，此行列出而不计数（见备注）
            n["dup"] += 1
        else:
            n["n"] += 1
            n["z"] += bool(zok)
            n["r"] += bool(rok)
            n["rn"] += rok is not None
        y = int(r["年"])
        yr = f"前 {1 - y}" if y <= 0 else str(y)
        ny = int(r["年"]) - y0 + 1
        lab = f"{r['年号']}{'元' if ny == 1 else ny}年{('閏' if r['闰'] == '1' else '')}{MON[int(r['月']) - 1]}"
        c, note = cls.get(j[0], (None, "")) if j and r["核对"] != "重出" else (None, "")
        cl = (c + (f"（{note}）" if note else "")) if c else ""
        md.append(f"| {r['书']} | {r['PDF页']} | {lab} | {yr} | {r['朔干支']} | {zg} | {'✓' if zok else '✗'} | {rg_} | "
                  f"{'' if rok is None else ('✓' if rok else '✗')} | {r['核对']} | {cl} | {r['原文'][:40]} |")
    md += ["", f"共 {n['n']} 条：合张表 {n['z']}；推步可比 {n['rn']} 条，合 {n['r']}。"
           + (f"另 {n['dup']} 条为同简异本重出（核对 = 重出），列出不计。" if n["dup"] else ""), ""]
    cats = [(c, "B" if c not in ("无证",) else "—") for c, _ in cls.values() if c]
    solo = [t for t, (c, _) in cls.items() if c is None]
    solo_ok = sum(1 for t in solo if any(o == by_month[t][0][1][1] for o, _ in by_month[t]))
    md += ["## 分类（按月）", "",
           "分类对象是两汉推步（太初后三统、元和二年后四分）与张表：简文是当时官文书（B 级当时文字），一条即可判，同月诸条冲突归③。"
           "同月多简各算一条（同简异本只算一条）。逐条表「分类」列为该条所在之月的分类。" + fl.RULE, "",
           f"推步可比 {len(cats)} 月：" + fl.headline(cats) + "。"
           + (f"无推步之月 {len(solo)}，只与张表比：合 {solo_ok}、不合 {len(solo) - solo_ok}，不入分类。" if solo else ""), ""]
    md += fl.grade_table(cats) + [""]
    # 各来源条数（计数之条、重出之条）
    by_book = defaultdict(lambda: [0, 0, 0])
    for r, _, _, j in keyed:
        b = by_book[r["书"]]
        if r["核对"] == "重出":
            b[2] += 1
        else:
            b[0] += 1
            b[1] += bool(j and j[2])
    md += ["## 各来源条数", "", "| 书 | 计数之条 | 合张表 | 重出（列出不计） |", "|---|---|---|---|"]
    md += [f"| {k} | {v[0]} | {v[1]} | {v[2]} |" for k, v in by_book.items()] + [""]
    md += NOTES
    (ROOT / "doc/calendar/hanjian-checks.md").write_text("\n".join(md), encoding="utf-8")
    print(next(l for l in md if l.startswith("推步可比")))


if __name__ == "__main__":
    if sys.argv[1] == "candidates":
        candidates(*sys.argv[2:5])
    elif sys.argv[1] == "candidates-text":
        candidates_text(*sys.argv[2:5])
    elif sys.argv[1] == "lipu":
        lipu()
    elif sys.argv[1] == "check":
        check()
