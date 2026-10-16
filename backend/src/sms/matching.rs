use rust_decimal::Decimal;

/// Pure matching logic (port of verifyPaymentTolerance pp-functions.php:588
/// and the transaction-verify / cron matching flow).

#[derive(Debug, Clone)]
pub struct MatchTransaction {
    pub ref_id: String,
    pub gateway_id: String,
    pub sender_key: String,  // provider, e.g. 'bkash'
    pub sender_type: String, // Personal | Merchant | Agent
    pub local_net_amount: Decimal,
    pub existing_trx_id: String, // '--' when none yet
}

#[derive(Debug, Clone)]
pub struct MatchSms {
    pub id: i64,
    pub provider_key: String,
    pub mfs_type: String,
    pub trx_id: String,
    pub amount: Decimal,
    pub number: String, // paying customer's number from the SMS
    pub status: String, // approved | awaiting-review | used | error
}

#[derive(Debug, PartialEq)]
pub enum MatchOutcome {
    Completed {
        sms_id: i64,
        sender_number: String,
        trx_id: String,
    },
    DuplicateTrxId,
    NoSmsMatch,
    AmountOutOfRange,
    SmsNotApproved,
}

/// Decide the result of submitting a TrxID against a transaction.
/// Mirrors PHP transaction-verify (pp-adapter.php:8758+) and the cron matcher
/// (index.php:1834+): dedupe on trx_id, find approved SMS by
/// (sender_key, type, trx_id), tolerance check on amounts.
pub fn match_trxid(
    txn: &MatchTransaction,
    submitted_trx_id: &str,
    candidates: &[MatchSms],
    tolerance: Decimal,
) -> MatchOutcome {
    // This transaction already carries the submitted id: duplicate.
    if txn.existing_trx_id != "--" && txn.existing_trx_id == submitted_trx_id {
        return MatchOutcome::DuplicateTrxId;
    }
    // TrxID already consumed by another SMS (marked used): duplicate.
    if candidates
        .iter()
        .any(|s| s.trx_id == submitted_trx_id && s.status == "used")
    {
        return MatchOutcome::DuplicateTrxId;
    }

    let sms = candidates.iter().find(|s| {
        s.trx_id == submitted_trx_id
            && s.provider_key == txn.sender_key
            && s.mfs_type == txn.sender_type
    });

    let sms = match sms {
        Some(s) => s,
        None => return MatchOutcome::NoSmsMatch,
    };

    if sms.status != "approved" {
        return MatchOutcome::SmsNotApproved;
    }

    if !verify_payment_tolerance(txn.local_net_amount, sms.amount, tolerance) {
        return MatchOutcome::AmountOutOfRange;
    }

    MatchOutcome::Completed {
        sms_id: sms.id,
        sender_number: sms.number.clone(),
        trx_id: submitted_trx_id.to_string(),
    }
}

/// PHP verifyPaymentTolerance: paid ∈ [checkout, checkout + tolerance],
/// both checkout and paid must be > 0. Exact decimal comparison.
pub fn verify_payment_tolerance(checkout: Decimal, paid: Decimal, tolerance: Decimal) -> bool {
    if checkout <= Decimal::ZERO || paid <= Decimal::ZERO {
        return false;
    }
    let max_allowed = checkout + tolerance;
    paid >= checkout && paid <= max_allowed
}

/// Derive invoice status from its transactions: all completed => paid.
/// Pure function per the brand-isolation/pure-logic requirement.
#[derive(Debug, Clone, PartialEq)]
pub enum InvoiceStatus {
    Paid,
    Unpaid,
}

pub fn derive_invoice_status(transactions: &[&str]) -> InvoiceStatus {
    if !transactions.is_empty() && transactions.iter().all(|s| *s == "completed") {
        InvoiceStatus::Paid
    } else {
        InvoiceStatus::Unpaid
    }
}

/// Quantity/stock guard for payment links (0 = unlimited).
pub fn quantity_available(current: i32, requested: i32) -> bool {
    if current == 0 {
        return true; // unlimited
    }
    current >= requested && requested > 0
}

