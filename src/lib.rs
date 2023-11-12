use std::ffi::CStr;
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::io::{Read, Write};
use skyline::{nn, hook};
use crate::logging::dbg_println;

mod logging;

struct SaveLoaderState {
    allowed_files: Vec<String>,
    ready: bool,
}

const BASE_SAVES_PATH: &'static str = "sd:/xc3-saves";
const ALLOW_LIST_PATH: &'static str = "sd:/xc3-saves/allow-list.txt";

const DEFAULT_ALLOW_LIST: &'static str = r#"
# This allow list is used to determine which save files will be loaded from the sd card
# as well as which save files will be mirrored to the sd card when the game saves.
# Remove preceding # to allow a file to be copied from sd to save
#
# Keep the minimum number of files (ideally just the ones you want to modify) in this list,
# ideally around 3 or 4. As game will crash when trying to copy too many saves from the
# sd card to the save data in a row.

# bf3system00.sav # system settings
# bf3game00a.sav  # base game auto save slot
# bf3game00a.tmb  # base game auto save slot thumbnail
# bf3game01.sav   # base game quick save slot
# bf3game01.tmb   # base game quick save slot thumbnail
# bf3game02.sav   # base game manual save slot A
# bf3game02.tmb   # base game manual save slot A thumbnail
# bf3game03.sav   # base game manual save slot B
# bf3game03.tmb   # base game manual save slot B thumbnail
# bf3game04.sav   # base game manual save slot C
# bf3game04.tmb   # base game manual save slot C thumbnail
# bf3dlc00a.sav   # future redeemed DLC auto save slot
# bf3dlc00a.tmb   # future redeemed DLC auto save slot thumbnail
# bf3dlc01.sav    # future redeemed DLC quick save slot
# bf3dlc01.tmb    # future redeemed DLC quick save slot thumbnail
# bf3dlc02.sav    # future redeemed DLC manual save slot A
# bf3dlc02.tmb    # future redeemed DLC manual save slot A thumbnail
# bf3dlc03.sav    # future redeemed DLC manual save slot B
# bf3dlc03.tmb    # future redeemed DLC manual save slot B thumbnail
# bf3dlc04.sav    # future redeemed DLC manual save slot C
# bf3dlc04.tmb    # future redeemed DLC manual save slot C thumbnail
"#;

static mut SAVE_LOADER_STATE: Option<SaveLoaderState> = None;

pub unsafe fn is_allowed_file(file_path: &str) -> bool {
    let state = SAVE_LOADER_STATE.as_mut().unwrap();
    if !state.ready {
        return false;
    }

    let mut allowed = false;
    for file in &state.allowed_files {
        if file_path.contains(file) {
            allowed = true;
            break;
        }
    }

    return allowed;
}

#[hook(offset = 0x002ffe7c)]
unsafe fn do_load_file(gstate: u64, file_path: *mut u8, data: *const u8, data_size: u64, p5: u32) {
    let orig_file_path = CStr::from_ptr(file_path as *const _);
    let orig_file_path = orig_file_path.to_str().unwrap();

    let state = SAVE_LOADER_STATE.as_mut().unwrap();
    if state.ready && is_allowed_file(orig_file_path) {
        let override_file_path = orig_file_path.replace("save:", BASE_SAVES_PATH);

        if std::path::Path::new(&override_file_path).exists() {
            dbg_println!("Overriding {} with {}", orig_file_path, override_file_path);       

            match std::fs::copy(override_file_path, orig_file_path) {
                Ok(_) => {},
                Err(e) => {
                    dbg_println!("Failed to copy file: {}", e);
                }
            };
        }
    }

    call_original!(gstate, file_path, data, data_size, p5);
}

#[hook(offset = 0x002ffd70)]
unsafe fn do_save_file(gstate: u64, file_path: *const u8, data: *const u8, data_size: u64, p5: u64) {
    let orig_file_path = CStr::from_ptr(file_path as *const _);
    let orig_file_path = orig_file_path.to_str().unwrap();

    call_original!(gstate, file_path, data, data_size, p5);

    if !is_allowed_file(orig_file_path) {
        return;
    }

    let dup_file_path = orig_file_path.replace("save:", BASE_SAVES_PATH);
    dbg_println!("Mirroring save from {} to {}", orig_file_path, dup_file_path);

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(dup_file_path)
        .unwrap();

    let mut buffer: Vec<u8> = Vec::with_capacity(data_size as usize);
    buffer.set_len(data_size as usize);

    std::ptr::copy(data, buffer.as_mut_ptr(), data_size as usize);

    file.write_all(&buffer).unwrap();

    drop(file);
}

