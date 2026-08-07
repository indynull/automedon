//! Offline checks that operator-facing adapter metadata stays complete.

use automedon::adapter::{product_names, AdapterKind};

#[test]
fn every_product_has_binary_and_multi_turn_summary() {
    for name in product_names() {
        let kind = AdapterKind::parse(name).unwrap();
        assert!(kind.is_product());
        let bins = kind.default_binaries();
        assert!(!bins.is_empty(), "{name} empty default_binaries");
        assert!(
            !bins.contains("TODO"),
            "{name} placeholder binaries: {bins}"
        );
        let mt = kind.multi_turn_summary();
        assert!(!mt.is_empty(), "{name} empty multi_turn_summary");
        assert!(
            mt.len() > 4,
            "{name} multi_turn_summary too short: {mt:?}"
        );
    }
}

#[test]
fn product_names_cover_tier_set() {
    let names = product_names();
    for expected in [
        "claude", "codex", "gemini", "opencode", "grok", "cursor", "aider", "pi", "copilot",
    ] {
        assert!(
            names.contains(&expected),
            "missing product {expected} in {names:?}"
        );
    }
    assert!(!names.contains(&"mock"));
    assert!(!names.contains(&"generic"));
}
