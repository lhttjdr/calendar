"""正史天文志月与五星凌犯、相合记录的通用解析：逐条解析纪年、月、日干支，按实历朔日表换算日期范围，目标匹配陈卓体系星官。
写出 data/phenomena/<book>_star_records.csv，供 rust/core 的 sky_culture::tests_records 检验。格式与 hanshu_star_records.csv 相同。

各书配置见 BOOKS（来源卷与节、年号年份范围、政权划分）：
- jinshu：《晋书·天文志》卷十二「月奄犯五纬」「五星聚舍」、卷十三「月五星犯列舍经星变附见」（魏、晋）；
- songshu：《宋书·天文志》卷二十三至二十六（魏、晋、宋；与《晋书》多有重出，可互校）；
- nanqishu：《南齐书·天文志》卷十二「月暈犯」、卷十三「五星相犯列宿雜災」（齐）；多以日序记日，ganzhi 列记作「26日」；
  卷十二一处「永元元年」据日干支校作「永明元年」（见 BOOKS 的 corrections）。

- 年号：由 vendor/calendar/chinese_calendar/年号表.csv（主线）取各年号元年；同名年号（如晋武帝泰始、宋明帝泰始）
  取「不早于前文年份 − 2 年、且最近」者（志文按时间顺序叙事）；年号末年取年号表与改元日表（era_gaiyuan_day.csv）的较晚者；
- 当时政权月名与建寅不同的时期（魏景初建丑）经 data/calendar/month_label_reforms.csv（`gaiyuan_date.table_month`，与
  rust/core calendar::month_labels 同源）换算到实历朔日表的建寅月；「後十二月」亦然；
- 多天体相合作一条（「水、木、火三合於東井」）；每段开始时日期上下文恢复为上一段段首纪年（可按书关闭）；「奄」同「掩」；
- 年数超出当前年号、且同段先前另有年号时，回到先前年号（占辞后应验叙述改写了年号，如「青龙二年……景初元年……四年三月」）；
- 只按历法换算日期，不用天象。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/qinhan_eval/zhi_star_records.py <book>
"""
import csv
import re
import sys
from pathlib import Path

import opencc

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
sys.path.insert(0, str(HERE))
import gaiyuan_date as g  # noqa: E402
import houhanshu_star_records as hh  # noqa: E402
import hzx  # noqa: E402

ROOT = HERE.parents[2]
ERA_TABLE = ROOT / "vendor/calendar/chinese_calendar/年号表.csv"

S2T = str.maketrans({"黄": "黃", "龙": "龍", "兴": "興", "宁": "寧", "宝": "寶", "义": "義", "凤": "鳳", "乐": "樂", "万": "萬",
                     "岁": "歲", "佑": "祐", "庆": "慶", "历": "曆", "圣": "聖", "长": "長", "东": "東", "阳": "陽", "汉": "漢", "统": "統",
                     "隆": "隆", "宁": "寧", "康": "康", "熙": "熙", "嘉": "嘉", "永": "永", "咸": "咸", "泰": "泰", "宁": "寧"})


ERA_LAST = {}  # 元年天文纪年 → 该年号末年（天文纪年）


def era_starts(y0, y1):
    """年号（繁体）→ 元年天文纪年列表（y0—y1 主线）；顺带填 ERA_LAST。"""
    out = {}
    for r in csv.DictReader(open(ERA_TABLE, encoding="utf-8")):
        y = int(r["lunar_year"])
        if not y0 - 1 <= y <= y1 + 1:
            continue
        for name, yr in ((r["era_name_simp"], r["era_year_simp"]), (r["gaiyuan_prev_era_name_simp"], r["gaiyuan_prev_era_year_simp"])):
            if not name:
                continue
            name = name[-2:]  # 改元行带「魏文帝」「元帝」等前缀；本时期年号均两字
            n = hh.cnnum(yr.rstrip("年")) if yr.rstrip("年") != "元" else 1
            start = y - n + 1
            ERA_LAST[start] = max(ERA_LAST.get(start, start), y)
            for nm in trad_variants(name):  # 异体（咸甯、甯康）与 OpenCC 繁体
                out.setdefault(nm, set()).add(start)
    return {k: sorted(v) for k, v in out.items()}


_OPENCC = opencc.OpenCC("s2t")
# 史书用的异体（《宋史》熙甯、崇甯；OpenCC 只给正体「寧」）
EXTRA_VARIANTS = {"宁": ("甯",), "升": ("昇",)}  # 甯：熙甯、咸甯；昇：昇明
_OPENCC_T2S = opencc.OpenCC("t2s")
# 目标位置上的天体简称（「太白犯歲」「太白犯木」「熒惑、歲相犯」）：单字或后接「于/於/在」时视为天体
TARGET_BODY_ABBR = {"岁": "Jupiter", "镇": "Saturn", "辰": "Mercury", "木": "Jupiter", "金": "Venus", "火": "Mars", "土": "Saturn",
                    "水": "Mercury"}


def match_target(target, ast):
    """先以 OpenCC 繁→简（houhanshu_star_records.T2S 只有几十字，「靈臺」「鍵閉」「鈎鈐」等转不出），再查别名与星官表。"""
    t = _OPENCC_T2S.convert(target)
    if t[:1] in TARGET_BODY_ABBR and (len(t) == 1 or t[1] in "于於在"):
        return ("body", TARGET_BODY_ABBR[t[0]], t[0], t[1:])
    return hh.match_target(t, ast)
MERGED_ERAS = ROOT / "data/calendar/汇总/era_names_merged.csv"


def trad_variants(name):
    """简体年号 → 可能的繁体写法：OpenCC 简→繁（opencc-python-reimplemented）逐字转换，再按
    houhanshu_star_records.T2S 的反向映射与 S2T 展开一简多繁（历→曆/歷 等）。"""
    inv = {}
    for ch in name:
        t = _OPENCC.convert(ch)
        if t != ch:
            inv.setdefault(ch, set()).add(t)
        for v in EXTRA_VARIANTS.get(ch, ()):
            inv.setdefault(ch, set()).add(v)
    for t, s_ in hh.T2S.items():
        inv.setdefault(chr(s_) if isinstance(s_, int) else s_, set()).add(chr(t) if isinstance(t, int) else t)
    for t, s_ in S2T.items():  # S2T：简 → 繁
        inv.setdefault(chr(t) if isinstance(t, int) else t, set()).add(chr(s_) if isinstance(s_, int) else s_)
    out = {""}
    for ch in name:
        out = {o + c for o in out for c in ({ch} | inv.get(ch, set()))}
    return out


