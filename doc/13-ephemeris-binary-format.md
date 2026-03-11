# 历表二进制化与零解析方案

本章描述**本项目历表**（VSOP87、ELPMPP02）的二进制格式与零解析加载。**JPL 星历**的 BSP（SPK）与原始二进制格式、体积对比见 [5-ephemerides-and-de-align.md](5-ephemerides-and-de-align.md) §「JPL 星历的提供形式与格式、体积」。

## 现状与瓶颈

当前 Web 端历表数据流：**TXT (ASCII) → fetch → 解析**。

1. **体积膨胀**：JPL/VSOP87 等文本中科学计数法（如 ` 0.123456789123456789D+03`）约 20–25 字节/数，物理上等价于 8 字节 Float64，纯文本至少浪费约 60% 带宽。
2. **反序列化成本**：大量 `strtod`/`parse::<f64>()` 消耗 CPU；在带 GC 的环境里还会产生大量临时字符串，加重 GC 停顿与掉帧。

目标：**端到端“零解析”——传输二进制，在 WASM 内直接当 `&[f64]` 使用，避免任何字符串转浮点。**

---

## 阶段一：后端/构建阶段“二进制化”

- 在 CI/构建或后端预处理中，将 VSOP87 周期项系数、ELP 主问题/摄动系数等**预先打成二进制文件**。
- **结构**：文件头（魔数、版本、块数、项数等元数据，几十字节）+ 紧接的 **IEEE 754 64-bit 小端 f64 数组**。
- **传输**：可选 **Brotli**：用脚本生成 `.bin.br`，前端优先请求 `url.bin.br` 并用 `DecompressionStream('brotli')` 解压后使用；或由 Web 服务器对 `.bin` 开启 Brotli，网络体积可小于 `.txt.gz`。

---

## 阶段二：WASM 零拷贝式使用

- 前端用 `fetch` 取回 **ArrayBuffer**（或先解压再得到 ArrayBuffer）。
- 将 ArrayBuffer 传入 WASM：
  - 若通过 wasm-bindgen 传入 `Uint8Array`，在 Rust 中收到 `&[u8]`；
  - 跳过头部后，按 8 字节对齐读取为 f64（可用 `from_le_bytes` 逐段转成 `Vec<f64>` 或做安全封装后当 `&[f64]` 使用）。
- **不做**任何“字符串 → 浮点”解析，仅做一次按字节 → f64 的拷贝/解释，计算循环直接在 f64 切片上迭代。

注意：浏览器中 fetch 得到的 ArrayBuffer 不能直接作为 WASM 线性内存的一部分，因此会有一层“拷贝到 WASM 内存”的开销，但相比海量 `strtod` + 临时对象，**一次二进制拷贝 + 无解析**仍能显著降低 CPU 与 GC 压力。

---

## 阶段三：主线程隔离与精度（已实现）

- **Web Worker**：岁数据计算（fetch 历表 + WASM `compute_year_data_full_binary`）在 `yearDataWorker.ts` 中执行；结果通过 `postMessage` 的 **Transferable**（`newMoonJds.buffer`、`zhongQiJds.buffer`）回传主线程，避免阻塞 UI。主线程仅在全二进制路径失败时回退到主线程计算。
- **精度**：Web 端统一使用 Float64；若需更高精度，仅在 WASM 内部用高精度库闭环计算，对外只暴露 64 位结果。

---

## VSOP87 二进制格式（当前实现）

- **魔数**：`VSB1`（4 字节 ASCII），表示 VSOP87 Binary format 1。
- **版本**：`u32` = 1，小端。
- **块数**：`num_blocks: u32`，小端。
- 对每个块：
  - `coord: u8`（1=L, 2=B, 3=R）
  - `alpha_t: u8`
  - 保留 2 字节对齐
  - `term_count: u32`，小端
  - 紧接 `term_count` 个项，每项 3 个 f64 小端：`[amplitude, phase_rad, frequency_rad_per_millennium]`
    - 对 L/B：amplitude 为弧度；对 R：amplitude 为 AU（与 VSOP87 文本一致）。
- 无额外尾部；文件可再整体做 Brotli 等压缩传输。

---

## ELP-MPP02 二进制（已实现）

