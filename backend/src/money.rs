use rust_decimal::Decimal;

/// Gateway charge/discount math (port of PHP transaction-verify
/// pp-adapter.php:8816-8828), pure: converted amount in, net amount out.
///
/// net = amount - (fixed_discount + amount * pct_discount/100)
///             + (fixed_charge + amount * pct_charge/100)
pub fn apply_gateway_charges(
    amount: Decimal,
    fixed_discount: Decimal,
    percentage_discount: Decimal,
    fixed_charge: Decimal,
    percentage_charge: Decimal,
) -> Decimal {
    let pct_disc = amount * percentage_discount / Decimal::from(100);
    let pct_charge = amount * percentage_charge / Decimal::from(100);
    amount - (fixed_discount + pct_disc) + (fixed_charge + pct_charge)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(v: i64, scale: u32) -> Decimal {
        // d(150050, 2) => 1500.50 : value * 10^-scale
        Decimal::new(v, scale)
    }

    #[test]
    fn no_charges_passes_through() {
        let a = d(100, 0);
        assert_eq!(
            apply_gateway_charges(
                a,
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::ZERO
            ),
            a
        );
    }

    #[test]
    fn fixed_charge_added() {
        assert_eq!(
            apply_gateway_charges(
                d(100, 0),
                Decimal::ZERO,
                Decimal::ZERO,
                d(10, 0),
                Decimal::ZERO
            ),
            d(110, 0)
        );
    }

    #[test]
    fn fixed_discount_subtracted() {
        assert_eq!(
            apply_gateway_charges(
                d(100, 0),
                d(10, 0),
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::ZERO
            ),
            d(90, 0)
        );
    }

    #[test]
    fn percentage_charge_added() {
        // 100 + 2.5% = 102.5
        assert_eq!(
            apply_gateway_charges(
                d(100, 0),
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::ZERO,
                d(25, 1)
            ),
            d(1025, 1)
        );
    }

    #[test]
    fn mixed_charge_and_discount() {
        // 100: fixed_discount 5 + 1% disc, fixed_charge 2 + 2% charge
        // = 100 - (5+1) + (2+2) = 98
        assert_eq!(
            apply_gateway_charges(d(100, 0), d(5, 0), d(1, 0), d(2, 0), d(2, 0)),
            d(98, 0)
        );
    }

    #[test]
    fn exact_decimal_no_drift() {
        // 33.33 with 3% pct charge = 33.33 + 0.9999 (Decimal exact)
        let r = apply_gateway_charges(
            d(3333, 2),
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            d(3, 0),
        );
        assert_eq!(r, Decimal::new(343299, 4));
    }
}
