# 15. 管线图：节点与边的分类、抽象与状态机设计

本章说明视位置计算管线中**变换图（Transformation Graph）**的底层抽象设计。系统通过严格区分节点的「物理语义」与「计算状态」、以及边的「概念分类（Kind）」与「执行形式（Form）」，构建出支持重边、无隐式回退、且类型安全的数据流驱动引擎。

---

## 1. 核心架构升级：从「参考架图」到「状态转移图」

在纯物理定义中，坐标变换仅涉及空间原点的平移或坐标轴的旋转（Frame 转换）。然而在工程落地的历表拟合（Fit）中，常存在**「坐标轴不转，仅对数值残差进行修正」**的操作（如黄道 DE406 Patch）。若将此类修正表示为同节点的自环边（MeanEcliptic → MeanEcliptic），会导致：最短路算法（Dijkstra/BFS）排斥自环、路径规划无法表达「必须/禁止经过该修正」；且状态无法区分「已修正」与「未修正」，存在双重修正风险。

为解决图论算法约束与类型系统防呆需求，管线图的**节点（Node）**语义被升级：**节点不再仅表示「参考架」，而表示「管线数据状态」**。这样，没有架的变换（同架纯数值修正）时就不必、也不应引入「中间架」——我们引入的是**虚拟状态节点**，而非新的物理架。

**管线图的节点 ≡ 管线数据状态（Pipeline Data State）**

节点分为两类：

1. **物理标架节点（Physical Frame Nodes）**：对应真实物理参考系（如 `ICRS`、`FK5`、`MeanEquator(epoch)`、`MeanEcliptic(epoch)`），既是架也是状态。
2. **虚拟状态节点（Virtual State Nodes）**：**不表示新架**；在物理上与某基础架共享相同的坐标轴和原点，但在图中代表**被特定算法处理后的独立数据状态**（如 `MeanEclipticEclipticPatch(epoch)`），仅用于状态区分与寻路。

**设计收益**：消灭自环边，保证最短路算法稳定；支持通过 `Via::Node(虚拟节点)` 强制路由；一旦数据经过修正边，其 FrameId 变为虚拟节点，可防止重复执行同一拟合。

---

## 2. 节点的分类：原点角色（OriginRole）与类型隔离

除节点 ID 外，图中每个节点可标注**原点角色（OriginRole）**，定义该状态的几何中心：

| 角色 | 物理含义 | 典型节点示例 | 核心用途 |
|------|----------|--------------|----------|
| **日心** | 以太阳质心为原点 | VSOP87、日心_MeanEcliptic(epoch)、Vsop87De406EclipticPatch | 校验视差与平移。 |
| **地心** | 以地球质心为原点 | MeanEcliptic(epoch)、ICRS、FK5、MeanEquator(epoch)、ApparentEcliptic(epoch) | 判定站心平移等。 |
| **质心** | 太阳系质心（SSB） | 可留作高精度扩展 | 明确引力势与光行差基准。 |

*注：物理架 `MeanEcliptic` 与虚拟节点 `MeanEclipticEclipticPatch` 共享同一原点角色（地心）。*

历表入口等概念节点（如 VSOP87、ELPMPP02、ELPMPP02_MEAN_LUNAR）仅用于可视化与语义标注，最短路在参考架节点间进行。实现：`OriginRole` 枚举与 `node_origin(node_id)`；可视化数据提供 `node_origins: Vec<(String, OriginRole)>`（`rust/core/src/astronomy/pipeline/transform_graph.rs`）。

---

## 3. 图的静态拓扑与关键连通性

图由 `default_graph()` 固定给出，展示与运算共用同一拓扑；支持**重边（Parallel Edges）**以容纳不同物理模型（岁差 P03/Vondrak2011、章动 IAU2000A/IAU2000B）。

### 3.1 黄道 → 赤道段（含虚拟状态节点的路由解耦）

不指定途径点时走直连；指定 via 经 Patch 时，走代价更低的虚拟节点路径。

