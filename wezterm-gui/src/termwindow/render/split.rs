use crate::termwindow::box_model::*;
use crate::termwindow::render::TripleLayerQuadAllocator;
use crate::termwindow::{UIItem, UIItemType};
use crate::utilsprites::RenderMetrics;
use config::{Dimension, DimensionContext, FontAttributes, FontWeight, TextStyle};
use mux::pane::{CachePolicy, Pane};
use mux::tab::{PositionedPane, PositionedSplit, SplitDirection};
use std::sync::Arc;

impl crate::TermWindow {
    pub fn paint_active_pane_border(
        &mut self,
        layers: &mut TripleLayerQuadAllocator,
        active_pane: &PositionedPane,
        pane: &Arc<dyn Pane>,
    ) -> anyhow::Result<()> {
        let palette = pane.palette();
        let border_color = palette.split_active.to_linear();

        let cell_width = self.render_metrics.cell_size.width as f32;
        let cell_height = self.render_metrics.cell_size.height as f32;

        let border = self.get_os_border();
        let first_row_offset = if self.show_tab_bar && !self.config.tab_bar_at_bottom {
            self.tab_bar_pixel_height()?
        } else {
            0.
        } + border.top.get() as f32;

        let (padding_left, padding_top) = self.padding_left_top();

        let pane_left = active_pane.left;
        let pane_right = active_pane.left + active_pane.width;
        let pane_top = active_pane.top;
        let pane_bottom = active_pane.top + active_pane.height;

        // Calculate pixel positions
        let left_px = pane_left as f32 * cell_width + padding_left + border.left.get() as f32;
        let right_px = pane_right as f32 * cell_width + padding_left + border.left.get() as f32;
        let top_px = pane_top as f32 * cell_height + first_row_offset + padding_top;
        let bottom_px = pane_bottom as f32 * cell_height + first_row_offset + padding_top;

        let line_width = self.render_metrics.underline_height as f32;

        // Minimum Y position for borders (below tab bar)
        let min_y = first_row_offset + padding_top;

        // Draw top border (clamp to visible area)
        let top_border_y = (top_px - (cell_height / 2.0)).max(min_y);
        self.filled_rectangle(
            layers,
            2,
            euclid::rect(
                left_px - (cell_width / 2.0),
                top_border_y,
                (pane_right - pane_left) as f32 * cell_width + cell_width,
                line_width,
            ),
            border_color,
        )?;

        // Draw bottom border
        self.filled_rectangle(
            layers,
            2,
            euclid::rect(
                left_px - (cell_width / 2.0),
                bottom_px + (cell_height / 2.0),
                (pane_right - pane_left) as f32 * cell_width + cell_width,
                line_width,
            ),
            border_color,
        )?;

        // Calculate adjusted height for left/right borders when top is clamped
        let border_top_y = (top_px - (cell_height / 2.0)).max(min_y);
        let border_height = bottom_px + (cell_height / 2.0) - border_top_y + line_width;

        // Draw left border
        self.filled_rectangle(
            layers,
            2,
            euclid::rect(
                left_px - (cell_width / 2.0),
                border_top_y,
                line_width,
                border_height,
            ),
            border_color,
        )?;

        // Draw right border
        self.filled_rectangle(
            layers,
            2,
            euclid::rect(
                right_px + (cell_width / 2.0),
                border_top_y,
                line_width,
                border_height,
            ),
            border_color,
        )?;

        Ok(())
    }

    pub fn paint_split(
        &mut self,
        layers: &mut TripleLayerQuadAllocator,
        split: &PositionedSplit,
        pane: &Arc<dyn Pane>,
        _active_pane_pos: Option<&PositionedPane>,
    ) -> anyhow::Result<()> {
        let palette = pane.palette();
        // Always use the normal split color for splits
        // The active pane border is drawn separately
        let foreground = palette.split.to_linear();

        let cell_width = self.render_metrics.cell_size.width as f32;
        let cell_height = self.render_metrics.cell_size.height as f32;

        let border = self.get_os_border();
        let first_row_offset = if self.show_tab_bar && !self.config.tab_bar_at_bottom {
            self.tab_bar_pixel_height()?
        } else {
            0.
        } + border.top.get() as f32;

        let (padding_left, padding_top) = self.padding_left_top();

        let pos_y = split.top as f32 * cell_height + first_row_offset + padding_top;
        let pos_x = split.left as f32 * cell_width + padding_left + border.left.get() as f32;

        if split.direction == SplitDirection::Horizontal {
            self.filled_rectangle(
                layers,
                2,
                euclid::rect(
                    pos_x + (cell_width / 2.0),
                    pos_y - (cell_height / 2.0),
                    self.render_metrics.underline_height as f32,
                    (1. + split.size as f32) * cell_height,
                ),
                foreground,
            )?;
            self.ui_items.push(UIItem {
                x: border.left.get() as usize
                    + padding_left as usize
                    + (split.left * cell_width as usize),
                width: cell_width as usize,
                y: padding_top as usize
                    + first_row_offset as usize
                    + split.top * cell_height as usize,
                height: split.size * cell_height as usize,
                item_type: UIItemType::Split(split.clone()),
            });
        } else {
            self.filled_rectangle(
                layers,
                2,
                euclid::rect(
                    pos_x - (cell_width / 2.0),
                    pos_y + (cell_height / 2.0),
                    (1.0 + split.size as f32) * cell_width,
                    self.render_metrics.underline_height as f32,
                ),
                foreground,
            )?;
            self.ui_items.push(UIItem {
                x: border.left.get() as usize
                    + padding_left as usize
                    + (split.left * cell_width as usize),
                width: split.size * cell_width as usize,
                y: padding_top as usize
                    + first_row_offset as usize
                    + split.top * cell_height as usize,
                height: cell_height as usize,
                item_type: UIItemType::Split(split.clone()),
            });
        }

        Ok(())
    }

