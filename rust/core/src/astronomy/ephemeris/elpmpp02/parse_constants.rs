//! ELP-MPP02 解析常数：Delaunay、行星幅角、经度系数等。
//! 从 Elpmpp02Common.DE405 移植（Fortran INITIAL icor=1，拟合区间 1950–2060）。

use crate::math::angle::dms2rad;
use crate::math::real::RealOps;
use crate::math::series::arcsec_to_rad;

/// 解析用常数：ra0、平均运动比、Delaunay（前 4 行）、Planetary（前 8 行）等。
#[derive(Clone, Debug)]
pub struct ParseConstants {
    pub ra0: f64,
    pub ratio_mean_motion: f64,
    pub r_sma2dr_mm3: f64,
    pub delta_nu: f64,
    pub delta_e: f64,
    pub delta_gamma: f64,
    pub delta_np: f64,
    pub delta_ep: f64,
    /// Delaunay 幅角系数：4 行 × 5 列（前 4 行，用于转置）
    pub delaunay: [[f64; 5]; 4],
    /// 行星平均经度：8 行 × 5 列
    pub planetary: [[f64; 5]; 8],
    pub longitude_lunar_zeta: [f64; 5],
    pub longitude_lunar1: [f64; 5],
    pub laskars_p: [f64; 6],
    pub laskars_q: [f64; 6],
}

