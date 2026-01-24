use crate::{Dimensions, FontBytes};
use chargrid_runtime::{FrameBuffer, Rgba32};
use glyphon::fontdb;
use grid_2d::{Grid, Size};
use std::sync::Arc;

const FONT_NAME_NORMAL: &'static str = "user-normal";
const FONT_NAME_BOLD: &'static str = "user-bold";

fn font_data_to_font_source(data: Vec<u8>) -> fontdb::Source {
    fontdb::Source::Binary(Arc::new(data))
}

fn font_bytes_to_font_system(FontBytes { normal, bold }: FontBytes) -> glyphon::FontSystem {
    let language = fontdb::Language::English_Australia; // arbitrary
    let normal_info = fontdb::FaceInfo {
        id: fontdb::ID::dummy(),
        source: font_data_to_font_source(normal),
        index: 0,
        families: vec![(FONT_NAME_NORMAL.to_string(), language)],
        post_script_name: FONT_NAME_NORMAL.to_string(),
        style: fontdb::Style::Normal,
        weight: fontdb::Weight::NORMAL,
        stretch: fontdb::Stretch::Normal,
        monospaced: true,
    };
    let bold_info = fontdb::FaceInfo {
        id: fontdb::ID::dummy(),
        source: font_data_to_font_source(bold),
        index: 0,
        families: vec![(FONT_NAME_BOLD.to_string(), language)],
        post_script_name: FONT_NAME_BOLD.to_string(),
        style: fontdb::Style::Normal,
        weight: fontdb::Weight::BOLD,
        stretch: fontdb::Stretch::Normal,
        monospaced: true,
    };
    let mut font_system = glyphon::FontSystem::new();
    let db = font_system.db_mut();
    db.push_face_info(normal_info);
    db.push_face_info(bold_info);
    font_system
}

pub struct TextRenderer {
    font_system: glyphon::FontSystem,
    swash_cache: glyphon::SwashCache,
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    text_renderer: glyphon::TextRenderer,
    text_buffer_grid: Grid<glyphon::Buffer>,
    string_buffer: String,
    cell_dimensions: Dimensions<f64>,
}

impl TextRenderer {
    pub fn new(
        font_bytes: FontBytes,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: wgpu::TextureFormat,
        font_size_px: f32,
        cell_dimensions: Dimensions<f64>,
        grid_size: Size,
    ) -> Self {
        let mut font_system = font_bytes_to_font_system(font_bytes);
        let swash_cache = glyphon::SwashCache::new();
        let cache = glyphon::Cache::new(&device);
        let viewport = glyphon::Viewport::new(&device, &cache);
        let mut atlas = glyphon::TextAtlas::new(&device, &queue, &cache, texture_format);
        let text_renderer = glyphon::TextRenderer::new(
            &mut atlas,
            &device,
            wgpu::MultisampleState::default(),
            None,
        );
        let text_buffer_grid = Grid::new_fn(grid_size, |_coord| {
            let mut text_buffer = glyphon::Buffer::new(
                &mut font_system,
                glyphon::Metrics {
                    font_size: font_size_px,
                    line_height: cell_dimensions.height as f32,
                },
            );
            text_buffer.set_size(
                &mut font_system,
                Some(cell_dimensions.width as f32),
                Some(cell_dimensions.height as f32),
            );
            text_buffer
        });
        Self {
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            text_buffer_grid,
            string_buffer: String::new(),
            cell_dimensions,
        }
    }

    pub fn render(
        &mut self,
        frame_buffer: &FrameBuffer,
        surface_configuration: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        self.viewport.update(
            &queue,
            glyphon::Resolution {
                width: surface_configuration.width,
                height: surface_configuration.height,
            },
        );
        for (cell, text_buffer) in frame_buffer.iter().zip(self.text_buffer_grid.iter_mut()) {
            self.string_buffer.clear();
            self.string_buffer.push(cell.character);
            let attrs = if cell.bold {
                glyphon::Attrs::new()
                    .family(fontdb::Family::Name(FONT_NAME_BOLD))
                    .weight(fontdb::Weight::BOLD)
            } else {
                glyphon::Attrs::new()
                    .family(fontdb::Family::Name(FONT_NAME_NORMAL))
                    .weight(fontdb::Weight::NORMAL)
            };
            text_buffer.set_text(
                &mut self.font_system,
                self.string_buffer.as_str(),
                &attrs,
                glyphon::Shaping::Advanced,
                None,
            );
            text_buffer.shape_until_scroll(&mut self.font_system, false);
        }
        // XXX this allocates every frame
        let mut text_areas = Vec::with_capacity(self.text_buffer_grid.len());
        for ((coord, cell), text_buffer) in
            frame_buffer.enumerate().zip(self.text_buffer_grid.iter())
        {
            text_areas.push(glyphon::TextArea {
                buffer: text_buffer,
                left: coord.x as f32 * self.cell_dimensions.width as f32,
                top: coord.y as f32 * self.cell_dimensions.height as f32,
                scale: 1.,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: surface_configuration.width as i32,
                    bottom: surface_configuration.height as i32,
                },
                default_color: {
                    let Rgba32 { r, g, b, a } = cell.foreground;
                    glyphon::Color::rgba(r, g, b, a)
                },
                custom_glyphs: &[],
            });
        }
        self.text_renderer
            .prepare(
                &device,
                &queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                text_areas.drain(..),
                &mut self.swash_cache,
            )
            .unwrap();
        self.text_renderer
            .render(&self.atlas, &self.viewport, render_pass)
            .unwrap();
    }
}
