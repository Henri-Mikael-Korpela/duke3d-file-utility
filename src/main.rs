use grp::GrpFileReader;
use std::fs::{self, File};

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args();

    args.next(); // Skip the executable name.

    let Some(command) = args.next() else {
        return Err("No arguments provided.".to_string());
    };

    match command.as_str() {
        "grp-extract" => {
            let mut grp_file_path: Option<String> = None;
            let mut entry_file_name: Option<String> = None;
            let mut output_file_path: Option<String> = None;

            while let (Some(option), Some(value)) = (args.next(), args.next()) {
                match option.as_str() {
                    "--entry" => {
                        entry_file_name = Some(value);
                    }
                    "--input-file" => {
                        grp_file_path = Some(value);
                    }
                    "--output-file" => {
                        output_file_path = Some(value);
                    }
                    _ => {}
                }
            }

            match (grp_file_path, entry_file_name, output_file_path) {
                (Some(grp_file_path), Some(entry_file_name), Some(output_file_path)) => {
                    let curr_dir = std::env::current_dir().unwrap();
                    let file_path = curr_dir.join(grp_file_path);
                    let file = File::open(file_path).unwrap();

                    let mut grp_reader = GrpFileReader::new(&file)?;

                    if let Ok(Some(file_entry)) = grp_reader.find_file_entry(&entry_file_name) {
                        let file = grp_reader.read_file(&file_entry)?;
                        println!("File size: {}", file.len());
                        fs::write(curr_dir.join(output_file_path), file).unwrap();
                    }
                }
                _ => {
                    return Err("Missing arguments.".to_string());
                }
            }
        }
        _ => {
            return Err(format!("Unknown command: {}", command));
        }
    }

    Ok(())
}
