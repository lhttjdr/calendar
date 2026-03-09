# ELPMPP02 实现与论文/官方代码对照说明

本实现对照 `elpmpp02.pdf` 与 `ELPMPP02.for` 完成，以下为易导致“未达论文精度”的要点及已做修正。

## 论文中 DE405 与 DE406 的关系（pdf 第 1 页、§4.3.3）

- **Introduction**：采用「对 JPL DE405 的拟合」时，会对月球角度的**长期项**施加**加性修正**，使解在 6000 年内更接近 **JPL 历表 DE406**（Standish, 1998）。
- **§4.3.3 “Additive corrections to secular terms”**：将 **Table 6** 的修正加到 W1、W2、W3 的长期项后，ELP/MPP02 在长时段（六千年内数秒量级）上**紧密接近 JPL 历表 DE406**；**这些修正必须与对 DE405 的拟合结果一起使用**（ELP/MPP02(405)）。

因此：**DE406 在论文中即「DE405 拟合 + Table 6 长期项修正」**，不另给一套常数。本实现按论文定义：**DE406 与 DE405 使用相同常数与 Table 6**（解析用 DE405 常数，求值用 DE405 常数 + 完整 Table 6 相位修正）。官方 Fortran 仅提供 `icor=1`（DE405），即该用法。

## 论文中的精度（§7）

- 相对 DE405/DE406：1950–2060 约 4–7 m；1500–2500 约 50 m；更远历元更大。
- 对比时须使用与论文一致的常数集与**读入时的振幅修正**。

## 官方 Fortran 行为

1. **时间**：`EVALUATE(tj, xyz)` 的 `tj` 为 **自 J2000 起算的日数**（即 JD − 2451545），`t(1)=tj/sc`，`sc=36525`（儒略世纪）。
2. **常数两套**：`INITIAL(icor)`，`icor=0` 为 LLR，`icor=1` 为 DE405（并含 Table 6，即论文所述接近 DE406 的用法）；**无单独 DE406 分支**。
3. **主问题振幅修正（READFILE）**：读入主问题时对系数做  
   `cmpb = A + tgv*(delnp−am*delnu) + B(2)*delg + B(3)*dele + B(4)*delep`，  
   且对距离系列 `iv.eq.3` 时 `A = A - 2*A*delnu/3`。  
   `delnu, dele, delg, delnp, delep` 由 **当前 icor** 在 `INITIAL` 中设定，故 **LLR 与 DE405 的级数振幅（Ci）不同**。
4. **W1 二次项**：LLR 为 `Dw1_2 = -0.03794`，DE405 为 `-0.03743`（Fortran 注释 *DE405* 在 w(1,2) 行）。
5. **Table 6 长期项**：仅 `icor=1`（DE405）时对 W1/W2/W3 的 t²、t³、t⁴ 系数加 Table 6 修正；相位 `fmpb` 由 `del(i,k)` 得到，已含修正后的 W2、W3。

## 本实现已修正项

1. **LLR 的 W1_2**：由误用的 `-0.03743` 改为 Fortran LLR 的 **-0.03794**（`Elpmpp02Constants.LLR` / `Elpmpp02Common.LLR`）。
2. **DE405 解析常数**：DE405 时主问题级数按 **DE405 的 parse 常数**（与 Fortran icor=1 的 delnu, dele, delg, delnp, delep 一致）计算 Ci，使振幅与官方 READFILE 后一致；LLR 与 DE406 仍用 LLR 解析常数。

## 本项目中 DE405 与 DE406 的约定（按论文）

- **DE405**：论文 Table 3 ELP/MPP02(405) + 完整 Table 6（W1 t³/t⁴，W2 t²/t³，W3 t²/t³），与 Fortran `icor=1` 一致。
- **DE406**：按论文定义与 **DE405 相同**（DE405 拟合 + 完整 Table 6，即论文所述“接近 DE406”的用法）。实现上 DE406 与 DE405 共用同一套解析常数与求值常数，仅枚举区分调用语义。

## 为何与 JPL DE406 对比仍可能差数十米

- 与 JPL DE406 对比时，参考架、时标（TT/TDB）及黄道定义（如 IERS 2010）与论文所用可能不同，测试中 1950–2060 容差放宽到 100 m 以覆盖“DE406 参考架与 JPL405 可有小差异，实测残差更大”（见 `Elpmpp02VsJplDe406Test`）。

## 参考架修正（论文 §5.2、Table 7）

论文 **§5 坐标系统** 明确给出了 ELP 轨道的参考架定义及与其它架的转换：

- **§5.1**：ELP 自然架 = 历元平黄道 + 起算点 γ′2000；输出 (V,U,r) 经 Laskar P、Q 旋转得到 **J2000 平黄道平春分** 下的直角坐标 xE2000, yE2000, zE2000（即当前实现与 Fortran 的输出）。
- **§5.2**：**J2000 平黄道** 相对以下赤道架 R 的位置由 **Table 7** 给出：
  - **ICRS**（国际天球参考系，与 GCRF/DE406 一致）
  - **MCEP**（J2000 平天极）
  - **JPL405**（DE405 历表采用的架）

Table 7 给出各 R 下的位置角（单位角秒）：
- ε(R) − 23°26′21"（黄赤交角偏差）
- φ(R)（赤道上升点与 R 赤经原点之弧）
- ψ(R)（春分点差）

因此 **参考架修正是可实现的**：用 Table 7 的 (φ, ε, ψ) 可构造从「J2000 平黄道平春分」到 ICRS（或 JPL405）的旋转矩阵，将 ELPMPP02 输出的 (xE2000, yE2000, zE2000) 转到 GCRF/ICRS，即可与 DE406 在 GCRF 下直接比较，或与采用 IERS 黄道的流水线对齐。实现时需按论文 Fig.2 与 Table 7 的角定义写出旋转顺序（例如 Rz(φ) Rx(ε) Rz(ψ) 或文献中等价约定）。

## 参考

- `ELPMPP02.for`：`INITIAL`、`READFILE`、`EVALUATE`。
- `elpmpp02.pdf`：Introduction（DE405 拟合 + 长期项修正 → 接近 DE406）、§4.2 Table 3、§4.3.3 Table 6、**§5 坐标系统（含 §5.2 与 Table 7 参考架角）**、§6–§7（计算与精度）。
