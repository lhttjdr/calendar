#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""生成 data/calendar/汇总/era_names_merged.csv（+.json）——"多来源交叉校对出汇总表"路线的第一版汇总表。

设计：
- 长表（tidy）格式：每年每个并存政权一行，而不是宽表拼接展示字符串（对比
  regime_era_calendar_matrix.csv 那种"帝-年号（历法）"拼接单元格，本表每个字段独立成列，供程序直接解析）。
- 年份用**天文纪年**（含公元0年=公元前1年），与 vendor/calendar/chinese_calendar/ 下的
  年号表.csv、实历朔日表.csv 一致，也与 rust/core 的公历↔儒略日转换（Meeus/Fliegel-Van Flandern
  算法，year=0 视为普通闰年）一致——这是本仓库 Rust 代码实际在用的纪年惯例。fang_shiming 转录数据用的是
  **历史纪年**（无公元0年，本书印刷体例），合并时统一换算成天文纪年。
- is_primary=1 的行来自 vendor/年号表.csv（ytliu0/ChineseCalendar 提取），是原有 EraNames 行为的等价延续
  （每年一条"正统主线"记录，保证从旧的单一年号表迁移过来的调用方不受影响）；南北朝/五代按年份区间把笼统的
  "南北朝"/"五代" regime_simp 换算成具体的"宋/齐/梁/陈"、"后梁/后唐/后晋/后汉/后周"，与并存的其他政权列同一命名体系。
- is_primary=0 的行来自 data/calendar/fang_shiming/ 各章节转录，补充"正统主线"之外并存的政权（蜀、吴、
  十六国、北朝各代、十国、辽金西夏蒙古等），multiple rows per year 天然支持多政权并存，不强行二选一。
- `gaiyuan_month` 列：改元/继位所在的农历月份（1-12），仅在该年是某政权某年号的**第一年**时有值。来自
  fang_shiming 转录里的圈码标注（①-⑫，已确认全书体例是月份而非脚注编号）；vendor/年号表.csv 本身不含月份
  信息，所以 `is_primary=1` 的行此列恒为空。这只是"哪个月改元"，不是精确到日——精确到日的改元时间见
  `rust/core/src/calendar/era_gaiyuan.rs` + `data/calendar/era_gaiyuan_day.csv`（另一套机制，由
  fill_era_gaiyuan_day.py 据逐条查证的 data/calendar/era_gaiyuan_sources.csv 填入）。
- 已知的真实史料分歧（见 data/calendar/汇总/校勘调研_年号表_vs_方诗铭.md）不做取舍，两种说法都保留：
  * 145 年冲帝年号"永嘉"（年号表.csv/后汉书等主流）与"永熹"（东观汉记），以 note 字段记录异说；
  * 1402 年"洪武三十五年"（明成祖靖难后官方记录）与"建文四年"（建文帝在位时实际年号），以 note 字段记录异说；
  * 551 年"大宝二年"（梁简文帝正统）与"天正元年"（豫章王萧栋，侯景之乱期间的并存称帝），
    这不是同一政权的异说而是两个并存政权，用两行分别表示（is_primary=1 的梁 + is_primary=0 的"梁(豫章王)"）。
- 年号表.csv 上游遗漏（非史料分歧，而是确认的漏记）用 MANUAL_PRIMARY_FIXES 覆盖主线行并在 note 记原值与依据：
  * 697 年九月改元神功（年号表.csv 原记"万岁通天二年"、未标改元）。
- 年号表.csv 起于前106年；此前（前841 共和元年起）的主线与并存诸侯取 fang_shiming 01-04 章，正统按《资治通鉴》
  体例 周→秦（前255起）→汉（前206起），见 main() 内 EARLY / early_primary_regime()。
- 近现代政权（中华民国、中华人民共和国）无 vendor/年号表.csv 或 fang_shiming 来源可依，源码内直接按史实
  生成（source=manual_近现代补录），覆盖到 lunar_year 2200（与 HISTORICAL_LUNAR_YEAR_MAX 一致）：中华民国
  1912 年起主线，1949 年中华人民共和国成立后中华民国主线让位、政府迁台但纪年延续记录（is_primary=0）；
  两次政权交接所在农历年（1911、1949）均以并存行表示，不改动交接前既有主线行，见脚本内该段注释。

用法（工作目录为仓库根）：
  uv run --project scripts python scripts/calendar_data/build_era_names_merged.py
