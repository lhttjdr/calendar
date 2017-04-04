const std=require('../lib/basic.js');
const Decimal = require('../lib/math/decimal.hp.js');
const Vector = require('../lib/math/vector.js');
const Angle = require('../lib/math/angle.js');

{
    console.log("===============Test Decimal=================");
    // (*->Decimal) -> IO
    const show= func => std.compose(console.log, Decimal.show, func);
    // (*->Decimal) -> Decimal -> IO
    const assert_same = std.uncurry(func => ans => std.compose(console.log, Decimal.eq(ans), func));
    // (*->String) -> IO
    const log= func => std.compose(console.log, func);

    show(Decimal.mult)(587569,"23489679082349798798676876576");
    assert_same(Decimal.mult)(30)(5,6);
    assert_same(Decimal.plus, 61.5)(5.5, "56");
    show(Decimal.plus)("23.4", "56");
    show(Decimal.plus)("23.4", 2);
    show(Decimal.plus)("2e3", 56e2);
    log(Decimal.sgn)("-35.3");
    show(Decimal.floor)("2452345.4535");
    log(Decimal.toFixed)(45.346, 2);
}
{
  console.log("================Test Vector===================");
  // (*->Vector) -> IO
  const show= func => std.compose(console.log, Vector.show, func);
  // (*->Vector) -> Vector -> IO
  const assert_same = std.uncurry(func => ans => std.compose(console.log, Vector.equals(ans), func));
  // (*->String) -> IO
  const log= func => std.compose(console.log, func);

  show(Vector.plus)([4,"6",3e2], ["234","234","563"]);
  show(Vector.vector)([7,"8.45",9]);
  log(Vector.show)([7,"34.6",9]);
  assert_same(Vector.cross)([0,-1,0])([0,0,1], ["-1",0,0]);
}
{
   console.log("================Test Angle===================");
   // (*->Angle) -> IO
   const show= func => std.compose(console.log, Angle.show, func);
   // (*->Angle) -> Angle -> IO
   const assert_same = std.uncurry(func => ans => std.compose(console.log, Angle.eq(ans), func));
   // (*->String) -> IO
   const log= func => std.compose(console.log, func);

   show(Angle.plus)("-3133°21′22\"","+63°21′22\"");
   show(Angle.toZeroDoublePi)("+3133°21′22\"");
   show(Angle.toPlusMinusPi)("363°21′22\"");
   assert_same(Angle.plus)(Angle.DoublePi)("180°","12h");
   log(Angle.format)(3.1415926535,"dms",5);
   log(Angle.format)(3.1415926535,"dms",4);
   log(Angle.format)(3.1415926535,"ms",4);
   log(Angle.format)(3.1415926535,"H",4);
   log(Angle.format)(Angle.plus("234h23m45s","1h22m32s.5"),"HMS",3);
}
