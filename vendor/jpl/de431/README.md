# JPL DE431（仅本地验证用，不入库）

`de431t.bsp`：JPL Planetary and Lunar Ephemeris DE431（含 TT−TDB 段），NAIF SPK 格式，3 696 366 592 字节，2014-06-30 发布，
覆盖前 13200—17191 年。下载自 <https://ssd.jpl.nasa.gov/ftp/eph/planets/bsp/de431t.bsp>（2026-10-26），原样保留。

用途：DE431 是 DE430 的长时段版本（同一套常数与月球潮汐模型）。Morrison–Stephenson 2016/2021 的 ΔT 以 DE430 的月球潮汐
加速度 ṅ = −25.82″/cy² 为准（Stephenson et al. 2016 §2），故以 DE431 为基准标定本仓库月历表（ELPMPP02）的月亮黄经差，
据此做 ΔT 与月历表的配套修正（见 `rust/core/src/astronomy/apparent/tests_lunar_ndot.rs`、
`doc/astronomy/celestial-phenomena.md` §九 P3）。

体积大，不入库（`.gitignore` 排除 `vendor/jpl/de431/*.bsp`），不进 wasm。需要时在仓库根执行：

```bash
curl -C - -o vendor/jpl/de431/de431t.bsp https://ssd.jpl.nasa.gov/ftp/eph/planets/bsp/de431t.bsp
```

Rust 侧 `De406Kernel::open` 读取，路径常量 `repo::paths::DE431_BSP`。

文献：Folkner W.M., Williams J.G., Boggs D.H., Park R.S., Kuchynka P. 2014, The Planetary and Lunar Ephemerides DE430 and DE431,
IPN Progress Report 42-196（按通行引文著录，未核对原件）.
