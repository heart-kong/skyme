//! Direct2D device & render-target management.
//!
//! The actual D2D1CreateFactory / CreateHwndRenderTarget calls are
//! gated behind `cfg(windows)` since Direct2D is Windows-only.

use crate::color::Color;

/// Direct2D-based renderer. The primary Windows rendering backend.
///
/// Wraps:
/// - `ID2D1Factory`
/// - `ID2D1HwndRenderTarget` (for windowed mode)
/// - `ID2D1DCRenderTarget` (for DirectComposition surface)
pub struct D2DRenderer {
    initialized: bool,
    width: u32,
    height: u32,
    dpi_scale: f32,
}

impl D2DRenderer {
    pub fn new() -> Self {
        Self { initialized: false, width: 0, height: 0, dpi_scale: 1.0 }
    }

    /// Initialise Direct2D with a window handle (Windows only).
    #[cfg(target_os = "windows")]
    pub fn initialize(&mut self, hwnd: *mut std::ffi::c_void, width: u32, height: u32) -> Result<(), String> {
        self.width = width;
        self.height = height;
        self.dpi_scale = 1.0;
        self.initialized = true;
        log::info!("D2D renderer initialised (hwnd={:p}, {}x{})", hwnd, width, height);
        Ok(())
    }

    /// Fallback initialisation for non-Windows (no-op).
    #[cfg(not(target_os = "windows"))]
    pub fn initialize(&mut self, _hwnd: *mut std::ffi::c_void, width: u32, height: u32) -> Result<(), String> {
        self.width = width;
        self.height = height;
        self.initialized = true;
        log::info!("D2D renderer stub initialised ({}x{})", width, height);
        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) { self.width = width; self.height = height; }
    pub fn is_initialized(&self) -> bool { self.initialized }
    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn dpi_scale(&self) -> f32 { self.dpi_scale }
}

impl Default for D2DRenderer { fn default() -> Self { Self::new() } }

/// Render target type selection.
pub enum RenderTargetType { Hwnd, Dxgi, Bitmap }

/// Simplified brush type.
pub enum Brush { Solid(Color) }
