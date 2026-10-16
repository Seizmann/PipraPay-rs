use rust_decimal::Decimal;
use std::sync::OnceLock;

use super::{ParsedSms, SmsRule, WhitelistEntry};

/// Normalize whitespace like PHP: trim(preg_replace('/\s+/', ' ', $message)).
pub fn normalize_message(message: &str) -> String {
    let mut out = String::with_capacity(message.len());
    let mut in_space = false;
    for c in message.trim().chars() {
        if c.is_whitespace() {
            if !in_space {
                out.push(' ');
                in_space = true;
            }
        } else {
            out.push(c);
            in_space = false;
        }
    }
    out
}

/// Strip commas from amount/balance/fee strings, then parse to Decimal.
pub fn parse_money(raw: &str) -> Option<Decimal> {
    let cleaned: String = raw.chars().filter(|c| *c != ',').collect();
    cleaned.parse::<Decimal>().ok()
}

/// Regex cache: rules are recompiled only when the rule set changes.
static REGEX_CACHE: OnceLock<std::sync::Mutex<std::collections::HashMap<String, regex::Regex>>> =
    OnceLock::new();

fn cached_regex(pattern: &str) -> Option<regex::Regex> {
    let cache = REGEX_CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    let mut guard = cache.lock().ok()?;
    if let Some(re) = guard.get(pattern) {
        return Some(re.clone());
    }
    let re = regex::Regex::new(pattern).ok()?;
    guard.insert(pattern.to_string(), re.clone());
    Some(re)
}

/// Set a field on the mutable builder by name (mirrors PHP's `map` arrays).
fn assign(p: &mut ParsedSms, field: &str, value: &str) {
    match field {
        "amount" => p.amount = parse_money(value),
        "balance" => p.balance = parse_money(value),
        "fee" => p.fee = parse_money(value),
        "sender" => p.sender = Some(value.to_string()),
        "receiver" => p.receiver = Some(value.to_string()),
        "trxid" => p.trxid = Some(value.to_string()),
        "ref" => p.reference = Some(value.to_string()),
        "datetime" => p.datetime = Some(value.to_string()),
        _ => {}
    }
}

/// Find the whitelist entry for a sender (PHP senderWhitelist($sender)):
/// exact match on lowercased trimmed sender. Falls back to bidirectional
/// substring match because the Android app stores the whitelist the same way
/// (PrefManager.isSenderAllowed) — an SMS from "bKash" must hit "bkash".
pub fn lookup_sender<'a>(
    whitelist: &'a [WhitelistEntry],
    sender: &str,
) -> Option<&'a WhitelistEntry> {
    let clean = sender.trim().to_lowercase();
    if let Some(e) = whitelist.iter().find(|e| e.sender == clean) {
        return Some(e);
    }
    whitelist
        .iter()
        .find(|e| clean.contains(&e.sender) || e.sender.contains(&clean))
}