| 起点 | 终点 | 代价 | 边类型 (Form) | 含义 |
|------|------|------|---------------|------|
| MeanEcliptic | FK5 | 8 | Rotation | **直连**：黄赤交角 R_x(ε₀) 一次旋转到平赤道。 |
| MeanEcliptic | MeanEclipticEclipticPatch | 0 | Mapping | **拟合入口**：同标架下 L,B,R 的 DE406 数值修正（进入已修正状态）。 |
| MeanEclipticEclipticPatch | FK5 | 4 | Rotation | **拟合出口**：修正后黄道经 R_x(ε₀) 到平赤道。 |

### 3.2 FK5 ↔ ICRS

| 起点 | 终点 | 代价 | 含义 |
|------|------|------|------|
| FK5 | ICRS | 1 | Frame bias + 可选赤道 DE406 拟合（由 `with_fk5_to_icrs_mapper` 设置）。 |
| ICRS | FK5 | 1 | 逆：GCRF → FK5。 |

执行层为两节点一条边；可视化层可在渲染数据中虚拟插入「拟合节点」展示为 FK5 → [复合拟合] → ICRS。

### 3.3 赤道链（岁差 → 章动 → 视黄道，重边）

同一对节点可有多条边，用 `edge_key` 区分；路径返回与执行时带 `edge_key` 指定走哪条。

| 起点 | 终点 | 代价 | edge_key | 含义 |
|------|------|------|----------|------|
| FK5 | MeanEquator(epoch) | 10 | "P03" | 岁差（IAU 2006 P03）。 |
| FK5 | MeanEquator(epoch) | 10 | "Vondrak2011" | 岁差（Vondrak 2011 长期）。 |
| MeanEquator(epoch) | TrueEquator(epoch) | 5 | "IAU2000A" | 章动（IAU 2000A 完整表）。 |
| MeanEquator(epoch) | TrueEquator(epoch) | 5 | "IAU2000B" | 章动（IAU 2000B 77 项）。 |
| TrueEquator(epoch) | ApparentEcliptic(epoch) | 2 | — | 真黄赤交角 R_x(ε)。 |

不指定途径边时，岁差由 `precession_model`、章动由 `nutation_model` 决定；可指定 `Via::Edge(..., Some("Vondrak2011"))` / `Via::Edge(..., Some("IAU2000A"))` 等强制走某条边。

### 3.4 带条件的边（缓存即快速跳跃边）

岁差/章动/黄赤交角段带**历元缓存**：同一 jd_tt、PrecessionModel、NutationModel 下不重复算矩阵。可视为**带条件的边**——条件满足（缓存命中）则边存在、直接跳跃；不满足则需计算并写缓存后再到达。路径规划仍按静态边；执行时按「有缓存走缓存，无则计算并更新」处理。实现见 `get_transition` 与 `epoch_cache`（同上文件）。

### 3.5 同架修正不引入新架：虚拟状态节点与状态同一性

在管线图中，**一个节点标识即状态的唯一标识**。若「未修正」与「已修正」共用同一标识，路由无法判断是否已修正，会导致重复或漏修正。

- **同标架内纯数值修正**（如黄道 L,B,R 拟合）：**没有架的变换，因此不引入「中间架」**。因节点语义已升级为「状态」，我们引入的是**虚拟状态节点**（如 `MeanEclipticEclipticPatch`），使「已修正」拥有独立状态标识；物理上仍属同一架（坐标轴与原点不变），图中为独立驿站以便寻路与防呆。
- **跨标架复合修正**（如 FK5→ICRS 黑盒）：起点与终点已是不同物理架，无需额外节点，一条 GeneralMapping 边即可。

---

## 4. 边的双重解耦：概念（Kind）与执行（Form）

每条边有两层属性，实现领域语义与执行方式的分离。

**概念分类（TransitionKind，回答「这是什么物理操作」）**：FrameRotation、FrameTranslation、LightTime、Aberration、LightTimeAberrationFolded、GeneralMapping。当前由 `edge_kind(from_id, to_id)` 根据 (from, to) 映射，用于可视化与文档。

**执行形式（TransitionForm，回答「代码如何计算」，默认由 Kind 推导）**：Rotation（6×6 状态转移矩阵）、Translation（6D 平移）、Mapping（FrameMapper / 闭包）、LightTime、Aberration、LightTimeAberrationFolded。实现：每条边的 viz 含 `kind` 与 `form`；`edge_form_for_frames` 供执行分派使用。

