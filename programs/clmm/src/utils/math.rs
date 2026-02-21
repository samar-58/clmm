use crate::errors::ClmmError;
use anchor_lang::prelude::*;

const Q96: u128 = 1 << 96;

pub const TICK_SPACING: i32 = 10;

pub const MIN_TICK: i32 = -443636;
pub const MAX_TICK: i32 = -MIN_TICK;

pub const MIN_SQRT_PRICE_X64: u128 = 4295048016;
pub const MAX_SQRT_PRICE_X64: u128 = 79226673521066979257578248091;

pub const MIN_SQRT_PRICE_X96: u128 = MIN_SQRT_PRICE_X64 << 32;
pub const MAX_SQRT_PRICE_X96: u128 = MAX_SQRT_PRICE_X64 << 32;

const BIT_PRECISION: u32 = 16;

pub fn integer_sqrt(value: u128) -> u64 {
    if value == 0 {
        return 0;
    }
    let mut x = value;
    let mut y = (value + 1) / 2;

    while y < x {
        x = y;
        y = (x + value / x) / 2;
    }
    x as u64
}

pub fn price_to_sqrt_price_x96(price: u64) -> Result<u128> {
    if price == 0 {
        return Err(ClmmError::ZeroAmount.into());
    }

    let price_scaled = price as u128;

    let mut x = price_scaled;
    let mut y = (price_scaled + 1) / 2;

    while y < x {
        x = y;
        y = (x + price_scaled / x) / 2;
    }

    let sqrt_price_x96 = x.checked_mul(Q96).ok_or(ClmmError::ArithmeticOverflow)?;

    Ok(sqrt_price_x96)
}

pub fn tick_to_sqrt_price_x96(tick: i32) -> Result<u128> {
    let abs_tick = tick.abs() as u32;
    require!(abs_tick <= MAX_TICK as u32, ClmmError::TickUpperOverflow);

    let mut ratio: u128 = if abs_tick & 0x1 != 0 {
        18445821805675392311u128
    } else {
        1u128 << 64
    };

    if abs_tick & 0x2 != 0 {
        ratio = (ratio * 18444899583751176498u128) >> 64;
    }
    if abs_tick & 0x4 != 0 {
        ratio = (ratio * 18443055278223354162u128) >> 64;
    }
    if abs_tick & 0x8 != 0 {
        ratio = (ratio * 18439367220385604838u128) >> 64;
    }
    if abs_tick & 0x10 != 0 {
        ratio = (ratio * 18431993317065449817u128) >> 64;
    }
    if abs_tick & 0x20 != 0 {
        ratio = (ratio * 18417254355718160513u128) >> 64;
    }
    if abs_tick & 0x40 != 0 {
        ratio = (ratio * 18387811781193591352u128) >> 64;
    }
    if abs_tick & 0x80 != 0 {
        ratio = (ratio * 18329067761203520168u128) >> 64;
    }
    if abs_tick & 0x100 != 0 {
        ratio = (ratio * 18212142134806087854u128) >> 64;
    }
    if abs_tick & 0x200 != 0 {
        ratio = (ratio * 17980523815641551639u128) >> 64;
    }
    if abs_tick & 0x400 != 0 {
        ratio = (ratio * 17526086738831147013u128) >> 64;
    }
    if abs_tick & 0x800 != 0 {
        ratio = (ratio * 16651378430235024244u128) >> 64;
    }
    if abs_tick & 0x1000 != 0 {
        ratio = (ratio * 15030750278693429944u128) >> 64;
    }
    if abs_tick & 0x2000 != 0 {
        ratio = (ratio * 12247334978882834399u128) >> 64;
    }
    if abs_tick & 0x4000 != 0 {
        ratio = (ratio * 8131365268884726200u128) >> 64;
    }
    if abs_tick & 0x8000 != 0 {
        ratio = (ratio * 3584323654723342297u128) >> 64;
    }
    if abs_tick & 0x10000 != 0 {
        ratio = (ratio * 696457651847595233u128) >> 64;
    }
    if abs_tick & 0x20000 != 0 {
        ratio = (ratio * 26294789957452057u128) >> 64;
    }
    if abs_tick & 0x40000 != 0 {
        ratio = (ratio * 37481735321082u128) >> 64;
    }

    if tick > 0 {
        ratio = u128::MAX / ratio;
    }

    Ok(ratio << 32)
}

