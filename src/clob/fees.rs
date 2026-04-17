//! V2 fee calculation for Polymarket CLOB orders.
//!
//! Platform fees in V2 are dynamic: `fee = (amount / price) * rate * (price * (1 - price))^exponent`.
//! This replaces the V1 flat `feeRateBps` model.

use rust_decimal::Decimal;

/// Computes the platform fee rate for a given price.
///
/// Formula: `rate * (price * (1 - price))^exponent`
#[must_use]
pub fn platform_fee_rate(rate: Decimal, exponent: u32, price: Decimal) -> Decimal {
    let base = price * (Decimal::ONE - price);
    let mut power = Decimal::ONE;
    for _ in 0..exponent {
        power *= base;
    }
    rate * power
}

/// Computes the platform fee in collateral (pUSD) terms.
///
/// Formula: `(amount / price) * platform_fee_rate`
///
/// where `amount` is the notional in pUSD and `price` is the token price.
#[must_use]
pub fn platform_fee(amount: Decimal, price: Decimal, rate: Decimal, exponent: u32) -> Decimal {
    (amount / price) * platform_fee_rate(rate, exponent, price)
}

/// Adjusts a buy amount downward so that `amount + fees <= user_balance`.
///
/// If the user's balance can cover the full amount plus all fees, returns
/// `amount` unchanged. Otherwise returns the largest amount such that
/// `adjusted + platform_fee(adjusted) + builder_fee(adjusted) == user_balance`.
#[must_use]
pub fn adjust_buy_amount_for_fees(
    amount: Decimal,
    price: Decimal,
    fee_rate: Decimal,
    fee_exponent: u32,
    builder_taker_fee_rate: Decimal,
    user_balance: Decimal,
) -> Decimal {
    let pfr = platform_fee_rate(fee_rate, fee_exponent, price);
    let pf = (amount / price) * pfr;
    let total_cost = amount + pf + amount * builder_taker_fee_rate;

    if user_balance <= total_cost {
        user_balance / (Decimal::ONE + pfr / price + builder_taker_fee_rate)
    } else {
        amount
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    fn calc_platform_fee(amount: Decimal, price: Decimal, rate: Decimal, exp: u32) -> Decimal {
        (amount / price) * platform_fee_rate(rate, exp, price)
    }

    fn calc_builder_fee(amount: Decimal, builder_rate: Decimal) -> Decimal {
        amount * builder_rate
    }

    // --- Platform fee tests (rate=0.25, exp=2, C=100 contracts) ---

    #[test]
    fn platform_fee_price_0_5() {
        let fee = calc_platform_fee(dec!(50), dec!(0.5), dec!(0.25), 2);
        assert_eq!(fee.round_dp(6), dec!(1.5625));
    }

    #[test]
    fn platform_fee_price_0_3() {
        let fee = calc_platform_fee(dec!(30), dec!(0.3), dec!(0.25), 2);
        assert_eq!(fee.round_dp(6), dec!(1.1025));
    }

    #[test]
    fn platform_fee_price_0_1() {
        let fee = calc_platform_fee(dec!(10), dec!(0.1), dec!(0.25), 2);
        assert_eq!(fee.round_dp(6), dec!(0.2025));
    }

    #[test]
    fn platform_fee_price_0_7_symmetric_with_0_3() {
        let fee = calc_platform_fee(dec!(70), dec!(0.7), dec!(0.25), 2);
        assert_eq!(fee.round_dp(6), dec!(1.1025));
    }

    #[test]
    fn platform_fee_price_0_9_symmetric_with_0_1() {
        let fee = calc_platform_fee(dec!(90), dec!(0.9), dec!(0.25), 2);
        assert_eq!(fee.round_dp(6), dec!(0.2025));
    }

    // --- Builder fee tests ---

    #[test]
    fn builder_fee_1_percent_at_50c() {
        let fee = calc_builder_fee(dec!(50), dec!(0.01));
        assert_eq!(fee.round_dp(6), dec!(0.5));
    }

    #[test]
    fn builder_fee_5_percent_at_75c() {
        let fee = calc_builder_fee(dec!(150), dec!(0.05));
        assert_eq!(fee.round_dp(6), dec!(7.5));
    }

    // --- Combined fees ---

    #[test]
    fn combined_platform_and_builder_fee() {
        let amount = dec!(50);
        let price = dec!(0.5);
        let pf = calc_platform_fee(amount, price, dec!(0.25), 2);
        let bf = calc_builder_fee(amount, dec!(0.01));
        assert_eq!(pf.round_dp(6), dec!(1.5625));
        assert_eq!(bf.round_dp(6), dec!(0.5));
        assert_eq!((pf + bf).round_dp(6), dec!(2.0625));
    }

    // --- adjustBuyAmountForFees tests ---

    #[test]
    fn no_adjustment_when_balance_exceeds_total_cost() {
        let result = adjust_buy_amount_for_fees(
            dec!(50),
            dec!(0.5),
            dec!(0.25),
            2,
            Decimal::ZERO,
            dec!(100), // plenty of balance
        );
        assert_eq!(result, dec!(50));
    }

    #[test]
    fn adjustment_platform_fee_only() {
        let amount = dec!(50);
        let price = dec!(0.5);
        let adjusted = adjust_buy_amount_for_fees(
            amount,
            price,
            dec!(0.25),
            2,
            Decimal::ZERO,
            amount, // balance = amount, no room for fees
        );
        let fee = calc_platform_fee(adjusted, price, dec!(0.25), 2);
        let diff = (adjusted + fee - amount).abs();
        assert!(
            diff < dec!(0.0000000001),
            "adjusted + fee should equal balance"
        );
    }

    #[test]
    fn adjustment_builder_fee_only() {
        let amount = dec!(50);
        let price = dec!(0.5);
        let builder_rate = dec!(0.01);
        let adjusted =
            adjust_buy_amount_for_fees(amount, price, Decimal::ZERO, 0, builder_rate, amount);
        let fee = calc_builder_fee(adjusted, builder_rate);
        let diff = (adjusted + fee - amount).abs();
        assert!(
            diff < dec!(0.0000000001),
            "adjusted + fee should equal balance"
        );
    }

    #[test]
    fn adjustment_combined_fees() {
        let amount = dec!(50);
        let price = dec!(0.5);
        let builder_rate = dec!(0.01);
        let adjusted =
            adjust_buy_amount_for_fees(amount, price, dec!(0.25), 2, builder_rate, amount);
        let pf = calc_platform_fee(adjusted, price, dec!(0.25), 2);
        let bf = calc_builder_fee(adjusted, builder_rate);
        let diff = (adjusted + pf + bf - amount).abs();
        assert!(
            diff < dec!(0.0000000001),
            "adjusted + fees should equal balance"
        );
    }

    #[test]
    fn adjusted_is_less_than_original() {
        let amount = dec!(50);
        let adjusted =
            adjust_buy_amount_for_fees(amount, dec!(0.5), dec!(0.25), 2, Decimal::ZERO, amount);
        assert!(adjusted < amount);
    }

    // --- Production fee rate tests ---

    fn assert_production_fee(
        amount: Decimal,
        price: Decimal,
        rate: Decimal,
        exp: u32,
        expected: Decimal,
    ) {
        let fee = calc_platform_fee(amount, price, rate, exp);
        assert_eq!(fee.round_dp(2), expected, "rate={rate}, price={price}");
    }

    #[test]
    fn sports_fees_v2() {
        assert_production_fee(dec!(100), dec!(0.5), dec!(0.03), 1, dec!(1.50));
        assert_production_fee(dec!(100), dec!(0.3), dec!(0.03), 1, dec!(2.10));
        assert_production_fee(dec!(100), dec!(0.7), dec!(0.03), 1, dec!(0.90));
    }

    #[test]
    fn politics_fees_v2() {
        assert_production_fee(dec!(100), dec!(0.5), dec!(0.04), 1, dec!(2.00));
        assert_production_fee(dec!(100), dec!(0.3), dec!(0.04), 1, dec!(2.80));
        assert_production_fee(dec!(100), dec!(0.7), dec!(0.04), 1, dec!(1.20));
    }

    #[test]
    fn culture_fees_v2() {
        assert_production_fee(dec!(100), dec!(0.5), dec!(0.05), 1, dec!(2.50));
        assert_production_fee(dec!(100), dec!(0.3), dec!(0.05), 1, dec!(3.50));
        assert_production_fee(dec!(100), dec!(0.7), dec!(0.05), 1, dec!(1.50));
    }

    #[test]
    fn crypto_fees_v2() {
        assert_production_fee(dec!(100), dec!(0.5), dec!(0.072), 1, dec!(3.60));
        assert_production_fee(dec!(100), dec!(0.3), dec!(0.072), 1, dec!(5.04));
        assert_production_fee(dec!(100), dec!(0.7), dec!(0.072), 1, dec!(2.16));
    }

    // --- Production: adjusted + fee = balance ---

    fn assert_adjusted_plus_fee_equals_balance(rate: Decimal, exp: u32, price: Decimal) {
        let amount = dec!(100);
        let adjusted = adjust_buy_amount_for_fees(amount, price, rate, exp, Decimal::ZERO, amount);
        let fee = calc_platform_fee(adjusted, price, rate, exp);
        let diff = (adjusted + fee - amount).abs();
        assert!(
            diff < dec!(0.0000000001),
            "rate={rate}, price={price}: adjusted({adjusted}) + fee({fee}) != balance({amount}), diff={diff}"
        );
    }

    #[test]
    fn production_adjusted_sports() {
        for price in [dec!(0.3), dec!(0.5), dec!(0.7)] {
            assert_adjusted_plus_fee_equals_balance(dec!(0.03), 1, price);
        }
    }

    #[test]
    fn production_adjusted_politics() {
        for price in [dec!(0.3), dec!(0.5), dec!(0.7)] {
            assert_adjusted_plus_fee_equals_balance(dec!(0.04), 1, price);
        }
    }

    #[test]
    fn production_adjusted_culture() {
        for price in [dec!(0.3), dec!(0.5), dec!(0.7)] {
            assert_adjusted_plus_fee_equals_balance(dec!(0.05), 1, price);
        }
    }

    #[test]
    fn production_adjusted_crypto() {
        for price in [dec!(0.3), dec!(0.5), dec!(0.7)] {
            assert_adjusted_plus_fee_equals_balance(dec!(0.072), 1, price);
        }
    }
}
