use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Simplified version of `wc`
struct Args {
    /// Input file(s)
    #[arg(value_name = "FILE", default_value = "-")]
    files: Vec<String>,
    /// Show line count
    #[arg(short, long)]
    lines: bool,
    /// Show word count
    #[arg(short, long)]
    words: bool,
    /// Show byte count
    #[arg(short('c'), long)]
    bytes: bool,
    /// Show character count
    #[arg(short('m'), long, conflicts_with("bytes"))]
    chars: bool,
}

#[derive(Debug, PartialEq)]
struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
}

impl std::ops::AddAssign for FileInfo {
    fn add_assign(&mut self, rhs: Self) {
        self.num_bytes += rhs.num_bytes;
        self.num_words += rhs.num_words;
        self.num_lines += rhs.num_lines;
    }
}

// --------------------------------------------------
fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

// --------------------------------------------------
fn run(mut args: Args) -> Result<()> {
    args.lines = true;
    args.words = true;
    args.bytes = true;
    let mut total = FileInfo {
        num_lines: 0,
        num_words: 0,
        num_bytes: 0,
    };
    for filename in &args.files {
        match open(filename) {
            Err(err) => eprintln!("{filename}: {err}"),
            Ok(file) => {
                if let Ok(info) = count(file) {
                    println!(
                        "{} = {} lines {} words {} bytes",
                        match filename {
                            "-" => "stdin",
                            default => default,
                        },
                        info.num_lines,
                        info.num_words,
                        info.num_bytes,
                    );
                    total += info;
                }
            }
        }
    }

    println!(
        "Total: {} lines {} words {} bytes",
        total.num_lines, total.num_words, total.num_bytes
    );
    Ok(())
}

// --------------------------------------------------
fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

// --------------------------------------------------
fn count(mut file: impl BufRead) -> Result<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut line = String::new();
    loop {
        let line_bytes = file.read_line(&mut line)?;
        if line_bytes == 0 {
            break;
        }
        num_bytes += line_bytes;
        num_lines += 1;
        for l in line.split(" ") {
            if l.trim() != "" {
                num_words += 1;
            }
        }
        line.clear();
    }
    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
    })
}
