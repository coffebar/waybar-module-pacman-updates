use waybar_module_pacman_updates::{
    highlight_semantic_version, is_version_newer, override_columns_from_packages, pad_or_truncate,
};

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
    let overrides = ["", "", "", ""];
    let result = highlight_semantic_version(input, colors, false, overrides, None);

    assert!(result.contains("span color='#00ff00'"));
    assert!(result.contains("package 1.0.0 -> 1.1.0"));
}

#[test]
fn test_highlight_semantic_version_major() {
    let input = "package 1.0.0 -> 2.0.0".to_string();
    let colors = ["ff0000", "00ff00", "0000ff", "ff00ff", "ffffff"];
    let overrides = ["", "", "", ""];
    let result = highlight_semantic_version(input, colors, false, overrides, None);

    assert!(result.contains("span color='#ff0000'"));
}

#[test]
fn test_highlight_semantic_version_patch() {
    let input = "package 1.0.0 -> 1.0.1".to_string();
    let colors = ["ff0000", "00ff00", "0000ff", "ff00ff", "ffffff"];
    let overrides = ["", "", "", ""];
    let result = highlight_semantic_version(input, colors, false, overrides, None);

    assert!(result.contains("span color='#0000ff'"));
}

#[test]
fn test_highlight_semantic_version_invalid_format() {
    let input = "invalid format".to_string();
    let colors = ["ff0000", "00ff00", "0000ff", "ff00ff", "ffffff"];
    let overrides = ["", "", "", ""];
    let result = highlight_semantic_version(input, colors, false, overrides, None);

    assert_eq!(result, "invalid format");
    assert!(!result.contains("span"));
}

#[test]
fn test_highlight_semantic_version_with_padding() {
    let input = "pkg 1.0.0 -> 1.1.0".to_string();
    let colors = ["ff0000", "00ff00", "0000ff", "ff00ff", "ffffff"];
    let padding = Some([10, 10, 10, 10]);
    let input_len = input.len();
    let overrides = ["", "", "", ""];
    let result = highlight_semantic_version(input, colors, false, overrides, padding);

    assert!(result.contains("span color='#00ff00'"));
    assert!(result.len() > input_len); // Should be padded
}

#[test]
fn test_pad_or_truncate_pads_to_width() {
    assert_eq!(pad_or_truncate("1.0.0", 8), "1.0.0   ");
    assert_eq!(pad_or_truncate("1.0.0", 5), "1.0.0");
    assert_eq!(pad_or_truncate("", 3), "   ");
}

#[test]
fn test_pad_or_truncate_truncates_to_width() {
    assert_eq!(pad_or_truncate("1.0.0.longsuffix", 10), "1.0.0.l...");
    assert_eq!(pad_or_truncate("1.0.0", 4), "1...");
}

#[test]
fn test_pad_or_truncate_multibyte() {
    // Cutting by byte offset here would panic inside the arrow character
    let word = "1.0.0→beta.longsuffix";
    for width in 0..word.chars().count() + 3 {
        assert_eq!(
            pad_or_truncate(word, width).chars().count(),
            width,
            "width {} not respected",
            width
        );
    }
    assert_eq!(pad_or_truncate(word, 7), "1.0....");
}

#[test]
fn test_pad_or_truncate_narrow_widths_never_overflow() {
    // Widths below 4 have no room for the "..." marker and must hard truncate
    for width in 0..=3 {
        let result = pad_or_truncate("1.0.0.longsuffix", width);
        assert_eq!(result.chars().count(), width, "width {} overflowed", width);
    }
}

#[test]
fn test_highlight_semantic_version_respects_narrow_padding() {
    let input = "package-name 1.0.0.longsuffix -> 2.0.0.longsuffix".to_string();
    let colors = ["ff0000", "00ff00", "0000ff", "ff00ff", "ffffff"];
    let overrides = ["", "", "", ""];
    let padding = [12, 7, 2, 7];
    let result = highlight_semantic_version(input, colors, false, overrides, Some(padding));

    // Drop the surrounding pango markup, if any, to inspect the bare columns
    let line = match result.strip_prefix("<span") {
        Some(rest) => rest
            .split_once('>')
            .map(|(_, body)| body)
            .unwrap_or(&result)
            .trim_end_matches("</span>"),
        None => &result,
    };
    for (index, column) in line.split(' ').enumerate() {
        assert!(
            column.chars().count() <= padding[index % 4],
            "column {} exceeds its budget: {:?}",
            index,
            column
        );
    }
}

#[test]
fn test_overwrite_columns() {
    let input = "pkg 1.0.0 -> 1.1.0".to_string();
    let overrides = ["808080", "dcdcdc", "d3d3d3", "c0c0c0"];
    let result = override_columns_from_packages(input, overrides, None);

    assert!(result.contains("span color='#808080'>pkg"));
    assert!(result.contains("span color='#dcdcdc'>1.0.0"));
    assert!(result.contains("span color='#d3d3d3'>->"));
    assert!(result.contains("span color='#c0c0c0'>1.1.0"));
}

#[test]
fn test_overwrite_columns_invalid_format() {
    let input = "invalid format".to_string();
    let overrides = ["808080", "dcdcdc", "d3d3d3", "c0c0c0"];
    let result = override_columns_from_packages(input, overrides, None);

    assert_eq!(
        result,
        "<span color='#808080'>invalid</span> <span color='#dcdcdc'>format</span>"
    )
}

#[test]
fn test_overwrite_columns_with_padding() {
    let input = "pkg 1.0.0 -> 1.1.0".to_string();
    let overrides = ["808080", "dcdcdc", "d3d3d3", "c0c0c0"];
    let padding = Some([10, 10, 10, 10]);
    let input_len = input.len();
    let result = override_columns_from_packages(input, overrides, padding);

    assert!(result.contains("span color='#808080'>pkg"));
    assert!(result.contains("span color='#dcdcdc'>1.0.0"));
    assert!(result.contains("span color='#d3d3d3'>->"));
    assert!(result.contains("span color='#c0c0c0'>1.1.0"));
    assert!(result.len() > input_len)
}
