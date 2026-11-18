"""抓取识典古籍《明实录》（LS0026，历朝实录馆，3118 卷）逐卷原文，存为纯文本，供月朔检验。

识典古籍每卷页面内嵌全书章节列表；正文区 <article class="chapter-reader"> 为服务端渲染的全卷文字（简体）。
本脚本先从任一卷页取章节列表，再逐卷取正文（每段一行）写入 hanshu_cache/mingshilu/<序号>_<卷名>.txt（缓存，不入库）。已存在的卷跳过；
每次请求间隔 3 秒。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/mingshilu_fetch.py
"""
import re
import sys
import time
import subprocess
from pathlib import Path

BASE = "https://www.shidianguji.com/ens/book/LS0026/chapter/"
SEED = "1k6qov9o2ic5m"  # 太祖实录卷之一百
OUT = Path("hanshu_cache/mingshilu")


def get(cid: str) -> str:
    for k in range(4):
        # 用 curl：本机 Python urllib 连此站会挂起
        try:
            r = subprocess.run(["curl", "-s", "-f", "-m", "240", BASE + cid], capture_output=True, check=True)
            return r.stdout.decode("utf-8")
        except Exception as e:  # noqa: BLE001
            print("重试", cid, e, file=sys.stderr)
            time.sleep(10 * (k + 1))
    raise RuntimeError(cid)


def chapters(html: str):
    pat = r'"chapterId":"([^"]+)","chapterType":\d+,"chapterDetailType":\d+,"chapterLevel":\d+,"chapterName":\[\{"content":"([^"]*)"'
    seen, out = set(), []
    for cid, name in re.findall(pat, html):
        if cid not in seen:
            seen.add(cid)
            out.append((cid, name))
    return out


def article(html: str) -> str:
    """服务端渲染的正文区（简体，含全卷）：每个 <p> 一行。"""
    a = html.find('<article class="chapter-reader">')
    b = html.find("</article>", a)
    t = re.sub(r"<[^>]+>", "\n", html[a:b])
    return re.sub(r"\n+", "\n", t).strip()


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    chs = chapters(get(SEED))
    print("章节", len(chs))
    for i, (cid, name) in enumerate(chs):
        f = OUT / f"{i:04d}_{name}.txt"
        if f.exists():
            continue
        html = get(cid)
        text = article(html)
        f.write_text(f"{cid}\n{text}\n", encoding="utf-8")
        if i % 10 == 0:
            print(i, name, len(text), flush=True)
        time.sleep(3)


if __name__ == "__main__":
    main()
