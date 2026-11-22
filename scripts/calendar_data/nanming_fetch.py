"""取维基文库南明史料原文（wikitext）到 hanshu_cache/nanming_ws/（不入库），供 nanming_jiri.py 抽取纪日。

书目（2026-11-22 查维基文库实有者）：杨英《先王實錄》（即《從征實錄》，明郑当时人记录）、徐鼒《小腆紀年》、《小腆紀傳》、
王夫之《行在陽秋》、钱澄之《所知錄》、《思文大紀》、《弘光實錄鈔》、江日昇《臺灣外紀》、翁洲老民《海東逸史》、
夏琳《閩海紀要》、阮旻锡《海上見聞錄》、计六奇《明季南略》。
用法（仓库根）：python3 -I scripts/calendar_data/nanming_fetch.py
维基文库 API 限流：每页间隔 4 秒，失败退避重试。UA 用通用名。
"""
import json
import re
import time
from html import unescape
import urllib.parse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "hanshu_cache/nanming_ws"
UA = {"User-Agent": "calendar-research-script/0.1"}
BOOKS = ["先王實錄", "小腆紀年", "小腆紀傳", "行在陽秋", "所知錄", "思文大紀", "弘光實錄鈔", "臺灣外紀", "海東逸史", "閩海紀要", "海上見聞錄", "明季南略"]


def get(url):
    for k in range(5):
        try:
            return urllib.request.urlopen(urllib.request.Request(url, headers=UA), timeout=60).read().decode("utf-8")
        except Exception:
            time.sleep(20 * (k + 1))
    raise RuntimeError(url)


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    import sys

    # 命令行可另给书目（如西夏：宋史/卷485 宋史/卷486 金史/卷134）
    for b in sys.argv[1:] or BOOKS:
        lf = OUT / f"_list_{b.replace('/', '__')}.json"
        if not lf.exists():
            q = urllib.parse.quote(b)
            lf.write_text(get(f"https://zh.wikisource.org/w/api.php?action=query&list=allpages&apprefix={q}&aplimit=500&format=json"))
            time.sleep(4)
        titles = [p["title"] for p in json.loads(lf.read_text())["query"]["allpages"]]
        for t in titles:
            fn = OUT / (t.replace("/", "__") + ".txt")
            if fn.exists() and fn.stat().st_size > 0:
                continue
            fn.write_text(get("https://zh.wikisource.org/w/index.php?action=raw&title=" + urllib.parse.quote(t)))
            time.sleep(4)
        print(b, len(titles), flush=True)
    # 正文由扫描页（Page: 名字空间）嵌入者，raw 只有 <pages index=…/>：改取解析后的 HTML，去标签存 .html.txt
    for fn in sorted(OUT.glob("*.txt")):
        if "<pages index" not in fn.read_text() or fn.name.endswith(".html.txt"):
            continue
        hf = fn.with_name(fn.stem + ".html.txt")
        if hf.exists() and hf.stat().st_size > 0:
            continue
        t = fn.stem.replace("__", "/")
        d = json.loads(get("https://zh.wikisource.org/w/api.php?action=parse&format=json&prop=text&page=" + urllib.parse.quote(t)))
        html = d["parse"]["text"]["*"]
        html = re.sub(r"<span[^>]*class=\"pagenum[^>]*>.*?</span>", "", html, flags=re.S)
        html = re.sub(r"<(p|br|div|li|h\d)[^>]*>", "\n", html)
        text = re.sub(r"<[^>]+>", "", html)
        hf.write_text(unescape(text))
        time.sleep(4)
        print("parse", t, flush=True)


if __name__ == "__main__":
    main()
