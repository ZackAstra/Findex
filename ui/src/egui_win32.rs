/// Egui-to-Win32 rendering bridge.
/// Renders egui UI to a pixel buffer and blits to a Win32 window.
/// Pure software rasterizer — zero GPU dependencies.

use epaint::{
    Color32, Primitive, Mesh, Tessellator, TessellationOptions,
};

/// Software renderer for egui output.
pub struct EguiRenderer {
    tessellator: Option<Tessellator>,
    pixels: Vec<Color32>,
    width: usize,
    height: usize,
    font_pixels: Vec<Color32>,
    font_size: [usize; 2],
}

impl EguiRenderer {
    pub fn new() -> Self {
        EguiRenderer {
            tessellator: None,
            pixels: Vec::new(),
            width: 0,
            height: 0,
            font_pixels: Vec::new(),
            font_size: [2048, 2048],
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
            self.pixels = vec![Color32::BLACK; width * height];
        }
    }

    /// Update the font texture from the egui context.
    pub fn update_font_texture(&mut self, ctx: &egui::Context) {
        ctx.fonts(|f| {
            let atlas = f.texture_atlas();
            let locked = atlas.lock();
            self.font_size = locked.size();
            // Convert font coverage values to RGBA
            let img = locked.image();
            self.font_pixels = img.pixels.iter().map(|&coverage| {
                let c = (coverage * 255.0) as u8;
                Color32::from_rgba_premultiplied(c, c, c, c)
            }).collect();
        });
    }

