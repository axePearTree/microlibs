use crate::Result;
use crate::types::*;
use alloc::vec::Vec;

pub trait Backend {
    fn window_set_config(&mut self, config: WindowConfig) -> Result;

    fn texture_create(&mut self, w: u32, h: u32) -> Result<TextureData>;
    fn texture_load(&mut self, path: &str) -> Result<TextureData>;
    fn texture_destroy(&mut self, id: TextureId) -> Result;

    fn font_load(&mut self, path: &str, scale: u8) -> Result<FontData>;
    fn font_destroy(&mut self, id: FontId) -> Result;
    fn font_glyph_metrics(&mut self, font: FontId, glyph: char) -> Result<GlyphMetrics>;

    fn render_set_logical_size(&mut self, w: u32, h: u32) -> Result;
    fn render_set_target(&mut self, target: Option<TextureId>) -> Result;
    fn render_set_draw_color(&mut self, color: Color) -> Result;
    fn render_clear(&mut self) -> Result;
    fn render_present(&mut self) -> Result;
    fn render_copy_texture(&mut self, texture: TextureId, options: CopyTextureOptions) -> Result;
    fn render_fill_rect(&mut self, rect: Option<Rect>, color: Color) -> Result;
    fn render_draw_rect(&mut self, rect: Option<Rect>, color: Color) -> Result;
    fn render_font_glyph(&mut self, font: FontId, glyph: char, origin: Point) -> Result;

    fn events_pump(&mut self, events: &mut Vec<Event>);

    fn input_mouse_position(&mut self) -> Result<(i32, i32)>;

    fn system_get_millis(&mut self) -> Result<u64>;
    fn system_log(&self, s: &str);
}
