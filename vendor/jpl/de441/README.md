# JPL DE441（仅本地验证用，不入库）

`de441.bsp`：JPL Planetary and Lunar Ephemeris DE441，NAIF SPK 格式，3 307 878 400 字节，2020-12-22 发布，覆盖前 13200—17191 年。
下载自 <https://ssd.jpl.nasa.gov/ftp/eph/planets/bsp/de441.bsp>（2026-10-26），原样保留。

体积大，按项目决定不入库（`.gitignore` 排除 `vendor/jpl/de441/*.bsp`），也不进 wasm；只用于本地验证（天象模块、节气参考表对照）。
需要时在仓库根执行：

```bash
curl -C - -o vendor/jpl/de441/de441.bsp https://ssd.jpl.nasa.gov/ftp/eph/planets/bsp/de441.bsp
```

Rust 侧用 `De406Kernel::open` 读取（按需读记录，不整体载入内存），路径常量 `repo::paths::DE441_BSP`。

文献：Park R.S., Folkner W.M., Williams J.G., Boggs D.H. 2021, The JPL Planetary and Lunar Ephemerides DE440 and DE441,
*AJ* 161, 105. <https://doi.org/10.3847/1538-3881/abd414>
