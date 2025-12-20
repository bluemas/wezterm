use crate::termwindow::box_model::*;
use config::Dimension;
use std::rc::Rc;
use wezterm_font::LoadedFont;
use window::color::LinearRgba;

pub struct Slider {
    pub label: String,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
}

impl Slider {
    pub fn new(label: &str, value: f64, min: f64, max: f64) -> Self {
        Self {
            label: label.to_string(),
            value,
            min,
            max,
            step: 1.0,
        }
    }

    pub fn with_step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    pub fn build(
        &self,
        font: &Rc<LoadedFont>,
        bg: LinearRgba,
        fg: LinearRgba,
        _border: LinearRgba,
    ) -> Element {
        let percentage = ((self.value - self.min) / (self.max - self.min) * 100.0) as usize;
        let bar_width = 20;
        let filled = (percentage * bar_width / 100).min(bar_width);
        let empty = bar_width - filled;

        let bar = format!("[{}{}]", "=".repeat(filled), " ".repeat(empty));

        let text = format!("{}: {} {:.1}", self.label, bar, self.value);

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
