import * as Decimal from '../../math/decimal.hp.js';
import * as Angle from '../../math/angle.js';
import * as Polynomial from '../../math/polynomial.js';

const decimal = Decimal.decimal;
const angle = Angle.angle;
const polynomial = Polynomial.polynomial;

// ftp://maia.usno.navy.mil/conventions/archive/2003/chapter5/NU2000B.f
const IAU2000B = {
    "fundamental_arguments": { // arcseconds
        //mean anomaly of the moon, zh-cn:月球平近点角
        "l": "485868.249036, 1717915923.2178, 31.8792, 0.051635, -0.00024470",
        //mean anomaly of the sun, zh-cn:太阳平近点角
        "lp": "1287104.79305, 129596581.0481, -0.5532, -0.000136, -0.00001149", // lp = l', p means prime
        //mean argument of the latitude of the moon, zh-cn:月球平黄经-月球升交点平黄经
        "F": "335779.526232, 1739527262.8478, -12.7512, -0.001037, 0.00000417",
        //mean elongation of the moon from the sun, zh-cn:日月间的平角距
        "D": "1072260.70369, 1602961601.2090, -6.3706, 0.006593, -0.00003169",
        //mean longitude of the ascending node of the moon, zh-cn:月球升交点平黄经
        "Omega": "450160.398036, -6962890.5431, 7.4722, 0.007702, 0.00005939",
    },
    "luni_solar_nutation": [ // 77 terms, unit 1e-7 arcsec, 0.1 microarcsecond
        /*    Coefficients for         longitude coefficients       obliquity coefficients
            fundamental arguments
          l   l'  F   D  Omega         A        A'      A"           B        B'     B"              */
        " 0,  0,  0,  0,  1,     -172064161, -174666,  33386,     92052331,  9086, 15377",
        " 0,  0,  2, -2,  2,      -13170906,   -1675, -13696,      5730336, -3015, -4587",
        " 0,  0,  2,  0,  2,       -2276413,    -234,   2796,       978459,  -485,  1374",
        " 0,  0,  0,  0,  2,        2074554,     207,   -698,      -897492,   470,  -291",
        " 0,  1,  0,  0,  0,        1475877,   -3633,  11817,        73871,  -184, -1924",
        " 0,  1,  2, -2,  2,        -516821,    1226,   -524,       224386,  -677,  -174",
        " 1,  0,  0,  0,  0,         711159,      73,   -872,        -6750,     0,   358",
        " 0,  0,  2,  0,  1,        -387298,    -367,    380,       200728,    18,   318",
        " 1,  0,  2,  0,  2,        -301461,     -36,    816,       129025,   -63,   367",
        " 0, -1,  2, -2,  2,         215829,    -494,    111,       -95929,   299,   132",

        " 0,  0,  2, -2,  1,         128227,     137,   181,        -68982,    -9,    39",
        "-1,  0,  2,  0,  2,         123457,      11,    19,        -53311,    32,    -4",
        "-1,  0,  0,  2,  0,         156994,      10,  -168,         -1235,     0,    82",
        " 1,  0,  0,  0,  1,          63110,      63,    27,        -33228,     0,    -9",
        "-1,  0,  0,  0,  1,         -57976,     -63,  -189,         31429,     0,   -75",
        "-1,  0,  2,  2,  2,         -59641,     -11,   149,         25543,   -11,    66",
        " 1,  0,  2,  0,  1,         -51613,     -42,   129,         26366,     0,    78",
        "-2,  0,  2,  0,  1,          45893,      50,    31,        -24236,   -10,    20",
        " 0,  0,  0,  2,  0,          63384,      11,  -150,         -1220,     0,    29",
        " 0,  0,  2,  2,  2,         -38571,      -1,   158,         16452,   -11,    68",

        " 0, -2,  2, -2,  2,          32481,       0,     0,        -13870,     0,     0",
        "-2,  0,  0,  2,  0,         -47722,       0,   -18,           477,     0,   -25",
        " 2,  0,  2,  0,  2,         -31046,      -1,   131,         13238,   -11,    59",
        " 1,  0,  2, -2,  2,          28593,       0,    -1,        -12338,    10,    -3",
        "-1,  0,  2,  0,  1,          20441,      21,    10,        -10758,     0,    -3",
        " 2,  0,  0,  0,  0,          29243,       0,   -74,          -609,     0,    13",
        " 0,  0,  2,  0,  0,          25887,       0,   -66,          -550,     0,    11",
        " 0,  1,  0,  0,  1,         -14053,     -25,    79,          8551,    -2,   -45",
        "-1,  0,  0,  2,  1,          15164,      10,    11,         -8001,     0,    -1",
        " 0,  2,  2, -2,  2,         -15794,      72,   -16,          6850,   -42,    -5",

        " 0,  0, -2,  2,  0,          21783,       0,    13,          -167,     0,    13",
        " 1,  0,  0, -2,  1,         -12873,     -10,   -37,          6953,     0,   -14",
        " 0, -1,  0,  0,  1,         -12654,      11,    63,          6415,     0,    26",
        "-1,  0,  2,  2,  1,         -10204,       0,    25,          5222,     0,    15",
        " 0,  2,  0,  0,  0,          16707,     -85,   -10,           168,    -1,    10",
        " 1,  0,  2,  2,  2,          -7691,       0,    44,          3268,     0,    19",
        "-2,  0,  2,  0,  0,         -11024,       0,   -14,           104,     0,     2",
        " 0,  1,  2,  0,  2,           7566,     -21,   -11,         -3250,     0,    -5",
        " 0,  0,  2,  2,  1,          -6637,     -11,    25,          3353,     0,    14",
        " 0, -1,  2,  0,  2,          -7141,      21,     8,          3070,     0,     4",

        " 0,  0,  0,  2,  1,          -6302,     -11,     2,          3272,     0,     4",
        " 1,  0,  2, -2,  1,           5800,      10,     2,         -3045,     0,    -1",
        " 2,  0,  2, -2,  2,           6443,       0,    -7,         -2768,     0,    -4",
        "-2,  0,  0,  2,  1,          -5774,     -11,   -15,          3041,     0,    -5",
        " 2,  0,  2,  0,  1,          -5350,       0,    21,          2695,     0,    12",
        " 0, -1,  2, -2,  1,          -4752,     -11,    -3,          2719,     0,    -3",
        " 0,  0,  0, -2,  1,          -4940,     -11,   -21,          2720,     0,    -9",
        "-1, -1,  0,  2,  0,           7350,       0,    -8,           -51,     0,     4",
        " 2,  0,  0, -2,  1,           4065,       0,     6,         -2206,     0,     1",
        " 1,  0,  0,  2,  0,           6579,       0,   -24,          -199,     0,     2",

        " 0,  1,  2, -2,  1,           3579,       0,     5,         -1900,     0,     1",
        " 1, -1,  0,  0,  0,           4725,       0,    -6,           -41,     0,     3",
        "-2,  0,  2,  0,  2,          -3075,       0,    -2,          1313,     0,    -1",
        " 3,  0,  2,  0,  2,          -2904,       0,    15,          1233,     0,     7",
        " 0, -1,  0,  2,  0,           4348,       0,   -10,           -81,     0,     2",
        " 1, -1,  2,  0,  2,          -2878,       0,     8,          1232,     0,     4",
        " 0,  0,  0,  1,  0,          -4230,       0,     5,           -20,     0,    -2",
        "-1, -1,  2,  2,  2,          -2819,       0,     7,          1207,     0,     3",
        "-1,  0,  2,  0,  0,          -4056,       0,     5,            40,     0,    -2",
        " 0, -1,  2,  2,  2,          -2647,       0,    11,          1129,     0,     5",

        "-2,  0,  0,  0,  1,          -2294,       0,   -10,          1266,     0,    -4",
        " 1,  1,  2,  0,  2,           2481,       0,    -7,         -1062,     0,    -3",
        " 2,  0,  0,  0,  1,           2179,       0,    -2,         -1129,     0,    -2",
        "-1,  1,  0,  1,  0,           3276,       0,     1,            -9,     0,     0",
        " 1,  1,  0,  0,  0,          -3389,       0,     5,            35,     0,    -2",
        " 1,  0,  2,  0,  0,           3339,       0,   -13,          -107,     0,     1",
        "-1,  0,  2, -2,  1,          -1987,       0,    -6,          1073,     0,    -2",
        " 1,  0,  0,  0,  2,          -1981,       0,     0,           854,     0,     0",
        "-1,  0,  0,  1,  0,           4026,       0,  -353,          -553,     0,  -139",
        " 0,  0,  2,  1,  2,           1660,       0,    -5,          -710,     0,    -2",

        "-1,  0,  2,  4,  2,          -1521,       0,     9,           647,     0,     4",
        "-1,  1,  0,  1,  1,           1314,       0,     0,          -700,     0,     0",
        " 0, -2,  2, -2,  1,          -1283,       0,     0,           672,     0,     0",
        " 1,  0,  2,  2,  1,          -1331,       0,     8,           663,     0,     4",
        "-2,  0,  2,  2,  2,           1383,       0,    -2,          -594,     0,    -2",
        "-1,  0,  0,  0,  2,           1405,       0,     4,          -610,     0,     2",
        " 1,  1,  2, -2,  2,           1290,       0,     0,          -556,     0,     0"
    ],
    "planetary_nutation": { // milliarcseconds
        "psi": "-0.135",
        "epsilon": "+0.388"
    }
};

