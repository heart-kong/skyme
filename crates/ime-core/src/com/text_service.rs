//! Skyme Text Service COM object.
//! Minimal ITfTextInputProcessor implementation.

use super::{ref_count_dec, ref_count_inc, ComObj, IUnknownVtbl, GUID};
use std::sync::atomic::AtomicU32;

// ITfTextInputProcessor IID: AA80E801-2021-11D2-93E0-0060B067B86E
const IID_ITS: GUID = GUID { data1: 0xAA80E801, data2: 0x2021, data3: 0x11D2, data4: [0x93, 0xE0, 0x00, 0x60, 0xB0, 0x67, 0xB8, 0x6E] };
const IID_IUNK: GUID = GUID { data1: 0x00000000, data2: 0x0000, data3: 0x0000, data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46] };

#[repr(C)]
pub struct TextServiceVtbl {
    pub base: IUnknownVtbl,
    pub Activate: unsafe extern "system" fn(*mut ComObj, *mut std::ffi::c_void, u32) -> i32,
    pub Deactivate: unsafe extern "system" fn(*mut ComObj) -> i32,
}

pub fn new_service() -> Box<ComObj> {
    Box::new(ComObj { lpVtbl: &VTBL as *const _ as *const super::IUnknownVtbl, ref_count: AtomicU32::new(1) })
}

unsafe extern "system" fn qi(this: *mut ComObj, iid: *const GUID, ppv: *mut *mut std::ffi::c_void) -> i32 {
    let id = *iid;
    if id == IID_IUNK || id == IID_ITS {
        *ppv = this as *mut _;
        ((*this).lpVtbl.as_ref().unwrap().AddRef)(this);
        return 0;
    }
    *ppv = std::ptr::null_mut();
    -2147024894
}
unsafe extern "system" fn add_ref(this: *mut ComObj) -> u32 { ref_count_inc(&(*this).ref_count) }
unsafe extern "system" fn release(this: *mut ComObj) -> u32 {
    if ref_count_dec(&(*this).ref_count) { let _ = Box::from_raw(this); 0 }
    else { (*this).ref_count.load(std::sync::atomic::Ordering::Relaxed) }
}
unsafe extern "system" fn activate(_this: *mut ComObj, _ptim: *mut std::ffi::c_void, tid: u32) -> i32 {
    log::info!("Skyme TSF: Activate (tid={})", tid); 0
}
unsafe extern "system" fn deactivate(_this: *mut ComObj) -> i32 {
    log::info!("Skyme TSF: Deactivate"); 0
}

const VTBL: TextServiceVtbl = TextServiceVtbl {
    base: IUnknownVtbl { QueryInterface: qi, AddRef: add_ref, Release: release },
    Activate: activate, Deactivate: deactivate,
};
