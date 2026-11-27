"""《中山世譜》（《琉球王国汉文文献集成》第 3—6 册）、《球陽》（第 7—13 册）纪日与天象检验。

步骤（仓库根）：
  uv run --project scripts python -I scripts/calendar_data/ryukyu_shipu_qiuyang_check.py merge   # 合并录文分片 → data/
  uv run --project scripts python -I scripts/calendar_data/ryukyu_shipu_qiuyang_check.py          # 检验并生成文档

录文分片（三人分册识读，字段相同）放在 scratchpad，`merge` 合并为
  data/calendar/ryukyu_shipu_qiuyang.csv（纪日）、data/phenomena/ryukyu_tianxiang.csv（天象）。
检验：
- 纪日：转成 ryukyu_jiri_check.check_rows 的录文格式，与张表、当时行用之推步（明大统、清新法 / 考成 / 后编）比；
  二书为琉球官修后修史书，相同之月与之合者按 C 级记「有证」，互异之月单条不足以判（按 D 级，同月独立两条以上才判），归 ③。
  1368 年以前（元授时）本项目此处无推步表，只列张表，记「无张表或推步」。
- 天象：日食、月食以 rust/core/examples/historical_eclipses.rs 计算记录日附近有无食、首里（26.217°N，127.719°E）可否见；
  彗星、客星、星变只录不核。记录日由张表换算（琉球奉明清正朔）：书日干支者取该月内该干支之日，书日序者取第几日，
  只书月者日食取该月朔、月食取朔后 14 日为搜索中心（±1.5 日）。
"""
import csv
import subprocess
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))
import gaiyuan_date as g  # noqa: E402
import ryukyu_jiri_check as rj  # noqa: E402
import shiwu_fenlei as fl  # noqa: E402

GZ = fl.GZ
PARTS = Path("/tmp/claude-1000/-home-demo-calendar/3c749f40-3dd5-47f3-ac92-1126b726ce29/scratchpad/parts")
JIRI = ROOT / "data/calendar/ryukyu_shipu_qiuyang.csv"
TIANX = ROOT / "data/phenomena/ryukyu_tianxiang.csv"
OUT_CSV = ROOT / "data/calendar/ryukyu_shipu_qiuyang_check.csv"
OUT_MD = ROOT / "doc/calendar/ryukyu-shipu-qiuyang-checks.md"
SHURI = (127.719, 26.217)
JIRI_COLS = "编号,书,册,PDF页,书页,王代,年号,年号年,干支年,公元,月,闰,日,日干支,朔干支,类型,原文,备注".split(",")
TX_COLS = "编号,书,册,PDF页,书页,王代,年号,年号年,公元,月,闰,日,日干支,类别,原文,备注".split(",")


def read_part(p):
    lines = [l for l in open(p, encoding="utf-8")]
    heads = [l for l in lines if l.startswith("#")]
    return heads, list(csv.DictReader(l for l in lines if not l.startswith("#")))


def norm(x):
    """统一分片间写法不一之处（閏/闰、空闰位、星變/星变）。"""
    if x.get("闰") in ("閏", "闰"):
        x["闰"] = "1"
    if x.get("类型") == "閏":
        x["类型"] = "闰"
    if x.get("类型") == "闰" and not x.get("闰"):
        x["闰"] = "1"
    if "闰" in x and not x["闰"]:
        x["闰"] = "0"
    if x.get("类别") == "星變":
        x["类别"] = "星变"
    return x


def group(book):
    """分三组：《中山世譜》正文（蔡鐸本、蔡温本）、《中山世譜》附卷（记遣使萨摩、江户之日，或转记日本历）、《球陽》。"""
    if book.startswith("中山世譜附卷"):
        return "中山世譜附卷"
    if book.startswith("中山世譜"):
        return "中山世譜"
    return book


def merge():
    for kind, out, cols, desc in [("jiri", JIRI, JIRI_COLS, "纪日"), ("tianxiang", TIANX, TX_COLS, "天象")]:
        heads, rows = [], []
        for p in sorted(PARTS.glob(f"{kind}_v*.csv")):
            h, r = read_part(p)
            heads += h
            for x in r:
                assert set(cols) <= set(x), (p, x.keys())
            rows += [norm({c: x[c] for c in cols}) for x in r]
        ids = Counter(r["编号"] for r in rows)
        assert all(v == 1 for v in ids.values()), [k for k, v in ids.items() if v > 1]
        with open(out, "w", encoding="utf-8", newline="") as f:
            f.write(f"# 《中山世譜》（《琉球王國漢文文獻集成》复旦大学出版社 2013，第 3—6 册）、《球陽》（第 7—13 册）{desc}条录文。"
                    "PDF 无文字层，据网盘扫描本渲染图人工识读（2026-11-27，分册三人分读，未互校）；读不清作 □，推定者备注注明。\n")
            f.write("# 字段见 scripts/calendar_data/ryukyu_shipu_qiuyang_check.py；各册书页换算：\n")
            for h in heads:
                f.write("#   " + h.lstrip("# ").rstrip() + "\n")
            w = csv.DictWriter(f, fieldnames=cols)
            w.writeheader()
            w.writerows(rows)
        print(f"写 {out.relative_to(ROOT)}：{len(rows)} 条")


