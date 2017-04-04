const Decimal = require('./decimal.js');
const dec = Decimal.decimal;

// tools
const partial = arity => f => function _(...args) {
    return args.length < arity ?
        (...args_) => _(...args.concat(args_)) :
        f(args);
};
const uncurry = arity => f => partial(arity)(args => args.reduce((g, x) => g(x), f));

const PI = Decimal.acos(-1);
const DoublePi = Decimal.plus(PI, PI);
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

const match = s => {
    for (let p of patterns) {
        let groups = p[0].exec(s);
        if (groups) return p[1](groups);
    }
    throw new Error("Parse Error!");
}
const sec2rad = s => Decimal.div(s, SecondPerRadian);
const ha2rad = uncurry(2)(a => ha => ha ? Decimal.mult(a, 15) : a);
const zip_sec = arr_hms => Decimal.plus(Decimal.plus(Decimal.mult(dec(arr_hms[0]), 3600), Decimal.mult(dec(arr_hms[1]), 60)), dec(arr_hms[2]));
const sign = uncurry(2)(a => sgn => sgn ? a : Decimal.neg(a));

// String -> Angle
const parse = s => {
    let grps = match(s.replace(/\s+/g, ''));
    return ang(sign(ha2rad(sec2rad(zip_sec(grps[1])), grps[2]), grps[0]));
}

// Angle = Decimal | String
const ang = x => {
    if (typeof x === "string") {
        // parse
        return parse(x);
    }
    return dec(x);
}

// dms | HMS,  case sensitive
const sym = fmt => /^H?M?S?$/.test(fmt) ? "hms" : "\u00b0\u2032\u2033";
const rad2dms = rad => {
    let res = [];
    res[0] = Decimal.div(Decimal.mult(rad, 180), PI); // s
    res[1] = Decimal.mult(res[0], 60); // m
    res[2] = Decimal.mult(res[1], 60); // s
    return res;
}
const rad2HMS = rad => {
    let res = [];
    res[0] = Decimal.div(Decimal.mult(rad, 12), PI); // H
    res[1] = Decimal.mult(res[0], 60); // M
    res[2] = Decimal.mult(res[1], 60); // S
    return res;
}
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

//将弧度转为字串
// --fixed为小数保留位数
// --format=dms 格式示例: -23°59' 48.23"
// --format=d   格式示例: -23.59999°
// --format=hms 格式示例:  18h 29m 44.52s
const rad2str = function(radian, format, fixed) {
    format = format || "dms";
    fixed = fixed || 2;
    if (false == /^(H?M?S?)|(d?m?s?)$/.test(format)) {
        throw new Error("Illegal format!");
    }
    let symbol = sym(format);
    let sign = "+";
    if (Decimal.isNeg(radian)) sign = "-", radian = Decimal.neg(radian);
    let parts = /^d?m?s?$/.test(format) ? rad2dms(radian) : rad2HMS(radian);
    if (format === "H" || format === "d") return sign + Decimal.toFixed(parts[0], fixed) + symbol[0];
    else if (format === "M" || format === "m") return sign + Decimal.toFixed(parts[1], fixed) + symbol[1];
    else if (format === "S" || format === "s") return sign + Decimal.toFixed(parts[3], fixed) + symbol[2];
    else if (format === "HM" || format === "dm") {
        let hm = carry_out([Decimal.floor(parts[0]), Decimal.toDP(Decimal.mod(parts[1], 60), fixed)]);
        return sign + Decimal.toFixed(hm[0], 0) + symbol[0] + Decimal.toFixed(hm[1], fixed) + symbol[1];
    } else if (format === "MS" || format === "ms") {
        let ms = carry_out([Decimal.floor(parts[1]), Decimal.toDP(Decimal.mod(parts[2], 60), fixed)]);
        return sign + Decimal.toFixed(ms[0], 0) + symbol[1] + Decimal.toFixed(ms[1], fixed) + symbol[2];
    } else if (format === "HMS" || format === "dms") {
        let hms = carry_out([Decimal.floor(parts[0]), Decimal.floor(Decimal.mod(parts[1], 60)), Decimal.toDP(Decimal.mod(parts[2], 60), fixed)]);
        return sign + Decimal.toFixed(hms[0], 0) + symbol[0] + Decimal.toFixed(hms[1], 0) + symbol[1] + Decimal.toFixed(hms[2], fixed) + symbol[2];
    } else throw new Error("Illegal format!");
}

// Angle -> Angle -> Angle
const plus = uncurry(2)(a => b => ang(Decimal.plus(ang(a), ang(b))));
const minus = uncurry(2)(a => b => ang(Decimal.minus(ang(a), ang(b))));
// Angle -> Decimal -> Angle
const mult = uncurry(2)(a => b => ang(Decimal.mult(ang(a), dec(b))));
const div = uncurry(2)(a => b => ang(Decimal.div(ang(a), dec(b))));
// Angle -> Angle
const to_0_2pi = a => {
    a = Decimal.mod(ang(a), DoublePi);
    if (Decimal.isNeg(a)) return ang(Decimal.plus(a, DoublePi));
    return ang(a);
};
const to_pi_pi = a => {
    a = Decimal.mod(ang(a), DoublePi);
    if (Decimal.lte(a, Decimal.neg(PI))) return ang(Decimal.plus(a, DoublePi));
    if (Decimal.gt(a, PI)) return ang(Decimal.minus(a, DoublePi));
    return ang(a);
};

// Angle-> String-> Number -> String
const format = uncurry(3)(a => fmt => dp => rad2str(a, fmt, dp));
// String -> Angle
// const parse = s => parseAngle(s);

const toString = a => rad2str(a);

module.exports = {
    plus: plus,
    minus: minus,
    mult: mult,
    div: div,
    to_0_2pi: to_0_2pi,
    to_pi_pi: to_pi_pi,
    format: format,
    parse: parse,
    toString: toString
};
