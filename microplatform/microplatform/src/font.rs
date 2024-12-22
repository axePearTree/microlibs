use crate::canvas::Canvas;
use crate::text::BoundedLines;
use crate::types::{FontId, GlyphMetrics};
use crate::{
    BackendRef, BackendWeakRef, Color, CopyTextureOptions, FontData, Point, Rect, Result,
    TextAlign, TextCrossAlign, TextPadding, Texture, TextureId,
};
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::str::Chars;
use hashbrown::HashMap;

const ATLAS_WIDTH: u32 = 1024;
const ATLAS_HEIGHT: u32 = 1024;

pub struct Font(RefCell<FontInner>);

impl Font {
    pub(crate) fn new(backend: &BackendRef, path: &str, scale: u8) -> Result<Self> {
        Ok(Self(RefCell::new(FontInner::new(backend, path, scale)?)))
    }

    pub(crate) fn draw_text(
        &self,
        canvas: &Canvas,
        text: &str,
        position: Point,
        color: Color,
    ) -> Result {
        self.0.borrow_mut().draw_text(canvas, text, position, color)
    }

    pub(crate) fn draw_text_bounded(
        &self,
        canvas: &Canvas,
        text: &str,
        color: Color,
        rect: Rect,
        align: TextAlign,
        cross_align: TextCrossAlign,
        padding: TextPadding,
    ) -> Result {
        self.0.borrow_mut().draw_text_bounded(
            canvas,
            text,
            color,
            rect,
            align,
            cross_align,
            padding,
        )
    }

    pub(crate) fn atlas(&self, index: usize) -> Option<TextureId> {
        self.0.borrow().atlases.get(index).map(|a| a.texture.id)
    }

    pub(crate) fn register_text(&self, text: &str, canvas: &Canvas) -> Result {
        self.0.borrow_mut().register_glyphs(text, canvas)
    }

    pub(crate) fn line_width(&self, text: &str, canvas: &Canvas) -> Result<u32> {
        self.0.borrow_mut().line_width(text, canvas)
    }
}

struct FontInner {
    id: FontId,
    _scale: u8,
    glyphs_height: u32,
    backend: BackendWeakRef,
    atlases: Vec<FontAtlas>,
    entries: HashMap<char, FontGlyphEntry>,
}

impl FontInner {
    fn new(backend: &BackendRef, path: &str, scale: u8) -> Result<Self> {
        let FontData { id, glyphs_height } = backend.borrow_mut().font_load(path, scale)?;
        let backend = Rc::downgrade(backend);
        let atlases = vec![FontAtlas::new(
            &backend,
            ATLAS_WIDTH,
            ATLAS_HEIGHT,
            glyphs_height,
        )?];
        Ok(Self {
            id,
            _scale: scale,
            glyphs_height,
            backend,
            atlases,
            entries: HashMap::new(),
        })
    }

    fn draw_text(&mut self, canvas: &Canvas, text: &str, position: Point, color: Color) -> Result {
        self.register_glyphs(text, canvas)?;
        self.draw_text_line(position, text, canvas, color)?;
        Ok(())
    }

    fn draw_text_bounded(
        &mut self,
        canvas: &Canvas,
        text: &str,
        color: Color,
        rect: Rect,
        align: TextAlign,
        cross_align: TextCrossAlign,
        padding: TextPadding,
    ) -> Result {
        self.register_glyphs(text, canvas)?;

        let inner_rect = Rect {
            x: rect.x + padding.left as i32,
            y: rect.y + padding.top as i32,
            w: rect.w - padding.left as u32 - padding.right as u32,
            h: rect.h - padding.top as u32 - padding.bottom as u32,
        };

        let lines = text
            .bounded_lines(inner_rect.w, |c| {
                self.entries.get(&c).unwrap().metrics.advance
            })
            .collect::<Vec<_>>();

        let mut y_cursor = inner_rect.y;
        let x = inner_rect.x;

        for (line, _) in lines.iter() {
            self.draw_text_line(Point::new(x, y_cursor), line, canvas, color)?;
            y_cursor += self.glyphs_height as i32;
        }

        Ok(())
    }

