#![allow(clippy::missing_transmute_annotations)]
#![allow(dead_code)]
#![allow(static_mut_refs)]
use std::{
    ffi::{CStr, CString},
    fmt::Debug,
    io,
    num::ParseIntError,
    ops::Not,
    path::PathBuf,
    str::FromStr,
};

use ini::Ini;
use rtss_sys::{
    APPFLAG_PROFILE_UPDATE_REQUESTED, LPRTSS_SHARED_MEMORY,
    RTSS_SHARED_MEMORY_LPRTSS_SHARED_MEMORY_APP_ENTRY,
};
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        System::Memory::{
            MapViewOfFile, OpenFileMappingA, UnmapViewOfFile, FILE_MAP_ALL_ACCESS,
            MEMORY_MAPPED_VIEW_ADDRESS,
        },
    },
};

use crate::get_write_permission;
pub const INSTALL_PATH: &CStr = c"C:\\Program Files (x86)\\RivaTuner Statistics Server";
#[derive(Debug)]
pub enum RtssError {
    NotInstalled,
    FailedToUpdateProfile,
    FailedToGetValue,
    FailedToSetValue,
    ProfileNotFound,
    FailedToMapSharedMemory,
    IO(std::io::Error),
    Ini(ini::Error),
    ParseError(ParseIntError),
    Windows(windows::core::Error),
}
pub fn load_profile(name: &CString) -> Result<Ini, RtssError> {
    let mut step = PathBuf::from_str(INSTALL_PATH.to_str().unwrap()).unwrap();
    step = step.join("Profiles");

    if name.is_empty().not() {
        if step.is_dir().not() {
            return Err(RtssError::IO(std::io::Error::new(
                io::ErrorKind::NotADirectory,
                io::Error::other(""),
            )));
        }
        step = step.join(format!("{}.cfg", name.to_str().unwrap()));
    } else {
        step = step.join("Global");
    }
    if step.exists().not() {
        return Err(RtssError::ProfileNotFound);
    }
    let ini = ini::Ini::load_from_file(step).map_err(RtssError::Ini)?;
    Ok(ini)
}
pub fn save_profile(name: &CString, profile: &Ini) -> Result<(), RtssError> {
    let mut step = PathBuf::from_str(INSTALL_PATH.to_str().unwrap()).unwrap();
    step = step.join("Profiles");

    if step.is_dir().not() {
        return Err(RtssError::IO(std::io::Error::new(
            io::ErrorKind::NotADirectory,
            io::Error::other(""),
        )));
    }
    step = step.join(format!("{}.cfg", name.to_str().unwrap()));
    if let Err(RtssError::IO(err)) = profile.write_to_file(step).map_err(RtssError::IO) {
        if let io::ErrorKind::PermissionDenied = err.kind() {
            get_write_permission();
        }
    }
    Ok(())
}

unsafe impl Send for RtssShem {}
unsafe impl Sync for RtssShem {}
pub struct RtssShem {
    handle: HANDLE,
    pub base_ptr: LPRTSS_SHARED_MEMORY,
}
impl Drop for RtssShem {
    fn drop(&mut self) {
        unsafe {
            UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                Value: self.base_ptr.cast(),
            })
            .unwrap();
            CloseHandle(self.handle).unwrap();
        }
    }
}
impl RtssShem {
    ///returns error if RivaTunner is not running
    pub fn init() -> Result<Self, RtssError> {
        unsafe {
            let handle = OpenFileMappingA(
                FILE_MAP_ALL_ACCESS.0,
                false,
                PCSTR(c"RTSSSharedMemoryV2".as_ptr().cast()),
            )
            .map_err(RtssError::Windows)?;
            let base_ptr = MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, 0);
            if base_ptr.Value.is_null() {
                return Err(RtssError::FailedToMapSharedMemory);
            }
            Ok(RtssShem {
                handle,
                base_ptr: base_ptr.Value.cast(),
            })
        }
    }
    pub fn update_profiles(&self) {
        unsafe {
            for entry in (0..(*self.base_ptr).dwAppArrSize)
                .map(|x| (*self.base_ptr).dwAppArrOffset + x * (*self.base_ptr).dwAppEntrySize)
                .map(|ptr| {
                    self.base_ptr.byte_add(ptr as usize)
                        as RTSS_SHARED_MEMORY_LPRTSS_SHARED_MEMORY_APP_ENTRY
                })
                .map(|entry_ptr| &mut *entry_ptr)
            {
                entry.dwFlags |= APPFLAG_PROFILE_UPDATE_REQUESTED;
            }
        }
    }
}