"""
from __future__ import annotations

import csv
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FS_DIR = ROOT / "data/calendar/fang_shiming"
OUT_DIR = ROOT / "data/calendar/汇总"

CN_DIGITS = {"零": 0, "一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}


def cn_year_to_int(s: str) -> int | None:
    s = s.strip()
    if not s:
        return None
    if s in ("元年", "元载"):
        return 1
    s = s.rstrip("年载")
    if not s:
        return None
    if s == "十":
        return 10
    if "十" in s:
        parts = s.split("十")
        tens = CN_DIGITS.get(parts[0], 1) if parts[0] else 1
        ones = CN_DIGITS.get(parts[1], 0) if len(parts) > 1 and parts[1] else 0
        return tens * 10 + ones
    if s in CN_DIGITS:
        return CN_DIGITS[s]
    try:
        return int(s)
    except ValueError:
        return None


def hist_to_astro(hist_year: int) -> int:
    """fang_shiming 历史纪年（无公元0年）-> 天文纪年（含公元0年）。公元后不变，公元前 +1（前1年=0年）。

    锚点核对：fang 04_汉纪年表 历史纪年 -110「元封 1」（元封元年=前110年）；年号表.csv 天文纪年 -105 为元封五年，
    即元封元年=天文 -109 = -110 + 1。（此前写成 -1，因合并的 fang 数据此前全是公元后，未暴露。）
    """
    return hist_year if hist_year >= 1 else hist_year + 1


# ①-⑫（U+2460-U+246B，此前范围写成 0x246A 漏了 ⑪⑫）与 ㊀-㊉（U+3280-3289，部分章节用这套圈码汉字数字）。
# fang_shiming 全书体例：这些圈码标注的是继位/改元所在的农历月份（1-12），不是脚注编号——已在多章节
# 转录中与史实交叉核对确认，见 data/calendar/fang_shiming/README.md「三、转录约定」。
_CIRCLE_ARABIC = {chr(0x2460 + i): i + 1 for i in range(12)}  # ①..⑫ -> 1..12
_CIRCLE_CJK = {c: i + 1 for i, c in enumerate("㊀㊁㊂㊃㊄㊅㊆㊇㊈㊉")}  # 一..十 -> 1..10
_ALL_CIRCLES = {**_CIRCLE_ARABIC, **_CIRCLE_CJK}
CIRCLE_RE = re.compile("[" + re.escape("".join(_ALL_CIRCLES)) + "]")


def strip_circles(s: str) -> str:
    return CIRCLE_RE.sub("", s)


def extract_month(seg: str) -> int | None:
    """从原书单元格片段里提取继位/改元月份圈码（同一段可能出现多个圈码，取第一个——按体例通常紧跟在
    年号/帝号之后，即改元当月；"闰"字紧邻圈码表示闰月，本函数不单独标记闰月，只返回月数字）。
    """
    for ch in seg:
        if ch in _ALL_CIRCLES:
            return _ALL_CIRCLES[ch]
    return None


# ---------- fang_shiming 单元格结构化解析 ----------
# 原书单元格把"君主（帝号/人名）""年号""年数""月份圈码"挤在一段文字里，且奉中原正朔、无自有年号的政权（吴、
# 荆南、吴越、南唐等）把所奉年号写在括号里，如「高保融③(乾祐) 1」「~从诲(大和)1 / (天成⑦)4」。这里拆成
# 君主/年号/年数/月份四个字段，判断"哪段文字是年号"以方诗铭自己的《年号索引》（16_年号索引.csv）为字典。

# 16_年号索引 未收、但原书正文确作年号使用的个例。
ERA_INDEX_EXTRA = {
    "赞普钟",  # 南诏阁逻凤年号（752 年起），09_唐纪年表「南诏」栏
    # 以下严格说不是年号（索引不收），但通行纪年写法即「共和元年」「文帝后元元年」，年号表.csv 武帝亦作「后元」，
    # 按纪年名称处理，放 era_name：
    "共和",  # 前 841—前 828 周共和行政，01_十二诸侯纪年表「周」栏
    "中元", "后元",  # 汉文帝后元、景帝中元/后元（04 章「汉」栏），魏惠王后元（02 章「魏」栏）
    "更元",  # 秦惠文王更元（前 324 年起），02_战国纪年表「秦」栏
}


def load_era_index() -> set[str]:
    with open(FS_DIR / "16_年号索引.csv", encoding="utf-8") as f:
        return {r["era_name"].strip() for r in csv.DictReader(f) if r["era_name"].strip()} | ERA_INDEX_EXTRA


ERA_INDEX = load_era_index()

# 任意带圈数字（①-⑳ U+2460-2473、㉑-㉟ U+3251-325F、㊱-㊿ U+32B1-32BF、㊀-㊉ U+3280-3289）与 10 章转录用的
# 方括号月份「[11]」（notes_10「带圈数字」第 1 条：⑪ 以上写作 [n]）。只有 1-12 当月份，⑬/㉚/[13] 等无法
# 确定含义（notes 中多为"十一/十二"压缩体存疑），不当月份、只剥掉。
_ANY_CIRCLE = "①-⑳㉑-㉟㊱-㊿㊀-㊉"
_MONTH_MARK_RE = re.compile(f"([{_ANY_CIRCLE}])|\\[(\\d+)\\]")
_CIRCLE_VALUE = {
    **{chr(0x2460 + i): i + 1 for i in range(20)},
    **{chr(0x3251 + i): i + 21 for i in range(15)},
    **{chr(0x32B1 + i): i + 36 for i in range(15)},
    **_CIRCLE_CJK,
}
_PAREN_RE = re.compile(r"[（(]([^（）()]*)[）)]")
_TILDES = "~〜～"
_UNCERTAIN = "?？□"
# 原书「~」= 同上，替代与上一位君主相同的姓氏（如吴「~渥」= 杨渥、渤海「15～玄锡」= 大玄锡）。
COMPOUND_SURNAMES = (
    "耶律", "慕容", "拓跋", "宇文", "赫连", "秃发", "沮渠", "乞伏", "完颜", "万俟", "司马", "诸葛",
    "独孤", "长孙", "尉迟", "公孙", "夏侯", "孛儿只斤", "嵬名",
)
# 公侯伯敖：01 章诸侯谥号「真公」「靖侯」「堵敖」；世：「二世」
_TITLE_END_RE = re.compile(r"(帝|宗|祖|王|主|后|可汗|公|侯|伯|敖|世)$")
# 03_秦纪年表 括号里用汉字写月份：「**1(十)」= 十月，「3(~子婴八)」= 子婴八月立
_CN_MONTH = {n: i + 1 for i, n in enumerate("一二三四五六七八九十")} | {"十一": 11, "十二": 12}
_CN_MONTH_TAIL_RE = re.compile(r"(闰?)(十[一二]|[一二三四五六七八九十])$")
# 南诏栏内先后几个政权的国号，原书直接冠在君主名前（「大长和郑买嗣安国」「大封民隆舜贞明」）。拆到 note 里，
# 否则既污染君主名，又会让「~仁旻」按"大"姓展开（应为郑仁旻）。
GUOHAO_PREFIXES = ("大长和", "大天兴", "大义宁", "大封民")


def first_month(text: str) -> int | None:
    for m in _MONTH_MARK_RE.finditer(text):
        v = _CIRCLE_VALUE.get(m.group(1)) if m.group(1) else int(m.group(2))
        return v if v is not None and 1 <= v <= 12 else None
    return None


def strip_marks(text: str, keep_brackets: bool = False) -> str:
    """剥掉月份圈码/方括号月份、脚注星号「*」「※」；默认也剥掉〔〕（原书农民起义政权/推定年号标志，见 notes_06）。"""
    text = _MONTH_MARK_RE.sub("", text).replace("*", "").replace("※", "")
    return text if keep_brackets else text.replace("〔", "").replace("〕", "")


def paren_era(inner: str) -> str | None:
    """括号内容若是年号则返回年号。「明〔显〕道」= 明道（元昊避讳改称显道），〔〕内是异写，去掉后再查。"""
    for cand in (re.sub(r"〔[^〕]*〕", "", inner), inner.replace("〔", "").replace("〕", "")):
        cand = cand.strip(" 　").lstrip(_TILDES)
        if era_key(cand) in ERA_INDEX:
            return cand
    return None


def era_key(text: str) -> str:
    return text.strip(" 　·" + _UNCERTAIN)


def split_ruler_era(text: str) -> tuple[str, str]:
    """「孝静帝元象」->("孝静帝","元象")，「沮渠蒙逊　永安」->("沮渠蒙逊","永安")，「高保融」->("高保融","")。

    年号取能在年号索引里查到的**最长**后缀（至少 2 字，单字后缀太容易误配人名末字）；年号保留原书的「?」
    存疑标记，去掉「□」（原书印刷方框，年数不详，见 README「三、转录约定」）。
    """
    t = text.strip(" 　·")
    if not t:
        return "", ""
    if era_key(t) in ERA_INDEX:
        return "", t.replace("□", "")
    for i in range(1, len(t) - 1):
        era = t[i:]
        if era[0] in " 　·" + _UNCERTAIN:
            continue  # 「段思平? 文德」「鞠□义和」：?/□ 是前面人名的存疑标记，归君主
        if len(era_key(era)) >= 2 and era_key(era) in ERA_INDEX:
            ruler = t[:i].strip(" 　·")
            if ruler and ruler not in _TILDES:
                return ruler, era.strip(" 　·").replace("□", "")
    return t, ""


def surname_of(ruler: str | None) -> str | None:
    """君主名的姓氏（「太祖(朱温)」取括号里的本名）；只有帝号、或本身还是未展开的「~X」时返回 None。"""
    if not ruler:
        return None
    m = re.search(r"\(([^()]*)\)", ruler)
    raw = m.group(1) if m else ruler
    person = raw.strip(_UNCERTAIN)
    if not person or person[0] in _TILDES or _TITLE_END_RE.search(person):
        return None
    for cs in COMPOUND_SURNAMES:
        if person.startswith(cs):
            return cs
    # 单字一般是只记了名、没记姓（如前燕「儁」），不当姓；但「鞠□」= 姓鞠、名缺（高昌，见 notes_16），姓是确定的
    return person[0] if len(person) >= 2 or "□" in raw else None


def expand_tilde(name: str, surname: str | None) -> str:
    """「~渥」+ 本栏最近已知姓氏「杨」-> 「杨渥」；推不出姓氏时原样保留「~」。
    「安禄山~庆绪」这类一段内先后两位君主，取后一位（年末状态）。"""
    if not name:
        return name
    if name[0] not in _TILDES and any(t in name for t in _TILDES):
        parts = re.split(f"[{_TILDES}]", name)
        return expand_tilde("~" + parts[-1], surname_of(parts[0]) or surname)
    if name[0] not in _TILDES:
        return name
    return surname + name[1:] if surname else name


def parse_segment(seg: str, surname: str | None) -> dict:
    """解析单元格里的一段（以 / ／ 换行 分隔），返回 ruler/era/num/month（缺的为 None）。
    `surname` 是本栏最近一位已知君主的姓氏，用于展开原书的「~」。"""
    month = first_month(seg)
    seg = strip_marks(seg, keep_brackets=True).replace("闰", "")
    # 多行单元格残留的前一位君主末年年数，如渤海 793 年「57 大元义 1」、871 年「15～玄锡 1」
    m = re.match(r"^\s*\d+\s*(?=[^\d(（])", seg)
    if m and re.search(r"\d+\s*$", seg[m.end() :]):
        seg = seg[m.end() :]

    feng_era = None  # 括号里的年号 = 奉行的他国年号（奉正朔）
    alias = None  # 紧跟帝号的括号人名 = 该帝号的本名，如「太祖(朱温)」
    successor = None  # 跟在年数后/单独出现的括号人名 = 年内嗣位的新君，如「2(~渥⑩)」
    kept: list[str] = []  # 去掉括号后的主体文字
    pos = 0
    for m in _PAREN_RE.finditer(seg):
        before = seg[: m.start()].rstrip(" 　")
        inner = m.group(1).strip(" 　")
        kept.append(seg[pos : m.start()])
        pos = m.end()
        mt = _CN_MONTH_TAIL_RE.search(inner)
        if mt and len(inner) > len(mt.group(0)) and inner[: mt.start()].lstrip(_TILDES):
            month = month or _CN_MONTH[mt.group(2)]  # 「~子婴八」：人名 + 汉字月份
            inner = inner[: mt.start()]
        elif mt and inner == mt.group(0):
            month = month or _CN_MONTH[mt.group(2)]  # 「(十)」：纯汉字月份
            continue
        inner = re.sub(r"立$", "", inner) if len(inner) > 1 else inner  # 「(二世立)」
        era = paren_era(inner)
        if era:
            feng_era = era
        elif len(inner) == 1 and before and not before[-1].isdigit() and not _TITLE_END_RE.search(before):
            pass  # 单字异写注，如「太(大)安」「寿昌(隆)」，丢弃
        elif before and not before[-1].isdigit() and before[-1] not in ")）":
            alias = inner.replace("〔", "").replace("〕", "")
        else:
            successor = inner.replace("〔", "").replace("〕", "")
    kept.append(seg[pos:])
    main = "".join(kept).replace("〔", "").replace("〕", "")
    num_m = re.search(r"(\d+)\s*$", main)
    num = int(num_m.group(1)) if num_m else None
    text = main[: num_m.start()] if num_m else main
    ruler, era = split_ruler_era(text)
    guohao = next((g for g in GUOHAO_PREFIXES if ruler.startswith(g)), None)
    if guohao:
        ruler = ruler[len(guohao) :].strip(" 　")
    paren_only = bool(feng_era and not era)
    if paren_only:
        era = feng_era
    if ruler and ruler.strip(_UNCERTAIN + _TILDES + " 　") == "":
        ruler = ""
    if ruler:
        ruler = expand_tilde(ruler, surname)
    if alias:
        if ruler:
            ruler = f"{ruler}({expand_tilde(alias, surname_of(ruler) or surname)})"
        else:
            ruler = expand_tilde(alias, surname)  # 「神武□(~思英?)」：括号前是年号，括号里才是君主
    if successor:
        ruler = expand_tilde(successor, surname_of(ruler) or surname)
    return {
        "ruler": ruler or None, "era": era or None, "num": num, "month": month, "guohao": guohao,
        "paren_era": paren_only,
    }


def parse_cell(cell: str, ruler: str | None = None, surname: str | None = None) -> list[dict]:
    """逐段解析；每段解析出的君主立即更新姓氏，供同一单元格后续段落展开「~」。"""
    segs = [x.strip() for x in re.split(r"[/／\n]", cell or "") if x.strip()]
    out = []
    for seg in segs:
        info = parse_segment(seg, surname)
        if info["ruler"]:
            info["ruler"] = keep_alias(info["ruler"], ruler)
            ruler = info["ruler"]
            surname = surname_of(ruler) or surname
        out.append(info)
    return out


def keep_alias(new: str, current: str | None) -> str:
    """同一帝号后续单元格不再重复括号本名（「宣武帝(元恪)」之后又写「宣武帝延昌」），保留带本名的写法。"""
    if current and "(" not in new and current.startswith(new + "("):
        return current
    return new


def load_fs(fname: str) -> dict[int, dict]:
    path = FS_DIR / f"{fname}.csv"
    with open(path, encoding="utf-8") as f:
        rows = list(csv.DictReader(f))
    return {int(r["year"]): r for r in rows}


def column_surname(col: str) -> str | None:
    """以人名命名的栏（「慕容麟(延平)」「赵廞(太平)」：括号里是年号）给出初始姓氏，供栏内「~麟」展开。"""
    m = re.match(r"^([^()（）]+)[(（]([^()（）]+)[)）]$", col)
    if m and m.group(2) in ERA_INDEX:
        return surname_of(m.group(1))
    return None


def forward_fill_column(
    rows_by_year: dict[int, dict], col: str, carry: dict | None = None
) -> dict[int, dict]:
    """返回 year -> {ruler, era, num, month}，取每年**年末状态**（单元格多段时依次生效，最后一段为准）。

    原书留空 = 延续上一行（README「三、转录约定」），君主与年号各自前向填充：新君未改元时沿用旧年号
    （踰年改元惯例），奉正朔政权只换括号年号时沿用君主。
    空白之后若单元格带有君主或年号（"表头型"单元格），先清空状态再取新值：同一列可能被先后两个政权复用
    （如 09_唐纪年表「吴」栏先是 619-621 李子通、后是 902-907 杨行密），不能把前一个政权的名称填过去；
    空白之后只有年数（如南诏 902 年「5(郑买嗣?)」前的 901 年留空）则照常延续——这类空白是原书对不确定
    年份的留空，政权并未中断。
    `month` 只在年数=1（改元/建号首年）时给出。
    `carry`：同一政权在上一章节末年的状态（各章只是同一张连续大表的分页，跨章续接，如 11 章吴越栏首行只印
    「(建隆) 1」，君主要从 10 章末年的钱弘俶接过来）。
    括号年号的判别：通常是奉中原正朔（「(天成)1」）；但若年数恰好**延续**当前年号（上一年 +1），括号年号只是
    原书附注的异说（渤海 774 年「**(宝历) 38」：38 续大兴纪年，脚注称大钦茂晚年仍用大兴），保留当前年号，
    异说记入 note。
    """
    out: dict[int, dict] = {}
    ruler: str | None = None
    era: str | None = None
    surname: str | None = column_surname(col)
    guohao: str | None = None
    num_prev: int | None = None
    if carry:
        ruler, era, surname = carry["ruler"], carry["era"], carry["surname"] or surname
        num_prev = carry["num"]
    after_gap = False
    for y in sorted(rows_by_year):
        cell = rows_by_year[y].get(col, "")
        if cell is None or cell.strip() == "":
            after_gap = True
            continue
        segs = parse_cell(cell, ruler, surname)
        if not segs:
            continue
        if after_gap and (segs[0]["ruler"] or segs[0]["era"]):
            # 姓氏不清：「~」只用于同姓相承，跨空白沿用也不会把另一政权的姓安错（新政权首行必写全名）
            ruler = era = guohao = None
            segs = parse_cell(cell, None, surname)
        note = ""
        # 单元格内年号前后不同（「废帝乾明1 / 孝昭帝皇建1」）= 年内改元：记下改元前的年号与年数
        seg_states: list[tuple[str | None, int | None]] = []
        for info in segs:
            if (
                info["paren_era"] and era and info["era"] != era and not after_gap
                and num_prev is not None and info["num"] == num_prev + 1
            ):
                note = f"原书括注「{info['era']}」（异说，年数仍续{era}）。"
                info["era"] = None
            if info["ruler"] and not info["era"] and info["num"] == 1:
                # 新君、无年号、年数从 1 重新起算 = 在位纪年，不是沿用旧年号（渤海大彝震「咸和」之后诸王原书只印
                # 王名与在位年数，不能把咸和一路填到 907 年）；年数延续的（「~渥 3」）才是新君沿用旧号。
                era = None
            ruler = info["ruler"] or ruler
            era = info["era"] or era
            guohao = info["guohao"] or guohao
            surname = surname_of(ruler) or surname
            if info["num"] is not None:
                num_prev = info["num"]
            seg_states.append((era, info["num"]))
        after_gap = False
        last = segs[-1]
        prev_era = prev_num = None
        for e, n in reversed(seg_states[:-1]):
            if e and e != era:
                prev_era, prev_num = e, n
                break
        # 一年三号以上（528 年北魏「武泰1 / 孝庄帝建义1 / 永安1」）：改元前年号之前更早的年号，依次记下
        earlier = []
        for e, n in seg_states[:-1]:
            if e and e not in (era, prev_era) and (e, n) not in earlier:
                earlier.append((e, n))
        out[y] = {
            "ruler": ruler,
            "era": era,
            "num": last["num"],
            "month": last["month"] if last["num"] == 1 else None,
            "guohao": guohao,
            "note": note,
            "surname": surname,
            "prev_era": prev_era,
            "prev_num": prev_num,
            "earlier": earlier,
        }
    return out


# 年号表.csv 在政权/朝代交接的改元年，era_name_simp 有时会把朝代/帝号也拼进去
# （如"宋元徽"、"魏文帝黄初"、"元文宗天历"、"明光宗泰昌"），只有结尾的 2-4 字才是真正的年号。
_DYNASTY_PREFIXES = sorted(
    [
        "东汉", "西汉", "汉", "新", "曹魏", "蜀汉", "孙吴", "魏", "蜀", "吴", "西晋", "东晋", "晋",
        "南朝宋", "南朝齐", "南朝梁", "南朝陈", "宋", "齐", "梁", "陈", "北魏", "东魏", "西魏", "北齐",
        "北周", "隋", "唐", "武周", "后梁", "后唐", "后晋", "后汉", "后周", "五代", "北宋", "南宋",
        "辽", "西辽", "金", "西夏", "蒙古", "元", "明", "清", "南明",
    ],
    key=len,
    reverse=True,
)
_TITLE_SUFFIX_RE = re.compile(r"^.{1,2}(帝|宗|祖|主)")


def clean_era_name(raw: str) -> str:
    """剥离 era_name_simp 里可能拼进去的朝代名/帝号前缀，只留核心年号文字。

    真正的年号绝大多数是 2 个字（少数 1/3/4 字），而"元封""元祐""明道""唐隆"这类年号本身第一个字
    恰好也在朝代名单里（尤其"元"，既是元朝简称也是极常见的年号起首字）——如果剥掉 1 个字后只剩 1 个字，
    大概率是把真年号的一部分错当成朝代前缀剥掉了，这种情况**不剥**，原样保留；只有剥完后仍余下 >= 2 个字
    时才当作真的剥对了（这个阈值下，"元封"→ 剩"封"(1字) 不剥；"元文宗天历"→ 剥"元"剩"文宗天历"(4字，
    再剥"文宗"剩"天历"(2字)）。
    """
    s = raw
    changed = True
    while changed:
        changed = False
        # 先剥帝号头衔，再剥朝代名：否则"元帝景元""明宗天成"里帝号的首字"元""明"会先被当成元朝/明朝前缀剥掉，
        # 剩下"帝景元""宗天成"，帝号正则就再也匹配不上了。
        m = _TITLE_SUFFIX_RE.match(s)
        if m and len(s) - len(m.group(0)) >= 2:
            s = s[len(m.group(0)) :]
            changed = True
            continue
        for prefix in _DYNASTY_PREFIXES:
            if s.startswith(prefix) and len(s) - len(prefix) >= 2:
                s = s[len(prefix) :]
                changed = True
                break
    return s if s else raw


# 相邻章节交界年份（如 960 年同时是"五代十国纪年表"末年与"宋辽金纪年表"首年）里，同一政权在不同章节
# 用了不同列名（如"契丹(辽)"/"辽"、"荆南(南平)"/"荆南"），统一成后一章节（更晚朝代惯用）的写法，
# 避免同一年同一政权被当成两个不同政权、生成重复行。
REGIME_ALIASES = {
    "契丹(辽)": "辽",
    "荆南(南平)": "荆南",
}


def normalize_regime(name: str) -> str:
    return REGIME_ALIASES.get(name, name)


def south_dynasty_for_year(y: int) -> str:
    if y <= 478:
        return "宋"
    if y <= 501:
        return "齐"
    if y <= 556:
        return "梁"
    return "陈"


def five_dynasties_for_year(y: int) -> str:
    if y <= 923:
        return "后梁"
    if y <= 936:
        return "后唐"
    if y <= 946:
        return "后晋"
    if y <= 950:
        return "后汉"
    return "后周"


# clean_era_name() 只处理"朝代名/帝号头衔"打头的污染；个别改元年年号表.csv 把新君**本名**也拼进
# era_name_simp（如"李从珂清泰"，李从珂是后唐末帝本名），本名不在朝代前缀表里，clean_era_name() 剥不掉，
# 这里按(年份,政权)手工列出已发现的个例。
MANUAL_ERA_NAME_FIXES: dict[tuple[int, str], str] = {
    (934, "后唐"): "清泰",
    (1278, "宋"): "祥兴",  # 年号表.csv 原字段「帝昺祥兴」，帝昺（赵昺）无庙号，帝号正则剥不掉
}

# 年号表.csv 上游数据本身的遗漏（经交叉比对 + 人工查证确认，见 校勘调研_年号表_vs_方诗铭.md「其余」类），
# 按(年份,政权)覆盖主线行的字段，并在 note 里写明原值与依据——这是"以汇总表裁决结果为准"，不改 vendor 原文件。
MANUAL_PRIMARY_FIXES: dict[tuple[int, str], dict] = {
    # 697 年（万岁通天二年）九月改元神功，次年正月又改圣历；年号表.csv 漏记这次年内改元（has_gaiyuan=0）。
    # 方诗铭表 09_唐纪年表「武周」栏作「神功⑨ 1」，16_年号索引「神功：周武则天，697」，与《旧唐书·则天皇后纪》
    # 《资治通鉴》卷二〇六一致。
    # 年号表.csv 原字段 era_name_simp=「肃宗至德元载」、era_year_simp 为空——年数被拼进了年号名里。
    (756, "唐"): {
        "fields": {"era_name": "至德", "era_year": 1},
        "note": "年号表.csv 原字段为「肃宗至德元载」且年数列为空，已拆为年号「至德」、年数 1（元载）。",
    },
    # 年号表.csv 原字段 gaiyuan_prev_era_name_simp=「昺祥兴」（帝昺本名拼进了前年号名）。
    (1279, "宋"): {
        "fields": {"gaiyuan_prev_era_name": "祥兴"},
        "note": "年号表.csv 改元前年号原字段为「昺祥兴」，已剥离帝昺本名。",
    },
    # 以下据 data/calendar/era_gaiyuan_sources.csv 逐条查证的改元日期订正「年初年号」（gaiyuan_prev_*）与改元标记：
    # 年号表.csv 这几年把改元前年号拼进了帝号字段（如「惠帝永安/建武/」）、误把年内第一个新号当年初号，或把只换
    # 皇帝未改元的年份标成改元年。订正后 GaiyuanDays::resolve 在第一个改元日之前才会返回正确的旧号。
    (304, "晋"): {
        "fields": {"emperor": "惠帝", "gaiyuan_prev_era_name": "太安", "gaiyuan_prev_era_year": 3},
        "note": "本年四次改元：正月永安、七月建武、十一月复永安、十二月永兴（见 era_gaiyuan_day.csv）；年号表.csv 原把"
        "「永安/建武/」拼入帝号字段、改元前号缺。",
    },
    (701, "武周"): {
        "fields": {"gaiyuan_prev_era_name": "久视", "gaiyuan_prev_era_year": 2},
        "note": "正月丁丑始改元大足，年初仍为久视二年（《新唐书·则天皇后纪》）；年号表.csv 原记年初号为大足。",
    },
    (705, "武周"): {
        "fields": {"emperor": "中宗", "gaiyuan_prev_era_name": "长安", "gaiyuan_prev_era_year": 5},
        "note": "正月甲辰皇太子监国改元神龙，丙午中宗复位；年号表.csv 原帝号字段为「/唐中宗」、改元前号缺。",
    },
    (712, "唐"): {
        "fields": {"gaiyuan_prev_era_name": "景云", "gaiyuan_prev_era_year": 3},
        "note": "本年三次改元：正月己丑太极、五月辛巳延和、八月甲辰先天；年初为景云三年，年号表.csv 原记年初号为太极。",
    },
    (874, "唐"): {
        "fields": {"emperor": "僖宗", "gaiyuan_prev_era_name": "咸通", "gaiyuan_prev_era_year": 15},
        "note": "十一月庚寅改元乾符；年号表.csv 原把「咸通十五/」拼入帝号字段、改元前号缺。",
    },
    (948, "后汉"): {
        "fields": {"gaiyuan_prev_era_name": "天福", "gaiyuan_prev_era_year": 13},
        "note": "正月乙卯改元乾祐（《新五代史·汉高祖纪》）；年号表.csv 原改元前号缺。",
    },
    (954, "后周"): {
        "fields": {"has_gaiyuan": 0},
        "note": "显德于正月丙子朔改元（《新五代史·周太祖纪》），即年首换号，非年内改元。",
    },
    (959, "后周"): {
        "fields": {"has_gaiyuan": 0},
        "note": "六月世宗崩、恭帝即位，沿用显德，未改元。",
    },
    (1329, "元"): {
        "fields": {"has_gaiyuan": 0},
        "note": "正月明宗即位于和林、八月暴卒，文宗复位，始终沿用天历，未改元。",
    },
    (1332, "元"): {
        "fields": {"has_gaiyuan": 0},
        "note": "八月文宗崩、十月宁宗即位、十一月宁宗崩，沿用至顺，未改元（顺帝次年即位）。",
    },
    (690, "唐"): {
        "fields": {"gaiyuan_prev_era_year": 1},
        "note": "载初只有元年：永昌元年十一月（周正正月）改元载初，次年（夏正）九月即改天授，年号表.csv 原记年初为载初二年。",
    },
    (762, "唐"): {
        "fields": {"gaiyuan_prev_era_name": "", "gaiyuan_prev_era_year": 1},
        "note": "年初为无年号的「元年」：上元二年九月壬寅去上元号只称元年、以建子月为岁首（见 761 年），本年建巳月改元宝应；"
        "年号表.csv 原记年初为上元三年。",
    },
    (697, "武周"): {
        "fields": {
            "era_name": "神功",
            "era_year": 1,
            "has_gaiyuan": 1,
            "gaiyuan_prev_era_name": "万岁通天",
            "gaiyuan_prev_era_year": 2,
        },
        "note": "年号表.csv 原记「万岁通天二年」且未标改元，漏记本年九月改元神功；据方诗铭表（09_唐纪年表「神功⑨ 1」、"
        "16_年号索引）及《旧唐书·则天皇后纪》订正。",
    },
}

# 相邻章节交界年份的冲突裁决（2026-10-25 逐条对照原书扫描页/PDF 高清重渲染核实）：(年份, 政权) ->
# 保留哪个章节的记录、note 写明核实结论（`fields` 可覆盖字段；结构化解析已能正确拆出君主/年号，目前都不需要）。未列在这里的交界冲突仍两条都保留并标"需人工核对"。
# 多数是同一事实的两种写法（帝号/人名是否拼进年号文字、前向填充带出的旧名）；960 荆南/南唐/吴越是
# 同一年改元/嗣位前后两个状态，取年末状态（与 forward_fill_column 取单元格最后一段的惯例一致）。
BOUNDARY_RESOLUTIONS: dict[tuple[int, str], dict] = {
    (420, "北燕"): {
        "keep": "fang_shiming:07_南北朝纪年表",
        "fields": {},
        "note": "章节交界年份：06/07 两章均为冯跋太平十二年（06 章前向填充带出帝名「冯跋　太平」），已核实一致。",
    },
    (907, "吴"): {
        "keep": "fang_shiming:10_五代十国纪年表",
        "fields": {},
        "note": "章节交界年份：09/10 两章均为杨渥沿用唐天祐四年（09 章栏内只有年数「4」）；吴奉唐正朔，无自有年号。",
    },
    (907, "渤海"): {
        "keep": "fang_shiming:10_五代十国纪年表",
        "fields": {},
        "note": "章节交界年份：09 章「~諲譔 1」与 10 章「大諲譔 1」同为渤海末王大諲譔元年（page_106 高清重渲染确认，"
        "10 章原转录「大谨谟」系误读，已订正）。",
    },
    (960, "南汉"): {
        "keep": "fang_shiming:11_宋辽金纪年表",
        "fields": {},
        "note": "章节交界年份：10/11 两章均为刘鋹大宝三年，已核实一致。",
    },
    (960, "大理"): {
        "keep": "fang_shiming:11_宋辽金纪年表",
        "fields": {},
        "note": "章节交界年份：10/11 两章均为段思聪「广德?」，原书年数不确定、留问号，已核实一致。",
    },
    (960, "荆南"): {
        "keep": "fang_shiming:11_宋辽金纪年表",
        "fields": {},
        "note": "章节交界年份：10 章记「高保融（显德）7」、11 章记「高保勗⑧（建隆）1」，并非矛盾——荆南奉中原正朔，"
        "本年正月宋改元建隆，八月高保融卒、弟高保勗嗣（《宋史·荆南高氏世家》），两章分别记年初/年末状态，"
        "本行取年末状态（page_109/page_113 核实；11 章月份圈码原转录作①，已订正为⑧）。",
    },
    # 南唐、吴越同荆南：奉中原正朔，10 章记后周显德七年、11 章记宋建隆元年，取年末状态。此前旧解析器丢弃了 11 章
    # 只印「(建隆) 1」的单元格，这两个政权 961-978 年整段缺失，改用结构化解析 + 跨章续接后才补上。
    (960, "南唐"): {
        "keep": "fang_shiming:11_宋辽金纪年表",
        "fields": {},
        "note": "章节交界年份：10 章记「(显德) 7」、11 章记「(建隆) 1」——南唐奉中原正朔，本年正月宋改元建隆，"
        "两章分别记年初/年末状态，本行取年末状态（李璟在位）。",
    },
    (960, "吴越"): {
        "keep": "fang_shiming:11_宋辽金纪年表",
        "fields": {},
        "note": "章节交界年份：10 章记「(显德) 7」、11 章记「(建隆) 1」——吴越奉中原正朔，本年正月宋改元建隆，"
        "两章分别记年初/年末状态，本行取年末状态（钱弘俶在位）。",
    },
    (960, "辽"): {
        "keep": "fang_shiming:11_宋辽金纪年表",
        "fields": {},
        "note": "章节交界年份：10/11 两章均为辽穆宗应历十年，已核实一致。",
    },
}

# 已知真实史料分歧的注记（年份, 政权）-> note 追加文字
KNOWN_VARIANT_NOTES: dict[tuple[int, str], str] = {
    (145, "汉"): "年号另有异说作「永熹」（刘珍《东观汉记》），清何焯据石刻考证作「永憙」；本行取后汉书等主流写法「永嘉」。",
    (1402, "明"): "本年官方记录（明成祖靖难后追认）为「洪武三十五年」；建文帝在位时实际使用的年号「建文四年」见"
    "本表 is_primary=0 的「明(建文)」行。",
}

# 近现代政权无年号，用"领导人"（国家元首/最高领导人）代替年号信息（emperor 字段），彼此按 (公历年,月,日) 排序，
# 取某天生效时"最后一位上任日期 <= 该天"的人。只收录被公认为实质国家元首/最高领导人者，短暂看守/代行职务者
# （如北洋政府时期数位"国务院摄行"、张勋复辟期间等）不单独列，避免细枝末节的史实分歧混进来。
# 来源：维基百科「中华民国国家元首列表」「中华人民共和国最高领导人」，2026-09 查核，见脚本内 WebFetch 记录。
ROC_LEADERS: list[tuple[tuple[int, int, int], str]] = [
    ((1912, 1, 1), "孙中山"),
    ((1912, 3, 10), "袁世凯"),
    ((1916, 6, 7), "黎元洪"),
    ((1917, 7, 6), "冯国璋"),
    ((1918, 10, 10), "徐世昌"),
    ((1922, 6, 11), "黎元洪"),
    ((1923, 10, 10), "曹锟"),
    ((1924, 11, 24), "段祺瑞"),
    ((1927, 6, 18), "张作霖"),
    ((1928, 2, 7), "谭延闿"),
    ((1928, 10, 10), "蒋中正"),
    ((1931, 12, 15), "林森"),
    ((1943, 8, 1), "蒋中正"),
    ((1949, 1, 21), "李宗仁"),
    ((1950, 3, 1), "蒋中正"),
    ((1975, 4, 6), "严家淦"),
    ((1978, 5, 20), "蒋经国"),
    ((1988, 1, 13), "李登辉"),
    ((2000, 5, 20), "陈水扁"),
    ((2008, 5, 20), "马英九"),
    ((2016, 5, 20), "蔡英文"),
    ((2024, 5, 20), "赖清德"),
]
PRC_LEADERS: list[tuple[tuple[int, int, int], str]] = [
    ((1949, 10, 1), "毛泽东"),
    ((1976, 10, 7), "华国锋"),
    ((1978, 12, 22), "邓小平"),
    ((1989, 11, 9), "江泽民"),
    ((2002, 11, 15), "胡锦涛"),
    ((2012, 11, 15), "习近平"),
]


def leader_at(timeline: list[tuple[tuple[int, int, int], str]], date: tuple[int, int, int]) -> str:
    """timeline 已按日期升序排列；返回 date 当天在任者（date 早于第一任起始日期则返回空串）。"""
    result = ""
    for start, name in timeline:
        if start <= date:
            result = name
        else:
            break
    return result


def leaders_between(
    timeline: list[tuple[tuple[int, int, int], str]], start_incl: tuple[int, int, int], end_excl: tuple[int, int, int]
) -> list[tuple[tuple[int, int, int], str]]:
    """[start_incl, end_excl) 区间内新上任的（日期,姓名），用于给跨越了换届的农历年加注。"""
    return [(d, n) for d, n in timeline if start_incl <= d < end_excl]


def load_lunar_new_year_dates() -> dict[int, tuple[int, int, int]]:
    """lunar_year -> 该年农历正月初一对应的儒略历/格里高利历 (y, m, d)，来自实历朔日表.csv（本仓库已有的
    实历数据，不是另行推算），用于把公历纪年的近现代政权换届日期换算到本表的农历年行上。
    """
    path = ROOT / "vendor/calendar/chinese_calendar/实历朔日表.csv"
    out: dict[int, tuple[int, int, int]] = {}
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("gregorian_year") or line.startswith("western_year"):
                continue
            parts = line.split(",")
            gy, gm, gd, ly, lm, leap = (int(x) for x in parts[:6])
            if lm == 1 and leap == 0:
                out[ly] = (gy, gm, gd)
    return out


GAIYUAN_SOURCES = ROOT / "data/calendar/era_gaiyuan_sources.csv"


def mark_inyear_gaiyuan(rows: list[dict]) -> None:
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    import gaiyuan_date as gd

    first_event: dict[int, dict] = {}
    with open(GAIYUAN_SOURCES, encoding="utf-8") as f:
        for s in csv.DictReader(f):
            first_event.setdefault(int(s["lunar_year"]), s)  # 同年按文件先后，第一条即年内第一个改元
    for r in rows:
        if r["is_primary"] != 1 or r["has_gaiyuan"] == 1:
            continue
        s = first_event.get(r["lunar_year"])
        if s is None:
            continue
        y, month, leap = r["lunar_year"], int(s["month"]), s["leap"] == "1"
        day = gd.resolve(y, month, leap, s["day"])["lunar_day"] if s["day"] else None
        if (month, leap, day) == (1, False, 1):
            continue  # 正月朔改元 = 年首换号
        prev_year = cn_year_to_int(s["from_year"])
        if not s["from_year"] or prev_year is None:
            sys.exit(f"era_gaiyuan_sources.csv {y} {s['to_era']}: 补标改元需要 from_year（改元前年数）")
        when = f"{'闰' if leap else ''}{month}月" + (f"{day}日" if day else "")
        r["has_gaiyuan"] = 1
        r["gaiyuan_prev_era_name"] = s["from_era"]
        r["gaiyuan_prev_era_year"] = prev_year
        r["note"] = (r["note"] + " " if r["note"] else "") + (
            f"{when}始改元{s['to_era']}，年初为{s['from_era']}{s['from_year']}（{s['source']}，见 era_gaiyuan_sources.csv）；"
            "年号表.csv 原未标改元。"
        )


def main() -> None:
    # ---------- 1. 主线：era_names.csv（天文纪年，原样为主，南北朝/五代按年份区间细化政权名）----------
    era_path = ROOT / "vendor/calendar/chinese_calendar/年号表.csv"
    rows: list[dict] = []
    with open(era_path, encoding="utf-8") as f:
        for r in csv.DictReader(f):
            y = int(r["lunar_year"])
            regime = r["regime_simp"]
            if not regime:
                continue
            if regime == "南北朝":
                regime = south_dynasty_for_year(y)
            elif regime == "五代":
                regime = five_dynasties_for_year(y)
            era_year = cn_year_to_int(r["era_year_simp"])
            note = KNOWN_VARIANT_NOTES.get((y, regime), "")
            raw_era_name = r["era_name_simp"]
            if (y, regime) in MANUAL_ERA_NAME_FIXES:
                clean_name = MANUAL_ERA_NAME_FIXES[(y, regime)]
            else:
                clean_name = clean_era_name(raw_era_name)
            if clean_name != raw_era_name and not note:
                note = f"政权交接年，年号表.csv 原字段为「{raw_era_name}」，已剥离朝代/帝号或新君本名前缀。"
            rows.append(
                {
                    "lunar_year": y,
                    "regime": regime,
                    "emperor": r["emperor_simp"],
                    "era_name": clean_name,
                    "era_year": era_year if era_year is not None else "",
                    "is_primary": 1,
                    "has_gaiyuan": int(r["has_gaiyuan"] or 0),
                    "gaiyuan_prev_era_name": clean_era_name(r["gaiyuan_prev_era_name_simp"]),
                    "gaiyuan_prev_era_year": cn_year_to_int(r["gaiyuan_prev_era_year_simp"]) or "",
                    "gaiyuan_month": "",  # 年号表.csv 本身不含月份信息
                    "source": "era_names_csv",
                    "note": note,
                }
            )
            fix = MANUAL_PRIMARY_FIXES.get((y, regime))
            if fix:
                rows[-1].update(fix["fields"])
                rows[-1]["note"] = (note + " " if note else "") + fix["note"]

    # ---------- 1c. 据逐条查证的改元日期补标年内改元 ----------
    # 年号表.csv 的 has_gaiyuan 只标了"一年两号"的年份；正月中旬才改元（如唐显庆正月初七、宋端拱正月）的年份被当成
    # 年首换号。data/calendar/era_gaiyuan_sources.csv 里每年第一个改元事件若不在正月朔，就补标改元并写入年初年号。
    # 正月朔改元（年首换号）的查证结论另记于 era_gaiyuan_yearstart.csv，此处不需处理。
    mark_inyear_gaiyuan(rows)

    # ---------- 2. 补充：fang_shiming 并存政权列（历史纪年 -> 天文纪年）----------
    # (文件, {列名: 输出政权名（默认用列名本身）})；列出的都是该文件里"正统主线"以外要补充的列。
    SUPPLEMENT: list[tuple[str, list[str]]] = [
        ("04_汉纪年表", ["更始", "隗嚣", "公孙述", "刘盆子"]),
        ("05_三国纪年表", ["蜀", "吴", "公孙渊(绍汉)"]),
        (
            "06_晋及十六国纪年表",
            [
                "赵廞(太平)", "司马伦(建始)", "成汉", "刘尼(神凤)", "汉(前赵)", "前凉", "后赵",
                "司马保(建康)", "句渠知(平赵)", "前燕", "代", "冉魏", "前秦", "李弘(凤凰)", "北魏",
                "后燕", "北凉", "西凉", "北燕", "西燕", "西秦", "后凉", "南凉", "夏",
                "白亚栗斯刘虎(建平)", "后秦", "南燕", "慕容详(建始)", "慕容麟(延平)", "兰汗(青龙)", "桓玄(楚)",
            ],
        ),
        (
            "07_南北朝纪年表",
            [
                "北魏", "北凉", "北燕", "西秦", "夏", "仇池(原表未印栏名，据史实推定)", "西凉(420-421)",
                "程道养(原表未印栏名)", "刘劭(原表未印栏名，宋内乱)", "柔然", "唐寓之(原表未印栏名，钱唐乱)",
                "高昌", "破六韩拔陵(原表未印栏名)", "鲜于修礼(原表未印栏名)", "邢杲(原表未印栏名)",
                "萧宝寅(原表未印栏名)", "元颢(原表未印栏名)", "刘蠡升(原表未印栏名，稽胡)", "雍道晞(原表未印栏名)",
                "吕苟儿(原表未印栏名)", "陈瞙(原表未印栏名)", "莫折念生(原表未印栏名)", "杜洛周(原表未印栏名)",
                "葛荣(原表未印栏名)", "万俟丑奴(原表未印栏名)", "西魏", "东魏", "北齐", "北周", "后梁",
                "鲜于琛(原表未印栏名)", "刘敬躬(原表未印栏名)", "萧正德(原表未印栏名)", "侯景(原表未印栏名)",
                "隋",  # 581—589 隋与陈并立（589 年后入主线）
            ],
        ),
        (
            "09_唐纪年表",
            [
                "隋", "楚", "刘武周", "宋", "梁（梁师都）", "梁（萧铣）", "夏", "郑", "燕", "朱粲",
                "沈法兴", "许", "吴", "高昌", "渤海", "南诏", "吐蕃", "齐",
            ],
        ),
        (
            "10_五代十国纪年表",
            [
                "吴", "南唐", "吴越", "前蜀", "后蜀", "闽", "北汉", "南汉", "荆南(南平)", "燕", "楚",
                "契丹(辽)", "南诏", "大理", "渤海", "东丹", "张遇贤", "于阗",
            ],
        ),
        (
            "11_宋辽金纪年表",
            ["南唐", "吴越", "后蜀", "北汉", "南汉", "荆南", "辽", "于阗", "大理", "西夏", "金", "北辽", "西辽", "齐", "蒙古"],
        ),
        ("14_清纪年表", ["南"]),
    ]

    # 551/553 年真实并存政权（侯景之乱期间豫章王萧栋、武陵王萧纪年号），07 文件本身没有单独列出这两位
    # （原书"梁"栏本身就是正统主线，这两位是同一"梁"栏内以"/"分隔的并列文字段——见 notes_07
    # "552-556年梁末诸王并立"），直接从"梁"列原始单元格里用正则摘出，不经过 forward_fill（那只保留最后一段）。
    LIANG_PRETENDER_RE = re.compile(r"(豫章王|武陵王)\(~?([^)]*)\)([^\d／/]*?)(\d+)")

    supplement_count = 0
    # 年号表.csv 起于前106年（天文 -105），此前的主线与并存诸侯取方诗铭 01-04 章。正统按《资治通鉴》体例：
    # 周（共和元年前841起，至前256年周赧王卒）→ 秦（前255年秦昭襄王五十二年起）→ 汉（前206年汉高祖元年起）。
    # 01/02 章其余诸侯列、03/04 章与主线并存的行一律 is_primary=0。04 章「汉」栏只取年号表.csv 覆盖范围之前的年份，
    # 之后的主线仍以年号表.csv 为准（不重复生成主线）。
    era_csv_min = min(r["lunar_year"] for r in rows)
    EARLY: list[tuple[str, list[str]]] = [
        ("01_十二诸侯纪年表", ["周", "鲁", "齐", "晋", "秦", "楚", "宋", "卫", "陈", "蔡", "曹", "郑", "燕", "吴"]),
        ("02_战国纪年表", ["周", "秦", "魏", "韩", "赵", "楚", "燕", "齐", "田齐", "晋"]),
        ("03_秦纪年表", ["秦"]),
        ("04_汉纪年表", ["汉"]),
    ]
    early_files = {f for f, _ in EARLY}

    def early_primary_regime(y_astro: int) -> str:
        if y_astro <= hist_to_astro(-256):
            return "周"
        if y_astro <= hist_to_astro(-207):
            return "秦"
        return "汉"

    EARLY_TRANSITION_NOTES = {
        (hist_to_astro(-841), "周"): "主线起点：周共和元年（方诗铭表起始年，亦即中国历史确切纪年之始）。前106年以前的主线据方诗铭表，"
        "正统取《资治通鉴》体例（周→秦→汉）。",
        (hist_to_astro(-255), "秦"): "周赧王卒于前256年，主线自本年起取秦（秦昭襄王五十二年，《资治通鉴》体例）。",
        (hist_to_astro(-206), "汉"): "主线自本年起取汉（汉高祖元年，《资治通鉴》体例）；同年秦王子婴见 is_primary=0 的「秦」行。",
    }

    # 同一政权跨章续接：记下每个政权在已处理章节里最后一年的状态（EARLY、SUPPLEMENT 均按章节先后排列）
    last_state: dict[str, tuple[int, dict]] = {}
    for fname, cols in EARLY + SUPPLEMENT:
        fs_rows = load_fs(fname)
        first_year = min(fs_rows)
        for col in cols:
            prev = last_state.get(normalize_regime(col))
            carry = prev[1] if prev and prev[0] >= first_year - 1 else None
            ff = forward_fill_column(fs_rows, col, carry)
            if ff:
                y_last = max(ff)
                last_state[normalize_regime(col)] = (y_last, ff[y_last])
            for y_hist, st in ff.items():
                if st["ruler"] is None and st["era"] is None:
                    continue
                y_astro = hist_to_astro(y_hist)
                regime = normalize_regime(col)
                is_early = fname in early_files
                if is_early and fname == "04_汉纪年表" and y_astro >= era_csv_min:
                    continue
                primary = is_early and y_astro < era_csv_min and regime == early_primary_regime(y_astro)
                num, month = st["num"], st["month"]
                note = (f"原书本栏国号「{st['guohao']}」。" if st["guohao"] else "") + st["note"]
                if primary and (y_astro, regime) in EARLY_TRANSITION_NOTES:
                    note += EARLY_TRANSITION_NOTES[(y_astro, regime)]
                rows.append(
                    {
                        "lunar_year": y_astro,
                        "regime": regime,
                        "emperor": st["ruler"] or "",
                        "era_name": st["era"] or "",
                        "era_year": num if num is not None else "",
                        "is_primary": 1 if primary else 0,
                        # 先秦/汉初无年号，嗣位按踰年改元记在位年数，不存在年内前后两号并存的改元年
                        "has_gaiyuan": 0 if primary else (1 if st.get("prev_era") else ""),
                        "gaiyuan_prev_era_name": st.get("prev_era") or "" if not primary else "",
                        "gaiyuan_prev_era_year": (st.get("prev_num") or "") if not primary and st.get("prev_era") else "",
                        "gaiyuan_month": month if month is not None else "",
                        "gaiyuan_earlier_eras": "" if primary else "/".join(f"{e}{n or ''}" for e, n in st.get("earlier", [])),
                        "source": f"fang_shiming:{fname}",
                        "note": note,
                    }
                )
                supplement_count += 1

    # 07_南北朝纪年表"梁"列里的藩王并存年号（见上说明）：直接在原始单元格文本里正则找豫章王/武陵王的段落，
    # 不经过只取"最后一段"的 forward_fill，避免漏掉与正统并列的段落。
    liang_rows = load_fs("07_南北朝纪年表")
    for y_hist, row in liang_rows.items():
        cell = row.get("梁", "") or ""
        for title, name, era, num in LIANG_PRETENDER_RE.findall(strip_circles(cell)):
            era_clean = era.strip(" ~　")
            rows.append(
                {
                    "lunar_year": hist_to_astro(y_hist),
                    "regime": f"梁({title})",
                    "emperor": f"{title}({name})",
                    "era_name": era_clean if era_clean else "天正",
                    "era_year": int(num),
                    "is_primary": 0,
                    "has_gaiyuan": "",
                    "gaiyuan_prev_era_name": "",
                    "gaiyuan_prev_era_year": "",
                    "gaiyuan_month": "",
                    "source": "fang_shiming:07_南北朝纪年表(梁栏内并存年号)",
                    "note": "侯景之乱期间与梁简文帝/梁元帝并存称帝者，见 notes_07.md「552-556年梁末诸王并立」。",
                }
            )

    # ---------- 3. 近现代政权：中华民国、中华人民共和国（无年号表.csv/fang_shiming 来源可依，按史实直接生成）----------
    # 中华民国 1912-01-01（农历辛亥年十一月十二）成立于南京，采用公历纪年（民国 N 年 = 公历 1911+N 年），
    # 与本表其余各行按农历年分一行、年号数按该历法自身历年计数的惯例不完全一致——此处取该农历年对应的
    # 公历起始年份算民国纪年数，是近似（同表内"改元只记月不记日"是同一精度取舍，不做逐日处理）。
    # 清帝溥仪 1912-02-12（农历辛亥年十二月廿五）退位，退位前中华民国已实际成立，故农历辛亥年（1911）内
    # 二者并存：主线仍记清（原有 is_primary=1 行不动），中华民国记一条 is_primary=0 并存行。
    # 中华人民共和国 1949-10-01（农历己丑年八月初十）成立，定都北京：成立当年（农历 1949 年）内主线仍记
    # 中华民国至年末，中华人民共和国记并存行；次年（1950）起主线互换。中华民国政府迁台后仅作纪年延续记录
    # （台湾地区民国纪年沿用至今），不代表对治权归属的认定——这是本仓库首次记录近现代政权，无先例可循，
    # 上述取舍原则见 doc/calendar/calendar-layers.md 第 3 层（派生层）。
    # emperor 字段：近现代政权无年号，改放"国家元首/最高领导人"姓名（见 ROC_LEADERS/PRC_LEADERS），
    # 取该农历年正月初一（实历朔日表.csv 查得，非推算）当天在任者；若该农历年内另有换届，在 note 里补记
    # 换届的具体公历/农历日期，emperor 字段本身仍只放正月初一时在任的那位，与全表"哪个月改元"精度一致。
    MODERN_LUNAR_YEAR_MAX = 2200  # 与 HISTORICAL_LUNAR_YEAR_MAX（rust/core/src/calendar/historical.rs）一致
    ROC_FOUND_LUNAR_YEAR = 1911
    ROC_PRIMARY_START = 1912
    PRC_FOUND_LUNAR_YEAR = 1949  # 公历 1949-10-01，农历己丑年八月初十
    cny = load_lunar_new_year_dates()

    rows.append(
        {
            "lunar_year": ROC_FOUND_LUNAR_YEAR,
            "regime": "中华民国",
            "emperor": "孙中山",
            "era_name": "民国",
            "era_year": 1,
            "is_primary": 0,
            "has_gaiyuan": 0,
            "gaiyuan_prev_era_name": "",
            "gaiyuan_prev_era_year": "",
            "gaiyuan_month": "",
            "source": "manual_近现代补录",
            "note": "中华民国临时政府 1912-01-01（农历辛亥年十一月十二）成立于南京，孙中山就任临时大总统；"
            "清帝溥仪 1912-02-12（农历辛亥年十二月廿五）颁布退位诏书。本年内二者并存，主线仍记清"
            "（见本年 is_primary=1 的「清」行）。",
        }
    )
    for y in range(ROC_PRIMARY_START, MODERN_LUNAR_YEAR_MAX + 1):
        notes: list[str] = []
        if y == ROC_PRIMARY_START:
            notes.append(
                "民国纪年以公历年为界（民国 N 年 = 公历 1911+N 年），与本表其余各行按农历年分行、"
                "年号年数按该历法历年计数的惯例不完全一致，此处取该农历年对应公历起始年份的民国纪年数，"
                "是近似，不做逐日精度处理。"
            )
        elif y == PRC_FOUND_LUNAR_YEAR:
            notes.append("本年中华人民共和国成立（见本年 is_primary=0 的「中华人民共和国」并存行），主线仍记中华民国至本年年末。")
        elif y == PRC_FOUND_LUNAR_YEAR + 1:
            notes.append(
                "中华人民共和国成立后，中华民国政府迁台，此后仅作纪年延续记录"
                "（台湾地区民国纪年沿用至今），不代表对治权归属的认定。"
            )
        cny_this = cny.get(y, (y, 2, 1))
        cny_next = cny.get(y + 1, (y + 1, 2, 1))
        emperor = leader_at(ROC_LEADERS, cny_this)
        for d, name in leaders_between(ROC_LEADERS, cny_this, cny_next):
            notes.append(f"本农历年内换届：{d[0]}-{d[1]:02d}-{d[2]:02d} 起由「{name}」就任。")
        rows.append(
            {
                "lunar_year": y,
                "regime": "中华民国",
                "emperor": emperor,
                "era_name": "民国",
                "era_year": y - 1911,
                "is_primary": 1 if y <= PRC_FOUND_LUNAR_YEAR else 0,
                "has_gaiyuan": 0,
                "gaiyuan_prev_era_name": "",
                "gaiyuan_prev_era_year": "",
                "gaiyuan_month": "",
                "source": "manual_近现代补录",
                "note": " ".join(notes),
            }
        )

    rows.append(
        {
            "lunar_year": PRC_FOUND_LUNAR_YEAR,
            "regime": "中华人民共和国",
            "emperor": "毛泽东",
            "era_name": "",
            "era_year": "",
            "is_primary": 0,
            "has_gaiyuan": 0,
            "gaiyuan_prev_era_name": "",
            "gaiyuan_prev_era_year": "",
            "gaiyuan_month": "",
            "source": "manual_近现代补录",
            "note": "中华人民共和国 1949-10-01（农历己丑年八月初十）成立，定都北京。不设年号，公历纪年。",
        }
    )
    for y in range(PRC_FOUND_LUNAR_YEAR + 1, MODERN_LUNAR_YEAR_MAX + 1):
        cny_this = cny.get(y, (y, 2, 1))
        cny_next = cny.get(y + 1, (y + 1, 2, 1))
        emperor = leader_at(PRC_LEADERS, cny_this)
        notes = [
            f"本农历年内换届：{d[0]}-{d[1]:02d}-{d[2]:02d} 起由「{name}」就任。"
            for d, name in leaders_between(PRC_LEADERS, cny_this, cny_next)
        ]
        rows.append(
            {
                "lunar_year": y,
                "regime": "中华人民共和国",
                "emperor": emperor,
                "era_name": "",
                "era_year": "",
                "is_primary": 1,
                "has_gaiyuan": 0,
                "gaiyuan_prev_era_name": "",
                "gaiyuan_prev_era_year": "",
                "gaiyuan_month": "",
                "source": "manual_近现代补录",
                "note": " ".join(notes),
            }
        )

    # 去重：相邻章节交界年份，同一(年份,政权)可能从两个 fang_shiming 章节各拿到一条几乎相同的记录
    # （如 960 年"于阗""北汉""后蜀"同时出现在 10_五代十国纪年表 与 11_宋辽金纪年表 两章）。
    # 完全相同的（年号、年数都一致）只保留一条；内容有出入的两条都保留，并加注提示需人工核对。
    # 先按 BOUNDARY_RESOLUTIONS 裁决已核实的交界冲突：该(年份,政权)只保留指定章节那条，覆盖字段并写 note。
    resolved_count = 0
    resolved_rows: list[dict] = []
    for r in rows:
        res = BOUNDARY_RESOLUTIONS.get((r["lunar_year"], r["regime"])) if not r["is_primary"] else None
        if res is None:
            resolved_rows.append(r)
            continue
        if r["source"] != res["keep"]:
            continue
        r.update(res["fields"])
        r["note"] = (r["note"] + " " if r["note"] else "") + res["note"]
        resolved_rows.append(r)
        resolved_count += 1
    if resolved_count != len(BOUNDARY_RESOLUTIONS):
        sys.exit(f"BOUNDARY_RESOLUTIONS 有 {len(BOUNDARY_RESOLUTIONS) - resolved_count} 条没匹配上保留来源，检查 keep 字段")
    rows = resolved_rows

    dedup_seen: dict[tuple, dict] = {}
    deduped_rows: list[dict] = []
    conflict_count = 0
    for r in rows:
        key = (r["lunar_year"], r["is_primary"], r["regime"])
        prior = dedup_seen.get(key)
        if prior is None:
            dedup_seen[key] = r
            deduped_rows.append(r)
            continue
        if prior["era_name"] == r["era_name"] and prior["era_year"] == r["era_year"]:
            # 年号年数一致：丢弃这条，但君主取后一章节的写法（交界年后一章记的是年末状态，如前 221 年
            # 02 章「秦王政(始皇帝)」/ 03 章「始皇帝(嬴政)」）
            if r["emperor"]:
                prior["emperor"] = r["emperor"]
            continue
        # 内容不同：两条都保留，加注（只加一次）
        conflict_count += 1
        if "章节交界年份" not in prior["note"]:
            prior["note"] = (prior["note"] + " " if prior["note"] else "") + (
                f"章节交界年份，另一来源记为「{r['era_name']}{r['era_year']}」（来源 {r['source']}），需人工核对。"
            )
        deduped_rows.append(r)
    rows = deduped_rows
    if conflict_count:
        print(f"注意：{conflict_count} 处章节交界年份的政权记录内容有出入，两条都保留并加了 note，见上说明。")

    rows.sort(key=lambda r: (r["lunar_year"], 0 if r["is_primary"] else 1, r["regime"]))
    fill_earlier_eras_from_gaiyuan_day(rows)

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    csv_path = OUT_DIR / "era_names_merged.csv"
    json_path = OUT_DIR / "era_names_merged.json"

    fieldnames = [
        "lunar_year", "regime", "emperor", "era_name", "era_year", "is_primary", "has_gaiyuan",
        "gaiyuan_prev_era_name", "gaiyuan_prev_era_year", "gaiyuan_month", "source", "note",
        # 追加在末列：Rust era_names 按位置读到 gaiyuan_month 为止，末列不影响
        "gaiyuan_earlier_eras",
    ]
    with open(csv_path, "w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)

    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(rows, f, ensure_ascii=False)

    n_primary = sum(1 for r in rows if r["is_primary"])
    print(f"Wrote {csv_path} and {json_path}")
    print(f"总行数 {len(rows)}（主线 is_primary=1: {n_primary}，并存补充 is_primary=0: {len(rows) - n_primary}，"
          f"其中来自 fang_shiming 补充列的: {supplement_count}）")


def fill_earlier_eras_from_gaiyuan_day(rows: list[dict]) -> None:
    """gaiyuan_earlier_eras：该年除主年号（era_name）与改元前年号（gaiyuan_prev_era_name）之外行用过的年号，「名年数」以 / 相隔、
    按先后。方诗铭表单元格三段以上者已在 forward_fill_column 取得；此处再按改元日表 data/calendar/era_gaiyuan_day.csv 补
    一年数改者（304 年晋惠帝太安→永安→建武→永安→永兴，主年号永兴、改元前年号太安，另有永安、建武）。"""
    chains: dict[tuple[int, str], list[tuple[str, int | None]]] = {}
    with open(ROOT / "data/calendar/era_gaiyuan_day.csv", encoding="utf-8") as f:
        for r in csv.DictReader(f):
            key = (int(r["lunar_year"]), r["regime_simp"])
            ch = chains.setdefault(key, [])
            for name, yr in ((r["from_era_name_simp"], r["from_era_year_simp"]), (r["to_era_name_simp"], r["to_era_year_simp"])):
                if name and (name, cn_year_to_int(yr)) not in ch:
                    ch.append((name, cn_year_to_int(yr)))
    # 次年元年的主年号不算本年行用：武周建子，改元表把「神功二年→圣历元年」记在 697 年十一月，实为 698 年（武周历）正月
    # （次年的改元前年号亦然：证圣元年为 695 年行 gaiyuan_prev；不论政权：王莽建丑，始建国元年在 9 年「新」行）
    next_first = {(r["lunar_year"] - 1, r["era_name"]) for r in rows if str(r["era_year"]) == "1"} | \
        {(r["lunar_year"] - 1, r["gaiyuan_prev_era_name"]) for r in rows if str(r["gaiyuan_prev_era_year"]) == "1"}
    for r in rows:
        r.setdefault("gaiyuan_earlier_eras", "")
        ch = chains.get((r["lunar_year"], r["regime"]))
        if not ch:
            continue
        have = [x for x in r["gaiyuan_earlier_eras"].split("/") if x]
        for name, yr in ch:
            item = f"{name}{yr or ''}"
            if name not in (r["era_name"], r["gaiyuan_prev_era_name"]) and item not in have \
                    and (r["lunar_year"], name) not in next_first:
                have.append(item)
        r["gaiyuan_earlier_eras"] = "/".join(have)


if __name__ == "__main__":
    main()
