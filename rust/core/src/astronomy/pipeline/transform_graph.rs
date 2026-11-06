//! 状态转移路由：纯物理旋转，图搜索最短路径（文档 3.1、4.2、7.2、15）。
//! 架变换使用完整 6×6 状态转移 [R 0; R_dot R]，R_dot 为旋转对时间的导数（科里奥利项）。
//! 下方共用段（岁差、章动、真黄赤交角）当 t 确定时变换矩阵仅依赖 t，按历元缓存。
//!
//! **图语义**：视位置等统一由此驱动。① 起止点最短路：`shortest_path(from, to)` 分析，`transform_to(state, target, jd_tt)` 执行。② 起止点+途径点受限最短路：`shortest_path_via(from, to, via)` 分析，`transform_to_via(state, target, via, jd_tt)` 执行。见 doc 15。
//!
//! **时间尺度**：岁差（P03/Vondrak）、章动（IAU）公式规定入参 **TT**；`transform_to(_, _, jd_tt)` 的 `jd_tt` 须为 TT。
//!
//! **变换分类**：标架旋转 / 标架平移 / 光行时 / 光行差 / 光行时+光行差折叠 / 一般映射；如需可再增。日心→地心为单独步骤。
//!
//! **地心→站心→视地平**：TrueEquator → TopocentricEquator → TopocentricAberratedEquator → HorizontalGeometric → HorizontalApparent，
//! 依次为平移、周日光行差、转站心地平直角坐标、大气折射（重边）。这些边需 `ObservationContext`，经 `transform_to_observed` 执行；`transform_to` 走到时 panic。
//!
//! **图是网络**：每条边都有逆边（旋转取转置，其余见各函数），黄道与赤道之间有横向边，所以任意两架之间都有路径。
//! 历表与星表不是架到架的变换（输入不是状态，光行时还要重新求历表），是 [`SourceEdge`] 入口，不参与最短路。

use crate::astronomy::frame::fixed::fk5_icrs;
use crate::astronomy::frame::precession::{
    mean_obliquity, mean_obliquity_for, mean_obliquity_rad_and_derivative_for, precession_derivative_times_vector_for, precession_transform_for, PrecessionModel,
};
use crate::astronomy::frame::nutation::{
    eps_true_dot, nutation_derivative_times_vector, nutation_for_model, NutationModel,
};
use crate::astronomy::apparent::{
    annual_aberration_state, diurnal_aberration_state, from_horizontal_state, geocentric_state, nutation_matrix_transposed_for_model, refract_state,
    to_horizontal_state, topocentric_state, undo_annual_aberration_state, undo_diurnal_aberration_state, unrefract_state, ObservationContext,
};
use crate::astronomy::correction::RefractionModel;
use crate::math::real::{real_const, real, zero, one, Real, RealOps, ToReal};
use crate::quantity::{epoch::Epoch, reference_frame::ReferenceFrame};
use std::cell::RefCell;
use super::state::State6;
use super::transition::StateTransition6;

/// 儒略世纪对应的秒数（Real）。
const SEC_PER_CENTURY: Real = real_const(36525.0 * 86400.0);

/// 由 (dR/dt)·e_j 列向量组构建 dR/dt 矩阵（单位：1/世纪 → 换算为 1/s）。
fn build_r_dot_from_derivative_times_vector<F>(f: F) -> [[Real; 3]; 3]
where
    F: Fn([Real; 3]) -> [Real; 3],
{
    let c0 = f([one(), zero(), zero()]);
    let c1 = f([zero(), one(), zero()]);
    let c2 = f([zero(), zero(), one()]);
    let scale = one() / SEC_PER_CENTURY;
    [
        [c0[0] * scale, c1[0] * scale, c2[0] * scale],
        [c0[1] * scale, c1[1] * scale, c2[1] * scale],
        [c0[2] * scale, c1[2] * scale, c2[2] * scale],
    ]
}

/// 赤道→黄道旋转 [1 0 0; 0 c s; 0 −s c] 对 ε 的导数。角支持 Real。
fn rotation_x_derivative(angle_rad: impl ToReal) -> [[Real; 3]; 3] {
    let a = real(angle_rad);
    let (c, s) = (a.cos(), a.sin());
    [[zero(), zero(), zero()], [zero(), -s, c], [zero(), -c, -s]]
}

fn scale_matrix(s: impl ToReal, m: &[[Real; 3]; 3]) -> [[Real; 3]; 3] {
    crate::math::algebra::mat::Mat::<Real, 3, 3>::from(*m).scale(real(s)).to_array()
}

/// 节点/边变换分类；如需可再增（如尺度、时间等）。
///
/// - **标架旋转**：仅坐标轴旋转（岁差、章动、黄赤交角、Frame bias）。
/// - **标架平移**：原点平移（日心↔地心↔质心等）。
/// - **光行时**：仅推迟时 tr，位置取 r(tr)，历表在 tr 求值。
/// - **光行差**：显式光行差改正（如周年光行差），单独一步。
/// - **光行时+光行差折叠**：Xproper 等总效应，不单独拆开（默认定气/视黄经管线）。
/// - **一般映射**：拟合、混合型等，既非纯旋转也非纯平移。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionKind {
    /// 标架旋转：仅坐标轴旋转，无平移、无拟合（岁差、章动、黄赤交角、Frame bias 等）。
    FrameRotation,
    /// 标架平移：原点平移（日心↔地心↔质心等）。
    FrameTranslation,
    /// 光行时：推迟时 tr，历表在 tr 求值。
    LightTime,
    /// 光行差：显式光行差改正（如周年光行差）。
    Aberration,
    /// 光行时+光行差折叠总效应（Xproper 等，不单独施光行差）。
    LightTimeAberrationFolded,
    /// 一般映射：拟合修正各分量、或混合型（如把 Frame bias 一并拟合掉），既非纯旋转也非纯平移。
    GeneralMapping,
}

impl TransitionKind {
    /// 中文标签，供前端/文档用。
    pub fn label_cn(self) -> &'static str {
        match self {
            TransitionKind::FrameRotation => "标架旋转",
            TransitionKind::FrameTranslation => "标架平移",
            TransitionKind::LightTime => "光行时",
            TransitionKind::Aberration => "光行差",
            TransitionKind::LightTimeAberrationFolded => "光行时+光行差折叠",
            TransitionKind::GeneralMapping => "一般映射",
        }
    }

    /// 由概念分类得到默认的“执行形式”，用于管线分支与文档；详见 doc 管线图设计。
    pub fn default_form(self) -> TransitionForm {
        kind_to_form(self)
    }
}

/// 节点的“原点角色”：该节点状态所在参考架的原点（日心/地心/质心）。
/// 用于图的语义分类与校验，例如边 日心→地心 的 kind 为 FrameTranslation 且 from 为 Heliocentric、to 为 Geocentric。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OriginRole {
    /// 日心
    Heliocentric,
    /// 地心
    Geocentric,
    /// 太阳系质心（SSB）
    Barycentric,
    /// 站心（地面观测者）
    Topocentric,
}

