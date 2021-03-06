
use rusttype;
use glium;

pub use descartes::{N, P3, P2, V3, V4, M4, Iso3, Persp3, ToHomogeneous, Norm, Into2d, Into3d,
                    WithUniqueOrthogonal, Inverse, Rotate};
use std::borrow::Cow;

use glium::Surface;
use glium::backend::glutin_backend::GlutinFacade;

mod font;
mod glyph;
mod rich_text;

pub use self::font::{Font, FontBank, FontDescription};
pub use self::glyph::{Glyph, GlyphIter};
pub use self::rich_text::{RichText, Formatting};

pub struct TextRenderer {
    text_program: glium::Program,
    text_cache_tex: glium::Texture2d,
    text_cache: rusttype::gpu_cache::Cache,

    font_bank: FontBank,
}

impl TextRenderer {
    pub fn new(window: &GlutinFacade, dpi_factor: f32) -> TextRenderer {
        let text_cache_size = 512 * dpi_factor as u32;

        let text_cache_tex = glium::Texture2d::with_format(
            window,
            glium::texture::RawImage2d {
                data: Cow::Owned(vec![128u8; (text_cache_size * text_cache_size) as usize]),
                width: text_cache_size, height: text_cache_size,
                format: glium::texture::ClientFormat::U8
            },
            glium::texture::UncompressedFloatFormat::U8,
            glium::texture::MipmapsOption::NoMipmap
        ).unwrap();

        #[allow(redundant_closure)]
        TextRenderer {
            text_program: program!(window, 140 => {
                vertex: include_str!("../shader/text_140.glslv"),
                fragment: include_str!("../shader/text_140.glslf")
            })
                .unwrap(),
            text_cache_tex: text_cache_tex,
            text_cache: rusttype::gpu_cache::Cache::new(text_cache_size, text_cache_size, 0.1, 0.1),
            font_bank: FontBank::new(dpi_factor),
        }
    }

    pub fn render_text(&mut self,
                       screen: (f32, f32),
                       window: &GlutinFacade,
                       target: &mut glium::Frame,
                       text: &[RichText]) {
        for text in text {
            for glyph in text.glyphs_iter() {
                let positioned = glyph.positioned();
                self.text_cache.queue_glyph(0, positioned.clone());
            }
        }

        {
            let text_cache_tex = &mut self.text_cache_tex;
            self.text_cache
                .cache_queued(|rect, data| {
                    let rect = glium::Rect {
                        left: rect.min.x,
                        bottom: rect.min.y,
                        width: rect.width(),
                        height: rect.height(),
                    };

                    let image = glium::texture::RawImage2d {
                        data: Cow::Borrowed(data),
                        width: rect.width,
                        height: rect.height,
                        format: glium::texture::ClientFormat::U8,
                    };

                    text_cache_tex.main_level().write(rect, image);
                })
                .unwrap();
        }

        for text in text {
            let vertices = text.vertices(self, screen);
            let vertices = glium::VertexBuffer::new(window, &vertices).unwrap();

            let text_uniforms = uniform! {
                tex: self.text_cache_tex
                    .sampled()
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
            };

            target.draw(&vertices,
                      glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                      &self.text_program,
                      &text_uniforms,
                      &glium::DrawParameters {
                          blend: glium::Blend::alpha_blending(),
                          ..Default::default()
                      })
                .unwrap();
        }
    }

    pub fn cache_rect_for(&self,
                          glyph: &Glyph)
                          -> Option<(rusttype::Rect<f32>, rusttype::Rect<i32>)> {
        let positioned = glyph.positioned();
        match self.text_cache.rect_for(0, positioned) {
            Ok(Some(rects)) => Some(rects),
            _ => None,
        }
    }

    #[inline]
    pub fn font_bank(&self) -> &FontBank {
        &self.font_bank
    }
}

#[derive(Copy, Clone)]
pub struct TextVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
    color: [f32; 4],
}
implement_vertex!(TextVertex, position, tex_coords, color);
