//! Direct2D device & render-target management.
//!
//! Uses real Direct2D + DirectWrite on Windows via the `windows` crate.
//! Falls back to a no-op stub on other platforms.

use crate::color::Color;
use crate::{Font, FontWeight, FontStyle, Renderer};
use skyme_common::Rect;

// ── types exposed for external use ──────────────────────────────────────────

/// Render target type selection.
pub enum RenderTargetType { Hwnd, Dxgi, Bitmap }

/// Simplified brush type.
pub enum Brush { Solid(Color) }

// ── platform-conditional D2DRenderer ───────────────────────────────────────

#[cfg(not(target_os = "windows"))]
mod platform {
    use crate::color::Color;
    use crate::{Font, Renderer};
    use skyme_common::Rect;

    /// Direct2D-based renderer — no-op stub (non-Windows).
    pub struct D2DRenderer {
        pub(super) initialized: bool,
        pub(super) width: u32,
        pub(super) height: u32,
        pub(super) dpi_scale: f32,
    }

    impl D2DRenderer {
        pub fn new() -> Self {
            Self { initialized: false, width: 0, height: 0, dpi_scale: 1.0 }
        }

        pub fn initialize(&mut self, _hwnd: *mut std::ffi::c_void, width: u32, height: u32) -> Result<(), String> {
            self.width = width;
            self.height = height;
            self.initialized = true;
            log::info!("D2DRenderer stub initialised ({}x{})", width, height);
            Ok(())
        }

        pub fn resize(&mut self, width: u32, height: u32) { self.width = width; self.height = height; }
        pub fn is_initialized(&self) -> bool { self.initialized }
        pub fn width(&self) -> u32 { self.width }
        pub fn height(&self) -> u32 { self.height }
        pub fn dpi_scale(&self) -> f32 { self.dpi_scale }
    }

    impl Default for D2DRenderer { fn default() -> Self { Self::new() } }

    impl Renderer for D2DRenderer {
        fn begin_frame(&mut self) -> bool { self.initialized }
        fn end_frame(&mut self) {}
        fn fill_rect(&mut self, _rect: &Rect, _color: &Color) {}
        fn stroke_rect(&mut self, _rect: &Rect, _color: &Color, _w: f32) {}
        fn draw_text(&mut self, _text: &str, _font: &Font, _color: &Color, _x: f32, _y: f32) {}
        fn draw_text_in_rect(&mut self, _text: &str, _font: &Font, _color: &Color, _rect: &Rect) {}
        fn measure_text(&self, text: &str, font: &Font) -> (f32, f32) {
            (text.len() as f32 * font.size * 0.5, font.size * 1.3)
        }
        fn push_clip_rounded_rect(&mut self, _rect: &Rect, _radius: f32) {}
        fn pop_clip(&mut self) {}
        fn size(&self) -> (f32, f32) { (self.width as f32, self.height as f32) }
        fn dpi_scale(&self) -> f32 { self.dpi_scale }
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use crate::color::Color;
    use crate::{Font, FontStyle, FontWeight, Renderer};
    use skyme_common::Rect;
    use std::ffi::c_void;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Direct2D::Common::*;
    use windows::Win32::Graphics::Direct2D::*;
    use windows::Win32::Graphics::DirectWrite::*;
    use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;

    // ── helpers ───────────────────────────────────────────────────────────────

    fn to_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    fn d2d_color(c: &Color) -> D2D1_COLOR_F {
        D2D1_COLOR_F { r: c.r, g: c.g, b: c.b, a: c.a }
    }

    fn d2d_rect(r: &Rect) -> D2D1_RECT_F {
        D2D1_RECT_F {
            left: r.x, top: r.y, right: r.x + r.width, bottom: r.y + r.height,
        }
    }

