#!/usr/bin/env python3
"""
由开发知识库（doc/）生成读者文档站中内容较长的各章：
- 天象（site/astronomy/）：日月食、行星、恒星与二十八宿、史书天象检验、当地所见的星空；
- 历法（site/calendar/）：实历与主表、历代历法总录、各朝推步算法、秦汉初历法复原；
- 附录（site/appendix/）：天象逐条校验、以本纪核验主表、历法原文摘录。

kb_to_site.py 做机械清理（去代码标识、知识库内链接、开发日期戳）；本脚本按章节切分、拼装，并对清理后读不通之处逐条修补。
知识库更新后重跑本脚本即可同步；若某条修补不再匹配，会打印出来，须人工核对。

用法（仓库根）：python3 scripts/figures/build_site_pages.py
"""

import re
import subprocess

SRC = "doc/astronomy/celestial-phenomena.md"
SKY = "doc/astronomy/sky-view-3d.md"


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
    (r"push-algorithms(\.md)?\s*", "《各朝代、政权的推步算法》"),
    (r"historical-calendar(\.md)?\s*", "《实历与主表》"),
    (r"calendar-compendium(\.md)?\s*", "《历代历法总录》"),
    (r"qin-han-calendar-evaluation(\.md)?\s*", "《秦与汉初历法复原》"),
    (r"celestial-phenomena(\.md)?\s*", "《日月食》等章"),
    (r"日食见第九节", "日食见《日月食》一章"),
    (r"与 第十节", "与《史书天象记录检验》一章"),
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
    "push-algorithms": "《各朝代、政权的推步算法》",
    "historical-calendar": "《实历与主表》",
    "calendar-compendium": "《历代历法总录》",
    "qin-han-calendar-evaluation": "《秦与汉初历法复原》",
    "celestial-phenomena": "天象各章",
    "sky-view-3d": "《当地所见的星空》",
    "main-table-checks": "附录「以本纪核验主表」",
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
    (r"与\s*第十节", "与《史书天象记录检验》一章"),
    (r"见第九节 P2", "见《日月食》一章"),
    (r"见第九节 P3", "见《行星》一章"),
    (r"第十节(?=西晋)", "《史书天象记录检验》一章中"),
]


def generic(t: str) -> str:
    for a, b in GENERIC_ALL:
        t = re.sub(a, b, t)
    return t


NASA_CHARTS = "\n\n" + "\n\n".join(f'<Chart src="nasa-{k}-{e}" :height="260" />' for k in ("solar", "lunar") for e in ("ancient", "modern"))

FIX_ECLIPSES = [
    (r"\*\*日月食表与图的批量检验\*\*（[^）]*）：", "**日月食表与图的批量检验**（NASA 日月食典前 199—前 100、2001—2100 两段的全部日月食）："),
    (r"\*\*日食地图\*\*（[^）]*）。", "**日食地图**（食带图）"),
    (r"检验（twofloat 构建，`cargo test --release`）：", "检验（双双精度计算）："),
    (r"网页端的 ΔT 滑块和 ±σ 带都靠平移实现，不重算。", "所以改变 ΔT 或画出 ±σ 带都只需平移，不必重算。"),
    (r"^- 测试容差按以上实测设定", "- 检验容差按以上实测设定"),
    (r"^- 耗时：.*\n", ""),
    (r"^\*\*下一步\*\*：.*\n?", ""),
    (r"（各目录每 20 条抽 1 条，各 12 次；`NASA_ECLIPSE_STRIDE=1` 全量）。", "（各目录每 20 条抽 1 条，各 12 次；全量逐次结果见下图）。"),
    (r"VSOP87（轻量，wasm）", "VSOP87（体积小，可在网页中计算）"),
    (r"；与日食同在网页端「日月食图」页", ""),
    (r"；网页端「日食图」页（地球/平面两种投影）", ""),
    (r"。网页逐帧复用同一插值窗", "。逐帧复用同一插值窗"),
    (r"\*\*《汉书·五行志》西汉日食检验\*\*（数据与列说明。", "**《汉书·五行志》西汉日食检验**"),
    (r"(\| 月食 2001～2100 \|[^\n]*)", r"\1" + NASA_CHARTS.replace("\\", "\\\\")),
    (r"^## P2 日月食", "## 实现与检验"),
    (r"\*\*月食的月面实况\*\*（[^）]*）。", "**月食的月面实况**（网页端日月食图中，穿影图右侧）。"),
    (r"LRO 彩色月面图（ 转成\n\s*）", "LRO 彩色月面图（转成 2K 贴图）"),
    (r"检验（f64，`cargo test --release[^`]*`）：", "检验："),
    (r"（DE 月球自转数值解，取，", "（DE 月球自转数值解，"),
]


