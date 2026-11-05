#!/usr/bin/env python3
"""
把开发知识库（doc/）的一段 Markdown 转成读者文档站（site/）可用的正文：保留全部专业内容（公式、参数、表格、检验数字、文献），
去掉只对开发者有意义的部分——代码路径与标识符、指向其它知识库文档的链接、开发日期戳、表格中的「代码」列。

转换只做机械清理，结果须人工通读修补；不在这里改写任何数字。

用法：python3 scripts/figures/kb_to_site.py <源.md> <起始行> <结束行>  > 片段.md
"""

import re
import sys

PATH_HINT = re.compile(r"::|/|\\|\.(rs|py|csv|md|txt|bin|br|tsx?|json|sh|toml|pdf|exr|png|jpg)\b")


def is_code(span: str) -> bool:
    """反引号内容是否是路径（文件路径、模块路径）。函数名、体系代号等名字保留为等宽字体，删掉反而读不通。"""
    return bool(PATH_HINT.search(span.strip()))


def drop_code_columns(lines: list[str]) -> list[str]:
    out, i = [], 0
    while i < len(lines):
        ln = lines[i]
        if ln.startswith("|") and i + 1 < len(lines) and re.match(r"^\|[\s\-:|]+\|$", lines[i + 1].strip()):
            head = [c.strip() for c in re.split(r"(?<!\\)\|", ln.strip().strip("|"))]
            drop = {k for k, h in enumerate(head) if h in ("代码", "实现", "实现位置", "代码位置")}
            j = i
            while j < len(lines) and lines[j].startswith("|"):
                cells = re.split(r"(?<!\\)\|", lines[j].strip().strip("|"))
                out.append("| " + " | ".join(c.strip() for k, c in enumerate(cells) if k not in drop) + " |")
                j += 1
            i = j
            continue
        out.append(ln)
        i += 1
    return out


def clean_line(ln: str) -> str:
    # 指向知识库其它文档的链接：「见 [x](y.md) §n」整段去掉；其它 md 链接只留文字
    ln = re.sub(r"[（(]?(?:详)?见\s*\[[^\]]*\]\([^)]*\.md[^)]*\)[^，。；）)]*[）)]?", "", ln)
    ln = re.sub(r"\[([^\]]*)\]\((?!https?:)[^)]*\)", r"\1", ln)
    # 代码标识与路径
    ln = re.sub(r"`([^`]+)`", lambda m: "" if is_code(m.group(1)) else m.group(0), ln)
    # 开发日期戳
    # 只去本项目的开发日期戳（2025—2026 年）；日月食等真实日期保留
    ln = re.sub(r"[（(]\s*202[56]-\d\d-\d\d[^）)]*[）)]", "", ln)
    ln = re.sub(r"202[56]-\d\d-\d\d\s*(起|前|后)?[，,]?", "", ln)
    # 去掉路径后留下的空壳
    ln = re.sub(r"[；;]\s*仓库内[^。；;）)]*", "", ln)
    ln = re.sub(r"(原始文件|原文件|PDF|作者程序与官方测试数据原件|转录)(见|存)\s*(?=[。，；）)]|$)", "", ln)
    ln = re.sub(r"(脚本|数据|代码|测试|解析|结果见|见)\s+(?=[，,。；;）)]|$)", "", ln)
    ln = re.sub(r"(逐条|逐气|逐条结果)见\s*phenomena-checks\.md", r"\1见附录「天象逐条校验」", ln)
    ln = re.sub(r"phenomena-checks\.md", "附录「天象逐条校验」", ln)
    ln = re.sub(r"main-table-checks\.md", "附录「以本纪核验主表」", ln)
    ln = re.sub(r"^(\s*[-*]\s*)[：:]\s*", r"\1", ln)
    ln = re.sub(r"(\*\*[^*]+\*\*)[：:]\s*[、，,]*\s*[。.]?\s*$", r"\1", ln)
    ln = re.sub(r"(\*\*[^*]+\*\*)[：:]\s*[、，,]+\s*", r"\1：", ln)
    ln = re.sub(r"本仓库", "本项目", ln)
    ln = re.sub(r"前端", "网页端", ln)
    # 清理残留标点
    ln = re.sub(r"[（(]\s*[，、；,;]*\s*[）)]", "", ln)
    ln = re.sub(r"（\s*[；;]\s*", "（", ln)
    ln = re.sub(r"\s*[，,]\s*[。.]", "。", ln)
    ln = re.sub(r"[，,]\s*[，,]", "，", ln)
    ln = re.sub(r"[：:]\s*[，,]", "：", ln)
    ln = re.sub(r"（\s*，", "（", ln)
    ln = re.sub(r"，\s*）", "）", ln)
    ln = re.sub(r"：\s*：", "：", ln)
    ln = re.sub(r"\*\*\s*\*\*", "", ln)
    ln = re.sub(r"（(进行中|已完成|完成)）", "", ln)
    # 只压缩行内多余空格，保留行首缩进（嵌套列表靠它）
    m = re.match(r"^[ \t]*", ln)
    ln = m.group(0) + re.sub(r"[ \t]{2,}", " ", ln[m.end():])
    return ln.rstrip()


def main() -> None:
    src, a, b = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
    lines = open(src).read().split("\n")[a - 1 : b]
    lines = drop_code_columns([clean_line(ln) for ln in lines])
    text = "\n".join(lines)
    text = re.sub(r"\n{3,}", "\n\n", text)
    print(text.strip())


if __name__ == "__main__":
    main()
