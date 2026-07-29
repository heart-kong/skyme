use super::{ref_count_dec, ref_count_inc, ComObj, GUID, SKYME_CLSID_S};
use std::sync::atomic::AtomicU32;

const IID_ICF: GUID = GUID { data1: 0x00000001, data2: 0x0000, data3: 0x0000, data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46] };
const IID_IUNK: GUID = GUID { data1: 0x00000000, data2: 0x0000, data3: 0x0000, data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46] };

#[repr(C)]
pub struct ClassFactoryVtbl {
    pub base: super::IUnknownVtbl,
    pub CreateInstance: unsafe extern "system" fn(*mut ComObj, *mut ComObj, *const GUID, *mut *mut std::ffi::c_void) -> i32,
    pub LockServer: unsafe extern "system" fn(*mut ComObj, bool) -> i32,
}

unsafe extern "system" fn qi(this: *mut ComObj, iid: *const GUID, ppv: *mut *mut std::ffi::c_void) -> i32 {
    let id = *iid;
    if id == IID_IUNK || id == IID_ICF {
        *ppv = this as *mut _;
        ((*this).lpVtbl.as_ref().unwrap().AddRef)(this);
        return 0;
    }
    *ppv = std::ptr::null_mut(); -2147024894
}
unsafe extern "system" fn add_ref(this: *mut ComObj) -> u32 { ref_count_inc(&(*this).ref_count) }
unsafe extern "system" fn release(this: *mut ComObj) -> u32 {
    if ref_count_dec(&(*this).ref_count) { let _ = Box::from_raw(this); 0 }
    else { (*this).ref_count.load(std::sync::atomic::Ordering::Relaxed) }
}
unsafe extern "system" fn create_instance(_this: *mut ComObj, outer: *mut ComObj, riid: *const GUID, ppv: *mut *mut std::ffi::c_void) -> i32 {
    if !outer.is_null() { return -2146969328; }
    let obj = super::text_service::new_service();
    let raw = Box::into_raw(obj);
    let result = qi(raw, riid, ppv);
    release(raw);
    result
}
unsafe extern "system" fn lock_server(_this: *mut ComObj, _lock: bool) -> i32 { 0 }

const VTBL: ClassFactoryVtbl = ClassFactoryVtbl {
    base: super::IUnknownVtbl { QueryInterface: qi, AddRef: add_ref, Release: release },
    CreateInstance: create_instance, LockServer: lock_server,
};

pub fn new_factory() -> Box<ComObj> {
    Box::new(ComObj { lpVtbl: &VTBL as *const _ as *const super::IUnknownVtbl, ref_count: AtomicU32::new(1) })
}

pub fn class_factory(clsid: *const GUID, iid: *const GUID, ppv: *mut *mut std::ffi::c_void) -> i32 {
    let clsid = unsafe { *clsid };
    let iid = unsafe { *iid };
    let our = guid_from_str(SKYME_CLSID_S);
    if clsid != our { unsafe { *ppv = std::ptr::null_mut(); } return -2146967212; }
    let factory = new_factory();
    let raw = Box::into_raw(factory);
    let result = unsafe { qi(raw, &iid as *const GUID, ppv) };
    unsafe { release(raw); }
    result
}

fn guid_from_str(s: &str) -> GUID {
    let hex: String = s.chars().filter(|c| *c != '-' && *c != '{' && *c != '}').collect();
    let b = (0..hex.len()).step_by(2).filter_map(|i| u8::from_str_radix(&hex[i..(i+2).min(hex.len())], 16).ok()).collect::<Vec<_>>();
    if b.len() < 16 { return GUID::default(); }
    GUID { data1: u32::from_be_bytes([b[0], b[1], b[2], b[3]]), data2: u16::from_be_bytes([b[4], b[5]]), data3: u16::from_be_bytes([b[6], b[7]]), data4: [b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]] }
}
