// SPDX-License-Identifier: MIT OR Apache-2.0
//! BigInt values.
use super::*;

use rquickjs::BigInt;
const NAPI_BIGINT_EXPECTED: napi_status = 17;

fn decimal_limbs(decimal: &str) -> (bool, Vec<u32>) {
    let (negative, digits) = decimal
        .strip_prefix('-')
        .map_or((false, decimal), |digits| (true, digits));
    let mut limbs = Vec::new();
    for digit in digits.bytes().filter_map(|b| b.checked_sub(b'0')) {
        let mut carry = digit as u64;
        for limb in &mut limbs {
            let value = (*limb as u64) * 10 + carry;
            *limb = value as u32;
            carry = value >> 32;
        }
        if carry != 0 {
            limbs.push(carry as u32);
        }
    }
    while limbs.last() == Some(&0) {
        limbs.pop();
    }
    (negative && !limbs.is_empty(), limbs)
}

fn limbs_decimal(limbs: &[u32], negative: bool) -> String {
    let mut value = limbs.to_vec();
    while value.last() == Some(&0) {
        value.pop();
    }
    if value.is_empty() {
        return "0".to_string();
    }
    let mut chunks = Vec::new();
    while !value.is_empty() {
        let mut remainder = 0u64;
        for limb in value.iter_mut().rev() {
            let current = (remainder << 32) | *limb as u64;
            *limb = (current / 1_000_000_000) as u32;
            remainder = current % 1_000_000_000;
        }
        chunks.push(remainder as u32);
        while value.last() == Some(&0) {
            value.pop();
        }
    }
    let mut decimal = chunks.pop().unwrap_or_default().to_string();
    for chunk in chunks.iter().rev() {
        decimal.push_str(&format!("{chunk:09}"));
    }
    if negative {
        decimal.insert(0, '-');
    }
    decimal
}

fn bigint_decimal(env: napi_env, value: &V) -> Status<String> {
    let c = ctx(env);
    let raw_ctx = c.as_raw().as_ptr();
    let mut length = 0;
    let text = unsafe { qjs::JS_ToCStringLen2(raw_ctx, &mut length, value.as_raw(), false) };
    if text.is_null() {
        unsafe { (*env).exception = Some(extend(c.catch())) };
        return Err(NAPI_PENDING_EXCEPTION);
    }
    let decimal = unsafe {
        let length = match usize::try_from(length) {
            Ok(length) => length,
            Err(_) => {
                qjs::JS_FreeCString(raw_ctx, text);
                return Err(NAPI_GENERIC_FAILURE);
            }
        };
        let bytes = std::slice::from_raw_parts(text as *const u8, length);
        let decimal = String::from_utf8_lossy(bytes).into_owned();
        qjs::JS_FreeCString(raw_ctx, text);
        decimal
    };
    Ok(decimal)
}

fn bigint_from_decimal(env: napi_env, decimal: String) -> Status<V> {
    let constructor: Function = js(env, ctx(env).globals().get("BigInt"))?;
    let value: Value = js(env, constructor.call((decimal,)))?;
    Ok(unsafe { extend(value) })
}

fn word_limbs(words: &[u64]) -> Vec<u32> {
    let mut limbs = Vec::with_capacity(words.len() * 2);
    for word in words {
        limbs.push(*word as u32);
        limbs.push((word >> 32) as u32);
    }
    while limbs.last() == Some(&0) {
        limbs.pop();
    }
    limbs
}

