"""《董公選擇日要覽》作择日页第二套宜忌：生成原文表与逐日判定表初稿。

用法（仓库根）：
  python3 scripts/folk/dong_gong_tables.py          # 只重写 data/folk/dong_gong.csv（原文表，确定性切分；保留「校」列）
  python3 scripts/folk/dong_gong_tables.py --draft  # 另把逐日判定初稿写到 data/folk/dong_gong_days.draft.csv（不入库，供 diff）

逐日判定表 data/folk/dong_gong_days.csv 是**人工校订后的依据层**：初稿由本脚本按 dong_vs_xieji.py 的解析规则生成，
再逐条对原文复核；校订逐条写在 scripts/folk/dong_gong_review.py，由它在初稿上施校并写出判定表，改动与疑点记在「校」列。
本脚本不写 dong_gong_days.csv。格式见 doc/folk/tongshu_yiji.md「择日页董公宜忌的实现」。

与 dong_vs_xieji.py 的不同：事目用董公自己的话（起造、動土、婚姻、安葬……，不先映射到协纪六十事，映射另见
data/folk/dong_gong_events.csv）；泛言（「百事不宜」「次吉」等不举事项者）记为一行「諸事」，不展开成各事；
另抽等第（大吉、上吉、次吉……更凶、大凶）。
"""
import csv
import re
import sys
from collections import Counter

sys.path.insert(0, "scripts/folk")
import dong_vs_xieji as base  # noqa: E402  解析规则沿用对照脚本

SRC = base.SRC
OUT_ENTRY = "data/folk/dong_gong.csv"
OUT_DRAFT = "data/folk/dong_gong_days.draft.csv"

# 董公事目（规范名, 原文用语）。顺序即表内排列顺序
DONG_EVENTS = [
    ("起造", r"起造|營建|建造"),
    ("修造", r"修造|修整|修理|修作|裝修|造作|營為|修為|動作|修方"),
    ("動土", r"動土|興工"),
    ("豎造", r"定磉|拴架|造架|上樑|上梁|豎柱"),
    ("作灶", r"作灶"),
    ("安門", r"安門"),
    ("作倉庫", r"倉庫|倉廩"),
    ("作欄", r"欄"),
    ("婚姻", r"婚姻|嫁娶|娶親|成親"),
    ("納采", r"納采|問名"),
    ("會親", r"會親"),
    ("入宅", r"入宅"),
    ("移居", r"移居|移徙|遷居|徙居"),
    ("開張", r"開張"),
    ("出行", r"出行|遠行"),
    ("上官", r"上官|赴任|參官"),
    ("見貴", r"見貴"),
    ("上學", r"上學|入學"),
    ("安葬", r"安葬|埋葬|葬"),
    ("破土", r"破土|斬草|開山"),
    ("生基", r"生基|合板|壽木"),
    ("還愿", r"還福愿|還愿|還願"),
]
EVENT_NAMES = [e for e, _ in DONG_EVENTS]
base.EVENTS = DONG_EVENTS
base.ALL_EVENTS = EVENT_NAMES

GRADE = r"又次吉|小用次吉|次吉|上吉|大吉|更加凶險|更凶|大凶"


def grade_of(clause, v):
    m = re.findall(GRADE, clause)
    if m:
        g = m[-1]
        if v == "宜" and "吉" in g or v == "忌" and "凶" in g:
            return g
    return ""


# 依据只留含事目、宜忌之辞、干支或神煞的分段（删去「招官司」「三」等应验残段）
KEEP = (r"宜|利|忌|不可|吉|凶|可用|用事|" + "|".join(p for _, p in DONG_EVENTS) +
        r"|[甲乙丙丁戊己庚辛壬癸][子丑寅卯辰巳午未申酉戌亥]|餘|往亡|賊|天羅|地網|煞|殺|紅沙|黃沙|沙黃|黃羅|紫檀|天皇|地皇|庫|德|福星|"
        r"四廢|朱雀|勾絞|到州|無氣|窮日|天喜|天赦|白虎|星|月厭|天空|轉|陰|陽|生人")


def basis(clause):
    return "，".join(f for f in clause.split("，") if re.search(KEEP, f))


