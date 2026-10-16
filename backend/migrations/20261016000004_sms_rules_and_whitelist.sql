-- Seed SMS parse rules (ported from PHP pp-functions.php MFSMessageVerified)
-- and sender whitelist (senderWhitelist()). Agent rules for bKash/Nagad are NEW:
-- derived from PHP's commented-out agent templates (pp-functions.php:919 / :950).

INSERT INTO sender_whitelist (sender, provider_key, name, currency, balance_verify) VALUES
    ('bkash', 'bkash', 'bKash', 'BDT', true),
    ('nagad', 'nagad', 'Nagad', 'BDT', true),
    ('16216', 'rocket', 'Rocket', 'BDT', true),
    ('upay', 'upay', 'Upay', 'BDT', true),
    ('tap.', 'tap', 'Tap', 'USD', true),
    ('ibbl .', 'cellfin', 'Cellfin', 'BDT', false),
    ('01847-348685', 'okwallet', 'Ok Wallet', 'BDT', true),
    ('16259', 'mcash', 'mCash', 'BDT', true),
    ('pathaopay', 'pathaopay', 'Pathao Pay', 'BDT', true),
    ('telecash', 'telecash', 'TeleCash', 'BDT', true),
    ('09638-900800', 'ipay', 'Ipay', 'BDT', true);

INSERT INTO sms_rules (provider_key, mfs_type, priority, pattern, field_map) VALUES
-- bKash Personal p100 (PHP line 898)
('bkash','Personal',100,
 '(?i)You have received Tk ([0-9,.]+) from ([0-9]+)\.(?:\s*Ref[:\-]?\s*(\S+))? Fee Tk ([0-9,.]+)\. Balance Tk ([0-9,.]+)\. TrxID ([A-Z0-9]+) at ([0-9/: ]+)',
 '["amount","sender","ref","fee","balance","trxid","datetime"]'),
-- bKash Personal p90 (PHP line 904)
('bkash','Personal',90,
 '(?i)Cash In Tk ([0-9,.]+) from ([0-9]+) successful\. Fee Tk ([0-9,.]+)\. Balance Tk ([0-9,.]+)\. TrxID ([A-Z0-9]+) at ([0-9/: ]+)',
 '["amount","sender","fee","balance","trxid","datetime"]'),
-- bKash Merchant p80 (PHP line 910)
('bkash','Merchant',80,
 '(?i)You have received payment Tk ([0-9,.]+) from ([0-9X]+)\.(?:\s*Ref[:\-]?\s*(\S+))? Fee Tk ([0-9,.]+)\. Balance Tk ([0-9,.]+)\. TrxID ([A-Z0-9]+) at ([0-9/: ]+)',
 '["amount","sender","ref","fee","balance","trxid","datetime"]'),
-- bKash Agent p60 — NEW, derived from PHP commented agent template (line 919)
('bkash','Agent',60,
 '(?i)Cash In Tk ([0-9,.]+) from ([0-9]+) successful\. Balance Tk ([0-9,.]+)\. TrxID ([A-Z0-9]+)',
 '["amount","sender","balance","trxid"]'),
-- Nagad Personal p100 (PHP line 928)
('nagad','Personal',100,
 '(?i)Money Received\. Amount: Tk ([0-9,.]+) Sender: ([0-9]+)(?:\s*Ref[:\-]?\s*(\S+))? TxnID: ([A-Z0-9]+) Balance: Tk ([0-9,.]+) ([0-9/: ]+)',
 '["amount","sender","ref","trxid","balance","datetime"]'),
-- Nagad Personal p90 (PHP line 934)
('nagad','Personal',90,
 '(?i)Cash In Received\. Amount: Tk ([0-9,.]+) Uddokta: ([0-9]+) TxnID: ([A-Z0-9]+) Balance: ([0-9,.]+) ([0-9/: ]+)',
 '["amount","sender","trxid","balance","datetime"]'),
-- Nagad Agent p60 — NEW, derived from PHP commented agent template (line 950)
('nagad','Agent',60,
 '(?i)Cash In Tk ([0-9,.]+) from ([0-9]+) successful\. Balance Tk ([0-9,.]+)\. TrxID ([A-Z0-9]+)',
 '["amount","sender","balance","trxid"]'),
-- Rocket Personal p100 (PHP line 959)
('rocket','Personal',100,
 '(?i)Tk([0-9,.]+) received from A/C:([*0-9]+) Fee:Tk([0-9,.]+), Your A/C Balance: Tk([0-9,.]+) TxnId:([A-Z0-9]+)(?: Date:([A-Za-z0-9\-: ]+))?',
 '["amount","sender","fee","balance","trxid","datetime"]'),
-- Upay Personal p100 (PHP line 984)
('upay','Personal',100,
 '(?i)Tk\. ([0-9,.]+) has been received from ([0-9]+)\.(?:\s*Ref[:\-]?\s*(\S+))? Balance Tk\. ([0-9,.]+)\. TrxID ([A-Z0-9]+) at ([0-9/: ]+)\.',
 '["amount","sender","ref","balance","trxid","datetime"]'),
-- Tap Personal p100 (PHP line 1009)
('tap','Personal',100,
 '(?i)Received Tk ([0-9,.]+) from ([0-9]+)\. Balance Tk\. ([0-9,.]+)\. TxID: ([A-Z0-9]+)\.',
 '["amount","sender","balance","trxid"]'),
-- Cellfin Personal p100 (PHP line 1034)
('cellfin','Personal',100,
 '(?i)Islami Bank CellFin Received ([0-9,.]+) Tk From CellFin: ([0-9]+) To CellFin: ([0-9]+) TrxId: ([A-Z0-9]+)',
 '["amount","sender","receiver","trxid"]'),
-- OkWallet Personal p100 (PHP line 1059)
('okwallet','Personal',100,
 '(?i)\(OK Wallet\) Successfully received Tk ([0-9,.]+) from A/C ([0-9]+)\.(?:\s*Ref[:\-]?\s*(\S+))? Balance Tk ([0-9,.]+)\. TrxID ([A-Z0-9]+)',
 '["amount","sender","ref","balance","trxid"]'),
-- mCash Personal p100 (PHP line 1084)
('mcash','Personal',100,
 '(?i)IBBL mCash You have received Tk: ([0-9,.]+) From: ([0-9]+)(?:\s*Reference:\s*(\S*))? Balance Tk: ([0-9,.]+) TrxID: ([A-Z0-9]+)',
 '["amount","sender","ref","balance","trxid"]'),
-- PathaoPay Personal p100 (PHP line 1109)
('pathaopay','Personal',100,
 '(?i)You have received BDT ([0-9,.]+) from (\+?[0-9]+)\. Balance BDT ([0-9,.]+) TrxID ([A-Z0-9]+)',
 '["amount","sender","balance","trxid"]');

-- Default settings (mirrors PHP env defaults; webhook retry limit = 5 per plan)
INSERT INTO settings (brand_id, option_name, value) VALUES
    ('both', 'webhook_retry_attempts_limit', '5'),
    ('both', 'geneal-application-settings-default_timezone', 'Asia/Dhaka');
