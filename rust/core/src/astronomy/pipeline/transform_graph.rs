//! 状态转移路由：纯物理旋转，图搜索最短路径（文档 3.1、4.2、7.2）。
//! 架变换使用完整 6×6 状态转移 [R R_dot; 0 R]，R_dot 为旋转对时间的导数（科里奥利项）。
//! 下方共用段（岁差、章动、真黄赤交角）当 t 确定时变换矩阵仅依赖 t，按历元缓存。
//!
//! **时间尺度**：岁差（P03/Vondrak）、章动（IAU）公式规定入参 **TT**；`transform_to(_, _, jd_tt)` 的 `jd_tt` 须为 TT。

use crate::astronomy::frame::precession::{
    mean_obliquity, precession_derivative_times_vector_for, precession_transform_for, PrecessionModel,
};
use crate::astronomy::frame::nutation::{eps_true_dot, nutation_derivative_times_vector, nutation_for_apparent};
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

/// R_x(ε) 对角的导数 dR_x/dε。角支持 Real。
fn rotation_x_derivative(angle_rad: impl ToReal) -> [[Real; 3]; 3] {
    let a = real(angle_rad);
    let (c, s) = (a.cos(), a.sin());
    [[zero(), zero(), zero()], [zero(), -s, -c], [zero(), c, -s]]
}

fn scale_matrix(s: impl ToReal, m: &[[Real; 3]; 3]) -> [[Real; 3]; 3] {
    crate::math::algebra::mat::Mat::<Real, 3, 3>::from(*m).scale(real(s)).to_array()
}

/// 黄道拟合路径的展示用节点（补丁在 L,B，再旋转；与赤道拟合并列）。
const ECLIPTIC_PATCH_NODE_ID: &'static str = "Vsop87De406EclipticPatch";
/// 历元平黄道拆为两节点，赤道修正与黄道修正两条路不交叉。
const MEAN_ECLIPTIC_EQUATOR: &'static str = "MeanEcliptic(epoch)_赤道";
const MEAN_ECLIPTIC_ECLIPTIC: &'static str = "MeanEcliptic(epoch)_黄道";
/// FK5 拆为两节点：赤道 patch 经未修正→拟合→ICRS→已修正。
const FK5_UNCORRECTED: &'static str = "FK5_未修正";
const FK5_CORRECTED: &'static str = "FK5_已修正";

/// 按 (from_id, to_id) 返回管线步骤标签；仅写动作，不重复标架/坐标系（框内已示）。
fn edge_label(from_id: &str, to_id: &str) -> Option<String> {
    let label = match (from_id, to_id) {
        ("VSOP87", MEAN_ECLIPTIC_EQUATOR) => "光行时→tr；历表输出",
        ("VSOP87", ECLIPTIC_PATCH_NODE_ID) => "光行时→tr；黄道拟合：L,B 拟合修正",
        (ECLIPTIC_PATCH_NODE_ID, MEAN_ECLIPTIC_ECLIPTIC) => "R_x(ε₀)+Frame bias",
        ("ELPMPP02", "ELPMPP02_MEAN_LUNAR") => "历表求值（含 DE405/Table6 修正）",
        ("ELPMPP02_MEAN_LUNAR", MEAN_ECLIPTIC_ECLIPTIC) => "Laskar P,Q 旋转",
        (MEAN_ECLIPTIC_EQUATOR, FK5_UNCORRECTED) => "黄赤交角 R_x(ε₀)",
        (MEAN_ECLIPTIC_ECLIPTIC, FK5_CORRECTED) => "黄赤交角 R_x(ε₀)",
        (FK5_UNCORRECTED, "VsopToDe406IcrsFit") => "Frame bias B⁻¹ + DE406 拟合修正",
        ("VsopToDe406IcrsFit", "ICRS") => "恒等",
        ("ICRS", FK5_CORRECTED) => "B（Frame bias，GCRS→FK5）",
        (FK5_CORRECTED, "MeanEquator(epoch)") => "岁差（P03）",
        ("MeanEquator(epoch)", "TrueEquator(epoch)") => "章动",
        ("TrueEquator(epoch)", "ApparentEcliptic(epoch)") => "R_x(ε) 真黄赤交角",
        _ => "几何变换",
    };
    Some(label.to_string())
}