def collapse(evs):
    """{事: (宜忌, 原句, 泛言)} → [(事, 宜忌, 原句, 泛言)]：泛言推及诸事者合为一行「諸事」"""
    blanket = Counter((v, c) for e, (v, c, b) in evs.items() if b)
    rows = []
    main = None
    if blanket:
        (mv, mc), n = blanket.most_common(1)[0]
        # 同一泛言之辞覆盖的事目过半，视作「諸事」；其余事目单列
        if n * 2 > len(EVENT_NAMES):
            main = (mv, mc)
            rows.append(("諸事", mv, mc, True))
    for e in EVENT_NAMES:
        if e in evs:
            v, c, b = evs[e]
            if main and b and (v, c) == main:
                continue
            rows.append((e, v, c, b))
    return rows


def entries():
    s = open(SRC, encoding="utf-8").read()
    parts = re.split(r"==([正二三四五六七八九十]+[一二]?月)[　 ](.)月==", s)
    for i in range(1, len(parts), 3):
        mname, mzhi, body = parts[i], parts[i + 1], parts[i + 2]
        month = base.CN_MONTH.index(mname[:-1]) + 1
        ents = re.findall(r"\n([建除滿平定執破危成收開閉])(.)日[：:](.*?)(?=\n[建除滿平定執破危成收開閉].日[：:]|\n==|\Z)", body, re.S)
        for jc, z, text in ents:
            yield month, mzhi, jc, z, text


def strip_ying(text):
    """删去应验之语，留宜忌之文（分段规则同解析）"""
    t = re.sub(r"\s+", "", text)
    out = []
    for sent in re.split(r"(?<=[。！])", t):
        end = sent[-1] if sent and sent[-1] in "。！" else ""
        frags = re.split(r"([，,；;：:、])", sent[:-1] if end else sent)
        keep = ""
        for k in range(0, len(frags), 2):
            f = frags[k]
            sep = frags[k + 1] if k + 1 < len(frags) else ""
            if f and re.search(base.YING, f):
                continue
            keep += f + sep
        keep = re.sub(r"^[，、；：]+|[，、；：]+$", "", keep)
        if keep:
            out.append(keep + end)
    return "".join(out)


def build():
    """返回 (原文表行, 逐日判定初稿行)；各行为 dict，值皆为 str"""
    erows, drows = [], []
    for month, mzhi, jc, z, text in entries():
        full = re.sub(r"\s+", "", text)
        erows.append({"月": str(month), "月建": mzhi, "建除": jc, "日支": z, "宜忌之文": strip_ying(full), "原文": full, "校": ""})
        res = base.parse_entry(base.ZHI.index(z), text)
        for k in sorted(res, key=lambda k: (k % 10)):
            for e, v, c, b in collapse(res[k]):
                note = ""
                if c.endswith("……犯之……"):  # 对照脚本的推定：条内只举神煞与应验
                    c, note = c[: -len("……犯之……")], "条内只举神煞与犯之应验，无明文宜忌，推作诸事忌"
                drows.append({"月": str(month), "干支": base.gz_name(k), "建除": jc, "事": e, "宜忌": v, "等第": grade_of(c, v),
                              "泛言": "是" if b else "", "依据": basis(c) or c, "条件": "", "校": note})
    assert len(erows) == 144, len(erows)
    # 表以半角逗号分列，不加引号（Rust 端直接切分）
    assert not any("," in x or '"' in x for r in erows + drows for x in r.values())
    return erows, drows


def write(path, rows):
    with open(path, "w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()), lineterminator="\n")
        w.writeheader()
        w.writerows(rows)


def main():
    erows, drows = build()
    # 「校」列为人工条目级校记，重写时保留
    try:
        notes = {(r["月"], r["建除"]): r.get("校", "") for r in csv.DictReader(open(OUT_ENTRY, encoding="utf-8"))}
    except FileNotFoundError:
        notes = {}
    for r in erows:
        r["校"] = notes.get((r["月"], r["建除"]), "")
    write(OUT_ENTRY, erows)
    print(OUT_ENTRY, len(erows))
    if "--draft" in sys.argv:
        write(OUT_DRAFT, drows)
        print(OUT_DRAFT, len(drows), "日", len({(r["月"], r["干支"]) for r in drows}))


if __name__ == "__main__":
    main()
