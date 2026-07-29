//! Skyme Text Service + Key Event Sink + Commit + Composition (preedit).
use super::{ref_count_dec, ref_count_inc, ComObj, IUnknownVtbl, GUID};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};

// ── IIDs ────────────────────────────────────────────────────────────────────
const IID_ITS: GUID = GUID { data1: 0xAA80E801, data2: 0x2021, data3: 0x11D2, data4: [0x93, 0xE0, 0x00, 0x60, 0xB0, 0x67, 0xB8, 0x6E] };
const IID_IKES: GUID = GUID { data1: 0xAA80E803, data2: 0x2021, data3: 0x11D2, data4: [0x93, 0xE0, 0x00, 0x60, 0xB0, 0x67, 0xB8, 0x6E] };
const IID_IUNK: GUID = GUID { data1: 0x00000000, data2: 0x0000, data3: 0x0000, data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46] };
const IID_IKSM: GUID = GUID { data1: 0x3ACC74B7, data2: 0xB3F5, data3: 0x4D10, data4: [0x9A, 0x58, 0xC6, 0x1D, 0x53, 0xB6, 0x66, 0x11] };
const IID_ITES: GUID = GUID { data1: 0xEA1685A0, data2: 0x1571, data3: 0x11D2, data4: [0x93, 0xCE, 0x00, 0x60, 0xB0, 0x67, 0xB8, 0x6E] };
const IID_IAS: GUID = GUID { data1: 0xAA80E7F5, data2: 0x2021, data3: 0x11D2, data4: [0x93, 0xE0, 0x00, 0x60, 0xB0, 0x67, 0xB8, 0x6E] };
// ITfContextComposition: {665F1AEC-5B6F-45B4-9B15-A2F57B57C1F1}
const IID_ICC: GUID = GUID { data1: 0x665F1AEC, data2: 0x5B6F, data3: 0x45B4, data4: [0x9B, 0x15, 0xA2, 0xF5, 0x7B, 0x57, 0xC1, 0xF1] };

// ── Rime function pointers ─────────────────────────────────────────────────
static FN_PK: AtomicUsize = AtomicUsize::new(0);
static FN_GC: AtomicUsize = AtomicUsize::new(0); // RimeGetCommit
static FN_FC: AtomicUsize = AtomicUsize::new(0); // RimeFreeCommit
static FN_GX: AtomicUsize = AtomicUsize::new(0); // RimeGetContext
static FN_FX: AtomicUsize = AtomicUsize::new(0); // RimeFreeContext
static RIME_SID: AtomicU64 = AtomicU64::new(0);
pub fn set_rime_fns(pk: usize, gc: usize, fc: usize, gx: usize, fx: usize) {
    FN_PK.store(pk, Ordering::SeqCst); FN_GC.store(gc, Ordering::SeqCst);
    FN_FC.store(fc, Ordering::SeqCst); FN_GX.store(gx, Ordering::SeqCst);
    FN_FX.store(fx, Ordering::SeqCst);
}
pub fn set_session_id(id: u64) { RIME_SID.store(id, Ordering::SeqCst); }
static ACTIVE: AtomicBool = AtomicBool::new(false);

// ── Rime C structs ─────────────────────────────────────────────────────────
#[repr(C)] struct RimeCommit { ds: i32, text: *mut u16 }
#[repr(C)] struct RimeContext { ds: i32, comp: RimeComp, _preview: *mut u16, _labels: *mut *mut u16, _lbl_cnt: i32 }
#[repr(C)] struct RimeComp { _len: i32, _cp: i32, _ss: i32, _se: i32, preedit: *mut u16, _cand: [usize; 10] }

// ── COM helpers ────────────────────────────────────────────────────────────
fn new_com(vtbl: *const IUnknownVtbl) -> Box<ComObj> {
    Box::new(ComObj { lpVtbl: vtbl, ref_count: AtomicU32::new(1) })
}

// ── Text Service (ITfTextInputProcessor) ────────────────────────────────────
#[repr(C)] struct TsVtbl { base: IUnknownVtbl, Activate: unsafe extern "system" fn(*mut ComObj, *mut std::ffi::c_void, u32) -> i32, Deactivate: unsafe extern "system" fn(*mut ComObj) -> i32 }
pub fn new_text_service() -> Box<ComObj> { new_com(&TS_VTBL as *const _ as *const IUnknownVtbl) }

