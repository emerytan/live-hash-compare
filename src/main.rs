use chrono::Local;
use colored::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use clap::{Parser, ArgGroup};
use walkdir::WalkDir;
use md5;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use terminal_size::{Width, terminal_size};
use rayon::prelude::*;

/// Compare live file hashes to a reference md5 file and report differences
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to directory containing files to hash
    #[arg(short, long)]
    files_path: String,

    /// Path to md5 reference file
    #[arg(short, long)]
    md5_file: String,

    /// Path to write the results report
    #[arg(short, long, value_name = "PATH")]
    report_path: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Set default report path if not provided
    let default_report = || {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/live-hash/report.txt", home)
    };
    let mut report_path = args.report_path.clone().unwrap_or_else(default_report);
    if Path::new(&report_path).exists() {
        let now = Local::now();
        let ts = now.format("%Y%m%d_%H-%M-%S");
        let path = Path::new(&report_path);
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let ext = path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        let new_name = format!("{}_{}{}", stem, ts, ext);
        report_path = parent.join(new_name).to_string_lossy().to_string();
    }

    // Collect all file paths first
    let file_paths: Vec<_> = WalkDir::new(&args.files_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();

    let total_files = file_paths.len();

    // Determine terminal width for progress bar
    let term_width = if let Some((Width(w), _)) = terminal_size() {
        (w as f32 * 0.9) as usize
    } else {
        80
    };

    let pb = ProgressBar::new(total_files as u64);
    pb.set_style(ProgressStyle::with_template(&format!(
        "{{bar:.{}}} {{pos}}/{{len}} files | {{percent}}% {{msg}}",
        term_width.saturating_sub(50)
    ))
    .unwrap()
    .progress_chars("=> "));

    use std::sync::{Arc, Mutex};
    let pb = Arc::new(pb);
    let live_hashes = Arc::new(Mutex::new(HashMap::new()));
    let pass_count = Arc::new(Mutex::new(0u64));
    let fail_count = Arc::new(Mutex::new(0u64));

    // Build a map from filename (no path) to hash for reference hashes (needed for comparison)
    let ref_hashes = read_md5_file(&args.md5_file)?;
    let mut ref_by_filename: HashMap<String, String> = HashMap::new();
    for (path, hash) in &ref_hashes {
        if let Some(filename) = Path::new(path).file_name().map(|s| s.to_string_lossy().to_string()) {
            ref_by_filename.insert(filename, hash.clone());
        }
    }

    file_paths.par_iter().for_each(|entry| {
        let path = entry.path();
        let rel_path = path.strip_prefix(entry.path().ancestors().last().unwrap())
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        let hash = md5_file(path).unwrap_or_else(|_| "ERROR".to_string());
        let filename = Path::new(&rel_path).file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| rel_path.clone());
        let mut pass = pass_count.lock().unwrap();
        let mut fail = fail_count.lock().unwrap();
        match ref_by_filename.get(&filename) {
            Some(ref_hash) if ref_hash == &hash => {
                *pass += 1;
            },
            _ => {
                *fail += 1;
            }
        }
        live_hashes.lock().unwrap().insert(rel_path, hash);
        pb.set_message(format!(
            "{} {}    {} {}",
            "PASS:".green(), *pass, "FAIL:".red(), *fail
        ));
        pb.inc(1);
    });
    pb.finish_with_message("Hashing complete");
    let live_hashes = Arc::try_unwrap(live_hashes).unwrap().into_inner().unwrap();
    let fail = Arc::try_unwrap(fail_count).unwrap().into_inner().unwrap();
    let ref_hashes = read_md5_file(&args.md5_file)?;
    // Ensure parent directory exists
    if let Some(parent) = Path::new(&report_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut report = File::create(&report_path)?;

    // Build a map from filename (no path) to hash for reference hashes
    let mut ref_by_filename: HashMap<String, String> = HashMap::new();
    for (path, hash) in &ref_hashes {
        if let Some(filename) = Path::new(path).file_name().map(|s| s.to_string_lossy().to_string()) {
            ref_by_filename.insert(filename, hash.clone());
        }
    }

    // For each file in live_hashes, compare by filename only
    let mut mismatches = Vec::new();
    for (path, live_hash) in &live_hashes {
        let filename = Path::new(path).file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| path.clone());
        match ref_by_filename.get(&filename) {
            Some(ref_hash) if ref_hash == live_hash => {
                writeln!(report, "{}\t{}\tMATCH", filename, live_hash)?;
            },
            Some(ref_hash) => {
                writeln!(report, "{}\t{}\tFAIL", filename, live_hash)?;
                mismatches.push((filename.clone(), live_hash.clone(), ref_hash.clone()));
            },
            None => {
                writeln!(report, "{}\t{}\tFAIL", filename, live_hash)?;
            }
        }
    }

    if mismatches.is_empty() {
        println!(
            "\n{}\n{} files checked, no mismatches found.\nAll good.\n",
            "====================".green(),
            total_files
        );
    } else {
        println!("Mismatched files:");
        for (filename, live_hash, ref_hash) in &mismatches {
            println!("{}\n  live: {}\n  ref:  {}", filename, live_hash, ref_hash);
        }
        // Print FAIL counter in red
        println!("{} {}", "FAIL count:".red(), fail);
    }
    println!("report written to {}", report_path);
    Ok(())
}


fn md5_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(&path)?;
    let mut buffer = Vec::new();
    io::copy(&mut file, &mut buffer)?;
    let digest = md5::compute(&buffer);
    Ok(format!("{:x}", digest))
}

fn read_md5_file<P: AsRef<Path>>(path: P) -> Result<HashMap<String, String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map = HashMap::new();
    for line in reader.lines() {
        let line = line?;
        let mut parts = line.split_whitespace();
        if let (Some(hash), Some(filename)) = (parts.next(), parts.next()) {
            map.insert(filename.to_string(), hash.to_string());
        }
    }
    Ok(map)
}
