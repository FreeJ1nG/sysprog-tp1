use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

#[derive(ValueEnum, PartialEq, Clone, Default, Debug)]
#[clap(rename_all = "lowercase")]
enum TotalWhen {
    #[default]
    Always,
    Never,
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Simplified version of `wc`
struct Args {
    /// Input file(s)
    #[arg(value_name = "FILE", default_value = "-")]
    files: Vec<String>,
    /// Show line count
    #[arg(short('l'), long)]
    lines: bool,
    /// Show word count
    #[arg(short('w'), long)]
    words: bool,
    /// Show byte count
    #[arg(short('c'), long)]
    bytes: bool,
    /// Show character count
    #[arg(short('m'), long, conflicts_with("bytes"))]
    chars: bool,
    // When to print a line with total counts; WHEN can be: auto, always, only, never
    #[arg(long, default_value_t, value_enum)]
    total: TotalWhen,
}

#[derive(Debug, PartialEq)]
struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

impl std::ops::AddAssign for FileInfo {
    fn add_assign(&mut self, rhs: Self) {
        self.num_bytes += rhs.num_bytes;
        self.num_words += rhs.num_words;
        self.num_lines += rhs.num_lines;
        self.num_chars += rhs.num_chars;
    }
}

// --------------------------------------------------
fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn display(lhs: &str, info: &FileInfo, args: &Args) {
    let mut res = String::new();
    if args.lines {
        res += &format!(" {} lines", info.num_lines);
    }
    if args.words {
        res += &format!(" {} words", info.num_words);
    }
    if args.chars {
        res += &format!(" {} chars", info.num_chars);
    }
    if args.bytes {
        res += &format!(" {} bytes", info.num_bytes);
    }
    println!("{lhs} ={res}");
}

// --------------------------------------------------
fn run(mut args: Args) -> Result<()> {
    if !args.words && !args.lines && !args.chars && !args.bytes {
        args.words = true;
        args.lines = true;
        args.chars = true;
        args.bytes = true;
    }
    let mut total = FileInfo {
        num_lines: 0,
        num_words: 0,
        num_bytes: 0,
        num_chars: 0,
    };

    if args.files.len() == 1 && args.files[0] == "-" {
        let mut content = Vec::<u8>::new();
        std::io::stdin().read_to_end(&mut content)?;
        let content = String::from_utf8(content)?;
        let info = count_in_str(&content);
        display("stdin (keyboard input)", &info, &args);
        total += info;
    } else {
        for filename in &args.files {
            match open(filename) {
                Err(err) => eprintln!("{filename}: {err}"),
                Ok(file) => {
                    if let Ok(info) = count_file(file) {
                        let display_filename = if filename == "-" { "stdin" } else { filename };
                        display(&display_filename, &info, &args);
                        total += info;
                    }
                }
            }
        }
    }

    if args.total == TotalWhen::Always {
        display("Total", &total, &args);
    }
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
fn count_file(mut file: impl BufRead) -> Result<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;
    let mut line = String::new();
    loop {
        let line_bytes = file.read_line(&mut line)?;
        if line_bytes == 0 {
            break;
        }
        num_bytes += line_bytes;
        num_lines += 1;
        num_chars += line.chars().count();
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
        num_chars,
    })
}

fn count_in_str(text: &str) -> FileInfo {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;
    for line in text.split("\n") {
        num_bytes += line.bytes().len() + 1; // +1 for the extra byte for \n
        num_lines += 1;
        num_chars += line.chars().count() + 1; // +1 for the extra char for \n
        for l in line.split(" ") {
            if l.trim() != "" {
                num_words += 1;
            }
        }
    }
    FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    }
}
