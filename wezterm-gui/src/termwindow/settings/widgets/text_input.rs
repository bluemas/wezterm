use crate::termwindow::box_model::*;
use config::Dimension;
use std::rc::Rc;
use wezterm_font::LoadedFont;
use window::color::LinearRgba;

pub struct TextInput {
    pub value: String,
    pub placeholder: String,
    pub focused: bool,
}

impl TextInput {
    pub fn new(placeholder: &str) -> Self {
        Self {
            value: String::new(),
            placeholder: placeholder.to_string(),
            focused: false,
        }
    }

    pub fn with_value(mut self, value: &str) -> Self {
        self.value = value.to_string();
        self
    }

    pub fn build(
        &self,
        font: &Rc<LoadedFont>,
        bg: LinearRgba,
        fg: LinearRgba,
        border: LinearRgba,
    ) -> Element {
        let display_text = if self.value.is_empty() {
            self.placeholder.clone()
        } else {
            self.value.clone()
        };

        let border_color = if self.focused {
            border.clone()
        } else {
            LinearRgba::with_components(border.0 * 0.5, border.1 * 0.5, border.2 * 0.5, border.3)
        };

        Element::new(font, ElementContent::Text(display_text))
            .colors(ElementColors {
                border: BorderColor::new(border_color),
                bg: bg.into(),
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
