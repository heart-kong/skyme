//! OCR plugin.
//!
//! Provides screen OCR candidates — extracts text from the screen
//! region near the cursor when triggered.
//!
//! This is a scaffold. Connect to Tesseract, Windows OCR API, or
//! a cloud OCR service for actual functionality.

use crate::Plugin;
use skyme_common::{Candidate, Modifiers, event::Event};

pub struct OcrPlugin {
    active: bool,
}

impl OcrPlugin {
    pub fn new() -> Self { Self { active: false } }

    /// Capture screen region and run OCR.
    /// Stub — returns empty list.
    fn ocr_screen(&self) -> Vec<String> {
        if !self.active { return Vec::new(); }
        log::info!("OCR: screen capture requested (stub)");
        Vec::new()
    }
}

impl Default for OcrPlugin { fn default() -> Self { Self::new() } }

impl Plugin for OcrPlugin {
    fn name(&self) -> &str { "ocr" }

    fn on_key(&mut self, keycode: u32, modifiers: Modifiers) -> bool {
        // Ctrl+Shift+O triggers OCR
        if keycode == 0x4F && modifiers.bits() & 0x03 == 0x03 {
            self.active = !self.active;
            if self.active {
                let results = self.ocr_screen();
                log::info!("OCR: {} texts found", results.len());
            }
            true
        } else {
            false
        }
    }

    fn on_candidate(&mut self, _candidates: &mut Vec<Candidate>) {
        if !self.active { return; }
        // Would inject OCR results as candidates
        self.active = false; // one-shot
    }

    fn on_event(&mut self, event: &Event) {
        if let Event::PluginEvent { plugin, payload } = event {
            if plugin == "ocr" && payload == "capture" {
                self.active = true;
            }
        }
    }
}
