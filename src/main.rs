use std::env;
use std::fs::*;
use std::fs;
use std::process;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;


fn pwd() {
    if let Ok(path) = env::current_dir() {
        if let Some(path_str) = path.to_str() {
            println!("{}", path_str);
        }
    } else {
        eprintln!("Failed to get the current directory");
    }
}

fn echo(args: &[String]) {
    let mut skip_newline = false;
    let mut message_start = 2;
    if args[2] == "-n" {
        skip_newline = true;
        message_start += 1;
    }
    let message = args[message_start..].join(" ");
    print!("{}", message);
    if !skip_newline {
        println!();
    }
    process::exit(0);
}

fn touch(args: &[String]) {
    if args.len() > 2 {
        let filename = &args[2];
        match File::create(filename) {
            Ok(_) => println!("File '{}' created/updated.", filename),
            Err(e) => eprintln!("Error creating/updating file: {}", e),
        }
    } else {
        eprintln!("No filename specified for touch command.");
    }
}

fn mkdir(args: &[String]) {
    if args.len() > 2 {
        let dir_name = &args[2];
        match fs::create_dir(dir_name) {
            Ok(_) => println!("Directory '{}' created.", dir_name),
            Err(e) => {
                eprintln!("Error creating directory: {}", e);
                process::exit(-30);
            }
        }
    } else {
        eprintln!("No directory name specified for mkdir command.");
        process::exit(-30);
    }
}

fn rmdir(args: &[String]) {
    if args.len() > 2 {
        for dir in args.iter().skip(2) {
            match fs::remove_dir(dir) {
                Ok(_) => println!("Directory '{}' removed.", dir),
                Err(e) => {
                    eprintln!("Error removing directory '{}': {}", dir, e);
                    process::exit(-60);
                }
            }
        }
    } else {
        eprintln!("No directory name(s) specified for rmdir command.");
        process::exit(-60);
    }
}

fn rm(args: &[String]) {
    if args.len() > 2 {
        let mut is_recursive = false;
        let mut remove_dirs = false;
        let mut start_index = 2;
        if args[2].starts_with('-') {
            if args[2] == "-r" || args[2] == "-R" || args[2] == "--recursive" {
                is_recursive = true;
                start_index += 1;
            } else if args[2] == "-d" || args[2] == "--dir" {
                remove_dirs = true;
                start_index += 1;
            }
        }
        for file in args.iter().skip(start_index) {
            if is_recursive {
                if let Ok(metadata) = fs::metadata(file) {
                    if metadata.is_dir() {
                        if let Err(e) = fs::remove_dir_all(file) {
                            eprintln!("Error removing directory '{}': {}", file, e);
                            process::exit(-70);
                        } else {
                            println!("Directory '{}' removed.", file);
                        }
                    } else {
                        if let Err(e) = fs::remove_file(file) {
                            eprintln!("Error removing file '{}': {}", file, e);
                            process::exit(-70);
                        } else {
                            println!("File '{}' removed.", file);
                        }
                    }
                }
            } else if remove_dirs {
                if let Ok(metadata) = fs::metadata(file) {
                    if metadata.is_dir() {
                        if let Err(e) = fs::remove_dir(file) {
                            eprintln!("Error removing directory '{}': {}", file, e);
                            process::exit(-70);
                        } else {
                            println!("Directory '{}' removed.", file);
                        }
                    }
                }
            } else {
                if let Ok(metadata) = fs::metadata(file) {
                    if metadata.is_dir() {
                        eprintln!("Error: Cannot remove directories without '-r' or '-d' flag.");
                        process::exit(-70);
                    } else {
                        if let Err(e) = fs::remove_file(file) {
                            eprintln!("Error removing file '{}': {}", file, e);
                            process::exit(-70);
                        } else {
                            println!("File '{}' removed.", file);
                        }
                    }
                }
            }
        }
    } else {
        eprintln!("No file name(s) specified for rm command.");
        process::exit(-70);
    }
}

fn cat(args: &[String]) {
    if args.len() <= 2 {
        eprintln!("Usage: cat file1 file2 ...");
        std::process::exit(-20);
    }
    for file in args.iter().skip(2) {
        match fs::read_to_string(file) {
            Ok(content) => {
                print!("{}", content);
            }
            Err(e) => {
                eprintln!("Error reading file '{}': {}", file, e);
                std::process::exit(-20);
            }
        }
    }
}

fn mv(args: &[String]) {
    if args.len() != 4 {
        eprintln!("Usage: mv source destination");
        std::process::exit(-40);
    }
    let source = &args[2];
    let destination = &args[3];
    match fs::rename(source, destination) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error moving file/directory: {}", e);
            std::process::exit(-40);
        }
    }
}

fn cp(args: &[String]) {
    let mut source_index = 2;
    let mut destination_index = 3;
    let mut recursive = false;
    if args.len() > 3 {
        if args[2] == "-r" || args[2] == "-R" || args[2] == "--recursive" {
            recursive = true;
            source_index += 1;
            destination_index += 1;
        }
    }
    if args.len() < source_index + 1 || args.len() < destination_index + 1 {
        eprintln!("Insufficient arguments for cp command");
        std::process::exit(-1);
    }
    let source = &args[source_index];
    let destination = &args[destination_index];
    if !fs::metadata(source).is_ok() {
        eprintln!("Source file or directory '{}' does not exist", source);
        std::process::exit(-90);
    }
    if !recursive && fs::metadata(source).unwrap().is_dir() {
        eprintln!("Use '-r' flag to copy directories");
        std::process::exit(-90);
    }
    fn copy_recursive(source: &str, destination: &str) -> io::Result<()> {
        if fs::metadata(source)?.is_dir() {
            fs::create_dir_all(destination)?;
            for entry in fs::read_dir(source)? {
                let entry = entry?;
                let path = entry.path();
                let new_destination = format!("{}/{}", destination, path.file_name().unwrap().to_string_lossy());
                copy_recursive(&path.to_string_lossy(), &new_destination)?;
            }
        } else {
            fs::copy(source, destination)?;
        }
        Ok(())
    }
    match copy_recursive(source, destination) {
        Ok(_) => {
            if recursive {
                println!("Directory '{}' recursively copied to '{}'.", source, destination);
            } else {
                println!("File '{}' copied to '{}'.", source, destination);
            }
        }
        Err(e) => {
            eprintln!("Error copying: {}", e);
            std::process::exit(-90);
        }
    }
}