// (A+Ap*t)*sin(fi)+App*cos(fi)
const luni_solar_longitude = (A, Ap, App, t, fi) => Decimal.plus(Decimal.mult(Decimal.plus(A, Decimal.mult(Ap, t)), Decimal.sin(fi)), Decimal.mult(App, Decimal.cos(fi)));
// (B+Bp*t)*cos(fi)+Bpp*sin(fi)
const luni_solar_obliquity = (B, Bp, Bpp, t, fi) => Decimal.plus(Decimal.mult(Decimal.plus(B, Decimal.mult(Bp, t)), Decimal.cos(fi)), Decimal.mult(Bpp, Decimal.sin(fi)));
//
const argument = (l, lp, F, D, Omega, coefficients) => Decimal.sum(Decimal.mult(l, coefficients[0]), Decimal.mult(lp, coefficients[1]), Decimal.mult(F, coefficients[2]), Decimal.mult(D, coefficients[3]), Decimal.mult(Omega, coefficients[4]));
//
const fundamental_arguments = (name, t) => Polynomial.value(IAU2000B.fundamental_arguments[name].replace(/s+/g, "").split(","), t);

export const nutaion = t => {
    t = decimal(t);
    let l = fundamental_arguments("l", t);
    let lp = fundamental_arguments("lp", t);
    let F = fundamental_arguments("F", t);
    let D = fundamental_arguments("D", t);
    let Omega = fundamental_arguments("Omega", t);
    let terms = IAU2000B.luni_solar_nutation.map(x => {
        x = x.replace(/s+/g, "").split(",").map(y => decimal(y));
        let fi = argument(l, lp, F, D, Omega, x.slice(0, 5));
        return [luni_solar_longitude(x[5], x[6], x[7], t, fi), luni_solar_obliquity(x[8], x[9], x[10], t, fi)];
    });
    let luni_solar_nutation = terms.reduce((lsn, x) => [Decimal.plus(lsn[0], x[0]), Decimal.plus(lsn[1], x[1])]);
    return {
        "psi": Angle.sec2rad(Decimal.plus(Decimal.mult(luni_solar_nutation[0], 1e-7), Decimal.mult(IAU2000B.planetary_nutation["psi"], 1e-6))),
        "epsilon": Angle.sec2rad(Decimal.plus(Decimal.mult(luni_solar_nutation[1], 1e-7), Decimal.mult(IAU2000B.planetary_nutation["epsilon"], 1e-6)))
    };
};