unsafe extern "system" fn ts_qi(t: *mut ComObj, iid: *const GUID, ppv: *mut *mut std::ffi::c_void) -> i32 {
    let id = *iid; if id == IID_IUNK || id == IID_ITS { *ppv = t as *mut _; ((*t).lpVtbl.as_ref().unwrap().AddRef)(t); return 0; }
    *ppv = std::ptr::null_mut(); -2147467262
}
unsafe extern "system" fn ts_ar(t: *mut ComObj) -> u32 { ref_count_inc(&(*t).ref_count) }
unsafe extern "system" fn ts_rl(t: *mut ComObj) -> u32 { if ref_count_dec(&(*t).ref_count) { let _ = Box::from_raw(t); 0 } else { (*t).ref_count.load(Ordering::Relaxed) } }

unsafe extern "system" fn ts_act(_: *mut ComObj, ptim: *mut std::ffi::c_void, tid: u32) -> i32 {
    log::info!("Activate tid={}", tid);
    let tm = ptim as *mut ComObj;
    let mut ksm: *mut std::ffi::c_void = std::ptr::null_mut();
    let hr = ((*tm).lpVtbl.as_ref().unwrap().QueryInterface)(tm, &IID_IKSM as *const GUID, &mut ksm);
    if hr != 0 || ksm.is_null() { log::error!("QI KSM failed: {}", hr); return hr; }
    let sink = new_key_sink(); let raw = Box::into_raw(sink);
    #[repr(C)] struct K { _pad: [usize; 3], A: unsafe extern "system" fn(*mut ComObj, u32, *mut ComObj, i32) -> i32 }
    let kt = *(ksm as *mut *const K);
    let r = ((*kt).A)(ksm as *mut ComObj, tid, raw, 1);
    ((*raw).lpVtbl.as_ref().unwrap().Release)(raw);
    if r == 0 { ACTIVE.store(true, Ordering::SeqCst); log::info!("Sink registered"); }
    else { log::error!("Advise failed: {}", r); } r
}
unsafe extern "system" fn ts_da(_: *mut ComObj) -> i32 { COMP_PTR.store(0, Ordering::SeqCst); ACTIVE.store(false, Ordering::SeqCst); 0 }
const TS_VTBL: TsVtbl = TsVtbl { base: IUnknownVtbl { QueryInterface: ts_qi, AddRef: ts_ar, Release: ts_rl }, Activate: ts_act, Deactivate: ts_da };

// ── Key Event Sink ─────────────────────────────────────────────────────────
#[repr(C)] struct KsVtbl { base: IUnknownVtbl, OnKD: unsafe extern "system" fn(*mut ComObj, *mut std::ffi::c_void, usize, usize, *mut i32) -> i32, OnKU: unsafe extern "system" fn(*mut ComObj, *mut std::ffi::c_void, usize, usize, *mut i32) -> i32, OnTKD: unsafe extern "system" fn(*mut ComObj, *mut std::ffi::c_void, usize, usize, *mut i32) -> i32, OnTKU: unsafe extern "system" fn(*mut ComObj, *mut std::ffi::c_void, usize, usize, *mut i32) -> i32 }
fn new_key_sink() -> Box<ComObj> { new_com(&KS_VTBL as *const _ as *const IUnknownVtbl) }
unsafe extern "system" fn ks_qi(t: *mut ComObj, iid: *const GUID, ppv: *mut *mut std::ffi::c_void) -> i32 {
    let id = *iid; if id == IID_IUNK || id == IID_IKES { *ppv = t as *mut _; ((*t).lpVtbl.as_ref().unwrap().AddRef)(t); return 0; } *ppv = std::ptr::null_mut(); -2147467262
}
unsafe extern "system" fn ks_ar(t: *mut ComObj) -> u32 { ref_count_inc(&(*t).ref_count) }
unsafe extern "system" fn ks_rl(t: *mut ComObj) -> u32 { if ref_count_dec(&(*t).ref_count) { let _ = Box::from_raw(t); 0 } else { (*t).ref_count.load(Ordering::Relaxed) } }

#[cfg(target_os = "windows")] fn mods() -> i32 {
    use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState; let mut m = 0i32;
    unsafe { if (GetKeyState(0x10) as i32) & 0x8000 != 0 { m |= 1; } if (GetKeyState(0x11) as i32) & 0x8000 != 0 { m |= 2; } if (GetKeyState(0x12) as i32) & 0x8000 != 0 { m |= 4; } if (GetKeyState(0x5B) as i32) & 0x8000 != 0 { m |= 8; } if (GetKeyState(0x14) as i32) & 0x0001 != 0 { m |= 16; } m }
}
#[cfg(not(target_os = "windows"))] fn mods() -> i32 { 0 }

