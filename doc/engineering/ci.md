# 持续集成：部署与覆盖率

两个工作流：`.github/workflows/pages.yml`（构建 WASM、网页与读者站，发布 GitHub Pages）、`.github/workflows/coverage.yml`（lunar-core 全量测试与覆盖率）。触发：推送到 master。

## 一、Git LFS 数据

仓库中大批数据表、历表由 Git LFS 管理（`git lfs ls-files`），检出时默认只得到约 130 字节的指针文本「version https://git-lfs…」。两个工作流都只按需 `git lfs pull --include=…`，不拉书影 PDF 等大文件。

| 工作流 | 拉取 | 用途 |
|---|---|---|
| pages | 编进 WASM 的 CSV（`include_str!`：年号、改元、主表分期、平气、民俗月表与月家神煞等）与前端拉取的表 | 指针编进 WASM 不报错、只会让线上数据静默失效，故另有一步逐个检查不是指针 |
| coverage | `data/**/*.csv`、`data/**/*.txt`、`vendor/calendar/**/*.csv`；DE406 BSP（301 MB）见下 | 测试读取的表 |

**DE406 BSP**：VSOP87、ELPMPP02、行星视位置、近距、行星现象等对 DE406 的精度测试要用。以其 LFS oid 为键存 `actions/cache`（restore/save 分开，拉到后立即存，不等作业成功），只在缓存未中时消耗一次 LFS 流量（GitHub 缓存 7 日未用即逐出，coverage 每次推送都跑，正常不会逐出）。

**测试跳过**：测试找历表文件用 `test_util::is_data_file`，存在且不是 LFS 指针才算有；缺则打印 skipped 并返回，不崩溃（2026-11-08 前只判 `is_file()`，指针被当成 BSP 打开，9 个测试崩溃，coverage 长期失败）。

## 二、工具版本

- WASM 构建的 wasm-pack 钉为 0.14.0，与根目录 `mise.toml` 一致（改一处要同步另一处）。原先装「最新」，同一代码时成时败（c8b5c09 成、797cf4d 败，其间 rust/ 与工作流无改动）；疑为 wasm-pack 构建中途从 GitHub Releases 下载 wasm-bindgen CLI、wasm-opt 失败（其下载不重试；未见日志，未核）。现缓存 `~/.cache/.wasm-pack`（键：wasm-pack 版本 + `Cargo.lock`），构建步失败整步重试至多 3 次。
- Rust 用 `dtolnay/rust-toolchain@stable`；本地 1.93.1（mise）与当时 stable 1.99.1 均能构建。

## 三、本地模拟 CI

在不取 LFS 的克隆里只补 CI 所拉的文件，可复现 CI 的数据环境：

```bash
GIT_LFS_SKIP_SMUDGE=1 git clone --depth 1 file://$PWD /tmp/ci
cd /tmp/ci && for f in $(git lfs ls-files -n); do case "$f" in data/*.csv|data/*.txt|vendor/calendar/*.csv) cp "$OLDPWD/$f" "$f";; esac; done
cd rust && cargo test --release -p lunar-core --no-default-features --features real-f64
```

带 `python-test`（CI 用 `--all-features`）时，另需 `PYO3_PYTHON`、`PYTHONHOME`、`PYTHONPATH` 与 `LD_LIBRARY_PATH`（libpython 所在 LIBDIR），见 `scripts/dev/run_tests_with_venv.sh`。
