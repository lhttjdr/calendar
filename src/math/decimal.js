import * from '../basic.js';

// Decimal = String | Number
export const decimal = x => {
    if (typeof x === "number") return x;
    if (typeof(x) === "string" && /^[+-]?(([0-9]*\.?[0-9]+|[0-9]+\.?[0-9]*)([eE][+-]?[0-9]+)?)$/.test(x.replace(/\s+/g, ''))) return parseFloat(x);
    throw new TypeError("Except a decimal number!");
};
// Decimal->Decimal->Decimal
export const plus = uncurry(a => b => decimal(decimal(a) + decimal(b)));
export const minus = uncurry(a => b => decimal(decimal(a) - decimal(b)));
export const mult = uncurry(a => b => decimal(decimal(a) * decimal(b)));
export const div = uncurry(a => b => decimal(decimal(a) / decimal(b)));
export const mod = uncurry(a => b => decimal(decimal(a) % decimal(b)));
export const atan2 = uncurry(a => b => decimal(Math.atan2(decimal(a), decimal(b))));
// Decimal->Decimal
export const neg = a => decimal(-decimal(a));
export const sqrt = a => decimal(Math.sqrt(decimal(a)));
export const sin = a => decimal(Math.sin(decimal(a)));
export const cos = a => decimal(Math.cos(decimal(a)));
export const tan = a => decimal(Math.tan(decimal(a)));
export const asin = a => decimal(Math.asin(decimal(a)));
export const acos = a => decimal(Math.acos(decimal(a)));
export const atan = a => decimal(Math.atan(decimal(a)));
export const hav = a=> decimal(Math.sqrt(Math.sin(0.5*decimal(a))));
export const ahav = a=> decimal(2* Math.asin(Math.sqrt(decimal(a))));
export const floor = a=> decimal(Math.floor(decimal(a)));
// Decimal->Decimal->Boolean
export const eq = uncurry(a => b => bool(decimal(a) === decimal(b)));
export const lt = uncurry(a => b => bool(decimal(a) < decimal(b)));
export const gt = uncurry(a => b => bool(decimal(a) > decimal(b)));
export const lte = uncurry(a => b => bool(decimal(a) <= decimal(b)));
export const gte = uncurry(a => b => bool(decimal(a) >= decimal(b)));
export const neq = uncurry(a => b => bool(decimal(a) !== decimal(b)));
// Decimal-> Number
export const sgn = a => num(Math.sign(decimal(a)));
// Decimal->Boolean
export const isPos = a => gt(a, 0);
export const isNeg = a => lt(a, 0);
export const isZero = a => eq(a, 0);
// Decimal->Number->String
export const toFixed = uncurry(a => b => str(decimal(a).toFixed(num(b))));
// Decimal->Number->Decimal
export const toDecimalPosition = uncurry(a => b => decimal(toFixed(a, b)));
// Decimal->String
export const toString = a => str(decimal(a).toString());