unsafe fn initialize_mod() {
    let sd_device = CString::new("sd").unwrap();    
    let res = nn::fs::MountSdCardForDebug(sd_device.as_ptr() as *const _ as *const _);
    if res == 0 {
        dbg_println!("Mounted sd card.");
    } else if res == 30722 {
        dbg_println!("SD card already mounted.");
    } else if res != 0 && res != 30722 {
        dbg_println!("Failed to mount sd card: {}", res);
        dbg_println!("Mod failed to initialize.");
        return;
    }

    dbg_println!("Checking for saves directory {}", BASE_SAVES_PATH);

    let base_saves_path = CString::new(BASE_SAVES_PATH).unwrap();

    let mut handle: nn::fs::DirectoryHandle = MaybeUninit::zeroed().assume_init();
    let res = nn::fs::OpenDirectory(
        &mut handle as *mut _, 
        base_saves_path.as_ptr() as *const _,
        nn::fs::OpenDirectoryMode_OpenDirectoryMode_All as i32
    );

    let state = SAVE_LOADER_STATE.as_mut().unwrap();

    if res == 0 {
        state.ready = true;
    } else if res != 0 {
        dbg_println!("Failed to open saves directory: {}", res);
        dbg_println!("Attempting to create saves directory: {}", res);
        let res = nn::fs::CreateDirectory(base_saves_path.as_ptr() as *const _);
        if res != 0 {
            dbg_println!("Failed to create saves directory: {}", res);
            dbg_println!("Mod failed to initialize.");
            return;
        }

        let res = nn::fs::OpenDirectory(
            &mut handle as *mut _, 
            base_saves_path.as_ptr() as *const _,
            nn::fs::OpenDirectoryMode_OpenDirectoryMode_All as i32
        );        
        if res != 0 {
            dbg_println!("Failed to open saves directory: {}", res);
            dbg_println!("Mod failed to initialize.");
            return;
        }
        state.ready = true;
    }

    if !state.ready {
        return;
    }

    dbg_println!("Mod initialized");
    dbg_println!("Loading allow list from {}", ALLOW_LIST_PATH);

    if !std::path::Path::new(ALLOW_LIST_PATH).exists() {
        dbg_println!("Allow list does not exist. Creating default allow list at {}", ALLOW_LIST_PATH);

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(ALLOW_LIST_PATH)
            .unwrap();

        file.write_all(DEFAULT_ALLOW_LIST.as_bytes()).unwrap();
        drop(file);
    }

    let mut allow_list_file = std::fs::File::open(ALLOW_LIST_PATH).unwrap();
    let mut allow_list_contents = String::new();
    allow_list_file.read_to_string(&mut allow_list_contents).unwrap();

    let allow_list_contents = allow_list_contents.replace("\r\n", "\n");
    let allow_list_contents = allow_list_contents.replace("\r", "\n");

    let allow_list_contents = allow_list_contents.split("\n");

    for line in allow_list_contents {
        let filename = line.trim().split("#").nth(0).unwrap().trim();

        if filename.len() == 0 {
            continue;
        }

        dbg_println!("Adding {} to allow list", filename);
        state.allowed_files.push(filename.to_string());
    }
}

// InitMountSaveData
#[hook(offset = 0x01252d7c)]
unsafe fn init_mount_save_data() {
    call_original!();

    // copy files from sd to save
    let state = SAVE_LOADER_STATE.as_mut().unwrap();
    if !state.ready {
        return;
    }

    dbg_println!("Game has mounted save data. Copying files.");

    for file in &state.allowed_files {
        let sd_file_path = format!("{}/{}", BASE_SAVES_PATH, file);
        let save_file_path = format!("save:/{}", file);

        if !std::path::Path::new(&sd_file_path).exists() {
            continue;
        }

        dbg_println!("Copying {} to {}", sd_file_path, save_file_path);

        match std::fs::copy(sd_file_path, save_file_path) {
            Ok(_) => {},
            Err(e) => {
                dbg_println!("Failed to copy file: {}", e);
            }
        };
    }
    dbg_println!("Done copying files");
}

#[skyline::main(name = "xc3_sd_save_loader")]
pub fn main() {
    dbg_println!("xc3_sd_save_loader loaded");
    unsafe {
        SAVE_LOADER_STATE = Some(SaveLoaderState {
            ready: false,
            allowed_files: Vec::new(),
        });
        initialize_mod()
    }
    skyline::install_hooks!(init_mount_save_data, do_load_file, do_save_file);
}
