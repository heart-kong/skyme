use crate::com::SKYME_CLSID_S;

pub struct ClsidRegistrar;
impl ClsidRegistrar { pub fn new() -> Self { Self } }
impl Default for ClsidRegistrar { fn default() -> Self { Self::new() } }

#[cfg(target_os = "windows")]
impl ClsidRegistrar {
    pub fn register(&self) -> i32 {
        log::info!("Registering Skyme TSF text service");
        let c = format!("{{{}}}", SKYME_CLSID_S);
        let b = format!(r"HKLM\SOFTWARE\Classes\CLSID\{}", c);
        let r1 = reg_add(&b, &["/ve", "/t", "REG_SZ", "/d", "Skyme Input Method"]);
        let r2 = reg_add(&format!(r"{}\InprocServer32", b), &["/ve", "/t", "REG_SZ", "/d", "skyme_ime_service.dll"]);
        let r3 = reg_add(&format!(r"{}\InprocServer32", b), &["/v", "ThreadingModel", "/t", "REG_SZ", "/d", "Apartment"]);
        r1.max(r2).max(r3)
    }
    pub fn unregister(&self) -> i32 {
        let c = format!("{{{}}}", SKYME_CLSID_S);
        let b = format!(r"HKLM\SOFTWARE\Classes\CLSID\{}", c);
        let _ = std::process::Command::new("reg").args(&["delete", &b, "/f"]).output();
        0
    }
}

#[cfg(not(target_os = "windows"))]
impl ClsidRegistrar { pub fn register(&self) -> i32 { 0 } pub fn unregister(&self) -> i32 { 0 } }

#[cfg(target_os = "windows")]
fn reg_add(key: &str, args: &[&str]) -> i32 {
    let mut cmd = std::process::Command::new("reg");
    cmd.arg("add").arg(key);
    for a in args { cmd.arg(a); }
    cmd.arg("/f");
    match cmd.output() {
        Ok(out) if out.status.success() => { log::info!("reg add: {} OK", key); 0 }
        Ok(out) => { log::error!("reg add failed: {} {}", key, String::from_utf8_lossy(&out.stderr)); 1 }
        Err(e) => { log::error!("reg add error: {} {}", key, e); 1 }
    }
}
