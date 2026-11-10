#!/usr/bin/env python3
"""
把开发知识库（doc/）的一段 Markdown 转成读者文档站（site/）可用的正文：保留全部专业内容（公式、参数、表格、检验数字、文献），
去掉只对开发者有意义的部分——代码路径与标识符、指向其它知识库文档的链接、开发日期戳、表格中的「代码」列。

转换只做机械清理，结果须人工通读修补；不在这里改写任何数字。

用法：python3 scripts/figures/kb_to_site.py [--keep-names | --verbatim] <源.md> <起始行> <结束行>  > 片段.md
"""

import re
import sys

PATH_HINT = re.compile(r"::|\\|[\w.-]+/[\w.-]+/|\.(rs|py|csv|md|txt|bin|br|tsx?|json|sh|toml|pdf|exr|png|jpg|js)\b|^[\w.-]+/$")


# 外部软件、文献中的名字：SOFA 函数、寿星天文历函数、上游 ChineseCalendar 函数等，作为引用保留
EXTERNAL = re.compile(r"^(iau[A-Z]\w*|[a-z]{2,4}[A-Z]{1,3}\w*|XL[01]|SSQ|SZJ|HXK|cmonthDate_\w+|ChineseToGregorian|HanZhuanXu|"
                      r"LiaoJinYuanCorrection|action=raw|swehel\.c|VISLIMIT\.BAS|precessionlib\.compute_precession|vsop87\.doc)$")


VERBATIM = False  # 原文模式：古籍原文一字不改，只去链接与路径，不做标点清理
KEEP_NAMES = False  # 保守模式：只删路径，名字一律保留（历法各章描述上游程序的推步算法，变量名是算法的一部分）


