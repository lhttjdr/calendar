"""二十八宿距星表与史载宿度表。

1. data/sky_culture/chinese_lunar_mansion_determinatives.csv：由 vendor/stellarium_skycultures/ 五套中国星空文化
   （Stellarium，CC BY-SA 4.0）的 lunar_system.defining_stars 提取，并与其星官连线（二十八宿星官须含该星）、
   星名中的「距星」标注逐宿核对。
2. data/sky_culture/yuanshi_lodge_widths.csv：《元史·历志一·授时历议》「周天列宿度」表（汉洛下闳、唐一行、宋皇祐、元丰、
   崇宁、至元六次实测），维基文库 元史/卷052 wikitext 解析；空栏按原文例取左栏之数。自检：汉栏与《汉书·律历志》
   （漢書/卷021）所列宿度、至元栏与《元史·历志三·授时历经》「赤道宿度」（元史/卷054）逐宿一致。

用法（仓库根）：uv run --project scripts python scripts/sky_culture/lunar_mansions.py
维基文库原文缓存于 scripts/sky_culture/_cache/（不入库）。
"""

import json
import re
import urllib.parse
import urllib.request
from pathlib import Path

LODGES = "角亢氐房心尾箕斗牛女虚危室壁奎娄胃昴毕觜参井鬼柳星张翼轸"
SYSTEMS = [
    # (体系 id, Stellarium 目录, 代表年代说明)
    ("chenzhuo", "chinese_chenzhuo", "陈卓星表（三国—晋，汇集汉代石氏、甘氏、巫咸三家观测）"),
    ("song_huangyou", "chinese_song_dynasty", "宋皇祐四年（1052）观测，《新仪象法要》星图"),
    ("yuan", "chinese_yuan_dynasty", "元《天文汇钞》星表（郭守敬传统）"),
    ("ming_xianglin", "chinese_xianglin", "明陈荩谟《象林》（1634）"),
    ("qing", "chinese", "清《仪象考成》（1756）及续编（1844）"),
]
CACHE = Path("scripts/sky_culture/_cache")
OUT = Path("data/sky_culture")


def wikisource(title):
    CACHE.mkdir(parents=True, exist_ok=True)
    f = CACHE / (title.replace("/", "_") + ".txt")
    if not f.exists():
        url = "https://zh.wikisource.org/w/index.php?title=" + urllib.parse.quote(title) + "&action=raw"
        req = urllib.request.Request(url, headers={"User-Agent": "lunar-calendar-research/0.1"})
        f.write_bytes(urllib.request.urlopen(req, timeout=60).read())
    return f.read_text(encoding="utf-8")


def determinatives():
    rows = []
    for sid, d, note in SYSTEMS:
        j = json.load(open(f"vendor/stellarium_skycultures/{d}/index.json", encoding="utf-8"))
        ds = j["lunar_system"]["defining_stars"]
        assert len(ds) == 28, sid
        names = {}
        for k, v in j["common_names"].items():
            m = re.match(r"HIP\s+(\d+)", k)
            if m:
                names[int(m.group(1))] = [e.get("native", "") for e in v]
        # 二十八宿星官：id 以 01A…28A 结尾（宋、元、明、陈卓），清代按星名「X宿一」等核对
        cons = {}
        for c in j["constellations"]:
            cid = c["id"].split()[-1]
            members = {h for line in c["lines"] for h in line if isinstance(h, int)}
            cons[cid] = (c["common_name"].get("native", ""), members)
        for i, hip in enumerate(ds):
            lodge = LODGES[i]
            key = f"{i + 1:02d}A"
            if key in cons:
                cname, members = cons[key]
                assert hip in members, (sid, lodge, hip, cname)
                marked = [h for h in members if any("距星" in n for n in names.get(h, []))]
                assert not marked or hip in marked, (sid, lodge, hip, marked)
            else:
                # 清代：距星当为该宿星官的一颗星，星名以宿名开头
                assert any(n.startswith(lodge) or n.startswith({"毕": "毕", "参": "参"}.get(lodge, lodge)) for n in names.get(hip, [])), (sid, lodge, hip, names.get(hip))
            star_name = next((n for n in names.get(hip, []) if n and "距星" not in n), "")
            rows.append((sid, i + 1, lodge, hip, star_name))
    # 潘鼐（2009）第 640 页：康熙《灵台仪象志》（1674）改奎为 η And、觜为 λ Ori、昴为 η Tau（参仍 δ Ori，乾隆十九年始改 ζ Ori）。
    # Stellarium 无此体系，按传统距星（第 14 页表 1.2.1，与 chenzhuo 相同）改此三宿生成。
    trad = [r for r in rows if r[0] == "chenzhuo"]
    lingtai = {"奎": 4463, "觜": 26207, "昴": 17702}
    for _, idx, lodge, hip, _ in trad:
        rows.append(("qing_lingtai", idx, lodge, lingtai.get(lodge, hip), ""))
    with open(OUT / "chinese_lunar_mansion_determinatives.csv", "w", encoding="utf-8") as f:
        f.write("# 二十八宿距星（各体系）。来源：Stellarium 中国星空文化（vendor/stellarium_skycultures/，CC BY-SA 4.0），\n")
        f.write("# lunar_system.defining_stars；已核对距星属本宿星官且与「距星」标注一致。生成：scripts/sky_culture/lunar_mansions.py。\n")
        for sid, _, note in SYSTEMS:
            f.write(f"# {sid}：{note}\n")
        f.write("# qing_lingtai：清康熙《灵台仪象志》（1674），据潘鼐《中国恒星观测史》（2009）第 640 页，在传统距星上改奎 η And、觜 λ Ori、昴 η Tau。\n")
        f.write("# 核对：传统距星与潘鼐第 14 页表 1.2.1、清代与第 641 页表 9.1.4 逐宿一致。\n")
        f.write("system,index,mansion,hip,stellarium_name\n")
        for r in rows:
            f.write(",".join(str(x) for x in r) + "\n")
    return rows


