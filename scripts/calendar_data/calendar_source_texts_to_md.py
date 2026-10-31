"""从维基文库底本摘录各历术文（气朔、日躔、月离及进朔相关段落），生成 doc/calendar/calendar-source-texts.md（勿手改）。

摘录按起止标记原样截取，只做格式转换（表格转 Markdown、注文 {{*|…}} 转（…）、校勘脚注移到段末），不改字。
用法（仓库根）：uv run --project scripts python scripts/calendar_data/calendar_source_texts_to_md.py
"""
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent / "qinhan_eval"))
import houhanshu_star_records as hh  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
TW = "新唐書/卷"
# (小节题, 说明, [(维基文库页, 起始标记, 终止标记)])
SECTIONS = [
    ("东汉四分历（《后汉书·律历志下》）", "常数与推朔、推气之术。", [("後漢書/卷93", "元法", "推沒滅術")]),
    ("乾象历（《晋书》卷十七）", "底本此段起于积年，常数在其后。", [("晉書/卷017", "上元己丑以來", 1500)]),
    ("景初历（《宋书》卷十二）", "《晋书》卷十八亦载。", [("宋書/卷12", "壬辰元以來", 1200)]),
    ("三纪甲子元历（《晋书》卷十八）", "日法底本作六千六十二，由通数证为 6063（179044 ÷ 6063 = 29.5306）。", [("晉書/卷018", "姜岌造", 99999)]),
    ("元嘉历（《宋书》卷十三）", "", [("宋書/卷13", "上元庚辰甲子紀首", "推弦望法")]),
    ("大明历（祖冲之，《宋书》卷十三）", "", [("宋書/卷13", "上元甲子至宋大明七年", 900)]),
    ("正光历（《魏书》卷一〇七上）", "", [("魏書/卷107上", "壬子元以來，至魯隱公", "推二十四氣術")]),
    ("兴和历（《魏书》卷一〇七下）", "", [("魏書/卷107下", "上元甲子以來至春秋", "推二十四氣術")]),
    ("天保、天和、大象、开皇诸历（《隋书》卷十七）", "天保积年底本作十一萬五百六，点校本作十一萬五百〔二十〕六。开皇之法在此卷后段。", [("隋書/卷17", "後齊文宣受禪", "推積月術")]),
    ("唐历总述与戊寅元历（《新唐书》卷二十五）", "唐代八历总述；戊寅历定朔之争与贞观十九年改用平朔；常数与表（盈缩积分列多讹）。",
     [(TW + "025", "唐終始二百九十餘年", "歲星率")]),
    ("麟德历（《新唐书》卷二十六）", "常数、日躔、月离。志文无进朔之法。", [(TW + "026", "高宗時，戊寅曆益疎", "遊交終率")]),
    ("大衍历（《新唐书》卷二十八上）", "步中朔、发敛、日躔、月离、轨漏（夜半漏表）。定朔与「若非朔望有交者」简法在日躔、月离两节。",
     [(TW + "028上", "開元大衍曆演紀", "各置其氣消息衰")]),
    ("五纪历（《新唐书》卷二十九）", "「依大衍戌初進初朔」与五纪进朔法在「與大衍小異者九事」第一事。", [(TW + "029", "寶應元年六月望", "交終三億")]),
    ("正元历（《新唐书》卷二十九）", "揲法各本作三萬三千三百三十六，当作三萬二千三百三十六（点校本校勘记八，从《合钞》）。此处照底本。",
     [(TW + "029", "德宗時，五紀曆", "交終")]),
    ("宣明历（《新唐书》卷三十上）", "前段为对大衍术语的改名，后段为常数与表。观象历「今有司無傳者」亦在此卷。",
     [(TW + "030上", "憲宗即位", "凡日蝕，以定朔"), (TW + "030上", "長慶宣明曆演紀", "辰刻八刻")]),
    ("崇玄历（《新唐书》卷三十下）", "只载常数与表，无求定朔之术。", [(TW + "030下", "昭宗時", "凡升降"), (TW + "030下", "景福崇玄曆演紀", "蔀率")]),
    ("重修大明历（《金史》卷二十一、二十二）", "维基文库本两表「表略」，表见下节庚午元历及点校本摘页。进朔法在「求朔弦望定日」。",
     [("金史/卷21", "昔者聖人", "赤道宿度"), ("金史/卷22", "==步月離第五==", "求定朔弦望中積")]),
    ("庚午元历诸表（《元史》卷五十六）", "「沿用金赵知微重修大明历」（点校本《金史》卷二十一校勘记四），诸表与重修大明历相同。括注（）〔〕为点校本改字。",
     [("元史/卷056", "====二十四氣中積及朓朒====", "====求每日盈縮朓朒===="), ("元史/卷056", "====二十四氣陟降及日出分====", "====二分前後陟降率===="),
      ("元史/卷056", "====求轉定分及積度朓朒====", "====")]),
    ("《辽史·历象志》所载大明历（《辽史》卷四十二）", "志自言所载为祖冲之法，不是辽历。", [("遼史/卷42", "遼以幽、營立國", "推閏術")]),
]


