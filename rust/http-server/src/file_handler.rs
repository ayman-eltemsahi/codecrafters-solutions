use std::{path::Path, env, fs};

fn get_files_dir() -> Option<String> {
    let args: Vec<String> = env::args().collect();
    if args.len() >= 3 && args[1] == "--directory" {
        Some(args[2].to_owned())
    } else {
        None
    }
}

pub fn read_file_content(filename: &str) -> Option<String> {
    match get_files_dir() {
        None => None,
        Some(dir) => {
            let file_path = Path::new(&dir).join(filename);
            match fs::read_to_string(file_path) {
                Ok(file) => Some(file),
                Err(_) => None,
            }
        }
    }
}

pub fn write_file_content(filename: &str, content: &str) -> Result<(), &'static str> {
    match get_files_dir() {
        None => Err("did not receive a dir"),
        Some(dir) => {
            let file_path = Path::new(&dir).join(filename);
            match fs::write(file_path, content) {
                Err(_) => Err("failed to write the file"),
                Ok(_) => Ok(()),
            }
        }
    }
}