impl OriginRole {
    pub fn label_cn(self) -> &'static str {
        match self {
            OriginRole::Heliocentric => "日心",
            OriginRole::Geocentric => "地心",
            OriginRole::Barycentric => "质心",
            OriginRole::Topocentric => "站心",
        }
    }
}

/// 边的“执行形式”：该步在实现上如何计算（旋转矩阵 / 平移 / 光行时迭代 / 映射等）。
/// 与 TransitionKind（概念分类）区分：Kind 回答“是什么”，Form 回答“怎么算”；通常由 kind 默认推导，见 kind_to_form。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionForm {
    /// 6×6 状态转移中的 3×3 旋转（岁差、章动、黄赤交角、Frame bias 等）
    Rotation,
    /// 原点平移（日心↔地心↔质心），状态空间中的平移
    Translation,
    /// 光行时：迭代求 tr，历表在 tr 求值
    LightTime,
    /// 显式光行差公式（如周年光行差）
    Aberration,
    /// 光行时+光行差折叠为一步，不再单独施光行差
    LightTimeAberrationFolded,
    /// 一般映射：FrameMapper / 拟合修正等
    Mapping,
}

impl TransitionForm {
    pub fn label_cn(self) -> &'static str {
        match self {
            TransitionForm::Rotation => "旋转",
            TransitionForm::Translation => "平移",
            TransitionForm::LightTime => "光行时",
            TransitionForm::Aberration => "光行差",
            TransitionForm::LightTimeAberrationFolded => "光行时+光行差折叠",
            TransitionForm::Mapping => "映射",
        }
    }
}

/// 由边的概念分类推导默认执行形式。
pub fn kind_to_form(kind: TransitionKind) -> TransitionForm {
    match kind {
        TransitionKind::FrameRotation => TransitionForm::Rotation,
        TransitionKind::FrameTranslation => TransitionForm::Translation,
        TransitionKind::LightTime => TransitionForm::LightTime,
        TransitionKind::Aberration => TransitionForm::Aberration,
        TransitionKind::LightTimeAberrationFolded => TransitionForm::LightTimeAberrationFolded,
        TransitionKind::GeneralMapping => TransitionForm::Mapping,
    }
}

/// 按节点 id 返回其原点角色（若有）；用于可视化与校验。
pub fn node_origin(node_id: &str) -> Option<OriginRole> {
    match node_id {
        "VSOP87" | HELIOCENTRIC_MEAN_ECLIPTIC => Some(OriginRole::Heliocentric),
        "MeanEcliptic(J2000)" | "MeanEcliptic(epoch)" | "TrueEcliptic(epoch)" | "FK5" | "MeanEquator(epoch)" | "TrueEquator(epoch)"
        | "ApparentEquator(epoch)" | "ApparentEcliptic(epoch)" | "ICRS" | "IcrsAberrated" => Some(OriginRole::Geocentric),
        "TopocentricEquator(epoch)" | "TopocentricAberratedEquator(epoch)" | "HorizontalGeometric(epoch)" | "HorizontalApparent(epoch)" => {
            Some(OriginRole::Topocentric)
        }
        _ => None,
    }
}

fn is_j2000_ecliptic(f: ReferenceFrame) -> bool {
    f.id_str() == "MeanEcliptic(J2000)"
}

/// 正向边的执行形式。
fn forward_edge_form(from: ReferenceFrame, to: ReferenceFrame) -> Option<TransitionForm> {
    use ReferenceFrame as F;
    match (from, to) {
        (F::MeanEcliptic(_), F::FK5) if is_j2000_ecliptic(from) => Some(TransitionForm::Rotation),
        (F::FK5, F::MeanEquator(_)) => Some(TransitionForm::Rotation),
        (F::MeanEquator(_), F::TrueEquator(_)) => Some(TransitionForm::Rotation),
        (F::MeanEquator(_), F::MeanEcliptic(_)) if !is_j2000_ecliptic(to) => Some(TransitionForm::Rotation),
        (F::MeanEcliptic(_), F::TrueEcliptic(_)) if !is_j2000_ecliptic(from) => Some(TransitionForm::Rotation),
        (F::TrueEquator(_), F::TrueEcliptic(_)) => Some(TransitionForm::Rotation),
        (F::TrueEquator(_), F::ApparentEcliptic(_)) => Some(TransitionForm::Rotation),
        (F::ICRS, F::IcrsAberrated) => Some(TransitionForm::Aberration),
        (F::IcrsAberrated, F::ApparentEquator(_)) => Some(TransitionForm::Rotation),
        (F::TrueEquator(_), F::TopocentricEquator(_)) => Some(TransitionForm::Translation),
        (F::TopocentricEquator(_), F::TopocentricAberratedEquator(_)) => Some(TransitionForm::Aberration),
        (F::TopocentricAberratedEquator(_), F::HorizontalGeometric(_)) => Some(TransitionForm::Rotation),
        (F::HorizontalGeometric(_), F::HorizontalApparent(_)) => Some(TransitionForm::Mapping),
        _ => None,
    }
}

/// 按 (from_frame, to_frame) 返回边的执行形式，供路径驱动执行时分派。仅对图中存在的边返回 Some；逆边与正向同形式。
pub fn edge_form_for_frames(from: ReferenceFrame, to: ReferenceFrame) -> Option<TransitionForm> {
    match (from, to) {
        (ReferenceFrame::FK5, ReferenceFrame::ICRS) => Some(TransitionForm::Mapping),
        (ReferenceFrame::ICRS, ReferenceFrame::FK5) => Some(TransitionForm::Rotation),
        _ => forward_edge_form(from, to).or_else(|| forward_edge_form(to, from)),
    }
}

/// 3×3 矩阵的转置。
fn transposed(m: &[[Real; 3]; 3]) -> [[Real; 3]; 3] {
    core::array::from_fn(|i| core::array::from_fn(|j| m[j][i]))
}

fn mat_mul(a: &[[Real; 3]; 3], b: &[[Real; 3]; 3]) -> [[Real; 3]; 3] {
    core::array::from_fn(|i| core::array::from_fn(|j| a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j]))
}

/// 复合旋转 A∘B 的 (R, Ṙ)，Ṙ = ȦB + AḂ。
fn compose(a: (&[[Real; 3]; 3], &[[Real; 3]; 3]), b: (&[[Real; 3]; 3], &[[Real; 3]; 3])) -> ([[Real; 3]; 3], [[Real; 3]; 3]) {
    let (p, q) = (mat_mul(a.1, b.0), mat_mul(a.0, b.1));
    (mat_mul(a.0, b.0), core::array::from_fn(|i| core::array::from_fn(|j| p[i][j] + q[i][j])))
}

