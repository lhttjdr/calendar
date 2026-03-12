# DE406 定气 vs TDBtimes 失败原因分析

## 现象

- 测试 `solar_term_2026_de406_vs_tdbtimes` 与参考表 TDBtimes.txt（DE441 + IAU2006 + IAU2000A）对照时，春分残差约 **+594 s**（本实现晚于参考），参考春分时刻处本实现视黄经约 **-24.62″**（尚未到 0°）。

## 根因：tab5.3a 章动列顺序解析错误

### 1. 公式与表格式

- IERS 表 5.3a 与项目 data/IAU2000/tab5.3a.txt 约定：  
  **Δψ = A_i·sin(ARG) + A"_i·cos(ARG)**  
  即「In Phase」= sin 系数，「Out of phase」= cos 系数。

- 表头为两段各 4 列：  
  **In Phase**：Psi(6)、dPsi/dt(7)、Eps(8)、dEps/dt(9)；  
  **Out of phase**：Psi(10)、dPsi/dt(11)、Eps(12)、dEps/dt(13)。

### 2. 原实现错误

- 原代码将 **列 6 当作 cos、列 7 当作 sin**，且 ψ 的 cos 系数误取列 7 而非列 10，ε 与速率列也按错误的「cos/sin 交替」顺序读取。
- 后果：完整 IAU2000A 章动中 **Δψ 的 sin/cos 与 IERS 不一致**，主项约 -17.2″ 被错误地加在 cos(Ω) 上而非 sin(Ω) 上，等效约 90° 相位错误，春分时刻系统偏晚约 **24″**，对应约 **594 s**。

### 3. 修复

- 在 `rust/core/src/astronomy/frame/nutation/table_parser.rs` 中按表头约定修正列映射：
  - **ψ**：psi_in = 列 6（sin），psi_out = 列 10（cos）；d_psi_in = 7，d_psi_out = 11。
  - **ε**：eps_in = 列 8（sin），eps_out = 列 12（cos）；d_eps_in = 9，d_eps_out = 13。
- 与 IERS 表 5.3a 及 77 项公式 Δψ = A·sin + A'·cos 一致。

### 4. 修复后结果

- 参考春分 JD(TDB)=2461120.116049 处，本实现视黄经由 **-24.62″** 变为 **+0.38″**。
- 春分残差由 **+594 s** 降为 **约 -9.3 s**（本实现略早于参考）。

## 剩余约 9 s 差异（应在约 0.7 s）

- 预期：DE406 vs DE441 历表差异约 **0.7 s**；当前残差 **约 9 s**，多出约 8 s 需解释。
- 现象：在参考春分 JD(TDB) 处，本实现视黄经 = **+0.38″**（应为 0″），即本实现系统偏早约 9 s。
- 已排除：
  - **光行时迭代**：max_iter 从 2 增至 5、10，诊断 0.38″ 与残差 9.3 s 不变，故非光行时收敛不足。
  - **TT vs TDB 历元**：岁差/章动按规定用 TT；同一物理时刻 TT 与 TDB 差约 2 ms，不足以产生 9 s。
  - **TT–TDB 换算公式**：周期项约 1.7 ms，无遗漏大项。
- 诊断输出（参考春分 JD(TDB)=2461120.116049）：
  - 本实现 (Δψ, Δε) = (6.63″, -6.18″)；视黄经 = +0.38″。
  - 可与 SOFA `iauNut00a` 在同一时刻（TDB→TT 得 t_cent）对比 (Δψ, Δε)，若一致则 0.38″ 可能来自历表差，若不一致则章动仍有偏差。
- 待查：
  1. **章动**：用 SOFA `iauNut00a` 在参考春分 t_cent 算 (Δψ, Δε)，与本实现 (6.63″, -6.18″) 对比；或逐项核对 tab5.3a 列序/单位。
  2. **历表**：若参考表生成脚本可改为 DE406，则可在「同历表」下比对，区分 9 s 中历表部分与架/章动部分。
  3. **参考表定义**：确认参考表「TDB 时刻」是「λ=0 的 TDB」且岁差/章动所用时间自变量（TT 或 TDB）与文档一致。

## 小结

| 项目         | 修复前     | 修复后   |
|--------------|------------|----------|
| 参考春分处视黄经 | -24.62″    | +0.38″   |
| 春分残差     | +594 s     | 约 -9.3 s |
| 根因         | tab5.3a Δψ/Δε 列顺序与 IERS 不一致 | 列映射已按表头修正 |

---

## 77 项 vs 完整表、权威表核实（2025-03）

### 权威来源

- **IERS Conventions**：<https://iers-conventions.obspm.fr/chapter5.php>  
  - Table 5.3a：经度 Δψ，单位 µas，列序 `A_i, A"_i, l, l', F, D, Om`（与项目 VLBI 合并表格式不同）。  
  - Table 5.3b：倾角 Δε，**Δε = B_i·cos(ARG) + B"_i·sin(ARG)**，列序 `B"_i, B_i, ...`。  
- **SOFA** `iauNut00a`：luni-solar 用 `dp += (sp+spt*t)*sin + cp*cos`，`de += (ce+cet*t)*cos + se*sin`，即 Δε = ce·cos + se·sin（与 IERS 5.3b 一致）。

### 本项目 VLBI 表与实现

- `data/IAU2000/tab5.3a.txt` 为「NUTATION SERIES FROM VLBI DATA」合并格式：每行 5 乘数 + Period + 8 列 (mas)，表头 In Phase(sin) / Out of phase(cos) 对 Psi、Eps 各 4 列。
- **Δε 公式**：IERS 5.3b 为 B·cos + B''·sin；表中 Eps 列为 (B=col8, B''=col12)。实现中已按 **deps += eps_in·cos + eps_out·sin** 使用（`iau2000a.rs`），与 77 项及 IERS 一致。
- **第 78 项起 ε 取反**：已移除。IERS 官方表与 SOFA 源码均无「从第 78 项起对 ε 取反」的约定；与 VLBI 表一致不再取反。

### 前 77 项一致性

- 测试 `tab53a_first_77_matches_nutation_77`：加载 tab5.3a，用 **前 77 项** 在 t=0 算 (Δψ, Δε)，与内联 `nutation_77(0)` 比较，**Δψ/Δε 差 &lt; 0.01″**，通过。
- 结论：**列序与 Δε 公式对前 77 项正确**；与 77 项序列一致。

### 约 9 s 的归属

- 用 **77 项章动** 时春分残差约 **+0.6 s**（满足 1 s 容差）；用 **完整 tab5.3a** 时约 **-9.3 s**。
- 因此约 9 s 来自 **第 78 项及之后的项**：当前 VLBI 格式表与 SOFA/IERS 在项序或符号上可能不一致，或需用 IERS 官方 tab5.3a/5.3b（µas、经度/倾角分表）重新生成合并表后再对 78+ 项逐项核对。

### 测试策略

- **DE406 定气 vs TDBtimes** 的测试（容差 1 s）当前使用 **77 项章动**，保证可重复通过。
- 待完整表 78+ 项与 IERS/SOFA 核对或换用权威表后，再切回完整章动并确认残差 ≤1 s。
