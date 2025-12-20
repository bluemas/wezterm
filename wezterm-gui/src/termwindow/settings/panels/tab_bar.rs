use crate::termwindow::box_model::*;
use crate::termwindow::settings::state::TabBarSettings;
use config::Dimension;
use std::rc::Rc;
use wezterm_font::LoadedFont;
use window::color::LinearRgba;

pub struct TabBarPanel;

impl TabBarPanel {
    pub fn build(
        settings: &TabBarSettings,
        font: &Rc<LoadedFont>,
        bg: LinearRgba,
        fg: LinearRgba,
        border: LinearRgba,
    ) -> Element {
        let mut children = vec![];

        // Title
        children.push(
            Element::new(font, ElementContent::Text("Tab Bar".to_string()))
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

        // Toggle options
        let toggle_items = [
            ("Enable Tab Bar", settings.enable_tab_bar),
            ("Tab Bar at Bottom", settings.tab_bar_at_bottom),
            (
                "Hide Tab Bar if Only One Tab",
                settings.hide_tab_bar_if_only_one_tab,
            ),
            ("Use Fancy Tab Bar", settings.use_fancy_tab_bar),
            ("Show New Tab Button", settings.show_new_tab_button),
            ("Show Close Tab Button", settings.show_close_tab_button),
        ];

        for (label, value) in toggle_items {
            let toggle_text = format!("{}: {}", label, if value { "[ON]" } else { "[OFF]" });
            children.push(
                Element::new(font, ElementContent::Text(toggle_text))
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

        Element::new(font, ElementContent::Children(children))
            .colors(ElementColors {
                border: BorderColor::default(),
                bg: bg.into(),
                text: fg.into(),
            })
            .display(DisplayType::Block)
    }
}
