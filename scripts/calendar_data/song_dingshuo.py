"""宋诸历定朔（《宋史·律历志》），复核主表与《宋史》本纪明书朔日。表直接解析维基文库底本的 wikitable。

算法（纪元历术文）：经朔 + 入气朏朒定数（恒气，每气内逐日损益率等差，「求每日盈缩分先后数」）+ 入转朏朒定数（「朏减朒加」）= 定朔。
进朔（纪元历注文，崇天历另有「朔小餘如日入分已上者，進一日」）：秋分后小余 ≥ 3/4 日法；春分后限减（春分日出分 − 定朔日出分）/ 3；有交者不进（未实现）。
日出分未取本历之表，用 daming_dingshuo.py 的二十四气日出分（比例）代之，属假定；影响只在夏半年临限之月。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/song_dingshuo.py
"""
import re
import sys
from fractions import Fraction as Fr
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.path.insert(0, str(Path(__file__).resolve().parent / "qinhan_eval"))
import daming_dingshuo as dm  # noqa: E402
import dingshuo_check as c  # noqa: E402
import gaiyuan_date as g  # noqa: E402
import houhanshu_star_records as hh  # noqa: E402

D = {"空": 0, "初": 0, "一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}


def cn(s):
    t, w, cur = 0, 0, 0
    for ch in s:
        if ch in D:
            cur = D[ch]
        elif ch == "廿":
            w += 20
        elif ch in "十百千":
            w += (cur or 1) * {"十": 10, "百": 100, "千": 1000}[ch]
            cur = 0
        elif ch == "萬":
            t += (w + cur) * 10000
            w, cur = 0, 0
    return t + w + cur


def wikitables(title):
    x = re.sub(r"<ref.*?</ref>", "", hh.fetch(title), flags=re.S)
    x = re.sub(r"\{\{\*\|([^{}]*)\}\}", r"\1", x)
    out = []
    for t in re.findall(r"\{\|.*?\|\}", x, flags=re.S):
        rows = []
        for r in t.split("|-")[1:]:
            r = r.replace("|}", "").strip()
            if r.startswith("!"):
                rows.append([c_.strip() for c_ in r.lstrip("! ").split("!!")])
            else:
                rows.append([c_.strip() for c_ in r.lstrip("| ").split("||")])
        out.append(rows)
    return out


def rate(cell):
    """「益七百一十四」→ [(符号, 数)]；「初益六十<br />末損五」→ 两段（益 +1、损 −1，相对该日所在名）。"""
    return [(1 if m.group(1) == "益" else -1, cn(m.group(2))) for m in re.finditer(r"([益損減])([一二三四五六七八九十廿百千萬]+)", cell)]


class Li:
    def __init__(self, name, title_sun, title_moon, f, sui, shuo, zhuan, ji, y_ji, day0, inits, rinru=False, fix_moon=None):
        self.rinru = rinru
        self.name, self.f, self.sui, self.shuo, self.zhuan, self.ji, self.y_ji, self.day0 = name, f, sui, shuo, zhuan, ji, y_ji, day0
        self.qi = Fr(sui, 24)
        sun = next(t for t in wikitables(title_sun) if t[0][0] in ("常氣", "氣節") and any("損益" in h for h in t[0]))
        col = next(i for i, h in enumerate(sun[0]) if "損益" in h)
        self.sun_abs = [rate(r[col])[0][1] for r in sun[1:25]]
        moon = next(t for t in wikitables(title_moon) if "日" in t[0][0] and any("損益" in h for h in t[0]) and t[1][0] == "一日")
        col = next(i for i, h in enumerate(moon[0]) if "損益" in h)
        # 朏向为负（「朏减朒加」）；前半名以表首行为准（纪元起朏、崇天起朒），后半相反
        first = -1 if "朏" in moon[1][-1] else 1
        self.moon, self.split = [], {}
        for i, r in enumerate(moon[1:29]):
            sg = first if i < 14 else -first
            parts = rate(r[col])
            if len(parts) == 1:
                self.moon.append(sg * parts[0][0] * parts[0][1])
            else:
                self.moon.append(None)
                self.split[i] = [sg * p[0] * p[1] for p in parts]
        for i, v in (fix_moon or {}).items():
            self.moon[i] = v
        self.moon[27], last = None, rate(moon[28][col])[0]
        self.split[27] = [-first * last[0] * last[1], 0]
        self.split[13][1] = (1 if self.split[13][0] > 0 else -1) * abs(self.split[13][1])   # 十四日：积至零后「末益」入异名，与初段同向
        self.inits = inits
        acc, self.mji = 0, []
        for i, v in enumerate(self.moon):
            self.mji.append(acc)
            acc += sum(self.split[i]) if i in self.split else v
        self.closure = acc
        self.sjp = self.sun_table()

    def sun_table(self):
        """每气 (朏朒积, 符号, 定初率, 日差)，按术文。"""
        q = self.qi / self.f
        out, ji = [], 0
        for k, L in enumerate(self.sun_abs):
            kk, half = k % 12, k // 12
            sign = (1 if kk < 6 else -1) * (1 if half == 0 else -1)
            nxt = self.sun_abs[(k + 1) % 24]
            prev = self.sun_abs[(k - 1) % 24]
            m = Fr(L) / q
            if kk < 6:   # 至后，率递减；分前一气无后气，因前气合差
                d = (Fr(L) - nxt) / q if kk != 5 else (Fr(prev) - L) / q
                cha = d / q
                a = m + d / 2 - cha / 2
                out.append((ji, sign, a, -cha))
            else:        # 分后，率递增；至前一气因前气合差
                d = (Fr(nxt) - L) / q if kk != 11 else (Fr(L) - prev) / q
                cha = d / q
                a = m - d / 2 + cha / 2
                out.append((ji, sign, a, cha))
            ji += sign * L
        return out

    def sun_def(self, t):
        k = int(t // self.qi)
        ji, sign, a, cha = self.sjp[k % 24]
        x = t - k * self.qi
        day, r = int(x // self.f), x % self.f
        acc = sum(a + i * cha for i in range(day)) + (a + day * cha) * r / self.f
        return ji + sign * acc

    def moon_def(self, x):
        i, r = int(x // self.f), x % self.f
        if i in self.split:
            n0 = self.inits[i]
            r0, r1 = self.split[i]
            if r <= n0:
                return self.mji[i] + r0 * r / n0
            return self.mji[i] + r0 + r1 * (r - n0) / (self.f - n0)
        return self.mji[i] + self.moon[i] * r / self.f

    def tong(self, y):
        return self.sui * (self.ji + (y - self.y_ji))

    def moons(self, y0, y1):
        out = []
        for y in range(y0, y1):
            tong = self.tong(y)
            t0 = tong - tong % self.shuo
            for m in range(14):
                jing = t0 + m * self.shuo
                ding = jing + self.sun_def(Fr(jing - tong)) + self.moon_def((t0 % self.zhuan + m * self.shuo) % self.zhuan)
                out.append(ding)
        return sorted(set(out))

    def limit(self, t):
        y = g.from_jdn(int(t // self.f) + self.off)[0]
        for yy in (y + 1, y, y + 2):
            d = t - self.tong(yy)
            if 0 <= d < self.sui:
                break
        if self.rinru:   # 崇天：「朔小餘如日入分已上者，進一日」
            return float((1 - dm.sunrise(d * dm.QI / self.qi) / dm.F) * 24)
        if 6 * self.qi <= d < 18 * self.qi:
            sr = dm.sunrise(d * dm.QI / self.qi) / dm.F
            return float(18 - (dm.SUNRISE[6] / dm.F - sr) * 24 / 3)
        return 18.0


JIYUAN = Li("纪元", "宋史/卷079", "宋史/卷079", 7290, 2662626, 215278, Fr(200873 * 10000 + 990, 10000),
            28613466, 1106, 15, {6: 6478, 13: 5666, 20: 4854, 27: 4043})


CHONGTIAN = Li("崇天", "宋史/卷072", "宋史/卷072", 10590, 3867940, 312729, Fr(291803 * 10000 + 594, 10000),
               97556340, 1024, 0, {6: 9441, 13: 8232, 20: 7052, 27: 5873}, rinru=True,
               fix_moon={17: -601})   # 十八日：宋史作益三百九十一（点校本册五书页 1638 同），朏积 2924→3525 须 601，转定分「增七十五」约 594；本项目校改


def run(li, y0, y1, approx):
    dz = li.tong(li.y_ji) // li.f
    li.off = c.align_offset(dz + li.day0, g.jdn(*approx)) + li.day0
    print(f"--- {li.name}历：月离闭合 {li.closure}；{li.y_ji} 年天正冬至", g.ganzhi_of_jdn(dz + li.off), g.from_jdn(dz + li.off))
    c.compare(li.name, li.f, li.off, li.moons(y0 - 2, y1 + 3), y0, y1, f"song_{li.name}_check.csv",
              f"{y0}—{y1} 年《宋史》本纪明书朔日与{li.name}历定朔（scripts/calendar_data/song_dingshuo.py）、主表比较；进朔限术文，夏半年日出分为假定。",
              limit=li.limit, books={"songshi_benji"})


if __name__ == "__main__":
    run(CHONGTIAN, 1024, 1064, (1023, 12, 18))
    run(JIYUAN, 1106, 1135, (1105, 12, 16))
