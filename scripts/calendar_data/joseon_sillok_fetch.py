"""抓取维基文库《朝鮮王朝實錄》太祖至仁祖（1392—1649）各年页面 wikitext，缓存到 hanshu_cache/joseon_sillok/（不入库）。

先用 API（list=allpages, apprefix=朝鮮王朝實錄/）列页，存 titles.txt；再逐页取 action=raw。
已缓存者跳过，可随时中断续抓；《光海君日記》各年子页中文维基文库缺，取韩文维基文库太白山本；每次请求间隔 ≥3 秒，失败（含 429/5xx）退避重试。
不设伪造的 User-Agent（curl 默认）。

用法（仓库根）：python3 scripts/calendar_data/joseon_sillok_fetch.py [--list]
"""
import json
import re
import subprocess
import sys
import time
import urllib.parse
from pathlib import Path

CACHE = Path("hanshu_cache/joseon_sillok")
API = "https://zh.wikisource.org/w/api.php"
RAW = "https://zh.wikisource.org/w/index.php?action=raw&title="
REIGNS = ["太祖實錄", "定宗實錄", "太宗實錄", "世宗實錄", "文宗實錄", "端宗實錄", "世祖實錄", "睿宗實錄", "成宗實錄",
          "燕山君日記", "中宗實錄", "仁宗實錄", "明宗實錄", "宣祖實錄", "宣祖修正實錄", "光海君日記", "仁祖實錄"]
# 《光海君日記》中文维基文库只有目录页（各年子页不存在），改取韩文维基文库的太白山本（中草本）原文
KO_RAW = "https://ko.wikisource.org/w/index.php?action=raw&title="
KO_TITLES = ["조선왕조실록/광해군일기태백산본/" + y for y in ["즉위년"] + [f"{n}년" for n in range(1, 16)]]
YEAR = re.compile(r"^(卽位年|即位年|元年|[一二三四五六七八九十]+年)$")
GAP = 3.5


def curl(args, out):
    for k in range(6):
        r = subprocess.run(["curl", "-sS", "-f", "--max-time", "120", "-o", str(out)] + args)
        if r.returncode == 0 and out.exists() and out.stat().st_size > 0:
            return True
        wait = GAP * (2 ** (k + 1))
        print(f"  失败（{r.returncode}），{wait:.0f} 秒后重试", flush=True)
        time.sleep(wait)
    return False


def list_titles():
    titles, cont, i = [], None, 0
    while True:
        i += 1
        args = ["-G", API, "--data-urlencode", "action=query", "--data-urlencode", "list=allpages",
                "--data-urlencode", "apprefix=朝鮮王朝實錄/", "--data-urlencode", "aplimit=500",
                "--data-urlencode", "format=json"]
        if cont:
            args += ["--data-urlencode", f"apcontinue={cont}"]
        out = CACHE / f"list{i}.json"
        curl(args, out)
        d = json.loads(out.read_text())
        titles += [x["title"] for x in d["query"]["allpages"]]
        cont = d.get("continue", {}).get("apcontinue")
        if not cont:
            break
        time.sleep(GAP)
    (CACHE / "titles.txt").write_text("\n".join(titles), encoding="utf-8")
    return titles


def wanted(titles):
    out = []
    for t in titles:
        parts = t.split("/")
        if len(parts) < 2 or parts[1] not in REIGNS:
            continue
        if parts[1] == "光海君日記" or (len(parts) == 3 and YEAR.match(parts[2])):
            out.append(t)
    return out


def fname(t):
    if t.startswith("조선왕조실록/"):
        return CACHE / ("ko_" + t.split("/", 1)[1].replace("/", "_") + ".txt")
    return CACHE / (t.replace("朝鮮王朝實錄/", "").replace("/", "_") + ".txt")


def main():
    CACHE.mkdir(parents=True, exist_ok=True)
    tf = CACHE / "titles.txt"
    titles = list_titles() if ("--list" in sys.argv or not tf.exists()) else tf.read_text(encoding="utf-8").split("\n")
    todo = [t for t in wanted(titles) + KO_TITLES if not fname(t).exists()]
    print(f"待抓 {len(todo)} 页", flush=True)
    for n, t in enumerate(todo, 1):
        tmp = fname(t).with_suffix(".part")
        ok = curl([(KO_RAW if t.startswith("조선") else RAW) + urllib.parse.quote(t)], tmp)
        if ok:
            tmp.rename(fname(t))
            print(f"[{n}/{len(todo)}] {t} {fname(t).stat().st_size}", flush=True)
        else:
            print(f"[{n}/{len(todo)}] {t} 放弃", flush=True)
        time.sleep(GAP)


if __name__ == "__main__":
    main()