/// 太阳历表日心架输出（J2000 平黄道；VSOP87 拟合到 DE406/DE441 的改正已含在历表求值中），经原点变换→地心后到 MeanEcliptic(J2000)。
const HELIOCENTRIC_MEAN_ECLIPTIC: &'static str = "日心_MeanEcliptic(J2000)";
/// 按 (from_id, to_id) 返回边的变换分类；逆边与正向同类。
fn edge_kind(from_id: &str, to_id: &str) -> TransitionKind {
    let forward = |a: &str, b: &str| match (a, b) {
        ("TrueEquator(epoch)", "TopocentricEquator(epoch)") => Some(TransitionKind::FrameTranslation),
        ("TopocentricEquator(epoch)", "TopocentricAberratedEquator(epoch)") | ("ICRS", "IcrsAberrated") => Some(TransitionKind::Aberration),
        ("HorizontalGeometric(epoch)", "HorizontalApparent(epoch)") => Some(TransitionKind::GeneralMapping),
        _ => None,
    };
    forward(from_id, to_id).or_else(|| forward(to_id, from_id)).unwrap_or(TransitionKind::FrameRotation)
}

/// 正向边的管线步骤标签；仅写动作，不重复标架/坐标系（框内已示）。
fn forward_edge_label(from_id: &str, to_id: &str, edge_key: Option<&str>) -> Option<&'static str> {
    Some(match (from_id, to_id) {
        ("MeanEcliptic(J2000)", "FK5") => "黄赤交角 R_x(ε₀)",
        ("FK5", "ICRS") => "Frame bias B⁻¹（FK5→ICRS）",
        ("FK5", "MeanEquator(epoch)") => match edge_key {
            Some("P03") => "岁差（P03）",
            _ => "岁差（Vondrak2011）",
        },
        ("MeanEquator(epoch)", "TrueEquator(epoch)") => match edge_key {
            Some("IAU2000A") => "章动（IAU2000A）",
            _ => "章动（IAU2000B）",
        },
        ("MeanEquator(epoch)", "MeanEcliptic(epoch)") => "R_x(ε) 平黄赤交角",
        ("MeanEcliptic(epoch)", "TrueEcliptic(epoch)") => "黄道上的章动 Δψ、Δε",
        ("TrueEquator(epoch)", "TrueEcliptic(epoch)") => "R_x(ε) 真黄赤交角",
        ("TrueEquator(epoch)", "ApparentEcliptic(epoch)") => "R_x(ε) 真黄赤交角（光行差已折叠在历表入口）",
        ("ICRS", "IcrsAberrated") => "周年光行差（显式，需地球速度）",
        ("IcrsAberrated", "ApparentEquator(epoch)") => "偏差、岁差、章动",
        ("TrueEquator(epoch)", "TopocentricEquator(epoch)") => "地心→站心（减观测者位置）",
        ("TopocentricEquator(epoch)", "TopocentricAberratedEquator(epoch)") => "周日光行差",
        ("TopocentricAberratedEquator(epoch)", "HorizontalGeometric(epoch)") => "转站心地平直角坐标（地方恒星时、大地纬度）",
        ("HorizontalGeometric(epoch)", "HorizontalApparent(epoch)") => "大气折射（光线追迹，可选拟合式）",
        _ => return None,
    })
}

/// 边的标签；逆边写「逆：」加正向标签。ICRS→FK5 与 FK5→ICRS 各有实现，不算逆边。
fn edge_label(from_id: &str, to_id: &str, edge_key: Option<&str>) -> Option<String> {
    if (from_id, to_id) == ("ICRS", "FK5") {
        return Some("B（Frame bias，GCRS→FK5）".to_string());
    }
    if let Some(l) = forward_edge_label(from_id, to_id, edge_key) {
        return Some(l.to_string());
    }
    Some(match forward_edge_label(to_id, from_id, edge_key) {
        Some(l) => format!("逆：{l}"),
        None => "几何变换".to_string(),
    })
}

fn make_edge_viz(from_id: &str, to_id: &str, cost: u32, edge_key: Option<&str>) -> TransformEdgeViz {
    let kind = edge_kind(from_id, to_id);
    TransformEdgeViz {
        from_id: from_id.to_string(),
        to_id: to_id.to_string(),
        cost,
        kind,
        form: kind_to_form(kind),
        label: edge_label(from_id, to_id, edge_key),
    }
}

/// 历表、星表入口：节点不是参考架，输入不是状态，由 [`super::EphemerisProvider`] 等执行，不参与最短路。
/// 光行时属于这里：要在推迟时刻 tr 重新求历表（[`super::LightTimeCorrector`]），状态的架不变。
#[derive(Clone, Debug)]
pub struct SourceEdge {
    pub from_id: &'static str,
    pub to_id: &'static str,
    pub kind: TransitionKind,
    pub label: &'static str,
}

/// 全部入口；终点架由 `source_edges_end_at_provider_frames` 测试对照各提供者的实际输出。
const SOURCE_EDGES: [SourceEdge; 6] = [
    SourceEdge { from_id: "VSOP87", to_id: HELIOCENTRIC_MEAN_ECLIPTIC, kind: TransitionKind::LightTime, label: "历表求值（含 DE406/DE441 拟合改正）；光行时可选，在 tr 求值" },
    SourceEdge { from_id: HELIOCENTRIC_MEAN_ECLIPTIC, to_id: "MeanEcliptic(J2000)", kind: TransitionKind::FrameTranslation, label: "日心→地心" },
    SourceEdge { from_id: "ELPMPP02", to_id: "ELPMPP02_MEAN_LUNAR", kind: TransitionKind::GeneralMapping, label: "历表求值（所选常数集，缺省拟合 DE431）；光行时可选" },
    SourceEdge { from_id: "ELPMPP02_MEAN_LUNAR", to_id: "ICRS", kind: TransitionKind::FrameRotation, label: "Laskar P,Q 旋转 → ELP 的 J2000 平黄道 → Table 7（ICRS 行）" },
    SourceEdge { from_id: "DE406/DE441", to_id: "ICRS", kind: TransitionKind::LightTime, label: "历表求值；光行时可选" },
    SourceEdge { from_id: "Hipparcos", to_id: "ICRS", kind: TransitionKind::GeneralMapping, label: "星表方向（自行、周年视差）" },
];

/// 单条有向边：从 from 到 to，代价 cost；可选 edge_key 区分同一对节点间的重边（如岁差 P03 / Vondrak2011）。
#[derive(Clone, Debug)]
pub struct TransformEdge {
    pub from_frame: ReferenceFrame,
    pub to_frame: ReferenceFrame,
    pub cost: u32,
    /// 重边标识；None 表示该 (from, to) 仅此一条边或使用图默认（如岁差用 precession_model）。
    pub edge_key: Option<String>,
}

/// 路径中的一条边，用于返回与执行；带 edge_key 以支持重边。
#[derive(Clone, Debug)]
pub struct PathEdge {
    pub from: ReferenceFrame,
    pub to: ReferenceFrame,
    pub edge_key: Option<String>,
}

/// 途径约束：途径点（必须经过的节点）或途径边（必须经过的边，可选 key 指定重边）。
#[derive(Clone, Debug)]
pub enum Via {
    Node(ReferenceFrame),
    Edge(ReferenceFrame, ReferenceFrame, Option<String>),
}