def era_starts_merged(regimes, y0, y1):
    """并存政权年号（data/calendar/汇总/era_names_merged.csv，按政权取）→ {年号（简、繁写法）: [元年]}；顺带填 ERA_LAST。
    北朝年号有四字者（太平真君），取全名。
    同一政权各年号的末年至少延到下一年号元年：年中改元者旧年号当年仍行用（《金史》「興定六年正月」，方诗铭表 1222 年
    只印「元光⑧1」不印興定六），正月改元者原文不会有该年的旧年号纪年，故只放宽年数越界判断。"""
    out = {}
    starts_by_regime = {}
    for r in csv.DictReader(open(MERGED_ERAS, encoding="utf-8")):
        y = int(r["lunar_year"])
        if r["regime"] not in regimes or not y0 - 1 <= y <= y1 + 1 or not r["era_name"] or not r["era_year"]:
            continue
        start = y - int(r["era_year"]) + 1
        ERA_LAST[start] = max(ERA_LAST.get(start, start), y)
        starts_by_regime.setdefault(r["regime"], set()).add(start)
        for nm in trad_variants(r["era_name"]):
            out.setdefault(nm, set()).add(start)
        # 年内改元前的年号（如意、證聖只行用于改元之年，主年号列不出现）
        if r.get("gaiyuan_prev_era_name") and r.get("gaiyuan_prev_era_year", "").isdigit():
            ps = y - int(r["gaiyuan_prev_era_year"]) + 1
            ERA_LAST[ps] = max(ERA_LAST.get(ps, ps), y)
            for nm in trad_variants(r["gaiyuan_prev_era_name"]):
                out.setdefault(nm, set()).add(ps)
    for st in starts_by_regime.values():
        st = sorted(st)
        for a, b in zip(st, st[1:]):
            ERA_LAST[a] = max(ERA_LAST.get(a, a), b)
    return {k: sorted(v) for k, v in out.items()}


def extend_era_last_by_gaiyuan(starts):
    """年号表按农历年记主年号，改元当年只记新年号（如 304 年记永兴），旧年号实际行用到改元之日（太安行至 304 年正月）。
    以改元日表 data/calendar/era_gaiyuan_day.csv 的改元年延长旧年号的末年，使「太安三年」有效。"""
    path = ROOT / "data/calendar/era_gaiyuan_day.csv"
    for r in csv.DictReader(open(path, encoding="utf-8")):
        y, name = int(r["lunar_year"]), r["from_era_name_simp"]
        for nm in {name, name.translate(S2T)}:
            for st in starts.get(nm, []):
                if st <= y <= st + 60:
                    ERA_LAST[st] = max(ERA_LAST.get(st, st), y)


GZ = "[甲乙丙丁戊己庚辛壬癸][子丑寅卯辰巳午未申酉戌亥]"
NUM = "(?:元|[一二三四五六七八九十]+)"  # 「元」只单用（「至元元年」不可读作「至」+「元元年」）
MON = "正|臘|十二|十一|十|[一二三四五六七八九]|建[子丑寅卯辰巳午未申酉戌亥]"  # 臘、一：武周建子月名（正月=子月、臘月=丑月、一月=寅月）
def date_re(era_re, emp_re=None):
    emp = rf"(?P<emp>{emp_re})?" if emp_re else "(?P<emp>(?!))?"
    return re.compile(
    rf"^(?!自)(?:至其年|至是年|其年|是年|是歲|其歲|至其|至|其)?(?:[^\s，。]{{0,6}}?(?:帝|公|王)$|(?:[^\s，。]{{0,6}}?(?=(?:{era_re}))))?{emp}(?P<era>{era_re})?(?:(?P<yr>{NUM})[年載])?[春夏秋冬]?"
        rf"(?:(?P<leap>閏)?(?P<hou>後)?(?P<mon>{MON})?月)?(?P<xun>[上中下]旬)?(?P<day0>[初]?[一二三四五六七八九十廿卅]+日)?(?P<gz>{GZ})?(?P<day>[初]?[一二三四五六七八九十廿卅]+日)?(?:晦|朔|[甲乙丙丁戊]夜|夕|晨|旦|昏|夜|晝|[初一二三四五]更|初昏|夜半|[一二三四五]鼓|[一二三四五六七八九十]+籌|[子丑寅卯辰巳午未申酉戌亥]時|後)*$"
    )
# 天体名：另收《旧唐书》简称「歲」「鎮」（「歲、太白、熒惑合于東井」「歲掩房星」）
NAMES = {"月": "Moon", "熒惑": "Mars", "火": "Mars", "太白": "Venus", "金": "Venus", "歲星": "Jupiter", "木": "Jupiter",
         "填星": "Saturn", "鎮星": "Saturn", "土": "Saturn", "辰星": "Mercury", "水": "Mercury",
         "歲": "Jupiter", "鎮": "Saturn", "太陰": "Moon"}  # 太陰：《元史》称月
BODY_RE = "|".join(sorted(NAMES, key=len, reverse=True))
# 多天体：「水、木、火三合於東井」「金、木俱在東井」作一条
MULTI = re.compile(rf"^(?P<bodies>(?:(?:{BODY_RE})[、與与]?)+?)(?:[二三四五]|俱)?(?P<verb>合|在)[於于]?(?P<target>.+)$")
# 動詞「去／近／逼」：《旧唐书》記距離（「月去東井一尺」「熒惑去南斗五寸」），距離記入 qualifier
# 動詞前可有運行狀態（《宋史》「退行入氐」「順行，又犯」）；「淩」同「犯」類
SINGLE = re.compile(rf"^(?P<body>{BODY_RE})?(?:星)?(?:退行|逆行|順行|退|又|復)?(?P<verb>逆行守|入犯|掩食|犯|入|乘|守|掩|奄|食|在|貫|觸|去|近|逼|淩|凌|距)[于於]?(?P<target>.+)$")
# 主語省略的「與太白合」（《宋史·月犯五緯》）與目標省略的「又犯」「又掩之」（承上一條目標）
WITH = re.compile(rf"^(?:又)?與(?P<target>{BODY_RE})(?:星)?(?P<verb>合|相犯)$")
AGAIN = re.compile(r"^(?:又|復)(?P<verb>犯|掩|入|守)(?:之)?$")
# 日月食（「日有食之」「日蝕」「又日有蝕之」「月食」「月有食之，既」「日當食不食」）；「月食昴」有目标，不在此列
ECLIPSE = re.compile(r"^(?:又)?(?P<b>日|月)(?:有|當)?(?:薄)?(?:食|蝕|蚀)之?(?P<ext>既|不食|不蝕)?$")
CAPITALS = ROOT / "data/phenomena/capitals.csv"


