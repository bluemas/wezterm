use crate::termwindow::box_model::*;
use crate::termwindow::settings::state::KeybindingsSettings;
use config::Dimension;
use std::rc::Rc;
use wezterm_font::LoadedFont;
use window::color::LinearRgba;

pub struct KeybindingsPanel;

impl KeybindingsPanel {
    pub fn build(
        settings: &KeybindingsSettings,
        font: &Rc<LoadedFont>,
        bg: LinearRgba,
        fg: LinearRgba,
        border: LinearRgba,
    ) -> Element {
        let mut children = vec![];

        // Title
        children.push(
            Element::new(font, ElementContent::Text("Keybindings".to_string()))
                .colors(ElementColors {
                    border: BorderColor::default(),
                    bg: border.clone().into(),
                    text: fg.clone().into(),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.5),
                    right: Dimension::Cells(0.5),
                    top: Dimension::Cells(0.25),
                    bottom: Dimension::Cells(0.25),
                })
                .display(DisplayType::Block),
        );

        // Description
        children.push(
            Element::new(
                font,
                ElementContent::Text(
                    "Configure keyboard shortcuts. (Coming soon - use wezterm.lua for now)"
                        .to_string(),
                ),
            )
            .colors(ElementColors {
                border: BorderColor::default(),
                bg: bg.clone().into(),
                text: fg.clone().into(),
            })
            .padding(BoxDimension {
                left: Dimension::Cells(0.5),
                right: Dimension::Cells(0.5),
                top: Dimension::Cells(0.5),
                bottom: Dimension::Cells(0.5),
            })
            .display(DisplayType::Block),
        );

        // Existing bindings (placeholder)
        if settings.bindings.is_empty() {
            children.push(
                Element::new(
                    font,
                    ElementContent::Text("No custom keybindings configured.".to_string()),
                )
                .colors(ElementColors {
                    border: BorderColor::default(),
                    bg: bg.clone().into(),
                    text: fg.clone().into(),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.5),
                    right: Dimension::Cells(0.5),
                    top: Dimension::Cells(0.25),
                    bottom: Dimension::Cells(0.25),
                })
                .display(DisplayType::Block),
            );
        }

        // Add binding button placeholder
        children.push(
            Element::new(font, ElementContent::Text("[+ Add Keybinding]".to_string()))
                .colors(ElementColors {
                    border: BorderColor::new(border.clone()),
                    bg: bg.clone().into(),
                    text: fg.clone().into(),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.5),
                    right: Dimension::Cells(0.5),
                    top: Dimension::Cells(0.25),
                    bottom: Dimension::Cells(0.25),
                })
                .margin(BoxDimension {
                    left: Dimension::Cells(0.5),
                    right: Dimension::Cells(0.5),
                    top: Dimension::Cells(0.5),
                    bottom: Dimension::Cells(0.25),
                })
                .display(DisplayType::Block),
        );

        Element::new(font, ElementContent::Children(children))
            .colors(ElementColors {
                border: BorderColor::default(),
                bg: bg.into(),
                text: fg.into(),
            })
            .display(DisplayType::Block)
    }
}