def cell(c):
    c = re.sub(r"<br ?/?>", "／", c)
    c = re.sub(r"<[^>]+>", "", c)
    return note(c).replace("\n", "").replace("　", " ").replace("|", "／").strip()


def note(t):
    """注文 {{*|…}} → （…），其余模板去掉。"""
    prev = None
    while prev != t:
        prev = t
        t = re.sub(r"\{\{\*\|([^{}]*)\}\}", r"（\1）", t)
        t = re.sub(r"\{\{YL\|([^{}|]*)[^{}]*\}\}", r"\1", t)
    return re.sub(r"\{\{[^{}]*\}\}", "", t)


def table(m):
    rows = [[cell(c) for c in re.findall(r"<t[dh][^>]*>(.*?)</t[dh]>", tr, flags=re.S)] for tr in re.findall(r"<tr>(.*?)</tr>", m.group(0), flags=re.S)]
    rows = [r for r in rows if r]
    if not rows:
        return ""
    w = max(len(r) for r in rows)
    out = ["| " + " | ".join(rows[0] + [""] * (w - len(rows[0]))) + " |", "|" + "---|" * w]
    out += ["| " + " | ".join(r + [""] * (w - len(r))) + " |" for r in rows[1:]]
    return "\n\n" + "\n".join(out) + "\n\n"


def excerpt(title, start, end):
    t = hh.fetch(title)
    i = t.index(start)
    # end 为整数时取定长（术文段落无现成终止标记者）
    j = min(i + end, len(t)) if isinstance(end, int) else t.index(end, i + len(start))
    refs = [re.sub(r"\s+", "", note(r)) for r in re.findall(r"<ref[^>]*>(.*?)</ref>", t[i:j], flags=re.S)]
    s = re.sub(r"<ref[^>]*>.*?</ref>", "", t[i:j], flags=re.S)
    s = re.sub(r"\[\[(?:[^\[\]|]*\|)?([^\[\]|]*)\]\]", r"\1", s)
    s = re.sub(r"<table.*?</table>", table, s, flags=re.S)
    s = re.sub(r"<sub>|</sub>|'''", "", s)
    s = note(s)
    s = re.sub(r"^=+[ \t]*([^=\n]+?)[ \t]*=+[ \t]*$", r"**\1**\n", s, flags=re.M)
    s = re.sub(r"[ \t　]*\n[ \t　]*", "\n", s).strip()
    s = re.sub(r"\n{2,}", "\n\n", re.sub(r"(?<![\n|])\n(?![\n|])", "\n\n", s))
    if refs:
        s += "\n\n**校勘记（底本所录）**\n\n" + "\n".join(f"- {r}" for r in refs)
    return s


def main():
    L = ["# 历法原文摘录", "",
         "[历代历法总录](./calendar-compendium.md)所用术文的原文，由 `scripts/calendar_data/calendar_source_texts_to_md.py` 从维基文库底本截取（勿手改），只转换格式、不改字；底本所录校勘记移到各段之末；",
         "术文是一手史料，但有传写之讹；据此所作的推算不是史实记录（见总录「术语与证据层级」）。**底本有讹字，引用前须核点校本**（`doc/references/history/`，本地，不入库）；已核得的改动见各节说明与总录。",
         "《金史》两表与校勘记的点校本书页图在 [doc/references/calendar/[点校本]摘页](https://github.com/lhttjdr/calendar/tree/master/doc/references/calendar)。", ""]
    for head, desc, parts in SECTIONS:
        L += [f"## {head}", ""] + ([desc, ""] if desc else [])
        for title, a, b in parts:
            L += [f"> 底本：维基文库「{title}」", "", excerpt(title, a, b), ""]
    p = ROOT / "doc/calendar/calendar-source-texts.md"
    p.write_text("\n".join(L) + "\n", encoding="utf-8")
    print(p.relative_to(ROOT), sum(len(x) for x in L), "字", "残留模板" if "{{" in "".join(L) else "无模板残留")


if __name__ == "__main__":
    main()