---

## 5. 寻路驱动与执行引擎（Path-Finding & Dispatch）

管线拒绝硬编码嵌套，运算逻辑完全由图路径规划驱动。

### 5.1 最短路与受限最短路

- **最短路**：`shortest_path(from, to) -> Vec<PathEdge>`，返回带 `edge_key` 的边序列（支持重边）。`PathEdge` 含 `from`、`to`、`edge_key`。
- **受限最短路**：`shortest_path_via(from, to, via) -> Vec<PathEdge>`，支持：
  - **途径点** `Via::Node(frame)`：路径必须经过该节点（如 `Via::Node(MeanEclipticEclipticPatch(epoch))` 强制走黄道拟合）。
  - **途径边** `Via::Edge(from, to, edge_key)`：路径必须经过该边（如强制岁差 "Vondrak2011" 或章动 "IAU2000A"）。

### 5.2 执行与分派

- **执行**：`transform_to(state, target, jd_tt)` / `transform_to_via(state, target, via, jd_tt)` 内部得到路径后，`transform_along_path(state, &path, jd_tt)` 按边依次执行。
- **按 Form 分派**：Rotation 经 `get_transition(..., edge_key)` 取/算矩阵并 `apply_transition`；Mapping 调用对应 mapper（`with_ecliptic_patch_mapper` / `with_fk5_to_icrs_mapper`）。历元相关段使用历元缓存（见 3.4）。
- **Fail-Fast**：图不连通、缺失 mapper 或路径不存在时 panic，不做隐式精度降级。

---

## 6. 数据加载与管线（与 repo 构成的对应）

管线所用**章动表**（IERS 5.3a/5.3b → IAU2000A 完整或 77 项）、**拟合表**（VSOP87–DE406 赤道/黄道 patch）、**历表**（VSOP87、ELPMPP02）均由 **repo** 统一管理路径与读写（见 `rust/core/src/repo.rs`、doc 14）。

- **Native**：`repo::repo_root()`、`repo::default_loader()`；章动/拟合/历表通过 `try_init_*_from_repo()`、`load_*_from_repo()` 加载，路径使用 `repo::paths::*`。
- **Wasm**：宿主 fetch 后 `repo::set_loader(loader)` 注入；同一套 `*_from_repo()` 在 loader 内读入数据。
- **管线侧**：执行层（岁差/章动/黄赤交角、`with_fk5_to_icrs_mapper`、`with_ecliptic_patch_mapper`）依赖上述初始化结果；测试与示例统一使用 `repo::default_loader()` 与各 `*_from_repo()`，与 §3 的边语义（章动 IAU2000A/IAU2000B、拟合边）一致。

---

## 小结

- **节点**：语义升级为**管线数据状态**；分为物理标架节点与虚拟状态节点；用 **OriginRole** 标注原点；通过 `node_origin(id)` 与 viz 的 `node_origins` 暴露。
- **边**：**TransitionKind**（概念）与 **TransitionForm**（执行形式）双重分类；Form 由 Kind 默认推导；支持重边 `edge_key`、带条件的边（缓存）。
- **途径与路径**：`Via`（途径点 / 途径边）、`PathEdge`（含 `edge_key`）、`shortest_path` / `shortest_path_via`、`transform_to_via`。
- **可视化数据**：`TransformGraphVizData` 含 `node_ids`、`node_origins`、`edges`（每条边含 `kind`、`form`、可选标签）。
- **数据加载**（§6）：章动/拟合/历表由 **repo** 统一加载（Native：`default_loader` + `*_from_repo`；Wasm：`set_loader` + `*_from_repo`），与管线边语义一致。

通过将「同标架数值修正」具象化为虚拟状态节点，在保全天文学物理定义严谨性的同时，满足图论算法与状态机的工程约束。`Via` 与 `shortest_path_via` 已在 Rust 中实现，类型与实现见 `rust/core/src/astronomy/pipeline/transform_graph.rs`，导出见 `pipeline/mod.rs`。数据加载与当前 repo 构成的对应见 §6。
