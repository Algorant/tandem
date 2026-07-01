use ratatui::style::{Color, Modifier, Style};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use super::super::{display_path, Workspace};

#[derive(Debug, Clone)]
pub(super) struct ThemeLoad {
    pub(super) theme: TuiTheme,
    pub(super) source: String,
    pub(super) warnings: Vec<String>,
}

#[derive(Debug, Default)]
struct UserThemeRegistry {
    themes: BTreeMap<String, UserTheme>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone)]
struct UserTheme {
    theme: TuiTheme,
    path: PathBuf,
}

#[derive(Debug, Clone)]
struct ResolvedTheme {
    theme: TuiTheme,
    source: String,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum StatusTone {
    Accent,
    Success,
    Warning,
    Error,
    Muted,
}

#[derive(Debug, Clone)]
pub(super) struct TuiTheme {
    name: String,
    colors: ThemeColors,
    priority: PriorityPalette,
    accord: AccordPalette,
    review: ReviewPalette,
    transparent_background: bool,
    no_color: bool,
}

#[derive(Debug, Clone, Copy)]
struct ThemeColors {
    background: Color,
    panel: Color,
    text: Color,
    muted: Color,
    accent: Color,
    success: Color,
    warning: Color,
    error: Color,
    border: Color,
    selected_bg: Color,
    selected_fg: Color,
}

#[derive(Debug, Clone, Copy)]
struct PriorityPalette {
    critical: Color,
    high: Color,
    medium: Color,
    low: Color,
    none: Color,
}

#[derive(Debug, Clone, Copy)]
struct AccordPalette {
    ready: Color,
    claimed: Color,
    delivered: Color,
    accepted: Color,
    rework: Color,
    failed: Color,
    blocked: Color,
    unknown: Color,
}

#[derive(Debug, Clone, Copy)]
struct ReviewPalette {
    not_ready: Color,
    pending: Color,
    accepted: Color,
    changes_requested: Color,
    rejected: Color,
    failed: Color,
    unknown: Color,
}

impl TuiTheme {
    pub(super) fn default_dark() -> Self {
        Self {
            name: "default-dark".to_string(),
            colors: ThemeColors {
                background: Color::Rgb(12, 14, 18),
                panel: Color::Rgb(22, 25, 31),
                text: Color::Rgb(229, 231, 235),
                muted: Color::Rgb(107, 114, 128),
                accent: Color::Rgb(56, 189, 248),
                success: Color::Rgb(74, 222, 128),
                warning: Color::Rgb(250, 204, 21),
                error: Color::Rgb(248, 113, 113),
                border: Color::Rgb(55, 65, 81),
                selected_bg: Color::Rgb(31, 41, 55),
                selected_fg: Color::Rgb(255, 255, 255),
            },
            priority: PriorityPalette {
                critical: Color::Rgb(239, 68, 68),
                high: Color::Rgb(248, 113, 113),
                medium: Color::Rgb(96, 165, 250),
                low: Color::Rgb(74, 222, 128),
                none: Color::Rgb(107, 114, 128),
            },
            accord: AccordPalette {
                ready: Color::Rgb(250, 204, 21),
                claimed: Color::Rgb(56, 189, 248),
                delivered: Color::Rgb(192, 132, 252),
                accepted: Color::Rgb(74, 222, 128),
                rework: Color::Rgb(251, 146, 60),
                failed: Color::Rgb(248, 113, 113),
                blocked: Color::Rgb(239, 68, 68),
                unknown: Color::Rgb(107, 114, 128),
            },
            review: ReviewPalette {
                not_ready: Color::Rgb(107, 114, 128),
                pending: Color::Rgb(250, 204, 21),
                accepted: Color::Rgb(74, 222, 128),
                changes_requested: Color::Rgb(251, 146, 60),
                rejected: Color::Rgb(248, 113, 113),
                failed: Color::Rgb(248, 113, 113),
                unknown: Color::Rgb(107, 114, 128),
            },
            transparent_background: false,
            no_color: false,
        }
    }

    pub(super) fn verdigris() -> Self {
        Self {
            name: "verdigris".to_string(),
            colors: ThemeColors {
                background: Color::Rgb(29, 32, 33),
                panel: Color::Rgb(34, 37, 38),
                text: Color::Rgb(235, 219, 178),
                muted: Color::Rgb(146, 131, 116),
                accent: Color::Rgb(142, 192, 124),
                success: Color::Rgb(142, 192, 124),
                warning: Color::Rgb(230, 191, 134),
                error: Color::Rgb(227, 111, 99),
                border: Color::Rgb(102, 92, 84),
                selected_bg: Color::Rgb(39, 42, 43),
                selected_fg: Color::Rgb(251, 241, 199),
            },
            priority: PriorityPalette {
                critical: Color::Rgb(227, 111, 99),
                high: Color::Rgb(227, 111, 99),
                medium: Color::Rgb(131, 165, 152),
                low: Color::Rgb(142, 192, 124),
                none: Color::Rgb(146, 131, 116),
            },
            accord: AccordPalette {
                ready: Color::Rgb(230, 191, 134),
                claimed: Color::Rgb(131, 165, 152),
                delivered: Color::Rgb(142, 192, 124),
                accepted: Color::Rgb(104, 157, 106),
                rework: Color::Rgb(230, 191, 134),
                failed: Color::Rgb(227, 111, 99),
                blocked: Color::Rgb(227, 111, 99),
                unknown: Color::Rgb(146, 131, 116),
            },
            review: ReviewPalette {
                not_ready: Color::Rgb(112, 118, 74),
                pending: Color::Rgb(230, 191, 134),
                accepted: Color::Rgb(142, 192, 124),
                changes_requested: Color::Rgb(230, 191, 134),
                rejected: Color::Rgb(227, 111, 99),
                failed: Color::Rgb(227, 111, 99),
                unknown: Color::Rgb(146, 131, 116),
            },
            transparent_background: false,
            no_color: false,
        }
    }

