import * as std from '../basic.js';
import * as Decimal from './decimal.js';
const decimal = Decimal.decimal;

export const PI = Decimal.acos(-1);
export const DoublePi = Decimal.plus(PI, PI);

const SecondPerRadian = Decimal.div(180 * 60 * 60, PI);
const MinutePerRadian = Decimal.div(180 * 60, PI);
const DegreePerRadian = Decimal.div(180, PI);

const isHA = x => x === 'h' || x === 'h' || x === 's';
const isH = x => x === 'h' || x === '°' || x === '\u00b0';
const isM = x => x === 'm' || x === '\'' || x === '\u2032';
const isS = x => x === 's' || x === '"' || x === '\u2033';

const patterns = [
    [ // 0           1         2 3                                4                 5
        new RegExp(/^([+-]?)(([0-9]*\.?[0-9]+|[0-9]+\.?[0-9]*)([eE][+-]?[0-9]+)?)([hms°'"\u00b0\u2032\u2033])$/, "iu"),
        g => [g[1] !== '-', [isH(g[5]) ? g[2] : 0, isM(g[5]) ? g[2] : 0, isS(g[5]) ? g[2] : 0], isHA(g[5])]
    ],
    [ // 0           1      2       3                           4
        new RegExp(/^([+-]?)([0-9]+)([hms°'"\u00b0\u2032\u2033])(\.[0-9]+)$/, "iu"),
        g => [g[1] !== '-', [isH(g[3]) ? g[2] + g[4] : 0, isM(g[3]) ? g[2] + g[4] : 0, isS(g[3]) ? g[2] + g[4] : 0], isHA(g[3])]
    ],
    [ // 0           1      2 3                               4                  5           6           7           8
        new RegExp(/^([+-]?)(([0-9]*\.?[0-9]+|[0-9]+\.?[0-9]*)([eE][+-]?[0-9]+)?)([h°\u00b0])([0-5]?[0-9](\.[0-9]+)?)([m'\u2032])$/, "iu"),
        g => [g[1] !== '-', [g[2], g[6], 0], isHA(g[5])]
    ],
    [ // 0           1      2 3                               4                  5           6            7           8
        new RegExp(/^([+-]?)(([0-9]*\.?[0-9]+|[0-9]+\.?[0-9]*)([eE][+-]?[0-9]+)?)([h°\u00b0])([0-5]?[0-9])([m'\u2032])(\.[0-9]+)?$/, "iu"),
        g => [g[1] !== '-', [g[2], g[6] + g[8], 0], isHA(g[5])]
    ],
    [ // 0           1      2 3                               4                  5           6           7           8
        new RegExp(/^([+-]?)(([0-9]*\.?[0-9]+|[0-9]+\.?[0-9]*)([eE][+-]?[0-9]+)?)([m'\u2032])([0-5]?[0-9](\.[0-9]+)?)([s"\u2033])$/, "iu"),
        g => [g[1] !== '-', [0, g[2], g[6]], isHA(g[5])]
    ],
    [ // 0           1      2 3                               4                  5           6            7           8
        new RegExp(/^([+-]?)(([0-9]*\.?[0-9]+|[0-9]+\.?[0-9]*)([eE][+-]?[0-9]+)?)([m'\u2032])([0-5]?[0-9])([s"\u2033])(\.[0-9]+)?$/, "iu"),
        g => [g[1] !== '-', [0, g[2], g[6] + g[8]], isHA(g[5])]
    ],
    [ // 0           1      2 3                               4                  5           6            7           8           9
        new RegExp(/^([+-]?)(([0-9]*\.?[0-9]+|[0-9]+\.?[0-9]*)([eE][+-]?[0-9]+)?)([h°\u00b0])([0-5]?[0-9])([m'\u2032])([0-5]?[0-9](\.[0-9]+)?)[s"\u2033]$/, "iu"),
        g => [g[1] !== '-', [g[2], g[6], g[8]], isHA(g[5])]
    ],
    [ // 0           1      2 3                               4                  5           6            7           8                      9
        new RegExp(/^([+-]?)(([0-9]*\.?[0-9]+|[0-9]+\.?[0-9]*)([eE][+-]?[0-9]+)?)([h°\u00b0])([0-5]?[0-9])([m'\u2032])([0-5]?[0-9])[s"\u2033](\.[0-9]+)?$/, "iu"),
        g => [g[1] !== '-', [g[2], g[6], g[8] + g[9]], isHA(g[5])]
    ]
];
// String -> (Boolean,(Decimal,Decimal,Decimal),Boolean)
const match = s => {
    for (let p of patterns) {
        let groups = p[0].exec(s);
        if (groups) return p[1](groups);
    }
    throw new Error("Parse Error!");
}
export const sec2rad = s => Decimal.div(s, SecondPerRadian);
const ha2rad = std.uncurry(a => ha => ha ? Decimal.mult(a, 15) : a);
const zip_sec = arr_hms => Decimal.plus(Decimal.plus(Decimal.mult(decimal(arr_hms[0]), 3600), Decimal.mult(decimal(arr_hms[1]), 60)), decimal(arr_hms[2]));
const sign = std.uncurry(a => sgn => sgn ? a : Decimal.neg(a));
// String -> Angle
export const parse = s => {
    let grps = match(s.replace(/\s+/g, ''));
    return angle(sign(ha2rad(sec2rad(zip_sec(grps[1])), grps[2]), grps[0]));
}
// Angle = Decimal | String
export const angle = x => typeof x === "string"? parse(x) : decimal(x);
// dms | HMS,  case sensitive
const sym = fmt => /^H?M?S?$/.test(fmt) ? "hms" : "\u00b0\u2032\u2033";
// Decimal -> Boolean -> [Decimal]
const rad2hdms = std.uncurry(rad => isHMS =>{
    let res = [];
    res[0] = Decimal.div(Decimal.mult(rad, isHMS? 12: 180), PI); // s
    res[1] = Decimal.mult(res[0], 60); // m
    res[2] = Decimal.mult(res[1], 60); // s
    return res;
});
// [Decimal] -> [Decimal]
const carry_out = arr => {
    for (let i = arr.length; i-- > 1;) {
        if (Decimal.gte(arr[i], 60)) {
            arr[i - 1] = Decimal.plus(arr[i - 1], 1);
            arr[i] = Decimal.minus(arr[i], 60);
        } else {
            break;
        }
    }
    return arr;
}
// Decimal->String->Number->String
const rad2str = std.uncurry(radian=> format=> fixed=> {
    if (false == /^(H?M?S?)|(d?m?s?)$/.test(format)) {
        throw new Error("Illegal format!");
    }
    let symbol = sym(format);
    let sign = "+";
    if (Decimal.isNeg(radian)) sign = "-", radian = Decimal.neg(radian);
    let parts = rad2hdms(radian)(/^H?M?S?$/.test(format));
    if (format === "H" || format === "d") return sign + Decimal.toFixed(parts[0], fixed) + symbol[0];
    else if (format === "M" || format === "m") return sign + Decimal.toFixed(parts[1], fixed) + symbol[1];
    else if (format === "S" || format === "s") return sign + Decimal.toFixed(parts[3], fixed) + symbol[2];
    else if (format === "HM" || format === "dm") {
        let hm = carry_out([Decimal.floor(parts[0]), Decimal.toDecimalPosition(Decimal.mod(parts[1], 60), fixed)]);
        return sign + Decimal.toFixed(hm[0], 0) + symbol[0] + Decimal.toFixed(hm[1], fixed) + symbol[1];
    } else if (format === "MS" || format === "ms") {
        let ms = carry_out([Decimal.floor(parts[1]), Decimal.toDecimalPosition(Decimal.mod(parts[2], 60), fixed)]);
        return sign + Decimal.toFixed(ms[0], 0) + symbol[1] + Decimal.toFixed(ms[1], fixed) + symbol[2];
    } else if (format === "HMS" || format === "dms") {
        let hms = carry_out([Decimal.floor(parts[0]), Decimal.floor(Decimal.mod(parts[1], 60)), Decimal.toDecimalPosition(Decimal.mod(parts[2], 60), fixed)]);
        return sign + Decimal.toFixed(hms[0], 0) + symbol[0] + Decimal.toFixed(hms[1], 0) + symbol[1] + Decimal.toFixed(hms[2], fixed) + symbol[2];
    } else throw new Error("Illegal format!");
});
// Angle -> Angle -> Angle
export const plus = std.uncurry(a => b => angle(Decimal.plus(angle(a), angle(b))));
export const minus = std.uncurry(a => b => angle(Decimal.minus(angle(a), angle(b))));
// Angle -> Decimal -> Angle
export const mult = std.uncurry(a => b => angle(Decimal.mult(angle(a), decimal(b))));
export const div = std.uncurry(a => b => angle(Decimal.div(angle(a), decimal(b))));
// Angle -> Angle
export const toZeroDoublePi = a => {
    a = Decimal.mod(angle(a), DoublePi);
    if (Decimal.isNeg(a)) return angle(Decimal.plus(a, DoublePi));
    return angle(a);
};
export const toPlusMinusPi = a => {
    a = Decimal.mod(angle(a), DoublePi);
    if (Decimal.lte(a, Decimal.neg(PI))) return angle(Decimal.plus(a, DoublePi));
    if (Decimal.gt(a, PI)) return angle(Decimal.minus(a, DoublePi));
    return angle(a);
};
// Angle-> String-> Number -> String
export const format = std.uncurry(a => fmt => dp => rad2str(a, fmt, dp));
// String -> Angle
export const show = a => rad2str(a, "dms", 2);

// Angle->Angle->Boolean
export const eq = std.uncurry(a => b => Decimal.eq(angle(a), angle(b)));
export const lt = std.uncurry(a => b => Decimal.lt(angle(a),angle(b)));
export const gt = std.uncurry(a => b => Decimal.gt(angle(a),angle(b)));
export const lte = std.uncurry(a => b => Decimal.lte(angle(a),angle(b)));
export const gte = std.uncurry(a => b => Decimal.gte(angle(a),angle(b)));
export const neq = std.uncurry(a => b => Decimal.neq(angle(a),angle(b)));
