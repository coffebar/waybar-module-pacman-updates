use anyhow::Result;
use clap::Parser;
use either::Either;
use hex_color::HexColor;
use tokio::select;
use tokio::time::{interval, Duration};
use ureq::json;
use waybar_module_pacman_updates::{AURepo, IsPackageRepo, OfficialRepo, Package, UpdateType};

#[derive(Debug)]
struct ColorScheme {
    major: HexColor,
    minor: HexColor,
    patch: HexColor,
    pre: HexColor,
    other: HexColor,
}

impl ColorScheme {
    fn default() -> Self {
        Self {
            major: HexColor::RED,
            minor: HexColor::GREEN,
            patch: HexColor::BLUE,
            pre: HexColor::MAGENTA,
            other: HexColor::WHITE,
        }
    }

    fn get_color(&self, update_type: &UpdateType) -> &HexColor {
        match update_type {
            UpdateType::Major => &self.major,
            UpdateType::Minor => &self.minor,
            UpdateType::Patch => &self.patch,
            UpdateType::Pre => &self.pre,
            UpdateType::Other => &self.other,
        }
    }

    fn from_cli(colors_str: &str) -> Self {
        let mut scheme = Self::default();
        let cli_colors: Vec<&str> = colors_str.split(',').collect();

        if cli_colors.len() != 5 {
            eprintln!(
                "Expected 5 colors (major,minor,patch,pre,other), got {} — using defaults.",
                cli_colors.len()
            );
            return scheme;
        }

        let mut targets = [
            &mut scheme.major,
            &mut scheme.minor,
            &mut scheme.patch,
            &mut scheme.pre,
            &mut scheme.other,
        ];

        for (input, target) in cli_colors.iter().zip(targets.iter_mut()) {
            let color_str = input.trim();
            if color_str.is_empty() {
                eprintln!("Empty color value detected — using default for this entry.");
                continue;
            }

            let color_with_hash = if color_str.starts_with('#') {
                color_str.to_string()
            } else {
                format!("#{color_str}")
            };

            match HexColor::parse(&color_with_hash) {
                Ok(parsed) => **target = parsed,
                Err(_) => eprintln!("Invalid color '{}', using default.", color_str),
            }
        }

        scheme
    }
}

#[derive(Parser, Debug)]
#[command(name = "waybar-pacman-updates")]
#[command(about = "Monitor pacman updates for Waybar", long_about = None)]
struct CliArgs {
    /// Set the interval between local updates (in seconds)
    #[arg(long, default_value = "5", value_parser = clap::value_parser!(u64).range(1..))]
    interval_seconds: u64,

    /// Set the interval between network updates (in seconds)
    #[arg(long, default_value = "300", value_parser = clap::value_parser!(u64).range(1..))]
    network_interval_seconds: u64,

    /// Don't output anything when there are zero updates
    #[arg(long)]
    no_zero_output: bool,

    /// Exclude AUR packages from updates
    #[arg(long)]
    no_aur: bool,

    /// Align tooltip in columns with specified font
    #[arg(long, value_name = "FONT",num_args=0..=1, default_missing_value = "monospace")]
    tooltip_align_columns: Option<String>,

    /// Color semantic version updates with custom colors (comma-separated: major,minor,patch,pre,other)
    #[arg(
    long,
    value_name = "COLORS",
    num_args(0..=1),
    default_missing_value = "ff0000,00ff00,0000ff,ff00ff,ffffff"
    )]
    color_semver_updates: Option<String>,
}

#[derive(Debug)]
struct AppContext {
    interval_seconds: u64,
    network_interval_seconds: u64,
    no_aur: bool,
    no_zero: bool,
    tooltip_align: bool,
    tooltip_font: String,
    color_semver_updates: bool,
    colors: ColorScheme,

    official_repo: OfficialRepo,
    au_repo: AURepo,
}

impl AppContext {
    fn local_updates(&mut self) {
        self.official_repo.local_updates();
        if !self.no_aur {
            self.au_repo.local_updates();
        }
    }

    fn sync_updates(&mut self) {
        self.official_repo.sync_updates();
        if !self.no_aur {
            self.au_repo.sync_updates();
        }
    }

