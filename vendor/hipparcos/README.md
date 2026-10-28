# Hipparcos 星表（原样保留）

自 CDS 存档下载（2026-10-27），未作任何改动：

| 文件 | 内容 | 来源 |
|---|---|---|
| `I_311/hip2.dat.gz`、`I_311/ReadMe` | Hipparcos 新归算天体测量星表（117 955 颗；ICRS 位置历元 J1991.25、视差、自行、Hp、B−V） | <https://cdsarc.cds.unistra.fr/ftp/I/311/> |
| `I_239/hip_main.dat`、`I_239/ReadMe` | Hipparcos 主表原版（118 218 颗；含 Johnson V、B−V） | <https://cdsarc.cds.unistra.fr/ftp/I/239/> |

`I_239/hip_main.dat` 53 MB，不入库（`.gitignore`）；需要时在仓库根执行：

```bash
curl -o vendor/hipparcos/I_239/hip_main.dat https://cdsarc.cds.unistra.fr/ftp/I/239/hip_main.dat
```

用途：`scripts/phenomena/hipparcos_bright.py` 合并两表，生成 `data/stars/hipparcos_bright.csv`（V ≤ 6.5，8870 颗；新归算
无解的 4 颗——HIP 31067、55203、78727、115125——未收），供 `rust/core/src/astronomy/stars` 使用。位置、自行、视差取新归算，
V 与 B−V 取原版（新归算只有 Hp）。

文献：

- van Leeuwen F. 2007, Validation of the new Hipparcos reduction, *A&A* 474, 653. doi:10.1051/0004-6361:20078357
- ESA 1997, *The Hipparcos and Tycho Catalogues*, ESA SP-1200.
