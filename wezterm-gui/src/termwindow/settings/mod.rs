use crate::termwindow::box_model::*;
use crate::termwindow::modal::Modal;
use crate::termwindow::render::corners::{
    BOTTOM_LEFT_ROUNDED_CORNER, BOTTOM_RIGHT_ROUNDED_CORNER, TOP_LEFT_ROUNDED_CORNER,
    TOP_RIGHT_ROUNDED_CORNER,
};
use crate::termwindow::{DimensionContext, TermWindow};
use crate::utilsprites::RenderMetrics;
use config::keyassignment::KeyAssignment;
use config::Dimension;
use std::cell::{Ref, RefCell};
use std::rc::Rc;
use wezterm_font::LoadedFont;
use wezterm_term::{KeyCode, KeyModifiers, MouseEvent};
use window::color::LinearRgba;

pub mod lua_writer;
pub mod panels;
pub mod state;
pub mod widgets;

pub use state::SettingsState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPanel {
    StartupLayout,
    Appearance,
    TabBar,
    Keybindings,
}

impl SettingsPanel {
    pub fn all() -> &'static [SettingsPanel] {
        &[
            SettingsPanel::StartupLayout,
            SettingsPanel::Appearance,
            SettingsPanel::TabBar,
            SettingsPanel::Keybindings,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            SettingsPanel::StartupLayout => "Startup Layout",
            SettingsPanel::Appearance => "Appearance",
            SettingsPanel::TabBar => "Tab Bar",
            SettingsPanel::Keybindings => "Keybindings",
        }
    }
}

pub struct SettingsModal {
    element: RefCell<Option<Vec<ComputedElement>>>,
    state: RefCell<SettingsState>,
}

impl SettingsModal {
    pub fn new(term_window: &TermWindow) -> Self {
        Self {
            element: RefCell::new(None),
            state: RefCell::new(SettingsState::new(term_window)),
        }
    }

    fn build_ui(&self, term_window: &mut TermWindow) -> anyhow::Result<Vec<ComputedElement>> {
        let font = term_window
            .fonts
            .command_palette_font()
            .expect("failed to resolve command palette font");
        let metrics = RenderMetrics::with_font_metrics(&font.metrics());

        let bg: LinearRgba = term_window.config.command_palette_bg_color.to_linear();
        let fg: LinearRgba = term_window.config.command_palette_fg_color.to_linear();
        let border_color = term_window.config.pane_select_fg_color.to_linear();

        let state = self.state.borrow();

        // Build sidebar
        let sidebar_items: Vec<Element> = SettingsPanel::all()
            .iter()
            .map(|panel| {
                let is_selected = *panel == state.current_panel;
                let panel_bg = if is_selected {
                    border_color.clone()
                } else {
                    bg.clone()
                };
                let panel_fg = if is_selected {
                    bg.clone()
                } else {
                    fg.clone()
                };

                Element::new(&font, ElementContent::Text(panel.label().to_string()))
                    .colors(ElementColors {
                        border: BorderColor::default(),
                        bg: panel_bg.into(),
                        text: panel_fg.into(),
                    })
                    .padding(BoxDimension {
                        left: Dimension::Cells(1.0),
                        right: Dimension::Cells(1.0),
                        top: Dimension::Cells(0.25),
                        bottom: Dimension::Cells(0.25),
                    })
                    .display(DisplayType::Block)
            })
            .collect();

        let sidebar = Element::new(&font, ElementContent::Children(sidebar_items))
            .colors(ElementColors {
                border: BorderColor::new(border_color.clone()),
                bg: bg.clone().into(),
                text: fg.clone().into(),
            })
            .padding(BoxDimension {
                left: Dimension::Cells(0.5),
                right: Dimension::Cells(0.5),
                top: Dimension::Cells(0.5),
                bottom: Dimension::Cells(0.5),
            })
            .border(BoxDimension::new(Dimension::Pixels(1.0)))
            .display(DisplayType::Inline)
            .vertical_align(VerticalAlign::Top);

        // Build content panel
        let content_str = match state.current_panel {
            SettingsPanel::StartupLayout => {
                let mut text = String::from("Startup Layout Configuration\n\n");
                text.push_str("Current panes in startup layout:\n\n");

                if state.startup_layout.panes.is_empty() {
                    text.push_str("  (No startup layout configured)\n\n");
                } else {
                    for (idx, pane) in state.startup_layout.panes.iter().enumerate() {
                        let cwd = pane.cwd.as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|| "(default)".to_string());
                        let cmd = pane.command.as_ref()
                            .map(|args| args.join(" "))
                            .unwrap_or_else(|| "(shell)".to_string());
                        text.push_str(&format!("  Pane {}: cwd={}, cmd={}\n", idx + 1, cwd, cmd));
                    }
                    text.push('\n');
                }

                text.push_str("Keys:\n");
                text.push_str("  [S] Save current window layout as startup\n");
                text.push_str("  [C] Clear startup layout\n");
                text.push_str("  [Esc] Close settings\n");
                text
            }
            SettingsPanel::Appearance => format!(
                "Appearance Settings:\n\n\
                 Font Size: {:.1}\n\
                 Color Scheme: {}\n\
                 Window Opacity: {:.0}%",
                state.appearance.font_size,
                state.appearance.color_scheme.as_deref().unwrap_or("(default)"),
                state.appearance.window_background_opacity * 100.0
            ),
            SettingsPanel::TabBar => format!(
                "Tab Bar Settings:\n\n\
                 Enable Tab Bar: {}\n\
                 Tab Bar at Bottom: {}\n\
                 Hide if Only One Tab: {}\n\
                 Use Fancy Tab Bar: {}",
                if state.tab_bar.enable_tab_bar { "Yes" } else { "No" },
                if state.tab_bar.tab_bar_at_bottom { "Yes" } else { "No" },
                if state.tab_bar.hide_tab_bar_if_only_one_tab { "Yes" } else { "No" },
                if state.tab_bar.use_fancy_tab_bar { "Yes" } else { "No" }
            ),
            SettingsPanel::Keybindings => {
                "Keybindings:\n\n\
                 Configure keyboard shortcuts.\n\
                 (Coming soon - use wezterm.lua for now)".to_string()
            }
        };