    fn map_font_weight(w: &FontWeight) -> DWRITE_FONT_WEIGHT {
        match w {
            FontWeight::Normal => DWRITE_FONT_WEIGHT_NORMAL,
            FontWeight::Bold => DWRITE_FONT_WEIGHT_BOLD,
            FontWeight::SemiBold => DWRITE_FONT_WEIGHT_SEMI_BOLD,
            FontWeight::Light => DWRITE_FONT_WEIGHT_LIGHT,
            FontWeight::Medium => DWRITE_FONT_WEIGHT_MEDIUM,
        }
    }

    fn map_font_style(s: &FontStyle) -> DWRITE_FONT_STYLE {
        match s {
            FontStyle::Normal => DWRITE_FONT_STYLE_NORMAL,
            FontStyle::Italic => DWRITE_FONT_STYLE_ITALIC,
            FontStyle::Oblique => DWRITE_FONT_STYLE_OBLIQUE,
        }
    }

    fn format_with_wrapping(fmt: &IDWriteTextFormat, wrap: bool) {
        let mode = if wrap { DWRITE_WORD_WRAPPING_WRAP } else { DWRITE_WORD_WRAPPING_NO_WRAP };
        unsafe { let _ = fmt.SetWordWrapping(mode); }
    }

    // ── context holding all Win32 resources ──────────────────────────────────

    struct D2DContext {
        factory: ID2D1Factory,
        dwrite_factory: IDWriteFactory,
        render_target: ID2D1HwndRenderTarget,
        clip_stack: Vec<ID2D1Layer>,
        hwnd: HWND,
    }

    impl D2DContext {
        fn create(hwnd: HWND, width: u32, height: u32) -> Result<Self, String> {
            // D2D factory
            let factory: ID2D1Factory = unsafe {
                D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED)
                    .map_err(|e| format!("D2D1CreateFactory: {}", e))?
            };

            // DWrite factory
            let dwrite_factory: IDWriteFactory = unsafe {
                DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)
                    .map_err(|e| format!("DWriteCreateFactory: {}", e))?
            };

            // HWND render target
            let rt_props = D2D1_RENDER_TARGET_PROPERTIES {
                _type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 0.0,
                dpiY: 0.0,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };
            let hwnd_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd,
                pixelSize: D2D1_SIZE_U { width, height },
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            };
            let render_target: ID2D1HwndRenderTarget = unsafe {
                factory.CreateHwndRenderTarget(&rt_props, &hwnd_props)
                    .map_err(|e| format!("CreateHwndRenderTarget: {}", e))?
            };

