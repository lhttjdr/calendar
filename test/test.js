const std=require('../lib/basic.js');
const Decimal = require('../lib/math/decimal.hp.js');
const Vector = require('../lib/math/vector.js');
const Angle = require('../lib/math/angle.js');
const Quaternion = require('../lib/math/quaternion.js');
const DualNumber =require('../lib/math/dual-number.js');
const DualQuaternion =require('../lib/math/dual-quaternion.js');
const Polynomial =require('../lib/math/polynomial.js');
{
    console.log("===============Test Decimal=================");
    // (*->Decimal) -> IO
    const show= func => std.compose(console.log, Decimal.show, func);
    // (*->Decimal) -> Decimal -> IO
    const assert_same = std.uncurry(func => ans => std.compose(console.log, Decimal.eq(ans), func));
    // (*->String) -> IO
    const log= func => std.compose(console.log, func);

    show(Decimal.mult)(587569,"23489679082349798798676876576E-12");
    assert_same(Decimal.mult)(30)(5,6);
    assert_same(Decimal.plus, 61.5)(5.5, "56");
    show(Decimal.plus)("23.4", "56");
    show(Decimal.plus)("23.4", 2);
    show(Decimal.plus)("2e3", 56e2);
    log(Decimal.sgn)("-35.3");
    show(Decimal.floor)("2452345.4535");
    show(Decimal.hav)("2313.23423");
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
  show(Vector.vector)(7,"8.45",9);
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
{
  console.log("================Test Quaternion===================");
  // (*->Quaternion) -> IO
  const show= func => std.compose(console.log, Quaternion.show, func);
  // (*->Quaternion) -> Quaternion -> IO
  const assert_same = std.uncurry(func => ans => std.compose(console.log, Quaternion.eq(ans), func));
  // (*->String) -> IO
  const log= func => std.compose(console.log, func);
  show(Quaternion.quaternion)("234",["34.34",234,23e-1]);
  let p=Quaternion.quaternion("1",["2",3,4e-1]), q=Quaternion.quaternion(-2,["2.45",23.4,23e2]);
  show(Quaternion.plus)(p,q);
  assert_same(Quaternion.plus)(Quaternion.quaternion(-1,[4.45,26.4,2300.4]))(p,q);
  show(Quaternion.grossman)(p,q);
  show(Quaternion.even)(p,q);
  show(Quaternion.inverse)(p);
}
{
  console.log("================Test Dual Number===================");
  // (*->Quaternion) -> IO
  const show= func => std.compose(console.log, DualNumber.show, func);
  // (*->Quaternion) -> Quaternion -> IO
  const assert_same = std.uncurry(func => ans => std.compose(console.log, DualNumber.eq(ans), func));
  // (*->String) -> IO
  const log= func => std.compose(console.log, func);
  show(DualNumber.dualnumber)(234,"2.23");
  let p=DualNumber.dualnumber(23,5) ,q =DualNumber.dualnumber(-3,"34.4");
  assert_same(DualNumber.plus)(DualNumber.dualnumber(20,39.4))(p,q);
  show(DualNumber.mult)(p,q);
  show(DualNumber.sqrt)(p);
}
{
  console.log("================Test Dual Quaternion===================");
  // (*->Quaternion) -> IO
  const show= func => std.compose(console.log, DualQuaternion.show, func);
  // (*->Quaternion) -> Quaternion -> IO
  const assert_same = std.uncurry(func => ans => std.compose(console.log, DualQuaternion.eq(ans), func));
  // (*->String) -> IO
  const log= func => std.compose(console.log, func);

  let p=Quaternion.quaternion("1",["2",3,4e-1]), q=Quaternion.quaternion(-2,["2.45",23.4,23e2]);
  show(DualQuaternion.dualquaternion)(p,q);
  let dq=DualQuaternion.dualquaternion(p,q), dp=dq=DualQuaternion.dualquaternion(q, p);
  //assert_same(DualNumber.plus)(DualNumber.dualnumber(20,39.4))(p,q);
  show(DualQuaternion.mult)(dp, dq);
  show(DualQuaternion.normalize)(dp);
}
{
  console.log("================Test Polynomial===================");
  const show= func => std.compose(console.log, Polynomial.show, func);
  const log= func => std.compose(console.log, func);

  let poly=Polynomial.polynomial(1,2,3,-4,5,-6,7);
  log(Polynomial.show)(poly);
  show(Polynomial.derivative)(poly);
  console.log(Decimal.show(Polynomial.value(poly,5)));
  console.log(Decimal.show(Polynomial.value(poly,1)));
  console.log(Decimal.show(Polynomial.valueOfLimitedItems(poly,2, 3)));
  console.log(Decimal.show(Polynomial.value(poly,5)));
}
