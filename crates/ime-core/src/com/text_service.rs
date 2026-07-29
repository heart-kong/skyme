//! Skyme Text Service + Key Event Sink + Commit to TSF.
//!
//! OnKeyDown → RimeProcessKey → If commit → RequestEditSession → InsertTextAtSelection.

use super::{ref_count_dec, ref_count_inc, ComObj, IUnknownVtbl, GUID};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};

// ── IIDs ────────────────────────────────────────────────────────────────────

const IID_ITS: GUID = GUID { data1: 0xAA80E801, data2: 0x2021, data3: 0x11D2, data4: [0x93, 0xE0, 0x00, 0x60, 0xB0, 0x67, 0xB8, 0x6E] };
const IID_IKES: GUID = GUID { data1: 0xAA80E803, data2: 0x2021, data3: 0x11D2, data4: [0x93, 0xE0, 0x00, 0x60, 0xB0, 0x67, 0xB8, 0x6E] };
const IID_IUNK: GUID = GUID { data1: 0x00000000, data2: 0x0000, data3: 0x0000, data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46] };
const IID_IKSM: GUID = GUID { data1: 0x3ACC74B7, data2: 0xB3F5, data3: 0x4D10, data4: [0x9A, 0x58, 0xC6, 0x1D, 0x53, 0xB6, 0x66, 0x11] };
const IID_ITES: GUID = GUID { data1: 0xEA1685A0, data2: 0x1571, data3: 0x11D2, data4: [0x93, 0xCE, 0x00, 0x60, 0xB0, 0x67, 0xB8, 0x6E] };

// ── Rime function pointers (loaded from rime.dll) ─────────────────────────

static FN_PROCESS_KEY: AtomicUsize = AtomicUsize::new(0); // fn(u64,i32,i32)->bool
static FN_GET_COMMIT: AtomicUsize = AtomicUsize::new(0);  // fn(u64,*mut RimeCommit)->bool
static FN_FREE_COMMIT: AtomicUsize = AtomicUsize::new(0); // fn(*mut RimeCommit)
static RIME_SESSION_ID: AtomicU64 = AtomicU64::new(0);

pub fn set_rime_fns(process_key: usize, get_commit: usize, free_commit: usize) {
    FN_PROCESS_KEY.store(process_key, Ordering::SeqCst);
    FN_GET_COMMIT.store(get_commit, Ordering::SeqCst);
    FN_FREE_COMMIT.store(free_commit, Ordering::SeqCst);
}
pub fn set_session_id(id: u64) { RIME_SESSION_ID.store(id, Ordering::SeqCst); }
static ACTIVATED: AtomicBool = AtomicBool::new(false);

#[repr(C)]
struct RimeCommit { data_size: i32, text: *mut u16 }

// ── COM helpers ────────────────────────────────────────────────────────────

fn new_com_obj(vtbl: *const IUnknownVtbl) -> Box<ComObj> {
    Box::new(ComObj { lpVtbl: vtbl, ref_count: AtomicU32::new(1) })
}

// ── Text Service (ITfTextInputProcessor) ────────────────────────────────────

#[repr(C)]
struct TextServiceVtbl {
    base: IUnknownVtbl,
    Activate: unsafe extern "system" fn(*mut ComObj, *mut std::ffi::c_void, u32) -> i32,
    Deactivate: unsafe extern "system" fn(*mut ComObj) -> i32,
}

pub fn new_text_service() -> Box<ComObj> { new_com_obj(&TS_VTBL as *const _ as *const IUnknownVtbl) }

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

