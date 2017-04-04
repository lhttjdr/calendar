import * from '../basic.js';
import * as Decimal from './decimalimal.js';
const decimal=Decimal.decimal;

// Vector = [Decimal]
export const vector= v =>{
  if(!Array.isArray(v)) throw new TypeError("Except a vector!");
  return v.map(x=>decimal(x));
}
const dimension_check = (u, v) => {
    if (u.length === v.length) return true;
    throw new Error("Two vectors must have same dimension!");
}
// Vector->Number
export const dimension= u => u.length;
// Vector -> Vector ->Vector
export const plus=uncurry( u=> v=> dimension_check(u, v) && u.map((u,i)=>Decimal.plus(u,v[i])));
export const minus=uncurry( u=> v=> dimension_check(u, v) && u.map((u,i)=>Decimal.minus(u,v[i])));
// Vector->Vector
export const neg= u=> vector(u).map(x => Decimal.neg(x));
// Vector -> Decimal -> Vector
export const mult=uncurry( u=> v=> decimal(v) && u.map((u)=>Decimal.mult(u,v)));
export const div=uncurry( u=> v=> decimal(v) && u.map((u)=>Decimal.div(u,v)));
// Vector -> Vector -> Decimal
export const dot=uncurry( u=> v=> dimension_check(u,v) && u.reduce((sum, x, i) => Decimal.plus(sum,Decimal.mult(x,v[i])), 0));
// Vector->Decimal | Vector
export const cross=uncurry( u=> v=> {
  dimension_check(u,v);
  if(u.length===2) return Decimal.minus(Decimal.mult(u[0],v[1]),Decimal.mult(u[1],v[0]));
  else if(u.length===3) return vector([
      Decimal.minus(Decimal.mult(u[1],v[2]),Decimal.mult(u[2],v[1])),
      Decimal.minus(Decimal.mult(u[2],v[0]),Decimal.mult(u[0],v[2])),
      Decimal.minus(Decimal.mult(u[0],v[1]),Decimal.mult(u[1],v[0])),
  ]);
  throw new Error("Cross product is only defined in 3 dimensional space!");
});
// Vector->Decimal
export const norm= u => Decimal.sqrt(dot(u,u));
// Vector->Vector
export const normalize= u => {
  let n=norm(vector(u));
  return u.map(x=>Decimal.div(x,n));
}
// Vector -> Vector -> Boolean
export const equals=uncurry( u=> v=> vector(u).length === vector(v).length && u.every((x,i)=>Decimal.eq(x,v[i])));
export const eq = equals;
// Vector->Boolean
export const isZero= u=> vector(u).every(x=>Decimal.eq(x,0));
// Vector->String
export const toString= u=> "(" + vector(u).map(x => Decimal.toString(x)).join(",") + ")";