            Ok(Self { factory, dwrite_factory, render_target, clip_stack: Vec::new(), hwnd })
        }

        fn resize_render_target(&mut self, width: u32, height: u32) {
            unsafe {
                let _ = self.render_target.Resize(&D2D1_SIZE_U { width, height });
            }
        }

        fn text_format(&self, font: &Font, wrap: bool) -> Result<IDWriteTextFormat, String> {
            let family_wide = to_wide(&font.family_name);
            let format: IDWriteTextFormat = unsafe {
                self.dwrite_factory.CreateTextFormat(
                    &family_wide,
                    None,
                    map_font_weight(&font.weight),
                    map_font_style(&font.style),
                    DWRITE_FONT_STRETCH_NORMAL,
                    font.size,
                    "",
                )
                .map_err(|e| format!("CreateTextFormat: {}", e))?
            };
            format_with_wrapping(&format, wrap);
            Ok(format)
        }

        fn brush(&self, color: &Color) -> Result<ID2D1SolidColorBrush, String> {
            let d2d = d2d_color(color);
            unsafe {
                self.render_target
                    .CreateSolidColorBrush(&d2d, None)
                    .map_err(|e| format!("CreateSolidColorBrush: {}", e))
            }
        }
    }

    // ── D2DRenderer ──────────────────────────────────────────────────────────

    /// Direct2D-based renderer backed by real D2D/DWrite on Windows.
    pub struct D2DRenderer {
        initialized: bool,
        width: u32,
        height: u32,
        dpi_scale: f32,
        ctx: Option<D2DContext>,
    }

    impl D2DRenderer {
        pub fn new() -> Self {
            Self { initialized: false, width: 0, height: 0, dpi_scale: 1.0, ctx: None }
        }

        pub fn initialize(&mut self, hwnd: *mut c_void, width: u32, height: u32) -> Result<(), String> {
            if self.initialized {
                log::warn!("D2DRenderer already initialised");
                return Ok(());
            }
            let hwnd = HWND(hwnd as _);
            let ctx = D2DContext::create(hwnd, width, height)?;
            self.ctx = Some(ctx);
            self.width = width;
            self.height = height;
            self.dpi_scale = unsafe { self.ctx.as_ref().unwrap().render_target.GetDpi().0 } / 96.0;
            self.initialized = true;
            log::info!("D2DRenderer initialised (hwnd={:p}, {}x{}, dpi={})",
                hwnd.0, width, height, self.dpi_scale);
            Ok(())
        }

        pub fn resize(&mut self, width: u32, height: u32) {
            self.width = width;
            self.height = height;
            if let Some(ctx) = &mut self.ctx {
                ctx.resize_render_target(width, height);
            }
        }

        pub fn is_initialized(&self) -> bool { self.initialized }
        pub fn width(&self) -> u32 { self.width }
        pub fn height(&self) -> u32 { self.height }
        pub fn dpi_scale(&self) -> f32 { self.dpi_scale }
    }

    impl Default for D2DRenderer { fn default() -> Self { Self::new() } }

    impl Renderer for D2DRenderer {
        fn begin_frame(&mut self) -> bool {
            let ctx = match &self.ctx { Some(c) => c, None => return false, };
            unsafe { ctx.render_target.BeginDraw(); }
            true
        }

        fn end_frame(&mut self) {
            if let Some(ctx) = &self.ctx {
                let hr = unsafe { ctx.render_target.EndDraw(None, None) };
                if let Err(e) = hr {
                    log::error!("EndDraw failed: {}", e);
                }
            }
        }

        fn fill_rect(&mut self, rect: &Rect, color: &Color) {
            let ctx = match &self.ctx {
                Some(c) => c,
                None => return,
            };
            if let Ok(brush) = ctx.brush(color) {
                unsafe { ctx.render_target.FillRectangle(&d2d_rect(rect), &brush); }
            }
        }

        fn stroke_rect(&mut self, rect: &Rect, color: &Color, stroke_width: f32) {
            let ctx = match &self.ctx {
                Some(c) => c,
                None => return,
            };
            if let Ok(brush) = ctx.brush(color) {
                unsafe {
                    ctx.render_target
                        .DrawRectangle(&d2d_rect(rect), &brush, stroke_width, None);
                }
            }
        }

        fn draw_text(&mut self, text: &str, font: &Font, color: &Color, x: f32, y: f32) {
            let ctx = match &self.ctx {
                Some(c) => c,
                None => return,
            };
            let format = match ctx.text_format(font, false) {
                Ok(f) => f,
                Err(e) => { log::warn!("draw_text text_format: {}", e); return; }
            };
            let brush = match ctx.brush(color) {
                Ok(b) => b,
                Err(e) => { log::warn!("draw_text brush: {}", e); return; }
            };
            let wide: Vec<u16> = to_wide(text);
            let layout_rect = D2D1_RECT_F {
                left: x, top: y, right: x + 4096.0, bottom: y + 4096.0,
            };
            unsafe {
                ctx.render_target.DrawTextW(
                    &wide,
                    &format,
                    &layout_rect,
                    &brush,
                    D2D1_DRAW_TEXT_OPTIONS_NONE,
                    DWRITE_MEASURING_MODE_NATURAL,
                );
            }
        }

        fn draw_text_in_rect(&mut self, text: &str, font: &Font, color: &Color, rect: &Rect) {
            let ctx = match &self.ctx {
                Some(c) => c,
                None => return,
            };
            let format = match ctx.text_format(font, true) {
                Ok(f) => f,
                Err(e) => { log::warn!("draw_text_in_rect text_format: {}", e); return; }
            };
            let brush = match ctx.brush(color) {
                Ok(b) => b,
                Err(e) => { log::warn!("draw_text_in_rect brush: {}", e); return; }
            };
            let wide: Vec<u16> = to_wide(text);
            unsafe {
                ctx.render_target.DrawTextW(
                    &wide,
                    &format,
                    &d2d_rect(rect),
                    &brush,
                    D2D1_DRAW_TEXT_OPTIONS_NONE,
                    DWRITE_MEASURING_MODE_NATURAL,
                );
            }
        }

        fn measure_text(&self, text: &str, font: &Font) -> (f32, f32) {
            let ctx = match &self.ctx {
                Some(c) => c,
                None => return (text.len() as f32 * font.size * 0.5, font.size * 1.3),
            };
            let format = match ctx.text_format(font, false) {
                Ok(f) => f,
                Err(_) => return (text.len() as f32 * font.size * 0.5, font.size * 1.3),
            };
            let wide: Vec<u16> = to_wide(text);
            let layout: IDWriteTextLayout = unsafe {
                match ctx.dwrite_factory.CreateTextLayout(&wide, &format, 4096.0, 4096.0) {
                    Ok(l) => l,
                    Err(_) => return (text.len() as f32 * font.size * 0.5, font.size * 1.3),
                }
            };
            let mut metrics = DWRITE_TEXT_METRICS::default();
            let hr = unsafe { layout.GetMetrics(&mut metrics) };
            if hr.is_err() {
                return (text.len() as f32 * font.size * 0.5, font.size * 1.3);
            }
            (metrics.width, metrics.height)
        }

        fn push_clip_rounded_rect(&mut self, rect: &Rect, radius: f32) {
            let ctx = match &mut self.ctx {
                Some(c) => c,
                None => return,
            };
            let rounded = D2D1_ROUNDED_RECT {
                rect: d2d_rect(rect),
                radiusX: radius,
                radiusY: radius,
            };
            let geometry: ID2D1RoundedRectangleGeometry = unsafe {
                match ctx.factory.CreateRoundedRectangleGeometry(&rounded) {
                    Ok(g) => g,
                    Err(e) => { log::warn!("CreateRoundedRectangleGeometry: {}", e); return; }
                }
            };
            let layer: ID2D1Layer = unsafe {
                match ctx.factory.CreateLayer(None) {
                    Ok(l) => l,
                    Err(e) => { log::warn!("CreateLayer: {}", e); return; }
                }
            };
            let params = D2D1_LAYER_PARAMETERS {
                contentBounds: d2d_rect(rect),
                geometricMask: &geometry as *const ID2D1RoundedRectangleGeometry as *mut ID2D1Geometry,
                maskAntialiasMode: D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
                maskTransform: D2D_MATRIX_3X2_F {
                    _11: 1.0, _12: 0.0, _21: 0.0, _22: 1.0, _31: 0.0, _32: 0.0,
                },
                opacity: 1.0,
                maskBrush: None,
                layerOptions: D2D1_LAYER_OPTIONS_NONE,
            };
            unsafe { ctx.render_target.PushLayer(&params, &layer); }
            ctx.clip_stack.push(layer);
        }

        fn pop_clip(&mut self) {
            if let Some(ctx) = &mut self.ctx {
                ctx.clip_stack.pop();
                unsafe { ctx.render_target.PopLayer(); }
            }
        }

        fn size(&self) -> (f32, f32) {
            if let Some(ctx) = &self.ctx {
                let size = unsafe { ctx.render_target.GetSize() };
                (size.width, size.height)
            } else {
                (self.width as f32, self.height as f32)
            }
        }

        fn dpi_scale(&self) -> f32 { self.dpi_scale }
    }
}

pub use platform::D2DRenderer;
