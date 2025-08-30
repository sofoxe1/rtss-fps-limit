use rtss_rs::{get_fps_limit, get_write_acess, set_fps_limit};

fn main() {
    if set_fps_limit("factorio.exe", 20).is_err() {
        get_write_acess();
        set_fps_limit("factorio.exe", 20).unwrap();
    }
    println!("{:?}", get_fps_limit("factorio.exe").unwrap());
    set_fps_limit("factorio.exe", 30).unwrap();
    println!("{:?}", get_fps_limit("factorio.exe").unwrap());
}