unsafe extern "system" fn ts_activate(_this: *mut ComObj, ptim: *mut std::ffi::c_void, tid: u32) -> i32 {
    log::info!("Skyme TSF: Activate (tid={})", tid);
    let tm = ptim as *mut ComObj;
    let mut ksm: *mut std::ffi::c_void = std::ptr::null_mut();
    let hr = ((*tm).lpVtbl.as_ref().unwrap().QueryInterface)(tm, &IID_IKSM as *const GUID, &mut ksm);
    if hr != 0 || ksm.is_null() { log::error!("QI KeystrokeMgr failed: hr={}", hr); return hr; }
    let sink = new_key_event_sink();
    let raw = Box::into_raw(sink);
    #[repr(C)] struct KSM { _pad: [usize; 3], Advise: unsafe extern "system" fn(*mut ComObj, u32, *mut ComObj, i32) -> i32 }
    let kt = *(ksm as *mut *const KSM);
    let r = ((*kt).Advise)(ksm as *mut ComObj, tid, raw, 1);
    let com_raw = raw as *mut ComObj; ((*com_raw).lpVtbl.as_ref().unwrap().Release)(com_raw);
    if r == 0 { ACTIVATED.store(true, Ordering::SeqCst); log::info!("Key sink registered"); }
    else { log::error!("AdviseKeyEventSink failed: hr={}", r); }
    r
}

unsafe extern "system" fn ts_deactivate(_this: *mut ComObj) -> i32 {
    log::info!("Skyme TSF: Deactivate"); ACTIVATED.store(false, Ordering::SeqCst); 0
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

fn new_key_event_sink() -> Box<ComObj> { new_com_obj(&KES_VTBL as *const _ as *const IUnknownVtbl) }

unsafe extern "system" fn kes_qi(this: *mut ComObj, iid: *const GUID, ppv: *mut *mut std::ffi::c_void) -> i32 {
    let id = *iid; if id == IID_IUNK || id == IID_IKES { *ppv = this as *mut _; ((*this).lpVtbl.as_ref().unwrap().AddRef)(this); return 0; }
    *ppv = std::ptr::null_mut(); -2147467262
}
unsafe extern "system" fn kes_add_ref(this: *mut ComObj) -> u32 { ref_count_inc(&(*this).ref_count) }
unsafe extern "system" fn kes_release(this: *mut ComObj) -> u32 {
    if ref_count_dec(&(*this).ref_count) { let _ = Box::from_raw(this); 0 }
    else { (*this).ref_count.load(Ordering::Relaxed) }
}

// ── Get modifier keys ─────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn mod_bits() -> i32 {
    use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
    let mut m = 0i32;
    unsafe {
        if (GetKeyState(0x10) as i32) & 0x8000 != 0 { m |= 1; }
        if (GetKeyState(0x11) as i32) & 0x8000 != 0 { m |= 2; }
        if (GetKeyState(0x12) as i32) & 0x8000 != 0 { m |= 4; }
        if (GetKeyState(0x5B) as i32) & 0x8000 != 0 { m |= 8; }
        if (GetKeyState(0x14) as i32) & 0x0001 != 0 { m |= 16; }
    }
    m
}
#[cfg(not(target_os = "windows"))] fn mod_bits() -> i32 { 0 }

fn vk_to_rime(vk: u32, mods: i32) -> i32 {
    let s = (mods & 1) != 0;
    match vk {
        0x41..=0x5A if !s => (vk + 0x20) as i32, 0x41..=0x5A => vk as i32,
        0x30..=0x39 => vk as i32, 0x60..=0x69 => (vk - 0x60 + 0x30) as i32,
        0x70..=0x7B => (vk - 0x70 + 0xFFBE) as i32,
        0x08 => 0xFF08, 0x09 => 0xFF09, 0x0D => 0xFF0D, 0x1B => 0xFF1B,
        0x20 => 0x0020, 0x2E => 0xFFFF,
        0x25 => 0xFF51, 0x26 => 0xFF52, 0x27 => 0xFF53, 0x28 => 0xFF54,
        0x21 => 0xFF55, 0x22 => 0xFF56, 0x23 => 0xFF57, 0x24 => 0xFF50,
        _ => vk as i32,
    }
}

// ── OnKeyDown: Rime → commit text → TSF edit session → InsertTextAtSelection ─

