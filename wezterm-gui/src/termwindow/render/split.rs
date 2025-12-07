use crate::termwindow::render::TripleLayerQuadAllocator;
use crate::termwindow::{UIItem, UIItemType};
use mux::pane::Pane;
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
}