/// 单条边的可视化数据：架 id、代价、分类与可选步骤标签，便于序列化到 WASM/JS。
#[derive(Clone, Debug)]
pub struct TransformEdgeViz {
    pub from_id: String,
    pub to_id: String,
    pub cost: u32,
    /// 变换分类（概念）：标架旋转 / 标架平移 / 光行时 / 光行差 / 光行时+光行差折叠 / 一般映射。
    pub kind: TransitionKind,
    /// 执行形式（如何算）：由 kind 默认推导，供管线分支与文档用。
    pub form: TransitionForm,
    /// 管线步骤说明，如 "P03岁差"、"章动"、"Frame bias"。
    pub label: Option<String>,
}

/// 变换图的可视化数据：节点 id 列表、节点原点角色（若有）、边列表，供前端绘图与管线语义用。
#[derive(Clone, Debug)]
pub struct TransformGraphVizData {
    pub node_ids: Vec<String>,
    /// 部分节点标注原点角色（日心/地心/质心），与 node_ids 对应；未标注的节点此项为空。
    pub node_origins: Vec<(String, OriginRole)>,
    pub edges: Vec<TransformEdgeViz>,
}

/// 岁差/章动/黄赤交角按 (jd_tt, PrecessionModel, NutationModel) 缓存。
/// 语义上可视为**带条件的快速跳跃边**：条件满足（缓存命中）则沿该边直接到下一节点；不满足则边不存在，需计算并写缓存后再到达。
#[derive(Clone, Debug)]
struct EpochTransitionsCache {
    jd_tt_key: f64,
    precession: StateTransition6,
    nutation: StateTransition6,
    obliquity: StateTransition6,
}

/// 历元缓存容差（日）：|jd_tt - cache_key| < 此值则复用岁差/章动/黄赤交角矩阵。
const EPOCH_CACHE_TOLERANCE_DAY: f64 = 1e-9;

/// 边 key 对应岁差模型；用于重边（如 "P03" / "Vondrak2011"）。key 不匹配时用 default。
fn edge_key_to_precession(key: Option<&str>, default: PrecessionModel) -> PrecessionModel {
    match key {
        Some("Vondrak2011") => PrecessionModel::Vondrak2011,
        Some("P03") | _ => default,
    }
}

/// 岁差边是否应参与最短路：key 为 None 或匹配当前 precession_model 时包含。
fn precession_edge_matches(e: &TransformEdge, precession_model: PrecessionModel) -> bool {
    match e.edge_key.as_deref() {
        None => true,
        Some("P03") => precession_model == PrecessionModel::P03,
        Some("Vondrak2011") => precession_model == PrecessionModel::Vondrak2011,
        Some(_) => true,
    }
}

/// 边 key 对应章动模型；用于重边（"IAU2000A" / "IAU2000B"）。key 不匹配时用 default。
fn edge_key_to_nutation(key: Option<&str>, default: NutationModel) -> NutationModel {
    match key {
        Some("IAU2000A") => NutationModel::IAU2000A,
        Some("IAU2000B") | _ => default,
    }
}

fn is_precession_edge(e: &TransformEdge) -> bool {
    matches!(
        (e.from_frame, e.to_frame),
        (ReferenceFrame::FK5, ReferenceFrame::MeanEquator(_)) | (ReferenceFrame::MeanEquator(_), ReferenceFrame::FK5)
    )
}

fn is_nutation_edge(e: &TransformEdge) -> bool {
    matches!(
        (e.from_frame, e.to_frame),
        (ReferenceFrame::MeanEquator(_), ReferenceFrame::TrueEquator(_)) | (ReferenceFrame::TrueEquator(_), ReferenceFrame::MeanEquator(_))
    )
}

fn is_refraction_edge(e: &TransformEdge) -> bool {
    matches!(e.to_frame, ReferenceFrame::HorizontalApparent(_)) || matches!(e.from_frame, ReferenceFrame::HorizontalApparent(_))
}

/// 折射边是否应参与最短路：key 匹配当前 refraction_model 时包含。
fn refraction_edge_matches(e: &TransformEdge, refraction_model: RefractionModel) -> bool {
    e.edge_key.as_deref().is_none_or(|k| k == refraction_model.edge_key())
}

/// 章动边是否应参与最短路：key 为 None 或匹配当前 nutation_model 时包含。
fn nutation_edge_matches(e: &TransformEdge, nutation_model: NutationModel) -> bool {
    match e.edge_key.as_deref() {
        None => true,
        Some("IAU2000A") => nutation_model == NutationModel::IAU2000A,
        Some("IAU2000B") => nutation_model == NutationModel::IAU2000B,
        Some(_) => true,
    }
}

/// 当日平赤道 → 当日平黄道的 (R, Ṙ)，Ṙ 单位 1/s。
fn mean_equator_to_ecliptic(jd_tt: Real, model: PrecessionModel) -> ([[Real; 3]; 3], [[Real; 3]; 3]) {
    let t_cent = (jd_tt - crate::astronomy::constant::J2000) / real(36525.0);
    let (eps, eps_dot) = mean_obliquity_rad_and_derivative_for(t_cent, model);
    let (c, s) = (eps.rad().cos(), eps.rad().sin());
    let r = [[one(), zero(), zero()], [zero(), c, s], [zero(), -s, c]];
    let r_dot = scale_matrix(eps_dot.rad_per_julian_century() / SEC_PER_CENTURY, &rotation_x_derivative(eps.rad()));
    (r, r_dot)
}

fn fill_epoch_transitions_cache(
    jd_tt: Real,
    jd_f64: f64,
    precession_model: PrecessionModel,
    nutation_model: NutationModel,
) -> EpochTransitionsCache {
    let t_cent = (jd_tt - crate::astronomy::constant::J2000) / real(36525.0);
    let to_me = ReferenceFrame::MeanEquator(Epoch::new(jd_tt));
    let to_te = ReferenceFrame::TrueEquator(Epoch::new(jd_tt));
    let to_ae = ReferenceFrame::ApparentEcliptic(Epoch::new(jd_tt));
    let pt = precession_transform_for(t_cent, precession_model);
    let precession = StateTransition6 {
        from_frame: ReferenceFrame::FK5,
        to_frame: to_me,
        r: pt.matrix,
        r_dot: build_r_dot_from_derivative_times_vector(|r| {
            precession_derivative_times_vector_for(r, t_cent, precession_model)
        }),
    };
    let n_t = nutation_matrix_transposed_for_model(t_cent, nutation_model, precession_model);
    let nutation = StateTransition6 {
        from_frame: to_me,
        to_frame: to_te,
        r: n_t,
        r_dot: build_r_dot_from_derivative_times_vector(|r| {
            nutation_derivative_times_vector(r, t_cent, precession_model)
        }),
    };
    let (_, deps) = nutation_for_model(t_cent, nutation_model);
    let eps_mean = mean_obliquity_for(t_cent, precession_model).rad();
    let eps_true = eps_mean + deps.rad();
    let (ct, st) = (eps_true.cos(), eps_true.sin());
    let r = [[one(), zero(), zero()], [zero(), ct, st], [zero(), -st, ct]];
    let eps_true_dot_rad_per_century = eps_true_dot(t_cent, precession_model);
    let r_dot_century = scale_matrix(eps_true_dot_rad_per_century, &rotation_x_derivative(eps_true));
    let obliquity = StateTransition6 {
        from_frame: to_te,
        to_frame: to_ae,
        r,
        r_dot: scale_matrix(one() / SEC_PER_CENTURY, &r_dot_century),
    };
    EpochTransitionsCache {
        jd_tt_key: jd_f64,
        precession,
        nutation,
        obliquity,
    }
}