    pub fn paint_pane_cwd_overlays(&mut self) -> anyhow::Result<()> {
        let panes = self.get_panes_to_render();

        // Only show CWD overlays when there are multiple panes
        if panes.len() <= 1 {
            return Ok(());
        }

        // Create bold font style for CWD display
        let config = self.config.clone();
        let bold_style = TextStyle {
            foreground: None,
            font: config
                .font
                .font
                .iter()
                .map(|f| FontAttributes {
                    weight: FontWeight::BOLD,
                    ..f.clone()
                })
                .collect(),
        };
        let font = self
            .fonts
            .resolve_font(&bold_style)
            .expect("to resolve bold font");
        let metrics = RenderMetrics::with_font_metrics(&font.metrics());

        let top_bar_height = if self.show_tab_bar && !self.config.tab_bar_at_bottom {
            self.tab_bar_pixel_height().unwrap_or(0.)
        } else {
            0.
        };
        let (padding_left, padding_top) = self.padding_left_top();
        let border = self.get_os_border();
        let top_pixel_y = top_bar_height + padding_top + border.top.get() as f32;

        let cell_width = self.render_metrics.cell_size.width as f32;
        let cell_height = self.render_metrics.cell_size.height as f32;

        // White background color for the header row
        let white_bg = window::color::LinearRgba::with_components(1.0, 1.0, 1.0, 1.0);
        // Dark text color for contrast
        let dark_text = window::color::LinearRgba::with_components(0.0, 0.0, 0.0, 1.0);

        for pos in &panes {
            // Calculate pane position
            let pane_left_px = padding_left
                + border.left.get() as f32
                + pos.left as f32 * cell_width;
            let pane_top_px = top_pixel_y + pos.top as f32 * cell_height;
            let pane_width_px = pos.width as f32 * cell_width;

            // Draw white background for the first row
            let gl_state = self.render_state.as_ref().unwrap();
            let layer = gl_state.layer_for_zindex(10)?;
            let mut layers = layer.quad_allocator();

            self.filled_rectangle(
                &mut layers,
                0,
                euclid::rect(
                    pane_left_px,
                    pane_top_px,
                    pane_width_px,
                    cell_height,
                ),
                white_bg,
            )?;

            drop(layers);

            // Get the current working directory for this pane
            let cwd = pos
                .pane
                .get_current_working_dir(CachePolicy::AllowStale)
                .and_then(|url| {
                    if url.scheme() == "file" {
                        Some(url.path().to_string())
                    } else {
                        Some(url.to_string())
                    }
                });

            let cwd_text = match cwd {
                Some(path) => path,
                None => continue,
            };

            // Truncate path if too long for the pane
            let max_chars = (pos.width as f32 * 0.9) as usize;
            let display_text = if cwd_text.len() > max_chars && max_chars > 3 {
                format!("…{}", &cwd_text[cwd_text.len() - (max_chars - 1)..])
            } else {
                cwd_text
            };

            let element = Element::new(&font, ElementContent::Text(display_text))
                .colors(ElementColors {
                    border: BorderColor::default(),
                    bg: white_bg.into(),
                    text: dark_text.into(),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.5),
                    right: Dimension::Cells(0.5),
                    top: Dimension::Cells(0.0),
                    bottom: Dimension::Cells(0.0),
                })
                .display(DisplayType::Block);

            let computed = self.compute_element(
                &LayoutContext {
                    height: DimensionContext {
                        dpi: self.dimensions.dpi as f32,
                        pixel_max: self.dimensions.pixel_height as f32,
                        pixel_cell: metrics.cell_size.height as f32,
                    },
                    width: DimensionContext {
                        dpi: self.dimensions.dpi as f32,
                        pixel_max: self.dimensions.pixel_width as f32,
                        pixel_cell: metrics.cell_size.width as f32,
                    },
                    bounds: euclid::rect(
                        pane_left_px,
                        pane_top_px,
                        pane_width_px,
                        cell_height,
                    ),
                    metrics: &metrics,
                    gl_state: self.render_state.as_ref().unwrap(),
                    zindex: 10,
                },
                &element,
            )?;

            let gl_state = self.render_state.as_ref().unwrap();
            self.render_element(&computed, gl_state, None)?;
        }

        Ok(())
    }
}