        let content = Element::new(&font, ElementContent::Text(content_str))
            .colors(ElementColors {
                border: BorderColor::new(border_color.clone()),
                bg: bg.clone().into(),
                text: fg.clone().into(),
            })
            .padding(BoxDimension {
                left: Dimension::Cells(1.0),
                right: Dimension::Cells(1.0),
                top: Dimension::Cells(0.5),
                bottom: Dimension::Cells(0.5),
            })
            .border(BoxDimension::new(Dimension::Pixels(1.0)))
            .min_width(Some(Dimension::Cells(50.0)))
            .display(DisplayType::Inline)
            .vertical_align(VerticalAlign::Top);

        // Title bar
        let title = Element::new(&font, ElementContent::Text("Settings".to_string()))
            .colors(ElementColors {
                border: BorderColor::default(),
                bg: border_color.clone().into(),
                text: bg.clone().into(),
            })
            .padding(BoxDimension {
                left: Dimension::Cells(1.0),
                right: Dimension::Cells(1.0),
                top: Dimension::Cells(0.5),
                bottom: Dimension::Cells(0.5),
            })
            .display(DisplayType::Block);

        // Main row containing sidebar and content (horizontal layout)
        let main_row = Element::new(
            &font,
            ElementContent::Children(vec![sidebar, content]),
        )
        .colors(ElementColors {
            border: BorderColor::default(),
            bg: bg.clone().into(),
            text: fg.clone().into(),
        })
        .display(DisplayType::Block);

        // Container with all elements
        let element = Element::new(
            &font,
            ElementContent::Children(vec![title, main_row]),
        )
        .colors(ElementColors {
            border: BorderColor::new(border_color.clone()),
            bg: bg.into(),
            text: fg.into(),
        })
        .border(BoxDimension::new(Dimension::Pixels(1.0)))
        .padding(BoxDimension::new(Dimension::Cells(0.5)))
        .border_corners(Some(Corners {
            top_left: SizedPoly {
                width: Dimension::Cells(0.5),
                height: Dimension::Cells(0.5),
                poly: TOP_LEFT_ROUNDED_CORNER,
            },
            top_right: SizedPoly {
                width: Dimension::Cells(0.5),
                height: Dimension::Cells(0.5),
                poly: TOP_RIGHT_ROUNDED_CORNER,
            },
            bottom_left: SizedPoly {
                width: Dimension::Cells(0.5),
                height: Dimension::Cells(0.5),
                poly: BOTTOM_LEFT_ROUNDED_CORNER,
            },
            bottom_right: SizedPoly {
                width: Dimension::Cells(0.5),
                height: Dimension::Cells(0.5),
                poly: BOTTOM_RIGHT_ROUNDED_CORNER,
            },
        }))
        .display(DisplayType::Block);

        let dimensions = term_window.dimensions;
        let size = term_window.terminal_size;

        let top_bar_height = if term_window.show_tab_bar && !term_window.config.tab_bar_at_bottom {
            term_window.tab_bar_pixel_height().unwrap_or(0.)
        } else {
            0.
        };
        let (padding_left, padding_top) = term_window.padding_left_top();
        let border = term_window.get_os_border();
        let top_pixel_y = top_bar_height + padding_top + border.top.get() as f32;

        let desired_width = (size.cols / 2).max(80).min(size.cols);
        let avail_pixel_width =
            size.cols as f32 * term_window.render_metrics.cell_size.width as f32;
        let desired_pixel_width =
            desired_width as f32 * term_window.render_metrics.cell_size.width as f32;

        let x_adjust = ((avail_pixel_width - padding_left) - desired_pixel_width) / 2.;