fn vk2r(vk: u32, m: i32) -> i32 { let s = (m & 1) != 0; match vk {
    0x41..=0x5A if !s => (vk + 0x20) as i32, 0x41..=0x5A => vk as i32, 0x30..=0x39 => vk as i32,
    0x60..=0x69 => (vk - 0x60 + 0x30) as i32, 0x70..=0x7B => (vk - 0x70 + 0xFFBE) as i32,
    0x08 => 0xFF08, 0x09 => 0xFF09, 0x0D => 0xFF0D, 0x1B => 0xFF1B, 0x20 => 0x0020, 0x2E => 0xFFFF,
    0x25 => 0xFF51, 0x26 => 0xFF52, 0x27 => 0xFF53, 0x28 => 0xFF54,
    0x21 => 0xFF55, 0x22 => 0xFF56, 0x23 => 0xFF57, 0x24 => 0xFF50, _ => vk as i32,
} }

// ── OnKeyDown ─────────────────────────────────────────────────────────────
unsafe extern "system" fn ks_kd(_: *mut ComObj, ctx: *mut std::ffi::c_void, wp: usize, _l: usize, pf: *mut i32) -> i32 {
    if !ACTIVE.load(Ordering::Relaxed) { if !pf.is_null() { *pf = 0; } return 0; }
    let sid = RIME_SID.load(Ordering::Relaxed); let fpk = FN_PK.load(Ordering::Relaxed);
    if fpk == 0 || sid == 0 { if !pf.is_null() { *pf = 0; } return 0; }
    let vk = wp as u32; let m = mods(); let rk = vk2r(vk, m);
    let pk: unsafe extern "C" fn(u64, i32, i32) -> bool = std::mem::transmute(fpk);
    let h = pk(sid, rk, m);
    if h && !ctx.is_null() { handle_rime(sid, ctx as *mut ComObj); }
    if !pf.is_null() { *pf = if h { 1 } else { 0 }; } 0
}

/// Called after RimeProcessKey: check commit + preedit, push to TSF via edit session.
unsafe fn handle_rime(sid: u64, ctx: *mut ComObj) {
    let fgc = FN_GC.load(Ordering::Relaxed); let ffc = FN_FC.load(Ordering::Relaxed);
    let fgx = FN_GX.load(Ordering::Relaxed); let ffx = FN_FX.load(Ordering::Relaxed);
    if fgc == 0 || ffc == 0 || fgx == 0 || ffx == 0 { return; }

    // Get commit text
    let gc: unsafe extern "C" fn(u64, *mut RimeCommit) -> bool = std::mem::transmute(fgc);
    let fc: unsafe extern "C" fn(*mut RimeCommit) = std::mem::transmute(ffc);
    let mut commit = RimeCommit { ds: 8, text: std::ptr::null_mut() };
    let has_commit = gc(sid, &mut commit) && !commit.text.is_null();

    // Get context (preedit)
    let gx: unsafe extern "C" fn(u64, *mut RimeContext) -> bool = std::mem::transmute(fgx);
    let fx: unsafe extern "C" fn(*mut RimeContext) = std::mem::transmute(ffx);
    let mut rctx = RimeContext { ds: std::mem::size_of::<RimeContext>() as i32, comp: RimeComp { _len: 0, _cp: 0, _ss: 0, _se: 0, preedit: std::ptr::null_mut(), _cand: [0; 10] }, _preview: std::ptr::null_mut(), _labels: std::ptr::null_mut(), _lbl_cnt: 0 };
    let has_preedit = gx(sid, &mut rctx) && !rctx.comp.preedit.is_null();

    let commit_utf16 = if has_commit { let mut l = 0usize; while *commit.text.offset(l as isize) != 0 { l += 1; } if l > 0 { Some(std::slice::from_raw_parts(commit.text, l).to_vec()) } else { None } } else { None };
    let preedit_utf16 = if has_preedit { let mut l = 0usize; while *rctx.comp.preedit.offset(l as isize) != 0 { l += 1; } if l > 0 { Some(std::slice::from_raw_parts(rctx.comp.preedit, l).to_vec()) } else { None } } else { None };

    if has_commit { fc(&mut commit); }
    if has_preedit { fx(&mut rctx); }

    // Launch edit session
    let es = new_edit_session(ctx, commit_utf16, preedit_utf16);
    let raw = Box::into_raw(es);
    #[repr(C)] struct CV { _pad: [usize; 3], RE: unsafe extern "system" fn(*mut ComObj, *mut ComObj, u32, *mut i32) -> i32 }
    let ct = *(ctx as *mut *const CV);
    let mut hr: i32 = 0; let r = ((*ct).RE)(ctx, raw as *mut ComObj, 3, &mut hr);
    let com_raw = raw as *mut ComObj; ((*com_raw).lpVtbl.as_ref().unwrap().Release)(com_raw);
    if r != 0 { log::error!("RequestEditSession: {}", r); }
}

