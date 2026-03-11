//! 年光行差：真方向 → 视方向及导数。光速使用物理量 [crate::astronomy::constant::light_speed]。

use crate::astronomy::constant;
use crate::math::real::{real, Real};

fn dot(a: [Real; 3], b: [Real; 3]) -> Real {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn scale(s: Real, v: [Real; 3]) -> [Real; 3] {
    [s * v[0], s * v[1], s * v[2]]
}

fn add(a: [Real; 3], b: [Real; 3]) -> [Real; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [Real; 3], b: [Real; 3]) -> [Real; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn norm(v: [Real; 3]) -> Real {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize(v: [Real; 3]) -> [Real; 3] {
    let n = norm(v);
    if n <= real(0.0) {
        return v;
    }
    [v[0] / n, v[1] / n, v[2] / n]
}

/// 年光行差：几何方向 r、地心速度 v（AU/day），返回视方向单位向量。
/// 约定：e 为观测者到源的几何方向；视方向 e' = e + (1/c)[v − (e·v)e]（文献 e.g. doc/6）。
pub fn annual_aberration_direction(r: [Real; 3], v: [Real; 3]) -> [Real; 3] {
    let e = normalize(r);
    let edotv = dot(e, v);
    let one_over_c = real(1.0) / constant::light_speed_au_per_day();
    let correction = sub(v, scale(edotv, e));
    let e_corrected = add(e, scale(one_over_c, correction));
    normalize(e_corrected)
}

/// 年光行差方向对时间的导数。
pub fn annual_aberration_direction_derivative(
    r: [Real; 3],
    v: [Real; 3],
    dr_dt: [Real; 3],
    dv_dt: [Real; 3],
) -> [Real; 3] {
    let r_norm = norm(r);
    if r_norm <= real(0.0) {
        return [real(0.0), real(0.0), real(0.0)];
    }
    let e = normalize(r);
    let edotv = dot(e, v);
    let one_over_c = real(1.0) / constant::light_speed_au_per_day();
    let edotdr = dot(e, dr_dt);
    let de_dt = scale(real(1.0) / r_norm, sub(dr_dt, scale(edotdr, e)));
    let dedotv_dt = dot(e, dv_dt) + dot(de_dt, v);
    let correction_dt = scale(
        one_over_c,
        sub(dv_dt, add(scale(dedotv_dt, e), scale(edotv, de_dt))),
    );
    let df_dt = add(de_dt, correction_dt);
    let correction = scale(one_over_c, sub(v, scale(edotv, e)));
    let f = add(e, correction);
    let f_norm = norm(f);
    if f_norm <= real(0.0) {
        return [real(0.0), real(0.0), real(0.0)];
    }
    let e_app = normalize(f);
    let e_app_dot_df = dot(e_app, df_dt);
    scale(real(1.0) / f_norm, sub(df_dt, scale(e_app_dot_df, e_app)))
}