/// 单条有向边：从 from 到 to，代价 cost（用于路径搜索）。
#[derive(Clone, Debug)]
pub struct TransformEdge {
    pub from_frame: ReferenceFrame,
    pub to_frame: ReferenceFrame,
    pub cost: u32,
}

/// 单条边的可视化数据：架 id、代价与可选步骤标签（如 "P03岁差"、"B+DE406 patch"），便于序列化到 WASM/JS。
#[derive(Clone, Debug)]
pub struct TransformEdgeViz {
    pub from_id: String,
    pub to_id: String,
    pub cost: u32,
    /// 管线步骤说明，如 "P03岁差"、"章动"、"B+DE406 patch"。
    pub label: Option<String>,
}

/// 变换图的可视化数据：节点 id 列表与边列表，供前端绘图。
#[derive(Clone, Debug)]
pub struct TransformGraphVizData {
    pub node_ids: Vec<String>,
    pub edges: Vec<TransformEdgeViz>,
}

/// 下方共用段（岁差、章动、R_x(ε)）按历元缓存：t 确定时矩阵不需重算。
#[derive(Clone, Debug)]
struct EpochTransitionsCache {
    jd_tt_key: f64,
    precession: StateTransition6,
    nutation: StateTransition6,
    obliquity: StateTransition6,
}

/// 历元缓存容差（日）：|jd_tt - cache_key| < 此值则复用岁差/章动/黄赤交角矩阵。
/// 1e-9 保证同一次调用内几乎只复用同一 jd；若批量算多年节气可适当放宽（如 0.01）以减少 77 项章动重算，精度影响在亚角秒级。
const EPOCH_CACHE_TOLERANCE_DAY: f64 = 1e-9;

