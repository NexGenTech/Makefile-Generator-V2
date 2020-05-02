use crate::config::Config;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

pub type DependencyMap = HashMap<String, Vec<String>>;

// These are some default mappings for dynamic linked libraries
lazy_static! {
    static ref DLL_MAP: HashMap<&'static str, &'static str> = {
        let mut dll_map = HashMap::new();
        dll_map.insert("math.h", "m");
        dll_map.insert("pthread.h", "pthread");
        dll_map.insert("ncurses.h", "ncurses");
        dll_map
    };
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn has_extension<P: AsRef<Path>>(path: P, ext: &str) -> bool {
    path.as_ref()
        .extension()
        .map(|e| e.to_str().unwrap_or("") == ext)
        .unwrap_or(false)
}

pub struct ParseResult {
    pub dependency_map: DependencyMap,
    pub dlls: Vec<String>,
}

impl ParseResult {
    pub fn new(dependency_map: DependencyMap, dlls: Vec<String>) -> Self {
        Self {
            dependency_map,
            dlls,
        }
    }
}

pub struct Parser<'conf> {
    root_dir: PathBuf,
    config: &'conf Config<'conf>,
}

impl<'conf> Parser<'conf> {
    pub fn new(root_dir: PathBuf, config: &'conf Config<'conf>) -> Self {
        Self { root_dir, config }
    }

    pub fn parse(&self) -> Result<ParseResult, Box<dyn Error>> {
        let mut map = HashMap::new();
        let mut dlls = Vec::new();

        let walker = WalkDir::new(&self.root_dir).into_iter();
        for entry in walker
            .filter_entry(|e| !is_hidden(e))
            .filter(|e| e.as_ref().map(|e| e.file_type().is_file()).unwrap_or(false))
            .filter(|e| {
                e.as_ref()
                    .map(|e| has_extension(e.path(), self.config.extension))
                    .unwrap_or(false)
            })
        {
            let mut seen = HashSet::new();
            if let Ok(entry) = entry {
                let filename = entry.path().strip_prefix(&self.root_dir)?;
                read_file_and_get_include_files_recursively(
                    &self.root_dir,
                    filename,
                    &mut map,
                    &mut dlls,
                    &mut seen,
                )?;
            }
        }

        Ok(ParseResult::new(map, dlls))
    }
}

fn extract_included_filename_and_update_dlls<'line>(
    line: &'line str,
    dlls: &mut Vec<String>,
) -> Option<&'line str> {
    let mut start_index = line.find("<");
    let mut end_index = if start_index.is_some() {
        line.find(">")
    } else {
        None
    };
    let mut is_system_file = true;
    if start_index.is_none() || end_index.is_none() {
        start_index = line.find("\"");
        end_index = if start_index.is_some() {
            line[(start_index.unwrap() + 1)..].find("\"")
        } else {
            None
        };
        if end_index.is_some() {
            end_index = Some(start_index.unwrap() + end_index.unwrap() + 1);
        }
        is_system_file = false;
    }
    let res = if start_index.is_some() && end_index.is_some() {
        let start_index = start_index.unwrap() + 1;
        let end_index = end_index.unwrap();
        Some(&line[start_index..end_index])
    } else {
        None
    };
    if !is_system_file {
        res
    } else {
        let res = res.unwrap();
        if DLL_MAP.contains_key(res) {
            let linkage_name = DLL_MAP.get(res).unwrap().to_string();
            if !dlls.contains(&linkage_name) {
                dlls.push(linkage_name);
            }
        }
        None
    }
}

fn get_include_files_and_update_dlls(source: String, dlls: &mut Vec<String>) -> Vec<String> {
    let mut include_files = Vec::new();
    for line in source.lines() {
        if line.starts_with("#include") {
            if let Some(include_file) = extract_included_filename_and_update_dlls(line, dlls) {
                include_files.push(include_file.to_string());
            }
        }
    }
    include_files
}

fn read_file_and_get_include_files_recursively(
    root_dir: &PathBuf,
    filename: &Path,
    map: &mut DependencyMap,
    dlls: &mut Vec<String>,
    seen: &mut HashSet<String>,
) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(filename)?;
    let mut include_files = get_include_files_and_update_dlls(contents, dlls);
    for include_file in &mut include_files {
        let mut full_path = root_dir.to_path_buf();
        full_path.push(filename);
        full_path.pop();
        full_path.push(&include_file);
        full_path = full_path.canonicalize()?;
        *include_file = full_path
            .strip_prefix(root_dir)?
            .to_str()
            .unwrap()
            .to_string();
        if !map.contains_key(include_file) && !seen.contains(include_file){
            seen.insert(include_file.to_string());
            read_file_and_get_include_files_recursively(
                root_dir,
                Path::new(include_file),
                map,
                dlls,
                seen,
            )?;
        }
    }
    let filename = filename.to_str().unwrap();
    if !map.contains_key(filename) {
        map.insert(filename.to_string(), include_files);
    }
    Ok(())
}
