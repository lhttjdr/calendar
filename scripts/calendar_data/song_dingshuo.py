"""宋诸历定朔（《宋史·律历志》），复核张表与《宋史》本纪明书朔日。表直接解析维基文库底本的 wikitable。

算法（纪元历术文）：经朔 + 入气朏朒定数（恒气，每气内逐日损益率等差，「求每日盈缩分先后数」）+ 入转朏朒定数（「朏减朒加」）= 定朔。
进朔（纪元历注文，崇天历另有「朔小餘如日入分已上者，進一日」）：秋分后小余 ≥ 3/4 日法；春分后限减（春分日出分 − 定朔日出分）/ 3；有交者不进（未实现）。
日出分于定朔所在日午中按各历《宋史》步晷漏求之：崇天（卷七十二）、明天（卷七十五）、观天（卷七十七）用消息定数求晨分，
纪元（卷七十九）由午中日行积度求赤道内外度再求日出分；南宋诸历志文「法與前曆同，此不載」，代用纪元之术（按一日之比）。
观天「春分後加二千一百少」从陈美东《古历新探》第七章校作二千一百五少。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/song_dingshuo.py
"""
import re
import sys
from fractions import Fraction as Fr
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.path.insert(0, str(Path(__file__).resolve().parent / "qinhan_eval"))
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


def pick(cell, v):
    """同表数历：cell 以 <br/> 分栏，取第 v 栏；v 为 None 则原样。"""
    if v is None:
        return cell
    parts = []
    for x in re.split(r"<br ?/?>", cell):      # 「初…<br/>末…」同属一历
        if x.startswith("末") and parts:
            parts[-1] += x
        else:
            parts.append(x)
    return parts[v] if v < len(parts) else parts[-1]