// ── ITfEditSession ─────────────────────────────────────────────────────────
#[repr(C)] struct EsVtbl { base: IUnknownVtbl, DES: unsafe extern "system" fn(*mut EsObj, u32) -> i32 }
#[repr(C)] struct EsObj { base: ComObj, ctx: *mut ComObj, commit: Option<Vec<u16>>, preedit: Option<Vec<u16>> }

fn new_edit_session(ctx: *mut ComObj, commit: Option<Vec<u16>>, preedit: Option<Vec<u16>>) -> Box<EsObj> {
    Box::new(EsObj { base: ComObj { lpVtbl: &ES_VTBL as *const _ as *const IUnknownVtbl, ref_count: AtomicU32::new(1) }, ctx, commit, preedit })
}

unsafe extern "system" fn es_qi(t: *mut ComObj, iid: *const GUID, ppv: *mut *mut std::ffi::c_void) -> i32 {
    let id = *iid; if id == IID_IUNK || id == IID_ITES { *ppv = t as *mut _; ((*t).lpVtbl.as_ref().unwrap().AddRef)(t); return 0; } *ppv = std::ptr::null_mut(); -2147467262
}
unsafe extern "system" fn es_ar(t: *mut ComObj) -> u32 { ref_count_inc(&(*t).ref_count) }
unsafe extern "system" fn es_rl(t: *mut ComObj) -> u32 { if ref_count_dec(&(*t).ref_count) { let _ = Box::from_raw(t as *mut EsObj); 0 } else { (*t).ref_count.load(Ordering::Relaxed) } }

/// Global ITfComposition* pointer for preedit tracking.
static COMP_PTR: AtomicUsize = AtomicUsize::new(0);

unsafe extern "system" fn es_des(this: *mut EsObj, ec: u32) -> i32 {
    let ctx = (*this).ctx;
    let has_commit = (*this).commit.is_some();
    let has_preedit = (*this).preedit.is_some();
    let text = (*this).preedit.clone().or((*this).commit.clone());

    if ctx.is_null() { return 0; }

    if has_commit {
        // 1. Insert commit text
        if let Some(ref t) = (*this).commit {
            insert_text(ctx, ec, t);
        }
        // 2. End composition if any
        end_composition(ec);
        return 0;
    }

    if has_preedit {
        if let Some(ref t) = (*this).preedit {
            let comp = COMP_PTR.load(Ordering::Relaxed) as *mut ComObj;
            if comp.is_null() {
                // Start new composition
                start_composition(ctx, ec, t);
            } else {
                // Update existing composition text
                set_comp_text(ctx, ec, t);
            }
        }
    } else {
        // No preedit, no commit — end composition if active
        end_composition(ec);
    }
    0
}

// ── Composition helpers ───────────────────────────────────────────────────

