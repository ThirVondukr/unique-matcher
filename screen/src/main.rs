use chrono::Local;
use ini::Ini;
use screenshots::Screen;
use std::path::Path;
use std::path::PathBuf;
use std::{env, println};

fn main() {
    // Prepare paths
    let cur_dir = env::current_dir().unwrap();
    let workdir = cur_dir.as_path();
    let screen_dir = workdir.join("data").join("queue");
    let cfg_path = workdir.join("config.ini");
    let config_path_str = cfg_path.as_path().to_str().unwrap();

    // Load config
    let mut screen_id: i32 = match Ini::load_from_file(config_path_str) {
        Ok(ini_file) => ini_file
            .get_from_or(Some("screenshot"), "screen", "0")
            .parse::<i32>()
            .unwrap(),
        Err(_) => 0 as i32,
    };

    if screen_id == -1 {
        // Auto-detect
        let poe_config_path = poe_config_path().unwrap();
        screen_id = match active_monitor_number_from_poe_config(poe_config_path) {
            Some(val) => val as i32,
            None => {
                println!("Couldn't auto-detect PoE screen, defaulting to 0");
                0
            }
        }
    }

    println!("Using screen ID: {}", screen_id);

    // Prepare image path
    let local_time = Local::now();
    let filename = String::from(local_time.format("%Y-%m-%d-%H-%M-%S").to_string())
        + String::from(".png").as_str();
    let image_path_buf = screen_dir.join(filename);
    let image_path = image_path_buf.as_path();

    // Get all screens
    let screens = Screen::all().unwrap();

    // Use the first one
    if screen_id as usize >= screens.len() {
        println!(
            "Error: Cannot use screen {}, you only have {} screen(s) (IDs go from 0 to {})",
            screen_id,
            screens.len(),
            screens.len() - 1,
        );
    } else {
        let screen = screens[screen_id as usize];

        // Make screenshot
        let image = screen.capture().unwrap();
        image.save(image_path).unwrap();

        println!("Screenshot saved to: {}", &image_path.to_str().unwrap());
    }
}

/// cross-platform C:\Users\username\Documents\My Games\Path of Exile\production_Config.ini
fn poe_config_path() -> Option<PathBuf> {
    dirs_next::document_dir().map(|docs_dir| {
        docs_dir
            .join("My Games")
            .join("Path of Exile")
            .join("production_Config.ini")
    })
}

/// Read poe production_Config.ini, try to find the index of the preferred minitor.
/// Something like this:
///
///  adapter_name=AMD Radeon RX 5700 XT(#0)
///
fn active_monitor_number_from_poe_config<P>(poe_config_path: P) -> Option<usize>
where
    P: AsRef<Path>,
{
    let poe_config_path = poe_config_path.as_ref();

    if !poe_config_path.exists() {
        println!("PoE config path doesn't exist (are you on Linux?)");
        return None;
    }

    let ini_file = Ini::load_from_file(poe_config_path.to_str().unwrap()).unwrap();

    let adapter_name = ini_file.get_from_or(Some("DISPLAY"), "adapter_name", "(#0)");

    let Some(start) = adapter_name.rfind("(#") else {
        return None;
    };

    let Some(end) = adapter_name.rfind(")") else {
        return None;
    };

    let Some(substr) = adapter_name.get(start + 2..end) else {
        return None;
    };

    let monitor_index = substr.parse::<usize>().ok()?;

    Some(monitor_index)
}
