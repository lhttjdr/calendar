JPL 星历数据目录（供 OreKit 读取，用于 ELPMPP02 vs DE405 对比测试）。

重要：OreKit 不使用 .bsp（SPK）格式
--------------------------------------
OreKit 的 JPLEphemeridesLoader 读取的是 JPL DE 的“原始二进制”格式，不是 NAIF 的 SPK/BSP 格式。
- de405.bsp、de406.bsp 等是 SPK 格式，OreKit 不能直接读取。
- OreKit 识别的文件名模式为：[lu]nx[mp]####.ddd（例如 lnx1600.405、lnxp1600p2200.405）。
- 仅有 header.406、testpo.406（ASCII 头/测试）不够，OreKit 需要二进制历表文件（如 lnx*.406、lnxp*.406）。DE406 二进制可从 https://ssd.jpl.nasa.gov/ftp/eph/planets/Linux/de406/ 下载到 data/jpl/de406。

体积对比（DE405）
--------------------------------------
- 原始二进制（OreKit 用）：约 53 MB（单文件全时段）或 13 MB×4 分段。
- BSP（SPK）：de405.bsp 约 62 MB；de405_1960_2020.bsp 约 6.2 MB（时段缩短）。
结论：全时段时原始二进制略小；若只要 1960–2020 则 BSP 有 6.2 MB 小文件，但 OreKit 不能直接用 BSP。

原格式哪里找（官方）
--------------------------------------
FTP 站点：ftp://ssd.jpl.nasa.gov/
  路径示例：pub/eph/planets/Linux/de405/（若不存在可试 eph/planets/Linux/de405/）
  wget 示例：wget -P data/jpl ftp://ssd.jpl.nasa.gov/pub/eph/planets/Linux/de405/lnxp1600p2200.405

- Linux 小端二进制（OreKit 可直接用）：
  HTTPS：https://ssd.jpl.nasa.gov/ftp/eph/planets/Linux/de405/
  推荐：下载 lnxp1600p2200.405（约 53 MB）和 header.405（约 6 KB），放入本目录 data/jpl。
- 或分段：lnx1600.405、lnx1750.405、lnx1900.405、lnx2050.405（各约 13 MB）+ header.405。
- ASCII 源（需自行转二进制）：https://ssd.jpl.nasa.gov/ftp/eph/planets/ascii/de405/
  FTP：ftp://ssd.jpl.nasa.gov/pub/eph/planets/ascii/de405/
  可用 asc2eph 等工具将 ASCII 转为 lnxm/unxm 二进制。

DE405 时间与坐标系（官方与 OreKit）
--------------------------------------
- 时间：DE405 使用 TDB（Barycentric Dynamical Time，质心力学时）。历表积分与存储的时间单位为 TDB 日。
  与 TT 的近似关系：TDB ≈ TT + 约 0.00166 s 量级的周期项（IAU 约定）。
- 坐标系：DE405 定向于 ICRF（International Celestial Reference Frame），即 J2000.0 国际天球参考系。
  内行星+月球+太阳的 DE405 相对彼此及相对 ICRF 的定向精度约 0.001″（Standish IOM 312.F-98-048）。
- 参考原点：JPL 发展历表通常将天体位置相对于太阳系质心（SSB）给出；月球如需地心坐标需自行换算
  （地心月 = 月相对 SSB − 地相对 SSB，或由地月质心 EMB 与质量比推导）。
- 官方说明：JPL IOM 312.F-98-048（E.M. Standish, 1998）；NAIF 摘要：de405.cmt
  https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/a_old_versions/de405.cmt
- OreKit：JPLEphemeridesLoader 使用 TDB（TimeScales.getTDB），支持 DE 4xx 二进制；
  其“Supported data types”页仅列出文件格式与来源，未单独写明时间/坐标系，与 JPL 约定一致。
  https://www.orekit.org/static/data/supported-data-types.html

如何与 ELPMPP02 对比
--------------------------------------
1) 时间
   - ELPMPP02：历表用 TT，驱动为 JD(TT)。
   - DE405/OreKit：历表用 TDB。同一“JD”下 TT 与 TDB 差约 0.00166 s 量级，地月距离约 38 万 km 时约带来 <1 km 量级差异；若需严格同一时刻，可用 IAU 公式把 JD(TT) 转为 TDB 再查 DE405。
   - JD 约定：JD 整数为当日正午，小数部分为自该正午起的日分数（如 JD 2444239.5 = 当日正午起 12 小时 = 子夜）。OreKit createJDDate(day, secondsSinceNoon, scale) 的 secondsSinceNoon 应为 (jd - floor(jd)) * 86400，勿误用 (jd - floor(jd) - 0.5) * 86400（会错成 12 小时）。