def rows(p):
    return list(csv.DictReader(l for l in open(p, encoding="utf-8") if not l.startswith("#")))


ITEM = {"朔": "朔干支", "日干支": "日干支", "闰": "闰月", "月大小": "月大"}


def to_check_format(R):
    out = []
    for r in R:
        item = ITEM.get(r["类型"], "无可验")
        if item == "日干支" and not r["日干支"]:
            item = "无可验"
        val = ""
        if item == "闰月":
            val = f"闰{r['月']}" if r["月"] else ""
        elif item == "月大":
            val = "大"
        out.append(dict(编号=r["编号"], 篇名=f"{r['书']}·{r['王代']}", 年号=r["年号"], 年号年=r["年号年"], 干支年=r["干支年"],
                        农历年=r["公元"], 月=r["月"], 闰=r["闰"] or "0", 日=r["日"], 日干支=r["日干支"], 朔干支=r["朔干支"],
                        验项=item, 书值=val, 原文=r["原文"], 册=r["册"], PDF页=r["PDF页"], 书页=r["书页"], 备注=r["备注"],
                        # 蔡鐸本与蔡温本（蔡温据蔡鐸本重修）同年同月同项之记载同出一源
                        史源=f"中山世譜正文{r['公元']}-{r['月']}-{r['类型']}" if group(r["书"]) == "中山世譜" else ""))
    return out


def record_day(r):
    """张表换算记录日（JDN）；(jdn 或 None, 搜索中心, 说明)"""
    if not r["公元"] or not r["月"]:
        return None, None, "年月不明"
    k = (int(r["公元"]), int(r["月"]), int(r["闰"] or 0))
    s = rj.zhang().get(k)
    if s is None:
        return None, None, "张表无此月"
    if r["日干支"] in GZ:
        off = (GZ.index(r["日干支"]) - (s + 49)) % 60
        if off < 30:
            return s + off, s + off, f"该月第 {off + 1} 日"
        return None, s, f"{r['日干支']}不在该月"
    if r["日"].isdigit():
        return s + int(r["日"]) - 1, s + int(r["日"]) - 1, f"第 {r['日']} 日"
    return None, s if r["类别"] == "日食" else s + 14, "只书月"


def eclipse_inputs(T):
    paths = {}
    for kind, cat in [("solar", "日食"), ("lunar", "月食")]:
        p = ROOT / f"data/phenomena/ryukyu_{kind}_eclipses.csv"
        with open(p, "w", encoding="utf-8", newline="") as f:
            w = csv.writer(f)
            w.writerow(["id", "year", "month", "leap", "ganzhi", "jdn", "guess_jdn", "site_lon", "site_lat"])
            for r in T:
                if r["类别"] != cat:
                    continue
                j, c, _ = record_day(r)
                if c is None:
                    continue
                w.writerow([r["编号"], r["公元"], r["月"], r["闰"] or 0, r["日干支"], j if j is not None else "", c, *SHURI])
        paths[kind] = p
    return paths


def run_eclipses(paths):
    res = {}
    for kind, p in paths.items():
        out = p.with_name(p.stem + "_check.csv")
        subprocess.run(["cargo", "run", "-q", "--release", "-p", "lunar-core", "--example", "historical_eclipses",
                        "--no-default-features", "--features", "real-f64", "--", kind, str(p), str(out), str(SHURI[0]), str(SHURI[1])],
                       cwd=ROOT / "rust", check=True)
        for r in rows(out):
            res[r["id"]] = (kind, r)
    return res


def tianxiang_verdict(r, ecl):
    if r["类别"] not in ("日食", "月食"):
        return "只录不核", ""
    if r["编号"] not in ecl:
        return "无从计算", record_day(r)[2]
    kind, e = ecl[r["编号"]]
    v = e.get("verdict", "")
    if not e.get("greatest_tt_jd"):
        return "未发生", "记录日 ±1.5 日内无食"
    mag = e.get("site_magnitude_0", "") or e.get("site_magnitude_0s", "")
    note = f"{e.get('kind', '')}，食甚 JD(TT) {e['greatest_tt_jd']}，日差 {e.get('day_offset', '')}，首里食分 {mag}"
    if "可见与否取决" in v:
        return "发生、首里可见与否取决于 ΔT", note
    if "不可见" in v or mag in ("0", "0.000", "0.0"):
        return "发生但首里不可见", note
    return "合（发生且首里可见）", note