def build() -> None:
    refs = conv(line_of("## 参考文献"), 10**6)
    p_planet, p_untested = line_of("### P3 行星"), line_of("**尚未做**：太阳引力光线偏折")
    p_mag, p_star = line_of("**行星视星等"), line_of("### P4 恒星与二十八宿")
    p_refr, p_rec = line_of("### 大气折射的模型与依据"), line_of("## 十、史书天象记录检验")
    p_eclipse = line_of("### P2 日月食")
    pages = {
        "eclipses": ("日月食",
                     "本章说明日月食的计算方法与所依据的权威算法、$\\Delta T$ 不确定度的处理，以及与 NASA 五千年日月食典和《汉书》《后汉书》记录的逐条检验。文中方括号数字指文末参考文献。",
                     [conv(line_of("## 四、ΔT 不确定度的处理"), line_of("## 五、数据需求") - 1),
                      conv(line_of("## 七、算法来源"), line_of("## 八、已定事项") - 1),
                      demote(conv(p_eclipse, p_planet - 1))], FIX_ECLIPSES),
        "planets": ("行星：位置、星等、会合与伏见",
                    "本章说明行星视位置的计算与检验、视星等、会合现象（合、冲、留、大距）、近距事件、晨见夕伏的可见性模型，以及月掩星。文中方括号数字指文末参考文献。",
                    [demote(conv(p_planet, p_untested)), demote(conv(p_mag, p_star - 1))], FIX_PLANETS),
        "stars": ("恒星与二十八宿",
                  "本章说明恒星视位置、二十八宿距星的历代体系、宿度与入宿度去极度的计算，以及用历代实测宿度、石氏星表、史书日躔对这些计算与古代坐标量法所做的检验。文中方括号数字指文末参考文献。",
                  [demote(conv(p_star, p_refr - 1))], FIX_STARS),
        "records": ("史书天象记录检验",
                    "二十四史的天文志、五行志、本纪记有数千条日月食与凌犯。本章逐条按当时实行的历法换算日期，在当时都城推算是否真有其事，按朝代汇总。",
                    [conv(p_rec + 1, line_of("## 参考文献") - 1)], FIX_RECORDS),
    }
    for name, (title, intro, parts, fixes) in pages.items():
        body = generic(fix("\n\n".join(parts), fixes, name))
        tail = "" if name == "records" else "\n\n" + generic(refs)
        open(f"site/astronomy/{name}.md", "w").write(f"# {title}\n\n{intro}\n\n{body}{tail}\n")
    sky = conv(1, 10**6, SKY).replace("# 3D 星空：当地所见的天空", "# 当地所见的星空", 1)
    sky = fix(sky + "\n\n" + demote(conv(p_refr, p_rec - 1)), FIX_SKY, "sky-view")
    open("site/astronomy/sky-view.md", "w").write(generic(sky) + "\n")
    # 附录：逐条校验结果（知识库中由脚本生成，原样转入）
    import os
    os.makedirs("site/appendix", exist_ok=True)
    for src, dst, title in [("doc/astronomy/phenomena-checks.md", "site/appendix/phenomena-checks.md", "# 附录：天象逐条校验"),
                            ("doc/calendar/main-table-checks.md", "site/appendix/main-table-checks.md", "# 附录：以本纪核验主表"),
                            ("doc/calendar/calendar-source-texts.md", "site/appendix/source-texts.md", "# 附录：历法原文摘录")]:
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


