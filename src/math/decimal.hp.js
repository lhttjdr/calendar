const DecimalJS = require('decimal.js');

// tools
const partial = arity => f => function _(...args) {
    return args.length < arity ?
        (...args_) => _(...args.concat(args_)) :
        f(args);
};
const uncurry = arity => f => partial(arity)(args => args.reduce((g, x) => g(x), f));

const num = x => {
    if (typeof x === "number") return x;
    throw new TypeError("Except a number!");
};
const str = s => {
    if (typeof s === "string") return s;
    throw new TypeError("Except a string!");
}
const bool = b => {
    if (typeof b === "boolean") return b;
    throw new TypeError("Except a boolean!");
}
const decjs = d => {
    if (typeof d instanceof DecimalJS) return d;
    throw new TypeError("Except a DecimalJS");
}

// Decimal = String | Number | DecimalJS
const dec = x => {
    if (typeof x === "number" || typeof(x) === "string" || x instanceof DecimalJS) return new DecimalJS(x);
    throw new TypeError("Except a decimal number!");
}
// Decimal->Decimal->Decimal
const plus = uncurry(2)(a => b => dec(dec(a).plus(dec(b))));
const minus = uncurry(2)(a => b => dec(dec(a).minus(dec(b))));
const mult = uncurry(2)(a => b => dec(dec(a).times(dec(b))));
const div = uncurry(2)(a => b => dec(dec(a).div(dec(b))));
const mod = uncurry(2)(a => b => dec(dec(a).mod(dec(b))));
const atan2 = uncurry(2)(a => b => dec(DecimalJS.atan2(dec(a), dec(b))));
// Decimal->Decimal
const neg = uncurry(1)(a => dec(dec(a).negated()));
const sqrt = uncurry(1)(a => dec(DecimalJS.sqrt(dec(a))));
const sin = uncurry(1)(a => dec(DecimalJS.sin(dec(a))));
const cos = uncurry(1)(a => dec(DecimalJS.cos(dec(a))));
const tan = uncurry(1)(a => dec(DecimalJS.tan(dec(a))));
const asin = uncurry(1)(a => dec(DecimalJS.asin(dec(a))));
const acos = uncurry(1)(a => dec(DecimalJS.acos(dec(a))));
const atan = uncurry(1)(a => dec(DecimalJS.atan(dec(a))));
const floor = a=> dec(DecimalJS.floor(dec(a)));
// Decimal->Decimal->Boolean
const eq = uncurry(2)(a => b => bool(dec(a).eq(dec(b))));
const lt = uncurry(2)(a => b => bool(dec(a).lt(dec(b))));
const gt = uncurry(2)(a => b => bool(dec(a).gt(dec(b))));
const lte = uncurry(2)(a => b => bool(dec(a).lte(dec(b))));
const gte = uncurry(2)(a => b => bool(dec(a).gte(dec(b))));
const neq = uncurry(2)(a => b => bool(!dec(a).eq(dec(b))));
// Decimal-> Number
const sgn = uncurry(1)(a => num(DecimalJS.sign(dec(a))));
// Decimal->Boolean
const isPos = uncurry(1)(a => gt(a, 0));
const isNeg = uncurry(1)(a => lt(a, 0));
const isZero = uncurry(1)(a => eq(a, 0));
// Decimal->Number->String
const toFixed = uncurry(2)(a => b => str(dec(a).toFixed(num(b))));
// Decimal->Number->Decimal
const toDP = uncurry(2)(a => b => dec(toFixed(a, b)));
// Decimal->String
const toString = uncurry(1)(a => str(dec(a).toString()));

module.exports = {
    decimal:dec,
    plus: plus,
    minus: minus,
    mult: mult,
    div: div,
    mod: mod,
    atan2: atan2,
    neg: neg,
    sqrt: sqrt,
    sin: sin,
    cos: cos,
    tan: tan,
    asin: asin,
    acos: acos,
    atan: atan,
    floor:floor,
    eq: eq,
    lt: lt,
    gt: gt,
    lte: lte,
    gte: gte,
    neq: neq,
    sgn: sgn,
    isPos: isPos,
    isNeg: isNeg,
    isZero: isZero,
    toFixed: toFixed,
    toDP: toDP,
    toString: toString
};
