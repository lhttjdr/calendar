"""唐宝应五纪历（762—783）、建中正元历（784—806）定朔（《新唐书·历志五》卷二十九），复核主表与本纪明书朔日。

术同大衍（「以写大衍旧术」），定朔用大衍非交会简法：入定气线性插值损益率，入转线性插值损益率。正元「气朔、发敛、日躔、月离、轨漏、交会，悉如五纪法」。
进朔（五纪历志）：「仲夏之朔，若月行极疾，合于亥正，朔不进……当视定朔小余不满五纪通法，如晨初余数减十刻已下者，进以明日为朔」，
即距次日夜半不过（晨初余数 − 十刻）者进；晨初余数即夜半定漏。此二历不载漏表（「与大衍同」），用大衍历定气初夜半漏，定气间线性（假定）。

正元揲法各本作三万三千三百三十六（合朔望月 30.44 日，误），从《合钞》作 32336：点校本《新唐书》卷二十九校勘记八（册三页 737）云「以通法除揲法，得四象之策。合钞是」。

用法（仓库根）：uv run --project scripts python scripts/calendar_data/wuji_zhengyuan_dingshuo.py
"""
import sys
from fractions import Fraction as Fr
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import dingshuo_check as c  # noqa: E402
import gaiyuan_date as g  # noqa: E402

# 大衍历定气初夜半漏（刻，分母 480），冬至至夏至，夏至后对称；春分、芒种从校勘记（底本 230、335）
LOU = [Fr(27 * 480 + a, 480) for a in (230, 135)] + [Fr(26 * 480 + 380, 480), Fr(25 * 480 + 475, 480), Fr(24 * 480 + 470, 480),
       Fr(23 * 480 + 360, 480), Fr(22 * 480 + 240, 480), Fr(21 * 480 + 120, 480), Fr(20 * 480 + 10, 480), Fr(19 * 480 + 5, 480),
       Fr(18 * 480 + 100, 480), Fr(17 * 480 + 345, 480), Fr(17 * 480 + 250, 480)]


class Li:
    def __init__(self, name, f, ce, she, zhuan, miao, ji, y_ji, xianhou, sun, moon, split, approx):
        self.name, self.f, self.ce, self.she, self.miao = name, f, ce, she, miao
        self.zhuan, self.ji, self.y_ji, self.split = zhuan, ji, y_ji, split
        self.qi = Fr(ce, 24)
        self.xh = list(xianhou) + [-x for x in xianhou]
        self.sun = sun + [-v for v in sun]
        self.moon = moon
        self.approx = approx
        acc, self.mji = 0, []
        for i, v in enumerate(moon):
            self.mji.append(acc)
            acc += split[i][1] + split[i][3] if i in split else v
        assert acc == 0, (name, acc)

    def moon_def(self, x):
        i, r = int(x // self.f), x % self.f
        if i in self.split:
            n0, r0, n1, r1 = self.split[i]
            return self.mji[i] + (r0 * r / n0 if r <= n0 else r0 + r1 * (r - n0) / n1)
        return self.mji[i] + self.moon[i] * r / self.f

    def ding_qi(self, k):
        return k * self.qi - self.xh[k % 24]

    def seg(self, t):
        k = int(t // self.qi) - 1
        while self.ding_qi(k + 1) <= t:
            k += 1
        return k

    def sun_def(self, t):
        k = self.seg(t)
        return sum(self.sun[:k % 24]) + self.sun[k % 24] * (t - self.ding_qi(k)) / (self.ding_qi(k + 1) - self.ding_qi(k))

    def tong(self, y):
        return self.ce * (self.ji + (y - self.y_ji))

    def moons(self, y0, y1):
        out = []
        for y in range(y0, y1):
            tong = self.tong(y)
            t0 = tong - tong % self.she
            for m in range(14):
                jing = t0 + m * self.she
                rz = Fr((t0 * self.miao + m * self.she * self.miao) % self.zhuan, self.miao)
                out.append(jing + self.sun_def(Fr(jing - tong)) + self.moon_def(rz))
        return sorted(set(out))

    def limit(self, t):
        """定朔 t 的进朔限（时）= 24 −（夜半漏 − 十刻）× 0.24。"""
        y = g.from_jdn(int(t // self.f) + self.off)[0]
        for yy in (y + 1, y, y + 2):
            d = t - self.tong(yy)
            if 0 <= d < self.ce:
                break
        k = self.seg(d)
        a, b = k % 24, (k + 1) % 24
        la, lb = LOU[a if a <= 12 else 24 - a], LOU[b if b <= 12 else 24 - b]
        lou = la + (lb - la) * (d - self.ding_qi(k)) / (self.ding_qi(k + 1) - self.ding_qi(k))
        return float(24 - (lou - 10) * Fr(24, 100))


WUJI = Li("五纪", 1340, 489428, 39571, 1366156, 37, 269978, 762,
          [0, 1037, 1850, 2463, 2893, 3152, 3246, 3152, 2893, 2463, 1850, 1037],
          [78, 61, 46, 32, 19, 7, -7, -19, -32, -46, -61, -78],
          [-135, -117, -99, -78, -56, -33, None, 14, 38, 62, 85, 103, 118, None, 128, 115, 95, 74, 52, 28, None, -20, -42, -65, -89, -109, -125, None],
          {6: (1191, -8, 149, 1), 13: (1042, 105, 298, 30), 20: (892, 6, 448, -3), 27: (743, -75, 597, 0)}, (761, 12, 19))
ZHENGYUAN = Li("正元", 1095, 399943, 32336, 301720132, 10000, 402900, 784,
               [0, 848, 1512, 2013, 2364, 2576, 2653, 2576, 2364, 2013, 1512, 848],
               [63, 50, 37, 26, 16, 6, -6, -16, -26, -37, -50, -63],
               [-110, -96, -81, -64, -46, -27, None, 12, 31, 51, 68, 85, 96, None, 107, 94, 78, 61, 42, 23, None, -16, -35, -53, -71, -88, -102, None],
               {6: (973, -7, 122, 1), 13: (851, 87, 244, 25), 20: (729, 5, 366, -2), 27: (607, -68, 488, 0)}, (783, 12, 19))


def main():
    for li, y0, y1, csv_name in ((WUJI, 762, 783, "wuji_dingshuo_check.csv"), (ZHENGYUAN, 784, 806, "zhengyuan_dingshuo_check.csv")):
        dz = li.tong(li.y_ji) // li.f
        li.off = c.align_offset(dz, g.jdn(*li.approx))
        print(f"--- {li.name}历 {li.y_ji} 年天正冬至", g.GANZHI[dz % 60], g.from_jdn(dz + li.off))
        c.compare(li.name, li.f, li.off, li.moons(y0 - 2, y1 + 3), y0, y1, csv_name,
                  f"{y0}—{y1} 年本纪明书朔日与{li.name}历定朔（scripts/calendar_data/wuji_zhengyuan_dingshuo.py）、主表比较；进朔限据五纪历志，夜半漏用大衍表。",
                  limit=li.limit)


if __name__ == "__main__":
    main()