    fn packages(&self) -> impl Iterator<Item = &Package> + '_ {
        if self.no_aur {
            Either::Left(self.official_repo.packages())
        } else {
            Either::Right(self.official_repo.packages().chain(self.au_repo.packages()))
        }
    }
    fn tooltip(&self) -> String {
        let pkgs: Vec<_> = self.packages().collect();
        if pkgs.is_empty() {
            return "System updated".to_string();
        }

        let mut tooltip = String::new();
        let (name_max_len, old_version_max_len) = if self.tooltip_align {
            let name_max = pkgs.iter().map(|p| p.name.len()).max().unwrap_or(0);
            let old_max = pkgs.iter().map(|p| p.old_version.len()).max().unwrap_or(0);

            (name_max, old_max)
        } else {
            (0, 0)
        };

        for package in pkgs {
            let package_line = if self.tooltip_align {
                format!(
                    "{:<name_max_len$} {:<old_version_max_len$} -> {}",
                    package.name, package.old_version, package.new_version,
                )
            } else {
                format!(
                    "{} {} -> {}",
                    package.name, package.old_version, package.new_version
                )
            };

            let formatted_line = if self.color_semver_updates {
                format!(
                    "<span color='{}'>{}</span>",
                    self.colors.get_color(&package.update_type).display_rgb(),
                    package_line
                )
            } else {
                package_line
            };
            tooltip.push_str(&formatted_line);
            tooltip.push('\n');
        }

        //Remove last \n
        tooltip.pop();

        format!(
            "<span font-family='{}'>{}</span>",
            self.tooltip_font, tooltip
        )
    }

    fn waybar_output(&self) -> String {
        let count_pkg = self.packages().count();
        if count_pkg == 0 && self.no_zero {
            json!({
                "text": "",
                "tooltip": self.tooltip(),
                "class": "updated",
                "alt": "updated"
            })
            .to_string()
        } else {
            json!({
                "text": count_pkg.to_string(),
                "tooltip": self.tooltip(),
                "class": if count_pkg > 0 { "has-updates" } else { "updated" },
                "alt": if count_pkg > 0 { "has-updates" } else { "updated" }
            })
            .to_string()
        }
    }
}

impl Default for AppContext {
    fn default() -> Self {
        Self {
            interval_seconds: 5,
            network_interval_seconds: 300,
            no_aur: false,
            no_zero: false,
            tooltip_align: false,
            tooltip_font: "monospace".to_string(),
            color_semver_updates: false,
            colors: ColorScheme::default(),
            official_repo: OfficialRepo::default(),
            au_repo: AURepo::default(),
        }
    }
}

impl From<CliArgs> for AppContext {
    fn from(cli: CliArgs) -> Self {
        let mut app_ctx = AppContext {
            no_aur: cli.no_aur,
            no_zero: cli.no_zero_output,
            ..Default::default()
        };

        if cli.interval_seconds > cli.network_interval_seconds {
            eprintln!(
                "--interval-seconds must be less than or equal to --network-interval-seconds\nUsing default value instead."
            );
        } else {
            app_ctx.interval_seconds = cli.interval_seconds;
            app_ctx.network_interval_seconds = cli.network_interval_seconds;
        }

        if let Some(font) = cli.tooltip_align_columns {
            app_ctx.tooltip_align = true;
            app_ctx.tooltip_font = font;
        }

        if let Some(colors_str) = cli.color_semver_updates {
            app_ctx.color_semver_updates = true;
            app_ctx.colors = ColorScheme::from_cli(&colors_str);
        }

        app_ctx
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CliArgs::parse();
    let mut app_ctx = AppContext::from(cli);

    // First output to display something
    println!("{}", app_ctx.waybar_output());

    // Then start to sync
    app_ctx.sync_updates();
    println!("{}", app_ctx.waybar_output());

    let mut local_interval = interval(Duration::from_secs(app_ctx.interval_seconds as u64));
    let mut network_interval =
        interval(Duration::from_secs(app_ctx.network_interval_seconds as u64));

    local_interval.tick().await;
    network_interval.tick().await;

    loop {
        select! {
            //Sync update executed first in case of local and sync updates at the same time
            biased;

            _ = network_interval.tick() => {
                app_ctx.sync_updates();
                println!("{}", app_ctx.waybar_output());

                // Reset local interval after syncn no double update
                local_interval.reset();
            }
            _ = local_interval.tick() => {
                app_ctx.local_updates();
                println!("{}", app_ctx.waybar_output());
            }
        }
    }
}
