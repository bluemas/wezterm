use crate::termwindow::box_model::*;
use crate::termwindow::modal::Modal;
use crate::termwindow::render::corners::{
    BOTTOM_LEFT_ROUNDED_CORNER, BOTTOM_RIGHT_ROUNDED_CORNER, TOP_LEFT_ROUNDED_CORNER,
    TOP_RIGHT_ROUNDED_CORNER,
};
use crate::termwindow::{DimensionContext, SettingsUIAction, TermWindow, UIItemType};
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

        // Build sidebar with clickable panel items
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
                    .item_type(UIItemType::Settings(SettingsUIAction::SelectPanel(
                        panel.label().to_string(),
                    )))
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

        // Helper function to create a text line element
        let make_line = |text: &str, font: &Rc<LoadedFont>, bg: LinearRgba, fg: LinearRgba| {
            Element::new(font, ElementContent::Text(text.to_string()))
                .colors(ElementColors {
                    border: BorderColor::default(),
                    bg: bg.into(),
                    text: fg.into(),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.0),
                    right: Dimension::Cells(0.0),
                    top: Dimension::Cells(0.0),
                    bottom: Dimension::Cells(0.0),
                })
                .display(DisplayType::Block)
        };

        // Helper function to create a clickable button
        let make_button = |text: &str,
                           font: &Rc<LoadedFont>,
                           bg: LinearRgba,
                           fg: LinearRgba,
                           border: LinearRgba,
                           action: SettingsUIAction| {
            Element::new(font, ElementContent::Text(text.to_string()))
                .colors(ElementColors {
                    border: BorderColor::new(border),
                    bg: bg.into(),
                    text: fg.into(),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(1.0),
                    right: Dimension::Cells(1.0),
                    top: Dimension::Cells(0.25),
                    bottom: Dimension::Cells(0.25),
                })
                .border(BoxDimension::new(Dimension::Pixels(1.0)))
                .margin(BoxDimension {
                    left: Dimension::Cells(0.0),
                    right: Dimension::Cells(0.5),
                    top: Dimension::Cells(0.25),
                    bottom: Dimension::Cells(0.25),
                })
                .display(DisplayType::Inline)
                .item_type(UIItemType::Settings(action))
        };

        // Helper function to create a clickable toggle option
        let make_toggle = |label: &str,
                           value: bool,
                           font: &Rc<LoadedFont>,
                           bg: LinearRgba,
                           fg: LinearRgba,
                           border: LinearRgba,
                           option_name: &str| {
            let checkbox = if value { "☑" } else { "☐" };
            let text = format!("{} {}", checkbox, label);
            Element::new(font, ElementContent::Text(text))
                .colors(ElementColors {
                    border: BorderColor::new(border),
                    bg: bg.into(),
                    text: fg.into(),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.5),
                    right: Dimension::Cells(0.5),
                    top: Dimension::Cells(0.25),
                    bottom: Dimension::Cells(0.25),
                })
                .display(DisplayType::Block)
                .item_type(UIItemType::Settings(SettingsUIAction::ToggleOption(
                    option_name.to_string(),
                )))
        };

        // Get current panes from window (for live display)
        // (idx, cwd, left, top, width, height)
        let current_panes_info: Vec<(usize, Option<String>, usize, usize, usize, usize)> = {
            use mux::Mux;
            let mux = Mux::get();
            let window_id = term_window.mux_window_id;
            let result: Vec<(usize, Option<String>, usize, usize, usize, usize)> = if let Some(mux_window) = mux.get_window(window_id) {
                if let Some(tab) = mux_window.get_active() {
                    tab.iter_panes_ignoring_zoom()
                        .iter()
                        .enumerate()
                        .map(|(idx, pos)| {
                            let cwd = pos.pane
                                .get_current_working_dir(mux::pane::CachePolicy::AllowStale)
                                .and_then(|url| {
                                    if url.scheme() == "file" {
                                        Some(url.path().to_string())
                                    } else {
                                        Some(url.to_string())
                                    }
                                });
                            (idx, cwd, pos.left, pos.top, pos.width, pos.height)
                        })
                        .collect()
                } else {
                    vec![]
                }
            } else {
                vec![]
            };
            result
        };

        // Build content panel based on selected panel with clickable GUI elements
        let content_children: Vec<Element> = match state.current_panel {
            SettingsPanel::StartupLayout => {
                let mut lines = vec![];
                lines.push(make_line("Startup Layout Configuration", &font, bg.clone(), fg.clone()));
                lines.push(make_line("", &font, bg.clone(), fg.clone()));
                lines.push(make_line("Current window panes:", &font, bg.clone(), fg.clone()));
                lines.push(make_line("", &font, bg.clone(), fg.clone()));

                if current_panes_info.is_empty() {
                    lines.push(make_line("  (No panes)", &font, bg.clone(), fg.clone()));
                } else {
                    for (idx, (_, cwd, left, top, width, height)) in current_panes_info.iter().enumerate() {
                        let cwd_str = cwd.as_deref().unwrap_or("(unknown)");
                        let pos_info = format!("pos:({},{}) size:{}x{}", left, top, width, height);
                        lines.push(make_line(
                            &format!("  Pane {}: {} [{}]", idx + 1, cwd_str, pos_info),
                            &font, bg.clone(), fg.clone()
                        ));
                    }
                }

                lines.push(make_line("", &font, bg.clone(), fg.clone()));

                if !state.startup_layout.panes.is_empty() {
                    lines.push(make_line("Saved startup layout:", &font, bg.clone(), fg.clone()));
                    for (idx, pane) in state.startup_layout.panes.iter().enumerate() {
                        let cwd = pane.cwd.as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|| "(default)".to_string());
                        lines.push(make_line(
                            &format!("  Pane {}: {}", idx + 1, cwd),
                            &font, bg.clone(), fg.clone()
                        ));
                    }
                    lines.push(make_line("", &font, bg.clone(), fg.clone()));
                }

                // Button row
                let button_row = Element::new(
                    &font,
                    ElementContent::Children(vec![
                        make_button(
                            "Save Layout",
                            &font,
                            border_color.clone(),
                            bg.clone(),
                            border_color.clone(),
                            SettingsUIAction::SaveButton,
                        ),
                        make_button(
                            "Clear",
                            &font,
                            bg.clone(),
                            fg.clone(),
                            border_color.clone(),
                            SettingsUIAction::ToggleOption("clear_layout".to_string()),
                        ),
                        make_button(
                            "Close",
                            &font,
                            bg.clone(),
                            fg.clone(),
                            border_color.clone(),
                            SettingsUIAction::CancelButton,
                        ),
                    ]),
                )
                .display(DisplayType::Block);
                lines.push(button_row);
                lines
            }
            SettingsPanel::Appearance => {
                // Helper for value row with +/- buttons
                let make_value_row =
                    |label: &str,
                     value: &str,
                     minus_action: &str,
                     plus_action: &str,
                     font: &Rc<LoadedFont>,
                     bg: LinearRgba,
                     fg: LinearRgba,
                     border: LinearRgba| {
                        Element::new(
                            font,
                            ElementContent::Children(vec![
                                // Label and value
                                Element::new(
                                    font,
                                    ElementContent::Text(format!("{}: {}", label, value)),
                                )
                                .colors(ElementColors {
                                    border: BorderColor::default(),
                                    bg: bg.clone().into(),
                                    text: fg.clone().into(),
                                })
                                .padding(BoxDimension {
                                    left: Dimension::Cells(0.0),
                                    right: Dimension::Cells(1.0),
                                    top: Dimension::Cells(0.25),
                                    bottom: Dimension::Cells(0.25),
                                })
                                .min_width(Some(Dimension::Cells(25.0)))
                                .display(DisplayType::Inline),
                                // Minus button
                                Element::new(font, ElementContent::Text("−".to_string()))
                                    .colors(ElementColors {
                                        border: BorderColor::new(border.clone()),
                                        bg: bg.clone().into(),
                                        text: fg.clone().into(),
                                    })
                                    .padding(BoxDimension {
                                        left: Dimension::Cells(0.5),
                                        right: Dimension::Cells(0.5),
                                        top: Dimension::Cells(0.1),
                                        bottom: Dimension::Cells(0.1),
                                    })
                                    .border(BoxDimension::new(Dimension::Pixels(1.0)))
                                    .margin(BoxDimension {
                                        left: Dimension::Cells(0.0),
                                        right: Dimension::Cells(0.25),
                                        top: Dimension::Cells(0.0),
                                        bottom: Dimension::Cells(0.0),
                                    })
                                    .display(DisplayType::Inline)
                                    .item_type(UIItemType::Settings(SettingsUIAction::ToggleOption(
                                        minus_action.to_string(),
                                    ))),
                                // Plus button
                                Element::new(font, ElementContent::Text("+".to_string()))
                                    .colors(ElementColors {
                                        border: BorderColor::new(border.clone()),
                                        bg: bg.clone().into(),
                                        text: fg.clone().into(),
                                    })
                                    .padding(BoxDimension {
                                        left: Dimension::Cells(0.5),
                                        right: Dimension::Cells(0.5),
                                        top: Dimension::Cells(0.1),
                                        bottom: Dimension::Cells(0.1),
                                    })
                                    .border(BoxDimension::new(Dimension::Pixels(1.0)))
                                    .display(DisplayType::Inline)
                                    .item_type(UIItemType::Settings(SettingsUIAction::ToggleOption(
                                        plus_action.to_string(),
                                    ))),
                            ]),
                        )
                        .display(DisplayType::Block)
                        .padding(BoxDimension {
                            left: Dimension::Cells(0.0),
                            right: Dimension::Cells(0.0),
                            top: Dimension::Cells(0.25),
                            bottom: Dimension::Cells(0.25),
                        })
                    };

                vec![
                    make_line("Appearance Settings", &font, bg.clone(), fg.clone()),
                    make_line("", &font, bg.clone(), fg.clone()),
                    // Font Size with +/- buttons
                    make_value_row(
                        "Font Size",
                        &format!("{:.1}", state.appearance.font_size),
                        "font_size_minus",
                        "font_size_plus",
                        &font,
                        bg.clone(),
                        fg.clone(),
                        border_color.clone(),
                    ),
                    // Color Scheme with cycle button
                    Element::new(
                        &font,
                        ElementContent::Children(vec![
                            Element::new(
                                &font,
                                ElementContent::Text(format!(
                                    "Color Scheme: {}",
                                    state.appearance.color_scheme.as_deref().unwrap_or("(default)")
                                )),
                            )
                            .colors(ElementColors {
                                border: BorderColor::default(),
                                bg: bg.clone().into(),
                                text: fg.clone().into(),
                            })
                            .padding(BoxDimension {
                                left: Dimension::Cells(0.0),
                                right: Dimension::Cells(1.0),
                                top: Dimension::Cells(0.25),
                                bottom: Dimension::Cells(0.25),
                            })
                            .min_width(Some(Dimension::Cells(25.0)))
                            .display(DisplayType::Inline),
                            // Cycle button
                            Element::new(&font, ElementContent::Text("Change".to_string()))
                                .colors(ElementColors {
                                    border: BorderColor::new(border_color.clone()),
                                    bg: bg.clone().into(),
                                    text: fg.clone().into(),
                                })
                                .padding(BoxDimension {
                                    left: Dimension::Cells(0.5),
                                    right: Dimension::Cells(0.5),
                                    top: Dimension::Cells(0.1),
                                    bottom: Dimension::Cells(0.1),
                                })
                                .border(BoxDimension::new(Dimension::Pixels(1.0)))
                                .display(DisplayType::Inline)
                                .item_type(UIItemType::Settings(SettingsUIAction::ToggleOption(
                                    "color_scheme_cycle".to_string(),
                                ))),
                        ]),
                    )
                    .display(DisplayType::Block)
                    .padding(BoxDimension {
                        left: Dimension::Cells(0.0),
                        right: Dimension::Cells(0.0),
                        top: Dimension::Cells(0.25),
                        bottom: Dimension::Cells(0.25),
                    }),
                    // Window Opacity with +/- buttons
                    make_value_row(
                        "Window Opacity",
                        &format!("{:.0}%", state.appearance.window_background_opacity * 100.0),
                        "opacity_minus",
                        "opacity_plus",
                        &font,
                        bg.clone(),
                        fg.clone(),
                        border_color.clone(),
                    ),
                    make_line("", &font, bg.clone(), fg.clone()),
                    // Button row
                    Element::new(
                        &font,
                        ElementContent::Children(vec![
                            make_button(
                                "Save",
                                &font,
                                border_color.clone(),
                                bg.clone(),
                                border_color.clone(),
                                SettingsUIAction::SaveButton,
                            ),
                            make_button(
                                "Close",
                                &font,
                                bg.clone(),
                                fg.clone(),
                                border_color.clone(),
                                SettingsUIAction::CancelButton,
                            ),
                        ]),
                    )
                    .display(DisplayType::Block),
                ]
            }
            SettingsPanel::TabBar => {
                vec![
                    make_line("Tab Bar Settings", &font, bg.clone(), fg.clone()),
                    make_line("", &font, bg.clone(), fg.clone()),
                    make_toggle(
                        "Enable Tab Bar",
                        state.tab_bar.enable_tab_bar,
                        &font,
                        bg.clone(),
                        fg.clone(),
                        border_color.clone(),
                        "enable_tab_bar",
                    ),
                    make_toggle(
                        "Tab Bar at Bottom",
                        state.tab_bar.tab_bar_at_bottom,
                        &font,
                        bg.clone(),
                        fg.clone(),
                        border_color.clone(),
                        "tab_bar_at_bottom",
                    ),
                    make_toggle(
                        "Hide if Only One Tab",
                        state.tab_bar.hide_tab_bar_if_only_one_tab,
                        &font,
                        bg.clone(),
                        fg.clone(),
                        border_color.clone(),
                        "hide_tab_bar_if_only_one_tab",
                    ),
                    make_toggle(
                        "Use Fancy Tab Bar",
                        state.tab_bar.use_fancy_tab_bar,
                        &font,
                        bg.clone(),
                        fg.clone(),
                        border_color.clone(),
                        "use_fancy_tab_bar",
                    ),
                    make_line("", &font, bg.clone(), fg.clone()),
                    // Button row
                    Element::new(
                        &font,
                        ElementContent::Children(vec![
                            make_button(
                                "Save",
                                &font,
                                border_color.clone(),
                                bg.clone(),
                                border_color.clone(),
                                SettingsUIAction::SaveButton,
                            ),
                            make_button(
                                "Close",
                                &font,
                                bg.clone(),
                                fg.clone(),
                                border_color.clone(),
                                SettingsUIAction::CancelButton,
                            ),
                        ]),
                    )
                    .display(DisplayType::Block),
                ]
            }
            SettingsPanel::Keybindings => {
                let mut lines = vec![];
                lines.push(make_line("Keybindings", &font, bg.clone(), fg.clone()));
                lines.push(make_line("", &font, bg.clone(), fg.clone()));
                lines.push(make_line("Common shortcuts:", &font, bg.clone(), fg.clone()));
                lines.push(make_line("", &font, bg.clone(), fg.clone()));

                // Get key bindings from config
                let bindings = [
                    ("Cmd+T", "New Tab"),
                    ("Cmd+W", "Close Tab"),
                    ("Cmd+D", "Split Right"),
                    ("Cmd+Shift+D", "Split Down"),
                    ("Cmd+[/]", "Navigate Panes"),
                    ("Cmd+1-9", "Switch Tab"),
                    ("Cmd+,", "Settings"),
                    ("Cmd+Shift+P", "Command Palette"),
                    ("Cmd+K", "Clear Scrollback"),
                    ("Cmd+F", "Search"),
                    ("Cmd++/-", "Zoom In/Out"),
                    ("Cmd+0", "Reset Zoom"),
                ];

                for (key, action) in bindings {
                    lines.push(make_line(
                        &format!("  {} → {}", key, action),
                        &font, bg.clone(), fg.clone()
                    ));
                }

                lines.push(make_line("", &font, bg.clone(), fg.clone()));
                lines.push(make_line("Edit wezterm.lua for custom bindings", &font, bg.clone(), fg.clone()));
                lines.push(make_line("", &font, bg.clone(), fg.clone()));
                // Button row
                lines.push(
                    Element::new(
                        &font,
                        ElementContent::Children(vec![make_button(
                            "Close",
                            &font,
                            bg.clone(),
                            fg.clone(),
                            border_color.clone(),
                            SettingsUIAction::CancelButton,
                        )]),
                    )
                    .display(DisplayType::Block),
                );
                lines
            }
        };

        let content = Element::new(&font, ElementContent::Children(content_children))
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

        // Calculate available area
        let avail_pixel_width =
            size.cols as f32 * term_window.render_metrics.cell_size.width as f32;
        let avail_pixel_height =
            size.rows as f32 * term_window.render_metrics.cell_size.height as f32;

        // Desired dialog size (60% of available area)
        let desired_width = (size.cols * 6 / 10).max(60).min(size.cols);
        let desired_height = (size.rows * 6 / 10).max(20).min(size.rows);
        let desired_pixel_width =
            desired_width as f32 * term_window.render_metrics.cell_size.width as f32;
        let desired_pixel_height =
            desired_height as f32 * term_window.render_metrics.cell_size.height as f32;

        // Center horizontally and vertically
        let x_offset = padding_left + border.left.get() as f32 +
            (avail_pixel_width - desired_pixel_width) / 2.0;
        let y_offset = top_bar_height + padding_top + border.top.get() as f32 +
            (avail_pixel_height - desired_pixel_height) / 2.0;

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
                    x_offset,
                    y_offset,
                    desired_pixel_width,
                    desired_pixel_height,
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
    /// Set the current panel (for mouse click handling)
    pub fn set_panel(&self, panel: SettingsPanel) {
        let mut state = self.state.borrow_mut();
        state.current_panel = panel;
        drop(state);
        self.element.borrow_mut().take();
    }

    /// Toggle an option (for mouse click handling)
    pub fn toggle_option(&self, option_name: &str) {
        let mut state = self.state.borrow_mut();
        match option_name {
            // Tab Bar options
            "enable_tab_bar" => {
                state.tab_bar.enable_tab_bar = !state.tab_bar.enable_tab_bar;
                state.has_changes = true;
            }
            "tab_bar_at_bottom" => {
                state.tab_bar.tab_bar_at_bottom = !state.tab_bar.tab_bar_at_bottom;
                state.has_changes = true;
            }
            "hide_tab_bar_if_only_one_tab" => {
                state.tab_bar.hide_tab_bar_if_only_one_tab =
                    !state.tab_bar.hide_tab_bar_if_only_one_tab;
                state.has_changes = true;
            }
            "use_fancy_tab_bar" => {
                state.tab_bar.use_fancy_tab_bar = !state.tab_bar.use_fancy_tab_bar;
                state.has_changes = true;
            }
            // Startup Layout options
            "clear_layout" => {
                state.startup_layout.panes.clear();
                state.startup_layout.splits.clear();
                state.has_changes = true;
            }
            // Appearance options
            "font_size_plus" => {
                state.appearance.font_size = (state.appearance.font_size + 0.5).min(72.0);
                state.has_changes = true;
            }
            "font_size_minus" => {
                state.appearance.font_size = (state.appearance.font_size - 0.5).max(6.0);
                state.has_changes = true;
            }
            "color_scheme_cycle" => {
                // Cycle through some popular color schemes
                let schemes = [
                    "Dracula",
                    "Solarized Dark",
                    "One Dark",
                    "Nord",
                    "Catppuccin Mocha",
                    "GruvboxDarkHard",
                    "Monokai Pro",
                    "Tokyo Night",
                ];
                let current = state.appearance.color_scheme.as_deref();
                let next_idx = schemes
                    .iter()
                    .position(|s| Some(*s) == current)
                    .map(|i| (i + 1) % schemes.len())
                    .unwrap_or(0);
                state.appearance.color_scheme = Some(schemes[next_idx].to_string());
                state.has_changes = true;
            }
            "opacity_plus" => {
                state.appearance.window_background_opacity =
                    (state.appearance.window_background_opacity + 0.05).min(1.0);
                state.has_changes = true;
            }
            "opacity_minus" => {
                state.appearance.window_background_opacity =
                    (state.appearance.window_background_opacity - 0.05).max(0.1);
                state.has_changes = true;
            }
            _ => {}
        }
        drop(state);
        self.element.borrow_mut().take();
    }

    /// Save current settings based on the active panel (for mouse click handling)
    pub fn save_current(&self, term_window: &mut TermWindow) {
        let panel = self.state.borrow().current_panel;
        match panel {
            SettingsPanel::StartupLayout => {
                self.save_current_layout(term_window);
            }
            SettingsPanel::Appearance => {
                self.save_appearance(term_window);
            }
            SettingsPanel::TabBar => {
                self.save_tab_bar(term_window);
            }
            SettingsPanel::Keybindings => {
                // Nothing to save for keybindings
            }
        }
        self.element.borrow_mut().take();
    }

    fn save_appearance(&self, _term_window: &mut TermWindow) {
        let state = self.state.borrow();

        if let Some(config_path) = lua_writer::LuaConfigWriter::default_config_path() {
            let writer = lua_writer::LuaConfigWriter::new(config_path.clone());
            if let Err(e) = writer.write_all(
                None,
                Some(&state.appearance),
                None,
            ) {
                log::error!("Failed to save appearance settings: {}", e);
            } else {
                log::info!("Saved appearance settings to {:?}", config_path);
            }
        }
    }

    fn save_tab_bar(&self, _term_window: &mut TermWindow) {
        let state = self.state.borrow();

        if let Some(config_path) = lua_writer::LuaConfigWriter::default_config_path() {
            let writer = lua_writer::LuaConfigWriter::new(config_path.clone());
            if let Err(e) = writer.write_all(
                None,
                None,
                Some(&state.tab_bar),
            ) {
                log::error!("Failed to save tab bar settings: {}", e);
            } else {
                log::info!("Saved tab bar settings to {:?}", config_path);
            }
        }
    }

    fn save_current_layout(&self, term_window: &mut TermWindow) {
        use mux::Mux;
        use mux::tab::SplitDirection;

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

                // Collect pane info with positions
                #[derive(Clone)]
                struct PaneInfo {
                    idx: usize,
                    left: usize,
                    top: usize,
                    width: usize,
                    height: usize,
                    cwd: Option<std::path::PathBuf>,
                }

                let mut pane_infos: Vec<PaneInfo> = vec![];

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

                    pane_infos.push(PaneInfo {
                        idx,
                        left: pos.left,
                        top: pos.top,
                        width: pos.width,
                        height: pos.height,
                        cwd,
                    });
                }

                // Sort panes by position (top-left to bottom-right)
                pane_infos.sort_by(|a, b| {
                    if a.top != b.top {
                        a.top.cmp(&b.top)
                    } else {
                        a.left.cmp(&b.left)
                    }
                });

                // Add panes to layout
                for (new_idx, info) in pane_infos.iter().enumerate() {
                    state.startup_layout.panes.push(state::PaneConfig {
                        id: new_idx,
                        cwd: info.cwd.clone(),
                        command: None,
                        name: Some(format!("Pane {}", new_idx + 1)),
                    });
                }

                // Calculate splits based on pane positions
                if pane_infos.len() > 1 {
                    let first = &pane_infos[0];
                    let total_width = pane_infos.iter().map(|p| p.left + p.width).max().unwrap_or(first.width);
                    let total_height = pane_infos.iter().map(|p| p.top + p.height).max().unwrap_or(first.height);

                    for (new_idx, info) in pane_infos.iter().enumerate().skip(1) {
                        let prev = &pane_infos[new_idx - 1];

                        // Determine split direction
                        let (direction, ratio) = if info.top == prev.top {
                            // Same row -> Right split
                            let ratio = info.width as f32 / (info.width + prev.width) as f32;
                            (state::SplitDirection::Horizontal, ratio)
                        } else if info.left == prev.left {
                            // Same column -> Bottom split
                            let ratio = info.height as f32 / (info.height + prev.height) as f32;
                            (state::SplitDirection::Vertical, ratio)
                        } else {
                            // Complex layout - estimate based on position
                            if info.left > prev.left + prev.width / 2 {
                                let ratio = info.width as f32 / total_width as f32;
                                (state::SplitDirection::Horizontal, ratio)
                            } else {
                                let ratio = info.height as f32 / total_height as f32;
                                (state::SplitDirection::Vertical, ratio)
                            }
                        };

                        state.startup_layout.splits.push(state::SplitConfig {
                            direction,
                            ratio,
                            first_pane: new_idx - 1,
                            second_pane: new_idx,
                        });
                    }
                }
            }
        }

        state.has_changes = true;

        // Save to Lua file
        if let Some(config_path) = lua_writer::LuaConfigWriter::default_config_path() {
            let writer = lua_writer::LuaConfigWriter::new(config_path.clone());
            if let Err(e) = writer.write_all(
                Some(&state.startup_layout),
                None,
                None,
            ) {
                log::error!("Failed to save startup layout: {}", e);
            } else {
                log::info!("Saved startup layout to {:?}", config_path);
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
                match state.current_panel {
                    SettingsPanel::StartupLayout => {
                        drop(state);
                        self.save_current_layout(term_window);
                    }
                    SettingsPanel::Appearance => {
                        drop(state);
                        self.save_appearance(term_window);
                    }
                    SettingsPanel::TabBar => {
                        drop(state);
                        self.save_tab_bar(term_window);
                    }
                    _ => {
                        drop(state);
                    }
                }
                self.element.borrow_mut().take();
                term_window.invalidate_modal();
                Ok(true)
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                let mut state = self.state.borrow_mut();
                if state.current_panel == SettingsPanel::StartupLayout {
                    state.startup_layout.panes.clear();
                    state.startup_layout.splits.clear();
                    state.has_changes = true;
                    drop(state);
                    self.element.borrow_mut().take();
                    term_window.invalidate_modal();
                }
                Ok(true)
            }
            KeyCode::Char('1') => {
                let mut state = self.state.borrow_mut();
                match state.current_panel {
                    SettingsPanel::Appearance => {
                        state.appearance.font_size = (state.appearance.font_size + 1.0).min(72.0);
                        state.has_changes = true;
                    }
                    SettingsPanel::TabBar => {
                        state.tab_bar.enable_tab_bar = !state.tab_bar.enable_tab_bar;
                        state.has_changes = true;
                    }
                    _ => {}
                }
                drop(state);
                self.element.borrow_mut().take();
                term_window.invalidate_modal();
                Ok(true)
            }
            KeyCode::Char('!') => {
                let mut state = self.state.borrow_mut();
                if state.current_panel == SettingsPanel::Appearance {
                    state.appearance.font_size = (state.appearance.font_size - 1.0).max(6.0);
                    state.has_changes = true;
                }
                drop(state);
                self.element.borrow_mut().take();
                term_window.invalidate_modal();
                Ok(true)
            }
            KeyCode::Char('2') => {
                let mut state = self.state.borrow_mut();
                match state.current_panel {
                    SettingsPanel::Appearance => {
                        // Cycle through some popular color schemes
                        let schemes = ["Dracula", "Solarized Dark", "One Dark", "Nord", "Catppuccin Mocha"];
                        let current = state.appearance.color_scheme.as_deref();
                        let next_idx = schemes.iter().position(|s| Some(*s) == current)
                            .map(|i| (i + 1) % schemes.len())
                            .unwrap_or(0);
                        state.appearance.color_scheme = Some(schemes[next_idx].to_string());
                        state.has_changes = true;
                    }
                    SettingsPanel::TabBar => {
                        state.tab_bar.tab_bar_at_bottom = !state.tab_bar.tab_bar_at_bottom;
                        state.has_changes = true;
                    }
                    _ => {}
                }
                drop(state);
                self.element.borrow_mut().take();
                term_window.invalidate_modal();
                Ok(true)
            }
            KeyCode::Char('3') => {
                let mut state = self.state.borrow_mut();
                match state.current_panel {
                    SettingsPanel::Appearance => {
                        state.appearance.window_background_opacity =
                            (state.appearance.window_background_opacity + 0.1).min(1.0);
                        state.has_changes = true;
                    }
                    SettingsPanel::TabBar => {
                        state.tab_bar.hide_tab_bar_if_only_one_tab = !state.tab_bar.hide_tab_bar_if_only_one_tab;
                        state.has_changes = true;
                    }
                    _ => {}
                }
                drop(state);
                self.element.borrow_mut().take();
                term_window.invalidate_modal();
                Ok(true)
            }
            KeyCode::Char('#') => {
                let mut state = self.state.borrow_mut();
                if state.current_panel == SettingsPanel::Appearance {
                    state.appearance.window_background_opacity =
                        (state.appearance.window_background_opacity - 0.1).max(0.1);
                    state.has_changes = true;
                }
                drop(state);
                self.element.borrow_mut().take();
                term_window.invalidate_modal();
                Ok(true)
            }
            KeyCode::Char('4') => {
                let mut state = self.state.borrow_mut();
                if state.current_panel == SettingsPanel::TabBar {
                    state.tab_bar.use_fancy_tab_bar = !state.tab_bar.use_fancy_tab_bar;
                    state.has_changes = true;
                }
                drop(state);
                self.element.borrow_mut().take();
                term_window.invalidate_modal();
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