pub fn sqrt_price_x96_to_tick(sqrt_price_x96: u128) -> Result<i32> {
    require!(
        sqrt_price_x96 >= MIN_SQRT_PRICE_X96 && sqrt_price_x96 < MAX_SQRT_PRICE_X96,
        ClmmError::SqrtPriceX96
    );

    let sqrt_price_x64 = sqrt_price_x96 >> 32;

    let msb: u32 = 128 - sqrt_price_x64.leading_zeros() - 1;
    let log2p_integer_x64: i128 = ((msb as i128) - 64) << 64;

    let mut r: u128 = if msb >= 64 {
        sqrt_price_x64 >> (msb - 63)
    } else {
        sqrt_price_x64 << (63 - msb)
    };

    let mut log2p_fraction_x64: i128 = 0;
    let mut bit: i128 = 1i128 << 63;

    for _ in 0..BIT_PRECISION {
        r = (r * r) >> 63;
        let is_r_more_than_two = (r >> 64) as u32;
        r >>= is_r_more_than_two;
        log2p_fraction_x64 |= bit * is_r_more_than_two as i128;
        bit >>= 1;
    }

    let log2p_x64 = log2p_integer_x64 + log2p_fraction_x64;

    let log_base: i128 = 1330580271462080i128;
    let tick_approx = (log2p_x64 / log_base) as i32;

    let tick_low = tick_approx - 1;
    let tick_high = tick_approx + 1;

    Ok(if let Ok(price_high) = tick_to_sqrt_price_x96(tick_high) {
        if price_high <= sqrt_price_x96 {
            tick_high
        } else if let Ok(price_approx) = tick_to_sqrt_price_x96(tick_approx) {
            if price_approx <= sqrt_price_x96 {
                tick_approx
            } else {
                tick_low
            }
        } else {
            tick_low
        }
    } else {
        tick_approx
    })
}

