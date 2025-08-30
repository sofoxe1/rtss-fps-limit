use std::{ffi::CString, str::FromStr};

mod backend;
pub use backend::*;
pub fn get_fps_limit(profile_name: &str) -> Result<u32, RtssError> {
    if !profile_exists(profile_name) {
        return Err(RtssError::ProfileNotFound);
    }
    load_profile(&CString::from_str(profile_name).unwrap());
    get_profile_property(&CString::from_str("FramerateLimit").unwrap())
        .ok_or(RtssError::FailedToGetValue)
}
pub fn set_fps_limit(profile_name: &str, value: u32) -> Result<(), RtssError> {
    let load_str = match profile_exists(profile_name) {
        true => profile_name.to_string(),
        //load global profile if game specific doesnt exist
        false => "".to_string(),
    };
    load_profile(&CString::from_str(&load_str).unwrap());
    set_profile_property(&CString::from_str("FramerateLimit").unwrap(), value)
        .ok_or(RtssError::FailedToSetValue)?;
    save_profile(&CString::from_str(profile_name).unwrap());
    if get_fps_limit(profile_name).unwrap() != value {
        return Err(RtssError::FailedToUpdateProfile);
    }
    update_profiles();
    Ok(())
}
