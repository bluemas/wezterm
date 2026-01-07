use super::SettingsPanel;
use crate::termwindow::TermWindow;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PaneConfig {
    pub id: usize,
    pub cwd: Option<PathBuf>,
    pub command: Option<Vec<String>>,
    pub name: Option<String>,
}

impl Default for PaneConfig {
    fn default() -> Self {
        Self {
            id: 0,
            cwd: None,
            command: None,
            name: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct SplitConfig {
    pub direction: SplitDirection,
    pub ratio: f32,
    pub first_pane: usize,
    pub second_pane: usize,
}

#[derive(Debug, Clone)]
pub struct StartupLayout {
    pub panes: Vec<PaneConfig>,
    pub splits: Vec<SplitConfig>,
}

impl Default for StartupLayout {
    fn default() -> Self {
        Self {
            panes: vec![],  // Empty by default - will be populated by Save Layout
            splits: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppearanceSettings {
    pub font_size: f64,
    pub color_scheme: Option<String>,
    pub window_background_opacity: f32,
    pub background_image: Option<PathBuf>,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            font_size: 12.0,
            color_scheme: None,
            window_background_opacity: 1.0,
            background_image: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TabBarSettings {
    pub enable_tab_bar: bool,
    pub tab_bar_at_bottom: bool,
    pub hide_tab_bar_if_only_one_tab: bool,
    pub use_fancy_tab_bar: bool,
    pub show_new_tab_button: bool,
    pub show_close_tab_button: bool,
}

impl Default for TabBarSettings {
    fn default() -> Self {
        Self {
            enable_tab_bar: true,
            tab_bar_at_bottom: false,
            hide_tab_bar_if_only_one_tab: false,
            use_fancy_tab_bar: true,
            show_new_tab_button: true,
            show_close_tab_button: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub key: String,
    pub modifiers: String,
    pub action: String,
}

#[derive(Debug, Clone)]
pub struct KeybindingsSettings {
    pub bindings: Vec<KeyBinding>,
}

impl Default for KeybindingsSettings {
    fn default() -> Self {
        Self { bindings: vec![] }
    }
}

#[derive(Debug)]
pub struct SettingsState {
    pub current_panel: SettingsPanel,
    pub startup_layout: StartupLayout,
    pub appearance: AppearanceSettings,
    pub tab_bar: TabBarSettings,
    pub keybindings: KeybindingsSettings,
    pub has_changes: bool,
    pub config_path: Option<PathBuf>,
}

impl SettingsState {
    pub fn new(term_window: &TermWindow) -> Self {
        let config = &term_window.config;

        // Load current settings from config
        let appearance = AppearanceSettings {
            font_size: config.font_size,
            color_scheme: config.color_scheme.clone(),
            window_background_opacity: config.window_background_opacity,
            background_image: config.window_background_image.clone(),
        };

        let tab_bar = TabBarSettings {
            enable_tab_bar: config.enable_tab_bar,
            tab_bar_at_bottom: config.tab_bar_at_bottom,
            hide_tab_bar_if_only_one_tab: config.hide_tab_bar_if_only_one_tab,
            use_fancy_tab_bar: config.use_fancy_tab_bar,
            show_new_tab_button: config.show_new_tab_button_in_tab_bar,
            show_close_tab_button: config.show_close_tab_button_in_tabs,
        };

        Self {
            current_panel: SettingsPanel::StartupLayout,
            startup_layout: StartupLayout::default(),
            appearance,
            tab_bar,
            keybindings: KeybindingsSettings::default(),
            has_changes: false,
            config_path: std::env::var_os("WEZTERM_CONFIG_FILE").map(PathBuf::from),
        }
    }

    pub fn mark_changed(&mut self) {
        self.has_changes = true;
    }
}