    fn built_in(name: &str) -> Option<Self> {
        match normalized(name).as_str() {
            "default-dark" => Some(Self::default_dark()),
            "verdigris" => Some(Self::verdigris()),
            _ => None,
        }
    }

    fn built_in_names() -> &'static [&'static str] {
        &["default-dark", "verdigris"]
    }

    pub(super) fn load_for_workspace(workspace: &Workspace) -> ThemeLoad {
        let no_color =
            env::var_os("NO_COLOR").is_some() || env::var_os("TANDEM_NO_COLOR").is_some();
        Self::load_for_workspace_with_options(
            workspace,
            user_theme_dir_from_env(),
            user_config_path_from_env(),
            no_color,
        )
    }

    fn load_for_workspace_with_options(
        workspace: &Workspace,
        user_theme_dir: Option<PathBuf>,
        user_config_path: Option<PathBuf>,
        no_color: bool,
    ) -> ThemeLoad {
        let mut theme = if no_color {
            Self::no_color()
        } else {
            Self::default_dark()
        };
        let mut source = if no_color {
            "built-in terminal/no-color".to_string()
        } else {
            format!("built-in {}", theme.name)
        };
        let mut warnings = Vec::new();
        let user_themes = if no_color {
            UserThemeRegistry::default()
        } else {
            match user_theme_dir.as_deref() {
                Some(dir) => load_user_themes(dir),
                None => UserThemeRegistry::default(),
            }
        };
        warnings.extend(user_themes.warnings.clone());

        if let Some(user_config_path) = user_config_path.as_deref() {
            apply_theme_config_file(
                &mut theme,
                &mut source,
                &mut warnings,
                user_config_path,
                &user_themes,
                user_theme_dir.as_deref(),
                no_color,
            );
        }

        let workspace_theme = workspace.config_path.with_file_name("theme.toml");
        apply_theme_config_file(
            &mut theme,
            &mut source,
            &mut warnings,
            &workspace_theme,
            &user_themes,
            user_theme_dir.as_deref(),
            no_color,
        );

        ThemeLoad {
            theme,
            source,
            warnings,
        }
    }

    pub(super) fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn source_label<'a>(&'a self, source: &'a str) -> &'a str {
        if source.is_empty() {
            self.name()
        } else {
            source
        }
    }

    pub(super) fn app_style(&self) -> Style {
        self.style(
            self.colors.text,
            self.background_option(self.colors.background),
            Modifier::empty(),
        )
    }

    pub(super) fn panel_style(&self) -> Style {
        self.style(
            self.colors.text,
            self.background_option(self.colors.panel),
            Modifier::empty(),
        )
    }

    pub(super) fn title_style(&self) -> Style {
        self.style(self.colors.accent, self.panel_background(), Modifier::BOLD)
    }

    pub(super) fn text_style(&self) -> Style {
        self.style(self.colors.text, self.panel_background(), Modifier::empty())
    }

    pub(super) fn muted_style(&self) -> Style {
        self.style(
            self.colors.muted,
            self.panel_background(),
            Modifier::empty(),
        )
    }

    pub(super) fn label_style(&self) -> Style {
        self.style(self.colors.muted, self.panel_background(), Modifier::BOLD)
    }

    pub(super) fn border_style(&self, active: bool) -> Style {
        self.style(
            if active {
                self.colors.accent
            } else {
                self.colors.border
            },
            self.panel_background(),
            Modifier::empty(),
        )
    }

