#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""交叉比对 vendor/calendar/chinese_calendar/年号表.csv 与 data/calendar/fang_shiming/*.csv，
按年份/政权对齐后比较年号与年数是否一致，输出分类后的 Markdown 报告。

背景：两个来源目前完全独立、从未互相校验（见 data/calendar/SOURCES_era_regime.md、
data/calendar/fang_shiming/README.md）。本脚本是"多来源交叉校对出汇总表"这条路线的第一步摸底工具，
只读不改，帮助估计两边实际分歧有多大、分歧集中在哪些年份/政权。

用法（工作目录为仓库根）：
  uv run --project scripts python scripts/calendar_data/compare_era_sources.py [输出路径.md]
  不给输出路径时默认写到 data/calendar/汇总/校勘调研_年号表_vs_方诗铭.md

已知局限（详见输出报告「方法说明」一节）：
- 年号表.csv 每年只有一条「政权主线」记录，fang_shiming 同一时期可能有多个政权并列列；
  本脚本按年份区间把年号表.csv 映射到 fang_shiming 的某一列，映射规则是本脚本自定的近似规则，
  朝代交接年份边界可能不准。
- fang_shiming 单元格是原书原样转录（含即位月份圈码、人名括注、〜承前省略等），本脚本用正则粗略
  剥离后取「年号文字」与「年数」比较，不追求 100% 精确解析。
- 只覆盖两边都有数据、且能建立映射的范围（约公元前206～公元1911年），不含十二诸侯/战国/秦
  （早于年号表.csv 覆盖范围）和民国以后（年号表.csv 不再记录帝王年号）。