def main():
    if sys.argv[1:] == ["merge"]:
        merge()
        return
    R = rows(JIRI)
    lines, cats = rj.check_rows(to_check_format(R), "世譜球陽", grade_same="C", grade_diff="D", what="后修史书")
    for x in lines:
        x["史书"] = next(r["书"] for r in R if r["编号"] == x["编号"])
    T = rows(TIANX)
    ecl = run_eclipses(eclipse_inputs(T))
    tx = []
    for r in T:
        v, note = tianxiang_verdict(r, ecl)
        tx.append(dict(r, 结论=v, 计算=note))
    with open(OUT_CSV, "w", encoding="utf-8", newline="") as fh:
        fh.write("# 《中山世譜》《球陽》纪日与张表、推步比较（scripts/calendar_data/ryukyu_shipu_qiuyang_check.py 生成，勿手改）。\n")
        w = csv.DictWriter(fh, fieldnames=list(lines[0].keys()))
        w.writeheader()
        w.writerows(lines)
    by_book = {}
    for b in ("中山世譜", "中山世譜附卷", "球陽"):
        ls = [x for x in lines if group(x["史书"]) == b]
        by_book[b] = ls
    jcols = ["编号", "篇名", "纪年", "干支年", "农历年", "月", "日", "日干支", "验项", "书", "推步法", "推步", "张表", "月内", "公历", "分类",
             "出处", "原文", "备注"]
    tcols = ["编号", "书", "王代", "年号", "年号年", "公元", "月", "闰", "日", "日干支", "类别", "结论", "计算", "原文", "备注"]
    md = ["# 《中山世譜》《球陽》纪日与天象检验（自动生成）", "",
          "> 由 `scripts/calendar_data/ryukyu_shipu_qiuyang_check.py` 生成，勿手改。录文 `data/calendar/ryukyu_shipu_qiuyang.csv`（纪日）、"
          "`data/phenomena/ryukyu_tianxiang.csv`（天象），逐条结果 `data/calendar/ryukyu_shipu_qiuyang_check.csv`、"
          "`data/phenomena/ryukyu_{solar,lunar}_eclipses_check.csv`；书目、证据等级与结论见 [ryukyu-sources.md](ryukyu-sources.md)。", "",
          "## 方法", "",
          "- 纪日：同 [ryukyu-jiri-checks.md](ryukyu-jiri-checks.md)（张表与当时行用之推步：明大统、清新法 / 考成 / 后编；康熙五至八年复用大统，月序闰位取大统本身推步）。1368 年以前无推步表，只列张表。同一史源（蔡鐸本、蔡温本同文）只算一条。",
          "- " + fl.RULE + " 二书为琉球官修后修史书：相同之月与之合者按 C 级记「有证」；互异之月同月独立之条不足两条者不判，归 ③ 并注倾向。",
          "- 天象：日食、月食以 `historical_eclipses` 计算记录日 ±1.5 日内有无食、首里可否见（ΔT 取 Morrison–Stephenson 2021，±σ、±2σ 各算）；"
          "记录日由张表换算。彗星、客星、星变只录不核。", "", "## 汇总", ""]
    for b, ls in by_book.items():
        c = [(x["分类"].split("；")[0], "C") for x in ls if x["分类"].split("；")[0] in fl.CATS]
        n_none = sum(x["分类"] == "无可验" for x in ls)
        md.append(f"- {b}纪日 {len(ls)} 条，可验 {len(ls) - n_none}：" + fl.headline(c, "推步", "项") + "。")
    tc = Counter((r["书"], r["类别"], r["结论"]) for r in tx)
    for (b, k, v), n in sorted(tc.items()):
        md.append(f"- {b}{k}：{v} {n} 条")
    md += ["", "## 纪日逐条", "", "| " + " | ".join(jcols) + " |", "|" + "---|" * len(jcols)]
    md += ["| " + " | ".join(str(x.get(c, "")).replace("|", "／") for c in jcols) + " |" for x in lines]
    md += ["", "## 天象逐条", "", "| " + " | ".join(tcols) + " |", "|" + "---|" * len(tcols)]
    md += ["| " + " | ".join(str(x.get(c, "")).replace("|", "／") for c in tcols) + " |" for x in tx]
    OUT_MD.write_text("\n".join(md) + "\n", encoding="utf-8")
    print("\n".join(md[md.index("## 汇总") + 2:md.index("## 纪日逐条")]))


if __name__ == "__main__":
    main()
