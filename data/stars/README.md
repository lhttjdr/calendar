# 恒星星表

## hipparcos_bright.csv

Hipparcos 肉眼亮星（V ≤ 6.5，另含中国星官引用的 72 颗较暗星，共 8942 颗），由 `scripts/phenomena/hipparcos_bright.py` 从 `vendor/hipparcos/` 原始星表生成：

| 列 | 含义 | 来源 |
|---|---|---|
| `hip` | Hipparcos 编号 | |
| `ra_deg`、`dec_deg` | ICRS 赤经、赤纬（度），历元 J1991.25（TT JD 2448349.0625） | I/311 新归算（van Leeuwen 2007） |
| `plx_mas` | 视差（mas） | I/311 |
| `pmra_mas_yr`、`pmdec_mas_yr` | 自行 μα cos δ、μδ（mas/年） | I/311 |
| `vmag`、`bv` | Johnson V、B−V（B−V 缺失为空） | I/239 原版主表 |
| `astrometry` | 天体测量来源：`I/311`；新归算无解者 `I/239`；两者皆无解（双星、聚星 HIP 55203、78727、115125）者 `I/239-H3H4`，取主表 H3、H4 近似位置，自行、视差记 0 | |

Rust 侧 `astronomy::stars::load_hipparcos_bright_from_repo()` 读取；视位置见 `astronomy::apparent::star_apparent_equatorial`
（空间运动按视向速度 0、周年视差、周年光行差、岁差章动；对 Skyfield 差 ≤ 0.08″）。