    pub(super) fn selected_style(&self) -> Style {
        if self.no_color {
            return Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED);
        }
        self.style(
            self.colors.selected_fg,
            Some(self.colors.selected_bg),
            Modifier::BOLD,
        )
    }

    pub(super) fn board_selected_style(&self) -> Style {
        if self.no_color {
            return Style::default().add_modifier(Modifier::BOLD);
        }
        Style::default()
    }

    pub(super) fn board_selected_title_style(&self) -> Style {
        self.style(self.colors.accent, self.panel_background(), Modifier::BOLD)
    }

    pub(super) fn board_doc_type_style(&self) -> Style {
        self.muted_style()
    }

    pub(super) fn badge_label(&self, label: &str) -> String {
        format!(" {label:<4} ")
    }

    pub(super) fn priority_chip_style(&self, priority: &str) -> Style {
        let color = match normalized(priority).as_str() {
            "critical" | "urgent" => self.priority.critical,
            "high" => self.priority.high,
            "medium" | "med" => self.priority.medium,
            "low" => self.priority.low,
            "" | "-" | "none" => self.priority.none,
            _ => self.priority.none,
        };
        self.chip_style(color)
    }

    pub(super) fn accord_chip_style(&self, status: &str) -> Style {
        let color = match normalized(status).as_str() {
            "ready" => self.accord.ready,
            "claimed" => self.accord.claimed,
            "delivered" => self.accord.delivered,
            "accepted" => self.accord.accepted,
            "rework" => self.accord.rework,
            "failed" => self.accord.failed,
            "blocked" => self.accord.blocked,
            _ => self.accord.unknown,
        };
        self.chip_style(color)
    }

    pub(super) fn review_chip_style(&self, status: &str) -> Style {
        let color = match normalized(status).as_str() {
            "not-ready" => self.review.not_ready,
            "pending" => self.review.pending,
            "accepted" => self.review.accepted,
            "changes-requested" => self.review.changes_requested,
            "rejected" => self.review.rejected,
            "failed" => self.review.failed,
            _ => self.review.unknown,
        };
        self.chip_style(color)
    }

    pub(super) fn progress_chip_style(&self, tone: StatusTone) -> Style {
        let color = match tone {
            StatusTone::Accent => self.colors.accent,
            StatusTone::Success => self.colors.success,
            StatusTone::Warning => self.colors.warning,
            StatusTone::Error => self.colors.error,
            StatusTone::Muted => self.colors.muted,
        };
        self.chip_style(color)
    }

    pub(super) fn tab_style(&self) -> Style {
        self.muted_style()
    }

    pub(super) fn tab_selected_style(&self) -> Style {
        if self.no_color {
            return Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED);
        }
        self.style(
            self.colors.selected_fg,
            Some(self.colors.selected_bg),
            Modifier::BOLD,
        )
    }

    pub(super) fn state_tab_selected_style(&self) -> Style {
        self.style(
            self.colors.accent,
            self.panel_background(),
            Modifier::BOLD | Modifier::UNDERLINED,
        )
    }

    pub(super) fn rule_category_style(&self, category: &str, selected: bool) -> Style {
        if self.no_color {
            return if selected {
                Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default()
            };
        }
        let color = match normalized(category).as_str() {
            "never" => self.colors.error,
            "prefer" => self.colors.warning,
            "context" => Color::Rgb(192, 132, 252),
            _ => self.colors.muted,
        };
        if selected {
            if normalized(category).as_str() == "always" {
                self.style(
                    self.colors.selected_fg,
                    Some(self.colors.selected_bg),
                    Modifier::BOLD,
                )
            } else {
                self.style(self.colors.background, Some(color), Modifier::BOLD)
            }
        } else {
            self.style(color, self.panel_background(), Modifier::empty())
        }
    }

    pub(super) fn status_style(&self, tone: StatusTone) -> Style {
        let color = match tone {
            StatusTone::Accent => self.colors.accent,
            StatusTone::Success => self.colors.success,
            StatusTone::Warning => self.colors.warning,
            StatusTone::Error => self.colors.error,
            StatusTone::Muted => self.colors.muted,
        };
        self.style(color, self.panel_background(), Modifier::empty())
    }

    pub(super) fn priority_style(&self, priority: &str) -> Style {
        let color = match normalized(priority).as_str() {
            "critical" | "urgent" => self.priority.critical,
            "high" => self.priority.high,
            "medium" | "med" => self.priority.medium,
            "low" => self.priority.low,
            "" | "-" | "none" => self.priority.none,
            _ => self.priority.none,
        };
        let modifier = match normalized(priority).as_str() {
            "critical" | "urgent" | "high" => Modifier::BOLD,
            _ => Modifier::empty(),
        };
        self.style(color, self.panel_background(), modifier)
    }

    pub(super) fn accord_style(&self, status: &str) -> Style {
        let color = match normalized(status).as_str() {
            "ready" => self.accord.ready,
            "claimed" => self.accord.claimed,
            "delivered" => self.accord.delivered,
            "accepted" => self.accord.accepted,
            "rework" => self.accord.rework,
            "failed" => self.accord.failed,
            "blocked" => self.accord.blocked,
            _ => self.accord.unknown,
        };
        self.style(color, self.panel_background(), Modifier::empty())
    }

    pub(super) fn review_style(&self, status: &str) -> Style {
        let color = match normalized(status).as_str() {
            "not-ready" => self.review.not_ready,
            "pending" => self.review.pending,
            "accepted" => self.review.accepted,
            "changes-requested" => self.review.changes_requested,
            "rejected" => self.review.rejected,
            "failed" => self.review.failed,
            _ => self.review.unknown,
        };
        self.style(color, self.panel_background(), Modifier::empty())
    }

    pub(super) fn markdown_heading_style(&self) -> Style {
        self.title_style()
    }

    pub(super) fn markdown_list_style(&self) -> Style {
        self.muted_style()
    }

    pub(super) fn markdown_code_style(&self) -> Style {
        self.status_style(StatusTone::Warning)
    }

    fn style(&self, fg: Color, bg: Option<Color>, modifier: Modifier) -> Style {
        if self.no_color {
            return Style::default().add_modifier(modifier);
        }
        let style = Style::default().fg(fg).add_modifier(modifier);
        match bg {
            Some(bg) => style.bg(bg),
            None => style,
        }
    }

    fn background_option(&self, color: Color) -> Option<Color> {
        if self.transparent_background {
            None
        } else {
            Some(color)
        }
    }

    fn panel_background(&self) -> Option<Color> {
        self.background_option(self.colors.panel)
    }

    fn chip_style(&self, color: Color) -> Style {
        if self.no_color {
            return Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED);
        }
        self.style(self.colors.background, Some(color), Modifier::BOLD)
    }

    fn no_color() -> Self {
        let mut theme = Self::default_dark();
        theme.name = "terminal/no-color".to_string();
        theme.colors = ThemeColors {
            background: Color::Reset,
            panel: Color::Reset,
            text: Color::Reset,
            muted: Color::Reset,
            accent: Color::Reset,
            success: Color::Reset,
            warning: Color::Reset,
            error: Color::Reset,
            border: Color::Reset,
            selected_bg: Color::Reset,
            selected_fg: Color::Reset,
        };
        theme.priority = PriorityPalette {
            critical: Color::Reset,
            high: Color::Reset,
            medium: Color::Reset,
            low: Color::Reset,
            none: Color::Reset,
        };
        theme.accord = AccordPalette {
            ready: Color::Reset,
            claimed: Color::Reset,
            delivered: Color::Reset,
            accepted: Color::Reset,
            rework: Color::Reset,
            failed: Color::Reset,
            blocked: Color::Reset,
            unknown: Color::Reset,
        };
        theme.review = ReviewPalette {
            not_ready: Color::Reset,
            pending: Color::Reset,
            accepted: Color::Reset,
            changes_requested: Color::Reset,
            rejected: Color::Reset,
            failed: Color::Reset,
            unknown: Color::Reset,
        };
        theme.no_color = true;
        theme
    }

    fn set_transparent_background(&mut self, enabled: bool) {
        self.transparent_background = enabled;
    }

    fn apply_theme_content(&mut self, content: &str) -> Vec<String> {
        let mut section = String::new();
        let mut warnings = Vec::new();

        for (index, raw_line) in content.lines().enumerate() {
            let line_number = index + 1;
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if line.starts_with('[') {
                if line.ends_with(']') {
                    section = line[1..line.len() - 1].trim().to_ascii_lowercase();
                } else {
                    warnings.push(format!(
                        "Theme warning line {line_number}: malformed section header"
                    ));
                }
                continue;
            }

            let Some((raw_key, raw_value)) = line.split_once('=') else {
                warnings.push(format!(
                    "Theme warning line {line_number}: expected key = value"
                ));
                continue;
            };
            let key = raw_key.trim().to_ascii_lowercase();
            if key.is_empty() {
                warnings.push(format!("Theme warning line {line_number}: empty key"));
                continue;
            }
            let value = parse_value(raw_value);

            if section.is_empty() {
                match key.as_str() {
                    "name" => {
                        if !value.trim().is_empty() {
                            self.name = value;
                        }
                    }
                    "theme" | "base" | "builtin" | "extends" => {}
                    "transparent_background" | "transparent-background" | "transparent" => {
                        match parse_bool(&value) {
                            Some(enabled) => self.set_transparent_background(enabled),
                            None => warnings.push(format!(
                                "Theme warning line {line_number}: invalid boolean for `{key}`: use true or false"
                            )),
                        }
                    }
                    _ => warnings.push(format!(
                        "Theme warning line {line_number}: unknown root key `{key}`"
                    )),
                }
                continue;
            }

            match parse_color(&value) {
                Ok(color) => {
                    if !self.apply_color(&section, &key, color) {
                        warnings.push(format!(
                            "Theme warning line {line_number}: unknown theme key `{section}.{key}`"
                        ));
                    }
                }
                Err(message) => warnings.push(format!(
                    "Theme warning line {line_number}: invalid color for `{section}.{key}`: {message}"
                )),
            }
        }

        warnings
    }

    fn apply_color(&mut self, section: &str, key: &str, color: Color) -> bool {
        match (section, key) {
            ("colors", "background") => self.colors.background = color,
            ("colors", "panel") => self.colors.panel = color,
            ("colors", "text") => self.colors.text = color,
            ("colors", "muted") => self.colors.muted = color,
            ("colors", "accent") => self.colors.accent = color,
            ("colors", "success") => self.colors.success = color,
            ("colors", "warning") => self.colors.warning = color,
            ("colors", "error") => self.colors.error = color,
            ("colors", "border") => self.colors.border = color,
            ("colors", "selected_bg") | ("colors", "selected-bg") => {
                self.colors.selected_bg = color
            }
            ("colors", "selected_fg") | ("colors", "selected-fg") => {
                self.colors.selected_fg = color
            }
            ("priority", "critical") => self.priority.critical = color,
            ("priority", "high") => self.priority.high = color,
            ("priority", "medium") => self.priority.medium = color,
            ("priority", "low") => self.priority.low = color,
            ("priority", "none") | ("priority", "unknown") => self.priority.none = color,
            ("badges.accord", "ready") => self.accord.ready = color,
            ("badges.accord", "claimed") => self.accord.claimed = color,
            ("badges.accord", "delivered") => self.accord.delivered = color,
            ("badges.accord", "accepted") => self.accord.accepted = color,
            ("badges.accord", "rework") => self.accord.rework = color,
            ("badges.accord", "failed") => self.accord.failed = color,
            ("badges.accord", "blocked") => self.accord.blocked = color,
            ("badges.accord", "unknown") => self.accord.unknown = color,
            ("badges.review", "not-ready") | ("badges.review", "not_ready") => {
                self.review.not_ready = color
            }
            ("badges.review", "pending") => self.review.pending = color,
            ("badges.review", "accepted") => self.review.accepted = color,
            ("badges.review", "changes-requested") | ("badges.review", "changes_requested") => {
                self.review.changes_requested = color
            }
            ("badges.review", "rejected") => self.review.rejected = color,
            ("badges.review", "failed") => self.review.failed = color,
            ("badges.review", "unknown") => self.review.unknown = color,
            _ => return false,
        }
        true
    }
}

