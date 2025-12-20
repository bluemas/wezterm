use crate::termwindow::box_model::*;
use config::Dimension;
use std::rc::Rc;
use wezterm_font::LoadedFont;
use window::color::LinearRgba;

pub struct ColorPicker {
    pub label: String,
    pub color: LinearRgba,
}

impl ColorPicker {
    pub fn new(label: &str, color: LinearRgba) -> Self {
        Self {
            label: label.to_string(),
            color,
        }
    }

    pub fn build(
        &self,
        font: &Rc<LoadedFont>,
        _bg: LinearRgba,
        fg: LinearRgba,
        border: LinearRgba,
    ) -> Element {
        let hex_color = format!(
            "#{:02X}{:02X}{:02X}",
            (self.color.0 * 255.0) as u8,
            (self.color.1 * 255.0) as u8,
            (self.color.2 * 255.0) as u8
        );

        let text = format!("{}: {}", self.label, hex_color);

        Element::new(font, ElementContent::Text(text))
            .colors(ElementColors {
                border: BorderColor::new(border),
                bg: self.color.clone().into(),
                text: fg.into(),
            })
            .border(BoxDimension::new(Dimension::Pixels(1.0)))
            .padding(BoxDimension {
                left: Dimension::Cells(0.5),
                right: Dimension::Cells(0.5),
                top: Dimension::Cells(0.25),
                bottom: Dimension::Cells(0.25),
            })
            .display(DisplayType::Block)
    }
}
