# 世界磁场模型 WMM2025（原样保留）

| 文件 | 内容 | 来源 |
|---|---|---|
| `WMM.COF` | WMM2025 高斯系数（n ≤ 12）及年变率；首行 `2025.0 WMM-2025 11/13/2024` | https://www.ncei.noaa.gov/sites/default/files/2024-12/WMM2025COF.zip |
| `WMM2025.COF` | 同上（与 `WMM.COF` 逐字节相同） | 同上 |
| `README-WMM-COEFS.txt` | 系数包说明（2024-12-17） | 同上 |
| `WMM2025_TestValues.txt`、`WMM2025_TestValues.pdf` | 官方测试值 100 点（全精度，含长期变化） | 同上 |
| `WMM2025_TEST_VALUES.txt` | 技术报告所载测试值 12 点（印到 0.1 nT、0.01°） | https://www.ncei.noaa.gov/sites/default/files/2025-02/WMM2025_TEST_VALUES.txt |
| `WMM2025testvalues.pdf` | 同上 PDF | https://www.ncei.noaa.gov/sites/default/files/2025-02/WMM2025testvalues.pdf |

- 发布者：美国国家海洋和大气管理局国家环境信息中心（NOAA/NCEI）与英国地质调查局（BGS），模型页 https://www.ncei.noaa.gov/products/world-magnetic-model ；技术报告 https://library.oarcloud.noaa.gov/noaa_documents.lib/NESDIS/NCEI/WWM_Report_2025.pdf 。
- 许可：美国政府作品，公有领域（NCEI 模型页称 WMM 及其软件可免费使用）。
- 取得日期：2026-11-08。文件未经改动。
- 有效期 2025.0—2030.0。本项目用法见 `rust/core/src/astronomy/geomag.rs`、`doc/astronomy/geomag.md`。
