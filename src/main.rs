use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::time::SystemTime;
use std::os::unix::fs::MetadataExt;

struct Options {
    long_list: bool,
    all: bool,
    directory: bool,
    recursive: bool,
}

impl Options {
    fn new() -> Options {
        Options {
            long_list: false,
            all: false,
            directory: false,
            recursive: false,
        }
    }
    fn options(&mut self, arg: &String) {
        if arg.contains("l") {
            self.long_list = true;
        }
        if arg.contains("a") {
            self.all = true;
        }
        if arg.contains("d") {
            self.directory = true;
        }
        if arg.contains("R") {
            self.recursive = true;
        }
    }
}

fn time_string(time: SystemTime) -> String {
    let time = SystemTime::now().duration_since(time).unwrap().as_secs();
    let secs = time % 60;
    let mins = (time / 60) % 60;
    let hours = (time / 3600) % 24;
    let days = (time / 86400) % 30;
    let months = (days / 30) % 12;
    let years = days / 365;
    let str = format!("last mod {}y {}m {}d {}h {}m {}s", years, months, days, hours, mins, secs);
    
    str
}

fn perm_string(mode: u32) -> String {
    let user = (mode & 0b111000000) >> 6;
    let group = (mode & 0b111000) >> 3;
    let other = mode & 0b111;

    let user = match user {
        0 => "---",
        1 => "--x",
        2 => "-w-",
        3 => "-wx",
        4 => "r--",
        5 => "r-x",
        6 => "rw-",
        7 => "rwx",
        _ => "???",
    };
    let group = format!("{}{}{}", if group & 0b100 == 0 {"-"} else {"r"}, if group & 0b010 == 0 {"-"} else {"w"}, if group & 0b001 == 0 {"-"} else {"x"});
    let other = format!("{}{}{}", if other & 0b100 == 0 {"-"} else {"r"}, if other & 0b010 == 0 {"-"} else {"w"}, if other & 0b001 == 0 {"-"} else {"x"});

    format!("{}{}{}", user, group, other)
}

fn print(entry_dir: &String, options: &Options, indent: &String) {
    let entries = {fs::read_dir(entry_dir).unwrap()};
    for entry in entries {

        if !options.all{
            if let Some(ref byte) = entry.as_ref().unwrap().file_name().as_encoded_bytes().get(0) {
                if **byte == b'.' {
                    continue;
                }
            }
        }
        if options.directory && entry.as_ref().expect("a").file_name().to_str() != Some(entry_dir.as_str()) {
            continue;
        }
        if !options.long_list {
            println!("{}{}", indent, entry.as_ref().unwrap().file_name().to_str().unwrap());
        } else {
            let metadata = entry.as_ref().expect("some error im not suer").metadata().expect("meta");
            println!("{}{} {} {} {} {} {} {} {}", indent,
                     time_string(metadata.modified().expect("mod")),
                     metadata.nlink(),
                     metadata.uid(),
                     metadata.gid(),
                     metadata.len(),
                     perm_string(metadata.permissions().mode()),
                     if entry.as_ref().expect("idk").file_type().expect("idk what I was expecting").is_dir() {"\x1b[94m \x1b[0m"} else {"\x1b[90m \x1b[0m"},
                     entry.as_ref().unwrap().file_name().to_str().unwrap());
        }
        if options.recursive && entry.as_ref().expect("idk").file_type().expect("idk what I was expecting").is_dir() {
            println!("\n{}{:?} :",indent , entry.as_ref().unwrap().path());
            print(&entry.as_ref().unwrap().path().to_str().unwrap().to_string(), options, &format!("{}\t", indent));
        }
    }
}

fn main() {
    let argv: Vec<String> = env::args().collect();
    let mut options = Options::new();
    let mut args: Vec<String> = Vec::new();

    for arg in argv.iter() {
        if !arg.starts_with("-") {
            args.push(arg.to_string());
            continue;
        }
        options.options(arg);
    }
    if args.len() > 1 {
        println!("{} {} {} {} {:?}", args.len(), options.directory, options.all, options.long_list, args);
        for i in 1..args.len() {
            print(&argv[i], &options, &"".to_string());
        }
    } else {
        print(&".".to_string(), &options, &"".to_string());
    }
}
