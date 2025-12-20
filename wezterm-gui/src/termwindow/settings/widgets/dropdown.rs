use crate::termwindow::box_model::*;
use config::Dimension;
use std::rc::Rc;
use wezterm_font::LoadedFont;
use window::color::LinearRgba;

pub struct Dropdown {
    pub label: String,
    pub options: Vec<String>,
    pub selected: usize,
    pub expanded: bool,
}

impl Dropdown {
    pub fn new(label: &str, options: Vec<String>, selected: usize) -> Self {
        Self {
            label: label.to_string(),
            options,
            selected,
            expanded: false,
        }
    }

    pub fn selected_value(&self) -> Option<&str> {
        self.options.get(self.selected).map(|s| s.as_str())
    }

    pub fn build(
        &self,
        font: &Rc<LoadedFont>,
        bg: LinearRgba,
        fg: LinearRgba,
        border: LinearRgba,
    ) -> Element {
        let selected_text = self
            .options
            .get(self.selected)
            .cloned()
            .unwrap_or_else(|| "(none)".to_string());

        let text = format!("{}: [{}] v", self.label, selected_text);

        Element::new(font, ElementContent::Text(text))
            .colors(ElementColors {
                border: BorderColor::new(border),
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