def capital_site(regime, year):
    """都城表 data/phenomena/capitals.csv（与 tests_records.rs 共用）→ (东经°, 北纬°, 都城)。"""
    for r in csv.DictReader(ln for ln in open(CAPITALS, encoding="utf-8") if not ln.startswith("#")):
        if r["regime"] == regime and (not r["from_year"] or year >= int(r["from_year"])) and (not r["to_year"] or year <= int(r["to_year"])):
            return float(r["lon"]), float(r["lat"]), r["name"]
    raise KeyError(f"都城表缺政权「{regime}」{year} 年")


# 《清史稿·五星合聚》：单字天体连写，「土木金聚於降婁」「土金同躔壁三度」（同躔记宿度，按「在」检验所在宿）
GATHER = re.compile(r"^(?P<bodies>[土木金火水]{2,5})(?P<verb>聚|同躔)[於于]?(?P<target>.+)$")
EXTRA_ALIASES = {"井鉞": "钺", "楗閉": "键闭", "健閉": "键闭", "左角": "角", "右角": "角", "羽林": "羽林军", "羽": "羽林军",
                 "牛宿": "牵牛", "五諸侯": "五诸侯(井宿)", "積尸": "积尸（舆鬼）", "鬼": "舆鬼",
                 # 《新唐书》：鬼質（質星）即舆鬼积尸；昂為昴之訛；壘即壘壁陣
                 "鬼質": "积尸（舆鬼）", "質星": "积尸（舆鬼）", "質": "积尸（舆鬼）", "昂": "昴", "壘": "垒壁阵", "壘壁": "垒壁阵", "壁壘": "垒壁阵",
                 # 五代以后：太微垣内星名不分东西者按太微二垣；别写与省称
                 "上將": "太微左垣+太微右垣", "上相": "太微左垣+太微右垣", "右掖": "太微左垣+太微右垣", "左掖": "太微左垣+太微右垣", "執法": "太微左垣+太微右垣", "次相": "太微左垣+太微右垣", "氏": "氐", "天鐏": "天樽", "天鑰": "天籥",
                 "牛": "牵牛", "星": "七星", "庶子": "心", "端門": "太微左垣+太微右垣",
                 # 《宋史》：井即东井，權即轩辕（權星），東/西上相即太微左/右垣
                 "井": "东井", "權": "轩辕", "東上相": "太微左垣", "西上相": "太微右垣",
                 # 《元史》：御女、少民为轩辕成员；天罇即天樽
                 "御女": "轩辕", "少民": "轩辕", "天罇": "天樽"}

