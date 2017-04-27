const std = require('../lib/basic.js');
const Decimal = require('../lib/math/decimal.hp.js');

console.log("===============Test Decimal=================");
// (*->Decimal) -> IO
const show = func => std.compose(console.log, Decimal.show, func);
// (*->Decimal) -> Decimal -> IO
const assert_same = std.uncurry(func => ans => std.compose(console.log, Decimal.eq(ans), func));
// (*->String) -> IO
const log = func => std.compose(console.log, func);

exports.test = () => {
    show(Decimal.mult)(587569, "23489679082349798798676876576E-12");
    assert_same(Decimal.mult)(30)(5, 6);
    assert_same(Decimal.plus, -50.5)(5.5, "-56");
    show(Decimal.plus)("23.4", "56");
    show(Decimal.plus)("23.4", 2);
    show(Decimal.plus)("2e3", 56e2);
    log(Decimal.sgn)("-35.3");
    show(Decimal.floor)("2452345.4535");
    show(Decimal.hav)("2313.23423");
    log(Decimal.toFixed)(45.346, 2);
}