/// DE405 拟合常数。
pub fn de405() -> ParseConstants {
    let a0_elp = 384747.980674318_f64;
    let a0_de405 = 384747.9613701725_f64;
    let ra0 = a0_de405 / a0_elp;
    let ratio_semi_major_axis = 0.002571881_f64;
    let ratio_mean_motion = 0.074801329_f64;
    let r_sma2d3 = 2.0 * ratio_semi_major_axis / 3.0;
    let r_sma2dr_mm3 = r_sma2d3 / ratio_mean_motion;
    let iau_precession = 5029.0966_f64;
    let delta_iau_precession = -0.29965_f64;

    let dlongitude_lunar1_0 = -0.07008_f64;
    let dlongitude_lunar2_0 = 0.20794_f64;
    let dlongitude_lunar3_0 = -0.07215_f64;
    let dlongitude_lunar1_1 = -0.35106_f64;
    let dlongitude_lunar2_1 = 0.08017_f64;
    let dlongitude_lunar3_1 = -0.04317_f64;
    let dlongitude_lunar1_2 = -0.03743_f64;
    let d_gamma = 0.00085_f64;
    let d_e = -0.00006_f64;
    let dlongitude_emb_0 = -0.00033_f64;
    let dlongitude_emb_1 = 0.00732_f64;
    let dperihelion_emb = -0.00749_f64;
    let d_ep = 0.00224_f64;

    let sec = |x: f64| arcsec_to_rad(x).as_f64();

    let mut longitude_lunar: [[f64; 5]; 4] = [
        [
            dms2rad(218.0, 18.0, 59.95571).as_f64() + sec(dlongitude_lunar1_0),
            sec(1732559343.73604 + dlongitude_lunar1_1),
            sec(-6.8084 + dlongitude_lunar1_2),
            sec(0.66040e-2),
            sec(-0.31690e-4),
        ],
        [
            dms2rad(83.0, 21.0, 11.67475).as_f64() + sec(dlongitude_lunar2_0),
            sec(14643420.3171 + dlongitude_lunar2_1),
            sec(-38.2631),
            sec(-0.45047e-1),
            sec(0.21301e-3),
        ],
        [
            dms2rad(125.0, 2.0, 40.39816).as_f64() + sec(dlongitude_lunar3_0),
            sec(-6967919.5383 + dlongitude_lunar3_1),
            sec(6.3590),
            sec(0.76250e-2),
            sec(-0.35860e-4),
        ],
        [0.0; 5],
    ];

    let longitude_emb: [f64; 5] = [
        dms2rad(100.0, 27.0, 59.13885).as_f64() + sec(dlongitude_emb_0),
        sec(129597742.29300 + dlongitude_emb_1),
        sec(-0.020200),
        sec(0.90000e-5),
        sec(0.15000e-6),
    ];

    let perihelion_emb: [f64; 5] = [
        dms2rad(102.0, 56.0, 14.45766).as_f64() + sec(dperihelion_emb),
        sec(1161.24342),
        sec(0.529265),
        sec(-0.11814e-3),
        sec(0.11379e-4),
    ];

    let d_bp: [[f64; 2]; 6] = [
        [0.0, 0.0],
        [0.311079095, -0.103837907],
        [-0.004482398, 0.000668287],
        [-0.001102485, -0.001298072],
        [0.001056062, -0.000178028],
        [0.000050928, -0.000037342],
    ];

    let x2 = longitude_lunar[1][1] / longitude_lunar[0][1];
    let x3 = longitude_lunar[2][1] / longitude_lunar[0][1];
    let y2 = ratio_mean_motion * d_bp[1][0] + r_sma2d3 * d_bp[5][0];
    let y3 = ratio_mean_motion * d_bp[1][1] + r_sma2d3 * d_bp[5][1];
    let d21 = x2 - y2;
    let d22 = longitude_lunar[0][1] * d_bp[2][0];
    let d23 = longitude_lunar[0][1] * d_bp[3][0];
    let d24 = longitude_lunar[0][1] * d_bp[4][0];
    let d25 = y2 / ratio_mean_motion;
    let d31 = x3 - y3;
    let d32 = longitude_lunar[0][1] * d_bp[2][1];
    let d33 = longitude_lunar[0][1] * d_bp[3][1];
    let d34 = longitude_lunar[0][1] * d_bp[4][1];
    let d35 = y3 / ratio_mean_motion;
    let corr_w2 = sec(
        d21 * dlongitude_lunar1_1 + d22 * d_gamma + d23 * d_e + d24 * d_ep + d25 * dlongitude_emb_1,
    );
    let corr_w3 = sec(
        d31 * dlongitude_lunar1_1 + d32 * d_gamma + d33 * d_e + d34 * d_ep + d35 * dlongitude_emb_1,
    );
    longitude_lunar[1][1] += corr_w2;
    longitude_lunar[2][1] += corr_w3;

    let l0 = longitude_lunar[0];
    let l1 = longitude_lunar[1];
    let l2 = longitude_lunar[2];
    let pi = core::f64::consts::PI;
    let delaunay: [[f64; 5]; 4] = [
        [
            l0[0] - longitude_emb[0] + pi,
            l0[1] - longitude_emb[1],
            l0[2] - longitude_emb[2],
            l0[3] - longitude_emb[3],
            l0[4] - longitude_emb[4],
        ],
        [
            l0[0] - l2[0],
            l0[1] - l2[1],
            l0[2] - l2[2],
            l0[3] - l2[3],
            l0[4] - l2[4],
        ],
        [
            l0[0] - l1[0],
            l0[1] - l1[1],
            l0[2] - l1[2],
            l0[3] - l1[3],
            l0[4] - l1[4],
        ],
        [
            longitude_emb[0] - perihelion_emb[0],
            longitude_emb[1] - perihelion_emb[1],
            longitude_emb[2] - perihelion_emb[2],
            longitude_emb[3] - perihelion_emb[3],
            longitude_emb[4] - perihelion_emb[4],
        ],
    ];

    let plan_l0_dms: [(f64, f64, f64); 8] = [
        (252.0, 15.0, 3.216919),
        (181.0, 58.0, 44.758419),
        (100.0, 27.0, 59.138850),
        (355.0, 26.0, 3.642778),
        (34.0, 21.0, 5.379392),
        (50.0, 4.0, 38.902495),
        (314.0, 3.0, 4.354234),
        (304.0, 20.0, 56.808371),
    ];
    let plan_l1: [f64; 8] = [
        538101628.66888,
        210664136.45777,
        129597742.29300,
        68905077.65936,
        10925660.57335,
        4399609.33632,
        1542482.57845,
        786547.89700,
    ];
    let mut planetary: [[f64; 5]; 8] = [[0.0; 5]; 8];
    for i in 0..8 {
        let (d, m, s) = plan_l0_dms[i];
        planetary[i][0] = dms2rad(d, m, s).as_f64();
        planetary[i][1] = sec(plan_l1[i]);
        planetary[i][2] = 0.0;
        planetary[i][3] = 0.0;
        planetary[i][4] = 0.0;
    }

    let longitude_lunar_zeta: [f64; 5] = [
        longitude_lunar[0][0],
        longitude_lunar[0][1] + sec(iau_precession + delta_iau_precession),
        longitude_lunar[0][2],
        longitude_lunar[0][3],
        longitude_lunar[0][4],
    ];

    let longitude_lunar1: [f64; 5] = longitude_lunar[0];

    let delta_nu = sec(0.55604 + dlongitude_lunar1_1) / longitude_lunar[0][1];
    let delta_e = sec(0.01789 + d_e);
    let delta_gamma = sec(-0.08066 + d_gamma);
    let delta_np = sec(-0.06424 + dlongitude_emb_1) / longitude_lunar[0][1];
    let delta_ep = sec(-0.12879 + d_ep);

    let laskars_p: [f64; 6] = [
        0.0,
        0.10180391e-04,
        0.47020439e-06,
        -0.5417367e-09,
        -0.2507948e-11,
        0.463486e-14,
    ];
    let laskars_q: [f64; 6] = [
        0.0,
        -0.113469002e-03,
        0.12372674e-06,
        0.1265417e-08,
        -0.1371808e-11,
        -0.320334e-14,
    ];

    ParseConstants {
        ra0,
        ratio_mean_motion,
        r_sma2dr_mm3,
        delta_nu,
        delta_e,
        delta_gamma,
        delta_np,
        delta_ep,
        delaunay,
        planetary,
        longitude_lunar_zeta,
        longitude_lunar1,
        laskars_p,
        laskars_q,
    }
}
