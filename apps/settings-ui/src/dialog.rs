use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::Graphics::Gdi::*;
use windows::core::PCWSTR;

fn w16(s: &str) -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() }
fn pw(v: &[u16]) -> PCWSTR { PCWSTR(v.as_ptr()) }

const ID_SV: i32 = 201; const ID_CN: i32 = 202;

// Window style values
const WS_CHILD_V: u32 = 0x40000000;
const WS_VISIBLE_V: u32 = 0x10000000;
const WS_BORDER_V: u32 = 0x00800000;
const WS_TABSTOP_V: u32 = 0x00010000;
const WS_VSCROLL_V: u32 = 0x00200000;
const BS_GROUPBOX_V: u32 = 0x00000007;
const CBS_DROPDOWNLIST_V: u32 = 0x00000003;

#[cfg(target_os = "windows")]
pub fn run_settings_dialog() {
    unsafe {
        let hi = GetModuleHandleA(None).unwrap();
        let cn = w16("SkymeSt");
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW, lpfnWndProc: Some(wndproc),
            hInstance: HINSTANCE(hi.0),
            hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as *mut std::ffi::c_void),
            lpszClassName: pw(&cn), cbClsExtra: 0, cbWndExtra: 0,
            hIcon: HICON::default(), hCursor: HCURSOR::default(), lpszMenuName: PCWSTR::null(),
        };
        RegisterClassW(&wc);
        let t = w16("Skyme Settings");
        if let Ok(h) = CreateWindowExW(
            WINDOW_EX_STYLE(0x00000100), pw(&cn), pw(&t),
            WINDOW_STYLE(0x00C00000 | 0x00080000 | 0x10000000),
            200, 100, 480, 500,
            HWND(std::ptr::null_mut()), HMENU(std::ptr::null_mut()), HINSTANCE(hi.0), None,
        ) {
            ShowWindow(h, SW_SHOW); UpdateWindow(h);
            let mut m = MSG::default();
            while GetMessageW(&mut m, None, 0, 0).into() { TranslateMessage(&m); DispatchMessageW(&m); }
        }
    }
}

unsafe extern "system" fn wndproc(h: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => { ctl(h); LRESULT(0) }
        WM_COMMAND => {
            let id = (wp.0 & 0xFFFF) as i32;
            if id == ID_SV { let _ = MessageBoxW(h, pw(&w16("Saved")), pw(&w16("Skyme")), MB_OK); }
            else if id == ID_CN { DestroyWindow(h); }
            LRESULT(0)
        }
        WM_CLOSE => { DestroyWindow(h); LRESULT(0) }
        WM_DESTROY => { PostQuitMessage(0); LRESULT(0) }
        _ => DefWindowProcW(h, msg, wp, lp),
    }
}

unsafe fn ctl(h: HWND) {
    let hi = HINSTANCE(GetModuleHandleA(None).unwrap().0);
    let mk = |cls: &str, txt: &str, st: u32, x: i32, y: i32, w: i32, ht: i32, id: i32| {
        let (c, t) = (w16(cls), w16(txt));
        let _ = CreateWindowExW(WINDOW_EX_STYLE(0), pw(&c), pw(&t),
            WINDOW_STYLE(WS_CHILD_V | WS_VISIBLE_V | st), x, y, w, ht, h,
            HMENU(id as *mut std::ffi::c_void), hi, None);
    };
    let mut y = 10i32;
    mk("BUTTON", "Display", BS_GROUPBOX_V, 10, y, 440, 80, 0); y += 18;
    mk("STATIC", "Mode:", 0, 20, y+3, 60, 20, 0);
    { let (c, t) = (w16("COMBOBOX"), w16(""));
      let _ = CreateWindowExW(WINDOW_EX_STYLE(0), pw(&c), pw(&t),
          WINDOW_STYLE(WS_CHILD_V | WS_VISIBLE_V | CBS_DROPDOWNLIST_V | WS_VSCROLL_V),
          85, y, 150, 120, h, HMENU(101 as *mut std::ffi::c_void), hi, None); }
    mk("STATIC", "Page Size:", 0, 250, y+3, 70, 20, 0); y += 35;
    mk("BUTTON", "Font", BS_GROUPBOX_V, 10, y, 440, 75, 0); y += 18;
    mk("STATIC", "Family:", 0, 20, y+3, 60, 20, 0);
    mk("EDIT", "", WS_BORDER_V | WS_TABSTOP_V, 85, y, 200, 22, 102);
    mk("STATIC", "Size:", 0, 300, y+3, 40, 20, 0);
    mk("EDIT", "", WS_BORDER_V | WS_TABSTOP_V, 345, y, 50, 22, 103);
    y += 40;
    mk("BUTTON", "Save", WS_TABSTOP_V, 150, y+5, 80, 28, ID_SV);
    mk("BUTTON", "Cancel", WS_TABSTOP_V, 250, y+5, 80, 28, ID_CN);
}
