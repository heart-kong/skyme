//! COM infrastructure for the Skyme TSF text service.
//!
//! Uses raw COM vtables — no macros needed.

pub mod class_factory;
pub mod text_service;

pub use class_factory::class_factory;
pub use text_service::new_service;

use std::sync::atomic::{AtomicU32, Ordering};

/// COM vtable for IUnknown (the first 3 methods of every COM interface).
#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface: unsafe extern "system" fn(*mut ComObj, *const GUID, *mut *mut std::ffi::c_void) -> i32,
    pub AddRef: unsafe extern "system" fn(*mut ComObj) -> u32,
    pub Release: unsafe extern "system" fn(*mut ComObj) -> u32,
}

/// Base COM object (must be the first field in any COM struct).
#[repr(C)]
pub struct ComObj {
    pub lpVtbl: *const IUnknownVtbl,
    pub ref_count: AtomicU32,
}

/// 16-byte GUID.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GUID {
    pub data1: u32, pub data2: u16, pub data3: u16, pub data4: [u8; 8],
}

impl Default for GUID { fn default() -> Self { Self { data1: 0, data2: 0, data3: 0, data4: [0; 8] } } }

pub const SKYME_CLSID_S: &str = "B3B5E5D0-1A2B-3C4D-5E6F-7890ABCDEF01";

pub fn ref_count_inc(rc: &AtomicU32) -> u32 { rc.fetch_add(1, Ordering::SeqCst) + 1 }
pub fn ref_count_dec(rc: &AtomicU32) -> bool { rc.fetch_sub(1, Ordering::SeqCst) == 1 }
