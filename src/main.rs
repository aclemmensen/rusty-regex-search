extern crate flate2;
extern crate stopwatch;
#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::io::{self, BufReader, BufRead};
use std::boxed::Box;
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::vec::Vec;
use stopwatch::Stopwatch;
use regex::Regex;

/// Scan the provided directory for files that have extensions that match
/// the provided pattern, or any files if no pattern is provided.
fn scandir(path: &Path, expected_ext: Option<&str>) -> io::Result<Vec<PathBuf>> {
    let dir = Path::new(path);
    let mut results: Vec<PathBuf> = Vec::new();

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let filepathbuf = entry.path();
            match expected_ext {
                Some(ext) => {
                    let filepath = filepathbuf.as_path();
                    let filename = filepath.file_name().unwrap().to_str().unwrap();
                    if filename.ends_with(ext) {
                        results.push(entry.path());
                    }
                },
                None => results.push(filepathbuf)
            }
        }
    }

    Ok(results)
}

/// Build a list of readers that are compatible with the file extensions discovered
fn build_readers(paths: &Vec<PathBuf>) -> Vec<Box<BufRead>> {
    let mut readers: Vec<Box<BufRead>> = Vec::new();
    for ref path in paths {
        println!("{:?}", path);
        match path.extension() {
            Some(ext) if ext == "gz" => readers.push(gz_reader(&path).unwrap()),
            Some(ext) if ext == "html" => readers.push(raw_reader(&path).unwrap()),
            _ => ()
        }
    }
    readers
}

/// Search the list of files
fn searchfiles(paths: &Vec<PathBuf>) -> io::Result<()> {
    for reader in build_readers(paths) {
        search(reader)?;
    }
    Ok(())
}

/// Parse the line into an (id, content) tuple
fn parseline(line: &String) -> Option<(i64, String)> {
    match line.find(':') {
        Some(pos) => {
            let (id, content) = line.split_at(pos);
            let id = id.parse::<i64>();
            match id {
                Ok(id) => {
                    let cleaned = cleanup2(&content);
                    Some((id, cleaned))
                },
                Err(_) => None
            }
        },
        None => None
    }
}

/// Handle zipped files
fn gz_reader(path: &PathBuf) -> io::Result<Box<BufRead>> {
    let file = File::open(path)?;
    let raw_reader = BufReader::new(file);
    let decoder = GzDecoder::new(raw_reader)?;
    let unzipped_reader = BufReader::new(decoder);
    Ok(Box::new(unzipped_reader))
}

/// Handle raw files
fn raw_reader(path: &PathBuf) -> io::Result<Box<BufRead>> {
    let file = File::open(path)?;
    let raw_reader = BufReader::new(file);
    Ok(Box::new(raw_reader))
}

// ###################################
// Cleanup methods below
// ###################################
#[allow(dead_code)]
fn cleanup3(line: &str) -> String {
    line.replace("\\n", "\n")
}

#[allow(dead_code)]
fn cleanup2(line: &str) -> String {
    lazy_static! {
        static ref REGEX: Regex = Regex::new("\\n").unwrap();
    }

    let replaced = REGEX.replace_all(line, "\n").into_owned();
    replaced
}

#[allow(dead_code)]
fn cleanup4(line: &str) -> String {
    lazy_static! {
        static ref REGEX: Regex = Regex::new("\\n").unwrap();
    }

    let mut last_match = 0;
    let mut begin: usize;
    let mut end: usize;
    let matches = REGEX.find_iter(line);
    let mut result = String::with_capacity(line.len());

    for m in matches {
        begin = m.start();
        end = m.end();
        result.push_str(&line[last_match..begin]);
        result.push('\n');
        last_match = end;
    }

    result.push_str(&line[last_match..]);

    result
}

#[allow(dead_code)]
fn cleanup1(line: &str) -> String {
    let mut buf = String::with_capacity(line.len());
    let mut nl = false;

    for c in line.chars() {
        if c == '\\' {
            nl = true;
            continue;
        }

        if nl {
            if c == 'n' {
                buf.push('\n');
                continue;
            }
            else {
                buf.push('\\');
            }

            nl = false;
        }

        buf.push(c);
    }

    buf
}

/// Do the searching - this is fake right now
fn search(reader: Box<BufRead>) -> io::Result<()> {
    let mut bytes: usize = 0;
    let mut lines: u32 = 0;
    let sw = Stopwatch::start_new();

    for line in reader.lines() {
        let line = line?;
        bytes += line.len();
        lines += 1;
        match parseline(&line) {
            Some((_id, _content)) => /*println!("{} -> {}", id, content.get(0..120).unwrap_or(&content))*/ (),
            _ => ()
        }
    }

    let elapsed = sw.elapsed_ms();
    let mb_sec = ((bytes as f64)/(elapsed as f64)/1024.0/1024.0)*1000.0;
    println!("Read {} bytes on {} lines in {} = {} mb/sec", bytes, lines, elapsed, mb_sec);

    Ok(())
}

/// Run the search in a directory
fn run(dir: &Path) -> io::Result<()> {
    let paths = scandir(dir, None)?;
    searchfiles(&paths)?;
    Ok(())
}

fn main() {
    println!("Hello, world!");
    run(&Path::new("C://temp//sites//cmp")).unwrap();
}

