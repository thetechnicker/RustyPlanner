#[cfg(not(test))]
use directories::BaseDirs;
#[cfg(not(test))]
use std::fs;
use std::path::PathBuf;

#[cfg(test)]
pub fn get_path() -> std::option::Option<PathBuf> {
    let tmp_dir = std::env::temp_dir();
    Some(tmp_dir.join("dates.json"))
}

#[cfg(not(test))]
pub fn get_path() -> std::option::Option<PathBuf> {
    let data_file_path: Option<PathBuf>;

    if let Some(base_dirs) = BaseDirs::new() {
        let data_base_dir = base_dirs.data_dir();

        println!("Data Directory: {:?}", data_base_dir);

        let data_dir = data_base_dir.join("RustyPlanner");

        fs::create_dir_all(data_dir.clone()).expect("Failed to create data directory");

        data_file_path = Some(data_dir.join("dates.json"));
    } else {
        eprintln!("Could not find base directories.");
        data_file_path = None;
    }

    return data_file_path;
}
