import * as std from '../basic.js';
const DecimalJS = require('decimal.js');

const decjs = d => {
    if (typeof d instanceof DecimalJS)
        return d;
    throw new TypeError("Except a DecimalJS");
}

// Decimal = String | Number | DecimalJS
export const decimal = x => {
    if (typeof x === "string") {
        x = x.replace(/\s+/g, '');
        if (x[0] === '+') x = x.substr(1); // a bug of decimal.js lib.
        return new DecimalJS(x);
    }
    if (typeof x === "number" || x instanceof DecimalJS) return new DecimalJS(x);
    throw new TypeError("Except a decimal number!");
}
// Decimal->Decimal->Decimal
export const plus = std.uncurry(a => b => decimal(decimal(a).plus(decimal(b))));
export const minus = std.uncurry(a => b => decimal(decimal(a).minus(decimal(b))));
export const mult = std.uncurry(a => b => decimal(decimal(a).times(decimal(b))));
export const div = std.uncurry(a => b => decimal(decimal(a).div(decimal(b))));
export const mod = std.uncurry(a => b => decimal(decimal(a).mod(decimal(b))));
export const atan2 = std.uncurry(a => b => decimal(DecimalJS.atan2(decimal(a), decimal(b))));
// Decimal->Decimal
export const neg = a => decimal(decimal(a).negated());
export const sqr = a => mult(a, a);
export const cube = a => mult(a, sqr(a));
export const sqrt = a => decimal(DecimalJS.sqrt(decimal(a)));
export const sin = a => decimal(DecimalJS.sin(decimal(a)));
export const cos = a => decimal(DecimalJS.cos(decimal(a)));
export const tan = a => decimal(DecimalJS.tan(decimal(a)));
export const asin = a => decimal(DecimalJS.asin(decimal(a)));
export const acos = a => decimal(DecimalJS.acos(decimal(a)));
export const atan = a => decimal(DecimalJS.atan(decimal(a)));
export const hav = a => decimal(DecimalJS.sqrt(DecimalJS.sin(DecimalJS.mul(0.5, decimal(a)))));
export const ahav = a => decimal(DecimalJS.mul(2, DecimalJS.asin(DecimalJS.sqrt(decimal(a)))));
export const floor = a => decimal(DecimalJS.floor(decimal(a)));
// Decimal->Decimal->Boolean
export const eq = std.uncurry(a => b => std.bool(decimal(a).eq(decimal(b))));
export const lt = std.uncurry(a => b => std.bool(decimal(a).lt(decimal(b))));
export const gt = std.uncurry(a => b => std.bool(decimal(a).gt(decimal(b))));
export const lte = std.uncurry(a => b => std.bool(decimal(a).lte(decimal(b))));
export const gte = std.uncurry(a => b => std.bool(decimal(a).gte(decimal(b))));
export const neq = std.uncurry(a => b => std.bool(!decimal(a).eq(decimal(b))));
// Decimal-> Number
export const sgn = a => std.num(DecimalJS.sign(decimal(a)));
// Decimal->Boolean
export const isPos = a => gt(a, 0);
export const isNeg = a => lt(a, 0);
export const isZero = a => eq(a, 0);
// Decimal->Number->String
export const toFixed = std.uncurry(a => b => std.str(decimal(a).toFixed(std.num(b))));
// Decimal->Number->Decimal
export const toDecimalPosition = std.uncurry(a => b => decimal(toFixed(a, b)));
// Decimal->String
export const show = a => std.str(decimal(a).toString());
// [Decimal]->Decimal
export const sum = (...args) => {
    if (args.length === 1) {
        if (args[0].isArray(args[0])) {
            return args[0].reduce((sum, x) => plus(sum, decimal(x)), 0);
        }
    } else {
        return args.reduce((sum, x) => plus(sum, decimal(x)), 0);
    }
}
