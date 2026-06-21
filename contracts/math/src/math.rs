use crate::SCALE;

pub fn fixed_mul(a: u128, b: u128) -> u128 {
    let result = a.saturating_mul(b);
    result / SCALE
}

pub fn fixed_div(a: u128, b: u128) -> u128 {
    if b == 0 {
        panic!("division by zero");
    }
    a.saturating_mul(SCALE) / b
}

const LN_2: u128 = 6_931_472;

pub fn fixed_ln(x: u128) -> u128 {
    if x == 0 {
        return 0;
    }

    let mut n = 0i32;
    let mut reduced = x;

    while reduced >= 2 * SCALE {
        reduced /= 2;
        n += 1;
    }
    while reduced < SCALE {
        reduced = reduced.saturating_mul(2);
        n -= 1;
    }

    let y = reduced.saturating_sub(SCALE);
    let y_sq = fixed_mul(y, y);
    let y_cu = fixed_mul(y_sq, y);
    let y_qu = fixed_mul(y_cu, y);

    let mut result = y;
    result = result.saturating_sub(fixed_div(y_sq, 2 * SCALE));
    result = result.saturating_add(fixed_div(y_cu, 3 * SCALE));
    result = result.saturating_sub(fixed_div(y_qu, 4 * SCALE));

    let y_quint = fixed_mul(y_qu, y);
    result = result.saturating_add(fixed_div(y_quint, 5 * SCALE));
    let y_sixth = fixed_mul(y_quint, y);
    result = result.saturating_sub(fixed_div(y_sixth, 6 * SCALE));
    let y_seventh = fixed_mul(y_sixth, y);
    result = result.saturating_add(fixed_div(y_seventh, 7 * SCALE));

    if n >= 0 {
        result
            .saturating_add(fixed_mul(LN_2, (n as u128).saturating_mul(SCALE)))
    } else {
        result.saturating_sub(fixed_mul(
            LN_2,
            ((-n) as u128).saturating_mul(SCALE),
        ))
    }
}

pub fn fixed_exp(x: u128) -> u128 {
    if x == 0 {
        return SCALE;
    }

    if x >= SCALE {
        let half = fixed_exp(x / 2);
        return fixed_mul(half, half);
    }

    let mut term = SCALE;
    let mut sum = SCALE;

    for n in 1..=40 {
        term = fixed_mul(term, x);
        term = term / n;
        sum = sum.saturating_add(term);
        if term == 0 {
            break;
        }
    }

    sum
}

pub fn fixed_pow(base: u128, exp: u128) -> u128 {
    if exp == 0 {
        return SCALE;
    }
    if base == 0 {
        return 0;
    }
    if base == SCALE {
        return SCALE;
    }

    let ln_base = fixed_ln(base);
    let exponent = fixed_mul(exp, ln_base);
    fixed_exp(exponent)
}

pub fn integral_logarithmic(a: u128, b: u128, s: u128) -> u128 {
    let s_plus_1 = s.saturating_add(SCALE);
    let ln_val = fixed_ln(s_plus_1);
    let term1 = fixed_mul(s_plus_1, ln_val);
    let term2 = term1.saturating_sub(s);
    let a_term = fixed_mul(a, term2);
    let b_term = fixed_mul(b, s);
    a_term.saturating_add(b_term)
}

pub fn sigmoid(x: u128, k: u128, mid: u128, max_val: u128) -> u128 {
    let diff = if x >= mid {
        x.saturating_sub(mid)
    } else {
        mid.saturating_sub(x)
    };
    let k_diff = fixed_mul(k, diff);
    let e_pow = fixed_exp(k_diff);
    if x >= mid {
        let numerator = fixed_mul(max_val, e_pow);
        fixed_div(numerator, SCALE.saturating_add(e_pow))
    } else {
        fixed_div(max_val, SCALE.saturating_add(e_pow))
    }
}
