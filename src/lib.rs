pub mod version_utils {
    use lenient_semver;

    // Helper function to compare versions properly
    pub fn is_version_newer(aur_version: &str, local_version: &str) -> bool {
        // Handle git packages with revision numbers (e.g., "r89.fe26a90-1" or "-r89.fe26a90-1")
        let extract_git_revision = |version: &str| -> Option<u32> {
            if let Some(r_pos) = version.find('r') {
                let after_r = &version[r_pos + 1..];
                if let Some(dot_pos) = after_r.find('.') {
                    if let Ok(rev) = after_r[..dot_pos].parse() {
                        return Some(rev);
                    }
                }
            }
            None
        };

        // If both versions have git revision numbers, compare them
        if let (Some(aur_rev), Some(local_rev)) = (
            extract_git_revision(aur_version),
            extract_git_revision(local_version),
        ) {
            return aur_rev > local_rev;
        }

        // Fall back to semantic version comparison
        match (
            lenient_semver::parse(aur_version),
            lenient_semver::parse(local_version),
        ) {
            (Ok(aur_semver), Ok(local_semver)) => aur_semver > local_semver,
            // If semantic parsing fails, fall back to string comparison
            // but only show as update if strings are different (conservative approach)
            _ => aur_version != local_version && aur_version > local_version,
        }
    }

    pub fn highlight_semantic_version(
        packages: String,
        colors: [&str; 5],
        padding: Option<[usize; 4]>,
    ) -> String {
        packages
            .lines()
            .map(|package| {
                let fragments = package.split_whitespace().collect::<Vec<_>>();

                let mut text = package.to_string();

                if let Some(padding) = padding {
                    text = fragments
                        .iter()
                        .enumerate()
                        .map(|(index, word)| {
                            word.to_string() + " ".repeat(padding[index % 4] - word.len()).as_str()
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                }

                if fragments.len() != 4 {
                    // unknown format, so we skip formatting
                    return text;
                }

                let (Ok(old_version), Ok(new_version)) = (
                    lenient_semver::parse(fragments[1]),
                    lenient_semver::parse(fragments[3]),
                ) else {
                    return text;
                };

                let color = {
                    if new_version.major > old_version.major {
                        colors[0]
                    } else if new_version.minor > old_version.minor {
                        colors[1]
                    } else if new_version.patch > old_version.patch {
                        colors[2]
                    } else if new_version.pre > old_version.pre {
                        colors[3]
                    } else {
                        colors[4]
                    }
                };

                format!("<span color='#{}'>{}</span>", color, text)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// Re-export for easier access
pub use version_utils::{highlight_semantic_version, is_version_newer};