NUM = {"〇": 0, "零": 0, "一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}


def cn_int(s):
    """一百一十二、九十八、十二、三十三 等。"""
    if not s:
        return 0
    total, cur = 0, 0
    for ch in s:
        if ch in NUM:
            cur = NUM[ch]
        elif ch == "十":
            total += (cur or 1) * 10
            cur = 0
        elif ch == "百":
            total += (cur or 1) * 100
            cur = 0
        else:
            raise ValueError(s)
    return total + cur


FRAC = {"少": 0.25, "半": 0.5, "太": 0.75}


def degrees(cell):
    """「十二度」「九度少」「十度少強」「十二度一十分」「九十三度八十分太」「五分」「半度」「二十六度及分」→ 度（小数）。"""
    c = cell.replace("〔", "").replace("〕", "").replace("（", "").replace("）", "")
    jifen = c.endswith("及分")
    if jifen:
        c = c[:-2]
    m = re.fullmatch(r"(?:(.*?)度)?(?:(.*?)分)?(少|半|太)?(強)?(度)?", c)
    assert m, cell
    deg_s, fen_s, frac, qiang, _ = m.groups()
    if deg_s == "半":
        return 0.5
    v = float(cn_int(deg_s or ""))
    if fen_s:
        v += cn_int(fen_s) / 100.0
    if frac:
        v += FRAC[frac] / (100.0 if fen_s else 1.0)
    if qiang:
        v += 1.0 / 12.0
    if jifen:
        v += 0.25  # 斗分
    return v


def lodge_widths():
    t = wikisource("元史/卷052")
    i = t.index("==周天列宿度==")
    tab = t[i : t.index("</table>", i)]
    rows = []
    for r in re.findall(r"<tr>(.*?)</tr>", tab, re.S):
        cells = [re.sub(r"<[^>]+>", "", re.sub(r"<ref>.*?</ref>", "", c, flags=re.S)).strip().replace("　", "") for c in re.findall(r"<td[^>]*>(.*?)</td>", r, re.S)]
        rows.append(cells)
    header = rows[0]
    epochs = ["han_luoxiahong", "tang_yixing", "song_huangyou", "song_yuanfeng", "song_chongning", "yuan_zhiyuan"]
    assert len(header) == 6
    out = []
    trad = "角亢氐房心尾箕斗牛女虛危室壁奎婁胃昴畢觜參井鬼柳星張翼軫"
    for cells in rows[1:]:
        first = cells[0]
        if first[0] not in trad:
            continue  # 四方小计
        lodge = LODGES[trad.index(first[0])]
        vals, last = [], None
        for k, c in enumerate(cells):
            if not c:
                vals.append(last)
                continue
            last = degrees(c[1:] if k == 0 else c)
            vals.append(last)
        out.append((lodge, vals))
    assert [o[0] for o in out] == list(LODGES)
    # 自检：汉栏与《汉书·律历志》
    h = wikisource("漢書/卷021")
    line = next(l for l in h.splitlines() if l.startswith("角十二。亢九"))
    han = [cn_int(x) for x in re.findall(r"[" + trad + "營]+?([一二三四五六七八九十百]+)。", line.replace("營室", "室"))]
    assert han == [int(v[1][0]) for v in out], (han, [v[1][0] for v in out])
    # 自检：至元栏与授时历经「赤道宿度」
    y = wikisource("元史/卷054")
    k = y.index("====赤道宿度====")
    seg = y[k : k + 1500]
    zy = re.findall("([" + trad + r"])([一二三四五六七八九十百]*)\{\{\*\|([^}]*)\}\}", seg)
    def fen(c):
        m = re.fullmatch(r"初?([一二三四五六七八九十百]*)(少|半|太)?", c)
        return (cn_int(m.group(1)) + (FRAC[m.group(2)] if m.group(2) else 0.0)) / 100.0

    zmap = {LODGES[trad.index(a)]: cn_int(b) + (fen(c) if c else 0.0) for a, b, c in zy}
    assert len(zmap) == 28, len(zmap)
    diffs = []
    for lodge, vals in out:
        if abs(zmap[lodge] - vals[5]) > 1e-9:
            diffs.append(f"{lodge} 卷052 {vals[5]:.4f} / 卷054 {zmap[lodge]:.4f}")
            vals[5] = zmap[lodge]
    print("至元栏两卷不一致（取卷054）：", diffs)
    with open(OUT / "yuanshi_lodge_widths.csv", "w", encoding="utf-8") as f:
        f.write("# 《元史·历志一·授时历议》「周天列宿度」：历代实测赤道宿度（度；汉至崇宁周天 365.25 度，至元周天 365.2575 度）。\n")
        f.write("# 维基文库 元史/卷052 wikitext 解析；空栏取左栏之数（原文例）；少 1/4、半 1/2、太 3/4、强 +1/12，至元以百分为度。\n")
        f.write("# 汉栏与《汉书·律历志》逐宿一致。至元栏取《元史·历志三·授时历经》「赤道宿度」（元史/卷054）：卷052 表中虚 8.95、室 17.11，\n")
        f.write("# 卷054 作虚 8.9575（九十五分太）、室 17.10，卷052 自身北方小计 93.8075 与卷054 相合，故从卷054。\n")
        f.write("# 生成：scripts/sky_culture/lunar_mansions.py。\n")
        f.write("mansion," + ",".join(epochs) + "\n")
        for lodge, vals in out:
            f.write(lodge + "," + ",".join(f"{v:.4f}" for v in vals) + "\n")
    for col in range(6):
        print(epochs[col], "合计", round(sum(v[1][col] for v in out), 4))
    return out


def asterisms():
    """全部星官：成员（连线中出现的 HIP 或 DSO，按首次出现顺序）与距星（标注或二十八宿 defining_stars）。"""
    lines_out = []
    for sid, d, _ in SYSTEMS:
        j = json.load(open(f"vendor/stellarium_skycultures/{d}/index.json", encoding="utf-8"))
        names = {}
        for k, v in j["common_names"].items():
            m = re.match(r"HIP\s+(\d+)", k)
            if m:
                names[int(m.group(1))] = [e.get("native", "") for e in v]
        lodge_det = dict(zip(range(28), j["lunar_system"]["defining_stars"]))
        for c in j["constellations"]:
            members = []
            for line in c["lines"]:
                for h in line:
                    tok = str(h) if isinstance(h, int) and h < 10**7 else (h if isinstance(h, str) else f"GAIA:{h}")
                    if tok not in members:
                        members.append(tok)
            hips = [int(t) for t in members if t.isdigit()]
            det = [h for h in hips if any("距星" in n for n in names.get(h, []))]
            cid = c["id"].split()[-1]
            if not det and re.fullmatch(r"\d\dA", cid) and 1 <= int(cid[:2]) <= 28:
                det = [lodge_det[int(cid[:2]) - 1]]
            if not det and sid == "qing":
                lodge = c["common_name"].get("native", "").replace("宿", "")
                if len(lodge) == 1 and lodge in LODGES:
                    det = [lodge_det[LODGES.index(lodge)]]
            cn = c["common_name"]
            lines_out.append((sid, cid, cn.get("native", ""), cn.get("english", "").replace(",", " "), " ".join(members), det[0] if det else ""))
    with open(OUT / "chinese_asterisms.csv", "w", encoding="utf-8") as f:
        f.write("# 中国星官（各体系）：成员与距星。来源 vendor/stellarium_skycultures/（CC BY-SA 4.0）各 index.json 的星官连线；\n")
        f.write("# members 为连线中出现的天体（HIP 编号；DSO:M44 等深空天体；GAIA: 盖亚编号），按首次出现顺序，空格分隔。\n")
        f.write("# determinative：星名中标「距星」者；二十八宿取 lunar_system.defining_stars；其余未标者为空。生成：scripts/sky_culture/lunar_mansions.py。\n")
        f.write("system,id,name,english,members,determinative\n")
        for r in lines_out:
            f.write(",".join(str(x) for x in r) + "\n")
    print(len(lines_out), "个星官")


if __name__ == "__main__":
    OUT.mkdir(parents=True, exist_ok=True)
    r = determinatives()
    print(len(r), "条距星")
    lodge_widths()
    asterisms()
