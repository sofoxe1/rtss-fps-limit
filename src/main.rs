use std::{ffi::c_uint, path::PathBuf};

use windows::{core::PCSTR, Win32::{Foundation::FreeLibrary, System::LibraryLoader::{GetProcAddress, LoadLibraryA}}};

fn main() {

    set_rtss_fps("factorio.exe", 60);
}
fn set_rtss_fps(profile_name:&str,value:u32){
    unsafe {
        let dll_path="C:\\Program Files (x86)\\RivaTuner Statistics Server\\RTSSHooks64.dll\0";
        let hook_dll=LoadLibraryA(PCSTR(dll_path.as_ptr())).unwrap();

        let func_name="UpdateProfiles\0";
        let update_profiles=core::mem::transmute::<unsafe extern "system" fn() -> isize,unsafe extern "C" fn ()>(GetProcAddress(hook_dll, PCSTR(func_name.as_ptr())).unwrap());
        let func_name="LoadProfile\0";
        let load_profile=core::mem::transmute::<unsafe extern "system" fn() -> isize,unsafe extern "C" fn (*const char)>(GetProcAddress(hook_dll, PCSTR(func_name.as_ptr())).unwrap());
        let func_name="SaveProfile\0";
        let save_profile=core::mem::transmute::<unsafe extern "system" fn() -> isize,unsafe extern "C" fn (*const char)>(GetProcAddress(hook_dll, PCSTR(func_name.as_ptr())).unwrap());
        let func_name="SetProfileProperty\0";
        let set_profile_property=core::mem::transmute::<unsafe extern "system" fn() -> isize,unsafe extern "C" fn (*const char,*const c_uint,c_uint)->bool>(GetProcAddress(hook_dll, PCSTR(func_name.as_ptr())).unwrap());

        let profile_path=PathBuf::from("C:\\Program Files (x86)\\RivaTuner Statistics Server\\Profiles\\").join(format!("{}.cfg",profile_name));
        let profile_exists=profile_path.is_file();
        let load_str=match profile_exists{
            true=>format!("{}\0",profile_name),
            //load global profile if game specific doesnt exist
            false=>"\0".to_owned()
        };
        load_profile(load_str.as_ptr().cast());
        let setting_name="FramerateLimit\0";
        //is always 4 unless setting a fontface
        let field_size=4;
        let ret=set_profile_property(setting_name.as_ptr().cast(),&raw const value,field_size);
        if !ret{
            eprintln!("failed to change framerate");
            FreeLibrary(hook_dll).unwrap();
            return;
        }
        save_profile(format!("{}\0",profile_name).as_ptr().cast());
        update_profiles();
        FreeLibrary(hook_dll).unwrap();
    }
}
