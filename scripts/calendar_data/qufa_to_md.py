"""由 data/calendar/main_series_qufa.csv（rust/core/examples/main_series_qufa.rs 生成）写《历代历法总录》的「主序列逐段取法」表。

知识库 doc/calendar/calendar-compendium.md 与读者站 site/calendar/compendium.md 各有一对标记
`<!-- qufa:begin -->` / `<!-- qufa:end -->`，两标记之间整块重写；标记外不动。两处写同一张表，只是读者站不写代码名。
取法以第 4 层仲裁实际所给来源为准（不手写），免得代码、知识库、读者站三处不同步。

同一历名相邻各段（中间只隔实物历日之岁）合为一行，实物历日之岁另计岁数。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/qufa_to_md.py
"""
import csv
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BEGIN, END = "<!-- qufa:begin -->", "<!-- qufa:end -->"
TARGETS = [ROOT / "doc/calendar/calendar-compendium.md", ROOT / "site/calendar/compendium.md"]


def yr(y: int) -> str:
    return f"前 {1 - y}" if y <= 0 else str(y)


def span(a: int, b: int) -> str:
    return yr(a) if a == b else f"{yr(a)}—{yr(b)}"


def load():
    with open(ROOT / "data/calendar/main_series_qufa.csv", encoding="utf-8") as f:
        return list(csv.DictReader(line for line in f if not line.startswith("#")))


def groups(rows):
    """按历名连段：相邻段同名（中间只隔实物历日）合并。"""
    out = []
    for r in rows:
        a, b = int(r["起"]), int(r["止"])
        g = out[-1] if out else None
        if g and g["历"] == r["历"] and g["止"] == a - 1:
            g["止"] = b
        elif r["来源"] == "实物历日" and g and g["止"] == a - 1 and g["历"] == r["历"]:
            g["止"] = b
        else:
            g = {"历": r["历"], "起": a, "止": b, "来源": set(), "实物": {}, "长历": "", "本纪": r}
            out.append(g)
        if r["来源"] == "实物历日":
            g["实物"][r["来源名"]] = g["实物"].get(r["来源名"], 0) + b - a + 1
        else:
            g["来源"].add(r["来源"])
            if r["来源"] == "古人长历":
                g["长历"] = r["来源名"]
            if r["本纪条数"]:
                g["本纪"] = r
    return out


def render(rows) -> str:
    lines = [
        BEGIN,
        "",
        "| 行用之历 | 年 | 缺省取法 | 其中实物历日 | 推步与张表同月 | 本纪明书朔：条数 / 合推步 / 合张表 | 其中从本纪修正 |",
        "|---|---|---|---|---|---|---|",
    ]
    for g in groups(rows):
        src = "、".join(sorted(g["来源"])) or "实物历日"
        if g["长历"]:
            src = f"古人长历：{g['长历']}"
        shiwu = "；".join(f"{k} {v} 岁" for k, v in g["实物"].items())
        b = g["本纪"]
        benji = f"{b['本纪条数']} / {b['本纪合推步']} / {b['本纪合张表']}" if b["本纪条数"] and b["本纪条数"] != "0" else ""
        same = f"{b['推步与张表同月']} / {b['推步月数']}" if b["推步月数"] else ""
        fix = b["从本纪修正"] if b["从本纪修正"] not in ("", "0") else ""
        lines.append(f"| {g['历']} | {span(g['起'], g['止'])} | {src} | {shiwu} | {same} | {benji} | {fix} |")
    lines += ["", END]
    return "\n".join(lines)


def main():
    block = render(load())
    for p in TARGETS:
        s = p.read_text(encoding="utf-8")
        i, j = s.find(BEGIN), s.find(END)
        assert i >= 0 and j > i, f"{p} 缺标记 {BEGIN} / {END}"
        s = s[:i] + block + s[j + len(END):]
        p.write_text(s, encoding="utf-8")
        print(f"写 {p.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
