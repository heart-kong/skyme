//! Skyme Text Service + Key Event Sink.
//! OnKeyDown dispatches to Rime via a stored function pointer.

use super::{ref_count_dec, ref_count_inc, ComObj, IUnknownVtbl, GUID};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};

// ── IIDs ────────────────────────────────────────────────────────────────────

const IID_ITS: GUID = GUID { data1: 0xAA80E801, data2: 0x2021, data3: 0x11D2, data4: [0x93, 0xE0, 0x00, 0x60, 0xB0, 0x67, 0xB8, 0x6E] };
const IID_IKES: GUID = GUID { data1: 0xAA80E803, data2: 0x2021, data3: 0x11D2, data4: [0x93, 0xE0, 0x00, 0x60, 0xB0, 0x67, 0xB8, 0x6E] };
const IID_IUNK: GUID = GUID { data1: 0x00000000, data2: 0x0000, data3: 0x0000, data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46] };
const IID_IKSM: GUID = GUID { data1: 0x3ACC74B7, data2: 0xB3F5, data3: 0x4D10, data4: [0x9A, 0x58, 0xC6, 0x1D, 0x53, 0xB6, 0x66, 0x11] };

// ── Global state ────────────────────────────────────────────────────────────

/// RimeProcessKey function pointer loaded from librime. Type: fn(u64,i32,i32)->bool
static RIME_PROCESS_KEY: AtomicUsize = AtomicUsize::new(0);
static RIME_SESSION_ID: AtomicU64 = AtomicU64::new(0);
pub fn set_rime_process_key(fn_ptr: usize) { RIME_PROCESS_KEY.store(fn_ptr, Ordering::SeqCst); }
pub fn set_session_id(id: u64) { RIME_SESSION_ID.store(id, Ordering::SeqCst); }
static ACTIVATED: AtomicBool = AtomicBool::new(false);

// ── Shared COM helpers ──────────────────────────────────────────────────────

fn new_com_obj(vtbl: *const IUnknownVtbl) -> Box<ComObj> {
    Box::new(ComObj { lpVtbl: vtbl, ref_count: AtomicU32::new(1) })
}

// ── Text Service (ITfTextInputProcessor) ────────────────────────────────────

#[repr(C)]
pub struct TextServiceVtbl {
    base: IUnknownVtbl,
    Activate: unsafe extern "system" fn(*mut ComObj, *mut std::ffi::c_void, u32) -> i32,
    Deactivate: unsafe extern "system" fn(*mut ComObj) -> i32,
}

pub fn new_text_service() -> Box<ComObj> {
    new_com_obj(&TS_VTBL as *const _ as *const IUnknownVtbl)
}

unsafe extern "system" fn ts_qi(this: *mut ComObj, iid: *const GUID, ppv: *mut *mut std::ffi::c_void) -> i32 {
    let id = *iid;
    if id == IID_IUNK || id == IID_ITS { *ppv = this as *mut _; ((*this).lpVtbl.as_ref().unwrap().AddRef)(this); return 0; }
    *ppv = std::ptr::null_mut(); -2147467262
}
unsafe extern "system" fn ts_add_ref(this: *mut ComObj) -> u32 { ref_count_inc(&(*this).ref_count) }
unsafe extern "system" fn ts_release(this: *mut ComObj) -> u32 {
    if ref_count_dec(&(*this).ref_count) { let _ = Box::from_raw(this); 0 }
    else { (*this).ref_count.load(Ordering::Relaxed) }
}