pub(super) fn user_theme_dir_from_env() -> Option<PathBuf> {
    user_config_dir_from_env().map(|dir| dir.join("themes"))
}

pub(super) fn user_config_path_from_env() -> Option<PathBuf> {
    user_config_dir_from_env().map(|dir| dir.join("config.toml"))
}

fn user_config_dir_from_env() -> Option<PathBuf> {
    if let Some(xdg_config_home) = env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty())
    {
        return Some(PathBuf::from(xdg_config_home).join("tandem"));
    }

    env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(|home| PathBuf::from(home).join(".config").join("tandem"))
}

fn load_user_themes(dir: &Path) -> UserThemeRegistry {
    let mut registry = UserThemeRegistry::default();
    if !dir.exists() {
        return registry;
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            registry.warnings.push(format!(
                "Theme warning: could not read user theme directory {}: {error}",
                display_path(dir)
            ));
            return registry;
        }
    };

    let mut paths = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.extension().and_then(|value| value.to_str()) == Some("toml") {
                    paths.push(path);
                }
            }
            Err(error) => registry.warnings.push(format!(
                "Theme warning: could not inspect user theme in {}: {error}",
                display_path(dir)
            )),
        }
    }
    paths.sort();

    for path in paths {
        let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
            registry.warnings.push(format!(
                "Theme warning: could not determine user theme name for {}",
                display_path(&path)
            ));
            continue;
        };
        let fallback_name = stem.trim().to_string();
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) => {
                registry.warnings.push(format!(
                    "Theme warning: could not read user theme {}: {error}",
                    display_path(&path)
                ));
                continue;
            }
        };

        let explicit_name = parse_theme_name(&content);
        let theme_name = explicit_name
            .clone()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or(fallback_name);
        let key = normalized(&theme_name);
        if key.is_empty() {
            registry.warnings.push(format!(
                "Theme warning: user theme {} has an empty name",
                display_path(&path)
            ));
            continue;
        }
        if registry.themes.contains_key(&key) {
            registry.warnings.push(format!(
                "Theme warning: duplicate user theme `{theme_name}` in {}; keeping the first definition",
                display_path(&path)
            ));
            continue;
        }

        let mut theme = match parse_theme_base(&content) {
            Some(base) => match resolve_theme_reference(&base, &registry) {
                Some(resolved) => resolved.theme,
                None => {
                    registry.warnings.push(format!(
                        "Theme warning: unknown base `{base}` in user theme {}; using default-dark",
                        display_path(&path)
                    ));
                    TuiTheme::default_dark()
                }
            },
            None => TuiTheme::default_dark(),
        };
        registry.warnings.extend(prefix_theme_warnings(
            &path,
            theme.apply_theme_content(&content),
        ));
        if explicit_name.is_none() {
            theme.name = theme_name.clone();
        }

        registry.themes.insert(key, UserTheme { theme, path });
    }

    registry
}

