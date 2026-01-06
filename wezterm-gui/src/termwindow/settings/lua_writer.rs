use super::state::{PaneConfig, SplitConfig, SplitDirection, StartupLayout};
use super::state::{AppearanceSettings, KeybindingsSettings, TabBarSettings};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Writes WezTerm configuration to Lua files
pub struct LuaConfigWriter {
    config_path: PathBuf,
}

impl LuaConfigWriter {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    /// Get default config path
    pub fn default_config_path() -> Option<PathBuf> {
        // Check environment variable first
        if let Some(path) = std::env::var_os("WEZTERM_CONFIG_FILE") {
            return Some(PathBuf::from(path));
        }

        // Try common locations using HOME environment variable
        if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
            let home = PathBuf::from(home);
            let wezterm_lua = home.join(".wezterm.lua");
            if wezterm_lua.exists() {
                return Some(wezterm_lua);
            }

            let config_dir = home.join(".config").join("wezterm").join("wezterm.lua");
            if config_dir.exists() {
                return Some(config_dir);
            }

            // Default to ~/.wezterm.lua if nothing exists
            return Some(wezterm_lua);
        }

        None
    }

    /// Create a backup of the config file
    pub fn create_backup(&self) -> anyhow::Result<PathBuf> {
        if !self.config_path.exists() {
            anyhow::bail!("Config file does not exist");
        }

        let backup_path = self.config_path.with_extension("lua.backup");
        fs::copy(&self.config_path, &backup_path)?;
        Ok(backup_path)
    }

    /// Generate Lua code for startup layout
    pub fn generate_startup_layout(&self, layout: &StartupLayout) -> String {
        if layout.panes.is_empty() {
            return String::new();
        }

        let mut lua = String::new();
        lua.push_str("-- GUI Settings: Startup Layout\n");
        lua.push_str("wezterm.on(\"gui-startup\", function(cmd)\n");
        lua.push_str("    local mux = wezterm.mux\n\n");

        // First pane
        let first_pane = &layout.panes[0];
        lua.push_str("    local tab, pane, window = mux.spawn_window(");
        lua.push_str(&self.pane_config_to_lua(first_pane));
        lua.push_str(")\n");

        // Subsequent panes with splits
        for (idx, split) in layout.splits.iter().enumerate() {
            if let Some(pane_config) = layout.panes.get(split.second_pane) {
                let direction = match split.direction {
                    SplitDirection::Horizontal => "Right",
                    SplitDirection::Vertical => "Bottom",
                };

                let var_name = format!("pane{}", idx + 2);
                lua.push_str(&format!(
                    "    local {} = pane:split({{\n",
                    var_name
                ));
                lua.push_str(&format!("        direction = \"{}\",\n", direction));
                lua.push_str(&format!("        size = {},\n", split.ratio));

                if let Some(cwd) = &pane_config.cwd {
                    lua.push_str(&format!(
                        "        cwd = \"{}\",\n",
                        cwd.display().to_string().replace('\\', "/")
                    ));
                }

                if let Some(cmd) = &pane_config.command {
                    if !cmd.is_empty() {
                        let args: Vec<String> = cmd.iter().map(|s| format!("\"{}\"", s)).collect();
                        lua.push_str(&format!("        args = {{ {} }},\n", args.join(", ")));
                    }
                }

                lua.push_str("    })\n");
            }
        }

        lua.push_str("end)\n");
        lua
    }

    fn pane_config_to_lua(&self, pane: &PaneConfig) -> String {
        let mut parts = vec![];

        if let Some(cwd) = &pane.cwd {
            parts.push(format!(
                "cwd = \"{}\"",
                cwd.display().to_string().replace('\\', "/")
            ));
        }

        if let Some(cmd) = &pane.command {
            if !cmd.is_empty() {
                let args: Vec<String> = cmd.iter().map(|s| format!("\"{}\"", s)).collect();
                parts.push(format!("args = {{ {} }}", args.join(", ")));
            }
        }

        if parts.is_empty() {
            "cmd or {}".to_string()
        } else {
            format!("{{ {} }}", parts.join(", "))
        }
    }

    /// Generate Lua code for appearance settings
    pub fn generate_appearance(&self, settings: &AppearanceSettings) -> String {
        let mut lua = String::new();
        lua.push_str("-- GUI Settings: Appearance\n");

        lua.push_str(&format!("config.font_size = {:.1}\n", settings.font_size));

        if let Some(scheme) = &settings.color_scheme {
            lua.push_str(&format!("config.color_scheme = \"{}\"\n", scheme));
        }

        lua.push_str(&format!(
            "config.window_background_opacity = {:.2}\n",
            settings.window_background_opacity
        ));

        if let Some(bg_image) = &settings.background_image {
            lua.push_str(&format!(
                "config.window_background_image = \"{}\"\n",
                bg_image.display().to_string().replace('\\', "/")
            ));
        }

        lua
    }

    /// Generate Lua code for tab bar settings
    pub fn generate_tab_bar(&self, settings: &TabBarSettings) -> String {
        let mut lua = String::new();
        lua.push_str("-- GUI Settings: Tab Bar\n");

        lua.push_str(&format!(
            "config.enable_tab_bar = {}\n",
            settings.enable_tab_bar
        ));
        lua.push_str(&format!(
            "config.tab_bar_at_bottom = {}\n",
            settings.tab_bar_at_bottom
        ));
        lua.push_str(&format!(
            "config.hide_tab_bar_if_only_one_tab = {}\n",
            settings.hide_tab_bar_if_only_one_tab
        ));
        lua.push_str(&format!(
            "config.use_fancy_tab_bar = {}\n",
            settings.use_fancy_tab_bar
        ));
        lua.push_str(&format!(
            "config.show_new_tab_button_in_tab_bar = {}\n",
            settings.show_new_tab_button
        ));
        lua.push_str(&format!(
            "config.show_close_tab_button_in_tabs = {}\n",
            settings.show_close_tab_button
        ));

        lua
    }

    /// Write all settings to the config file
    pub fn write_all(
        &self,
        startup_layout: Option<&StartupLayout>,
        appearance: Option<&AppearanceSettings>,
        tab_bar: Option<&TabBarSettings>,
    ) -> anyhow::Result<()> {
        // Create backup first
        if self.config_path.exists() {
            self.create_backup()?;
        }

        // Read existing content or create new
        let existing = if self.config_path.exists() {
            fs::read_to_string(&self.config_path)?
        } else {
            self.default_config_template()
        };

        // Generate new content
        let mut new_content = existing;

        // Remove old GUI Settings sections
        new_content = self.remove_gui_settings_section(&new_content, "Startup Layout");
        new_content = self.remove_gui_settings_section(&new_content, "Appearance");
        new_content = self.remove_gui_settings_section(&new_content, "Tab Bar");

        // Remove any existing gui-startup handler (including non-GUI-Settings ones)
        if startup_layout.is_some() {
            new_content = self.remove_existing_gui_startup(&new_content);
        }

        // Find the return statement and insert before it
        let return_pos = new_content.rfind("return config").or_else(|| new_content.rfind("return {"));

        let insert_pos = return_pos.unwrap_or(new_content.len());
        let (before_return, after_return) = new_content.split_at(insert_pos);

        let mut settings_lua = String::new();
        settings_lua.push('\n');

        if let Some(layout) = startup_layout {
            if !layout.panes.is_empty() {
                settings_lua.push_str(&self.generate_startup_layout(layout));
                settings_lua.push('\n');
            }
        }

        if let Some(appearance) = appearance {
            settings_lua.push_str(&self.generate_appearance(appearance));
            settings_lua.push('\n');
        }

        if let Some(tab_bar) = tab_bar {
            settings_lua.push_str(&self.generate_tab_bar(tab_bar));
            settings_lua.push('\n');
        }

        let final_content = format!("{}{}{}", before_return.trim_end(), settings_lua, after_return);

        // Write to file
        let mut file = fs::File::create(&self.config_path)?;
        file.write_all(final_content.as_bytes())?;

        Ok(())
    }

    fn remove_gui_settings_section(&self, content: &str, section_name: &str) -> String {
        let marker = format!("-- GUI Settings: {}", section_name);
        let mut result = String::new();
        let mut skip_section = false;

        for line in content.lines() {
            if line.starts_with(&marker) {
                skip_section = true;
                continue;
            }

            if skip_section {
                // End section at next comment or blank line followed by non-indented code
                if line.starts_with("--") || (line.trim().is_empty()) {
                    if line.starts_with("-- GUI Settings:") {
                        // Another GUI settings section
                        skip_section = true;
                    } else if line.starts_with("--") {
                        skip_section = false;
                        result.push_str(line);
                        result.push('\n');
                    } else {
                        // Blank line - check next
                    }
                } else if !line.starts_with("    ") && !line.starts_with("config.") && !line.starts_with("wezterm.") {
                    skip_section = false;
                    result.push_str(line);
                    result.push('\n');
                }
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }

        result
    }

    /// Remove existing gui-startup event handler from content
    fn remove_existing_gui_startup(&self, content: &str) -> String {
        let mut result = String::new();
        let mut in_gui_startup = false;
        let mut brace_depth = 0;
        let mut paren_depth = 0;
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // Check if this line starts a gui-startup handler
            if !in_gui_startup
                && (line.contains("wezterm.on('gui-startup'")
                    || line.contains("wezterm.on(\"gui-startup\""))
            {
                in_gui_startup = true;
                // Count opening braces/parens on this line
                for ch in line.chars() {
                    match ch {
                        '(' => paren_depth += 1,
                        ')' => paren_depth -= 1,
                        '{' => brace_depth += 1,
                        '}' => brace_depth -= 1,
                        _ => {}
                    }
                }
                // Skip comment line before gui-startup if it's a description
                if i > 0 && lines[i - 1].starts_with("--") && !lines[i - 1].starts_with("-- GUI Settings:") {
                    // Remove the previous comment line too
                    if result.ends_with('\n') {
                        // Check if last line in result is the comment
                        let result_lines: Vec<&str> = result.lines().collect();
                        if let Some(last) = result_lines.last() {
                            if last.starts_with("--") {
                                // Remove last line
                                result = result_lines[..result_lines.len() - 1].join("\n");
                                if !result.is_empty() {
                                    result.push('\n');
                                }
                            }
                        }
                    }
                }
                i += 1;
                continue;
            }

            if in_gui_startup {
                // Track nested braces and parens
                for ch in line.chars() {
                    match ch {
                        '(' => paren_depth += 1,
                        ')' => paren_depth -= 1,
                        '{' => brace_depth += 1,
                        '}' => brace_depth -= 1,
                        _ => {}
                    }
                }

                // Check if we've closed all braces/parens (handler ended)
                if paren_depth <= 0 && brace_depth <= 0 {
                    in_gui_startup = false;
                    brace_depth = 0;
                    paren_depth = 0;
                }
                i += 1;
                continue;
            }

            result.push_str(line);
            result.push('\n');
            i += 1;
        }

        // Clean up multiple consecutive blank lines
        let mut cleaned = String::new();
        let mut prev_blank = false;
        for line in result.lines() {
            let is_blank = line.trim().is_empty();
            if is_blank && prev_blank {
                continue;
            }
            cleaned.push_str(line);
            cleaned.push('\n');
            prev_blank = is_blank;
        }

        cleaned
    }

    fn default_config_template(&self) -> String {
        r#"local wezterm = require 'wezterm'
local config = wezterm.config_builder()

-- Your configuration here

return config
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_appearance() {
        let writer = LuaConfigWriter::new(PathBuf::from("/tmp/test.lua"));
        let settings = AppearanceSettings {
            font_size: 14.0,
            color_scheme: Some("Dracula".to_string()),
            window_background_opacity: 0.95,
            background_image: None,
        };

        let lua = writer.generate_appearance(&settings);
        assert!(lua.contains("font_size = 14.0"));
        assert!(lua.contains("color_scheme = \"Dracula\""));
        assert!(lua.contains("window_background_opacity = 0.95"));
    }

    #[test]
    fn test_generate_tab_bar() {
        let writer = LuaConfigWriter::new(PathBuf::from("/tmp/test.lua"));
        let settings = TabBarSettings {
            enable_tab_bar: true,
            tab_bar_at_bottom: true,
            hide_tab_bar_if_only_one_tab: false,
            use_fancy_tab_bar: true,
            show_new_tab_button: true,
            show_close_tab_button: true,
        };

        let lua = writer.generate_tab_bar(&settings);
        assert!(lua.contains("enable_tab_bar = true"));
        assert!(lua.contains("tab_bar_at_bottom = true"));
    }
}