# 各书：(来源卷, 起始标记, 截止标记)；起始标记为 None 取全卷。years：年号表取年范围。regimes：(起年, 政权) 升序，月名改制按政权查
BOOKS = {
    "jinshu": dict(
        corrections=[("晉書/卷012", "嘉平元年二月二月己示朔", "嘉平元年二月己未朔", "衍「二月」，「示」為「未」之訛（實歷朔日表是月戊午朔，己未為二日）"),
                     ("晉書/卷012", "太寧三年十一月癸己朔", "太寧三年十一月癸巳朔", "「己」為「巳」之訛")],
        eclipse_sections=[("晉書/卷012", "===日蝕===", "===")],
        sections=[("晉書/卷012", "===月奄犯五緯===", "==="), ("晉書/卷012", "===五星聚舍===", "=="), ("晉書/卷013", "==月五星犯列舍經星變附見==", "==")],
        years=(220, 420), start_year=220, regimes=[(220, "魏"), (266, "晋")],
    ),
    "songshu": dict(
        eclipse_sections=[("宋書/卷34", None, None)],
        sections=[("宋書/卷23", None, None), ("宋書/卷24", None, None), ("宋書/卷25", None, None), ("宋書/卷26", None, None)],
        years=(220, 479), start_year=220, regimes=[(220, "魏"), (266, "晋"), (420, "宋")],
    ),
    # 《南齐书·天文志》：卷十二「月暈犯」、卷十三「五星相犯列宿雜災」；多以日序记日（「十二月二十六日」）
    # 《魏书·天象志》：卷一百五之二（月）、之三、之四（五星）；北魏、东魏按北朝推步历（calendar::push_calendar），年号取并存政权表
    "weishu": dict(
        eclipse_sections=[("魏書/卷105之一", None, None)],
        sections=[("魏書/卷105之二", None, None), ("魏書/卷105之三", None, None), ("魏書/卷105之四", None, None)],
        years=(386, 550), start_year=386, regimes=[(386, "北魏"), (534, "东魏")],
        era_regimes=["北魏", "东魏"],
        calendars=[(-10000, "北魏/西魏/北周/隋"), (534, "东魏/北齐")],
    ),
    # 《隋书·天文志》卷二十一「五代災變應」：梁、陈、北齐、北周、隋依次叙述，段首标朝代；各按当时政权的年号与历
    "suishu": dict(
        sections=[("隋書/卷21", "==五代災變應==", "==")],
        years=(500, 618), start_year=502, regimes=[(502, "梁"), (557, "陈"), (589, "隋")],
        era_regimes=["梁", "陈", "北齐", "北周", "隋"],
        # 段首「朝代 + 帝号」才切换政权（句中「周將……」「齊兵……」不算）
        regime_markers=[(r"^梁.{0,3}帝", "梁"), (r"^陳.{0,3}帝", "陈"), (r"^齊.{0,3}帝", "北齐"), (r"^周.{0,3}帝", "北周"),
                        (r"^高祖文皇帝|^隋.{0,3}帝|^煬帝", "隋")],
        reigns={"周閔帝": ("北周", 557), "明帝": ("北周", 557)},
        regime_calendars={
            "梁": [(-10000, "主表")], "陈": [(-10000, "主表")],
            "北齐": [(-10000, "东魏/北齐")], "北周": [(-10000, "北魏/西魏/北周/隋")],
            "隋": [(-10000, "北魏/西魏/北周/隋"), (589, "主表")],
        },
    ),
    # 《新唐书·天文三》「月五星凌犯及星變」「五星聚合」（617—907）；武周建子（載初元年至久視元年九月，正月=子月、臘月=丑月、
    # 一月=寅月）经月序改制表换算；天寶、至德以「載」纪年；年号取合并年号表（武周有四字年号，主线年号表只截两字不可用）
    "xintangshu": dict(
        corrections=[("新唐書/卷032", "長慶二年四年辛酉朔", "長慶二年四月辛酉朔", "「四年」為「四月」之訛")],
        eclipse_sections=[("新唐書/卷032", "==日食==", "==")],
        sections=[("新唐書/卷033", "==月五星凌犯及星變==", "=="), ("新唐書/卷033", "==五星聚合==", "'''校勘記'''")],
        years=(600, 907), start_year=617, regimes=[(617, "隋"), (618, "唐"), (690, "武周"), (705, "唐")],
        era_regimes=["隋", "唐", "武周"],
    ),
    # 《旧唐书·天文下》「災異」（日食列表等）与「災異編年至德後」（756—845，至「舊儀」司天臺沿革前止）；多以「夜」
    # 「一更」「一鼓二籌」记时，多记距离（「月去東井一尺」）；肃宗元年以斗建名月（建子月……）
    "jiutangshu": dict(
        sections=[("舊唐書/卷36", "===災異===", "====", None, "solar_dates"), ("舊唐書/卷36", "====災異編年至德後====", "舊儀")],
        years=(600, 907), start_year=618, regimes=[(618, "唐"), (690, "武周"), (705, "唐")],
        era_regimes=["唐", "武周"],
    ),
    # 《旧五代史·天文志》（卷一百三十九，全卷：五星淩犯、月暈等各节按年编次，「梁太祖開平二年，正月乙亥」年后有逗号）；
    # 《新五代史·司天考二》（卷五十九，逐年总记日月五星）。五代各朝都城见 tests_records.rs::capital
    "jiuwudaishi": dict(
        sections=[("舊五代史/卷139", None, None)],
        years=(905, 960), start_year=907, regimes=[(907, "后梁"), (923, "后唐"), (936, "后晋"), (947, "后汉"), (951, "后周")],
        era_regimes=["后梁", "后唐", "后晋", "后汉", "后周"],
    ),
    # 《宋史·天文志》按天象类型编排：卷五十三「月犯五緯」「月犯列舍」、卷五十四「月犯列舍下」主语为月（省略）；卷五十五
    # 五星犯列舍，以「歲星」「熒惑」……小标题分段；卷五十六「五緯相犯」主语齐全。北宋开封、南宋临安
    "songshi": dict(
        eclipse_sections=[("宋史/卷052", "==日食==", "==日變=="), ("宋史/卷052", "==月食==", "==月變==")],
        sections=[("宋史/卷053", "==月犯五緯==", "==", "月"), ("宋史/卷053", "==月犯列舍==", "@@", "月"),
                  ("宋史/卷054", None, None, "月"), ("宋史/卷055", None, None), ("宋史/卷056", "==五緯相犯==", "==老人星==")],
        years=(958, 1280), start_year=960, regimes=[(960, "宋")], era_regimes=["宋"],
        para_reset=False,  # 一年之事常分数段（「淳化……三年三月癸亥」下段「九月戊午」仍为三年），新段不回到上段段首纪年
    ),
    # 《金史·天文志》（卷二十「月五星淩犯及星變」）：金历与宋历偶有一两日之差，暂按实历朔日表（宋历），干支不合者或即其差
    "jinshi": dict(
        eclipse_sections=[("金史/卷20", "==日薄食煇珥雲氣==", "==")],
        sections=[("金史/卷20", "==月五星淩犯及星變==", "@@")],
        corrections=[("金史/卷20", "九年二年癸亥", "九年二月癸亥", "「二年」為「二月」之訛（上承八年閏八月至十二月，下接七月）")],
        years=(1113, 1235), start_year=1115, regimes=[(1115, "金")], era_regimes=["金"], para_reset=False,
    ),
    # 《元史·天文志》（卷四十八「月五星凌犯及星變上」、卷四十九「下」）：月称「太陰」；点校本改字（「十（二）〔一〕月」）照改
    "yuanshi": dict(
        eclipse_sections=[("元史/卷048", "==日薄食暈珥及日變==", "==")],
        sections=[("元史/卷048", "==月五星凌犯及星變上==", "==="), ("元史/卷049", "==月五星凌犯及星變下==", "===")],
        years=(1258, 1370), start_year=1260, regimes=[(1260, "元")], era_regimes=["蒙古", "元"], para_reset=False,
    ),
    # 《明史·天文志》卷二十六「月掩犯五緯」（主语为月）、「五緯掩犯」「五緯合聚」「五緯掩犯恆星」；维基文库为简体，转繁后解析。
    # 「五緯掩犯恆星」以「△歲星」「△熒惑」……小标题分段。节标题前有全角空格，止于「　　▲」。南京（1421 年迁北京）
    "mingshi": dict(
        sections=[("明史/卷26", "▲月掩犯五纬", "　　▲", "月"), ("明史/卷26", "▲五纬掩犯", "　　▲"), ("明史/卷26", "▲五纬合聚", "　　▲"),
                  ("明史/卷26", "▲五纬掩犯恒星", "@@")],
        years=(1366, 1645), start_year=1368, regimes=[(1368, "明")], era_regimes=["明"], para_reset=False, simplified=True,
    ),
    # 《清史稿·天文志》：卷三十六「五星合聚」、卷三十七「月五星淩犯掩距」（「犯」相距一度内、「距」入相距书限、「於某宿」
    # 记所在宿；卷首定义见原文）；卷三十至三十五为恒星、仪象等，维基文库无录文。北京
    "qingshigao": dict(
        sections=[("清史稿/卷36", None, None), ("清史稿/卷37", "月五星淩犯掩距天官書", "太白晝見")],
        years=(1615, 1912), start_year=1644, regimes=[(1644, "清")], era_regimes=["清"], para_reset=False,
    ),
    # 《汉书·天文志》（卷二十六，自「漢元年十月」起至校勘记）：太初以前按秦汉初复原历（hzx.py，十月岁首、闰为後九月），
    # 以后按实历朔日表；汉初以帝号纪年（漢、孝惠、高后、孝文、孝景），文景「中」「後」分段按年号表「中元」「後元」
    "hanshu": dict(
        sections=[("漢書/卷026", "@漢元年十月", "==校勘記==")],
        years=(-210, 25), start_year=-206, regimes=[(-205, "西汉"), (9, "新"), (23, "东汉")],
        calendars=[(-10000, "复原历"), (-103, "主表")],
        reigns={"漢": ("西汉", -205), "高皇帝": ("西汉", -205), "高帝": ("西汉", -205), "孝惠": ("西汉", -193),
                "高后": ("西汉", -186), "孝文": ("西汉", -178), "孝景": ("西汉", -155)},
        era_regimes=["汉", "新"], sub_era=True, eclipses=False,
    ),
    # 《后汉书·天文志》（续汉书志，今本卷一百至一百二）：正文与刘昭注引《古今注》（source 列）；止于校勘记；
    # 日月食另由 houhanshu_eclipse_dates.py 从五行志整理，此处不输出
    "houhanshu": dict(
        sections=[("後漢書/卷100", None, "==校勘記=="), ("後漢書/卷101", None, "==校勘記=="), ("後漢書/卷102", None, "==校勘記==")],
        years=(23, 221), start_year=25, regimes=[(25, "东汉")], notes_prefix="古今注", eclipses=False, para_reset=False,
        simplified=True,  # 维基文库卷一百二桓帝以下为简体，统一转繁
    ),
    "xinwudaishi": dict(
        sections=[("新五代史/卷59", None, None)],
        years=(905, 960), start_year=907, regimes=[(907, "后梁"), (923, "后唐"), (936, "后晋"), (947, "后汉"), (951, "后周")],
        era_regimes=["后梁", "后唐", "后晋", "后汉", "后周"],
    ),
    "nanqishu": dict(
        eclipse_sections=[("南齊書/卷12", "==日蝕==", "==日光色==")],
        sections=[("南齊書/卷12", "==月暈犯==", "==="), ("南齊書/卷13", "==五星相犯列宿雜災==", "==")],
        years=(454, 502), start_year=479, regimes=[(420, "宋"), (479, "齐")],
        # 校改：(卷, 原文, 改作, 依据)。只按历法（日干支是否在所记之月）定，不用天象
        corrections=[(
            "南齊書/卷12", "永元元年正月己亥，月犯心後星", "永明元年正月己亥，月犯心後星",
            "「月暈犯」此段承建元四年，其后「二年」……「十一年」连记，永元只三年、永明十一年；按永明计日干支在月 122/132 条，"
            "按永元计 23/132 条（维基文库本，2026-10-27 取）",
        )],
    ),
}


