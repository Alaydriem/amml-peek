use peek_core::structs::Rgb;

/// Every colour, alpha, border and draw-layer value of the panel, in one place. The cell size, padding and gap are
/// in `PreviewLayout`.
pub struct PanelStyle;

impl PanelStyle {
    pub const FRAME: Rgb = Rgb { r: 0.10, g: 0.10, b: 0.10 };
    pub const FRAME_ALPHA: f32 = 0.92;
    pub const CELL: Rgb = Rgb { r: 0.23, g: 0.23, b: 0.23 };
    pub const CELL_ALPHA: f32 = 1.0;
    pub const BORDER: Rgb = Rgb { r: 0.55, g: 0.35, b: 0.70 };
    pub const BORDER_ALPHA: f32 = 1.0;
    pub const BORDER_THICKNESS: i32 = 1;
    pub const BAR_BACKGROUND: Rgb = Rgb { r: 0.0, g: 0.0, b: 0.0 };
    pub const BAR_ALPHA: f32 = 1.0;
    // the vanilla bundle bar blue, #7086FF
    pub const FILL_BAR: Rgb = Rgb { r: 112.0 / 255.0, g: 134.0 / 255.0, b: 1.0 };
    pub const OUTLINE: Rgb = Rgb { r: 1.0, g: 1.0, b: 1.0 };
    pub const OUTLINE_ALPHA: f32 = 1.0;
    pub const OUTLINE_THICKNESS: i32 = 1;
    pub const ITEM_ALPHA: f32 = 1.0;
    pub const ITEM_Z_ORDER: i32 = 17;
    // the glint overlay is added in one pass at this alpha, which matches the vanilla slot
    pub const GLINT_ALPHA: f32 = 1.35;
}
