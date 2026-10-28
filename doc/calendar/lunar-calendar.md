# 10. 农历编算

现代农历按《农历的编算和颁行》GB/T 33661–2017 及 [农历法则](https://ytliu0.github.io/ChineseCalendar/rules_simp.html) 等：**岁以冬至为年首**（一岁 = 冬至到下一冬至），**月以合朔为初一**，**含冬至之月为十一月**；含大寒为十二月、雨水为正月、…、小雪为十月；**无中气之月置闰**（闰月名同上一月）。详见国标与《月相和二十四节气的计算》相关章节。

## 流程顺序

1. 排格里高利历 → 2. 取得岁数据（14 朔 + 12 中气，UTC+8）→ 3. 排农历置闰 → 4. 排干支历

## 整月一致性

整月只算一次朔日日号 `new_moon_day_numbers_utc8(year_data)`，逐日查表用 `from_julian_day_in_year(jd, year_data, Some(&precomputed))`，避免循环内多次 TT→UTC 导致日界漂移（如两个初一）。

## 分层

- **core**：结构化 ChineseLunarDate，无汉字
- **wasm**：结构化返回
- **显示层**：汉字/繁简/多语言，不兜底改初一初二

（农历规定与国标见 `doc/references/` 及仓库 wiki。）