def day_number(s):
    """「二十六日」「初三日」「廿一日」→ 日序。"""
    s = s.rstrip("日").lstrip("初").replace("廿", "二十").replace("卅", "三十")
    return hh.cnnum(s)


def calendar_of(year, cfg, regime=None):
    """该年所用历表：「主表」（实历朔日表）或推步政权名（data/calendar/推步政权朔日表.csv）。
    一书多政权者（cfg 有 regime_calendars）按当时政权取。"""
    table = cfg.get("regime_calendars", {}).get(regime) or cfg.get("calendars", [(-10000, "主表")])
    return [c for y, c in table if y <= year][-1]


DYNASTY_CHARS = {"漢": "汉", "魏": "魏", "晉": "晋", "宋": "宋", "齊": "齐", "梁": "梁", "陳": "陈", "隋": "隋", "唐": "唐", "周": "北周"}


def dynasty_span(prefix, regimes):
    """年号前缀首字为朝代名（「魏明帝」「晉武帝」）→ 该朝在 regimes 配置中的 [起年, 下一朝起年)；否则 None。"""
    if not prefix or prefix[0] not in DYNASTY_CHARS:
        return None
    name = DYNASTY_CHARS[prefix[0]]
    for i, (y, r) in enumerate(regimes):
        if r == name:
            return y, (regimes[i + 1][0] if i + 1 < len(regimes) else 10**6)
    return None


def expand_together(clauses, date_re_):
    """并列多日共一事，把动作展开到每个日期之后（A，犯X，B，犯X，C，犯X）：
    「A，B，C，俱犯X」——「俱/並/皆」起头的动作展开到其前连续的日期分句；
    「A及B，犯X」——日期间以「及」连接（DATE_AND 替换为「，§」，§ 标记）者，其后不带「俱」的动作同样展开。"""
    out, dates, joined = [], [], False
    for c in clauses:
        c = c.strip("　 ")
        if c.startswith("§"):
            c, joined = c[1:], True
        m = re.match(r"^(?:俱|並|并|皆)(.+)$", c)
        dm = date_re_.match(c) if c else None
        is_date = bool(dm and (dm["mon"] or dm["gz"] or dm["leap"]))
        action = m.group(1) if m else (c if joined and not is_date else None)
        if c and action and len(dates) >= 2:
            out = out[: len(out) - len(dates)]
            for d in dates:
                out += [d, action]
            dates, joined = [], False
            continue
        if is_date:
            dates.append(c)
        elif c:
            dates, joined = [], False
        out.append(c)
    return out


def calendar_label(table):
    """记录表 calendar 列：主表记「实历朔日表」，推步历记「推步:政权」（data/calendar/推步政权朔日表.csv 的 regime）。"""
    if table == "复原历":
        return "复原历"
    return "实历朔日表" if table == "主表" else f"推步:{table}"


def leap_month_span(table, year):
    """只记「闰月」：该历该年的闰月。"""
    for m in range(1, 13):
        try:
            return g.calendar_month_span(table, year, m, True)
        except KeyError:
            continue
    raise KeyError(f"{table} {year} 无闰月")


def regime_of(year, regimes):
    """志文纪年所属政权（月名改制按政权区分）。"""
    return [r for y, r in regimes if y <= year][-1]


def section_text(title, head, stop, corrections=()):
    t = hh.fetch(title)
    t = re.sub(r"-\{([^{}]*)\}-", r"\1", t)  # 维基文库繁简转换标记「-{丑}-」
    # 点校本改字：「十（二）〔一〕月」删圆括号内字、补方括号内字，取改正后文字
    t = re.sub(r"（[^（）〔〕\n]{1,8}）(?=〔)", "", t)
    t = re.sub(r"〔([^〔〕\n]{1,8})〕", r"\1", t)
    for juan, old, new, _why in corrections:
        if juan == title:
            assert t.count(old) == 1, f"校改原文须在 {title} 中恰出现一次：{old}"
            t = t.replace(old, new)
    if head is not None or stop:
        # 起始标记以「@」开头者本身含在节内（《汉书》自「漢元年十月」起）
        i = (t.index(head[1:]) if head.startswith("@") else t.index(head) + len(head)) if head is not None else 0
        j = t.find("\n" + stop, i)
        t = t[i : j if j > 0 else len(t)]
    # 干支形讹：阳干后「戍」为「戌」，阴干后「已」为「巳」（「丙戍」「癸已」）
    t = re.sub(r"(?<=[甲丙戊庚壬])戍", "戌", t)
    t = re.sub(r"(?<=[乙丁己辛癸])已", "巳", t)
    return re.sub(r"^=+[^=\n]+=+[ \t]*$", "\n", t, flags=re.M)  # 节标题单独成段（「==日蝕==\n魏文帝……」不并入首句）


