#[test]
fn gitignore_protects_local_key() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let gi = std::fs::read_to_string(root.join(".gitignore")).expect(".gitignore");
    assert!(
        gi.lines().any(|l| l.trim() == ".parakeys/local.key" || l.trim().ends_with("local.key")),
        ".gitignore must ignore .parakeys/local.key"
    );
    let readme = std::fs::read_to_string(root.join("README.md")).expect("README");
    assert!(readme.contains("vault.enc"), "README must document vault.enc");
    assert!(readme.contains("local.key"), "README must document local.key never commit");
    assert!(readme.contains("NEVER commit") || readme.contains("never commit") || readme.contains("NEVER"), "README must say never commit local key");
}
