//! Skyme Text Service COM object + Key Event Sink.
//!
//! Implements ITfTextInputProcessor (activate/deactivate) and
//! ITfKeyEventSink (receive keyboard input from TSF).

use super::{ref_count_dec, ref_count_inc, ComObj, IUnknownVtbl, GUID, SKYME_CLSID_S};
use std::sync::atomic::AtomicU32;

// ── IIDs (GUIDs) ───────────────────────────────────────────────────────────

// ITfTextInputProcessor: AA80E801-2021-11D2-93E0-0060B067B86E
const IID_ITS: GUID = GUID { data1: 0xAA80E801, data2: 0x2021, data3: 0x11D2, data4: [0x93, 0xE0, 0x00, 0x60, 0xB0, 0x67, 0xB8, 0x6E] };
// ITfKeyEventSink: A3B8E8B9-9681-49B8-9A31-6222C279C476 (example, check actual)
const IID_IKES: GUID = GUID { data1: 0xAA80E803, data2: 0x2021, data3: 0x11D2, data4: [0x93, 0xE0, 0x00, 0x60, 0xB0, 0x67, 0xB8, 0x6E] };
// IUnknown
const IID_IUNK: GUID = GUID { data1: 0x00000000, data2: 0x0000, data3: 0x0000, data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46] };
// ITfKeystrokeMgr: 3ACC74B7-B3F5-4D10-9A58-C61D53B66611
const IID_IKSM: GUID = GUID { data1: 0x3ACC74B7, data2: 0xB3F5, data3: 0x4D10, data4: [0x9A, 0x58, 0xC6, 0x1D, 0x53, 0xB6, 0x66, 0x11] };

// ── Text Service (ITfTextInputProcessor) ────────────────────────────────────

#[repr(C)]
pub struct TextServiceVtbl {
    pub base: IUnknownVtbl,
    pub Activate: unsafe extern "system" fn(*mut ComObj, *mut std::ffi::c_void, u32) -> i32,
    pub Deactivate: unsafe extern "system" fn(*mut ComObj) -> i32,
}

pub fn new_text_service() -> Box<ComObj> {
    Box::new(ComObj { lpVtbl: &TS_VTBL as *const _ as *const super::IUnknownVtbl, ref_count: AtomicU32::new(1) })
}

unsafe extern "system" fn ts_qi(this: *mut ComObj, iid: *const GUID, ppv: *mut *mut std::ffi::c_void) -> i32 {
    let id = *iid;
    if id == IID_IUNK || id == IID_ITS {
        *ppv = this as *mut _;
        ((*this).lpVtbl.as_ref().unwrap().AddRef)(this);
        return 0;
    }
    *ppv = std::ptr::null_mut();
    -2147467262 // E_NOINTERFACE
}
unsafe extern "system" fn ts_add_ref(this: *mut ComObj) -> u32 { ref_count_inc(&(*this).ref_count) }
unsafe extern "system" fn ts_release(this: *mut ComObj) -> u32 {
    if ref_count_dec(&(*this).ref_count) { let _ = Box::from_raw(this); 0 }
    else { (*this).ref_count.load(std::sync::atomic::Ordering::Relaxed) }
}

/// Activate: register the key event sink with TSF's keystroke manager.
unsafe extern "system" fn ts_activate(this: *mut ComObj, ptim: *mut std::ffi::c_void, tid: u32) -> i32 {
    log::info!("Skyme TSF: Activate (tid={})", tid);

    // ptim is ITfThreadMgr*. QI for ITfKeystrokeMgr.
    let thread_mgr = ptim as *mut ComObj;
    let mut ksm_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let hr = ((*thread_mgr).lpVtbl.as_ref().unwrap().QueryInterface)(
        thread_mgr, &IID_IKSM as *const GUID, &mut ksm_ptr,
    );
    if hr != 0 || ksm_ptr.is_null() {
        log::error!("Failed to QI ITfKeystrokeMgr: hr={}", hr);
        return hr;
    }

    // Create a key event sink and register it.
    let sink = new_key_event_sink(this);
    let sink_raw = Box::into_raw(sink);

    // ITfKeystrokeMgr::AdviseKeyEventSink — vtable index 3
    // Signature: HRESULT(tid: u32, sink: *mut ComObj, fForeground: i32)
    struct KSMVtbl { _pad: [usize; 3], pub Advise: unsafe extern "system" fn(*mut ComObj, u32, *mut ComObj, i32) -> i32 }
    let ksm_vtbl = *(ksm_ptr as *mut *const KSMVtbl);
    let result = ((*ksm_vtbl).Advise)(ksm_ptr as *mut ComObj, tid, sink_raw, 1);

    // Release our ref (sink keeps itself alive until Unadvise)
    let sink_vtbl = *(sink_raw as *mut *const KeyEventSinkVtbl);
    ((*sink_vtbl).base.Release)(sink_raw);

    if result == 0 {
        log::info!("Skyme TSF: Key event sink registered");
        // Store the sink ptr on the text service for later cleanup
        // For now we use a static — simplified for the skeleton.
        SINK_PTR.store(sink_raw as usize, std::sync::atomic::Ordering::SeqCst);
    } else {
        log::error!("AdviseKeyEventSink failed: hr={}", result);
    }
    result
}

