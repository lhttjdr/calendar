备份与编译（gfortran）
======================

1) 备份原版（若尚未备份）：
   cp ELPMPP02.for ELPMPP02.for.bak

2) 当前 ELPMPP02.for 已加入 DEBUG 输出：
   - JD 2444239.5 (tj≈-7305.5)：v_main(1:3)、full v(1)v(2)v(3)、after W1、x1,x2,x3
   - JD 2500000.5 (tj≈48455.5, DE405)：full v(1)v(2)v(3)、after W1、x1,x2,x3、X,Y,Z

3) 编译（需在含 ELP_MAIN.S1 等数据的目录下）：
   cd data/elpmpp02
   gfortran -o elpmpp02_exe ELPMPP02.for

4) 运行：
   ./elpmpp02_exe
   程序会先算 21 个 LLR 日期（含 JD 2444239.5），再算 15 个 DE405 日期（第一个即 JD 2500000.5）。
   在终端里搜 "DEBUG JD2500000.5" 可得该 JD 的 4 行：
     DEBUG JD2500000.5 full v(1)=... v(2)=... v(3)=...
     DEBUG JD2500000.5 after W1: v1_rad=... v2_rad=... v3_km=...
     DEBUG JD2500000.5 x1=... x2=... x3=...
     DEBUG JD2500000.5 X=... Y=... Z=...

5) 供校验用（Fortran JD 2500000.5 中间量）：
   顺序为 (v1_arcsec, v2_arcsec, v3_elp, lon_rad, lat_rad, r_km, x1, x2, x3, X, Y, Z)。
   - v1_arcsec,v2_arcsec,v3_elp = 上面 full v(1), v(2), v(3)
   - lon_rad,lat_rad,r_km = 上面 after W1 的 v1_rad, v2_rad, v3_km
   - x1,x2,x3 = 上面 x1,x2,x3
   - X,Y,Z = 上面 X,Y,Z
   可用于与 Rust 或其它实现的中间量逐个核对。

6) 恢复原版：
   cp ELPMPP02.for.bak ELPMPP02.for
