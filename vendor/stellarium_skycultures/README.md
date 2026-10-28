# Stellarium 中国星空文化（原样保留）

自 Stellarium 仓库 `skycultures/` 取下（commit `8e3062e220808537936e492b108dabaed0c3faa4`，2026-10-13），未作改动：

| 目录 | 体系 | 主要依据（见各 description.md 的 References） |
|---|---|---|
| `chinese_chenzhuo/` | 陈卓星表（汇集汉代石氏、甘氏、巫咸三家观测，约 3 世纪） | 杨伯顺新发现的陈卓坐标数据，《三家星经》、天象列次分野之图等 |
| `chinese_song_dynasty/` | 宋皇祐四年（1052）观测，《新仪象法要》星图 | 潘鼐《中国恒星观测史》（2009）复原的皇祐星表；齐锐、万昊宜《漫步中国星空》 |
| `chinese_yuan_dynasty/` | 元《天文汇钞》星表 | 潘鼐发现（1983），潘鼐、曹军、杨伯顺校补 |
| `chinese_xianglin/` | 明陈荩谟《象林》（1634） | 潘鼐《中国古天文图录》《中国恒星观测史》 |
| `chinese/` | 清《仪象考成》（1756）及续编（1844） | 伊世同《中西对照恒星图表 1950.0》 |

每个目录含 `index.json`（星官连线、星名、`lunar_system.defining_stars` 二十八宿距星）与 `description.md`。

许可证：各 description.md 均声明 **CC BY-SA 4.0**。由此提取的 `data/sky_culture/chinese_lunar_mansion_determinatives.csv`
同受 CC BY-SA 4.0 约束（署名 Stellarium 各星空文化作者，相同方式共享）。

已知问题：`chinese/description.md` 的二十八宿表把斗宿距星写作 μ Sgr，而同目录 `index.json` 的 `defining_stars` 与星名
「斗宿一」均为 φ Sgr（HIP 92041）；`lunar_system` 各体系均注「Needs confirmation」。本项目以 `index.json` 为准，
并以《元史》实测宿度检验（见 `doc/astronomy/celestial-phenomena.md`）。
