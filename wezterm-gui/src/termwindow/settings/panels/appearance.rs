use crate::termwindow::box_model::*;
use crate::termwindow::settings::state::AppearanceSettings;
use config::Dimension;
use std::rc::Rc;
use wezterm_font::LoadedFont;
use window::color::LinearRgba;

pub struct AppearancePanel;

impl AppearancePanel {
    pub fn build(
        settings: &AppearanceSettings,
        font: &Rc<LoadedFont>,
        bg: LinearRgba,
        fg: LinearRgba,
        border: LinearRgba,
    ) -> Element {
        let mut children = vec![];

        // Title
        children.push(
            Element::new(font, ElementContent::Text("Appearance".to_string()))
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

        // Font size
        let font_size_text = format!("Font Size: {:.1}", settings.font_size);
        children.push(
            Element::new(font, ElementContent::Text(font_size_text))
                .colors(ElementColors {
                    border: BorderColor::default(),
                    bg: bg.clone().into(),
                    text: fg.clone().into(),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.5),
                    right: Dimension::Cells(0.5),
                    top: Dimension::Cells(0.5),
                    bottom: Dimension::Cells(0.25),
                })
                .display(DisplayType::Block),
        );

        // Color scheme
        let color_scheme_text = format!(
            "Color Scheme: {}",
            settings.color_scheme.as_deref().unwrap_or("(default)")
        );
        children.push(
            Element::new(font, ElementContent::Text(color_scheme_text))
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

        // Window opacity
        let opacity_text = format!(
            "Window Opacity: {:.0}%",
            settings.window_background_opacity * 100.0
        );
        children.push(
            Element::new(font, ElementContent::Text(opacity_text))
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

        // Background image
        let bg_image_text = format!(
            "Background Image: {}",
            settings
                .background_image
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(none)".to_string())
        );
        children.push(
            Element::new(font, ElementContent::Text(bg_image_text))
                .colors(ElementColors {
                    border: BorderColor::default(),
                    bg: bg.clone().into(),
                    text: fg.clone().into(),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.5),
                    right: Dimension::Cells(0.5),
                    top: Dimension::Cells(0.25),
                    bottom: Dimension::Cells(0.5),
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