fn apply_theme_config_file(
    theme: &mut TuiTheme,
    source: &mut String,
    warnings: &mut Vec<String>,
    path: &Path,
    user_themes: &UserThemeRegistry,
    user_theme_dir: Option<&Path>,
    no_color: bool,
) {
    if !path.exists() {
        return;
    }

    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) => {
            warnings.push(format!(
                "Theme warning: could not read {}: {error}",
                display_path(path)
            ));
            return;
        }
    };

    if no_color {
        if let Some(name) = parse_theme_selection(&content).or_else(|| parse_theme_name(&content)) {
            theme.name = format!("{name} (no-color)");
        }
        let current_source = source.clone();
        *source = format!("{current_source} + {}", display_path(path));
        return;
    }

    if let Some(selection) = parse_theme_selection(&content) {
        match resolve_theme_reference(&selection, user_themes) {
            Some(resolved) => {
                *theme = resolved.theme;
                *source = resolved.source;
            }
            None => warnings.push(format!(
                "Theme warning: unknown theme `{selection}`; searched built-ins [{}] and user themes{}; keeping {source}",
                TuiTheme::built_in_names().join(", "),
                user_theme_dir
                    .map(|dir| format!(" in {}", display_path(dir)))
                    .unwrap_or_else(|| "".to_string())
            )),
        }
    }

    let selected_source = source.clone();
    warnings.extend(prefix_theme_warnings(
        path,
        theme.apply_theme_content(&content),
    ));
    *source = format!("{selected_source} + {}", display_path(path));
}

