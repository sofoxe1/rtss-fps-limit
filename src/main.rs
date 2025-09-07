use std::time::SystemTime;

use rtss_rs::{get_fps_limit, get_profile_list, get_write_permission, has_write_permission, set_fps_limit};

fn main() {
    println!("profiles:{:#?}",get_profile_list());
    let now = SystemTime::now();
    if !has_write_permission() {
        get_write_permission();
    }
    set_fps_limit("factorio.exe", 20).unwrap();
    println!("{:?}", get_fps_limit("factorio.exe").unwrap());
    set_fps_limit("factorio.exe", 30).unwrap();
    println!("{:?}", get_fps_limit("factorio.exe").unwrap());
    println!("executed in:{}ms", now.elapsed().unwrap().as_millis());
}