unsafe extern "system" fn ts_activate(this: *mut ComObj, ptim: *mut std::ffi::c_void, tid: u32) -> i32 {
    log::info!("Skyme TSF: Activate (tid={})", tid);

    // QI ITfKeystrokeMgr from thread manager
    let tm = ptim as *mut ComObj;
    let mut ksm: *mut std::ffi::c_void = std::ptr::null_mut();
    let hr = ((*tm).lpVtbl.as_ref().unwrap().QueryInterface)(tm, &IID_IKSM as *const GUID, &mut ksm);
    if hr != 0 || ksm.is_null() { log::error!("Failed QI KeystrokeMgr: hr={}", hr); return hr; }

    // Create + register key event sink
    let sink = new_key_event_sink();
    let raw = Box::into_raw(sink);
    // KeystrokeMgr vtable: [IUnknown(3)], Advise=idx3
    #[repr(C)] struct KSM { _pad: [usize; 3], Advise: unsafe extern "system" fn(*mut ComObj, u32, *mut ComObj, i32) -> i32 }
    let kt = *(ksm as *mut *const KSM);
    let r = ((*kt).Advise)(ksm as *mut ComObj, tid, raw, 1);
    ((*raw).lpVtbl.as_ref().unwrap().Release)(raw);

    if r == 0 { log::info!("Key sink registered"); ACTIVATED.store(true, Ordering::SeqCst); }
    else { log::error!("AdviseKeyEventSink failed: hr={}", r); }
    r
}

unsafe extern "system" fn ts_deactivate(_this: *mut ComObj) -> i32 {
    log::info!("Skyme TSF: Deactivate");
    ACTIVATED.store(false, Ordering::SeqCst);
    0
}

const TS_VTBL: TextServiceVtbl = TextServiceVtbl {
    base: IUnknownVtbl { QueryInterface: ts_qi, AddRef: ts_add_ref, Release: ts_release },
    Activate: ts_activate, Deactivate: ts_deactivate,
};

// ── Key Event Sink (ITfKeyEventSink) ───────────────────────────────────────

#[repr(C)]
struct KeyEventSinkVtbl {
    base: IUnknownVtbl,
    OnKeyDown: unsafe extern "system" fn(*mut ComObj, *mut std::ffi::c_void, usize, usize, *mut i32) -> i32,
    OnKeyUp: unsafe extern "system" fn(*mut ComObj, *mut std::ffi::c_void, usize, usize, *mut i32) -> i32,
    OnTestKeyDown: unsafe extern "system" fn(*mut ComObj, *mut std::ffi::c_void, usize, usize, *mut i32) -> i32,
    OnTestKeyUp: unsafe extern "system" fn(*mut ComObj, *mut std::ffi::c_void, usize, usize, *mut i32) -> i32,
}

fn new_key_event_sink() -> Box<ComObj> {
    new_com_obj(&KES_VTBL as *const _ as *const IUnknownVtbl)
}

unsafe extern "system" fn kes_qi(this: *mut ComObj, iid: *const GUID, ppv: *mut *mut std::ffi::c_void) -> i32 {
    let id = *iid;
    if id == IID_IUNK || id == IID_IKES { *ppv = this as *mut _; ((*this).lpVtbl.as_ref().unwrap().AddRef)(this); return 0; }
    *ppv = std::ptr::null_mut(); -2147467262
}
unsafe extern "system" fn kes_add_ref(this: *mut ComObj) -> u32 { ref_count_inc(&(*this).ref_count) }
unsafe extern "system" fn kes_release(this: *mut ComObj) -> u32 {
    if ref_count_dec(&(*this).ref_count) { let _ = Box::from_raw(this); 0 }
    else { (*this).ref_count.load(Ordering::Relaxed) }
}

// ── Modifier key state ─────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn get_modifier_bits() -> i32 {
    use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
    // VK_* constants are VIRTUAL_KEY structs, GetKeyState takes i32 (the VK value)
    const VK_SHIFT: i32 = 0x10;
    const VK_CONTROL: i32 = 0x11;
    const VK_MENU: i32 = 0x12;
    const VK_LWIN: i32 = 0x5B;
    const VK_CAPITAL: i32 = 0x14;
    let mut m = 0i32;
    unsafe {
        if (GetKeyState(VK_SHIFT) as i32) & 0x8000 != 0 { m |= 1; }
        if (GetKeyState(VK_CONTROL) as i32) & 0x8000 != 0 { m |= 2; }
        if (GetKeyState(VK_MENU) as i32) & 0x8000 != 0 { m |= 4; }
        if (GetKeyState(VK_LWIN) as i32) & 0x8000 != 0 { m |= 8; }
        if (GetKeyState(VK_CAPITAL) as i32) & 0x0001 != 0 { m |= 16; }
    }
    m
}
#[cfg(not(target_os = "windows"))]
fn get_modifier_bits() -> i32 { 0 }