def main(book):
    cfg = BOOKS[book]
    out_path = ROOT / f"data/phenomena/{book}_star_records.csv"
    eras = era_starts_merged(cfg["era_regimes"], *cfg["years"]) if cfg.get("era_regimes") else era_starts(*cfg["years"])
    extend_era_last_by_gaiyuan(eras)
    # 一书多政权：各政权各自的年号表（同名年号按当时政权取，如北齐天保与后梁天保）
    eras_by_regime = {r: era_starts_merged([r], *cfg["years"]) for r in cfg.get("era_regimes", [])} if cfg.get("regime_markers") else {}
    era_re = "|".join(sorted(eras, key=len, reverse=True))
    reigns = cfg.get("reigns", {})  # 无年号、以帝号纪年者：帝号 → (政权, 元年)
    DATE = date_re(era_re, "|".join(sorted(reigns, key=len, reverse=True)) or None)
    # 段首分句以「年号+年数」起头而整句不成日期式（「天復元年五月自丁酉至于己亥」）：至少切换年号与年
    ERA_HEAD = re.compile(rf"^(?:{era_re})(?:{NUM})[年載]")
    markers = [(re.compile(p_), r_) for p_, r_ in cfg.get("regime_markers", [])]
    # 日期之間的「及」作分句（《明史》「十一月癸卯及永樂四年正月戊午，五年六月丙午，……俱犯熒惑」）
    DATE_AND = re.compile(rf"及(?=(?:{era_re})?(?:{NUM}[年載])?閏?(?:{MON})月|{GZ}[，、])")
    ast = hh.load_asterisms()
    for k, v in EXTRA_ALIASES.items():
        hh.ALIASES.setdefault(_OPENCC_T2S.convert(k), v)
    rows, unmatched = [], []
    eclipses = {"solar": [], "lunar": []}

    def month_span_of(year, mon, mon_name, hou, leap, cur_regime):
        """所记之月（当时月名）→ (朔 JDN, 下月朔 JDN, 表月)；无此月抛异常。"""
        t = None
        if calendar_of(year, cfg, cur_regime) == "复原历":
            # 太初以前：秦汉初复原历（hzx.year，十月岁首，闰为「後九月」），year 为汉历年
            labs = hzx.year(year)
            keys = [lab for lab, _ in labs]
            i = keys.index("后9" if (hou or leap) else str(mon))
            nxt = labs[i + 1][1] if i + 1 < len(labs) else hzx.year(year + 1)[0][1]
            return labs[i][1], nxt, None
        if mon is not None:
            name = ("后" if hou else "") + mon_name
            rg = cur_regime or regime_of(year, cfg["regimes"])
            t = g.table_month(rg, year, name, leap)
            if t is None and name.startswith("建"):
                # 「上元二年……其年建子月」：改制表把建子、建丑月记在次年（元年）名下
                t = g.table_month(rg, year + 1, name, leap)
            if t is None:
                raise KeyError(f"{rg} {year} 无{name}")
            s0, e0 = g.calendar_month_span(calendar_of(year, cfg, cur_regime), *t)
        else:
            # 只记「闰月」不记月序：取该历该年的闰月
            s0, e0 = leap_month_span(calendar_of(year, cfg, cur_regime), year)
        return s0, e0, t

    def add_eclipse(kind, sent, extent, note):
        """日月食记录：日期取当前上下文；日食无干支者取朔（记「晦」者取月末日），月食无干支者取望（朔后 14 日）。"""
        if era_start is None or yr is None or (mon is None and not leap):
            return None
        year = era_start + yr - 1
        try:
            s0, e0, t = month_span_of(year, mon, mon_name, hou, leap, cur_regime)
        except Exception:
            unmatched.append(("日期", sent.strip()[:40]))
            return None
        if gz:
            off = (g.GANZHI.index(gz) - (s0 + 49) % 60) % 60
            in_month = s0 + off < e0
            jdn = s0 + off if in_month else min((s0 + off - 60, s0 + off), key=lambda x: abs(x - (s0 + e0) // 2))
        elif day is not None:
            in_month, jdn = day <= e0 - s0, s0 + day - 1
        else:
            in_month = True
            jdn = (e0 - 1 if pos_mark == "晦" else s0) if kind == "solar" else s0 + 14
        rg = cur_regime or regime_of(year, cfg["regimes"])
        lon, lat, place = capital_site(rg, year)
        y0, m0, d0 = g.from_jdn(jdn)
        rec = dict(regime=rg, era=era_label.translate(S2T), year_in_era=yr, year=year,
                   month=("闰" if leap else "") + (str(mon) if mon else ""), leap=int(leap), ganzhi=gz or "",
                   position=pos_mark if pos_mark else ("" if gz or day else "只记到月"), extent=extent, lodge="",
                   calendar=calendar_label(calendar_of(year, cfg, cur_regime)), jdn=jdn, julian=f"{y0}-{m0:02d}-{d0:02d}",
                   lunar_day=jdn - s0 + 1, in_month=int(in_month), site=place, site_lon=lon, site_lat=lat, guess_jdn=jdn,
                   note=note, text=sent.strip()[:80])
        eclipses[kind].append(rec)
        return rec

    sections = [(sec, False) for sec in cfg["sections"]] + [(sec, True) for sec in cfg.get("eclipse_sections", [])]
    for sec, eclipse_only in sections:
        # 第四项（可选）：该节省略主语时的默认天体（《宋史·月犯五緯》「犯歲星」主语为月）
        title, head, stop = sec[:3]
        default_body = NAMES[sec[3]] if len(sec) > 3 and sec[3] else None
        solar_dates = len(sec) > 4 and sec[4] == "solar_dates"  # 节内带朔日期即日食（《旧唐书》）
        pos_mark, last_ecl, last_ecl_sent = "", None, -1
        last_target = None
        text = section_text(title, head, stop, cfg.get("corrections", ()))
        if cfg.get("simplified"):
            text = _OPENCC.convert(text)  # 简体原文（维基文库《明史》天文志）转繁体，与年号表繁体写法、本解析器正则一致
        era_start = outer_era = None
        era_label = outer_label = ""  # 当前/段内先前年号（或帝号）的原文写法，按起始年反查会撞上同年起始的别国年号
        cur_regime = None
        para_first_clause = False
        prev_year = cfg["start_year"]
        yr = mon = mon_name = gz = xun = day = None
        leap = hou = False
        last_bodies, last_sentence, fresh_sentence = None, -1, -1
        sent_no = 0
        lead_ctx = None
        pieces = []
        for para in text.split("\n\n"):
            head_line = para.strip("　 \n△▲")  # 《明史》小标题「△歲星」
            if head_line in NAMES or head_line in ("五緯相合", "五緯俱見"):
                # 小標題（《宋史》卷五十五「歲星」「熒惑」……）：其下各條主語
                pieces.append(("SUBJ", NAMES.get(head_line)))
                continue
            pieces.append(("PARA", False))
            pieces.extend(hh.segments(para))
        para_first_date = False
        for seg, note in pieces:
            if seg == "SUBJ":
                default_body, last_target = note, None
                continue
            if seg == "PARA":
                if lead_ctx is not None and cfg.get("para_reset", True):
                    era_start, era_label, yr, mon, mon_name, gz, xun, leap, day = lead_ctx
                para_first_date = True
                para_first_clause = True  # 政权标记只看段首
                outer_era = None  # 年数越界只回退到同段内先前的年号（段内占应叙述），不跨段
                continue
            if note:
                # 注文一般不收；配置 notes_prefix 者收以之起头的注文（《后汉书》刘昭注引《古今注》）
                if not (cfg.get("notes_prefix") and seg.lstrip().startswith(cfg["notes_prefix"])):
                    continue
                source = cfg["notes_prefix"]
            else:
                source = "正文"
            seg = hh.drop_other_quotes(seg)
            for sent in re.split(r"[。！？]", seg):
                sent_no += 1
                for clause in expand_together(re.split(r"[，；：「」『』\s]", DATE_AND.sub("，§", sent)), DATE):
                    clause = clause.strip("　 ")
                    if not clause:
                        continue
                    if cfg.get("sub_era"):
                        # 汉初帝号纪年分段：「孝文後二年」「中三年」「後元年」→ 年号表的「後元」「中元」
                        clause = re.sub(rf"(?<![元年])([中後])(?!元)({NUM}年)", r"\1元\2", clause)
                        clause = re.sub(r"(?<![元年])([中後])元年", r"\1元元年", clause)
                    if para_first_clause:
                        para_first_clause = False
                        for mk, rg in markers:
                            if mk.match(clause):
                                cur_regime = rg
                                break
                    m = DATE.match(clause)
                    if not m and para_first_date and (mh := ERA_HEAD.match(clause)):  # 只限段首，段中多为追述
                        m = DATE.match(mh.group(0))
                    if m and any(m.group(k) for k in ("emp", "era", "yr", "mon", "gz", "xun", "leap", "day", "day0")):
                        if m["emp"] and not m["era"]:
                            cur_regime, era_start = reigns[m["emp"]]
                            era_label, outer_era = m["emp"], None
                        if m["era"]:
                            pool = eras_by_regime.get(cur_regime, {}).get(m["era"]) or eras[m["era"]]
                            # 年号前明书朝代（「魏明帝太和」「晉武帝泰始」）：只在该朝年代内取（《宋书·五行志》各门类自魏重起，
                            # 「不早于前文」之法失效）
                            span = dynasty_span(clause[: clause.find(m["era"])], cfg["regimes"])
                            if span:
                                pool = [s for s in pool if span[0] - 1 <= s < span[1]] or pool  # 易代之年改元（泰始元年 265 年十二月受禅）
                            cands = [s for s in pool if s >= prev_year - 2] or pool
                            if span:
                                cands = pool
                            new_start = min(cands, key=lambda s: abs(s - prev_year))
                            if not span and m["era"] == era_label and era_start in pool:
                                new_start = era_start  # 同一年号接续（「晉武帝泰始二年……泰始七年」），不另取别朝同名年号
                            if new_start != era_start:
                                outer_era, outer_label = era_start, era_label
                            era_start, era_label = new_start, m["era"]
                        elif m["yr"] and era_start is not None and era_start + hh.cnnum(m["yr"]) - 1 > ERA_LAST.get(era_start, 10**6) \
                                and outer_era is not None:
                            # 年数超出当前年号、且同段先前另有年号：当前年号来自占辞后的应验叙述（如「青龙二年……景初元年……
                            # 三年正月，天子崩。四年三月己巳」），回到叙述前的年号（青龙）。不跨段：如「太安三年」承上段太安，
                            # 太安实行至 304 年正月，年号表只记两年，不应回退
                            era_start, era_label, outer_era = outer_era, outer_label, None
                        if m["yr"]:
                            yr, mon, gz, xun, day = hh.cnnum(m["yr"]), None, None, None, None
                        elif m["era"]:
                            yr = None
                        if m["mon"] or m["leap"]:
                            leap = bool(m["leap"])
                            if m["mon"] and m["mon"][0] == "建":  # 肃宗元年以斗建名月（建子月=十一月）
                                mon = "寅卯辰巳午未申酉戌亥子丑".index(m["mon"][1]) + 1
                            else:
                                mon = {"正": 1, "臘": 12}.get(m["mon"]) or (hh.cnnum(m["mon"]) if m["mon"] else None)
                            mon_name = (m["mon"] + "月").translate(hh.T2S) if m["mon"] else None
                            hou = bool(m["hou"])
                            gz, xun, day = None, None, None
                            fresh_sentence = sent_no
                        if m["xun"]:
                            xun = "上中下".index(m["xun"][0])
                        if m["gz"]:
                            gz, day = m["gz"], None
                            fresh_sentence = sent_no
                        if m["gz"] or m["mon"] or m["leap"] or m["day"] or m["day0"]:
                            pos_mark = "朔" if "朔" in clause else ("晦" if "晦" in clause else "")
                        if solar_dates and m["gz"] and pos_mark == "朔":
                            # 《旧唐书·災異》日食以朔日列举（「武德元年十月壬申朔，四年八月丙戌朔……」），不另书「日食」
                            last_ecl = add_eclipse("solar", sent, "", "不蝕" if re.search(r"不[蝕食]", sent) else "")
                            last_ecl_sent = sent_no
                        if (m["day"] or m["day0"]) and not m["gz"]:
                            # 以日序记日（《南齐书》「十二月二十六日」）；日序与干支并记（「四月二十七日丙戌」）时以干支为准
                            day, gz = day_number(m["day"] or m["day0"]), None
                            fresh_sentence = sent_no
                        if era_start is not None and yr is not None:
                            prev_year = max(prev_year, era_start + yr - 1)
                        if para_first_date:
                            lead_ctx = (era_start, era_label, yr, mon, mon_name, gz, xun, leap, day)
                            para_first_date = False
                        continue
                    me = ECLIPSE.match(clause)
                    if me and (fresh_sentence == sent_no or sent_no == last_sentence):
                        kind = "solar" if me["b"] == "日" else "lunar"
                        last_ecl = add_eclipse(kind, sent, "既" if me["ext"] == "既" else "",
                                               "当食不食" if me["ext"] in ("不食", "不蝕") else "")
                        last_ecl_sent = sent_no
                        continue
                    if last_ecl is not None and last_ecl_sent == sent_no:
                        if clause in ("既", "蝕既", "食既"):
                            last_ecl["extent"] = "既"
                            continue
                        if clause.startswith("在") and hh.lodge_degree(clause[1:]):
                            last_ecl["lodge"] = "{}{}".format(*hh.lodge_degree(clause[1:]))
                            continue
                        if re.search(r"不見|陰雲|雲陰", clause):
                            last_ecl["note"] = (last_ecl["note"] + " " + clause).strip()
                    if eclipse_only:
                        continue
                    mm = MULTI.match(clause)
                    ms = SINGLE.match(clause)
                    mw = WITH.match(clause) if default_body else None
                    ma = AGAIN.match(clause) if last_target else None
                    mg = GATHER.match(clause)
                    if mg:
                        bodies = [NAMES[b] for b in mg["bodies"]]
                        verb, target = ("在" if mg["verb"] == "同躔" else "合"), mg["target"]
                    elif mw:
                        bodies, verb, target = [default_body], mw["verb"], mw["target"]
                    elif ma:
                        bodies, verb, target = (last_bodies or [default_body]), ma["verb"], last_target
                    elif mm and len(re.findall(BODY_RE, mm["bodies"])) >= 2:
                        bodies = [NAMES[b] for b in re.findall(BODY_RE, mm["bodies"])]
                        verb, target = mm["verb"], mm["target"]
                    elif ms:
                        if ms["body"]:
                            bodies = [NAMES[ms["body"]]]
                        elif sent_no == last_sentence and last_bodies:
                            bodies = last_bodies
                        elif default_body:
                            bodies = [default_body]
                        else:
                            continue
                        verb, target = ms["verb"].replace("奄", "掩"), ms["target"]
                    else:
                        continue
                    if not (fresh_sentence == sent_no or sent_no == last_sentence):
                        continue
                    if verb == "距" and target.startswith("星"):
                        continue  # 「距星東南一尺」：距星为某宿距星，非動詞「距」+七星
                    if target[:2] in ("上相", "次相", "上將", "次將") and sent_no == last_sentence and last_target \
                            and last_target.startswith("房"):
                        target = "房" + target  # 房宿四星亦名上相、次相、次將、上將（「歲星守房，掩上相」），不是太微垣的
                    if era_start is None or yr is None or (mon is None and not leap):
                        continue
                    year = era_start + yr - 1
                    try:
                        s0, e0, t = month_span_of(year, mon, mon_name, hou, leap, cur_regime)
                    except Exception:
                        unmatched.append(("日期", sent.strip()[:40]))
                        continue
                    if gz:
                        off = (g.GANZHI.index(gz) - (s0 + 49) % 60) % 60
                        in_month = s0 + off < e0
                        jdn = s0 + off if in_month else min((s0 + off - 60, s0 + off), key=lambda x: abs(x - (s0 + e0) // 2))
                        j0 = j1 = jdn
                    elif day is not None:
                        in_month, j0, j1 = day <= e0 - s0, s0 + day - 1, s0 + day - 1
                    elif xun is not None:
                        in_month, j0, j1 = True, s0 + 10 * xun, (s0 + 10 * xun + 9 if xun < 2 else e0 - 1)
                    else:
                        in_month, j0, j1 = True, s0, e0 - 1
                    kind, tid, tname, qual = match_target(target, ast)
                    ld = hh.lodge_degree(target) if verb == "在" else None
                    if ld and re.search(r"[一二三四五六七八九十]+[寸尺分]", target) and "度" not in target:
                        ld = None  # 「在南斗七寸」是距离，不是入宿度
                    # 「犯歲星於昴」：目标后的「於某宿」记入 lodge 列（无度数），供推算所在宿比对（《清史稿》）
                    qlodge = ""
                    if not ld and qual[:1] in ("于", "於") and kind == "body":
                        q = qual[1:]
                        qlodge = next((v for k, v in sorted(hh.LODGE_FULL.items(), key=lambda x: -len(x[0])) if q.startswith(k)), "") \
                            or (q[0] if q[:1] and q[0] in hh.LODGES else "")
                    if kind == "unknown" and not ld:
                        unmatched.append(("目标", clause))
                        continue
                    y0, m0, d0 = g.from_jdn(j0)
                    era_name = era_label.translate(S2T)
                    rows.append(dict(
                        juan=title.split("卷")[-1].zfill(3), source=source, reign="", regime=cur_regime or regime_of(year, cfg["regimes"]), era=era_name, year_in_era=yr, year=year,
                        month=("闰" if leap else "") + ("后" if hou else "") + ((str(mon) if t is None or t[:2] == (year, mon) else mon_name[:-1]) if mon else "") + ("" if xun is None else "上中下"[xun] + "旬"),
                        ganzhi=gz or (f"{day}日" if day else ""), calendar=calendar_label(calendar_of(year, cfg, cur_regime)), jdn_start=j0, jdn_end=j1, julian=f"{y0}-{m0:02d}-{d0:02d}",
                        in_month=int(in_month), bodies=" ".join(bodies), verb=verb.translate(hh.T2S), target_kind=kind,
                        target_id=tid, target_name=tname, qualifier=qual[:12], lodge=ld[0] if ld else qlodge,
                        lodge_deg=ld[1] if ld else "", text=sent.strip()[:120],
                    ))
                    last_bodies, last_sentence, last_target = bodies, sent_no, target
    for r in rows:
        for k, v in r.items():
            if isinstance(v, str):
                r[k] = v.replace(",", "，").replace('"', "＂").replace("\n", " ").replace("\r", "")
    with open(out_path, "w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)
    for kind, recs in eclipses.items() if cfg.get("eclipses", True) else ():
        dst = ROOT / f"data/phenomena/{book}_{kind}_eclipses.csv"
        if not recs:
            continue
        for r in recs:
            for k, v in r.items():
                if isinstance(v, str):
                    r[k] = v.replace(",", "，").replace('"', "＂").replace("\n", " ")
        with open(dst, "w", encoding="utf-8", newline="") as f:
            w = csv.DictWriter(f, fieldnames=list(recs[0].keys()))
            w.writeheader()
            w.writerows(recs)
        print(dst.name, len(recs), "条，干支不在月", sum(not r["in_month"] for r in recs))
    from collections import Counter
    print(len(rows), "条", Counter(r["verb"] for r in rows), Counter(r["target_kind"] for r in rows),
          "多天体", sum(" " in r["bodies"] for r in rows), "只记到月", sum(not r["ganzhi"] for r in rows),
          "干支不在月", sum(not r["in_month"] for r in rows))
    print("未匹配：", len(unmatched), Counter(u[1][:4] for u in unmatched if u[0] == "目标").most_common(25))


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "jinshu")