/// FK5→ICRS 等“映射边”的执行器：输入 (state, jd_tt)，输出变换后的 state。
pub type EdgeMapperFn = Box<dyn Fn(State6, Real) -> State6 + Send + 'static>;

/// 变换图：注册边与矩阵提供者，按最短路驱动执行并施加 6D 变换。
pub struct TransformGraph {
    edges: Vec<TransformEdge>,
    /// 岁差模型：FK5 → MeanEquator 时使用。
    pub precession_model: PrecessionModel,
    /// 章动模型：MeanEquator → TrueEquator 时使用。
    pub nutation_model: NutationModel,
    /// 折射模型：HorizontalGeometric → HorizontalApparent 时使用。
    pub refraction_model: RefractionModel,
    /// 带条件的快速跳跃边状态：满足 (precession, nutation, jd) 命中则执行时走缓存边，否则计算并更新。
    epoch_cache: RefCell<Option<(PrecessionModel, NutationModel, EpochTransitionsCache)>>,
    /// FK5 → ICRS 边的执行器（Frame bias）；未设则路径含该边时无法执行。
    optional_fk5_to_icrs: Option<EdgeMapperFn>,
}

impl TransformGraph {
    pub fn new() -> Self {
        Self {
            edges: vec![],
            precession_model: PrecessionModel::Vondrak2011,
            nutation_model: NutationModel::IAU2000B,
            refraction_model: RefractionModel::RayTrace,
            epoch_cache: RefCell::new(None),
            optional_fk5_to_icrs: None,
        }
    }

    /// 指定岁差模型（缺省 Vondrak2011；P03 可选）。
    pub fn with_precession_model(mut self, model: PrecessionModel) -> Self {
        self.precession_model = model;
        self
    }

    /// 指定章动模型（IAU2000A 完整表 / IAU2000B 77 项）。
    pub fn with_nutation_model(mut self, model: NutationModel) -> Self {
        self.nutation_model = model;
        self
    }

    /// 指定折射模型（缺省 RayTrace）。Smart、Meeus 只适用于高度 ≥ 15°，低于此值执行时 panic。
    pub fn with_refraction_model(mut self, model: RefractionModel) -> Self {
        self.refraction_model = model;
        self
    }

    /// 设置 FK5→ICRS 边的执行器；路径含该边时调用，未设则 transform_to 在该边 panic。
    pub fn with_fk5_to_icrs_mapper<F>(mut self, f: F) -> Self
    where
        F: Fn(State6, Real) -> State6 + Send + 'static,
    {
        self.optional_fk5_to_icrs = Some(Box::new(f));
        self
    }

    pub fn add_edge(&mut self, from: ReferenceFrame, to: ReferenceFrame, cost: u32) {
        self.edges.push(TransformEdge {
            from_frame: from,
            to_frame: to,
            cost,
            edge_key: None,
        });
    }

    /// 添加带 key 的边（重边）；同一 (from, to) 可有多个 key，如岁差 "P03" / "Vondrak2011"。
    pub fn add_edge_with_key(
        &mut self,
        from: ReferenceFrame,
        to: ReferenceFrame,
        cost: u32,
        key: impl Into<String>,
    ) {
        self.edges.push(TransformEdge {
            from_frame: from,
            to_frame: to,
            cost,
            edge_key: Some(key.into()),
        });
    }

    /// 添加一对互逆的边。
    fn add_pair(&mut self, a: ReferenceFrame, b: ReferenceFrame, cost: u32, key: Option<&str>) {
        for (from, to) in [(a, b), (b, a)] {
            self.edges.push(TransformEdge { from_frame: from, to_frame: to, cost, edge_key: key.map(str::to_string) });
        }
    }

    /// 构建完整图（静态拓扑）：展示与运算共用此图。
    /// 未设 `with_fk5_to_icrs_mapper` 时，最短路会避开 FK5→ICRS 映射边。
    ///
    /// 显式光行差是单独一支：`ICRS ↔ IcrsAberrated ↔ ApparentEquator`，只能经光行差边进出。ApparentEcliptic 属于折叠路径
    /// （光行差已含在历表入口的 tr 里），两支不能相连，否则最短路会绕过光行差边、悄悄不施加。见 `aberration_edge_cannot_be_bypassed` 测试。
    pub fn default_graph() -> Self {
        use ReferenceFrame as F;
        let date = Epoch::new(real(0.0));
        let mut g = Self::new();
        g.add_pair(F::MeanEcliptic(Epoch::j2000()), F::FK5, 8, None);
        for key in ["P03", "Vondrak2011"] {
            g.add_pair(F::FK5, F::MeanEquator(date), 10, Some(key));
        }
        for key in ["IAU2000A", "IAU2000B"] {
            g.add_pair(F::MeanEquator(date), F::TrueEquator(date), 5, Some(key));
        }
        g.add_pair(F::MeanEquator(date), F::MeanEcliptic(date), 2, None);
        g.add_pair(F::MeanEcliptic(date), F::TrueEcliptic(date), 5, None);
        g.add_pair(F::TrueEquator(date), F::TrueEcliptic(date), 2, None);
        g.add_pair(F::TrueEquator(date), F::ApparentEcliptic(date), 2, None);
        g.add_pair(F::ICRS, F::IcrsAberrated, 1, None);
        g.add_pair(F::IcrsAberrated, F::ApparentEquator(date), 16, None);
        g.add_pair(F::TrueEquator(date), F::TopocentricEquator(date), 2, None);
        g.add_pair(F::TopocentricEquator(date), F::TopocentricAberratedEquator(date), 1, None);
        g.add_pair(F::TopocentricAberratedEquator(date), F::HorizontalGeometric(date), 1, None);
        for model in RefractionModel::ALL {
            g.add_pair(F::HorizontalGeometric(date), F::HorizontalApparent(date), 2, Some(model.edge_key()));
        }
        g.add_edge(F::ICRS, F::FK5, 1);
        g.add_edge(F::FK5, F::ICRS, 1);
        g
    }