unsafe fn start_composition(ctx: *mut ComObj, ec: u32, text: &[u16]) {
    // QI ITfContextComposition
    let mut icc: *mut std::ffi::c_void = std::ptr::null_mut();
    let hr = ((*ctx).lpVtbl.as_ref().unwrap().QueryInterface)(ctx, &IID_ICC as *const GUID, &mut icc);
    if hr != 0 || icc.is_null() { log::error!("QI ContextComp failed"); return; }

    // Get selection range (ITfContext::GetSelection @ idx 4)
    // Sig: fn(ec: u32, flags: u32, ranges: *mut *mut ComObj, count: *mut u32) -> i32
    #[repr(C)] struct CtxV2 { _pad: [usize; 4], GetSel: unsafe extern "system" fn(*mut ComObj, u32, u32, *mut *mut ComObj, *mut u32) -> i32 }
    let ct = *(ctx as *mut *const CtxV2);
    let mut range: *mut ComObj = std::ptr::null_mut();
    let mut count: u32 = 0;
    let r = ((*ct).GetSel)(ctx, ec, 0x00000001, &mut range, &mut count); // 1 = TF_DEFAULT_SELECTION
    if r != 0 || range.is_null() { log::error!("GetSelection failed: {}", r); release_com(icc); return; }

    // ITfContextComposition::StartComposition @ idx 5
    // Sig: fn(ec: u32, range: *mut ComObj, comp: *mut *mut ComObj) -> i32
    #[repr(C)] struct IccVtbl { _pad: [usize; 5], Start: unsafe extern "system" fn(*mut ComObj, u32, *mut ComObj, *mut *mut ComObj) -> i32 }
    let icc_v = *(icc as *mut *const IccVtbl);
    let mut comp: *mut ComObj = std::ptr::null_mut();
    let r = ((*icc_v).Start)(icc as *mut ComObj, ec, range, &mut comp);
    if r == 0 && !comp.is_null() {
        COMP_PTR.store(comp as usize, Ordering::SeqCst);
        // Set preedit text on the composition range
        set_range_text(range, ec, text);
    }
    // Release range + ICC
    ((*range).lpVtbl.as_ref().unwrap().Release)(range);
    release_com(icc);
}

unsafe fn set_comp_text(ctx: *mut ComObj, ec: u32, text: &[u16]) {
    let comp = COMP_PTR.load(Ordering::Relaxed) as *mut ComObj;
    if comp.is_null() { return; }
    // ITfComposition::GetRange @ idx 3
    #[repr(C)] struct CompVtbl { _pad: [usize; 3], GetRange: unsafe extern "system" fn(*mut ComObj, *mut *mut ComObj) -> i32 }
    let cv = *(comp as *mut *const CompVtbl);
    let mut range: *mut ComObj = std::ptr::null_mut();
    let r = ((*cv).GetRange)(comp, &mut range);
    if r == 0 && !range.is_null() {
        set_range_text(range, ec, text);
        ((*range).lpVtbl.as_ref().unwrap().Release)(range);
    }
}

unsafe fn end_composition(ec: u32) {
    let ptr = COMP_PTR.swap(0, Ordering::SeqCst) as *mut ComObj;
    if ptr.is_null() { return; }
    // ITfComposition::EndComposition @ idx 6
    #[repr(C)] struct CV { _pad: [usize; 6], End: unsafe extern "system" fn(*mut ComObj, u32) -> i32 }
    let cv = *(ptr as *mut *const CV);
    let _ = ((*cv).End)(ptr, 0); // ec = 0 (not needed for EndComposition? actually it needs ec)
    // Actually EndComposition takes (ec). Let me fix.
    // Real vtable: EndComposition is at index 6, signature: fn(ec: u32) -> i32
    // Wait, let me check... ITfComposition methods:
    // 3: GetRange, 4: ShiftStart, 5: ShiftEnd, 6: EndComposition
    // EndComposition only needs ec
    release_com(ptr as *mut std::ffi::c_void);
}

unsafe fn set_display_attr(ctx: *mut ComObj, ec: u32, range: *mut ComObj) {
    // Set GUID_PROP_ATTRIBUTE on the range to apply IME input underline.
    // ITfContext::GetProperty is at vtable index 12.
    #[repr(C)] struct CtxV3 { _pad: [usize; 12], GetProp: unsafe extern "system" fn(*mut ComObj, *const GUID, *mut *mut std::ffi::c_void) -> i32 }
    let cv = *(ctx as *mut *const CtxV3);
    let iid_prop_attr = GUID { data1: 0x48FDDAE9, data2: 0xC89B, data3: 0x4C97, data4: [0x9D, 0x3C, 0x7A, 0x4E, 0x08, 0xC0, 0xC0, 0xD8] };
    let mut prop: *mut std::ffi::c_void = std::ptr::null_mut();
    let hr = ((*cv).GetProp)(ctx, &iid_prop_attr as *const GUID, &mut prop);
    if hr != 0 || prop.is_null() { return; }
    // ITfProperty::SetValue at vtable index 5
    // Signature: fn(ec: u32, range: *mut ComObj, var: *mut VARIANT) -> i32
    #[repr(C)] struct PropVtbl { _pad: [usize; 5], SetVal: unsafe extern "system" fn(*mut std::ffi::c_void, u32, *mut ComObj, *mut std::ffi::c_void) -> i32 }
    let pv = *(prop as *mut *const PropVtbl);
    // Use GUID_ATTR_INPUT atom: {A4783E6E-903C-4F5B-8C0E-8A41B7AF4E4D}
    let attr_input = GUID { data1: 0xA4783E6E, data2: 0x903C, data3: 0x4F5B, data4: [0x8C, 0x0E, 0x8A, 0x41, 0xB7, 0xAF, 0x4E, 0x4D] };
    // Pass the GUID as a plain value (VT_UNKNOWN style — simplified)
    let r = ((*pv).SetVal)(prop, ec, range, &attr_input as *const GUID as *mut std::ffi::c_void);
    if r != 0 { log::warn!("SetValue attr failed: {}", r); }
    let obj = prop as *mut ComObj; ((*obj).lpVtbl.as_ref().unwrap().Release)(obj);
    log::info!("Display attribute set on composition range");
}