pub fn quantity_after_sale(current: i32, requested: i32) -> i32 {
    if current == 0 {
        return 0;
    }
    current - requested
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(v: i64, scale: u32) -> Decimal {
        Decimal::new(v * 10i64.pow(scale), scale)
    }

    fn d_plain(v: i64) -> Decimal {
        Decimal::from(v)
    }

    fn sms(id: i64, trx: &str, amount: i64, status: &str, key: &str, ty: &str) -> MatchSms {
        MatchSms {
            id,
            provider_key: key.into(),
            mfs_type: ty.into(),
            trx_id: trx.into(),
            amount: d_plain(amount),
            number: "01700000000".into(),
            status: status.into(),
        }
    }

    fn txn(net: i64) -> MatchTransaction {
        MatchTransaction {
            ref_id: "pp1".into(),
            gateway_id: "gw".into(),
            sender_key: "bkash".into(),
            sender_type: "Personal".into(),
            local_net_amount: d_plain(net),
            existing_trx_id: "--".into(),
        }
    }

    #[test]
    fn full_match_completes() {
        let t = txn(500);
        let cands = vec![sms(1, "TRX123", 500, "approved", "bkash", "Personal")];
        let out = match_trxid(&t, "TRX123", &cands, Decimal::ZERO);
        assert_eq!(
            out,
            MatchOutcome::Completed {
                sms_id: 1,
                sender_number: "01700000000".into(),
                trx_id: "TRX123".into()
            }
        );
    }

    #[test]
    fn duplicate_trxid_rejected() {
        let t = txn(500);
        // SMS already used by another transaction.
        let cands = vec![sms(1, "TRX123", 500, "used", "bkash", "Personal")];
        let out = match_trxid(&t, "TRX123", &cands, Decimal::ZERO);
        assert_eq!(out, MatchOutcome::DuplicateTrxId);
    }

    #[test]
    fn no_sms_match_when_provider_differs() {
        let t = txn(500);
        let cands = vec![sms(1, "TRX123", 500, "approved", "nagad", "Personal")];
        assert_eq!(
            match_trxid(&t, "TRX123", &cands, Decimal::ZERO),
            MatchOutcome::NoSmsMatch
        );
    }

    #[test]
    fn no_sms_match_when_type_differs() {
        let t = txn(500);
        let cands = vec![sms(1, "TRX123", 500, "approved", "bkash", "Merchant")];
        assert_eq!(
            match_trxid(&t, "TRX123", &cands, Decimal::ZERO),
            MatchOutcome::NoSmsMatch
        );
    }

    #[test]
    fn amount_out_of_range_rejected() {
        let t = txn(500);
        let cands = vec![sms(1, "TRX123", 400, "approved", "bkash", "Personal")];
        assert_eq!(
            match_trxid(&t, "TRX123", &cands, Decimal::ZERO),
            MatchOutcome::AmountOutOfRange
        );
    }

    #[test]
    fn tolerance_boundaries() {
        let checkout = d(100, 0);
        // paid < checkout
        assert!(!verify_payment_tolerance(
            checkout,
            Decimal::new(9999, 2),
            Decimal::ZERO
        ));
        // paid == checkout
        assert!(verify_payment_tolerance(checkout, checkout, Decimal::ZERO));
        // paid == checkout + tolerance
        assert!(verify_payment_tolerance(checkout, d(105, 0), d(5, 0)));
        // paid > checkout + tolerance
        assert!(!verify_payment_tolerance(checkout, d(106, 0), d(5, 0)));
        // zero checkout / zero paid always false
        assert!(!verify_payment_tolerance(Decimal::ZERO, d(10, 0), d(5, 0)));
        assert!(!verify_payment_tolerance(checkout, Decimal::ZERO, d(5, 0)));
    }

    #[test]
    fn exact_decimal_no_float_drift() {
        // 0.1 + 0.2 must equal 0.3 in Decimal world (floats fail this).
        let a = Decimal::new(1, 1);
        let b = Decimal::new(2, 1);
        assert_eq!((a + b), Decimal::new(3, 1));
    }

    #[test]
    fn invoice_paid_only_when_all_completed() {
        assert_eq!(derive_invoice_status(&["completed"]), InvoiceStatus::Paid);
        assert_eq!(
            derive_invoice_status(&["completed", "completed"]),
            InvoiceStatus::Paid
        );
        assert_eq!(
            derive_invoice_status(&["completed", "initiated"]),
            InvoiceStatus::Unpaid
        );
        assert_eq!(derive_invoice_status(&[]), InvoiceStatus::Unpaid);
    }

    #[test]
    fn quantity_guard() {
        assert!(quantity_available(0, 5)); // unlimited
        assert!(quantity_available(10, 5));
        assert!(!quantity_available(3, 5));
        assert!(!quantity_available(5, 0)); // nonsensical request
        assert_eq!(quantity_after_sale(10, 5), 5);
        assert_eq!(quantity_after_sale(0, 5), 0); // stays unlimited
    }
}
