# 《中国历史纪年表》（方诗铭）—— 原始来源材料

方诗铭编著《中国历史纪年表》（新修订本，上海书店出版社）。本目录仅存放**原样保留的来源材料**，不做任何
结构化整理：

- `中国历史纪年表 (方诗铭) (z-library.sk, 1lib.sk, z-lib.sk).pdf`：原书 PDF 扫描版。
- `pages/page_001.png` – `pages/page_175.png`：PDF 逐页导出的图片。

**图片页码与书内印刷页码为固定偏移：图片页码 = 书内印刷页码 + 5**（已用书内第 2/34/63/169 页等多组页面交叉
验证）；每页顶部印有形如「（N）中国历史纪年表·XX纪年表」的页眉，可据此核对。

**结构化数据在别处**：本书的结构化转录（宽表 CSV/JSON，按章节拆分，含逐条复核记录）属于本项目自己生成/整理
的数据，按仓库「vendor 原样保留 / data 本项目整理」的惯例存放在
[`data/calendar/fang_shiming/`](../../../data/calendar/fang_shiming/README.md)，详见该目录 README。

**旧产物已废弃**：本目录曾有 `方诗铭纪年表.xlsx`、`ocr_pages.jsonl`，是早期用 rapidocr 自动 OCR 的产物，精度
很差、不可用，已删除。如需重新生成结构化数据，请直接对 `pages/*.png` 逐页视觉识别（而非用 OCR 工具），参照
`data/calendar/fang_shiming/README.md` 中记录的转录约定。
