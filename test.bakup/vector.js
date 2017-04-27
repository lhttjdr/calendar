const std = require('../lib/basic.js');
const Math =require('../lib/math');
const Vector = Math.Vector;

// (*->Vector) -> IO
const show = func => std.compose(console.log, Vector.show, func);
// (*->Vector) -> Vector -> IO
const assert_same = std.uncurry(func => ans => std.compose(console.log, Vector.equals(ans), func));
// (*->String) -> IO
const log = func => std.compose(console.log, func);

exports.test = () => {
    console.log("================Test Vector===================");
    show(Vector.plus)([4, "6", 3e2], ["234", "234", "563"]);
    show(Vector.vector)([7, "8.45", 9]);
    show(Vector.vector)(7, "8.45", 9);
    log(Vector.show)([7, "34.6", 9]);
    assert_same(Vector.cross)([0, -1, 0])([0, 0, 1], ["-1", 0, 0]);
}