def is_code(span: str) -> bool:
    """反引号内容是否应删：路径一律删；本项目内部的函数、测试、类型名删；外部软件与文献中的名字保留。"""
    t = span.strip()
    if KEEP_NAMES:
        return bool(PATH_HINT.search(t)) or not t
    if EXTERNAL.match(t):
        return False
    if PATH_HINT.search(t) or not t:
        return True
    return bool(re.fullmatch(r"[A-Za-z_][\w.]*(\([^)]*\))?", t)) and ("_" in t or re.search(r"[a-z][A-Z]", t) is not None)


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
    orig = ln
    # 指向知识库其它文档的链接：「见 [x](y.md) §n」整段去掉；其它 md 链接只留文字
    ln = re.sub(r"[（(]?(?:详)?见\s*\[[^\]]*\]\([^)]*\.md[^)]*\)[^，。；）)]*[）)]?", "", ln)
    ln = re.sub(r"\[([^\]]*)\]\((?!https?:)[^)]*\)", r"\1", ln)
    # 代码标识与路径
    ln = re.sub(r"`([^`]+)`", lambda m: "" if is_code(m.group(1)) else m.group(0), ln)
    # 开发日期戳
    # 只去本项目的开发日期戳（2025—2026 年）；日月食等真实日期保留
    ln = re.sub(r"[（(]\s*202[56]-\d\d-\d\d[^）)]*[）)]", "", ln)
    ln = re.sub(r"202[56]-\d\d-\d\d\s*(起|前|后)?[，,]?", "", ln)
    # 作为补充说明出现的程序标识（测试名、函数名、类型名）连同引导词一起去掉；做主语的保留
    ident = r"`[A-Za-z_][\w:<>,.&\[\]{}() ]*`"
    if KEEP_NAMES:
        ident = r"`(?!)`"  # 保守模式不动名字
    ln = re.sub(r"[（(]\s*(?:测试|检验|实现|函数|见|由|用|入口|代码)?\s*" + ident + r"(?:\s*[、，,]\s*" + ident + r")*\s*[）)]", "", ln)
    ln = re.sub(r"(检验|测试|验证)\s*" + ident + r"(?:\s*[、，,]\s*" + ident + r")*\s*[：:]", r"\1：", ln)
    ln = re.sub(r"[，,]?\s*(?:测试|由)\s*" + ident + r"\s*(?:输出|给出|保证)?", "", ln)
    ln = re.sub(ident + r"\s*(输出|的输出)", "计算结果", ln)
    ln = re.sub(r"Rust 侧?(给|按|算|用)", r"计算时\1", ln)
    ln = re.sub(r"Rust 给", "计算端给出", ln)
    ln = re.sub(r"（(进行中|已完成|完成)）", "", ln)  # 开发进度标注
    # 作者而非仓库名：ytliu0/ChineseCalendar → 廖育栋（Yuk Tung Liu）的 ChineseCalendar（网址保留）
    ln = re.sub(r"\[ytliu0/ChineseCalendar\]\((https?://[^)]*)\)", "廖育栋（Yuk Tung Liu）的 ChineseCalendar", ln)
    ln = re.sub(r"(?<![/.\w])ytliu0/ChineseCalendar(?![\w/])", "廖育栋（Yuk Tung Liu）的 ChineseCalendar", ln)
    ln = re.sub(r"(?<![/.\w])ytliu0(?![\w/.])", "廖育栋（Yuk Tung Liu）", ln)
    ln = re.sub(r"的 ChineseCalendar 的 ChineseCalendar", "的 ChineseCalendar", ln)
    # 外部程序文件名改作描述；其余裸写的文件名（不在反引号里）删去
    ln = ln.replace("VISLIMIT.BAS", "VISLIMIT 程序").replace("t_sofa_c.c", "SOFA 测试程序")
    ln = re.sub(r"`?\b(?!three\.js)[\w./-]*[A-Za-z_][\w-]*\.(py|csv|rs|md|c|doc|txt|json|bsp|js|tsx?)\b`?", "", ln)
    # 去掉路径后留下的空壳
    ln = re.sub(r"[；;]\s*仓库内[^。；;）)]*", "", ln)
    ln = re.sub(r"(原始文件|原文件|PDF|作者程序与官方测试数据原件|转录)(见|存)\s*(?=[。，；）)]|$)", "", ln)
    ln = re.sub(r"(脚本|数据|代码|测试|解析|结果见|见)\s+(?=[，,。；;）)]|$)", "", ln)
    ln = re.sub(r"(逐条|逐气|逐条结果)见\s*phenomena-checks\.md", r"\1见附录「天象逐条校验」", ln)
    ln = re.sub(r"phenomena-checks\.md", "附录「天象逐条校验」", ln)
    ln = re.sub(r"benji-checks\.md", "附录「以本纪核验历日」", ln)
    ln = re.sub(r"^(\s*[-*]\s*)[：:]\s*", r"\1", ln)
    ln = re.sub(r"(\*\*[^*]+\*\*)[：:]\s*[、，,]*\s*[。.]?\s*$", r"\1", ln)
    ln = re.sub(r"(\*\*[^*]+\*\*)[：:]\s*[、，,]+\s*", r"\1：", ln)
    ln = re.sub(r"本仓库", "本项目", ln)
    ln = re.sub(r"前端", "网页端", ln)
    ln = re.sub(r"(实现在|实现见|数据在|结果在|实现：)\s*(?=[。，；]|$)", "", ln)
    if VERBATIM or ln == orig:
        # 原文模式，或本行没有删过任何东西：不做标点清理，以免误改引文
        return ln.rstrip()
    # 清理残留标点
    ln = re.sub(r"[（(]\s*[，、；,;]*\s*[）)]", "", ln)
    ln = re.sub(r"（\s*[；;]\s*", "（", ln)
    ln = re.sub(r"\s*[；;，,]\s*[）)]", "）", ln)
    ln = re.sub(r"\s*[，,]\s*[。.]", "。", ln)
    ln = re.sub(r"[，,]\s*[，,]", "，", ln)
    ln = re.sub(r"[：:]\s*[，,]", "：", ln)
    ln = re.sub(r"（\s*，", "（", ln)
    ln = re.sub(r"，\s*）", "）", ln)
    ln = re.sub(r"：\s*：", "：", ln)
    ln = re.sub(r"\*\*\s*\*\*", "", ln)
    # 只压缩行内多余空格，保留行首缩进（嵌套列表靠它）
    m = re.match(r"^[ \t]*", ln)
    ln = m.group(0) + re.sub(r"[ \t]{2,}", " ", ln[m.end():])
    ln = re.sub(r"^(\s*(?:>\s*)?)[。，、；：]+\s*", r"\1", ln)
    return ln.rstrip()


def main() -> None:
    global KEEP_NAMES, VERBATIM
    args = sys.argv[1:]
    if args and args[0] == "--verbatim":
        VERBATIM = KEEP_NAMES = True
        args = args[1:]
    if args and args[0] == "--keep-names":
        KEEP_NAMES = True
        args = args[1:]
    src, a, b = args[0], int(args[1]), int(args[2])
    lines = open(src).read().split("\n")[a - 1 : b]
    lines = drop_code_columns([clean_line(ln) for ln in lines])
    text = "\n".join(lines)
    text = re.sub(r"\n{3,}", "\n\n", text)
    print(text.strip())


if __name__ == "__main__":
    main()
