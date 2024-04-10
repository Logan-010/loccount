use serde::Deserialize;
use std::collections::HashMap;
use colored::*;

const JSON_DATA: &str = include_str!("../data.json");

#[derive(Deserialize)]
struct LangDatabase {
    name: String,
    extensions: Vec<String>,
}

fn add_dirs(num: &mut u64, dir: &str, lang: &mut (&mut HashMap<String, u64>, Vec<LangDatabase>)) {
    // Get all files from directory (user error if failure)
    let paths = std::fs::read_dir(dir).unwrap();

    // Iterate through files
    for path in paths {
        // Never fails
        let file_name = path.unwrap().path();

        // Recursive call to function if the entry is a directory
        if file_name.is_dir() {
            add_dirs(num, &file_name.display().to_string(), lang);
            continue;
        }

        // Read file
        let file_contents = match std::fs::read_to_string(&file_name) {
            Ok(v) => v,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::InvalidData {
                    // Skips file if it is binary
                    continue;
                } else {
                    // Actual failure to open file
                    eprintln!("File: {:?} - Error: {:?}", file_name, e);
                    std::process::exit(1);
                }
            }
        };

        let loc_num = file_contents.split('\n').collect::<Vec<&str>>().len() as u64;

        // Add to LOC count
        *num += loc_num;

        // Add to lang hashmap
        let file_extension = file_name.extension().and_then(std::ffi::OsStr::to_str);
        let mut matched = false;
        if let Some(ext) = file_extension {
            let ext = format!(".{}", ext);

            // Iterate through known languages
            for language in &lang.1 {
                for extension in &language.extensions {
                    if &ext == extension {
                        // Get & update value or insert it
                        match lang.0.get_mut(&language.name) {
                            Some(v) => *v += loc_num,
                            None => {
                                lang.0.insert(language.name.clone(), loc_num);
                            }
                        }

                        matched = true;
                    }
                }
            }
        }

        // If no known language was found
        if !matched {
            let temp = String::from("Unknown");

            // Get & update value or insert it
            match lang.0.get_mut(&temp) {
                Some(v) => *v += loc_num,
                None => {
                    lang.0.insert(temp, loc_num);
                }
            }
        }
    }
}

fn main() {
    // Get args
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("{}: {} <{}>", "USAGE".bold(), &args[0].purple(), "DIR".italic());
        std::process::exit(1);
    }

    // Initialize app state
    let dir = &args[1];
    let mut loc_s: u64 = 0;
    let mut lang_hashmap = HashMap::new();
    // Should NEVER fail
    let lang_data: Vec<LangDatabase> = serde_json::from_str(JSON_DATA).unwrap();

    println!("{}", "Searching directory...".bold());

    // Call function to count LOC
    add_dirs(&mut loc_s, dir, &mut (&mut lang_hashmap, lang_data));

    println!("{}", "Done!".bold());

    // Sort languages based on LOC count
    let mut langs: Vec<(&String, &u64)> = lang_hashmap.iter().collect();
    langs.sort_by(|a, b| b.1.cmp(a.1));

    // Print language data
    for (name, count) in langs {
        println!(
            "{} - {} lines ({} of {}).",
            name.purple().bold(),
            format!("{}", count).green().bold(),
            format!("%{:.2}", (100.0 / loc_s as f32) * *count as f32).red().bold(),
            "LOC".cyan()
        );
    }

    // Print total LOC
    println!("{} total {}.", format!("{}", loc_s).green().bold(), "LOC".cyan());
}