fn resolve_theme_reference(name: &str, user_themes: &UserThemeRegistry) -> Option<ResolvedTheme> {
    let key = normalized(name);
    if let Some(user_theme) = user_themes.themes.get(&key) {
        return Some(ResolvedTheme {
            theme: user_theme.theme.clone(),
            source: format!(
                "user theme {} ({})",
                user_theme.theme.name(),
                display_path(&user_theme.path)
            ),
        });
    }

    TuiTheme::built_in(name).map(|theme| ResolvedTheme {
        source: format!("built-in {}", theme.name()),
        theme,
    })
}

fn prefix_theme_warnings(path: &Path, warnings: Vec<String>) -> Vec<String> {
    warnings
        .into_iter()
        .map(|warning| match warning.strip_prefix("Theme warning ") {
            Some(rest) => format!("Theme warning {}: {rest}", display_path(path)),
            None => format!("Theme warning {}: {warning}", display_path(path)),
        })
        .collect()
}

fn parse_theme_selection(content: &str) -> Option<String> {
    parse_root_value(content, &["theme"]).or_else(|| parse_theme_base(content))
}

fn parse_theme_base(content: &str) -> Option<String> {
    parse_root_value(content, &["base", "builtin", "extends"])
}

fn parse_theme_name(content: &str) -> Option<String> {
    parse_root_value(content, &["name"])
}

fn parse_root_value(content: &str, keys: &[&str]) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            break;
        }
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if keys
            .iter()
            .any(|expected| key.trim().eq_ignore_ascii_case(expected))
        {
            let value = parse_value(value);
            if !value.trim().is_empty() {
                return Some(value);
            }
        }
    }
    None
}

fn parse_value(raw: &str) -> String {
    let raw = raw.trim();
    if let Some(quote @ ('"' | '\'')) = raw.chars().next() {
        let mut escaped = false;
        for (index, ch) in raw.char_indices().skip(1) {
            if escaped {
                escaped = false;
                continue;
            }
            if quote == '"' && ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == quote {
                let inner = &raw[1..index];
                return inner.replace("\\\"", "\"").replace("\\'", "'");
            }
        }
    }
    if raw.starts_with('#') {
        raw.to_string()
    } else {
        raw.split('#').next().unwrap_or(raw).trim().to_string()
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => Some(true),
        "false" | "no" | "off" | "0" => Some(false),
        _ => None,
    }
}

fn parse_color(value: &str) -> Result<Color, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("empty color".to_string());
    }
    if let Some(hex) = value.strip_prefix('#') {
        return parse_hex_color(hex);
    }

    match normalized(value).as_str() {
        "reset" | "default" | "none" => Ok(Color::Reset),
        "black" => Ok(Color::Black),
        "red" => Ok(Color::Red),
        "green" => Ok(Color::Green),
        "yellow" => Ok(Color::Yellow),
        "blue" => Ok(Color::Blue),
        "magenta" => Ok(Color::Magenta),
        "cyan" => Ok(Color::Cyan),
        "gray" | "grey" => Ok(Color::Gray),
        "dark-gray" | "dark-grey" | "darkgray" | "darkgrey" => Ok(Color::DarkGray),
        "light-red" | "lightred" => Ok(Color::LightRed),
        "light-green" | "lightgreen" => Ok(Color::LightGreen),
        "light-yellow" | "lightyellow" => Ok(Color::LightYellow),
        "light-blue" | "lightblue" => Ok(Color::LightBlue),
        "light-magenta" | "lightmagenta" => Ok(Color::LightMagenta),
        "light-cyan" | "lightcyan" => Ok(Color::LightCyan),
        "white" => Ok(Color::White),
        _ => Err(format!("unsupported color `{value}`")),
    }
}

fn parse_hex_color(hex: &str) -> Result<Color, String> {
    let expanded;
    let hex = if hex.len() == 3 {
        expanded = hex.chars().flat_map(|ch| [ch, ch]).collect::<String>();
        expanded.as_str()
    } else {
        hex
    };

    if hex.len() != 6 || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err("expected #RRGGBB or a supported color name".to_string());
    }

    let red = u8::from_str_radix(&hex[0..2], 16).map_err(|error| error.to_string())?;
    let green = u8::from_str_radix(&hex[2..4], 16).map_err(|error| error.to_string())?;
    let blue = u8::from_str_radix(&hex[4..6], 16).map_err(|error| error.to_string())?;
    Ok(Color::Rgb(red, green, blue))
}

