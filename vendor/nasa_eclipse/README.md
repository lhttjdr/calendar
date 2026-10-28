# NASA 五千年日月食典目录页（验证基准）

原样保留的第三方数据，不改动内部文件。2026-10-26 下载自 NASA GSFC Eclipse Web Site（Fred Espenak），美国政府作品，公有领域。

| 文件 | 内容 | 来源 |
|---|---|---|
| `SE-0199--0100.html` | 日食目录，−199 ～ −100 | <https://eclipse.gsfc.nasa.gov/SEcat5/SE-0199--0100.html> |
| `SE2001-2100.html` | 日食目录，2001 ～ 2100 | <https://eclipse.gsfc.nasa.gov/SEcat5/SE2001-2100.html> |
| `SE-0299--0200.html` | 日食目录，−299 ～ −200 | <https://eclipse.gsfc.nasa.gov/SEcat5/SE-0299--0200.html> |
| `SE-0099-0000.html` | 日食目录，−99 ～ 0 | <https://eclipse.gsfc.nasa.gov/SEcat5/SE-0099-0000.html> |
| `SE0001-0100.html` | 日食目录，1 ～ 100 | <https://eclipse.gsfc.nasa.gov/SEcat5/SE0001-0100.html> |
| `SE0101-0200.html` | 日食目录，101 ～ 200 | <https://eclipse.gsfc.nasa.gov/SEcat5/SE0101-0200.html> |
| `SE0201-0300.html` | 日食目录，201 ～ 300 | <https://eclipse.gsfc.nasa.gov/SEcat5/SE0201-0300.html> |
| `LE-0199--0100.html` | 月食目录，−199 ～ −100 | <https://eclipse.gsfc.nasa.gov/LEcat5/LE-0199--0100.html> |
| `LE0001-0100.html` | 月食目录，1 ～ 100 | <https://eclipse.gsfc.nasa.gov/LEcat5/LE0001-0100.html> |
| `LE0101-0200.html` | 月食目录，101 ～ 200 | <https://eclipse.gsfc.nasa.gov/LEcat5/LE0101-0200.html> |
| `LE2001-2100.html` | 月食目录，2001 ～ 2100 | <https://eclipse.gsfc.nasa.gov/LEcat5/LE2001-2100.html> |

文献：Espenak F., Meeus J. 2006, *Five Millennium Canon of Solar Eclipses: −1999 to +3000*, NASA/TP-2006-214141；
Espenak F., Meeus J. 2009, *Five Millennium Canon of Lunar Eclipses: −1999 to +3000*, NASA/TP-2009-214172。
月食地影采用 Danjon 法：Rp = 1.01·Pm + Ss + Ps，Ru = 1.01·Pm − Ss + Ps（<https://eclipse.gsfc.nasa.gov/LEcat5/shadow.html>）。
目录中「TD of Greatest Eclipse」为力学时，与 ΔT 无关；ΔT 列为其所用 Espenak–Meeus 多项式值。

用于 `rust/core/src/astronomy/phenomena` 的验证测试；其余各页（−299～−200、−99～300 日食，1～200 月食）用于核对《汉书》《后汉书》日月食检验
（见 `data/phenomena/README.md`）。