/// Parse a message against the rule table. Rules must be pre-sorted by
/// priority DESC; first match wins (PHP usort + foreach). Returns None when no
/// rule matches (caller maps that to error Code 101).
pub fn parse_message(mfs: &str, message: &str, rules: &[SmsRule]) -> Option<ParsedSms> {
    let normalized = normalize_message(message);
    let mut candidates: Vec<&SmsRule> = rules
        .iter()
        .filter(|r| r.active && r.provider_key.eq_ignore_ascii_case(mfs))
        .collect();
    candidates.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then(b.pattern.len().cmp(&a.pattern.len()))
    });

    for rule in candidates {
        let re = cached_regex(&rule.pattern)?;
        if let Some(caps) = re.captures(&normalized) {
            let mut parsed = ParsedSms {
                mfs: mfs.to_lowercase(),
                mfs_type: rule.mfs_type.clone(),
                amount: None,
                balance: None,
                fee: None,
                sender: None,
                receiver: None,
                trxid: None,
                reference: None,
                datetime: None,
            };
            for (i, field) in rule.field_map.iter().enumerate() {
                if let Some(m) = caps.get(i + 1) {
                    assign(&mut parsed, field, m.as_str());
                }
            }
            return Some(parsed);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> Vec<SmsRule> {
        // Mirrors migration 20261016000004 seeds.
        let mk = |provider: &str, ty: &str, prio: i32, pattern: &str, map: &[&str]| SmsRule {
            provider_key: provider.into(),
            mfs_type: ty.into(),
            priority: prio,
            pattern: pattern.into(),
            field_map: map.iter().map(|s| s.to_string()).collect(),
            active: true,
        };
        vec![
            mk(
                "bkash",
                "Personal",
                100,
                r"(?i)You have received Tk ([0-9,.]+) from ([0-9]+)\.(?:\s*Ref[:\-]?\s*(\S+))? Fee Tk ([0-9,.]+)\. Balance Tk ([0-9,.]+)\. TrxID ([A-Z0-9]+) at ([0-9/: ]+)",
                &[
                    "amount", "sender", "ref", "fee", "balance", "trxid", "datetime",
                ],
            ),
            mk(
                "bkash",
                "Personal",
                90,
                r"(?i)Cash In Tk ([0-9,.]+) from ([0-9]+) successful\. Fee Tk ([0-9,.]+)\. Balance Tk ([0-9,.]+)\. TrxID ([A-Z0-9]+) at ([0-9/: ]+)",
                &["amount", "sender", "fee", "balance", "trxid", "datetime"],
            ),
            mk(
                "bkash",
                "Merchant",
                80,
                r"(?i)You have received payment Tk ([0-9,.]+) from ([0-9X]+)\.(?:\s*Ref[:\-]?\s*(\S+))? Fee Tk ([0-9,.]+)\. Balance Tk ([0-9,.]+)\. TrxID ([A-Z0-9]+) at ([0-9/: ]+)",
                &[
                    "amount", "sender", "ref", "fee", "balance", "trxid", "datetime",
                ],
            ),
            mk(
                "bkash",
                "Agent",
                60,
                r"(?i)Cash In Tk ([0-9,.]+) from ([0-9]+) successful\. Balance Tk ([0-9,.]+)\. TrxID ([A-Z0-9]+)",
                &["amount", "sender", "balance", "trxid"],
            ),
            mk(
                "nagad",
                "Personal",
                100,
                r"(?i)Money Received\. Amount: Tk ([0-9,.]+) Sender: ([0-9]+)(?:\s*Ref[:\-]?\s*(\S+))? TxnID: ([A-Z0-9]+) Balance: Tk ([0-9,.]+) ([0-9/: ]+)",
                &["amount", "sender", "ref", "trxid", "balance", "datetime"],
            ),
            mk(
                "nagad",
                "Personal",
                90,
                r"(?i)Cash In Received\. Amount: Tk ([0-9,.]+) Uddokta: ([0-9]+) TxnID: ([A-Z0-9]+) Balance: ([0-9,.]+) ([0-9/: ]+)",
                &["amount", "sender", "trxid", "balance", "datetime"],
            ),
            mk(
                "nagad",
                "Agent",
                60,
                r"(?i)Cash In Tk ([0-9,.]+) from ([0-9]+) successful\. Balance Tk ([0-9,.]+)\. TrxID ([A-Z0-9]+)",
                &["amount", "sender", "balance", "trxid"],
            ),
        ]
    }

    #[test]
    fn bkash_personal_with_ref() {
        let r = rules();
        let msg = "You have received Tk 1,500.50 from 01712345678. Ref: INV-42 Fee Tk 0.00. Balance Tk 12,345.67. TrxID 9AB7CXYZ12 at 16/10/2026 14:03";
        let p = parse_message("bkash", msg, &r).expect("parses");
        assert_eq!(p.mfs_type, "Personal");
        assert_eq!(p.amount, Some(Decimal::new(150050, 2)));
        assert_eq!(p.sender.as_deref(), Some("01712345678"));
        assert_eq!(p.reference.as_deref(), Some("INV-42"));
        assert_eq!(p.fee, Some(Decimal::ZERO));
        assert_eq!(p.balance, Some(Decimal::new(1234567, 2)));
        assert_eq!(p.trxid.as_deref(), Some("9AB7CXYZ12"));
        assert!(p.has_required_fields());
    }

    #[test]
    fn bkash_personal_without_ref() {
        let r = rules();
        // Ref clause optional — must still parse.
        let msg = "You have received Tk 250 from 01811111111. Fee Tk 0.00. Balance Tk 999.00. TrxID ABC123 at 16/10/2026 09:10";
        let p = parse_message("bkash", msg, &r).unwrap();
        assert_eq!(p.amount, Some(Decimal::new(250, 0)));
        assert_eq!(p.reference, None);
    }

    #[test]
    fn bkash_cashin_personal() {
        let r = rules();
        let msg = "Cash In Tk 5,000 from 01922222222 successful. Fee Tk 5.00. Balance Tk 15,000.00. TrxID XYZ999 at 16/10/2026 10:00";
        let p = parse_message("bkash", msg, &r).unwrap();
        assert_eq!(p.mfs_type, "Personal");
        assert_eq!(p.amount, Some(Decimal::new(500000, 2)));
        assert_eq!(p.trxid.as_deref(), Some("XYZ999"));
    }

    #[test]
    fn bkash_merchant() {
        let r = rules();
        let msg = "You have received payment Tk 800 from 01733333333. Ref: order-9 Fee Tk 0.00. Balance Tk 20,000.00. TrxID M12345678 at 16/10/2026 11:30";
        let p = parse_message("bkash", msg, &r).unwrap();
        assert_eq!(p.mfs_type, "Merchant");
    }

    #[test]
    fn bkash_agent_synthetic() {
        // SYNTHETIC sample: constructed from the agent pattern derived from
        // PHP's commented-out agent template (pp-functions.php:919), not a
        // real-world capture.
        let r = rules();
        let msg =
            "Cash In Tk 3,000 from 01644444444 successful. Balance Tk 30,000.00. TrxID A9Z8Y7X6W5";
        let p = parse_message("bkash", msg, &r).unwrap();
        assert_eq!(p.mfs_type, "Agent");
        assert_eq!(p.amount, Some(Decimal::new(300000, 2)));
        assert_eq!(p.balance, Some(Decimal::new(3000000, 2)));
        assert_eq!(p.trxid.as_deref(), Some("A9Z8Y7X6W5"));
    }

    #[test]
    fn nagad_personal_with_ref() {
        let r = rules();
        let msg = "Money Received. Amount: Tk 2,000.75 Sender: 01755555555 Ref: 88 TxnID: 7QW2ER4T Balance: Tk 5,000.75 16/10/2026 12:00";
        let p = parse_message("nagad", msg, &r).unwrap();
        assert_eq!(p.mfs_type, "Personal");
        assert_eq!(p.amount, Some(Decimal::new(200075, 2)));
        assert_eq!(p.trxid.as_deref(), Some("7QW2ER4T"));
        assert_eq!(p.balance, Some(Decimal::new(500075, 2)));
    }

    #[test]
    fn nagad_cashin_personal() {
        let r = rules();
        let msg = "Cash In Received. Amount: Tk 1,000 Uddokta: 01766666666 TxnID: 99K8J7H6 Balance: 4,000 16/10/2026 13:00";
        let p = parse_message("nagad", msg, &r).unwrap();
        assert_eq!(p.mfs_type, "Personal");
        assert_eq!(p.amount, Some(Decimal::new(1000, 0)));
    }

    #[test]
    fn nagad_agent_synthetic() {
        // SYNTHETIC sample: constructed from the Nagad agent pattern derived
        // from PHP's commented-out agent template (pp-functions.php:950).
        let r = rules();
        let msg = "Cash In Tk 750 from 01577777777 successful. Balance Tk 1,750. TrxID N0P1Q2R3S4";
        let p = parse_message("nagad", msg, &r).unwrap();
        assert_eq!(p.mfs_type, "Agent");
        assert_eq!(p.amount, Some(Decimal::new(750, 0)));
    }

    #[test]
    fn merchant_pattern_does_not_shadow_personal() {
        // "You have received Tk ..." (personal) must match the p100 personal
        // rule, not fall into the p80 merchant rule.
        let r = rules();
        let msg = "You have received Tk 100 from 01111111111. Fee Tk 0. Balance Tk 100. TrxID P1 at 16/10/2026 08:00";
        let p = parse_message("bkash", msg, &r).unwrap();
        assert_eq!(p.mfs_type, "Personal");
    }

    #[test]
    fn unmatched_message_returns_none() {
        let r = rules();
        assert_eq!(
            parse_message("bkash", "Your account was debited Tk 50", &r),
            None
        );
        assert_eq!(parse_message("unknown-provider", "anything", &r), None);
    }

    #[test]
    fn whitespace_normalized_multiline() {
        let r = rules();
        // App concatenates multi-part SMS; newlines collapse like PHP's preg_replace.
        let msg = "You have received Tk 500\n  from 01234567890. Fee Tk 0.00.\nBalance Tk 500.00. TrxID W1X2Y3Z4 at 16/10/2026 07:00";
        let p = parse_message("bkash", msg, &r).unwrap();
        assert_eq!(p.amount, Some(Decimal::new(500, 0)));
    }

    #[test]
    fn money_comma_stripping() {
        assert_eq!(parse_money("12,345.67"), Some(Decimal::new(1234567, 2)));
        assert_eq!(parse_money("500"), Some(Decimal::new(500, 0)));
        assert_eq!(parse_money(""), None);
    }

    #[test]
    fn sender_lookup_exact_and_substring() {
        let wl = vec![
            WhitelistEntry {
                sender: "bkash".into(),
                provider_key: "bkash".into(),
                name: "bKash".into(),
                currency: "BDT".into(),
                balance_verify: true,
            },
            WhitelistEntry {
                sender: "ibbl .".into(),
                provider_key: "cellfin".into(),
                name: "Cellfin".into(),
                currency: "BDT".into(),
                balance_verify: false,
            },
        ];
        assert_eq!(lookup_sender(&wl, "BKASH").unwrap().provider_key, "bkash");
        assert_eq!(lookup_sender(&wl, "bKash ").unwrap().provider_key, "bkash");
        assert_eq!(
            lookup_sender(&wl, "ibbl .").unwrap().provider_key,
            "cellfin"
        );
        assert!(lookup_sender(&wl, "random-sender").is_none());
    }
}
