#!/usr/bin/env python3
"""
由开发知识库（doc/）生成读者文档站中内容较长的各章：
- 天象（site/astronomy/）：日月食、行星、恒星与二十八宿、史书天象检验、当地星空；
- 历法（site/calendar/）：实历与主表、历代历法总录、各朝推步算法、秦汉初历法复原；
- 附录（site/appendix/）：天象逐条校验、以本纪核验主表、历法原文摘录。

kb_to_site.py 做机械清理（去代码标识、知识库内链接、开发日期戳）；本脚本按章节切分、拼装，并对清理后读不通之处逐条修补。
知识库更新后重跑本脚本即可同步；若某条修补不再匹配，会打印出来，须人工核对。

用法（仓库根）：python3 scripts/figures/build_site_pages.py
"""

import re
import subprocess

SRC = "doc/astronomy/celestial-phenomena.md"


def conv(a: int, b: int, src: str = SRC, keep_names: bool = False, verbatim: bool = False) -> str:
    flag = ["--verbatim"] if verbatim else (["--keep-names"] if keep_names else [])
    return subprocess.run(["python3", "scripts/figures/kb_to_site.py"] + flag + [src, str(a), str(b)],
                          capture_output=True, text=True, check=True).stdout.strip()


def demote(t: str) -> str:
    t = re.sub(r"^### ", "## ", t, flags=re.M)
    return re.sub(r"^#### ", "### ", t, flags=re.M)


def line_of(marker: str, src: str = SRC) -> int:
    for i, ln in enumerate(open(src).read().split("\n"), 1):
        if ln.startswith(marker):
            return i
    raise SystemExit(f"找不到分节标记：{marker}")


# 所有生成章节通用的措辞替换（各页不一定都有，不报未匹配）：知识库文档代号换成读者站章名，开发者用语换成读者能懂的说法
GENERIC_ALL = [
    (r"push-algorithms(\.md)?\s*", "《各朝推步》"),
    (r"historical-calendar(\.md)?\s*", "《实历与主表》"),
    (r"calendar-compendium(\.md)?\s*", "《历代历法总录》"),
    (r"qin-han-calendar-evaluation(\.md)?\s*", "《秦与汉初历法》"),
    (r"celestial-phenomena(\.md)?\s*", "《日月食》等章"),
    (r"日食见第九节", "日食见《日月食》一章"),
    (r"与 第十节", "与《史书天象检验》一章"),
    (r"（P1 中 SOFA", "（本项目 SOFA"),
    (r"本模块", "本项目"),
    (r"主表上游", "主表所据"),
    (r"上游 廖育栋", "廖育栋"),
    (r"上游的", "ChineseCalendar 的"),
    (r"上游(?=推步|代码|说明页|参数|对比|数据|程序|实现|版本|采用|用于|所用)", "ChineseCalendar "),
    (r"上游", "ChineseCalendar"),
]


def fix(text: str, pairs: list[tuple[str, str]], page: str) -> str:
    for a, b in GENERIC_ALL:
        text = re.sub(a, b, text)
    for a, b in pairs:
        new = re.sub(a, b, text, flags=re.M)
        if new == text:
            print(f"[{page}] 修补未匹配：{a[:50]}")
        text = new
    return text


# 对所有生成页面统一生效：知识库文档名换成读者站章名；「上游」写明所指；去掉知识库内部节号与开发阶段编号
KB_CHAPTERS = {
    "push-algorithms": "《各朝推步》",
    "historical-calendar": "《实历与主表》",
    "calendar-compendium": "《历代历法总录》",
    "qin-han-calendar-evaluation": "《秦与汉初历法》",
    "celestial-phenomena": "天象各章",
    "sky-view-3d": "《当地星空》",
    "main-table-checks": "附录《本纪核验主表》",
}
GENERIC_ALL = [(rf"(?<![\w/-]){re.escape(k)}(?:\.md)?(?![\w-])", v) for k, v in KB_CHAPTERS.items()] + [
    (r"主表上游", "主表所据"),
    (r"上游\s*(?=廖育栋)", ""),
    (r"上游(?=的)", "ChineseCalendar"),
    (r"上游(?=推步|代码|说明页|参数|对比|数据|程序|实现|版本|表|采用|用于)", "ChineseCalendar 的"),
    (r"上游", "ChineseCalendar"),
    (r"本模块", "本项目"),
    (r"（P1 中 SOFA 测试值", "（本项目引用 SOFA 测试值时"),
    (r"日食见第九节(?: P2)?", "日食见《日月食》一章"),
    (r"与\s*第十节", "与《史书天象检验》一章"),
    (r"见第九节 P2", "见《日月食》一章"),
    (r"见第九节 P3", "见《行星》一章"),
    (r"第十节(?=西晋)", "《史书天象检验》一章中"),
]


