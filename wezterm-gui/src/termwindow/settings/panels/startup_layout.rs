use crate::termwindow::box_model::*;
use crate::termwindow::settings::state::StartupLayout;
use config::Dimension;
use std::rc::Rc;
use wezterm_font::LoadedFont;
use window::color::LinearRgba;

pub struct StartupLayoutPanel;

impl StartupLayoutPanel {
    pub fn build(
        layout: &StartupLayout,
        font: &Rc<LoadedFont>,
        bg: LinearRgba,
        fg: LinearRgba,
        border: LinearRgba,
    ) -> Element {
        let mut children = vec![];

        // Title
        children.push(
            Element::new(font, ElementContent::Text("Startup Layout".to_string()))
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
                    "Configure the pane layout and commands to run when WezTerm starts."
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

        // Pane list
        for (idx, pane) in layout.panes.iter().enumerate() {
            let pane_label = pane
                .name
                .clone()
                .unwrap_or_else(|| format!("Pane {}", idx + 1));

            let cwd_text = pane
                .cwd
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(default)".to_string());

            let cmd_text = pane
                .command
                .as_ref()
                .map(|args| args.join(" "))
                .unwrap_or_else(|| "(shell)".to_string());

            let pane_info = format!("{}: cwd={}, cmd={}", pane_label, cwd_text, cmd_text);

            children.push(
                Element::new(font, ElementContent::Text(pane_info))
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
                    .border(BoxDimension::new(Dimension::Pixels(1.0)))
                    .margin(BoxDimension {
                        left: Dimension::Cells(0.5),
                        right: Dimension::Cells(0.5),
                        top: Dimension::Cells(0.25),
                        bottom: Dimension::Cells(0.25),
                    })
                    .display(DisplayType::Block),
            );
        }

        // Add pane button placeholder
        children.push(
            Element::new(font, ElementContent::Text("[+ Add Pane]".to_string()))
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