        let computed = term_window.compute_element(
            &LayoutContext {
                height: DimensionContext {
                    dpi: dimensions.dpi as f32,
                    pixel_max: dimensions.pixel_height as f32,
                    pixel_cell: metrics.cell_size.height as f32,
                },
                width: DimensionContext {
                    dpi: dimensions.dpi as f32,
                    pixel_max: dimensions.pixel_width as f32,
                    pixel_cell: metrics.cell_size.width as f32,
                },
                bounds: euclid::rect(
                    padding_left + x_adjust,
                    top_pixel_y,
                    desired_pixel_width,
                    size.rows as f32 * term_window.render_metrics.cell_size.height as f32,
                ),
                metrics: &metrics,
                gl_state: term_window.render_state.as_ref().unwrap(),
                zindex: 100,
            },
            &element,
        )?;

        Ok(vec![computed])
    }
}

impl SettingsModal {
    fn save_current_layout(&self, term_window: &mut TermWindow) {
        use mux::Mux;

        let mux = Mux::get();
        let mut state = self.state.borrow_mut();

        // Clear existing layout
        state.startup_layout.panes.clear();
        state.startup_layout.splits.clear();

        // Get current window's panes
        let window_id = term_window.mux_window_id;
        if let Some(mux_window) = mux.get_window(window_id) {
            if let Some(tab) = mux_window.get_active() {
                let panes = tab.iter_panes_ignoring_zoom();

                for (idx, pos) in panes.iter().enumerate() {
                    let pane = &pos.pane;

                    // Get the pane's current working directory
                    let cwd = pane
                        .get_current_working_dir(mux::pane::CachePolicy::FetchImmediate)
                        .and_then(|url| {
                            if url.scheme() == "file" {
                                Some(std::path::PathBuf::from(url.path()))
                            } else {
                                None
                            }
                        });

                    state.startup_layout.panes.push(state::PaneConfig {
                        id: idx,
                        cwd,
                        command: None, // Can't easily get the running command
                        name: Some(format!("Pane {}", idx + 1)),
                    });
                }
            }
        }

        state.has_changes = true;

        // Save to Lua file
        if let Some(config_path) = lua_writer::LuaConfigWriter::default_config_path() {
            let writer = lua_writer::LuaConfigWriter::new(config_path);
            if let Err(e) = writer.write_all(
                Some(&state.startup_layout),
                None,
                None,
            ) {
                log::error!("Failed to save startup layout: {}", e);
            }
        }
    }
}

impl Modal for SettingsModal {
    fn mouse_event(&self, _event: MouseEvent, _term_window: &mut TermWindow) -> anyhow::Result<()> {
        // Mouse handling will be implemented later
        Ok(())
    }

    fn key_down(
        &self,
        key: KeyCode,
        _mods: KeyModifiers,
        term_window: &mut TermWindow,
    ) -> anyhow::Result<bool> {
        match key {
            KeyCode::Escape => {
                term_window.cancel_modal();
                Ok(true)
            }
            KeyCode::UpArrow | KeyCode::Char('k') => {
                let mut state = self.state.borrow_mut();
                let panels = SettingsPanel::all();
                let current_idx = panels
                    .iter()
                    .position(|p| *p == state.current_panel)
                    .unwrap_or(0);
                if current_idx > 0 {
                    state.current_panel = panels[current_idx - 1];
                    drop(state);
                    self.element.borrow_mut().take();
                    term_window.invalidate_modal();
                }
                Ok(true)
            }
            KeyCode::DownArrow | KeyCode::Char('j') => {
                let mut state = self.state.borrow_mut();
                let panels = SettingsPanel::all();
                let current_idx = panels
                    .iter()
                    .position(|p| *p == state.current_panel)
                    .unwrap_or(0);
                if current_idx + 1 < panels.len() {
                    state.current_panel = panels[current_idx + 1];
                    drop(state);
                    self.element.borrow_mut().take();
                    term_window.invalidate_modal();
                }
                Ok(true)
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                let state = self.state.borrow();
                if state.current_panel == SettingsPanel::StartupLayout {
                    drop(state);
                    // Capture current window layout
                    self.save_current_layout(term_window);
                    self.element.borrow_mut().take();
                    term_window.invalidate_modal();
                }
                Ok(true)
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                let mut state = self.state.borrow_mut();
                if state.current_panel == SettingsPanel::StartupLayout {
                    // Clear startup layout
                    state.startup_layout.panes.clear();
                    state.startup_layout.splits.clear();
                    state.has_changes = true;
                    drop(state);
                    self.element.borrow_mut().take();
                    term_window.invalidate_modal();
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn computed_element(
        &self,
        term_window: &mut TermWindow,
    ) -> anyhow::Result<Ref<'_, [ComputedElement]>> {
        if self.element.borrow().is_none() {
            let elements = self.build_ui(term_window)?;
            *self.element.borrow_mut() = Some(elements);
        }

        Ok(Ref::map(self.element.borrow(), |opt| {
            opt.as_ref().map(|v| v.as_slice()).unwrap_or(&[])
        }))
    }

    fn reconfigure(&self, _term_window: &mut TermWindow) {
        self.element.borrow_mut().take();
    }
}
