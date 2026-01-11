pub mod version_utils {
    use alpm::vercmp;
    use lenient_semver;
    use std::cmp::Ordering;

    // Helper function to compare versions using ALPM's vercmp for production consistency
    pub fn is_version_newer(aur_version: &str, local_version: &str) -> bool {
        // Use ALPM's vercmp which follows Arch Linux's official version comparison algorithm
        // vercmp returns Ordering::Greater if aur_version is newer than local_version
        match vercmp(aur_version, local_version) {
            Ordering::Greater => true,
            _ => false,
        }
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
                } else {
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
        let mut overwritten_text= text.to_string();

        text.split_whitespace().enumerate().for_each(|(element_index, element)| {
            let mut word: String = String::from(element);

            if !overrides[element_index].is_empty() {
                let color = overrides[element_index];

                if let Some(padding) = padding {
                    word.push_str(" ".repeat(padding[element_index % 4] - element.len()).as_str());
                    overwritten_text = overwritten_text.replace(element, &format!("<span color='#{}'>{}</span>", color, word));
                } else {
                    overwritten_text = overwritten_text.replace(element, &format!("<span color='#{}'>{}</span>", color, element));
                }
            } else {
                if let Some(padding) = padding {
                    word.push_str(" ".repeat(padding[element_index % 4] - element.len()).as_str());
                }
                overwritten_text = overwritten_text.replace(element, &word);
            }
        });

        overwritten_text
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
pub use version_utils::{highlight_semantic_version, is_version_newer, override_columns_from_packages};