FIX_PLANETS = [
    (r"^## P3 行星", "## 行星视位置"),
    (r"\*\*取舍\*\*：Native 有 DE406 时以 DE406 为准；无 DE406（如 wasm）用",
     "**取舍**：本地有 DE406 时以 DE406 为准；在网页中（不带 DE406）用"),
    (r"\*\*改正统一\*\*：.*\n.*\n二进制 （约 10 KB）。", "地球改正同时用于节气、定朔与天象（缺省 DE406 版，可选 DE441 版）；原地心太阳赤道补丁与黄道补丁已移除；网页端只加载地球一段（约 10 KB）。"),
    (r"^- 附带：`Real` 新增.*\n.*双双精度。\n", ""),
    (r"（此测试约 20 分钟，标 `#\[ignore\]`，需 `--ignored` 单独运行）", ""),
    (r"所据文献 16、29–39，PDF 存 。", "所据文献 16、29–39。"),
    (r"^-\n  - 用法：.*\n.*\n.*单个天体也可用 。\n", "- 伏见的计算：\n"),
    (r"；保留开关 。", "；保留为可选项。"),
    (r"合日时刻取自已对 Skyfield 验证的 。", "合日时刻取自已对 Skyfield 验证的会合现象计算。"),
    (r"（太阳几何高度在 −18° 与 −50′ 之间， 可调）", "（太阳几何高度在 −18° 与 −50′ 之间，下限可调）"),
    (r"，经 由几何高度求，", "，由几何高度求，"),
]
FIX_STARS = [
    (r"^## P4 恒星与二十八宿", "## 星表、距星与宿度"),
    (r"一并收入 。", "一并收入星表。"),
    (r"^- 解析：。\n", ""),
    (r"^- 读取：。\n", ""),
    (r"；日期有效规则与省略主语的处理见脚本说明", ""),
    (r"^\*\*下一步\*\*：.*\n?", ""),
]
FIX_RECORDS = [
    (r"^- 日月食：见《日月食》一章。", "- 日月食：方法见《日月食》一章。"),
    (r"^- 月、五星凌犯与相合：。", "- 月、五星凌犯与相合："),
    (r"^- 日期偏移：。", "- 日期偏移："),
    (r"(\n\| 清 \|[^\n]*)", r'\1\n\n<Chart src="records-conjunction" />'),
]
FIX_SKY: list[tuple[str, str]] = []

CAL_PAGES = [  # (知识库源, 站点页, 标题, 修补)
    ("doc/calendar/historical-calendar.md", "site/calendar/historical.md", "实历与主表", "FIX_HISTORICAL"),
    ("doc/calendar/calendar-compendium.md", "site/calendar/compendium.md", "历代历法总录", "FIX_COMPENDIUM"),
    ("doc/calendar/push-algorithms.md", "site/calendar/push-algorithms.md", "各朝代、政权的推步算法", "FIX_PUSH"),
    ("doc/calendar/qin-han-calendar-evaluation.md", "site/calendar/qin-han.md", "秦与汉初历法复原", "FIX_QINHAN"),
]
FIX_HISTORICAL = [
    (r"ChineseCalendar 的 里是", "ChineseCalendar 的程序里是"),
    (r"原库 \[ChineseCalendar\]\([^)]*\) 即按此思路：主表用实历，其它政权用 `cmonthDate_Shu` / `cmonthDate_Wu` / `cmonthDate_WeiZhouSui` 等，",
     "廖育栋的 ChineseCalendar 即按此思路：主表用实历，蜀、吴、北朝等政权各按其历术推步，"),
    (r"我们已提取为 。", "本项目已提取为主表。"),
    (r"(\| 明 \| 178 \| 177 \|[^\n]*)", r'\1\n\n<Chart src="main-table-benji" :height="260" />'),
]
FIX_COMPENDIUM = [
    (r"与书的统计全同。转录在 。", "与书的统计全同。"),
]
FIX_PUSH = [
    (r"；以上记录均在 。", "。"),
]
FIX_QINHAN: list[tuple[str, str]] = []


DEV_SECTION = re.compile(r"实现位置|使用方式|^实现$|代码位置|本仓库实现|Rust 实现")
GENERIC_CAL = [
    (r"（wasm `[^`]*`）", ""),
    (r"\bwasm\b ?", ""),
    (r"Rust 错误", "程序错误"),
    (r"（Rust，有理数）", "（有理数运算）"),
    (r"，见 INVENTORY 待办", ""),
    (r"（古历复原的定朔，）", ""),
]


def drop_dev_sections(t: str) -> str:
    """删掉讲实现位置、调用方式的小节（标题到下一个同级或更高级标题）。"""
    out, skip_level = [], 0
    for ln in t.split("\n"):
        m = re.match(r"^(#{2,6}) (.*)", ln)
        if m:
            level = len(m.group(1))
            if skip_level and level <= skip_level:
                skip_level = 0
            if not skip_level and DEV_SECTION.search(m.group(2).strip()):
                skip_level = level
        if not skip_level:
            out.append(ln)
    return "\n".join(out)


def build_calendar() -> None:
    for src, dst, title, fixes in CAL_PAGES:
        t = drop_dev_sections(conv(1, 10**6, src, keep_names=True))
        t = re.sub(r"^# .*", f"# {title}", t, count=1)
        for a, b in GENERIC_CAL:  # 通用替换：各页不一定都有，不报未匹配
            t = re.sub(a, b, t, flags=re.M)
        open(dst, "w").write(generic(fix(t, globals()[fixes], dst)) + "\n")


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
    build_calendar()
    split_checks()
