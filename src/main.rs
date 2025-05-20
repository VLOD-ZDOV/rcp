use clap::{Arg, ArgAction, Command};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    fs,
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;
use filetime::FileTime;

fn main() {
    let matches = Command::new("rcp")
    .version("0.1")
    .about("Rust-based cp/mv alternative")
    .arg(Arg::new("source").required(true))
    .arg(Arg::new("destination").required(true))
    .arg(Arg::new("recursive")
    .short('r')
    .long("recursive")
    .action(ArgAction::SetTrue)
    .help("Copy directories recursively"))
    .arg(Arg::new("preserve")
    .short('p')
    .long("preserve")
    .action(ArgAction::SetTrue)
    .help("Preserve file attributes"))
    .arg(Arg::new("verbose")
    .short('v')
    .long("verbose")
    .action(ArgAction::SetTrue)
    .help("Verbose output"))
    .get_matches();

    let src = PathBuf::from(matches.get_one::<String>("source").unwrap());
    let dst = PathBuf::from(matches.get_one::<String>("destination").unwrap());
    let recursive = matches.get_flag("recursive");
    let preserve = matches.get_flag("preserve");
    let verbose = matches.get_flag("verbose");

    if src.is_file() {
        copy_file(&src, &dst, preserve, verbose).unwrap();
    } else if src.is_dir() {
        if !recursive {
            eprintln!("{}", "Use -r to copy directories.".bright_red());
            std::process::exit(1);
        }
        copy_dir_recursive(&src, &dst, preserve, verbose).unwrap();
    } else {
        eprintln!("{}", "Source not found.".bright_red());
    }
}

fn copy_file(src: &Path, dst: &Path, preserve: bool, verbose: bool) -> std::io::Result<()> {
    if verbose {
        println!(
            "{} {} → {}",
            "Copying".cyan(),
                 src.display(),
                 dst.display()
        );
    }

    let metadata = fs::metadata(src)?;
    let total_size = metadata.len();
    let bar = ProgressBar::new(total_size);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta})",
        )
        .unwrap()
        .progress_chars("=> "),
    );

    let mut reader = File::open(src)?;
    let mut writer = OpenOptions::new().create(true).write(true).truncate(true).open(dst)?;

    let mut buffer = [0u8; 8192];
    let mut copied = 0;

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        writer.write_all(&buffer[..n])?;
        copied += n as u64;
        bar.set_position(copied);
    }

    bar.finish_and_clear();

    if preserve {
        fs::set_permissions(dst, metadata.permissions())?;
        let atime = FileTime::from_last_access_time(&metadata);
        let mtime = FileTime::from_last_modification_time(&metadata);
        filetime::set_file_times(dst, atime, mtime)?;
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path, preserve: bool, verbose: bool) -> std::io::Result<()> {
    for entry in WalkDir::new(src) {
        let entry = entry?;
        let rel_path = entry.path().strip_prefix(src).unwrap();
        let dest_path = dst.join(rel_path);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            copy_file(entry.path(), &dest_path, preserve, verbose)?;
        }
    }
    Ok(())
}

