use crate::{
    fixed_div, fixed_exp, fixed_ln, fixed_mul, fixed_pow, integral_logarithmic, sigmoid, SCALE,
};

#[test]
fn test_fixed_mul_basic() {
    let result = fixed_mul(SCALE, SCALE);
    assert_eq!(result, SCALE);
    let half = SCALE / 2;
    let quarter = fixed_mul(half, half);
    let expected = SCALE / 4;
    let diff = if quarter > expected {
        quarter - expected
    } else {
        expected - quarter
    };
    assert!(diff <= 1);
}

#[test]
fn test_fixed_mul_zero() {
    assert_eq!(fixed_mul(0, SCALE), 0);
    assert_eq!(fixed_mul(SCALE, 0), 0);
    assert_eq!(fixed_mul(0, 0), 0);
}

#[test]
fn test_fixed_mul_saturating_overflow() {
    let result = fixed_mul(u128::MAX, SCALE);
    assert_eq!(result, u128::MAX / SCALE);
}

#[test]
fn test_fixed_div_basic() {
    assert_eq!(fixed_div(SCALE, SCALE), SCALE);
    let half = SCALE / 2;
    let result = fixed_div(SCALE, 2 * SCALE);
    assert_eq!(result, half);
    let result = fixed_div(half, SCALE);
    assert_eq!(result, SCALE / 2);
}

#[test]
fn test_fixed_div_scaled() {
    let a = 5 * SCALE;
    let b = 2 * SCALE;
    let result = fixed_div(a, b);
    assert_eq!(result, 2 * SCALE + SCALE / 2);
}

#[test]
#[should_panic(expected = "division by zero")]
fn test_fixed_div_by_zero() {
    fixed_div(SCALE, 0);
}

#[test]
fn test_fixed_pow_base_zero() {
    assert_eq!(fixed_pow(0, SCALE), 0);
}

#[test]
fn test_fixed_pow_exp_zero() {
    assert_eq!(fixed_pow(SCALE, 0), SCALE);
    assert_eq!(fixed_pow(0, 0), SCALE);
}

#[test]
fn test_fixed_pow_base_scale() {
    assert_eq!(fixed_pow(SCALE, SCALE), SCALE);
    assert_eq!(fixed_pow(SCALE, 2 * SCALE), SCALE);
}

#[test]
fn test_fixed_pow_fractional_exponent() {
    let base = 4 * SCALE;
    let exp = SCALE / 2;
    let result = fixed_pow(base, exp);
    let expected = 2 * SCALE;
    let diff = if result > expected {
        result - expected
    } else {
        expected - result
    };
    assert!(diff <= 100);
}

#[test]
fn test_fixed_ln_one() {
    assert_eq!(fixed_ln(SCALE), 0);
}

#[test]
fn test_fixed_ln_e() {
    let e_approx = 27182818;
    let result = fixed_ln(e_approx);
    let diff = if result > SCALE {
        result - SCALE
    } else {
        SCALE - result
    };
    assert!(diff <= 1000);
}

#[test]
fn test_fixed_ln_monotonic() {
    let ln_small = fixed_ln(SCALE / 2);
    let ln_mid = fixed_ln(SCALE);
    let ln_large = fixed_ln(2 * SCALE);
    assert!(ln_small <= ln_mid);
    assert!(ln_mid <= ln_large);
}

#[test]
fn test_fixed_exp_zero() {
    assert_eq!(fixed_exp(0), SCALE);
}

#[test]
fn test_fixed_exp_one() {
    let result = fixed_exp(SCALE);
    let expected = 27182809;
    let diff = if result > expected {
        result - expected
    } else {
        expected - result
    };
    assert!(diff <= 100);
}

#[test]
fn test_sigmoid_asymptotes() {
    let max_val = 10 * SCALE;
    let k = SCALE / 10;
    let mid = 50 * SCALE;
    let at_min = sigmoid(0, k, mid, max_val);
    assert!(at_min < SCALE);
    let at_max = sigmoid(100 * SCALE, k, mid, max_val);
    assert!(at_max >= max_val.saturating_sub(700000));
}

#[test]
fn test_sigmoid_midpoint() {
    let max_val = 10 * SCALE;
    let k = SCALE / 10;
    let mid = 50 * SCALE;
    let at_mid = sigmoid(mid, k, mid, max_val);
    let expected_half = max_val / 2;
    let diff = if at_mid > expected_half {
        at_mid - expected_half
    } else {
        expected_half - at_mid
    };
    assert!(diff <= 100);
}

#[test]
fn test_integral_logarithmic_zero() {
    let result = integral_logarithmic(SCALE, SCALE, 0);
    assert_eq!(result, 0);
}

#[test]
fn test_integral_logarithmic_positive() {
    let a = SCALE;
    let b = SCALE / 2;
    let s = 10 * SCALE;
    let result = integral_logarithmic(a, b, s);
    assert!(result > 0);
    let s_double = 20 * SCALE;
    let result_double = integral_logarithmic(a, b, s_double);
    assert!(result_double > result);
}

#[test]
fn test_fixed_pow_monotonic() {
    let base = SCALE + SCALE / 2;
    let small = fixed_pow(base, SCALE / 2);
    let large = fixed_pow(base, SCALE);
    assert!(small <= large);
}