    fn draw_text_line(
        &mut self,
        position: Point,
        text: &str,
        canvas: &Canvas<'_>,
        color: Color,
    ) -> Result {
        let mut x_cursor = position.x;
        Ok(for glyph in text.chars() {
            let entry = self.entries.get(&glyph).unwrap();
            let atlas = &self.atlases[entry.atlas_index];
            canvas.copy_texture(
                &atlas.texture,
                CopyTextureOptions {
                    src: Some(entry.rect),
                    dest: Some(Rect {
                        x: x_cursor,
                        y: position.y,
                        w: entry.metrics.advance,
                        h: self.glyphs_height,
                    }),
                    color_mod: Some(color),
                    ..Default::default()
                },
            )?;
            x_cursor += entry.metrics.advance as i32;
        })
    }

    fn line_width(&mut self, text: &str, canvas: &Canvas<'_>) -> Result<u32> {
        self.register_glyphs(text, canvas)?;
        let width = text
            .chars()
            .map(|c| self.entries.get(&c).unwrap().metrics.advance)
            .sum::<u32>();
        Ok(width)
    }

    fn register_glyphs(&mut self, text: &str, canvas: &Canvas<'_>) -> Result {
        let mut glyphs = text.chars();
        let mut atlas_index = self.atlases.len() - 1;
        let mut atlas = &mut self.atlases[atlas_index];
        loop {
            if register_glyphs(
                self.id,
                atlas_index,
                atlas,
                canvas,
                &mut self.entries,
                &mut glyphs,
            )? {
                break;
            } else {
                self.atlases.push(FontAtlas::new(
                    &self.backend,
                    ATLAS_WIDTH,
                    ATLAS_HEIGHT,
                    self.glyphs_height,
                )?);
                atlas_index += 1;
                atlas = &mut self.atlases[atlas_index];
            }
        }
        Ok(())
    }
}

struct FontGlyphEntry {
    atlas_index: usize,
    rect: Rect,
    metrics: GlyphMetrics,
}

struct FontAtlas {
    texture: Texture,
    glyph_height: u32,
    x_cursor: u32,
    y_cursor: u32,
}

impl FontAtlas {
    fn new(backend: &BackendWeakRef, width: u32, height: u32, glyph_height: u32) -> Result<Self> {
        let backend = backend.upgrade().unwrap();
        let texture = Texture::new_target(&backend, width, height)?;
        Ok(Self {
            texture,
            glyph_height,
            x_cursor: 0,
            y_cursor: 0,
        })
    }
}

/// Returns true if all glyphs were successfully registered inside the `FontAtlas`.
fn register_glyphs(
    font_id: FontId,
    atlas_index: usize,
    atlas: &mut FontAtlas,
    canvas: &Canvas,
    entries: &mut HashMap<char, FontGlyphEntry>,
    glyphs: &mut Chars,
) -> Result<bool> {
    let mut finished = false;
    let atlas_width = atlas.texture.width();
    let atlas_height = atlas.texture.height();
    canvas.with_target(Some(&mut atlas.texture), |canvas| {
        while let Some(glyph) = glyphs.next() {
            if entries.contains_key(&glyph) {
                continue;
            }
            let metrics = canvas.glyph_metrics(font_id, glyph)?;

            if atlas.x_cursor + metrics.advance > atlas_width {
                // go to next line
                atlas.x_cursor = 0;
                atlas.y_cursor += atlas.glyph_height;
                if atlas.y_cursor + atlas.glyph_height > atlas_height {
                    // atlas is full
                    return Ok(());
                }
            }

            // render the glyph to this target texture...
            canvas.render_glyph(
                font_id,
                glyph,
                Point::new(atlas.x_cursor as i32, atlas.y_cursor as i32),
            )?;

            entries.insert(
                glyph,
                FontGlyphEntry {
                    atlas_index,
                    rect: Rect::new(
                        atlas.x_cursor as i32,
                        atlas.y_cursor as i32,
                        metrics.advance,
                        atlas.glyph_height,
                    ),
                    metrics,
                },
            );

            atlas.x_cursor += metrics.advance;
        }
        finished = true;
        Ok(())
    })?;
    Ok(finished)
}