    /// Render egui shapes to the pixel buffer.
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        shapes: Vec<egui::epaint::ClippedShape>,
        _textures_delta: &egui::epaint::textures::TexturesDelta,
        pixels_per_point: f32,
        bg_color: Color32,
    ) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 { return; }

        self.pixels.fill(bg_color);

        // Update font texture if needed
        self.update_font_texture(ctx);

        // Create tessellator with correct font texture size
        let mut tess = Tessellator::new(
            pixels_per_point,
            TessellationOptions::default(),
            self.font_size,
            Vec::new(), // prepared_discs
        );

        // Tessellate shapes into primitives
        let primitives = tess.tessellate_shapes(shapes);

        // Rasterize each primitive
        for prim in &primitives {
            if let Primitive::Mesh(mesh) = &prim.primitive {
                self.rasterize_mesh(mesh, &prim.clip_rect);
            }
        }
    }

    fn rasterize_mesh(&mut self, mesh: &Mesh, clip_rect: &egui::Rect) {
        let w = self.width as f32;
        let h = self.height as f32;
        let font_w = self.font_size[0] as f32;
        let font_h = self.font_size[1] as f32;

        let clip_min_x = clip_rect.min.x.max(0.0).min(w) as usize;
        let clip_min_y = clip_rect.min.y.max(0.0).min(h) as usize;
        let clip_max_x = clip_rect.max.x.max(0.0).min(w) as usize;
        let clip_max_y = clip_rect.max.y.max(0.0).min(h) as usize;

        if clip_min_x >= clip_max_x || clip_min_y >= clip_max_y {
            return;
        }

        for tri in mesh.indices.chunks(3) {
            if tri.len() < 3 { continue; }
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;

            if i0 >= mesh.vertices.len() || i1 >= mesh.vertices.len() || i2 >= mesh.vertices.len() {
                continue;
            }

            let v0 = &mesh.vertices[i0];
            let v1 = &mesh.vertices[i1];
            let v2 = &mesh.vertices[i2];

            let x0 = v0.pos.x; let y0 = v0.pos.y;
            let x1 = v1.pos.x; let y1 = v1.pos.y;
            let x2 = v2.pos.x; let y2 = v2.pos.y;

            let min_x = (x0.min(x1).min(x2).max(0.0) as usize).max(clip_min_x);
            let min_y = (y0.min(y1).min(y2).max(0.0) as usize).max(clip_min_y);
            let max_x = (x0.max(x1).max(x2).min(w) as usize).min(clip_max_x);
            let max_y = (y0.max(y1).max(y2).min(h) as usize).min(clip_max_y);

            let area = (x1 - x0) * (y2 - y0) - (x2 - x0) * (y1 - y0);
            if area.abs() < 0.0001 { continue; }
            let inv_area = 1.0 / area;

            let is_text = mesh.texture_id == epaint::TextureId::Managed(0);

            for y in min_y..max_y {
                for x in min_x..max_x {
                    let px = x as f32 + 0.5;
                    let py = y as f32 + 0.5;

                    let w0 = ((x1 - px) * (y2 - py) - (x2 - px) * (y1 - py)) * inv_area;
                    let w1 = ((x2 - px) * (y0 - py) - (x0 - px) * (y2 - py)) * inv_area;
                    let w2 = 1.0 - w0 - w1;

                    if w0 >= -0.0001 && w1 >= -0.0001 && w2 >= -0.0001 {
                        let idx = y * self.width + x;
                        if idx >= self.pixels.len() { continue; }

                        // Interpolate vertex color
                        let r = (v0.color.r() as f32 * w0 + v1.color.r() as f32 * w1 + v2.color.r() as f32 * w2) as u8;
                        let g = (v0.color.g() as f32 * w0 + v1.color.g() as f32 * w1 + v2.color.g() as f32 * w2) as u8;
                        let b = (v0.color.b() as f32 * w0 + v1.color.b() as f32 * w1 + v2.color.b() as f32 * w2) as u8;
                        let a = (v0.color.a() as f32 * w0 + v1.color.a() as f32 * w1 + v2.color.a() as f32 * w2) as u8;

                        if is_text && !self.font_pixels.is_empty() {
                            // Sample font texture for text rendering
                            let uv_x = v0.uv.x * w0 + v1.uv.x * w1 + v2.uv.x * w2;
                            let uv_y = v0.uv.y * w0 + v1.uv.y * w1 + v2.uv.y * w2;
                            let tx = (uv_x * font_w) as usize;
                            let ty = (uv_y * font_h) as usize;
                            if tx < self.font_size[0] && ty < self.font_size[1] {
                                let texel = self.font_pixels[ty * self.font_size[0] + tx];
                                // Multiply vertex color by font coverage
                                let src = Color32::from_rgba_premultiplied(
                                    (r as u16 * texel.r() as u16 / 255) as u8,
                                    (g as u16 * texel.g() as u16 / 255) as u8,
                                    (b as u16 * texel.b() as u16 / 255) as u8,
                                    (a as u16 * texel.a() as u16 / 255) as u8,
                                );
                                self.pixels[idx] = Self::blend(self.pixels[idx], src);
                            } else {
                                let src = Color32::from_rgba_premultiplied(r, g, b, a);
                                self.pixels[idx] = Self::blend(self.pixels[idx], src);
                            }
                        } else {
                            // Non-text rendering (shapes, panels, etc.)
                            let src = Color32::from_rgba_premultiplied(r, g, b, a);
                            self.pixels[idx] = Self::blend(self.pixels[idx], src);
                        }
                    }
                }
            }
        }
    }

    fn blend(dst: Color32, src: Color32) -> Color32 {
        let a = src.a();
        if a == 255 { return src; }
        if a == 0 { return dst; }
        Color32::from_rgba_premultiplied(
            ((dst.r() as u16 * (255 - a as u16) + src.r() as u16 * a as u16) / 255) as u8,
            ((dst.g() as u16 * (255 - a as u16) + src.g() as u16 * a as u16) / 255) as u8,
            ((dst.b() as u16 * (255 - a as u16) + src.b() as u16 * a as u16) / 255) as u8,
            (dst.a() as u16 + src.a() as u16 - (dst.a() as u16 * src.a() as u16) / 255) as u8,
        )
    }

    /// Get pixel buffer as BGRA bytes for GDI blitting.
    pub fn as_bgra(&self) -> Vec<u8> {
        self.pixels.iter().flat_map(|c| vec![c.b(), c.g(), c.r(), c.a()]).collect()
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
}