fn fill_epoch_transitions_cache(
    jd_tt: Real,
    jd_f64: f64,
    precession_model: PrecessionModel,
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
    let n_t = crate::astronomy::apparent::nutation_matrix_transposed(t_cent);
    let nutation = StateTransition6 {
        from_frame: to_me,
        to_frame: to_te,
        r: n_t,
        r_dot: build_r_dot_from_derivative_times_vector(|r| {
            nutation_derivative_times_vector(r, t_cent, precession_model)
        }),
    };
    let (_, deps) = nutation_for_apparent(t_cent);
    let eps_mean = mean_obliquity(t_cent).rad();
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

/// 变换图：注册边与矩阵提供者，按需求路径并施加 6D 变换。
pub struct TransformGraph {
    edges: Vec<TransformEdge>,
    /// 岁差模型：FK5 → MeanEquator 时使用。
    pub precession_model: PrecessionModel,
    /// 下方共用段按 jd_tt 缓存，避免同历元重复计算岁差/章动/真黄赤交角矩阵。
    epoch_cache: RefCell<Option<EpochTransitionsCache>>,
}

impl TransformGraph {
    pub fn new() -> Self {
        Self {
            edges: vec![],
            precession_model: PrecessionModel::P03,
            epoch_cache: RefCell::new(None),
        }
    }

    /// 指定岁差模型（定气用 P03，长期可用 Vondrak2011）。
    pub fn with_precession_model(mut self, model: PrecessionModel) -> Self {
        self.precession_model = model;
        self
    }

    pub fn add_edge(&mut self, from: ReferenceFrame, to: ReferenceFrame, cost: u32) {
        self.edges.push(TransformEdge {
            from_frame: from,
            to_frame: to,
            cost,
        });
    }

    /// 构建默认图：历表输出架 MeanEcliptic(J2000) → FK5，FK5 ↔ MeanEquator ↔ TrueEquator ↔ ApparentEcliptic，ICRS ↔ FK5。
    pub fn default_graph() -> Self {
        let mut g = Self::new();
        g.add_edge(
            ReferenceFrame::MeanEcliptic(Epoch::j2000()),
            ReferenceFrame::FK5,
            8,
        );
        g.add_edge(ReferenceFrame::FK5, ReferenceFrame::MeanEquator(Epoch::new(real(0.0))), 10);
        g.add_edge(ReferenceFrame::MeanEquator(Epoch::new(real(0.0))), ReferenceFrame::TrueEquator(Epoch::new(real(0.0))), 5);
        g.add_edge(ReferenceFrame::TrueEquator(Epoch::new(real(0.0))), ReferenceFrame::ApparentEcliptic(Epoch::new(real(0.0))), 2);
        g.add_edge(ReferenceFrame::ICRS, ReferenceFrame::FK5, 1);
        g.add_edge(ReferenceFrame::FK5, ReferenceFrame::ICRS, 1);
        g
    }

    /// Patch 步骤节点 id（可视化用）：FK5 → VsopToDe406IcrsFit → ICRS。
    const PATCH_NODE_ID: &'static str = "VsopToDe406IcrsFit";

    /// 返回图的可视化数据：赤道/黄道拆节点；FK5 拆为未修正→patch→ICRS→已修正。
    pub fn visualization_data(&self) -> TransformGraphVizData {
        let mean_ecliptic_id = "MeanEcliptic(epoch)";
        let fk5_id = "FK5";
        let mut node_ids: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::<&str>::new();
        for e in &self.edges {
            let f = e.from_frame.id_str();
            if f == mean_ecliptic_id {
                if seen.insert(MEAN_ECLIPTIC_EQUATOR) {
                    node_ids.push(MEAN_ECLIPTIC_EQUATOR.to_string());
                }
                if seen.insert(MEAN_ECLIPTIC_ECLIPTIC) {
                    node_ids.push(MEAN_ECLIPTIC_ECLIPTIC.to_string());
                }
            } else if f == fk5_id {
                if seen.insert(FK5_UNCORRECTED) {
                    node_ids.push(FK5_UNCORRECTED.to_string());
                }
                if seen.insert(FK5_CORRECTED) {
                    node_ids.push(FK5_CORRECTED.to_string());
                }
            } else if seen.insert(f) {
                node_ids.push(f.to_string());
            }
            let t = e.to_frame.id_str();
            if t == mean_ecliptic_id {
                if seen.insert(MEAN_ECLIPTIC_EQUATOR) {
                    node_ids.push(MEAN_ECLIPTIC_EQUATOR.to_string());
                }
                if seen.insert(MEAN_ECLIPTIC_ECLIPTIC) {
                    node_ids.push(MEAN_ECLIPTIC_ECLIPTIC.to_string());
                }
            } else if t == fk5_id {
                if seen.insert(FK5_UNCORRECTED) {
                    node_ids.push(FK5_UNCORRECTED.to_string());
                }
                if seen.insert(FK5_CORRECTED) {
                    node_ids.push(FK5_CORRECTED.to_string());
                }
            } else if seen.insert(t) {
                node_ids.push(t.to_string());
            }
        }
        let mut edges: Vec<TransformEdgeViz> = Vec::new();
        for e in &self.edges {
            let from_id = e.from_frame.id_str().to_string();
            let to_id = e.to_frame.id_str().to_string();
            if from_id == fk5_id && to_id == "ICRS" {
                if seen.insert(Self::PATCH_NODE_ID) {
                    node_ids.push(Self::PATCH_NODE_ID.to_string());
                }
                edges.push(TransformEdgeViz {
                    from_id: FK5_UNCORRECTED.to_string(),
                    to_id: Self::PATCH_NODE_ID.to_string(),
                    cost: e.cost,
                    label: edge_label(FK5_UNCORRECTED, Self::PATCH_NODE_ID),
                });
                edges.push(TransformEdgeViz {
                    from_id: Self::PATCH_NODE_ID.to_string(),
                    to_id: "ICRS".to_string(),
                    cost: 0,
                    label: edge_label(Self::PATCH_NODE_ID, "ICRS"),
                });
            } else if from_id == "ICRS" && to_id == fk5_id {
                edges.push(TransformEdgeViz {
                    from_id: "ICRS".to_string(),
                    to_id: FK5_CORRECTED.to_string(),
                    cost: e.cost,
                    label: edge_label("ICRS", FK5_CORRECTED),
                });
            } else if from_id == fk5_id && to_id == "MeanEquator(epoch)" {
                edges.push(TransformEdgeViz {
                    from_id: FK5_CORRECTED.to_string(),
                    to_id: to_id.clone(),
                    cost: e.cost,
                    label: edge_label(FK5_CORRECTED, &to_id),
                });
            } else if from_id == mean_ecliptic_id && to_id == fk5_id {
                edges.push(TransformEdgeViz {
                    from_id: MEAN_ECLIPTIC_EQUATOR.to_string(),
                    to_id: FK5_UNCORRECTED.to_string(),
                    cost: e.cost,
                    label: edge_label(MEAN_ECLIPTIC_EQUATOR, FK5_UNCORRECTED),
                });
                edges.push(TransformEdgeViz {
                    from_id: MEAN_ECLIPTIC_ECLIPTIC.to_string(),
                    to_id: FK5_CORRECTED.to_string(),
                    cost: e.cost,
                    label: edge_label(MEAN_ECLIPTIC_ECLIPTIC, FK5_CORRECTED),
                });
            } else {
                edges.push(TransformEdgeViz {
                    from_id: from_id.clone(),
                    to_id: to_id.clone(),
                    cost: e.cost,
                    label: edge_label(&from_id, &to_id),
                });
            }
        }
        if node_ids.iter().any(|s| s == MEAN_ECLIPTIC_EQUATOR) {
            let prepend_nodes: Vec<String> = vec![
                "VSOP87".to_string(),
                ECLIPTIC_PATCH_NODE_ID.to_string(),
                "ELPMPP02".to_string(),
                "ELPMPP02_MEAN_LUNAR".to_string(),
            ];
            node_ids.splice(0..0, prepend_nodes);
            let prepend_edges = vec![
                TransformEdgeViz {
                    from_id: "VSOP87".to_string(),
                    to_id: MEAN_ECLIPTIC_EQUATOR.to_string(),
                    cost: 0,
                    label: edge_label("VSOP87", MEAN_ECLIPTIC_EQUATOR),
                },
                TransformEdgeViz {
                    from_id: "VSOP87".to_string(),
                    to_id: ECLIPTIC_PATCH_NODE_ID.to_string(),
                    cost: 0,
                    label: edge_label("VSOP87", ECLIPTIC_PATCH_NODE_ID),
                },
                TransformEdgeViz {
                    from_id: ECLIPTIC_PATCH_NODE_ID.to_string(),
                    to_id: MEAN_ECLIPTIC_ECLIPTIC.to_string(),
                    cost: 0,
                    label: edge_label(ECLIPTIC_PATCH_NODE_ID, MEAN_ECLIPTIC_ECLIPTIC),
                },
                TransformEdgeViz {
                    from_id: "ELPMPP02".to_string(),
                    to_id: "ELPMPP02_MEAN_LUNAR".to_string(),
                    cost: 0,
                    label: edge_label("ELPMPP02", "ELPMPP02_MEAN_LUNAR"),
                },
                TransformEdgeViz {
                    from_id: "ELPMPP02_MEAN_LUNAR".to_string(),
                    to_id: MEAN_ECLIPTIC_ECLIPTIC.to_string(),
                    cost: 0,
                    label: edge_label("ELPMPP02_MEAN_LUNAR", MEAN_ECLIPTIC_ECLIPTIC),
                },
            ];
            edges.splice(0..0, prepend_edges);
        }
        TransformGraphVizData { node_ids, edges }
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
        match (a, b) {
            (ReferenceFrame::FK5, ReferenceFrame::FK5) => true,
            (ReferenceFrame::ICRS, ReferenceFrame::ICRS) => true,
            (ReferenceFrame::MeanEcliptic(_), ReferenceFrame::MeanEcliptic(_)) => true,
            (ReferenceFrame::MeanEquator(_), ReferenceFrame::MeanEquator(_)) => true,
            (ReferenceFrame::TrueEquator(_), ReferenceFrame::TrueEquator(_)) => true,
            (ReferenceFrame::ApparentEcliptic(_), ReferenceFrame::ApparentEcliptic(_)) => true,
            _ => false,
        }
    }

    /// 获取从 from 到 to 在给定 jd_tt 下的 6×6 转移（若已知）；含 R_dot 用于科里奥利项。
    /// 下方共用段（岁差、章动、真黄赤交角）按 jd_tt 缓存，t 确定时矩阵不重算。
    pub fn get_transition(&self, from: ReferenceFrame, to: ReferenceFrame, jd_tt: Real) -> Option<StateTransition6> {
        let jd_f64 = jd_tt.as_f64();
        match (from, to) {
            // 黄道→赤道：R_x(-ε)，与历元无关，不缓存
            (ReferenceFrame::MeanEcliptic(_), ReferenceFrame::FK5) => {
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
                if cache.as_ref().map(|c| (c.jd_tt_key - jd_f64).abs() < EPOCH_CACHE_TOLERANCE_DAY).unwrap_or(false) {
                    return Some(cache.as_ref().unwrap().precession.clone());
                }
                let filled = fill_epoch_transitions_cache(jd_tt, jd_f64, self.precession_model);
                let out = filled.precession.clone();
                *cache = Some(filled);
                Some(out)
            }
            (_, ReferenceFrame::TrueEquator(_)) if matches!(from, ReferenceFrame::MeanEquator(_)) => {
                let mut cache = self.epoch_cache.borrow_mut();
                if cache.as_ref().map(|c| (c.jd_tt_key - jd_f64).abs() < EPOCH_CACHE_TOLERANCE_DAY).unwrap_or(false) {
                    return Some(cache.as_ref().unwrap().nutation.clone());
                }
                let filled = fill_epoch_transitions_cache(jd_tt, jd_f64, self.precession_model);
                let out = filled.nutation.clone();
                *cache = Some(filled);
                Some(out)
            }
            (_, ReferenceFrame::ApparentEcliptic(_)) if matches!(from, ReferenceFrame::TrueEquator(_)) => {
                let mut cache = self.epoch_cache.borrow_mut();
                if cache.as_ref().map(|c| (c.jd_tt_key - jd_f64).abs() < EPOCH_CACHE_TOLERANCE_DAY).unwrap_or(false) {
                    return Some(cache.as_ref().unwrap().obliquity.clone());
                }
                let filled = fill_epoch_transitions_cache(jd_tt, jd_f64, self.precession_model);
                let out = filled.obliquity.clone();
                *cache = Some(filled);
                Some(out)
            }
            _ => None,
        }
    }

    /// 将 state 变换到 target 架；jd_tt 用于动态架。标量 Real。
    pub fn transform_to(&self, state: State6, target: ReferenceFrame, jd_tt: Real) -> State6 {
        let mut s = state;
        let to_me = ReferenceFrame::MeanEquator(Epoch::new(jd_tt));
        let to_te = ReferenceFrame::TrueEquator(Epoch::new(jd_tt));
        let to_ae = ReferenceFrame::ApparentEcliptic(Epoch::new(jd_tt));

        if matches!(s.frame(), ReferenceFrame::MeanEcliptic(_)) {
            if let Some(tr) = self.get_transition(s.frame(), ReferenceFrame::FK5, jd_tt) {
                s = s.apply_transition(&tr.r, &tr.r_dot, ReferenceFrame::FK5);
            }
        }
        if s.frame() == ReferenceFrame::ICRS {
            let (pos_m, vel_m) = s.to_meters_and_m_per_s();
            let (a, b, c) = crate::astronomy::frame::fk5_icrs::rotate_equatorial_icrs_to_fk5(
                pos_m[0], pos_m[1], pos_m[2],
            );
            let (va, vb, vc) = crate::astronomy::frame::fk5_icrs::rotate_equatorial_icrs_to_fk5(
                vel_m[0], vel_m[1], vel_m[2],
            );
            s = State6::from_si_in_frame(ReferenceFrame::FK5, a, b, c, va, vb, vc);
        }
        let need_me = matches!(target, ReferenceFrame::MeanEquator(_) | ReferenceFrame::TrueEquator(_) | ReferenceFrame::ApparentEcliptic(_));
        let need_te = matches!(target, ReferenceFrame::TrueEquator(_) | ReferenceFrame::ApparentEcliptic(_));
        let need_ae = matches!(target, ReferenceFrame::ApparentEcliptic(_));
        if s.frame() == ReferenceFrame::FK5 && need_me {
            if let Some(tr) = self.get_transition(s.frame(), to_me, jd_tt) {
                s = s.apply_transition(&tr.r, &tr.r_dot, to_me);
            }
        }
        if s.frame() == to_me && need_te {
            if let Some(tr) = self.get_transition(s.frame(), to_te, jd_tt) {
                s = s.apply_transition(&tr.r, &tr.r_dot, to_te);
            }
        }
        if s.frame() == to_te && need_ae {
            if let Some(tr) = self.get_transition(s.frame(), to_ae, jd_tt) {
                s = s.apply_transition(&tr.r, &tr.r_dot, to_ae);
            }
        }
        s
    }
}