unsafe extern "system" fn kes_on_key_down(
    _this: *mut ComObj, ctx_ptr: *mut std::ffi::c_void, wparam: usize, _lparam: usize, pf_eaten: *mut i32,
) -> i32 {
    if !ACTIVATED.load(Ordering::Relaxed) { if !pf_eaten.is_null() { *pf_eaten = 0; } return 0; }

    let vkey = wparam as u32;
    let mods = mod_bits();
    let rk = vk_to_rime(vkey, mods);
    let sid = RIME_SESSION_ID.load(Ordering::Relaxed);
    let fp_pk = FN_PROCESS_KEY.load(Ordering::Relaxed);

    if fp_pk == 0 || sid == 0 { if !pf_eaten.is_null() { *pf_eaten = 0; } return 0; }

    let process: unsafe extern "C" fn(u64, i32, i32) -> bool = std::mem::transmute(fp_pk);
    let handled = process(sid, rk, mods);

    if handled && !ctx_ptr.is_null() {
        handle_commit(sid, ctx_ptr as *mut ComObj);
    }

    if !pf_eaten.is_null() { *pf_eaten = if handled { 1 } else { 0 }; }
    0
}

/// After Rime processes a key, check for committed text and push to TSF.
unsafe fn handle_commit(sid: u64, ctx: *mut ComObj) {
    let fp_gc = FN_GET_COMMIT.load(Ordering::Relaxed);
    let fp_fc = FN_FREE_COMMIT.load(Ordering::Relaxed);
    if fp_gc == 0 || fp_fc == 0 { return; }

    let get_commit: unsafe extern "C" fn(u64, *mut RimeCommit) -> bool = std::mem::transmute(fp_gc);
    let free_commit: unsafe extern "C" fn(*mut RimeCommit) = std::mem::transmute(fp_fc);

    let mut commit = RimeCommit { data_size: 8, text: std::ptr::null_mut() };
    if !get_commit(sid, &mut commit) || commit.text.is_null() { return; }

    // Get UTF-16 string length
    let mut len = 0usize;
    while *commit.text.offset(len as isize) != 0 { len += 1; }
    if len == 0 { free_commit(&mut commit); return; }

    let text = std::slice::from_raw_parts(commit.text, len).to_vec();
    log::info!("Rime commit: {} chars", len);

    // Start TSF edit session to insert the text
    let session = new_edit_session(ctx, text.clone());
    let raw = Box::into_raw(session);

    // ITfContext::RequestEditSession @ vtable index 3
    #[repr(C)] struct CtxVtbl { _pad: [usize; 3], ReqEdit: unsafe extern "system" fn(*mut ComObj, *mut ComObj, u32, *mut i32) -> i32 }
    let ct = *(ctx as *mut *const CtxVtbl);
    let mut hr: i32 = 0;
    // TF_ES_SYNC(1) | TF_ES_READWRITE(2) = 3
    let r = ((*ct).ReqEdit)(ctx, raw as *mut ComObj, 3, &mut hr);
    let com_raw = raw as *mut ComObj; ((*com_raw).lpVtbl.as_ref().unwrap().Release)(com_raw);
    if r != 0 { log::error!("RequestEditSession failed: {}", r); }

    free_commit(&mut commit);
}

// ── ITfEditSession COM object ─────────────────────────────────────────────

#[repr(C)]
struct EditSessionVtbl {
    base: IUnknownVtbl,
    DoEditSession: unsafe extern "system" fn(*mut EditSessionObj, u32) -> i32,
}

#[repr(C)]
struct EditSessionObj {
    base: ComObj,
    context: *mut ComObj,  // ITfContext*
    text: Vec<u16>,
}

fn new_edit_session(ctx: *mut ComObj, text: Vec<u16>) -> Box<EditSessionObj> {
    Box::new(EditSessionObj {
        base: ComObj { lpVtbl: &ES_VTBL as *const _ as *const IUnknownVtbl, ref_count: AtomicU32::new(1) },
        context: ctx, text,
    })
}

