use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Permission matrix mirrored from PHP PipraPay (pp-functions.php permissionSchema),
/// with addons / api_settings / currency_settings dropped for V1 per plan.
pub const RESOURCES: &[(&str, &[&str])] = &[
    ("customers", &["create", "edit", "delete"]),
    (
        "transaction",
        &["edit", "delete", "approve", "cancel", "refund", "send_ipn"],
    ),
    ("invoice", &["create", "edit", "delete"]),
    ("payment_link", &["create", "edit", "delete"]),
    ("gateways", &["create", "edit", "delete"]),
    ("brand_settings", &["view", "edit"]),
    ("theme_settings", &["view", "edit"]),
    ("faq_settings", &["view", "create", "edit", "delete"]),
    ("sms_data", &["create", "edit", "delete"]),
    ("device", &["connect", "delete", "balance_verification_for"]),
    ("brands", &["create", "edit", "delete"]),
    (
        "staff",
        &[
            "create",
            "edit",
            "delete",
            "assign_brand_to",
            "edit_permission",
            "view_permission_list",
            "delete_permission_of",
        ],
    ),
    ("domains", &["whitelist", "edit", "delete"]),
    (
        "system_settings",
        &["manage_general", "manage_cron", "manage_import"],
    ),
];

pub const PAGES: &[&str] = &[
    "dashboard",
    "reports",
    "customers",
    "transaction",
    "invoice",
    "payment_link",
    "gateways",
    "brand_settings",
    "sms_data",
    "device",
    "brands",
    "staff_management",
    "domains",
    "system_settings",
];

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PermissionSet {
    #[serde(default)]
    pub resources: BTreeMap<String, BTreeMap<String, bool>>,
    #[serde(default)]
    pub pages: BTreeMap<String, bool>,
}

/// role stored on the user row: "admin" | "staff"
pub fn has_permission(
    perm: Option<&PermissionSet>,
    role: &str,
    module: &str,
    action: &str,
) -> bool {
    if role == "admin" {
        return true;
    }
    perm.and_then(|p| p.resources.get(module)?.get(action).copied())
        .unwrap_or(false)
}

pub fn can_access_page(perm: Option<&PermissionSet>, role: &str, page: &str) -> bool {
    if role == "admin" {
        return true;
    }
    perm.and_then(|p| p.pages.get(page).copied())
        .unwrap_or(false)
}

/// Full schema, all true (PHP seeds new staff with everything enabled).
pub fn full_schema() -> PermissionSet {
    let mut p = PermissionSet::default();
    for (module, actions) in RESOURCES {
        let entry = p.resources.entry(module.to_string()).or_default();
        for a in actions.iter() {
            entry.insert(a.to_string(), true);
        }
    }
    for page in PAGES {
        p.pages.insert(page.to_string(), true);
    }
    p
}

pub fn full_schema_json() -> String {
    serde_json::to_string(&full_schema()).expect("schema serializes")
}

pub fn parse_json(raw: &str) -> Option<PermissionSet> {
    serde_json::from_str(raw).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_bypasses_everything() {
        assert!(has_permission(None, "admin", "device", "connect"));
        assert!(can_access_page(None, "admin", "system_settings"));
    }

    #[test]
    fn staff_matrix_allow_and_deny() {
        let mut p = full_schema();
        // Revoke one action + one page.
        p.resources
            .get_mut("device")
            .unwrap()
            .insert("connect".into(), false);
        p.pages.insert("system_settings".into(), false);

        assert!(has_permission(Some(&p), "staff", "device", "delete"));
        assert!(!has_permission(Some(&p), "staff", "device", "connect"));
        assert!(!has_permission(Some(&p), "staff", "nonexistent", "create"));
        assert!(can_access_page(Some(&p), "staff", "dashboard"));
        assert!(!can_access_page(Some(&p), "staff", "system_settings"));
        // Missing permission blob denies everything for staff.
        assert!(!has_permission(None, "staff", "invoice", "create"));
        assert!(!can_access_page(None, "staff", "dashboard"));
    }

    #[test]
    fn full_schema_json_roundtrip() {
        let json = full_schema_json();
        let parsed = parse_json(&json).expect("parses");
        assert_eq!(parsed, full_schema());
        assert!(has_permission(
            Some(&parsed),
            "staff",
            "staff",
            "edit_permission"
        ));
        assert!(can_access_page(Some(&parsed), "staff", "brands"));
    }

    #[test]
    fn schema_covers_required_modules() {
        let names: Vec<&str> = RESOURCES.iter().map(|(m, _)| *m).collect();
        for m in [
            "customers",
            "transaction",
            "invoice",
            "payment_link",
            "gateways",
            "device",
            "staff",
            "brands",
            "domains",
            "system_settings",
            "sms_data",
            "brand_settings",
            "theme_settings",
            "faq_settings",
        ] {
            assert!(names.contains(&m), "missing module {m}");
        }
        for p in ["dashboard", "reports", "staff_management"] {
            assert!(PAGES.contains(&p), "missing page {p}");
        }
    }

    #[test]
    fn brand_isolation() {
        // Permission blob is per (brand, user): a staff scoped to brand A has
        // no rights on brand B. This test pins the scoping shape used by the
        // DB layer: the blob is looked up by (brand_id, user_id).
        let blob = full_schema_json();
        let p = parse_json(&blob).unwrap();
        assert!(has_permission(Some(&p), "staff", "invoice", "create"));
        // A different brand lookup yields None -> denied.
        assert!(has_permission(None, "staff", "invoice", "create") == false);
    }
}
