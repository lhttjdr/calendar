import * as std from '../../basic.js';
import * as Decimal from '../../math/decimal.hp.js';
import * as Expression from '../../math/expression.js';
import * as Angle from '../../math/angle.js';

const decimal = Decimal.decimal;
const expression = Expression.expression;
const evaluate = Expression.evaluate;
const angle = Angle.angle;

// Sæmundsson formula
// It is consistent with Bennett’s to within 0.1′
export const apparent = (h, P, T) => Decimal.div(evaluate(expression("f* 1.02 / tan( h + 10.3/ (h + 5.11))"), {
    f: evaluate(expression("(P/101)*(283/(273+T))"), {
        P: P || 101.0, // pressure, default 101.0 kPa
        T: T || 10 // temperature, default 10 °C
    }),
    h: Decimal.div(angle(h), Angle.DegreePerRadian) // real height, Radian to degree 
}), Angle.MinutePerRadian); // arc of minutes to Radian