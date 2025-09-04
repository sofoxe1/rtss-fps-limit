use std::{
    ffi::CString,
    str::FromStr,
    sync::{LazyLock, Mutex},
};

mod backend;
pub use backend::*;
mod helpers;
pub use helpers::*;

//to edit any other field u have to
//1. load the profile
//2. set the field value
//3. save it
//4. call update_profiles to make rtss load a modified version
static RTSS_SHEM: LazyLock<Mutex<RtssShem>> =
    LazyLock::new(|| Mutex::new(RtssShem::init().unwrap()));
pub fn get_fps_limit(profile_name: &str) -> Result<u32, RtssError> {
    if !profile_exists(profile_name) {
        return Err(RtssError::ProfileNotFound);
    }
    let profile = load_profile(&CString::from_str(profile_name).unwrap())?;
    let value = profile
        .get_from(Some("Framerate"), "Limit")
        .ok_or(RtssError::FailedToGetValue)?;
    value.parse().map_err(RtssError::ParseError)
}
pub fn set_fps_limit(profile_name: &str, value: u32) -> Result<(), RtssError> {
    let load_str = match profile_exists(profile_name) {
        true => profile_name.to_string(),
        //load global profile if game specific doesnt exist
        false => "".to_string(),
    };
    let mut profile = load_profile(&CString::from_str(&load_str).unwrap())?;
    profile.set_to(Some("Framerate"), "Limit".to_owned(), value.to_string());
    save_profile(&CString::from_str(profile_name).unwrap(), &profile)?;
    if get_fps_limit(profile_name).unwrap() != value {
        return Err(RtssError::FailedToUpdateProfile);
    }
    RTSS_SHEM.lock().unwrap().update_profiles();
    Ok(())
}
