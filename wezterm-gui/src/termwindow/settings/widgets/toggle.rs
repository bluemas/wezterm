use crate::termwindow::box_model::*;
use config::Dimension;
use std::rc::Rc;
use wezterm_font::LoadedFont;
use window::color::LinearRgba;

pub struct Toggle {
    pub label: String,
    pub value: bool,
}

impl Toggle {
    pub fn new(label: &str, value: bool) -> Self {
        Self {
            label: label.to_string(),
            value,
        }
    }

    pub fn build(
        &self,
        font: &Rc<LoadedFont>,
        bg: LinearRgba,
        fg: LinearRgba,
        _border: LinearRgba,
    ) -> Element {
        let toggle_indicator = if self.value { "[ON]" } else { "[OFF]" };
        let text = format!("{}: {}", self.label, toggle_indicator);

        Element::new(font, ElementContent::Text(text))
            .colors(ElementColors {
                border: BorderColor::default(),
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
    }
}
