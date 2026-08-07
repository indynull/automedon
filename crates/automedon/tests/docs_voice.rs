//! Structural check: user-facing markdown stays free of fake adoption claims
//! and states alpha quality in the README and handbook introduction.

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

/// Phrases that oversell maturity or invent adoption (product claims only).
const FORBIDDEN: &[&str] = &[
    "Production driver",
    "production driver",
    "Teams use",
    "built for teams",
    "customer script",
    "production-ready",
    "production-grade",
    "Vendor QA",
    "vendor QA",
];

/// Fancy punctuation that should not decorate user-facing prose.
fn has_fancy_unicode(s: &str) -> bool {
    s.chars().any(|c| {
        matches!(
            c,
            '\u{2013}' // en dash
                | '\u{2014}' // em dash
                | '\u{2018}'
                | '\u{2019}'
                | '\u{201c}'
                | '\u{201d}'
                | '\u{2026}' // ellipsis
                | '\u{2192}'
                | '\u{2190}'
                | '\u{00b7}' // middle dot
        )
    })
}

fn collect_md(dir: &Path, out: &mut Vec<PathBuf>) {
    if !dir.is_dir() {
        return;
    }
    for entry in std::fs::read_dir(dir).expect("read_dir") {
        let entry = entry.expect("entry");
        let p = entry.path();
        if p.is_dir() {
            if p.file_name().and_then(|s| s.to_str()) == Some("automedon_demo") {
                continue;
            }
            collect_md(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("md") {
            out.push(p);
        }
    }
}

#[test]
fn user_facing_docs_avoid_adoption_and_production_claims() {
    let root = workspace_root();
    let mut files = vec![root.join("README.md")];
    collect_md(&root.join("docs"), &mut files);
    collect_md(&root.join("examples"), &mut files);

    let mut phrase_hits = Vec::new();
    let mut unicode_hits = Vec::new();
    for path in &files {
        let text = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        for (i, line) in text.lines().enumerate() {
            for needle in FORBIDDEN {
                if line.contains(needle) {
                    phrase_hits.push(format!("{}:{}: {line}", path.display(), i + 1));
                }
            }
            if has_fancy_unicode(line) {
                unicode_hits.push(format!("{}:{}: {line}", path.display(), i + 1));
            }
        }
    }
    assert!(
        phrase_hits.is_empty(),
        "forbidden adoption/maturity claims in user docs:\n{}",
        phrase_hits.join("\n")
    );
    assert!(
        unicode_hits.is_empty(),
        "fancy unicode in user docs (use ASCII punctuation):\n{}",
        unicode_hits.join("\n")
    );
}

#[test]
fn readme_and_introduction_state_alpha() {
    let root = workspace_root();
    let readme = std::fs::read_to_string(root.join("README.md")).expect("README");
    let intro = std::fs::read_to_string(root.join("docs/introduction.md")).expect("introduction");
    // Alpha must appear early (first screen of content).
    let readme_head: String = readme.chars().take(800).collect();
    let intro_head: String = intro.chars().take(800).collect();
    assert!(
        readme_head.to_ascii_lowercase().contains("alpha"),
        "README must state alpha near the top"
    );
    assert!(
        intro_head.to_ascii_lowercase().contains("alpha"),
        "docs/introduction.md must state alpha near the top"
    );
}

#[test]
fn public_getting_started_docs_lead_with_product_not_mock() {
    // Operators should not be steered to mock as the primary path.
    let root = workspace_root();
    for rel in ["README.md", "docs/getting-started.md", "docs/examples.md"] {
        let text = std::fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
        let head: String = text.chars().take(1800).collect();
        let head_l = head.to_ascii_lowercase();
        assert!(
            head_l.contains("examples/harnesses") || head_l.contains("harnesses/"),
            "{rel} head must mention product harness examples"
        );
        // First code fence (if any) must not be a mock run as the first command.
        if let Some(fence) = head.find("```") {
            let after = &head[fence..];
            let end = after[3..]
                .find("```")
                .map(|i| i + 3)
                .unwrap_or(after.len().min(400));
            let block = &after[..end.min(after.len())];
            assert!(
                !block.contains("examples/mock/") && !block.contains("shot mock"),
                "{rel}: first code block must not feature mock:\n{block}"
            );
        }
    }
}

#[test]
fn readme_docs_row_is_two_column_table() {
    // Bare `|` inside a table cell splits the row; Docs must join links with ` / `.
    let readme = std::fs::read_to_string(workspace_root().join("README.md")).expect("README");
    let docs_line = readme
        .lines()
        .find(|l| l.trim_start().starts_with("| Docs |") || l.trim_start().starts_with("| Docs|"))
        .expect("README must have a Docs table row");
    // Outer pipes + one cell separator = three pipe chars for a 2-column row.
    let pipe_count = docs_line.chars().filter(|c| *c == '|').count();
    assert_eq!(
        pipe_count, 3,
        "Docs row must be a 2-column markdown table row (3 pipes), got {pipe_count} in: {docs_line}"
    );
    assert!(
        docs_line.contains(" / ")
            && docs_line.contains("Handbook")
            && docs_line.contains("Smoke checklist"),
        "Docs row should list handbook and smoke checklist joined by ` / `: {docs_line}"
    );
}