2) 坐标系与原点
   - ELPMPP02：输出为 J2000 惯性平黄道、地心月 (x,y,z)，单位 m（或换算为 km）。
   - DE405 文件格式：第 3 列为 Earth-Moon barycenter 相对 SSB；第 10 列为 Moon，中心为 Earth（即地心月，geocentric）。单位 km；OreKit 读入后转为米。
   - elpmpp02.pdf Table 7（第 8 页）与图 2（§5.2）：J2000 惯性平黄道相对于赤道系 R 的位置角。
     * 定义（图 2）：γ_I_2000(R)=黄道在 R 赤道上的升交点；ε(R)=黄道对 R 赤道的倾角；o(R)=R 赤道赤经原点；φ(R)=o(R) 到 γ_I_2000(R) 的弧长；ψ(R)=γ_I_2000 与 ICRS 下升交点之间的弧长。
     * JPL405（DE405 赤道）拟合值（单位角秒，形式误差见表；平均历元 Jan 1990）：
       ε − 23°26′21″ = 0.40960 ± 0.00001  →  全值 ε = 23°26′21″ + 0.40960″
       φ = -0.05028 ± 0.00001
       ψ = 0.0064 ± 0.0003
     * 与 ELPMPP02 比较时：JPL405 赤道坐标 → J2000 平黄道用 Table 7 的 ε、φ：先绕 z 转 -φ，再绕 x 转 ε（R_x(ε)·R_z(-φ)）。
3) 使用 OreKit 时（有理论依据的换算）
   - ICRF 原点为 SSB；Earth_SSB = EMB_SSB − geocentric/(1+emrat)，故 Moon_SSB−EMB_SSB = geocentric×emrat/(1+emrat)，
     即 地心月 = (Moon_SSB−EMB_SSB)×(1+emrat)/emrat。OreKit 取 GCRF 地心月（与 DE405 第 10 列一致）后，用 Table 7 的 ε、φ 转为 J2000 平黄道，与 ELPMPP02 比较。
   - 若自行读 DE 原始系数：可直接用第 10 列地心月（km），或由第 3 列 EMB_SSB 与地心月推 Moon_SSB 后按上式比较。
4) 比较
   - 将 ELPMPP02 在 JD(TT) 下的地心月 (x,y,z) 与 OreKit 得到的地心月 (x,y,z) 在同一单位（km）下比较；容差按 elpmpp02.pdf §7（[1950,2060] 约 4 m，[1500,2500] 约 50 m，±3000 年 10 km）。

测试
--------------------------------------
本仓库测试：Elpmpp02VsJplTest 用 data/jpl（或 data/jpl/de405）下的 DE405 二进制，按上述步骤与 ELPMPP02(DE405) 在若干 JD 上对比。测试前需保证 data/jpl 存在且含匹配文件；可通过 -Dorekit.data.path 指定此目录路径。

Rust 测试（两种方式二选一或并存）：
  (1) 实时 Python 插件（推荐）：elpmpp02_vs_jpl_de406_python。Rust 通过 pyo3 在进程内调用 Python/jplephem 算 DE406 月球位置，内存传递，无需 CSV。
      jplephem 只支持 DAF/SPK 格式，不支持 “JPL PLAN” 头的原始二进制。请用 de406.bsp（如 https://ssd.jpl.nasa.gov/ftp/eph/planets/bsp/de406.bsp，约 287 MB），不要用 lnxm3000p3000.406 等 Linux 二进制（后者为 OreKit 用格式）。
      - 该测试默认参与 cargo test；未设置 PYO3_PYTHON 或未找到 DE406 时在运行时自动跳过。
      - 要实际执行对比（从仓库根目录）：
        uv venv && uv pip install jplephem
        PYO3_PYTHON 建议用绝对路径（否则 --manifest-path 时 Cargo 在 rust/ 下跑 build 会找不到 .venv）。
        运行测试时需：(1) 能加载 libpython，若报错 "cannot open shared object file: libpython3.x.so" 则设置 LD_LIBRARY_PATH；(2) 嵌入的 Python 需找到标准库（PYTHONHOME 为 sys.base_prefix）；(3) 嵌入解释器默认不加载 venv 的 site-packages，若报 "No module named 'jplephem'" 则设置 PYTHONPATH 指向 venv 的 site-packages。示例：
        export PYO3_PYTHON=$PWD/.venv/bin/python
        export PYTHONHOME=$($PYO3_PYTHON -c "import sys; print(sys.base_prefix)")
        export PYTHONPATH=$($PYO3_PYTHON -c "import site; print(site.getsitepackages()[0])")
        export LD_LIBRARY_PATH=$($PYO3_PYTHON -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR') or '')"):${LD_LIBRARY_PATH:-}
        cargo test -p lunar-core --manifest-path rust/Cargo.toml elpmpp02_vs_jpl_de406_python
      - 可选 DE406_PATH=data/jpl 指定历表目录。
  (2) CSV 方案：elpmpp02_vs_jpl_de406_samples 读取 elp_vs_jpl_de406_samples.csv（格式 jd_tdb,x_km,y_km,z_km）。生成：scripts/gen_elp_vs_jpl_de406_samples.py（需 jplephem+DE406）。无需 feature，默认 cargo test 即会运行。
  (3) VSOP87 vs DE406：vsop87_vs_jpl_de406_python。Rust VSOP87B 地心太阳（J2000 平黄道）与 jplephem DE406 同框架比较；需 data/vsop87/VSOP87B.ear、de406.bsp，容差 12 000 km。默认参与 cargo test，无 PYO3_PYTHON/DE406 时运行时跳过。运行方式同 (1)，测试名改为 vsop87_vs_jpl_de406_python。
