use waybar_module_pacman_updates::{highlight_semantic_version, is_version_newer};

#[test]
fn test_version_comparison_semantic() {
    assert!(is_version_newer("1.2.1", "1.2.0"));
    assert!(is_version_newer("1.3.0", "1.2.9"));
    assert!(is_version_newer("2.0.0", "1.9.9"));
    assert!(!is_version_newer("1.2.0", "1.2.1"));
    assert!(!is_version_newer("1.2.0", "1.2.0"));
}

#[test]
fn test_version_comparison_git_revisions() {
    assert!(is_version_newer("r100.abc123-1", "r99.def456-1"));
    assert!(!is_version_newer("r99.abc123-1", "r100.def456-1"));
    assert!(!is_version_newer("r100.abc123-1", "r100.def456-1"));
    assert!(is_version_newer(
        "0.48.0.r62.gd775686-1",
        "0.47.0.r63.ccdddddd-2"
    ));
}

#[test]
fn test_version_comparison_mixed() {
    assert!(is_version_newer("1.2.1-r50.abc123", "1.2.0"));
    assert!(is_version_newer("2.0.0", "r100.abc123-1")); // ALPM: semantic version is newer than git revision
    assert!(!is_version_newer("r101.abc123-1", "1.2.0")); // ALPM: git revision is older than semantic version
}

#[test]
fn test_highlight_semantic_version_basic() {
    let input = "package 1.0.0 -> 1.1.0".to_string();
    let colors = ["ff0000", "00ff00", "0000ff", "ff00ff", "ffffff"];
    let result = highlight_semantic_version(input, colors, None);

    assert!(result.contains("span color='#00ff00'"));
    assert!(result.contains("package 1.0.0 -> 1.1.0"));
}

#[test]
fn test_highlight_semantic_version_major() {
    let input = "package 1.0.0 -> 2.0.0".to_string();
    let colors = ["ff0000", "00ff00", "0000ff", "ff00ff", "ffffff"];
    let result = highlight_semantic_version(input, colors, None);

    assert!(result.contains("span color='#ff0000'"));
}

#[test]
fn test_highlight_semantic_version_patch() {
    let input = "package 1.0.0 -> 1.0.1".to_string();
    let colors = ["ff0000", "00ff00", "0000ff", "ff00ff", "ffffff"];
    let result = highlight_semantic_version(input, colors, None);

    assert!(result.contains("span color='#0000ff'"));
}

#[test]
fn test_highlight_semantic_version_invalid_format() {
    let input = "invalid format".to_string();
    let colors = ["ff0000", "00ff00", "0000ff", "ff00ff", "ffffff"];
    let result = highlight_semantic_version(input, colors, None);

    assert_eq!(result, "invalid format");
    assert!(!result.contains("span"));
}

#[test]
fn test_highlight_semantic_version_with_padding() {
    let input = "pkg 1.0.0 -> 1.1.0".to_string();
    let colors = ["ff0000", "00ff00", "0000ff", "ff00ff", "ffffff"];
    let padding = Some([10, 10, 10, 10]);
    let input_len = input.len();
    let result = highlight_semantic_version(input, colors, padding);

    assert!(result.contains("span color='#00ff00'"));
    assert!(result.len() > input_len); // Should be padded
}