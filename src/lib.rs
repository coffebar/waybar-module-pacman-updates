pub mod version_utils {
    use alpm::vercmp;
    use lenient_semver;
    use std::cmp::Ordering;

    // Helper function to compare versions using ALPM's vercmp for production consistency
    pub fn is_version_newer(aur_version: &str, local_version: &str) -> bool {
        // Use ALPM's vercmp which follows Arch Linux's official version comparison algorithm
        // vercmp returns Ordering::Greater if aur_version is newer than local_version
        matches!(vercmp(aur_version, local_version), Ordering::Greater)
    }

    pub fn highlight_semantic_version(
        packages: String,
        colors: [&str; 5],
        override_colors: bool,
        overrides: [&str; 4],
        padding: Option<[usize; 4]>,
    ) -> String {
        packages
            .lines()
            .map(|package| {
                let fragments = package.split_whitespace().collect::<Vec<_>>();
                let mut text = package.to_string();

                if override_colors {
                    text = override_columns(text, overrides, padding);
                } else if let Some(padding) = padding {
                    text = fragments
                        .iter()
                        .enumerate()
                        .map(|(index, word)| {
                            let segment_padding = padding[index % 4];
                            if word.len() <= segment_padding {
                                word.to_string() + " ".repeat(segment_padding - word.len()).as_str()
                            } else {
                                word[..segment_padding.saturating_sub(3)].to_string() + "..."
                            }
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

    pub fn override_columns(
        text: String,
        overrides: [&str; 4],
        padding: Option<[usize; 4]>,
    ) -> String {
        text.split_whitespace()
            .enumerate()
            .map(|(element_index, element)| {
                // Apply padding if specified
                let padded_element = if let Some(padding) = padding {
                    let segment_padding = padding[element_index % 4];
                    if element.len() <= segment_padding {
                        format!(
                            "{}{}",
                            element,
                            " ".repeat(segment_padding - element.len())
                        )
                    } else {
                        element[..segment_padding.saturating_sub(3)].to_string() + "..."
                    }
                } else {
                    element.to_string()
                };

                // Apply color override if specified
                if !overrides[element_index].is_empty() {
                    format!(
                        "<span color='#{}'>{}</span>",
                        overrides[element_index], padded_element
                    )
                } else {
                    padded_element
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn override_columns_from_packages(
        packages: String,
        overrides: [&str; 4],
        padding: Option<[usize; 4]>,
    ) -> String {
        packages
            .lines()
            .map(|package| {
                let text = package.to_string();
                override_columns(text, overrides, padding)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// Re-export for easier access
pub use version_utils::{
    highlight_semantic_version, is_version_newer, override_columns_from_packages,
};