unsafe extern "system" fn es_qi(this: *mut ComObj, iid: *const GUID, ppv: *mut *mut std::ffi::c_void) -> i32 {
    let id = *iid;
    if id == IID_IUNK || id == IID_ITES { *ppv = this as *mut _; ((*this).lpVtbl.as_ref().unwrap().AddRef)(this); return 0; }
    *ppv = std::ptr::null_mut(); -2147467262
}
unsafe extern "system" fn es_add_ref(this: *mut ComObj) -> u32 { ref_count_inc(&(*this).ref_count) }
unsafe extern "system" fn es_release(this: *mut ComObj) -> u32 {
    if ref_count_dec(&(*this).ref_count) { let _ = Box::from_raw(this as *mut EditSessionObj); 0 }
    else { (*this).ref_count.load(Ordering::Relaxed) }
}

/// DoEditSession: TSF gives us an edit cookie. Query ITfInsertAtSelection and insert text.
unsafe extern "system" fn es_do_edit_session(this: *mut EditSessionObj, ec: u32) -> i32 {
    let ctx = (*this).context;
    let text = &(*this).text;
    if ctx.is_null() || text.is_empty() { return 0; }

    // QI for ITfInsertAtSelection
    let iid_ias = IID_IAS;
    let mut ias: *mut std::ffi::c_void = std::ptr::null_mut();
    let hr = ((*ctx).lpVtbl.as_ref().unwrap().QueryInterface)(ctx, &iid_ias as *const GUID, &mut ias);
    if hr != 0 || ias.is_null() { log::error!("QI InsertAtSelection failed"); return 0; }

    // ITfInsertAtSelection::InsertTextAtSelection is at vtable index 3
    // Signature: fn(*mut ComObj, u32, u32, *const u16, u32, *mut u32) -> i32
    #[repr(C)] struct IasVtbl { _pad: [usize; 3], Insert: unsafe extern "system" fn(*mut ComObj, u32, u32, *const u16, u32, *mut u32) -> i32 }
    let ias_v = *(ias as *mut *const IasVtbl);
    let mut inserted: u32 = 0;
    // TF_IAS_NOQUERY(1) — just insert, don't query
    let r = ((*ias_v).Insert)(ias as *mut ComObj, ec, 1, text.as_ptr(), text.len() as u32, &mut inserted);

    // Release ITfInsertAtSelection
    let iunknown = ias as *mut ComObj;
    ((*iunknown).lpVtbl.as_ref().unwrap().Release)(iunknown);

    if r == 0 { log::info!("InsertTextAtSelection: {} chars inserted", inserted); }
    else { log::error!("InsertTextAtSelection failed: {}", r); }
    r
}

const ES_VTBL: EditSessionVtbl = EditSessionVtbl {
    base: IUnknownVtbl { QueryInterface: es_qi, AddRef: es_add_ref, Release: es_release },
    DoEditSession: es_do_edit_session,
};

// InsertAtSelection IID
const IID_IAS: GUID = GUID { data1: 0xAA80E7F5, data2: 0x2021, data3: 0x11D2, data4: [0x93, 0xE0, 0x00, 0x60, 0xB0, 0x67, 0xB8, 0x6E] };

// ── Other key event handlers ───────────────────────────────────────────────

unsafe extern "system" fn kes_on_key_up(_: *mut ComObj, _: *mut std::ffi::c_void, _: usize, _: usize, pf: *mut i32) -> i32 { if !pf.is_null() { *pf = 0; } 0 }
unsafe extern "system" fn kes_on_test_key_down(_: *mut ComObj, _: *mut std::ffi::c_void, _: usize, _: usize, pf: *mut i32) -> i32 { if !pf.is_null() { *pf = if ACTIVATED.load(Ordering::Relaxed) { 1 } else { 0 }; } 0 }
unsafe extern "system" fn kes_on_test_key_up(_: *mut ComObj, _: *mut std::ffi::c_void, _: usize, _: usize, pf: *mut i32) -> i32 { if !pf.is_null() { *pf = 0; } 0 }

const KES_VTBL: KeyEventSinkVtbl = KeyEventSinkVtbl {
    base: IUnknownVtbl { QueryInterface: kes_qi, AddRef: kes_add_ref, Release: kes_release },
    OnKeyDown: kes_on_key_down,
    OnKeyUp: kes_on_key_up,
    OnTestKeyDown: kes_on_test_key_down,
    OnTestKeyUp: kes_on_test_key_up,
};
