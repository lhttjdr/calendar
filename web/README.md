# 日历 Web 版（SPA）

Vite + React + TypeScript 单页应用，公历/农历逻辑由 `lunar-wasm`（Rust WASM）提供。

## 可选 SPA 库

- **Vite + React**（当前）：主流、生态好，与 wasm 集成简单。
- **Vite + Vue**：上手快。
- **Vite + Svelte**：更轻量。
- **Leptos / Yew**：Rust 全栈，可直接用 lunar-core，需把更多逻辑暴露到 wasm。

## 运行

```bash
# 安装依赖（在 web 目录）
npm install

# 开发
npm run dev

# 构建
npm run build
```

## 农历与 wasm

农历显示需要「岁数据」（`lunar_year`、`new_moon_jds`、`zhong_qi_jds`）。两种方式：

1. **预生成 JSON**：用仓库内脚本或 Rust 工具生成 `public/data/year-YYYY.json`，前端请求 `/data/year-2026.json` 等，并配合 `lunar-wasm` 的 `gregorian_to_chinese_lunar` 显示农历。
2. **后端 API**：由后端（如 Rust actix）加载 VSOP/ELP 数据、调用 `lunar-core` 的 `compute_year_data`，向前端提供岁数据或气朔列表。

当前页面已支持上月/下月/今年；接入 wasm 与岁数据后即可显示农历。