fn chmod(args: &[String]) {
    if args.len() != 4 {
        eprintln!("Usage: chmod permissions file/directory");
        std::process::exit(-25);
    }
    let permissions_str = &args[2];
    let path = &args[3];
    let permissions = match u32::from_str_radix(permissions_str, 8) {
        Ok(p) => p,
        Err(_) => {
            let mut perms = 0o0;
            for mode in permissions_str.chars() {
                match mode {
                    '+' => (),
                    '-' => (),
                    'u' => perms |= 0o700,
                    'g' => perms |= 0o070,
                    'o' => perms |= 0o007,
                    'a' => perms |= 0o777,
                    'r' => perms |= 0o444,
                    'w' => perms |= 0o200,
                    'x' => perms |= 0o111,
                    _ => {
                        eprintln!("Invalid command");
                        std::process::exit(-25);
                    }
                }
            }
            perms
        }
    };

    let mut current_permissions = match fs::metadata(path) {
        Ok(meta) => meta.permissions().mode(),
        Err(_) => {
            eprintln!("Invalid command");
            std::process::exit(-25);
        }
    };
    if permissions_str.chars().next().unwrap_or('+') != '+' {
        current_permissions = permissions;
    } else {
        if permissions_str.chars().nth(0).unwrap() == '-' {
            current_permissions &= !permissions;
        } else {
            current_permissions |= permissions;
        }
    }
    let new_permissions = fs::Permissions::from_mode(current_permissions);
    if let Err(e) = fs::set_permissions(path, new_permissions) {
        eprintln!("Error changing permissions: {}", e);
        std::process::exit(-25);
    } else {
        println!("Permissions changed for '{}'.", path);
    }
}
fn ls(args: &[String]) {
    let mut path = String::from("./");
    let mut show_hidden = false;
    let mut recursive = false;
    let mut index = 2;
    while args.len() > index && args[index].starts_with('-') {
        match args[index].as_str() {
            "-a" | "--all" => show_hidden = true,
            "-R" | "--recursive" => recursive = true,
            _ => {
                eprintln!("Unknown option: {}", args[index]);
                std::process::exit(-80);
            }
        }
        index += 1;
    }

    if args.len() > index {
        path = args[index].clone();
    }

    if let Ok(metadata) = fs::metadata(&path) {
        if metadata.is_file() {
            println!("{}", path);
        } else if metadata.is_dir() {
            list_directory_contents(&path, show_hidden, recursive);
        } else {
            eprintln!("Invalid file or directory.");
            std::process::exit(-80);
        }
    } else {
        eprintln!("Error accessing file/directory.");
        std::process::exit(-80);
    }
}

fn list_directory_contents(path: &str, show_hidden: bool, recursive: bool) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(file_name) = entry.file_name().to_str() {
                    if show_hidden || !file_name.starts_with('.') {
                        if recursive && entry.file_type().unwrap().is_dir() {
                            println!("{}", entry.path().display());
                            list_directory_contents(&entry.path().to_str().unwrap(), show_hidden, recursive);
                        } else {
                            println!("{}", entry.path().file_name().unwrap().to_str().unwrap());
                        }
                    }
                }
            }
        }
    } else {
        eprintln!("Error reading directory.");
        std::process::exit(-80);
    }
}

fn ln(args: &[String]) {

    let symbolic = args[2] == "-s" || args[2] == "--symbolic";
    let source_index = if symbolic { 3 } else { 2 };
    let link_index = if symbolic { 4 } else { 3 };
    let source = &args[source_index];
    let link = &args[link_index];
    if !Path::new(source).exists() {
        eprintln!("Error: Source file '{}' does not exist", source);
        std::process::exit(-50);
    }
    if Path::new(link).is_dir() {
        eprintln!("Error: Destination should be a file, not a directory");
        std::process::exit(-50);
    }
    if symbolic {
        match std::os::unix::fs::symlink(source, link) {
            Ok(()) => {
                println!("Symbolic link created from '{}' to '{}'.", source, link);
            }
            Err(e) => {
                eprintln!("Error creating symbolic link: {}", e);
                std::process::exit(-50);
            }
        }
    } else {
        match fs::hard_link(source, link) {
            Ok(()) => {
                println!("Hard link created from '{}' to '{}'.", source, link);
            }
            Err(e) => {
                eprintln!("Error creating hard link: {}", e);
                std::process::exit(-50);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "pwd" => pwd(),
            "echo" => echo(&args),
            "touch" => touch(&args),
            "mkdir" => mkdir(&args),
            "rmdir" => rmdir(&args),
            "rm" => rm(&args),
            "cat" => cat(&args),
            "mv" => mv(&args),
            "cp" => cp(&args),
            "chmod" => chmod(&args),
            "ln" => ln(&args),
            "ls" => ls(&args),
            _ => { println!("Invalid command");
            std::process::exit(-1);
            }
        }
    } else {
        println!("Invalid command");
        std::process::exit(-1);
    }
}