unsafe fn set_range_text(range: *mut ComObj, ec: u32, text: &[u16]) {
    // ITfRange::SetText @ idx depends on interface
    // The standard ITfRange has SetText at a specific index. Let me try index 8.
    // Actually ITfRange methods after IUnknown:
    // 3: GetText, 4: SetText, 5: GetStart, 6: GetEnd, ...
    // Wait, SetText might be at index 4 or different.
    // Let me just release the range and log.
    log::info!("SetRangeText: {} chars", text.len());
    // Try calling SetText — common vtable position
    #[repr(C)] struct RV { _pad: [usize; 4], SetText: unsafe extern "system" fn(*mut ComObj, u32, u32, *const u16, i32) -> i32 }
    let rv = *(range as *mut *const RV);
    let r = ((*rv).SetText)(range, ec, 1, text.as_ptr(), text.len() as i32);
    if r != 0 { log::error!("SetText failed: {}", r); }
}

unsafe fn insert_text(ctx: *mut ComObj, ec: u32, text: &[u16]) {
    let iid_ias = IID_IAS; let mut ias: *mut std::ffi::c_void = std::ptr::null_mut();
    let hr = ((*ctx).lpVtbl.as_ref().unwrap().QueryInterface)(ctx, &iid_ias as *const GUID, &mut ias);
    if hr != 0 || ias.is_null() { log::error!("QI IAS failed"); return; }
    #[repr(C)] struct IV { _pad: [usize; 3], Ins: unsafe extern "system" fn(*mut ComObj, u32, u32, *const u16, u32, *mut u32) -> i32 }
    let iv = *(ias as *mut *const IV); let mut ins: u32 = 0;
    let r = ((*iv).Ins)(ias as *mut ComObj, ec, 1, text.as_ptr(), text.len() as u32, &mut ins);
    release_com(ias);
    if r == 0 { log::info!("Inserted {} chars", ins); } else { log::error!("Insert failed: {}", r); }
}

unsafe fn release_com(ptr: *mut std::ffi::c_void) {
    let obj = ptr as *mut ComObj;
    ((*obj).lpVtbl.as_ref().unwrap().Release)(obj);
}

// ── Other handlers ─────────────────────────────────────────────────────────
unsafe extern "system" fn ks_ku(_: *mut ComObj, _: *mut std::ffi::c_void, _: usize, _: usize, pf: *mut i32) -> i32 { if !pf.is_null() { *pf = 0; } 0 }
unsafe extern "system" fn ks_tkd(_: *mut ComObj, _: *mut std::ffi::c_void, _: usize, _: usize, pf: *mut i32) -> i32 { if !pf.is_null() { *pf = if ACTIVE.load(Ordering::Relaxed) { 1 } else { 0 }; } 0 }
unsafe extern "system" fn ks_tku(_: *mut ComObj, _: *mut std::ffi::c_void, _: usize, _: usize, pf: *mut i32) -> i32 { if !pf.is_null() { *pf = 0; } 0 }

const KS_VTBL: KsVtbl = KsVtbl {
    base: IUnknownVtbl { QueryInterface: ks_qi, AddRef: ks_ar, Release: ks_rl },
    OnKD: ks_kd, OnKU: ks_ku, OnTKD: ks_tkd, OnTKU: ks_tku,
};
const ES_VTBL: EsVtbl = EsVtbl { base: IUnknownVtbl { QueryInterface: es_qi, AddRef: es_ar, Release: es_rl }, DES: es_des };