"""
from __future__ import annotations

import csv
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

CN_DIGITS = {"零": 0, "一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}


def cn_year_to_int(s: str) -> int | None:
    """年号表.csv 的 era_year_simp 列是中文数字（如"元年""十一载"），转为整数年数。"""
    s = s.strip()
    if not s:
        return None
    if s in ("元年", "元载"):
        return 1
    s = s.rstrip("年载")  # 唐玄宗天宝年间一度改"年"为"载"
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


def load_era_names() -> dict[int, dict]:
    path = ROOT / "vendor/calendar/chinese_calendar/年号表.csv"
    by_year: dict[int, dict] = {}
    with open(path, encoding="utf-8") as f:
        for row in csv.DictReader(f):
            by_year[int(row["lunar_year"])] = row
    return by_year


CIRCLED = "".join(chr(c) for c in list(range(0x2460, 0x246A)) + list(range(0x3280, 0x328A)))
CIRCLE_RE = re.compile(f"[{re.escape(CIRCLED)}]")


def strip_circles(s: str) -> str:
    return CIRCLE_RE.sub("", s)


def load_fs(fname: str) -> dict[int, dict]:
    path = ROOT / "data/calendar/fang_shiming" / f"{fname}.csv"
    with open(path, encoding="utf-8") as f:
        rows = list(csv.DictReader(f))
    return {int(r["year"]): r for r in rows}


def parse_cell(cell: str) -> tuple[str | None, int | None]:
    """解析 fang_shiming 单元格文本，返回 (年号文字, 年数)。多段（"/" 或换行分隔）取最后一段。"""
    cell = cell.strip()
    if not cell:
        return None, None
    segs = [s.strip() for s in re.split(r"[/\n]", cell) if s.strip()]
    if not segs:
        return None, None
    seg = segs[-1]
    seg = strip_circles(seg)
    seg = seg.replace("闰", "")
    seg = re.sub(r"[（(][^（）()]*[）)]", "", seg)  # 括号人名/月份注记
    seg = seg.replace("〔", "").replace("〕", "")
    seg = seg.strip()
    m = re.search(r"(\d+)\s*$", seg)
    year_num = int(m.group(1)) if m else None
    name_part = seg[: m.start()] if m else seg
    name_part = name_part.strip(" ~　*")
    return (name_part if name_part else None), year_num


def forward_fill_column(rows_by_year: dict[int, dict], col: str) -> dict[int, tuple]:
    """原书对同一年号的延续年份不重复印年号文字，只印递增年数；此处前向填充年号文字状态。

    只在"年数=1"（即真正新年号开始的那一行）时更新当前年号文字；年数>1 的行即使文字段非空
    （常见于原书在延续行里重新印一次帝号方便阅读，如"世宗(柴荣) 2"），也不当作新年号，
    避免把帝号误判成年号覆盖掉之前正确记录的年号。
    """
    out: dict[int, tuple] = {}
    current_name = None
    for y in sorted(rows_by_year):
        cell = rows_by_year[y].get(col, "")
        if cell is None or cell.strip() == "":
            continue
        name, num = parse_cell(cell)
        if name and (num == 1 or num is None or current_name is None):
            current_name = name
        out[y] = (current_name, num)
    return out


def build_year_source_map() -> tuple[dict[int, tuple], set[int]]:
    """年份 -> (fang_shiming 文件, 列名) 的近似映射；边界年份用于后续把不一致分类为"朝代交接"。"""
    m: dict[int, tuple] = {}
    boundary_years: set[int] = set()

    def add(lo: int, hi: int, fname: str, col: str) -> None:
        for y in range(lo, hi + 1):
            m[y] = (fname, col)
        boundary_years.add(lo)
        boundary_years.add(hi)

    add(-206, 220, "04_汉纪年表", "汉")
    add(221, 264, "05_三国纪年表", "魏")
    add(265, 280, "05_三国纪年表", "晋")
    add(281, 419, "06_晋及十六国纪年表", "晋")
    add(420, 478, "07_南北朝纪年表", "宋")
    add(479, 501, "07_南北朝纪年表", "齐")
    add(502, 556, "07_南北朝纪年表", "梁")
    add(557, 589, "07_南北朝纪年表", "陈")
    add(590, 618, "08_隋纪年表", "隋")
    add(619, 690, "09_唐纪年表", "唐")
    add(691, 705, "09_唐纪年表", "武周")
    add(706, 906, "09_唐纪年表", "唐")
    add(907, 923, "10_五代十国纪年表", "后梁")
    add(924, 936, "10_五代十国纪年表", "后唐")
    add(937, 946, "10_五代十国纪年表", "后晋")
    add(947, 950, "10_五代十国纪年表", "后汉")
    add(951, 960, "10_五代十国纪年表", "后周")
    add(961, 1279, "11_宋辽金纪年表", "宋")
    add(1280, 1368, "12_元纪年表", "元")
    add(1369, 1644, "13_明纪年表", "明")
    add(1645, 1911, "14_清纪年表", "清")
    return m, boundary_years


def era_lookup_year(hist_year: int) -> int:
    """年号表.csv 用天文纪年（含公元0年=公元前1年）；fang_shiming 等用历史纪年（无公元0年）。
    hist_year（fang_shiming 的 year 列）-> 年号表.csv 的 lunar_year。公元后（含1）不变，公元前 +1。
    """
    return hist_year if hist_year >= 1 else hist_year + 1


VARIANT_PAIRS = [("佑", "祐")]  # 已知异体字/通用替换字对，非实质分歧


def normalize_variant(s: str) -> str:
    for a, b in VARIANT_PAIRS:
        s = s.replace(a, b)
    return s


def main() -> None:
    out_md = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "data/calendar/汇总/校勘调研_年号表_vs_方诗铭.md"

    era = load_era_names()
    ysrc, boundary_years = build_year_source_map()

    fs_cache: dict[str, dict] = {}
    ff_cache: dict[tuple, dict] = {}

    total = compared = agree = no_fs_data = no_era_data = 0
    mismatches: list[dict] = []

    for y in range(-106, 1912):
        if y == 0:
            continue
        erow = era.get(era_lookup_year(y))
        if erow is None or not erow["regime_simp"]:
            no_era_data += 1
            continue
        total += 1
        src = ysrc.get(y)
        if not src:
            continue
        fname, col = src
        if fname not in fs_cache:
            fs_cache[fname] = load_fs(fname)
        if (fname, col) not in ff_cache:
            ff_cache[(fname, col)] = forward_fill_column(fs_cache[fname], col)
        ff = ff_cache[(fname, col)]
        if y not in ff:
            no_fs_data += 1
            continue
        fs_name, fs_year = ff[y]
        era_name = erow["era_name_simp"]
        era_year_int = cn_year_to_int(erow["era_year_simp"])

        compared += 1
        # 年号表.csv 的 era_name_simp 在政权/朝代交接的改元年有时会把朝代名也拼进去
        # （如"宋元徽"、"元文宗天历"），核心年号文字通常是其结尾 2 字，故额外按结尾子串比对一次。
        if fs_name and era_name:
            name_match = (era_name in fs_name) or (fs_name in era_name)
            if not name_match and len(era_name) > 2:
                name_match = any(era_name[-k:] in fs_name for k in (2, 3) if k <= len(era_name))
        else:
            name_match = None
        year_match = (fs_year == era_year_int) if (fs_year is not None and era_year_int is not None) else None
        ok = (name_match is not False) and (year_match is not False)

        if ok:
            agree += 1
            continue

        category = "genuine"
        if name_match is False and fs_name and era_name:
            n1, n2 = normalize_variant(fs_name), normalize_variant(era_name)
            if n1 == n2 or n1 in n2 or n2 in n1:
                category = "variant_char"
        if category == "genuine" and (y in boundary_years or (y - 1) in boundary_years or (y + 1) in boundary_years):
            category = "boundary"
        if category == "genuine" and (fs_year is None or era_year_int is None):
            category = "parse_limitation"

        mismatches.append(
            {
                "year": y,
                "regime": erow["regime_simp"],
                "file": fname,
                "col": col,
                "era_names_csv": f"{erow['emperor_simp']}{era_name}{erow['era_year_simp']}",
                "fang_shiming": f"{fs_name}{fs_year}",
                "category": category,
            }
        )

    by_cat: dict[str, list[dict]] = {}
    for m in mismatches:
        by_cat.setdefault(m["category"], []).append(m)

    cat_names = {
        "variant_char": "异体字/同名不同写法（如「佑/祐」）——不是实质分歧",
        "boundary": "朝代交接边界年——两边分别记录各自政权末年/首年，非真正矛盾（需人工核对具体是否矛盾）",
        "parse_limitation": "本次摸底脚本解析局限（多段/多行单元格未完全拆解等）——需要更细致的解析才能判断",
        "genuine": "其余——需要逐条人工核实是否为真实史料分歧",
    }

    lines = [
        "# 年号表.csv vs fang_shiming 转录 —— 交叉比对摸底报告",
        "",
        "生成方式：`scripts/calendar_data/compare_era_sources.py` 脚本比对，仅覆盖两边都有数据、且能建立"
        "年份↔政权列映射的范围（约公元前206～公元1911年，不含十二诸侯/战国/秦——早于年号表.csv 覆盖范围，"
        "也不含民国及以后——年号表.csv 不再记录帝王年号）。只读，不改任何数据文件。",
        "",
        "## 一、总体结果",
        "",
        f"- 年号表.csv 有记录的年份：{total}",
        f"- 落在本次映射范围内、双方都有数据、实际参与比对：{compared}",
        f"- 一致：{agree}（{agree / compared * 100:.1f}%）",
        f"- 不一致：{len(mismatches)}（{len(mismatches) / compared * 100:.1f}%）",
        f"- fang_shiming 该列当年无数据（未建立/已灭亡/原书留空，不计入比对）：{no_fs_data}",
        "",
        "## 二、不一致分类",
        "",
    ]
    for cat in ["variant_char", "boundary", "parse_limitation", "genuine"]:
        items = by_cat.get(cat, [])
        lines.append(f"### {cat_names[cat]}：{len(items)} 条")
        lines.append("")
        if not items:
            continue
        lines.append("| 年份 | 政权 | 来源 | 年号表.csv | fang_shiming |")
        lines.append("|---|---|---|---|---|")
        for m in items:
            lines.append(f"| {m['year']} | {m['regime']} | {m['file']}/{m['col']} | {m['era_names_csv']} | {m['fang_shiming']} |")
        lines.append("")

    lines += [
        "## 三、方法说明与已知局限",
        "",
        "- 年号表.csv 用天文纪年（含公元0年），fang_shiming/本仓库其余数据用历史纪年（无公元0年）；本次比对已做该换算。",
        "- fang_shiming 单元格为原书原样转录（含即位月份圈码、人名括注等），本次用规则粗略剥离后取「年号文字」与「年数」比较，"
        "不追求 100% 精确解析，粗解析导致的误判归入「本次摸底脚本解析局限」类。",
        "- 年号表.csv 每年只有一条「政权主线」记录（如三国期间只有魏），fang_shiming 同一时期有多个政权并列列；"
        "本次按年份区间把年号表.csv 映射到 fang_shiming 对应的单一列，映射本身是本次摸底自定的近似规则，"
        "朝代交接年份的边界选择可能不准确，归入「朝代交接边界年」类。",
        "- 本报告只统计「能对上号」的比较结果，不代表 fang_shiming 或 年号表.csv 各自的整体正确性；"
        "也没有引入第三方源（如李崇智《中国历代年号考》）做进一步裁决。",
        "",
        "## 四、后续",
        "",
        "这是「多来源交叉校对出汇总表」这条路线的摸底步骤，尚未做任何取舍/裁决。下一步需要：",
        "1. 对上面「其余」类逐条判断是本次脚本解析问题还是真实史料分歧；",
        "2. 真实分歧的年份/政权引入第三方来源（见 SOURCES_era_regime.md）辅助裁决；",
        "3. 设计一份干净的结构化汇总表格式（供程序解析，而非现在 regime_era_calendar_matrix.csv 那种拼接展示"
        "字符串的格式），把裁决结果写进去；",
        "4. 把 `LifaRegistry`/`EraNames` 等的加载路径从 vendor 原始文件切到裁决后的汇总表。",
    ]

    out_md.parent.mkdir(parents=True, exist_ok=True)
    out_md.write_text("\n".join(lines), encoding="utf-8")
    print(f"Wrote {out_md}")
    print(
        f"一致 {agree}/{compared} ({agree / compared * 100:.1f}%)；不一致 {len(mismatches)}，分类："
        + ", ".join(f"{cat}={len(by_cat.get(cat, []))}" for cat in ["variant_char", "boundary", "parse_limitation", "genuine"])
    )


if __name__ == "__main__":
    main()
