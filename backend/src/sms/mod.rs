// Table-driven SMS parsing (PRD §6). Rules come from the `sms_rules` table
// (seeded by migration); this module is pure: rule structs in, parse result out.

pub mod balance;
pub mod matching;
pub mod parse;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsRule {
    pub provider_key: String,
    pub mfs_type: String, // Personal | Agent | Merchant
    pub priority: i32,
    pub pattern: String,
    pub field_map: Vec<String>,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedSms {
    pub mfs: String,
    pub mfs_type: String,
    pub amount: Option<Decimal>,
    pub balance: Option<Decimal>,
    pub fee: Option<Decimal>,
    pub sender: Option<String>, // phone number that paid (from message body)
    pub receiver: Option<String>,
    pub trxid: Option<String>,
    pub reference: Option<String>,
    pub datetime: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WhitelistEntry {
    pub sender: String,
    pub provider_key: String,
    pub name: String,
    pub currency: String,
    pub balance_verify: bool,
}

impl ParsedSms {
    /// PHP Code 102 check: type, amount, sender(phone), trxid required.
    pub fn has_required_fields(&self) -> bool {
        self.amount.is_some()
            && self.sender.is_some()
            && self.trxid.is_some()
            && !self.mfs_type.is_empty()
    }
}