fn vk_to_rime_keycode(vk: u32, mods: i32) -> i32 {
    let shifted = (mods & 1) != 0;
    match vk {
        0x41..=0x5A if !shifted => (vk + 0x20) as i32,
        0x41..=0x5A => vk as i32,
        0x30..=0x39 => vk as i32,
        0x60..=0x69 => (vk - 0x60 + 0x30) as i32,
        0x70..=0x7B => (vk - 0x70 + 0xFFBE) as i32,
        0x08 => 0xFF08, 0x09 => 0xFF09, 0x0D => 0xFF0D, 0x1B => 0xFF1B,
        0x20 => 0x0020, 0x2E => 0xFFFF,
        0x25 => 0xFF51, 0x26 => 0xFF52, 0x27 => 0xFF53, 0x28 => 0xFF54,
        0x21 => 0xFF55, 0x22 => 0xFF56, 0x23 => 0xFF57, 0x24 => 0xFF50,
        _ => vk as i32,
    }
}

// ── Event handlers ─────────────────────────────────────────────────────────

unsafe extern "system" fn kes_on_key_down(
    _this: *mut ComObj, _ctx: *mut std::ffi::c_void, wparam: usize, _lparam: usize, pf_eaten: *mut i32,
) -> i32 {
    if !ACTIVATED.load(Ordering::Relaxed) { if !pf_eaten.is_null() { *pf_eaten = 0; } return 0; }

    let vkey = wparam as u32;
    let mods = get_modifier_bits();
    let rime_key = vk_to_rime_keycode(vkey, mods);
    let sid = RIME_SESSION_ID.load(Ordering::Relaxed);
    let fp = RIME_PROCESS_KEY.load(Ordering::Relaxed);

    log::debug!("KeyDown: vk=0x{:x} rime=0x{:x} mods=0x{:x}", vkey, rime_key, mods);

    if fp == 0 || sid == 0 { if !pf_eaten.is_null() { *pf_eaten = 0; } return 0; }

    let process_key: unsafe extern "C" fn(u64, i32, i32) -> bool = std::mem::transmute(fp);
    let handled = process_key(sid, rime_key, mods);

    if !pf_eaten.is_null() { *pf_eaten = if handled { 1 } else { 0 }; }
    if handled { log::debug!("Rime handled key 0x{:x}", rime_key); }
    0
}

unsafe extern "system" fn kes_on_key_up(
    _this: *mut ComObj, _ctx: *mut std::ffi::c_void, wparam: usize, _lparam: usize, pf_eaten: *mut i32,
) -> i32 {
    log::debug!("KeyUp: vk=0x{:x}", wparam);
    if !pf_eaten.is_null() { *pf_eaten = 0; } 0
}

unsafe extern "system" fn kes_on_test_key_down(
    _this: *mut ComObj, _ctx: *mut std::ffi::c_void, _w: usize, _l: usize, pf_eaten: *mut i32,
) -> i32 {
    if !pf_eaten.is_null() { *pf_eaten = if ACTIVATED.load(Ordering::Relaxed) { 1 } else { 0 }; } 0
}

unsafe extern "system" fn kes_on_test_key_up(
    _this: *mut ComObj, _ctx: *mut std::ffi::c_void, _w: usize, _l: usize, pf_eaten: *mut i32,
) -> i32 {
    if !pf_eaten.is_null() { *pf_eaten = 0; } 0
}

const KES_VTBL: KeyEventSinkVtbl = KeyEventSinkVtbl {
    base: IUnknownVtbl { QueryInterface: kes_qi, AddRef: kes_add_ref, Release: kes_release },
    OnKeyDown: kes_on_key_down,
    OnKeyUp: kes_on_key_up,
    OnTestKeyDown: kes_on_test_key_down,
    OnTestKeyUp: kes_on_test_key_up,
};