class Li:
    def __init__(self, name, title_sun, title_moon, f, sui, shuo, zhuan, ji, y_ji, day0, inits=None, rinru=False, fix_moon=None, v=None, delta=None, end14=None, sun_fn=None, cha=None, fix_sun=None):
        self.sun_fn, self.cha = sun_fn, cha or {}
        self.rinru = rinru
        self.name, self.f, self.sui, self.shuo, self.zhuan, self.ji, self.y_ji, self.day0 = name, f, sui, shuo, zhuan, ji, y_ji, day0
        self.qi = Fr(sui, 24)
        if not sun_fn:
            sun = next(t for t in wikitables(title_sun) if t[0][0] in ("常氣", "氣節") and any("損益" in h for h in t[0]))
            col = next(i for i, h in enumerate(sun[0]) if "損益" in h)
            half = [rate(pick(r[col], v))[0][1] for r in sun[1:7]]    # 冬至至惊蛰；其余气与之同数（表中后段偶有错栏、讹字，如统元小满、淳熙小雪）
            for i, x in (fix_sun or {}).items():
                half[i] = x
            self.sun_abs = (half + half[::-1]) * 2
        moon = next(t for t in wikitables(title_moon) if len(t) > 1 and any("損益" in h for h in t[0]) and t[1][0] == "一日")
        col = next(i for i, h in enumerate(moon[0]) if "損益" in h)
        jcol = next(i for i, h in enumerate(moon[0]) if "朏朒" in h)
        # 朏向为负（「朏减朒加」）；前半名以表首行为准（纪元起朏、崇天起朒），后半相反
        first = -1 if "朏" in pick(moon[1][jcol], v) else 1
        self.moon, self.split = [], {}
        for i, r in enumerate(moon[1:29]):
            sg = first if i < 14 else -first
            parts = rate(pick(r[col], v))
            if len(parts) == 1:
                self.moon.append(sg * parts[0][0] * parts[0][1])
            else:
                self.moon.append(None)
                self.split[i] = [sg * p[0] * p[1] for p in parts]
        for i, x in (fix_moon or {}).items():
            self.moon[i] = x
        for i, x in (delta or {}).items():
            self.moon[i] += x
        self.moon[27], last = None, rate(pick(moon[28][col], v))[0]
        self.split[27] = [-first * last[0] * last[1], 0]
        if end14:
            self.split[13][1] = end14
        self.split[13][1] = (1 if self.split[13][0] > 0 else -1) * abs(self.split[13][1])   # 十四日：积至零后「末益」入异名，与初段同向
        # 初数：转周之四分点减整日（志文所列有讹字，如乾道十四日、会元十四日；由此推得与未讹者相合）
        zd = Fr(zhuan) / f
        self.inits = {i: (Fr(q) * zd / 4 - int(Fr(q) * zd / 4)) * f for q, i in ((1, 6), (2, 13), (3, 20), (4, 27))}
        acc, self.mji = 0, []
        for i, x in enumerate(self.moon):
            self.mji.append(acc)
            acc += sum(self.split[i]) if i in self.split else x
        self.closure = acc
        self.sjp = None if sun_fn else self.sun_table()

    def jiaozheng(self, mags, end14=None):
        """按陈美东《历代律历志校证》（2007，宋史卷七十一至八十四；条目出自陈美东、张培瑜《月离表初探》）改月离损益率：
        mags = {日序（0 起）: 正值（绝对值）}，保原符号；end14 改十四日末率。改后重算朏朒积与闭合。"""
        for i, v in mags.items():
            s = -1 if self.moon[i] < 0 else 1
            self.moon[i] = s * v
        if end14 is not None:
            s = -1 if self.split[13][1] < 0 else 1
            self.split[13][1] = s * end14
        acc, self.mji = 0, []
        for i, x in enumerate(self.moon):
            self.mji.append(acc)
            acc += sum(self.split[i]) if i in self.split else x
        self.closure = acc
        return self

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
        if self.sun_fn:
            return self.sun_fn(t % self.sui, self.f)
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
            return self.mji[i] + r0 + r1 * (r - n0) / (self.f - n0) if r1 else self.mji[i] + r0
        return self.mji[i] + self.moon[i] * r / self.f

    def tong(self, y):
        """冬至气积分。统天另减气差与躔差（距差 = 积算 − 距算）。"""
        n = self.ji + (y - self.y_ji)
        if not self.cha:
            return self.sui * n
        ju = n - self.cha["ju"]
        return self.sui * n - self.cha["qi"] - Fr(127 * ju * ju, 10000)

    def first_shuo(self, y):
        tong = self.tong(y)
        if not self.cha:
            return tong - tong % self.shuo
        ju = self.ji + (y - self.y_ji) - self.cha["ju"]
        return tong - (tong - self.cha["run"]) % self.shuo - Fr(105 * ju, 10)

    def moons(self, y0, y1):
        out = []
        for y in range(y0, y1):
            tong, t0 = self.tong(y), self.first_shuo(y)
            base = (t0 - self.cha["zhuan"]) % self.zhuan if self.cha else t0 % self.zhuan
            for m in range(14):
                jing = t0 + m * self.shuo
                ding = jing + self.sun_def(Fr(jing - tong)) + self.moon_def((base + m * self.shuo) % self.zhuan)
                out.append(ding)
        return sorted(set(out))

    def yingsuo_deg(self, d):
        """定朔所在日午中距冬至之盈缩（度）：崇天中积为常气积日，不加；观天按本历盈缩分术；明天盈缩差度 = 定差 × 567/(400 × 4135)；
        纪元先后数与朏朒积成比例（春分先二萬四千、朒一千三百九，万分度）；南宋诸历以纪元之比折算。"""
        if self.name == "崇天":
            return Fr(0)
        if self.name == "观天":
            return guantian_curve(d % self.sui, self.f, Fr(1, 3294), Fr(1, 3659))
        if self.name == "明天":
            return self.sun_def(d) * 567 / (400 * 4135)
        return self.sun_def(d) / self.f * 7290 * 24000 / (1309 * 10000)

    def sunrise(self, x):
        if self.name in LOU:
            return xiaoxi_sunrise(LOU[self.name], x)
        return jiyuan_sunrise(x)

    def sunrise_chunfen(self):
        if self.name in LOU:
            return xiaoxi_sunrise(LOU[self.name], LOU[self.name]["l"] / 2)
        return jiyuan_sunrise(JY_L / 2)

    def limit(self, t):
        """进朔限（时）"""
        noon = (t // self.f + Fr(1, 2)) * self.f
        y = g.from_jdn(int(t // self.f) + self.off)[0]
        for yy in (y + 1, y, y + 2):
            d = noon - self.tong(yy)
            if 0 <= d < self.sui:
                break
        x = Fr(d) / self.f + self.yingsuo_deg(Fr(d))
        sr = self.sunrise(x)
        if self.rinru:   # 崇天：「朔小餘如日入分已上者，進一日」
            return float((1 - sr) * 24)
        if 6 * self.qi <= d < 18 * self.qi:
            return float((Fr(3, 4) - (self.sunrise_chunfen() - sr) / 3) * 24)
        return 18.0


def xiaoxi_sunrise(p, x):
    """消息定数法（崇天、明天、观天）：距冬至 x（日或度）→ 日出分之比。常数 = a·u²/fa（u 距最近二至），
    定数 = 常数 + (b − 常数)·常数/c；春分后晨分 = p + 定数，秋分后 = q − 定数；日出分 = 晨分 + 昏明。"""
    l = p["l"]
    m = x % (2 * l)
    chunfen_hou = l / 2 <= m < 3 * l / 2
    s = x % l
    u = s if 2 * s <= l else l - s
    chang = Fr(p["a"]) * u * u / p["fa"]
    ding = chang + (p["b"] - chang) * chang / p["c"]
    chen = p["p"] + ding if chunfen_hou else p["q"] - ding
    return (chen + p["hm"]) / p["f"]


LOU = {
    "崇天": dict(l=Fr(18262, 100), a=500, fa=7873, b=Fr(1059, 2), c=2350, p=Fr(7413, 4), q=Fr(11649, 4), hm=Fr(1059, 4), f=10590),
    "观天": dict(l=Fr(18262, 100), a=700, fa=9703, b=Fr(1203, 2), c=2670, p=Fr(8421, 4), q=Fr(13233, 4), hm=Fr(1203, 4), f=12030),
    "明天": dict(l=Fr(182 * 39000 + 24250, 39000), a=2500, fa=10689, b=Fr(1950), c=8650, p=Fr(6825), q=Fr(10725), hm=Fr(975), f=39000),
}
JY_L = Fr(182 * 7290 + 4533, 7290)


def jiyuan_sunrise(x):
    """纪元历：午中日行积度 x → 赤道内外度 δ = 1000·b(L − b)/348856（b = 象限 − [u + u(象限 − u)/517 或 /400]）→
    日出分 = 1822½ ± 3630δ/239（赤道外加、内减），日法 7290。"""
    l, xiang = JY_L, JY_L / 2
    m = x % (2 * l)
    near_dongzhi = m < xiang or m >= 3 * xiang
    s = x % l
    u = s if s <= xiang else l - s
    a = u + u * (xiang - u) / (517 if near_dongzhi else 400)
    b = xiang - a
    dd = Fr(3630) * (Fr(1000) * b * (l - b) / 348856) / 239
    return (Fr(3645, 2) + (dd if near_dongzhi else -dd)) / 7290


JIYUAN = Li("纪元", "宋史/卷079", "宋史/卷079", 7290, 2662626, 215278, Fr(200873 * 10000 + 990, 10000),
            28613466, 1106, 15, fix_sun={3: 188})   # 立春益 187（点校本册六页 1852 同），朒积 965→1153 须 188，对称之谷雨作 188；本项目校改


CHONGTIAN = Li("崇天", "宋史/卷072", "宋史/卷072", 10590, 3867940, 312729, Fr(291803 * 10000 + 594, 10000),
               97556340, 1024, 0, rinru=True,
               fix_moon={17: -601})   # 十八日：宋史作益三百九十一（点校本册五书页 1638 同）；下文按《校证》改作十四日末 233、十八日 591


def guantian_sun(t, f):
    """观天历：入盈缩限（距天正冬至 t 分）→ 朏朒定数（朒 +）。积 = x(2L − x) × 100 / D，x 为距二至或距二分之全日，日内线性。"""
    return guantian_curve(t, f, Fr(100, 366), Fr(100, 407))


def guantian_curve(t, f, k1, k2):
    """观天历日躔曲线 x(2L − x) × k（盈初缩末 k1、盈末缩初 k2）；k 取 100/366、100/407 得朏朒积，取 1/3294、1/3659 得盈缩分（度）。"""
    hz = Fr(182 * f + 7480, f)
    l1, l2 = Fr(88 * f + 10958, f), Fr(93 * f + 8552, f)
    d = Fr(t) / f
    half = 1 if d < hz else -1
    x = d if d < hz else d - hz

    def val(u):
        if half == 1:      # 盈初（366）、盈末（407，距夏至）
            return u * (2 * l1 - u) * k1 if u <= l1 else (hz - u) * (2 * l2 - (hz - u)) * k2
        return u * (2 * l2 - u) * k2 if u <= l2 else (hz - u) * (2 * l1 - (hz - u)) * k1   # 缩初（407）、缩末（366）
    i = int(x)
    return half * (val(Fr(i)) + (val(min(Fr(i + 1), hz)) - val(Fr(i))) * (x - i))


GUANTIAN = Li("观天", None, "宋史/卷077", 12030, 4393880, 355253, Fr(331482 * 10000 + 389, 10000), 5944808, 1092, 0, sun_fn=guantian_sun)
# 统天：转实志作三十二萬六百五十五，与转策 27 余 6655 不合，作 330655（点校本卷八十四校勘记八同）；距算 3830 志未明言（取绍熙五年积算），距差小，影响不足 1 分
TONGTIAN = Li("统天", "宋史/卷084", "宋史/卷084", 12000, 4382910, 354368, 330655, 3835, 1199, 0, v=0, cha={"ju": 3830, "qi": 237811, "run": 21704, "zhuan": 188800},
              delta={8: 360}, end14=260)   # 九日损 44 作 404（朏积 4855→4451），十四日末益 360 作 260（次日朒积 260）
class Mingtian(Li):
    """明天历（周琮，1065—1074）：日月皆用公式，不用立成表（《宋史》卷七十四、七十五）。
    入盈缩：x 距二至度，定差 = 400 × x'(L − x') ÷ 567（x' 取初末，L = 二至限）。
    入转：积月 × 朔差 mod 转终分 ÷ 转度母 = 度；中度以下为疾；x' 取初末，定差 = 10000 × x'(201.09 − x') ÷ 6773.5。"""
    moon_sign = 1       # 从注文「疾加、遲減」；反号则本纪只合 10/25

    def __init__(self):
        self.name, self.f, self.sui, self.shuo, self.ji, self.y_ji, self.day0 = "明天", 39000, 14244500, 1151693, 711760, 1064, 0
        self.qi, self.cha, self.rinru, self.closure = Fr(14244500, 24), {}, False, 0
        self.l = Fr(182 * 39000 + 24250, 39000)

    def sun_def(self, t):
        d = Fr(t % self.sui) / self.f
        half = 1 if d < self.l else -1
        x = d if d < self.l else d - self.l
        x = min(x, self.l - x)
        return half * Fr(400, 567) * x * (self.l - x)

    def moon_of(self, jiyue):
        r = Fr(jiyue * 2142887000 % 29882242251, 81120000)
        zhong = Fr(184) + Fr(15041125 * 2 + 1, 2 * 81120000)
        fast = r < zhong
        x = r if fast else r - zhong
        x = min(x, zhong - x)
        return (1 if fast else -1) * Fr(10000 * 2, 13547) * x * (Fr(20109, 100) - x)

    def moons(self, y0, y1):
        out = []
        for y in range(y0, y1):
            tong = self.tong(y)
            jy = tong // self.shuo
            for m in range(14):
                jing = (jy + m) * self.shuo
                out.append(jing + self.sun_def(Fr(jing - tong)) + self.moon_sign * self.moon_of(jy + m))
        return sorted(set(out))


MINGTIAN = Mingtian()
T83, T84 = "宋史/卷083", "宋史/卷084"
TONGYUAN = Li("统元", T83, T83, 6930, 2531138, 204647, Fr(190953 * 10000 + 2563, 10000), 94251591, 1135, 0, v=0, delta={0: -72})   # 一日益 608，朏积 680 证作 680
QIANDAO = Li("乾道", T83, T83, 30000, 10957308, Fr(88591776, 100), Fr(826637 * 10000 + 7395, 10000), 91645823, 1167, 0, v=1, delta={11: -1000, 24: -1000})   # 十二、二十五日损益率差千，朏朒积证之
CHUNXI = Li("淳熙", T83, T83, 5640, 2059974, Fr(16655256, 100), Fr(155407 * 10000 + 9740, 10000), 52421972, 1176, 0, v=2, end14=127)   # 十四日末益 227，朒积 127 证之
HUIYUAN = Li("会元", T83, T83, 38700, 14134932, 1142834, Fr(1066361 * 10000 + 7310, 10000), 25494767, 1191, 0, v=3)
KAIXI = Li("开禧", T84, T84, 16900, 6172608, 499067, Fr(465672 * 10000 + 5396, 10000), 7848183, 1207, 0, v=1)
CHENGTIAN = Li("成天", T84, T84, 7420, 2710101, 219117, Fr(204455 * 10000 + 1641, 10000), 71758147, 1271, 0, v=2)


# 陈美东《历代律历志校证》（下称《校证》）校月离损益率（点校本《宋史》页码；均经表闭合核验：改后二十八日之后朏朒积归零）。
# 原用表内累计所作的校改与《校证》相合者不重复（统元一日 680、乾道十二日与二十五日、淳熙十四日末 127、统天九日与十四日末、
# 纪元立春 188）；崇天十八日原作 601（并十四日末之差），今从《校证》分作十四日末益 233、十八日益 591。
# 未从：统元二十八日《校证》作「[初]損七十七」（页 1979），与闭合相差 300；淳熙十七日「益四百(二)[三]十六」（页 1975）改后
# 闭合差 10，其朏朒数亦改为 1269，前有未明之处——二条待对点校本原页。
CHONGTIAN.jiaozheng({17: 591}, end14=233)             # 页 1638：十四日末益(二)[三]十三，十八日益(三)[五]百九十一
GUANTIAN.jiaozheng({0: 1179, 1: 1098, 2: 936, 3: 774, 4: 558, 5: 324})   # 页 1808：一至六日
TONGYUAN.jiaozheng({2: 523, 12: 633, 22: 228})        # 页 1970、1974、1977：三、十三、二十三日
QIANDAO.jiaozheng({8: 853, 9: 1391, 21: 539})         # 页 1972、1973、1977：九、十、二十二日
HUIYUAN.jiaozheng({8: 1303})                          # 页 1972：九日
TONGTIAN.jiaozheng({10: 808})                         # 页 2042：十一日（《校证》误标「统元」，卷八十四只统天、开禧、成天）
KAIXI.jiaozheng({17: 1011, 23: 936})                  # 页 2043、2045：十八、二十四日


def run(li, y0, y1, approx, skip=()):
    dz = li.tong(li.y_ji) // li.f
    li.off = c.align_offset(dz + li.day0, g.jdn(*approx)) + li.day0
    print(f"--- {li.name}历：月离闭合 {li.closure}；{li.y_ji} 年天正冬至", g.ganzhi_of_jdn(dz + li.off), g.from_jdn(dz + li.off))
    c.compare(li.name, li.f, li.off, li.moons(y0 - 2, y1 + 3), y0, y1, f"song_{li.name}_check.csv",
              f"{y0}—{y1} 年《宋史》本纪明书朔日与{li.name}历定朔（scripts/calendar_data/song_dingshuo.py）、张表比较；进朔限术文，日出分按本历步晷漏（南宋诸历代用纪元之术）。",
              limit=li.limit, books={"songshi_benji"}, skip=set(skip))


if __name__ == "__main__":
    # 行用年据张培瑜等《中国古代历法》页 606—608 所引宋志：崇天 1024—1064、1068—1074；明天 1065—1067；
    # 纪元 1106—1127、1133—1135、1151—1166；统元 1136—1150、1167。统元、纪元在 1151—1166 年本纪不能区分（皆 47/47）
    run(CHONGTIAN, 1024, 1074, (1023, 12, 18), skip=range(1065, 1068))
    run(JIYUAN, 1106, 1166, (1105, 12, 16), skip=list(range(1128, 1133)) + list(range(1136, 1151)))
    run(MINGTIAN, 1065, 1067, (1063, 12, 16))
    run(GUANTIAN, 1094, 1102, (1091, 12, 16))
    run(TONGYUAN, 1136, 1167, (1134, 12, 15), skip=range(1151, 1167))
    run(QIANDAO, 1168, 1176, (1166, 12, 15))
    run(CHUNXI, 1177, 1190, (1175, 12, 15))
    run(HUIYUAN, 1191, 1198, (1190, 12, 15))
    run(TONGTIAN, 1199, 1207, (1198, 12, 15))
    run(KAIXI, 1208, 1251, (1206, 12, 15))
    run(CHENGTIAN, 1271, 1276, (1270, 12, 14))