- 每个文件（ELP_MAIN.S1/S2/S3、ELP_PERT.S1/S2/S3）对应一项列表，二进制格式：
  - 魔数 `ELP1`（4 字节）+ 版本 `u32` + 项数 `u32`；
  - 每项：`ci_arcsec` f64、`fi[5]` f64（弧度）、`alpha` i32、`ilu[4]` i32，共 68 字节。
- `terms_to_binary` / `terms_from_binary` 在 `elpmpp02::parse`；`load_all_from_binary(s1..s3, p1..p3, correction)` 从 6 个 buffer 构建 `Elpmpp02Data`。
- 生成 .bin：`cargo run -p lunar-core --example elpmpp02_to_bin --no-default-features --features twofloat -- ../data/elpmpp02`，在 `data/elpmpp02/` 下生成 6 个 `.bin`。

---

## 文件与 API 约定

| 资源       | 文本（当前）     | 二进制（可选）   | 说明 |
|------------|------------------|------------------|------|
| VSOP87 地心 | `VSOP87B.ear`    | `VSOP87B.ear.bin`| 优先请求 .bin，404 则 .ear |
| ELP_MAIN/PERT | `.S1/.S2/.S3`  | `.S1.bin` 等     | 6 个 .bin 均可用时走 `compute_year_data_full_binary` |
| **章动 tab5.3a** | `data/IAU2000/tab5.3a.txt` | `tab5.3a.bin` / `.bin.br` | `load_iau2000a_from_binary` 或 `try_init_full_nutation_from_binary`；.br 由前端解压后传入 |

- 前端：优先并行 fetch 7 个 .bin；若全部 200 则走 `compute_year_data_full_binary`；否则若 VSOP87 .bin 可用则走 `compute_year_data_from_binary(vsop87_bin, elp_*_text)`；否则全部文本 + `compute_year_data_wasm(...)`。
- 章动：若有 `tab5.3a.bin`（或 .br 解压后），可调用 `try_init_full_nutation_from_binary(bytes)` 启用完整 IAU2000A，与星历表 .bin/.br 用法一致。

---

## 构建脚本

- **一次性生成 VSOP87 + ELP + 章动 tab5.3a 的 .bin**（推荐）：在项目根目录执行  
  `./scripts/gen_ephemeris_bin.sh`  
  默认使用 `./data`，可传参指定数据目录。会生成 `data/vsop87/VSOP87B.ear.bin`、`data/elpmpp02/` 下 6 个 `.bin`、`data/IAU2000/tab5.3a.bin`。
- **分别生成**：在 `rust` 目录下  
  VSOP87：`cargo run -p lunar-core --example vsop87_to_bin --no-default-features --features twofloat -- ../data/vsop87/VSOP87B.ear ../data/vsop87/VSOP87B.ear.bin`  
  ELP：`cargo run -p lunar-core --example elpmpp02_to_bin --no-default-features --features twofloat -- ../data/elpmpp02`  
  章动：`cargo run -p lunar-core --example tab53a_to_bin --no-default-features --features twofloat -- ..`（项目根）
- **Brotli 压缩**（可选）：在项目根目录执行 `node scripts/compress_ephemeris_brotli.mjs [数据目录]`，会在 `data/` 下为每个 .bin（含 `IAU2000/tab5.3a.bin`）生成同名的 .br；前端会优先请求 .bin.br 并解压。
- **前端数据**：在 `web` 目录执行 `npm run copy-data` 会把 `data/vsop87/`、`data/elpmpp02/`、`data/IAU2000/` 拷贝到 `web/public/data/`（含 .txt/.ear、.bin、.br 若存在），供 fetch 使用。

---

## 章动 tab5.3a 二进制格式

- **魔数**：`N53A`（4 字节），版本 `u32` = 1，行数 `u32`。
- **每行**：4 项 ×（14×i32 LE + 2×f64 LE）= 288 字节/行。
- **API**：`Iau2000a::from_binary(bytes)` / `to_binary()`；`load_iau2000a_from_binary(bytes)`；`try_init_full_nutation_from_binary(bytes)` 启用完整章动。
- **.br**：与历表一致，由前端用 `DecompressionStream('brotli')` 解压后传入 `from_binary` 或 `try_init_full_nutation_from_binary`。

上述方案在保持与现有文本管线兼容的前提下，实现“可压缩 + 零解析”的历表与章动加载路径。
