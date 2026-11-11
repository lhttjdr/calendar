"""《新法算书》四库本（CADAL 影印，Wikimedia Commons「Category:新法算書」）月离表各册按页取图（Commons 缩略图服务渲染 DjVu 页）。

用法（仓库根）：python scripts/calendar_data/xinfa_scan_fetch.py <CADAL 编号> <起页> <止页> <宽度>
图存 hanshu_cache/xinfa_scan/（不入库）。卷三十一至三十二 06054904（134 页）、卷三十三至三十四 06054905（121 页；
第 2 页起卷三十三太陰自行加減表，第 55 页起卷三十四二三均數表）、卷三十五至三十六 06054906（180 页）。
"""
import json, sys, time, urllib.parse, urllib.request
from pathlib import Path
UA = {"User-Agent": "lunar-calendar-research/1.0 (research script)"}
NAMES = {
    "06054904": "CADAL06054904_新法算書·卷三十一~卷三十二.djvu",
    "06054905": "CADAL06054905_新法算書·卷三十三~卷三十四.djvu",
    "06054906": "CADAL06054906_新法算書·卷三十五~卷三十六.djvu",
}
def get(code, page, width):
    out = Path(__file__).resolve().parents[2] / "hanshu_cache/xinfa_scan" / f"{code}_p{page:03d}_{width}.jpg"
    out.parent.mkdir(parents=True, exist_ok=True)
    if out.exists() and out.stat().st_size > 0:
        return out
    q = urllib.parse.urlencode({"action": "query", "titles": "File:" + NAMES[code], "prop": "imageinfo",
                                "iiprop": "url", "iiurlparam": f"page{page}-{width}px", "format": "json"})
    d = json.load(urllib.request.urlopen(urllib.request.Request("https://commons.wikimedia.org/w/api.php?" + q, headers=UA)))
    url = next(iter(d["query"]["pages"].values()))["imageinfo"][0]["thumburl"]
    for k in range(5):
        try:
            out.write_bytes(urllib.request.urlopen(urllib.request.Request(url, headers=UA)).read())
            break
        except Exception as e:
            time.sleep(3 * (k + 1))
    time.sleep(0.5)
    return out
if __name__ == "__main__":
    code, a, b, w = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
    for p in range(a, b + 1):
        print(get(code, p, w))