/// Deactivate: unregister the key event sink.
unsafe extern "system" fn ts_deactivate(this: *mut ComObj) -> i32 {
    log::info!("Skyme TSF: Deactivate");
    // Get ITfKeystrokeMgr to unadvise. 
    // For the skeleton, we just log.
    SINK_PTR.store(0, std::sync::atomic::Ordering::SeqCst);
    0
}

const TS_VTBL: TextServiceVtbl = TextServiceVtbl {
    base: IUnknownVtbl { QueryInterface: ts_qi, AddRef: ts_add_ref, Release: ts_release },
    Activate: ts_activate, Deactivate: ts_deactivate,
};

// ── Key Event Sink (ITfKeyEventSink) ────────────────────────────────────────

static SINK_PTR: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// ITfKeyEventSink vtable: IUnknown + 4 methods
#[repr(C)]
pub struct KeyEventSinkVtbl {
    pub base: IUnknownVtbl,
    pub OnKeyDown: unsafe extern "system" fn(*mut ComObj, *mut std::ffi::c_void, usize, usize, *mut i32) -> i32,
    pub OnKeyUp: unsafe extern "system" fn(*mut ComObj, *mut std::ffi::c_void, usize, usize, *mut i32) -> i32,
    pub OnTestKeyDown: unsafe extern "system" fn(*mut ComObj, *mut std::ffi::c_void, usize, usize, *mut i32) -> i32,
    pub OnTestKeyUp: unsafe extern "system" fn(*mut ComObj, *mut std::ffi::c_void, usize, usize, *mut i32) -> i32,
}

pub fn new_key_event_sink(_text_service: *mut ComObj) -> Box<ComObj> {
    Box::new(ComObj { lpVtbl: &KES_VTBL as *const _ as *const super::IUnknownVtbl, ref_count: AtomicU32::new(1) })
}

unsafe extern "system" fn kes_qi(this: *mut ComObj, iid: *const GUID, ppv: *mut *mut std::ffi::c_void) -> i32 {
    let id = *iid;
    if id == IID_IUNK || id == IID_IKES {
        *ppv = this as *mut _;
        ((*this).lpVtbl.as_ref().unwrap().AddRef)(this);
        return 0;
    }
    *ppv = std::ptr::null_mut();
    -2147467262
}
unsafe extern "system" fn kes_add_ref(this: *mut ComObj) -> u32 { ref_count_inc(&(*this).ref_count) }
unsafe extern "system" fn kes_release(this: *mut ComObj) -> u32 {
    if ref_count_dec(&(*this).ref_count) { let _ = Box::from_raw(this); 0 }
    else { (*this).ref_count.load(std::sync::atomic::Ordering::Relaxed) }
}

/// OnKeyDown: called by TSF when a key is pressed.
/// wParam = virtual key code, lParam = scan code + flags.
unsafe extern "system" fn kes_on_key_down(
    _this: *mut ComObj, _context: *mut std::ffi::c_void, wparam: usize, _lparam: usize, pf_eaten: *mut i32,
) -> i32 {
    let vkey = wparam as u32;
    log::debug!("TSF KeyDown: vkey=0x{:x}", vkey);

    // TODO: Look up rime-engine session and call process_key.
    // For now, eat all keys to verify the pipeline works.
    if !pf_eaten.is_null() {
        *pf_eaten = 1; // true — we handled it
    }
    0 // S_OK
}

unsafe extern "system" fn kes_on_key_up(
    _this: *mut ComObj, _context: *mut std::ffi::c_void, wparam: usize, _lparam: usize, pf_eaten: *mut i32,
) -> i32 {
    let vkey = wparam as u32;
    log::debug!("TSF KeyUp: vkey=0x{:x}", vkey);
    if !pf_eaten.is_null() { *pf_eaten = 0; }
    0
}

unsafe extern "system" fn kes_on_test_key_down(
    _this: *mut ComObj, _context: *mut std::ffi::c_void, _wparam: usize, _lparam: usize, pf_eaten: *mut i32,
) -> i32 {
    if !pf_eaten.is_null() { *pf_eaten = 1; } // we want all keys
    0
}

unsafe extern "system" fn kes_on_test_key_up(
    _this: *mut ComObj, _context: *mut std::ffi::c_void, _wparam: usize, _lparam: usize, pf_eaten: *mut i32,
) -> i32 {
    if !pf_eaten.is_null() { *pf_eaten = 0; }
    0
}

const KES_VTBL: KeyEventSinkVtbl = KeyEventSinkVtbl {
    base: IUnknownVtbl { QueryInterface: kes_qi, AddRef: kes_add_ref, Release: kes_release },
    OnKeyDown: kes_on_key_down,
    OnKeyUp: kes_on_key_up,
    OnTestKeyDown: kes_on_test_key_down,
    OnTestKeyUp: kes_on_test_key_up,
};