pub fn get_amounts_for_liquidity(
    sqrt_price_x96_current: u128,
    sqrt_price_x96_lower: u128,
    sqrt_price_x96_upper: u128,
    liquidity: u128,
) -> Result<(u64, u64)> {
    let amount_a: u64;
    let amount_b: u64;

    if sqrt_price_x96_current <= sqrt_price_x96_lower {
        // token A only: L * (sqrt_upper - sqrt_lower) / sqrt_lower * Q96 / sqrt_upper
        let delta = sqrt_price_x96_upper
            .checked_sub(sqrt_price_x96_lower)
            .ok_or(ClmmError::ArithmeticOverflow)?;
        // L * delta fits u128 for reasonable values; then divide by sqrt_lower first
        let step1 = liquidity
            .checked_mul(delta)
            .ok_or(ClmmError::ArithmeticOverflow)?
            / sqrt_price_x96_lower;
        // multiply by Q96, then divide by sqrt_upper
        amount_a = (step1
            .checked_mul(Q96)
            .ok_or(ClmmError::ArithmeticOverflow)?
            / sqrt_price_x96_upper)
        .try_into()
        .map_err(|_| ClmmError::ArithmeticOverflow)?;
        amount_b = 0;
    } else if sqrt_price_x96_current >= sqrt_price_x96_upper {
        // token B only: L * (sqrt_upper - sqrt_lower) / Q96
        amount_a = 0;
        amount_b = (liquidity
            .checked_mul(
                sqrt_price_x96_upper
                    .checked_sub(sqrt_price_x96_lower)
                    .ok_or(ClmmError::ArithmeticOverflow)?,
            )
            .ok_or(ClmmError::ArithmeticOverflow)?
            .checked_div(Q96)
            .ok_or(ClmmError::ArithmeticOverflow)?)
        .try_into()
        .map_err(|_| ClmmError::ArithmeticOverflow)?;
    } else {
        // both tokens
        // token A: L * (sqrt_upper - sqrt_current) / sqrt_current * Q96 / sqrt_upper
        let delta = sqrt_price_x96_upper
            .checked_sub(sqrt_price_x96_current)
            .ok_or(ClmmError::ArithmeticOverflow)?;
        let step1 = liquidity
            .checked_mul(delta)
            .ok_or(ClmmError::ArithmeticOverflow)?
            / sqrt_price_x96_current;
        amount_a = (step1
            .checked_mul(Q96)
            .ok_or(ClmmError::ArithmeticOverflow)?
            / sqrt_price_x96_upper)
        .try_into()
        .map_err(|_| ClmmError::ArithmeticOverflow)?;
        // amount_b = L * (current - lower) / Q96
        amount_b = (liquidity
            .checked_mul(
                sqrt_price_x96_current
                    .checked_sub(sqrt_price_x96_lower)
                    .ok_or(ClmmError::ArithmeticOverflow)?,
            )
            .ok_or(ClmmError::ArithmeticOverflow)?
            .checked_div(Q96)
            .ok_or(ClmmError::ArithmeticOverflow)?)
        .try_into()
        .map_err(|_| ClmmError::ArithmeticOverflow)?;
    }
    Ok((amount_a, amount_b))
}
pub fn compute_swap_step(
    sqrt_price_current_x96: u128,
    sqrt_price_target_x96: u128,
    liquidity: u128,
    amount_remaining: u128,
    a_to_b: bool,
) -> Result<(u128, u128, u128)> {
    require!(liquidity > 0, ClmmError::InsufficientLiquidity);

    if a_to_b {

        // a_to_b (selling token_a, price goes DOWN)
        // formula:
        //   required_in = L * (1/√P_target - 1/√P_current)
        //
        // in Q96 fixed-point, 1/√P = Q96 / √P_x96, so:
        //   required_in = L * Q96 / √P_target - L * Q96 / √P_current
        //
        // we compute each term separately to avoid overflow.

        let l_q96_div_target = liquidity
            .checked_mul(Q96)
            .ok_or(ClmmError::ArithmeticOverflow)?
            .checked_div(sqrt_price_target_x96)
            .ok_or(ClmmError::ArithmeticOverflow)?;

        let l_q96_div_current = liquidity
            .checked_mul(Q96)
            .ok_or(ClmmError::ArithmeticOverflow)?
            .checked_div(sqrt_price_current_x96)
            .ok_or(ClmmError::ArithmeticOverflow)?;

        let required_in = l_q96_div_target
            .checked_sub(l_q96_div_current)
            .ok_or(ClmmError::ArithmeticOverflow)?;

        // can we reach the target tick, or do we stop partway?
        let (next_sqrt_price_x96, amount_in) = if amount_remaining >= required_in {
            // we have enough input to fully cross to the target tick
            (sqrt_price_target_x96, required_in)
        } else {
            // not enough input — price stops between current and target
            //
            // formula:
            //   1/√P_new = 1/√P_current + amount_in / (L * Q96)
            //   √P_new = L * Q96 / (L * Q96 / √P_current + amount_in)
            //
            // we already computed L*Q96/√P_current above, so reuse it.

            let denom = l_q96_div_current
                .checked_add(amount_remaining)
                .ok_or(ClmmError::ArithmeticOverflow)?;

            let next_price = liquidity
                .checked_mul(Q96)
                .ok_or(ClmmError::ArithmeticOverflow)?
                .checked_div(denom)
                .ok_or(ClmmError::ArithmeticOverflow)?;

            (next_price, amount_remaining)
        };

        //  how much token_b the user receives
        //   amount_out = L * (√P_current - √P_next) / Q96
        //
        // this is safe because (√P_current - √P_next) < Q96 and L is small
        let out_diff = sqrt_price_current_x96
            .checked_sub(next_sqrt_price_x96)
            .ok_or(ClmmError::ArithmeticOverflow)?;

        let amount_out = liquidity
            .checked_mul(out_diff)
            .ok_or(ClmmError::ArithmeticOverflow)?
            .checked_div(Q96)
            .ok_or(ClmmError::ArithmeticOverflow)?;

        Ok((next_sqrt_price_x96, amount_in, amount_out))
    } else {
        // CASE: b_to_a (selling token_b, price goes UP)
        //
        // formula:
        //   required_in = L * (√P_target - √P_current) / Q96
        //
        // this is safe: L * price_diff can overflow, but we divide by Q96
        // immediately, so we split it: (L * price_diff) / Q96

        let price_diff = sqrt_price_target_x96
            .checked_sub(sqrt_price_current_x96)
            .ok_or(ClmmError::ArithmeticOverflow)?;

        let required_in = liquidity
            .checked_mul(price_diff)
            .ok_or(ClmmError::ArithmeticOverflow)?
            .checked_div(Q96)
            .ok_or(ClmmError::ArithmeticOverflow)?;

        // full cross or partial step?
        let (next_sqrt_price_x96, amount_in) = if amount_remaining >= required_in {
            // enough input to reach the target tick
            (sqrt_price_target_x96, required_in)
        } else {
            // partial step — price stops between current and target
            //
            //   next_price = √P_current + (amount_in * Q96 / L)

            let price_delta = amount_remaining
                .checked_mul(Q96)
                .ok_or(ClmmError::ArithmeticOverflow)?
                .checked_div(liquidity)
                .ok_or(ClmmError::ArithmeticOverflow)?;

            let next_price = sqrt_price_current_x96
                .checked_add(price_delta)
                .ok_or(ClmmError::ArithmeticOverflow)?;

            (next_price, amount_remaining)
        };

        // calculate output: how much token_a the user receives
        //   amount_out = L * (1/√P_current - 1/√P_next)
        //             = L * Q96 / √P_current - L * Q96 / √P_next
        //
        // same divide-first trick as the a_to_b required_in calculation.

        let l_q96_div_current = liquidity
            .checked_mul(Q96)
            .ok_or(ClmmError::ArithmeticOverflow)?
            .checked_div(sqrt_price_current_x96)
            .ok_or(ClmmError::ArithmeticOverflow)?;

        let l_q96_div_next = liquidity
            .checked_mul(Q96)
            .ok_or(ClmmError::ArithmeticOverflow)?
            .checked_div(next_sqrt_price_x96)
            .ok_or(ClmmError::ArithmeticOverflow)?;

        let amount_out = l_q96_div_current
            .checked_sub(l_q96_div_next)
            .ok_or(ClmmError::ArithmeticOverflow)?;

        Ok((next_sqrt_price_x96, amount_in, amount_out))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_to_sqrt_price_x96_perfect_square() {
        let price = 4;
        let result = price_to_sqrt_price_x96(price).unwrap();
        let expected = 2u128 * Q96;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_price_to_sqrt_price_x96_non_perfect_square() {
        let price = 10;
        let result = price_to_sqrt_price_x96(price).unwrap();
        let expected = 3u128 * Q96;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_price_to_sqrt_price_x96_zero_price() {
        let price = 0;
        let result = price_to_sqrt_price_x96(price);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ClmmError::ZeroAmount.into());
    }

    #[test]
    fn test_price_to_sqrt_price_x96_large_price() {
        let price = 1_000_000;
        let result = price_to_sqrt_price_x96(price).unwrap();
        let expected = 1000u128 * Q96;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tick_to_sqrt_price_at_tick_zero() {
        let tick = 0;
        let result = tick_to_sqrt_price_x96(tick).unwrap();
        let expected = 1u128 << 96;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tick_to_sqrt_price_positive_tick() {
        let tick = 100;
        let result = tick_to_sqrt_price_x96(tick).unwrap();
        let base = 1u128 << 96;
        assert!(result > base);
    }

    #[test]
    fn test_tick_to_sqrt_price_negative_tick() {
        let tick = -100;
        let result = tick_to_sqrt_price_x96(tick).unwrap();
        let base = 1u128 << 96;
        assert!(result < base);
    }

    #[test]
    fn test_sqrt_price_to_tick_at_price_one() {
        let sqrt_price = 1u128 << 96;
        let result = sqrt_price_x96_to_tick(sqrt_price).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_roundtrip_tick_to_price_to_tick() {
        let original_tick = 1000;
        let sqrt_price = tick_to_sqrt_price_x96(original_tick).unwrap();
        let recovered_tick = sqrt_price_x96_to_tick(sqrt_price).unwrap();
        assert!((recovered_tick - original_tick).abs() <= 1);
    }

    #[test]
    fn test_roundtrip_negative_tick() {
        let original_tick = -5000;
        let sqrt_price = tick_to_sqrt_price_x96(original_tick).unwrap();
        let recovered_tick = sqrt_price_x96_to_tick(sqrt_price).unwrap();
        assert!((recovered_tick - original_tick).abs() <= 1);
    }

    #[test]
    fn test_tick_overflow() {
        let tick = MAX_TICK + 1;
        let result = tick_to_sqrt_price_x96(tick);
        assert!(result.is_err());
    }

    #[test]
    fn test_sqrt_price_below_min() {
        let sqrt_price = MIN_SQRT_PRICE_X96 - 1;
        let result = sqrt_price_x96_to_tick(sqrt_price);
        assert!(result.is_err());
    }

    // compute_swap_step tests

    #[test]
    fn test_swap_a_to_b_full_cross() {
        // price at tick 0 (1.0), target at tick -10 (slightly lower price)
        // using a small tick distance to keep numbers manageable
        let sqrt_current = tick_to_sqrt_price_x96(0).unwrap();
        let sqrt_target = tick_to_sqrt_price_x96(-10).unwrap();

        let liquidity: u128 = 1_000_000_000;

        // large enough amount to fully cross to the target
        let amount_remaining: u128 = 1_000_000_000;

        let (next_price, amount_in, amount_out) =
            compute_swap_step(sqrt_current, sqrt_target, liquidity, amount_remaining, true)
                .unwrap();

        // price should land exactly at target since we had more than enough input
        assert_eq!(next_price, sqrt_target);
        // should have consumed some input and produced some output
        assert!(amount_in > 0);
        assert!(amount_out > 0);
        // amount_in should be less than what we provided (we had more than enough)
        assert!(amount_in <= amount_remaining);
    }

    #[test]
    fn test_swap_a_to_b_partial() {
        // price at tick 0, target at tick -100
        let sqrt_current = tick_to_sqrt_price_x96(0).unwrap();
        let sqrt_target = tick_to_sqrt_price_x96(-100).unwrap();
        let liquidity: u128 = 1_000_000_000;

        // tiny amount should not reach the target
        let amount_remaining: u128 = 1;

        let (next_price, amount_in, _amount_out) =
            compute_swap_step(sqrt_current, sqrt_target, liquidity, amount_remaining, true)
                .unwrap();

        // price should be between current and target
        assert!(next_price < sqrt_current);
        assert!(next_price > sqrt_target);
        // all input consumed in partial step
        assert_eq!(amount_in, amount_remaining);
    }

    #[test]
    fn test_swap_b_to_a_full_cross() {
        // price at tick 0, target at tick 10 (slightly higher price)
        let sqrt_current = tick_to_sqrt_price_x96(0).unwrap();
        let sqrt_target = tick_to_sqrt_price_x96(10).unwrap();
        let liquidity: u128 = 1_000_000_000;

        // large enough to fully cross
        let amount_remaining: u128 = 1_000_000_000;

        let (next_price, amount_in, amount_out) =
            compute_swap_step(sqrt_current, sqrt_target, liquidity, amount_remaining, false)
                .unwrap();

        assert_eq!(next_price, sqrt_target);
        assert!(amount_in > 0);
        assert!(amount_out > 0);
        assert!(amount_in <= amount_remaining);
    }

    #[test]
    fn test_swap_zero_liquidity_fails() {
        let sqrt_current = tick_to_sqrt_price_x96(0).unwrap();
        let sqrt_target = tick_to_sqrt_price_x96(-100).unwrap();

        let result = compute_swap_step(sqrt_current, sqrt_target, 0, 1000, true);
        assert!(result.is_err());
    }
}