def generic(t: str) -> str:
    for a, b in GENERIC_ALL:
        t = re.sub(a, b, t)
    return t





def build() -> None:
    """正文各章已全部改为手写（见 README）；这里只生成三个附录。"""
    # 附录：逐条校验结果（知识库中由脚本生成，原样转入）
    import os
    os.makedirs("site/appendix", exist_ok=True)
    for src, dst, title in [("doc/astronomy/phenomena-checks.md", "site/appendix/phenomena-checks.md", "# 天象逐条校验"),
                            ("doc/calendar/main-table-checks.md", "site/appendix/main-table-checks.md", "# 本纪核验主表"),
                            ("doc/calendar/calendar-source-texts.md", "site/appendix/source-texts.md", "# 历法原文摘录")]:
        t = conv(1, 10**6, src, verbatim="source-texts" in dst)
        t = re.sub(r"^# .*", title, t, count=1)
        # 生成说明一段（指向脚本与知识库）换成读者说明
        t = re.sub(r"\A(# [^\n]*\n\n)由 .*?\n\n", r"\1本附录是逐条检验的完整结果，由检验程序自动生成；方法与结论见正文各章。\n\n", t, count=1, flags=re.S)
        t = re.sub(r"^数据：\s*$\n?", "", t, flags=re.M)
        # 原文附录的说明行（正文不动）
        t = t.replace("由  从维基文库底本截取（勿手改），", "从维基文库底本截取，")
        t = t.replace("**底本有讹字，引用前须核点校本**（，本地，不入库）", "**底本有讹字，引用前须核中华书局点校本**")
        if "source-texts" not in dst:  # 原文附录一字不动
            t = generic(t)
        open(dst, "w").write(t + "\n")



def split_checks() -> None:
    """天象逐条校验全文太大（数 MB），按大节拆页，第七节（史书记录）再按朝代拆；侧栏条目写入 appendix-sidebar.json。"""
    import json
    import os
    import shutil

    full = open("site/appendix/phenomena-checks.md").read()
    head, *secs = re.split(r"\n(?=## )", full)
    out_dir = "site/appendix/checks"
    shutil.rmtree(out_dir, ignore_errors=True)
    os.makedirs(out_dir)
    items, toc = [], []
    for k, sec in enumerate(secs, 1):
        title = sec.split("\n", 1)[0][3:].strip()
        if title.startswith("七"):
            intro, *subs = re.split(r"\n(?=### )", sec)
            sub_items = []
            for j, sub in enumerate(subs, 1):
                st = sub.split("\n", 1)[0][4:].strip()
                slug = f"{k}-{j}"
                body = re.sub(r"^### ", "# ", sub, count=1)
                body = re.sub(r"^#### ", "## ", body, flags=re.M)
                open(f"{out_dir}/{slug}.md", "w").write(body + "\n")
                sub_items.append({"text": st, "link": f"/appendix/checks/{slug}"})
            slug = f"{k}"
            open(f"{out_dir}/{slug}.md", "w").write("# " + intro[3:].strip() + "\n\n" +
                "\n".join(f"- [{x['text']}]({x['link']})" for x in sub_items) + "\n")
            items.append({"text": title, "link": f"/appendix/checks/{slug}", "collapsed": True, "items": sub_items})
        else:
            slug = f"{k}"
            body = re.sub(r"^## ", "# ", sec, count=1)
            body = re.sub(r"^(#{3,}) ", lambda m: m.group(1)[1:] + " ", body, flags=re.M)
            open(f"{out_dir}/{slug}.md", "w").write(body + "\n")
            items.append({"text": title, "link": f"/appendix/checks/{slug}"})
        toc.append(f"- [{title}](/appendix/checks/{slug})")
    open("site/appendix/phenomena-checks.md", "w").write(head.strip() + "\n\n" + "\n".join(toc) + "\n")
    json.dump(items, open("site/.vitepress/appendix-sidebar.json", "w"), ensure_ascii=False, indent=1)


if __name__ == "__main__":
    build()
    split_checks()