fn low_u64(limbs: &[u32]) -> u64 {
    limbs.first().copied().unwrap_or_default() as u64
        | ((limbs.get(1).copied().unwrap_or_default() as u64) << 32)
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_bigint_int64(
    env: napi_env,
    value: i64,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let b = js(env, BigInt::from_i64(ctx(env), value))?;
        out(result, push(env, extend(b.into_value())))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_bigint_uint64(
    env: napi_env,
    value: u64,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        let b = js(env, BigInt::from_u64(ctx(env), value))?;
        out(result, push(env, extend(b.into_value())))
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_create_bigint_words(
    env: napi_env,
    sign_bit: c_int,
    word_count: usize,
    words: *const u64,
    result: *mut napi_value,
) -> napi_status {
    run_js(env, || {
        if (word_count > 0 && words.is_null()) || word_count > i32::MAX as usize {
            return Err(NAPI_INVALID_ARG);
        }
        let mut words_vec = Vec::with_capacity(word_count);
        for i in 0..word_count {
            words_vec.push(*words.add(i));
        }
        let limbs = word_limbs(&words_vec);
        let v = bigint_from_decimal(env, limbs_decimal(&limbs, sign_bit != 0))?;
        out(result, push(env, v))
    })
}

unsafe fn bigint_arg(value: napi_value) -> Status<&'static V> {
    let v = get(value)?;
    if v.is_big_int() {
        Ok(v)
    } else {
        Err(NAPI_BIGINT_EXPECTED)
    }
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_bigint_int64(
    env: napi_env,
    value: napi_value,
    result: *mut i64,
    lossless: *mut bool,
) -> napi_status {
    run(env, || {
        let decimal = bigint_decimal(env, bigint_arg(value)?)?;
        let (negative, limbs) = decimal_limbs(&decimal);
        let bits = low_u64(&limbs);
        let value = if negative {
            bits.wrapping_neg() as i64
        } else {
            bits as i64
        };
        out(lossless, decimal == value.to_string())?;
        out(result, value)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_bigint_uint64(
    env: napi_env,
    value: napi_value,
    result: *mut u64,
    lossless: *mut bool,
) -> napi_status {
    run(env, || {
        let decimal = bigint_decimal(env, bigint_arg(value)?)?;
        let (negative, limbs) = decimal_limbs(&decimal);
        let bits = low_u64(&limbs);
        let value = if negative { bits.wrapping_neg() } else { bits };
        out(lossless, decimal == value.to_string())?;
        out(result, value)
    })
}

#[no_mangle]
pub unsafe extern "C" fn napi_get_value_bigint_words(
    env: napi_env,
    value: napi_value,
    sign_bit: *mut c_int,
    word_count: *mut usize,
    words: *mut u64,
) -> napi_status {
    run(env, || {
        if word_count.is_null() {
            return Err(NAPI_INVALID_ARG);
        }
        let decimal = bigint_decimal(env, bigint_arg(value)?)?;
        let (negative, limbs) = decimal_limbs(&decimal);
        let needed = limbs.len().div_ceil(2);
        if words.is_null() {
            *word_count = needed;
            return Ok(());
        }
        let count = needed.min(*word_count);
        for i in 0..count {
            let low = limbs.get(i * 2).copied().unwrap_or_default() as u64;
            let high = limbs.get(i * 2 + 1).copied().unwrap_or_default() as u64;
            *words.add(i) = low | (high << 32);
        }
        if !sign_bit.is_null() {
            *sign_bit = negative as c_int;
        }
        *word_count = count;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::{decimal_limbs, limbs_decimal, low_u64, word_limbs};

    #[test]
    fn decimal_limb_conversion_round_trips_large_values() {
        for decimal in [
            "0",
            "1",
            "-1",
            "18446744073709551616",
            "-18446744073709551617",
            "340282366920938463463374607431768211455",
            "-12345678901234567890123456789012345678901234567890",
        ] {
            let (negative, limbs) = decimal_limbs(decimal);
            assert_eq!(limbs_decimal(&limbs, negative), decimal);
        }
    }

    #[test]
    fn word_conversion_preserves_little_endian_words() {
        let words = [0x0123_4567_89ab_cdef, 0xfedc_ba98_7654_3210];
        let limbs = word_limbs(&words);
        assert_eq!(low_u64(&limbs), words[0]);
        assert_eq!(
            limbs_decimal(&limbs, false),
            "338770000845734292516042252062085074415"
        );
    }
}
