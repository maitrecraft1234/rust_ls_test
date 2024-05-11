use std::env;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::time::SystemTime;

#[allow(clippy::struct_excessive_bools)]
struct Options {
    long_list: bool,
    all: bool,
    directory: bool,
    recursive: bool,
}

impl Options {
    const fn new() -> Self {
        Self {
            long_list: false,
            all: false,
            directory: false,
            recursive: false,
        }
    }
    fn options(&mut self, arg: &str) {
        if arg.contains('l') {
            self.long_list = true;
        }
        if arg.contains('a') {
            self.all = true;
        }
        if arg.contains('d') {
            self.directory = true;
        }
        if arg.contains('R') {
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
    let str = format!(
        "last mod {}y {}m {}d {}h {}m {}s",
        years, months, days, hours, mins, secs
    );

    str
}

fn perm_string(mode: u32) -> String {
    let user = (mode & 0b111_000_000) >> 6;
    let group = (mode & 0b111_000) >> 3;
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
        _ => unreachable!(),
    };
    let group = format!(
        "{}{}{}",
        if group & 0b100 == 0 { "-" } else { "r" },
        if group & 0b010 == 0 { "-" } else { "w" },
        if group & 0b001 == 0 { "-" } else { "x" }
    );
    let other = format!(
        "{}{}{}",
        if other & 0b100 == 0 { "-" } else { "r" },
        if other & 0b010 == 0 { "-" } else { "w" },
        if other & 0b001 == 0 { "-" } else { "x" }
    );

    format!("{}{}{}", user, group, other)
}

fn print(entry_dir: &str, options: &Options, indent: &str) {
    let entries = { fs::read_dir(entry_dir).unwrap() };
    for entry in entries {
        let entry = entry.unwrap();

        if !options.all {
            if let Some(byte) = entry.file_name().as_encoded_bytes().first() {
                if *byte == b'.' {
                    continue;
                }
            }
        }
        if options.directory && entry.file_name().to_str() != Some(entry_dir) {
            continue;
        }
        if options.long_list {
            let metadata = entry.metadata().expect("meta");
            println!(
                "{}{} {} {} {} {} {} {} {}",
                indent,
                time_string(metadata.modified().expect("mod")),
                metadata.nlink(),
                metadata.uid(),
                metadata.gid(),
                metadata.len(),
                perm_string(metadata.permissions().mode()),
                if entry
                    .file_type()
                    .expect("idk what I was expecting")
                    .is_dir()
                {
                    "\x1b[94m \x1b[0m"
                } else {
                    "\x1b[90m \x1b[0m"
                },
                entry.file_name().to_str().unwrap()
            );
        } else {
            println!("{}{}", indent, entry.file_name().to_str().unwrap());
        }
        if options.recursive
            && entry
                .file_type()
                .expect("idk what I was expecting")
                .is_dir()
        {
            println!("\n{}{:?} :", indent, entry.path());
            print(
                entry.path().to_str().unwrap(),
                options,
                &format!("{}\t", indent),
            );
        }
    }
}

fn main() {
    let mut options = Options::new();
    let mut args: Vec<String> = Vec::new();

    for arg in env::args().skip(1) {
        if !arg.starts_with('-') {
            args.push(arg.to_string());
            continue;
        }
        options.options(&arg);
    }
    if args.is_empty() {
        print(".", &options, "");
    } else {
        println!(
            "{} {} {} {} {:?}",
            args.len(),
            options.directory,
            options.all,
            options.long_list,
            args
        );
        for arg in &args {
            print(arg, &options, "");
        }
    }
}
