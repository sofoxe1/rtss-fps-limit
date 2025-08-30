#![allow(clippy::missing_transmute_annotations)]
#![allow(dead_code)]
use std::{
    ffi::{CStr, CString, c_uint},
    fmt::Debug,
    fs,
    os::windows::process::CommandExt,
    path::PathBuf,
    process::Command,
    sync::LazyLock,
    thread,
    time::Duration,
};

use windows::{
    Win32::{
        Foundation::{FreeLibrary, HMODULE},
        System::LibraryLoader::{GetProcAddress, LoadLibraryA},
    },
    core::PCSTR,
};
const DLL_PATH: &CStr = c"C:\\Program Files (x86)\\RivaTuner Statistics Server\\RTSSHooks64.dll";
unsafe impl Send for Rtss {}
unsafe impl Sync for Rtss {}
pub struct Rtss {
    pub lib_handle: HMODULE,
    pub update_profiles: unsafe extern "C" fn(),
    pub load_profile: unsafe extern "C" fn(*const char),
    pub save_profile: unsafe extern "C" fn(*const char),
    //fn(profile_name,in/out value,data size in bytes)->success,
    pub set_profile_property: unsafe extern "C" fn(*const char, *const c_uint, c_uint) -> bool,
    pub get_profile_property: unsafe extern "C" fn(*const char, *mut c_uint, c_uint) -> bool,
}
impl Drop for Rtss {
    fn drop(&mut self) {
        unsafe {
            let _ = FreeLibrary(self.lib_handle);
        }
    }
}
pub enum RtssError {
    NotInstalled,
    UnableToLoadLibrary(windows::core::Error),
    FailedToLoadFunction,
    LibraryNotPresent,
    FailedToUpdateProfile,
    FailedToGetValue,
    FailedToSetValue,
    ProfileNotFound,
}
impl Debug for RtssError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInstalled => write!(f, "NotInstalled"),
            Self::UnableToLoadLibrary(arg0) => {
                f.debug_tuple("UnableToLoadLibrary").field(arg0).finish()
            }
            Self::FailedToLoadFunction => write!(f, "FailedToLoadFunction"),
            Self::LibraryNotPresent => write!(f, "LibraryNotPresent"),
            Self::FailedToUpdateProfile => {
                write!(f, "check if you have write permission for profiles folder")
            }
            Self::FailedToGetValue => write!(f, "FailedToGetValue"),
            Self::FailedToSetValue => write!(f, "FailedToSetValue"),
            Self::ProfileNotFound => write!(f, "ProfileNotFound"),
        }
    }
}
impl Rtss {
    fn new() -> Result<Self, RtssError> {
        let mut path = PathBuf::from(DLL_PATH.to_str().unwrap());
        if !path.is_file() {
            return Err(RtssError::LibraryNotPresent);
        }
        path.pop();
        if !path.is_dir() {
            return Err(RtssError::NotInstalled);
        }
        unsafe {
            let lib_handle = LoadLibraryA(PCSTR(DLL_PATH.as_ptr().cast()))
                .map_err(RtssError::UnableToLoadLibrary)?;
            Ok(Self {
                update_profiles: core::mem::transmute(
                    GetProcAddress(lib_handle, PCSTR(c"UpdateProfiles".as_ptr().cast()))
                        .ok_or(RtssError::FailedToLoadFunction)?,
                ),
                load_profile: core::mem::transmute(
                    GetProcAddress(lib_handle, PCSTR(c"LoadProfile".as_ptr().cast()))
                        .ok_or(RtssError::FailedToLoadFunction)?,
                ),
                save_profile: core::mem::transmute(
                    GetProcAddress(lib_handle, PCSTR(c"SaveProfile".as_ptr().cast()))
                        .ok_or(RtssError::FailedToLoadFunction)?,
                ),
                set_profile_property: core::mem::transmute(
                    GetProcAddress(lib_handle, PCSTR(c"SetProfileProperty".as_ptr().cast()))
                        .ok_or(RtssError::FailedToLoadFunction)?,
                ),
                get_profile_property: core::mem::transmute(
                    GetProcAddress(lib_handle, PCSTR(c"GetProfileProperty".as_ptr().cast()))
                        .ok_or(RtssError::FailedToLoadFunction)?,
                ),
                lib_handle,
            })
        }
    }
}
static RTSS: LazyLock<Rtss> = LazyLock::new(|| Rtss::new().unwrap());
pub fn update_profiles() {
    unsafe { (RTSS.update_profiles)() }
}
pub fn load_profile(name: &CString) {
    unsafe { (RTSS.load_profile)(name.as_ptr().cast()) }
}
pub fn save_profile(name: &CString) {
    unsafe { (RTSS.save_profile)(name.as_ptr().cast()) }
}
pub fn set_profile_property(field: &CString, value: u32) -> Option<()> {
    unsafe {
        if !(RTSS.set_profile_property)(field.as_ptr().cast(), &raw const value, 4) {
            return None;
        }
        Some(())
    }
}
pub fn get_profile_property(field: &CString) -> Option<u32> {
    unsafe {
        let mut out = 0;
        if !(RTSS.get_profile_property)(field.as_ptr().cast(), &raw mut out, 4) {
            return None;
        }
        Some(out)
    }
}
pub fn profile_exists(name: &str) -> bool {
    PathBuf::from("C:\\Program Files (x86)\\RivaTuner Statistics Server\\Profiles\\")
        .join(format!("{}.cfg", name))
        .exists()
}
static SCRIPT: &str = include_str!("get_permissions.ps1");
///will show uac prompt
pub fn get_write_acess() {
    let path = std::env::temp_dir().join("rtss_rs_get_permissions.ps1");
    fs::write(
        &path,
        SCRIPT.replacen("$username$", &std::env::var("username").unwrap(), 1),
    )
    .unwrap();
    Command::new("powershell")
        .raw_arg(format!(
            "start-process powershell -verb runAs {}",
            path.to_str().unwrap()
        ))
        .output()
        .unwrap();
    thread::sleep(Duration::from_millis(300)); //wait for elevated powershell
    fs::remove_file(path).ok();
}
