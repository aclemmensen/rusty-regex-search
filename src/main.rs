extern crate flate2;
extern crate stopwatch;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate clap;

use std::io::{self, BufReader, BufRead};
use std::boxed::Box;
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::vec::Vec;
use stopwatch::Stopwatch;
use regex::Regex;
use clap::{Arg, App};

#[derive(Debug)]
enum ReaderType {
    Chunk,
    Delta,
    Delete
}

struct SearchReader<'a> {
    reader: Box<BufRead>,
    reader_type: ReaderType,
    path: &'a Path
}

impl<'a> SearchReader<'a> {
    fn new(reader: Box<BufRead>, path: &Path, reader_type: ReaderType) -> SearchReader {
        SearchReader {
            reader: reader,
            reader_type: reader_type,
            path: path
        }
    }
}

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
fn build_readers(paths: &Vec<PathBuf>) -> Vec<SearchReader> {
    let mut readers: Vec<SearchReader> = Vec::new();
    for ref path in paths {
        println!("{:?}", path);
        match path.extension() {
            Some(ext) if ext == "gz" => readers.push(SearchReader::new(gz_reader(&path).unwrap(), path, ReaderType::Chunk)),
            Some(ext) if ext == "html" => readers.push(SearchReader::new(raw_reader(&path).unwrap(), path, ReaderType::Chunk)),
            Some(ext) if ext == "html-changed" => readers.push(SearchReader::new(raw_reader(&path).unwrap(), path, ReaderType::Delta)),
            Some(ext) if ext == "deleted" => readers.push(SearchReader::new(raw_reader(&path).unwrap(), path, ReaderType::Delete)),
            _ => ()
        }
    }
    readers
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

/// Clean up escaped input
fn cleanup2(line: &str) -> String {
    lazy_static! {
        static ref REGEX: Regex = Regex::new("\\n").unwrap();
    }

    let replaced = REGEX.replace_all(line, "\n").into_owned();
    replaced
}

/// Do the searching - this is fake right now
fn search(reader: SearchReader) -> io::Result<()> {
    let mut bytes: usize = 0;
    let mut lines: u32 = 0;
    let sw = Stopwatch::start_new();
    println!("{:?} -> {:?}", reader.path, reader.reader_type);

    for line in reader.reader.lines() {
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

/// Search the list of files
fn searchfiles(paths: &Vec<PathBuf>) -> io::Result<()> {
    for reader in build_readers(paths) {
        search(reader)?;
    }
    Ok(())
}


/// Run the search in a directory
fn run(dir: &Path) -> io::Result<()> {
    let paths = scandir(dir, None)?;
    println!("Scanning {} files from {:?}", paths.len(), dir);
    searchfiles(&paths)?;
    Ok(())
}

fn main() {
    let matches = App::new("regex-search")
        .version("1.0")
        .about("Search text content fastly")
        .author("aclemmensen")
        .arg(Arg::with_name("path")
            .short("p")
            .long("path")
            .help("Path to folder containing HTML to search")
            .takes_value(true)
            .default_value("C://temp//sites//cmp")
            .required(true))
        .get_matches();
    
    let path = matches.value_of("path").unwrap();
    run(&Path::new(path)).unwrap();
}