fn normalized(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('_', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_and_named_colors() {
        assert_eq!(parse_color("#c4a7e7").unwrap(), Color::Rgb(196, 167, 231));
        assert_eq!(parse_color("#abc").unwrap(), Color::Rgb(170, 187, 204));
        assert_eq!(parse_color("light-blue").unwrap(), Color::LightBlue);
        assert_eq!(
            parse_color(&parse_value("\"#abc\" # comment")).unwrap(),
            Color::Rgb(170, 187, 204)
        );
    }

    #[test]
    fn default_board_semantic_styles_keep_row_metadata_legible() {
        let theme = TuiTheme::default_dark();
        assert_eq!(theme.priority.low, Color::Rgb(74, 222, 128));
        assert_eq!(theme.priority.medium, Color::Rgb(96, 165, 250));
        assert_eq!(theme.priority.high, Color::Rgb(248, 113, 113));
        assert_eq!(theme.board_selected_style().fg, None);
        assert_eq!(theme.board_selected_style().bg, None);
        assert_eq!(
            theme.board_selected_title_style().fg,
            Some(theme.colors.accent)
        );
    }

    #[test]
    fn selected_always_rule_category_reads_active_not_disabled() {
        let theme = TuiTheme::default_dark();
        let selected = theme.rule_category_style("always", true);
        assert_eq!(selected.fg, Some(theme.colors.selected_fg));
        assert_eq!(selected.bg, Some(theme.colors.selected_bg));
        assert!(selected.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn applies_workspace_theme_content() {
        let mut theme = TuiTheme::default_dark();
        let warnings = theme.apply_theme_content(
            r##"
base = "verdigris"
name = "rose-pine-custom"

[colors]
accent = "#c4a7e7"
selected_bg = "#26233a"

[priority]
high = "#f6c177"

[badges.accord]
delivered = "#c4a7e7"

[badges.review]
changes-requested = "#eb6f92"
"##,
        );
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
        assert_eq!(theme.name(), "rose-pine-custom");
        assert_eq!(theme.colors.accent, Color::Rgb(196, 167, 231));
        assert_eq!(theme.colors.selected_bg, Color::Rgb(38, 35, 58));
        assert_eq!(theme.priority.high, Color::Rgb(246, 193, 119));
        assert_eq!(theme.accord.delivered, Color::Rgb(196, 167, 231));
        assert_eq!(theme.review.changes_requested, Color::Rgb(235, 111, 146));
    }

    #[test]
    fn transparent_background_is_opt_in_and_removes_panel_fills() {
        let mut theme = TuiTheme::default_dark();
        assert_eq!(theme.panel_style().bg, Some(theme.colors.panel));
        assert_eq!(theme.app_style().bg, Some(theme.colors.background));

        let warnings = theme.apply_theme_content("transparent_background = true\n");
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
        assert!(theme.transparent_background);
        assert_eq!(theme.colors.background, Color::Rgb(12, 14, 18));
        assert_eq!(theme.colors.panel, Color::Rgb(22, 25, 31));
        assert_eq!(theme.panel_style().bg, None);
        assert_eq!(theme.app_style().bg, None);
        assert_eq!(theme.text_style().bg, None);
        assert_eq!(theme.selected_style().bg, Some(theme.colors.selected_bg));
    }

    #[test]
    fn invalid_transparent_background_value_warns_and_preserves_opaque_default() {
        let mut theme = TuiTheme::default_dark();
        let warnings = theme.apply_theme_content("transparent_background = maybe\n");
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("invalid boolean"));
        assert!(!theme.transparent_background);
        assert_eq!(theme.panel_style().bg, Some(theme.colors.panel));
    }

    #[test]
    fn badges_use_fixed_legacy_filled_rendering() {
        let theme = TuiTheme::default_dark();
        assert_eq!(theme.badge_label("HIGH"), " HIGH ");
        assert_eq!(theme.badge_label("MED"), " MED  ");
        assert_eq!(theme.badge_label("LOW"), " LOW  ");

        let high = theme.priority_chip_style("high");
        assert_eq!(high.fg, Some(theme.colors.background));
        assert_eq!(high.bg, Some(theme.priority.high));
        assert!(high.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn badge_style_config_is_reported_as_unknown_theme_keys() {
        let mut theme = TuiTheme::default_dark();
        let warnings = theme.apply_theme_content("badge_style = \"ghost\"\n");
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("unknown root key `badge_style`"));
        assert_eq!(theme.badge_label("MED"), " MED  ");

        let warnings = theme.apply_theme_content("[badges]\nstyle = \"ghost\"\n");
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("invalid color for `badges.style`"));
        assert_eq!(theme.badge_label("MED"), " MED  ");
    }

    #[test]
    fn selects_and_maps_verdigris_builtin() {
        let theme = TuiTheme::built_in("verdigris").expect("verdigris theme exists");
        assert_eq!(
            parse_theme_base("base = \"verdigris\""),
            Some("verdigris".to_string())
        );
        assert_eq!(theme.name(), "verdigris");
        assert_eq!(theme.colors.background, Color::Rgb(29, 32, 33));
        assert_eq!(theme.colors.text, Color::Rgb(235, 219, 178));
        assert_eq!(theme.colors.accent, Color::Rgb(142, 192, 124));
        assert_eq!(theme.priority.critical, Color::Rgb(227, 111, 99));
        assert_eq!(theme.priority.high, Color::Rgb(227, 111, 99));
        assert_eq!(theme.priority.medium, Color::Rgb(131, 165, 152));
        assert_eq!(theme.priority.low, Color::Rgb(142, 192, 124));
        assert_eq!(theme.accord.claimed, Color::Rgb(131, 165, 152));
        assert_eq!(theme.accord.delivered, Color::Rgb(142, 192, 124));
        assert_eq!(theme.accord.accepted, Color::Rgb(104, 157, 106));
        assert_eq!(theme.review.rejected, Color::Rgb(227, 111, 99));
    }

    #[test]
    fn discovers_user_theme_and_applies_workspace_selector() {
        let root = temp_theme_dir("selector");
        let workspace = workspace_at(&root);
        let user_theme_dir = root.join("xdg").join("tandem").join("themes");
        std::fs::create_dir_all(&user_theme_dir).unwrap();
        std::fs::write(
            user_theme_dir.join("calm-dark.toml"),
            r##"
name = "calm-dark"
base = "default-dark"

[colors]
accent = "#010203"
"##,
        )
        .unwrap();
        std::fs::write(
            workspace.config_path.with_file_name("theme.toml"),
            r##"
theme = "calm-dark"

[colors]
border = "#040506"
"##,
        )
        .unwrap();

        let load = TuiTheme::load_for_workspace_with_options(
            &workspace,
            Some(user_theme_dir.clone()),
            None,
            false,
        );

        assert!(
            load.warnings.is_empty(),
            "unexpected warnings: {:?}",
            load.warnings
        );
        assert_eq!(load.theme.name(), "calm-dark");
        assert_eq!(load.theme.colors.accent, Color::Rgb(1, 2, 3));
        assert_eq!(load.theme.colors.border, Color::Rgb(4, 5, 6));
        assert!(load.source.contains("user theme calm-dark"));
        assert!(load.source.contains("theme.toml"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn global_user_config_selects_theme_without_workspace_mirror() {
        let root = temp_theme_dir("global-config");
        let workspace = workspace_at(&root);
        let user_config_path = root.join("xdg").join("tandem").join("config.toml");
        std::fs::create_dir_all(user_config_path.parent().unwrap()).unwrap();
        std::fs::write(
            &user_config_path,
            r##"
theme = "verdigris"
transparent_background = true
"##,
        )
        .unwrap();

        let load = TuiTheme::load_for_workspace_with_options(
            &workspace,
            None,
            Some(user_config_path.clone()),
            false,
        );

        assert!(
            load.warnings.is_empty(),
            "unexpected warnings: {:?}",
            load.warnings
        );
        assert_eq!(load.theme.name(), "verdigris");
        assert_eq!(load.theme.colors.accent, Color::Rgb(142, 192, 124));
        assert!(load.theme.transparent_background);
        assert_eq!(load.theme.panel_style().bg, None);
        assert!(load.source.contains("built-in verdigris"));
        assert!(load.source.contains("config.toml"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn workspace_theme_overrides_global_user_config() {
        let root = temp_theme_dir("workspace-overrides-global");
        let workspace = workspace_at(&root);
        let user_config_path = root.join("xdg").join("tandem").join("config.toml");
        std::fs::create_dir_all(user_config_path.parent().unwrap()).unwrap();
        std::fs::write(&user_config_path, "theme = \"verdigris\"\n").unwrap();
        std::fs::write(
            workspace.config_path.with_file_name("theme.toml"),
            r##"
theme = "default-dark"

[colors]
accent = "#010203"
"##,
        )
        .unwrap();

        let load = TuiTheme::load_for_workspace_with_options(
            &workspace,
            None,
            Some(user_config_path),
            false,
        );

        assert!(
            load.warnings.is_empty(),
            "unexpected warnings: {:?}",
            load.warnings
        );
        assert_eq!(load.theme.name(), "default-dark");
        assert_eq!(load.theme.colors.accent, Color::Rgb(1, 2, 3));
        assert!(load.source.contains(".tandem/theme.toml") || load.source.contains("theme.toml"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn invalid_user_theme_warns_without_blocking_fallback_theme() {
        let root = temp_theme_dir("invalid");
        let workspace = workspace_at(&root);
        let user_theme_dir = root.join("xdg").join("tandem").join("themes");
        std::fs::create_dir_all(&user_theme_dir).unwrap();
        std::fs::write(
            user_theme_dir.join("broken.toml"),
            r##"
name = "broken"

[colors]
accent = "wat"
"##,
        )
        .unwrap();

        let load = TuiTheme::load_for_workspace_with_options(
            &workspace,
            Some(user_theme_dir),
            None,
            false,
        );

        assert_eq!(load.theme.name(), "default-dark");
        assert!(load.source.contains("built-in default-dark"));
        assert_eq!(load.warnings.len(), 1);
        assert!(load.warnings[0].contains("broken.toml"));
        assert!(load.warnings[0].contains("invalid color"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn root_selector_parsing_stops_at_sections() {
        assert_eq!(
            parse_theme_selection("theme = \"verdigris\""),
            Some("verdigris".to_string())
        );
        assert_eq!(parse_theme_selection("[colors]\ntheme = \"ignored\""), None);
    }

    #[test]
    fn reports_unknown_keys_and_bad_colors() {
        let mut theme = TuiTheme::default_dark();
        let warnings = theme.apply_theme_content(
            r##"
[colors]
not_a_key = "#ffffff"
accent = "wat"
"##,
        );
        assert_eq!(warnings.len(), 2);
        assert!(warnings[0].contains("unknown theme key"));
        assert!(warnings[1].contains("invalid color"));
    }

    fn temp_theme_dir(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!(
            "tandem-theme-{name}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn workspace_at(root: &Path) -> Workspace {
        let tandem_dir = root.join(".tandem");
        std::fs::create_dir_all(&tandem_dir).unwrap();
        let config_path = tandem_dir.join("tandem.md");
        std::fs::write(
            &config_path,
            "---\ntitle: Test\nstates: [todo, in-progress, review]\n---\n",
        )
        .unwrap();
        Workspace {
            board_dir: tandem_dir.join("board"),
            logs_dir: tandem_dir.join("logs"),
            events_path: tandem_dir.join("events.jsonl"),
            config_path,
        }
    }
}
