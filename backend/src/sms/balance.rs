use rust_decimal::Decimal;

/// In-memory row for balance-chain reconciliation (port of
/// reconcileByLongestChain, pp-functions.php:1170). DB layer maps sms_data
/// rows into these structs; the algorithm itself is pure.
#[derive(Debug, Clone)]
pub struct ChainSms {
    pub id: i64,
    pub amount: Decimal,
    pub balance: Decimal,
    pub status: String, // approved | awaiting-review
}

#[derive(Debug, PartialEq)]
pub struct ChainOutcome {
    pub approve_ids: Vec<i64>,
    pub new_canonical_balance: Decimal,
}

/// BFS longest chain from the canonical balance, mirroring PHP:
/// each SMS defines prev = balance - amount; a link is consistent when
/// prev == running balance. Longest chain wins; ties resolved by lowest ids
/// (stable BFS order like PHP's queue).
pub fn reconcile_longest_chain(canonical_balance: Decimal, sms: &[ChainSms]) -> ChainOutcome {
    let usable: Vec<&ChainSms> = sms
        .iter()
        .filter(|s| s.amount > Decimal::ZERO && s.balance > Decimal::ZERO)
        .collect();

    let mut next: std::collections::HashMap<String, Vec<&ChainSms>> =
        std::collections::HashMap::new();
    for s in &usable {
        let prev = s.balance - s.amount;
        next.entry(prev.normalize().to_string())
            .or_default()
            .push(s);
    }

    let mut best: Vec<&ChainSms> = Vec::new();
    let mut best_end_balance = canonical_balance;

    let start_key = canonical_balance.normalize().to_string();
    let start_list = next.get(&start_key).cloned().unwrap_or_default();
    for first in start_list {
        let mut chain = vec![first];
        let mut running = first.balance;
        loop {
            let key = running.normalize().to_string();
            let candidates = next.get(&key).cloned().unwrap_or_default();
            // pick the next link not already in the chain (lowest id first —
            // usable preserves input order, which the DB layer orders by id)
            let nxt = candidates
                .iter()
                .find(|c| !chain.iter().any(|x| x.id == c.id));
            match nxt {
                Some(c) => {
                    chain.push(c);
                    running = c.balance;
                }
                None => break,
            }
        }
        if chain.len() > best.len() {
            best = chain;
            best_end_balance = running;
        }
    }

    ChainOutcome {
        approve_ids: best.iter().map(|s| s.id).collect(),
        new_canonical_balance: best_end_balance,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BalanceCheckInput {
    pub current_balance: Decimal,
    pub stored_simslot: String, // Sim1 | Sim2 | Any
    pub amount: Decimal,
    pub sms_balance: Decimal,
    pub sms_simslot_raw: String, // "1" | "2" from the app
}

#[derive(Debug, PartialEq)]
pub struct BalanceCheckOutcome {
    pub approved: bool,
    pub reasons: Vec<String>,
    pub new_current_balance: Decimal,
}

/// Port of the PHP balance-verification branch in sms-transmit-bulk
/// (pp-adapter.php:9400-9485). Reasons use PHP's exact strings.
pub fn check_balance(input: &BalanceCheckInput) -> BalanceCheckOutcome {
    let expected = input.current_balance + input.amount;
    let recorded_sim = match input.sms_simslot_raw.trim() {
        "1" => "Sim1".to_string(),
        _ => "Sim2".to_string(),
    };
    let sim_mismatch = input.stored_simslot != "Any" && input.stored_simslot != recorded_sim;

    let expected_balance_matches = expected == input.sms_balance;

    if expected_balance_matches {
        if sim_mismatch {
            BalanceCheckOutcome {
                approved: false,
                reasons: vec![format!(
                    "SIM slot and expected slot do not match. Recorded: {recorded_sim}; Expected: {}",
                    input.stored_simslot
                )],
                new_current_balance: input.current_balance, // unchanged on mismatch path
            }
        } else {
            BalanceCheckOutcome {
                approved: true,
                reasons: vec![],
                new_current_balance: expected,
            }
        }
    } else {
        let mut reasons = vec![format!(
            "SMS balance and expected balance do not match. Recorded SMS balance: {}; Expected balance: {}",
            input.sms_balance, expected
        )];
        if sim_mismatch {
            reasons.push(format!(
                "SIM slot and expected slot do not match. Recorded: {recorded_sim}; Expected: {}",
                input.stored_simslot
            ));
        }
        BalanceCheckOutcome {
            approved: false,
            reasons,
            new_current_balance: input.current_balance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(v: i64) -> Decimal {
        Decimal::from(v)
    }

    #[test]
    fn balance_match_approves_and_advances() {
        let out = check_balance(&BalanceCheckInput {
            current_balance: d(1000),
            stored_simslot: "Any".into(),
            amount: d(500),
            sms_balance: d(1500),
            sms_simslot_raw: "1".into(),
        });
        assert!(out.approved);
        assert_eq!(out.new_current_balance, d(1500));
    }

    #[test]
    fn balance_mismatch_awaits_review_with_php_reason() {
        let out = check_balance(&BalanceCheckInput {
            current_balance: d(1000),
            stored_simslot: "Any".into(),
            amount: d(500),
            sms_balance: d(999), // corrupt/missing intermediate
            sms_simslot_raw: "1".into(),
        });
        assert!(!out.approved);
        assert_eq!(
            out.reasons[0],
            "SMS balance and expected balance do not match. Recorded SMS balance: 999; Expected balance: 1500"
        );
    }

    #[test]
    fn sim_slot_mismatch_reason() {
        let out = check_balance(&BalanceCheckInput {
            current_balance: d(100),
            stored_simslot: "Sim1".into(),
            amount: d(10),
            sms_balance: d(110),
            sms_simslot_raw: "2".into(),
        });
        assert!(!out.approved);
        assert_eq!(
            out.reasons[0],
            "SIM slot and expected slot do not match. Recorded: Sim2; Expected: Sim1"
        );
    }

    #[test]
    fn both_mismatches_join_reasons() {
        let out = check_balance(&BalanceCheckInput {
            current_balance: d(100),
            stored_simslot: "Sim1".into(),
            amount: d(10),
            sms_balance: d(999),
            sms_simslot_raw: "2".into(),
        });
        assert_eq!(out.reasons.len(), 2); // PHP joins with " | "
    }

    #[test]
    fn chain_reconciles_longest() {
        // canonical = 100; sms 1: amount 50 balance 150 (prev 100 ✓),
        // sms 2: amount 30 balance 180 (prev 150 ✓), sms 3: amount 70 balance 999 (prev 929 ✗)
        let sms = vec![
            ChainSms {
                id: 1,
                amount: d(50),
                balance: d(150),
                status: "awaiting-review".into(),
            },
            ChainSms {
                id: 2,
                amount: d(30),
                balance: d(180),
                status: "awaiting-review".into(),
            },
            ChainSms {
                id: 3,
                amount: d(70),
                balance: d(999),
                status: "awaiting-review".into(),
            },
        ];
        let out = reconcile_longest_chain(d(100), &sms);
        assert_eq!(out.approve_ids, vec![1, 2]);
        assert_eq!(out.new_canonical_balance, d(180));
    }

    #[test]
    fn chain_skips_zero_amounts() {
        let sms = vec![
            ChainSms {
                id: 1,
                amount: Decimal::ZERO,
                balance: d(150),
                status: "awaiting-review".into(),
            },
            ChainSms {
                id: 2,
                amount: d(50),
                balance: d(150),
                status: "awaiting-review".into(),
            },
        ];
        let out = reconcile_longest_chain(d(100), &sms);
        assert_eq!(out.approve_ids, vec![2]);
    }

    #[test]
    fn chain_empty_when_no_link() {
        let sms = vec![ChainSms {
            id: 1,
            amount: d(50),
            balance: d(999),
            status: "approved".into(),
        }];
        let out = reconcile_longest_chain(d(100), &sms);
        assert!(out.approve_ids.is_empty());
        assert_eq!(out.new_canonical_balance, d(100));
    }
}