    /// 历表与星表入口。
    pub fn source_edges(&self) -> &'static [SourceEdge] {
        &SOURCE_EDGES
    }

    /// 返回图的可视化数据：入口在前，其后是图中全部有向边，互逆的两条各占一项。
    pub fn visualization_data(&self) -> TransformGraphVizData {
        let mut node_ids: Vec<String> = Vec::new();
        let mut add_node = |id: &str| {
            if !node_ids.iter().any(|n| n == id) {
                node_ids.push(id.to_string());
            }
        };
        let mut edges: Vec<TransformEdgeViz> = Vec::new();
        for e in &SOURCE_EDGES {
            add_node(e.from_id);
            add_node(e.to_id);
            edges.push(TransformEdgeViz {
                from_id: e.from_id.to_string(),
                to_id: e.to_id.to_string(),
                cost: 0,
                kind: e.kind,
                form: kind_to_form(e.kind),
                label: Some(e.label.to_string()),
            });
        }
        for e in &self.edges {
            // 折射重边只画缺省模型一对，标签列明可选：多条平行线标签会完全重叠
            if is_refraction_edge(e) && e.edge_key.as_deref() != Some(RefractionModel::RayTrace.edge_key()) {
                continue;
            }
            add_node(e.from_frame.id_str());
            add_node(e.to_frame.id_str());
            edges.push(make_edge_viz(e.from_frame.id_str(), e.to_frame.id_str(), e.cost, e.edge_key.as_deref()));
        }
        let node_origins: Vec<(String, OriginRole)> = node_ids
            .iter()
            .filter_map(|id| node_origin(id).map(|r| (id.clone(), r)))
            .collect();
        TransformGraphVizData {
            node_ids,
            node_origins,
            edges,
        }
    }

    /// 从 state 的 frame 到 target 的路径（简化：仅按“同型”动态架匹配，不泛化任意 Epoch）。
    pub fn find_path(&self, from: ReferenceFrame, to: ReferenceFrame) -> Vec<(ReferenceFrame, ReferenceFrame)> {
        let mut path = vec![];
        let mut current = from;
        while current != to {
            let next = self.edges.iter().find(|e| self.frames_match(e.from_frame, current) && self.frames_match(e.to_frame, to));
            match next {
                Some(e) => {
                    path.push((e.from_frame, e.to_frame));
                    current = e.to_frame;
                }
                None => break,
            }
        }
        path
    }

    fn frames_match(&self, a: ReferenceFrame, b: ReferenceFrame) -> bool {
        a.id_str() == b.id_str()
    }

    /// 根据图与起止点分析最短路，返回边序列（含 edge_key，支持重边）。
    #[inline]
    pub fn shortest_path(&self, from: ReferenceFrame, to: ReferenceFrame) -> Vec<PathEdge> {
        self.find_path_by_cost(from, to)
    }

    /// 根据图、起止点与途径点/途径边，计算受限最短路，返回整条边序列（含 edge_key）。
    pub fn shortest_path_via(
        &self,
        from: ReferenceFrame,
        to: ReferenceFrame,
        via: &[Via],
    ) -> Vec<PathEdge> {
        let mut path = Vec::new();
        let mut current = from;
        for v in via {
            match v {
                Via::Node(n) => {
                    path.extend(self.find_path_by_cost(current, *n));
                    current = *n;
                }
                Via::Edge(a, b, key) => {
                    path.extend(self.find_path_by_cost(current, *a));
                    path.push(PathEdge {
                        from: *a,
                        to: *b,
                        edge_key: key.clone(),
                    });
                    current = *b;
                }
            }
        }
        path.extend(self.find_path_by_cost(current, to));
        path
    }

    /// 按边代价求从 from 到 to 的最短路径，返回带 edge_key 的边序列（支持重边）。
    pub fn find_path_by_cost(&self, from: ReferenceFrame, to: ReferenceFrame) -> Vec<PathEdge> {
        use std::collections::{BinaryHeap, HashMap};
        let from_id = from.id_str();
        let to_id = to.id_str();
        if from_id == to_id {
            return vec![];
        }
        type NodeId = &'static str;
        type Cost = u32;
        #[derive(Ord, PartialOrd, Eq, PartialEq)]
        struct State {
            cost: Cost,
            node: NodeId,
        }
        let mut adj: HashMap<NodeId, Vec<(NodeId, Cost, PathEdge)>> = HashMap::new();
        for e in &self.edges {
            if is_precession_edge(e) && !precession_edge_matches(e, self.precession_model) {
                continue;
            }
            if is_nutation_edge(e) && !nutation_edge_matches(e, self.nutation_model) {
                continue;
            }
            if matches!((e.from_frame, e.to_frame), (ReferenceFrame::FK5, ReferenceFrame::ICRS))
                && self.optional_fk5_to_icrs.is_none()
            {
                continue;
            }
            if is_refraction_edge(e) && !refraction_edge_matches(e, self.refraction_model) {
                continue;
            }
            let a = e.from_frame.id_str();
            let b = e.to_frame.id_str();
            let pe = PathEdge {
                from: e.from_frame,
                to: e.to_frame,
                edge_key: e.edge_key.clone(),
            };
            adj.entry(a).or_default().push((b, e.cost, pe));
        }
        let mut dist: HashMap<NodeId, Cost> = HashMap::new();
        let mut prev: HashMap<NodeId, (NodeId, PathEdge)> = HashMap::new();
        let mut heap: BinaryHeap<std::cmp::Reverse<State>> = BinaryHeap::new();
        dist.insert(from_id, 0);
        heap.push(std::cmp::Reverse(State { cost: 0, node: from_id }));
        while let Some(std::cmp::Reverse(State { cost: c, node: u })) = heap.pop() {
            if u == to_id {
                let mut path = vec![];
                let mut cur = to_id;
                while let Some((p, pe)) = prev.get(cur) {
                    path.push(pe.clone());
                    if *p == from_id {
                        break;
                    }
                    cur = p;
                }
                path.reverse();
                return path;
            }
            if *dist.get(u).unwrap_or(&u32::MAX) < c {
                continue;
            }
            for (v, edge_cost, pe) in adj.get(u).into_iter().flatten() {
                let (v, edge_cost, pe) = (*v, *edge_cost, pe.clone());
                let new_cost = c + edge_cost;
                if new_cost < *dist.get(v).unwrap_or(&u32::MAX) {
                    dist.insert(v, new_cost);
                    prev.insert(v, (u, pe));
                    heap.push(std::cmp::Reverse(State { cost: new_cost, node: v }));
                }
            }
        }
        vec![]
    }

    /// 获取从 from 到 to 在给定 jd_tt 下的 6×6 转移；edge_key 用于重边（如岁差 "P03"/"Vondrak2011"）。
    /// 岁差/章动/黄赤交角段：缓存命中视为“快速跳跃边”存在、直接返回；未命中则边不存在、计算并写缓存后返回。
    pub fn get_transition(
        &self,
        from: ReferenceFrame,
        to: ReferenceFrame,
        jd_tt: Real,
        edge_key: Option<&str>,
    ) -> Option<StateTransition6> {
        let jd_f64 = jd_tt.as_f64();
        let precession_model = edge_key_to_precession(edge_key, self.precession_model);
        let nutation_model = edge_key_to_nutation(edge_key, self.nutation_model);
        match (from, to) {
            (ReferenceFrame::MeanEcliptic(_), ReferenceFrame::FK5) if is_j2000_ecliptic(from) => {
                let eps0 = mean_obliquity(0.0).rad();
                let (ct, st) = (eps0.cos(), eps0.sin());
                let r = [[one(), zero(), zero()], [zero(), ct, -st], [zero(), st, ct]];
                let r_dot = [[zero(), zero(), zero()], [zero(), zero(), zero()], [zero(), zero(), zero()]];
                Some(StateTransition6 {
                    from_frame: from,
                    to_frame: to,
                    r,
                    r_dot,
                })
            }
            (ReferenceFrame::FK5, _) if matches!(to, ReferenceFrame::MeanEquator(_)) => {
                let mut cache = self.epoch_cache.borrow_mut();
                let hit = cache.as_ref().map(|(pm, nm, c)| {
                    *pm == precession_model && *nm == self.nutation_model
                        && (c.jd_tt_key - jd_f64).abs() < EPOCH_CACHE_TOLERANCE_DAY
                }).unwrap_or(false);
                if hit {
                    return Some(cache.as_ref().unwrap().2.precession.clone());
                }
                let filled = fill_epoch_transitions_cache(jd_tt, jd_f64, precession_model, self.nutation_model);
                let out = filled.precession.clone();
                *cache = Some((precession_model, self.nutation_model, filled));
                Some(out)
            }
            (_, ReferenceFrame::TrueEquator(_)) if matches!(from, ReferenceFrame::MeanEquator(_)) => {
                let mut cache = self.epoch_cache.borrow_mut();
                let hit = cache.as_ref().map(|(pm, nm, c)| {
                    *pm == self.precession_model && *nm == nutation_model
                        && (c.jd_tt_key - jd_f64).abs() < EPOCH_CACHE_TOLERANCE_DAY
                }).unwrap_or(false);
                if hit {
                    return Some(cache.as_ref().unwrap().2.nutation.clone());
                }
                let filled = fill_epoch_transitions_cache(jd_tt, jd_f64, self.precession_model, nutation_model);
                let out = filled.nutation.clone();
                *cache = Some((self.precession_model, nutation_model, filled));
                Some(out)
            }
            (_, ReferenceFrame::ApparentEcliptic(_)) if matches!(from, ReferenceFrame::TrueEquator(_)) => {
                let mut cache = self.epoch_cache.borrow_mut();
                let hit = cache.as_ref().map(|(pm, nm, c)| {
                    *pm == self.precession_model && *nm == self.nutation_model
                        && (c.jd_tt_key - jd_f64).abs() < EPOCH_CACHE_TOLERANCE_DAY
                }).unwrap_or(false);
                if hit {
                    return Some(cache.as_ref().unwrap().2.obliquity.clone());
                }
                let filled = fill_epoch_transitions_cache(jd_tt, jd_f64, self.precession_model, self.nutation_model);
                let out = filled.obliquity.clone();
                *cache = Some((self.precession_model, self.nutation_model, filled));
                Some(out)
            }
            (ReferenceFrame::ICRS, ReferenceFrame::FK5) => {
                let r = fk5_icrs::rotation_matrix();
                let r_t = [
                    [r[0][0], r[1][0], r[2][0]],
                    [r[0][1], r[1][1], r[2][1]],
                    [r[0][2], r[1][2], r[2][2]],
                ];
                let r_dot = [[zero(), zero(), zero()], [zero(), zero(), zero()], [zero(), zero(), zero()]];
                Some(StateTransition6 {
                    from_frame: from,
                    to_frame: to,
                    r: r_t,
                    r_dot,
                })
            }
            (ReferenceFrame::MeanEquator(_), ReferenceFrame::MeanEcliptic(_)) if !is_j2000_ecliptic(to) => {
                let (r, r_dot) = mean_equator_to_ecliptic(jd_tt, self.precession_model);
                Some(StateTransition6 { from_frame: from, to_frame: ReferenceFrame::MeanEcliptic(Epoch::new(jd_tt)), r, r_dot })
            }
            (ReferenceFrame::MeanEcliptic(_), ReferenceFrame::TrueEcliptic(_)) if !is_j2000_ecliptic(from) => {
                // 平黄道 → 平赤道 → 真赤道 → 真黄道
                let date = Epoch::new(jd_tt);
                let (me, me_dot) = mean_equator_to_ecliptic(jd_tt, self.precession_model);
                let n = self.get_transition(ReferenceFrame::MeanEquator(date), ReferenceFrame::TrueEquator(date), jd_tt, None)?;
                let e = self.get_transition(ReferenceFrame::TrueEquator(date), ReferenceFrame::ApparentEcliptic(date), jd_tt, None)?;
                let inner = compose((&n.r, &n.r_dot), (&transposed(&me), &transposed(&me_dot)));
                let (r, r_dot) = compose((&e.r, &e.r_dot), (&inner.0, &inner.1));
                Some(StateTransition6 { from_frame: from, to_frame: ReferenceFrame::TrueEcliptic(date), r, r_dot })
            }
            (ReferenceFrame::TrueEquator(_), ReferenceFrame::TrueEcliptic(_)) => {
                let date = Epoch::new(jd_tt);
                let e = self.get_transition(ReferenceFrame::TrueEquator(date), ReferenceFrame::ApparentEcliptic(date), jd_tt, None)?;
                Some(StateTransition6 { from_frame: from, to_frame: ReferenceFrame::TrueEcliptic(date), r: e.r, r_dot: e.r_dot })
            }
            (ReferenceFrame::IcrsAberrated, ReferenceFrame::ApparentEquator(_)) => {
                let date = Epoch::new(jd_tt);
                let b = self.get_transition(ReferenceFrame::ICRS, ReferenceFrame::FK5, jd_tt, None)?;
                let p = self.get_transition(ReferenceFrame::FK5, ReferenceFrame::MeanEquator(date), jd_tt, None)?;
                let n = self.get_transition(ReferenceFrame::MeanEquator(date), ReferenceFrame::TrueEquator(date), jd_tt, None)?;
                let inner = compose((&p.r, &p.r_dot), (&b.r, &b.r_dot));
                let (r, r_dot) = compose((&n.r, &n.r_dot), (&inner.0, &inner.1));
                Some(StateTransition6 { from_frame: from, to_frame: ReferenceFrame::ApparentEquator(date), r, r_dot })
            }
            // 逆边：[R 0; Ṙ R]⁻¹ = [Rᵀ 0; Ṙᵀ Rᵀ]（旋转矩阵正交）
            _ if forward_edge_form(to, from) == Some(TransitionForm::Rotation) => {
                let f = self.get_transition(to, from, jd_tt, edge_key)?;
                Some(StateTransition6 { from_frame: f.to_frame, to_frame: f.from_frame, r: transposed(&f.r), r_dot: transposed(&f.r_dot) })
            }
            _ => None,
        }
    }

    /// 将 state 变换到 target 架：根据图与起止点自动分析最短路并执行。无路径或缺 mapper 时 panic。
    pub fn transform_to(&self, state: State6, target: ReferenceFrame, jd_tt: Real) -> State6 {
        let path = self.shortest_path(state.frame(), target);
        if path.is_empty() {
            if self.frames_match(state.frame(), target) {
                return state;
            }
            panic!(
                "transform_to: no path from {} to {}",
                state.frame().id_str(),
                target.id_str()
            );
        }
        self.transform_along_path(state, &path, jd_tt)
    }

    /// 将 state 沿给定边序列（含 edge_key）依次变换。
    pub fn transform_along_path(&self, state: State6, path: &[PathEdge], jd_tt: Real) -> State6 {
        let mut s = state;
        for pe in path {
            s = self.apply_edge(s, pe, jd_tt);
        }
        s
    }

    /// 根据图、起止点与途径点/途径边计算受限最短路并执行。via 可为 Via::Node（途径点）或 Via::Edge（途径边，可选 key 指定重边）。
    pub fn transform_to_via(&self, state: State6, target: ReferenceFrame, via: &[Via], jd_tt: Real) -> State6 {
        let path = self.shortest_path_via(state.frame(), target, via);
        if path.is_empty() {
            if self.frames_match(state.frame(), target) && via.is_empty() {
                return state;
            }
            panic!(
                "transform_to_via: no path from {} via {} constraints to {}",
                state.frame().id_str(),
                via.len(),
                target.id_str()
            );
        }
        self.transform_along_path(state, &path, jd_tt)
    }

    /// 将 state 沿图变换到需要上下文的 target（站心、几何地平、视地平等，及其逆）。无路径或 state 已在 target 时 panic。
    /// 走到周年光行差边时要求 `ctx.earth_velocity` 已设。
    pub fn transform_to_observed(&self, state: State6, target: ReferenceFrame, ctx: &ObservationContext, jd_tt: Real) -> State6 {
        self.run_path(state, target, Some(ctx), ctx.earth_velocity, jd_tt)
    }

    /// 将 state 沿图变换到 target，周年光行差用给定地球速度（ICRS，m/s）；不涉及观测者。
    pub fn transform_with_earth_velocity(&self, state: State6, target: ReferenceFrame, earth_velocity: [Real; 3], jd_tt: Real) -> State6 {
        self.run_path(state, target, None, Some(earth_velocity), jd_tt)
    }

    fn run_path(&self, state: State6, target: ReferenceFrame, ctx: Option<&ObservationContext>, earth_velocity: Option<[Real; 3]>, jd_tt: Real) -> State6 {
        let path = self.shortest_path(state.frame(), target);
        if path.is_empty() {
            panic!("transform: no path from {} to {}", state.frame().id_str(), target.id_str());
        }
        let mut s = state;
        for pe in &path {
            s = if let Some(out) = ctx.and_then(|c| self.apply_observation_edge(&s, pe, c)) {
                out
            } else {
                match (pe.from, pe.to, earth_velocity) {
                    (ReferenceFrame::ICRS, ReferenceFrame::IcrsAberrated, Some(v)) => annual_aberration_state(&s, v),
                    (ReferenceFrame::IcrsAberrated, ReferenceFrame::ICRS, Some(v)) => undo_annual_aberration_state(&s, v),
                    _ => self.apply_edge(s, pe, jd_tt),
                }
            };
        }
        s
    }

    /// 观测侧的边；pe 不属于这几条时返回 None。折射模型取 edge_key，缺省用 refraction_model。
    fn apply_observation_edge(&self, state: &State6, pe: &PathEdge, ctx: &ObservationContext) -> Option<State6> {
        use ReferenceFrame as F;
        let model = || pe.edge_key.as_deref().and_then(RefractionModel::from_edge_key).unwrap_or(self.refraction_model);
        Some(match (pe.from, pe.to) {
            (F::TrueEquator(_), F::TopocentricEquator(_)) => topocentric_state(state, &ctx.observer, ctx.gast),
            (F::TopocentricEquator(_), F::TrueEquator(_)) => geocentric_state(state, &ctx.observer, ctx.gast),
            (F::TopocentricEquator(_), F::TopocentricAberratedEquator(_)) => diurnal_aberration_state(state, ctx),
            (F::TopocentricAberratedEquator(_), F::TopocentricEquator(_)) => undo_diurnal_aberration_state(state, ctx),
            (F::TopocentricAberratedEquator(_), F::HorizontalGeometric(_)) => to_horizontal_state(state, ctx),
            (F::HorizontalGeometric(_), F::TopocentricAberratedEquator(_)) => from_horizontal_state(state, ctx),
            (F::HorizontalGeometric(_), F::HorizontalApparent(_)) => refract_state(state, model(), &ctx.atmosphere),
            (F::HorizontalApparent(_), F::HorizontalGeometric(_)) => unrefract_state(state, model(), &ctx.atmosphere),
            _ => return None,
        })
    }

    /// 对 state 施加单条边（PathEdge，含 edge_key），返回新 state。
    fn apply_edge(&self, state: State6, pe: &PathEdge, jd_tt: Real) -> State6 {
        let (from_f, to_f) = (pe.from, pe.to);
        let needs_observer = |f: ReferenceFrame| {
            matches!(
                f,
                ReferenceFrame::TopocentricEquator(_)
                    | ReferenceFrame::TopocentricAberratedEquator(_)
                    | ReferenceFrame::HorizontalGeometric(_)
                    | ReferenceFrame::HorizontalApparent(_)
            )
        };
        if needs_observer(from_f) || needs_observer(to_f) {
            panic!("apply_edge: {} -> {} 需要观测者，请用 transform_to_observed", from_f.id_str(), to_f.id_str());
        }
        if matches!((from_f, to_f), (ReferenceFrame::ICRS, ReferenceFrame::IcrsAberrated) | (ReferenceFrame::IcrsAberrated, ReferenceFrame::ICRS)) {
            panic!("apply_edge: {} -> {} 需要地球速度，请用 transform_with_earth_velocity", from_f.id_str(), to_f.id_str());
        }
        let form = edge_form_for_frames(from_f, to_f)
            .unwrap_or_else(|| panic!("apply_edge: unknown form {} -> {}", from_f.id_str(), to_f.id_str()));
        match form {
            TransitionForm::Rotation => {
                let tr = self
                    .get_transition(from_f, to_f, jd_tt, pe.edge_key.as_deref())
                    .unwrap_or_else(|| panic!("apply_edge: no transition {} -> {}", from_f.id_str(), to_f.id_str()));
                state.apply_transition(&tr.r, &tr.r_dot, tr.to_frame)
            }
            TransitionForm::Mapping => {
                if matches!((from_f, to_f), (ReferenceFrame::FK5, ReferenceFrame::ICRS)) {
                    let f = self
                        .optional_fk5_to_icrs
                        .as_ref()
                        .expect("apply_edge: FK5→ICRS mapper required");
                    f(state, jd_tt)
                } else {
                    panic!("apply_edge: unknown mapping {} -> {}", from_f.id_str(), to_f.id_str());
                }
            }
            _ => panic!("apply_edge: unsupported form {:?} {} -> {}", form, from_f.id_str(), to_f.id_str()),
        }
    }
}
