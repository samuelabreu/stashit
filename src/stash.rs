use chrono::prelude::*;
use dirs::home_dir;
use glob::glob;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, DirEntry};
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

extern crate confy;

#[derive(Serialize, Deserialize, Debug)]
pub struct StashConfig {
    pub path: String,
}

impl Default for StashConfig {
    fn default() -> Self {
        Self {
            path: "~/.local/share/stashit/".to_string(),
        }
    }
}

pub struct StashIt {
    pub path: PathBuf,
}

#[derive(Clone)]
pub struct Stash {
    pub stash_dir_name: i64,
    pub files: Vec<String>,
}

impl Default for StashIt {
    fn default() -> Self {
        Self {
            path: PathBuf::from(StashConfig::default().path),
        }
    }
}

impl StashIt {
    pub fn from_config() -> Self {
        let cfg: Result<StashConfig, confy::ConfyError> = confy::load("stashit");
        let stash_config = match cfg {
            Ok(stash_config) => stash_config,
            Err(_) => StashConfig::default(),
        };
        let path = stash_config.path;
        let mut path_buf = PathBuf::from(path);
        if path_buf.starts_with("~") {
            let mut home = home_dir().unwrap_or(PathBuf::from("/tmp/"));
            home.push(path_buf.strip_prefix("~").unwrap());
            path_buf = home;
        }
        debug!("Stashes path: {:#?}", path_buf);
        Self { path: path_buf }
    }

    pub fn list(&self, indexes: Vec<&str>) -> Vec<Stash> {
        let stash_path_vec = match self.get_stash_list() {
            Some(v) => v,
            None => vec![],
        };
        let mut result: Vec<Stash> = Vec::default();
        if stash_path_vec.is_empty() {
            debug!("No stashes found");
            return result;
        }
        let mut pos = 0;
        for path in stash_path_vec.iter() {
            let timestamp = path.path();
            let timestamp = match timestamp.strip_prefix(&self.path) {
                Ok(v) => v,
                Err(e) => {
                    debug!(
                        "Error retrieving stashes (trying to strip {} from {}): {}",
                        self.path.display(),
                        timestamp.display(),
                        e
                    );
                    continue;
                }
            };
            debug!("indexes is_empty: {}", indexes.is_empty());
            debug!("pos: {}", pos);
            match timestamp.display().to_string().parse::<i64>() {
                Ok(stash_dir_name) => {
                    if indexes.is_empty()
                        || indexes.iter().any(|e| {
                            debug!("e: {}", e);
                            pos.to_string() == *e
                        })
                    {
                        let files = self.file_list_from_path(stash_dir_name);
                        result.push(Stash {
                            stash_dir_name,
                            files,
                        });
                    }
                    pos = pos + 1;
                }
                Err(e) => debug!("Path not a valid timestamp: {}", e),
            };
        }
        result
    }

    fn file_list_from_path(&self, stash_dir_name: i64) -> Vec<String> {
        let mut stash_path = self.path.clone();
        stash_path.push(format!("{}", stash_dir_name));
        let glob_str = format!("{}/**/*", stash_path.display());
        let mut result: Vec<String> = vec![];
        let mut counter = 0;

        // TODO: Remover esse expect
        for entry in glob(&glob_str).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    if path.is_file() {
                        debug!("{}", path.display());
                        let path_str = path.file_name().expect("File doesn't exists");
                        let path_str = path_str.to_str().expect("File doesn't exists");
                        result.push(String::from(path_str));
                        counter = counter + 1;
                        if counter > 3 {
                            break;
                        }
                    }
                }
                Err(e) => debug!("{}", e),
            }
        }
        result
    }

    fn get_stash_list(&self) -> Option<Vec<DirEntry>> {
        let read_dir = fs::read_dir(self.path.clone());
        let mut stash_path_vec: Vec<_> = match read_dir {
            Ok(v) => v.map(|r| r.unwrap()).collect(),
            Err(e) => {
                debug!("Error reading dir: {}, message: {}", self.path.display(), e);
                return None;
            }
        };

        if stash_path_vec.is_empty() {
            return None;
        }
        stash_path_vec.retain(|p| {
            p.path()
                .strip_prefix(&self.path)
                .expect("Can't store stashes on /")
                .display()
                .to_string()
                .parse::<i64>()
                .is_ok()
        });
        stash_path_vec.sort_by_key(|dir| dir.path());
        stash_path_vec.reverse();
        Some(stash_path_vec)
    }

    fn get_stash_path_by_index(&self, number: i32) -> Option<Stash> {
        let stashes = self.list(vec![&number.to_string()]);
        stashes.first().cloned()
    }

    pub fn remove(&self, number: i32) -> std::io::Result<()> {
        match self.get_stash_path_by_index(number) {
            Some(stash) => self.remove_stash_path(stash.stash_dir_name.to_string()),
            None => return Err(Error::new(ErrorKind::NotFound, "Stash not found")),
        }
    }

    pub fn pop(&self, number: i32) -> std::io::Result<u32> {
        debug!("Poping number: {}", number);
        match self.get_stash_path_by_index(number) {
            Some(stash) => {
                let count = self.restore(&stash)?;
                self.remove_stash_path(stash.stash_dir_name.to_string())?;
                Ok(count)
            }
            None => return Err(Error::new(ErrorKind::NotFound, "Stash not found")),
        }
    }

    fn restore(&self, stash: &Stash) -> std::io::Result<u32> {
        let mut stash_path = self.path.clone();
        stash_path.push(stash.stash_dir_name.to_string());
        let glob_str = format!("{}/**/*", stash_path.display());
        let mut counter = 0;
        for entry in glob(&glob_str).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    if path.is_file() {
                        let p = path.strip_prefix(&stash_path).expect("Wrong prefix");
                        let path_from_root = format!("/{}", p.display());
                        let parent = PathBuf::from(path_from_root.clone());
                        let parent = parent.parent().unwrap_or(p);
                        if fs::metadata(parent).is_err() {
                            fs::create_dir_all(parent)?;
                        }
                        fs::copy(path, path_from_root)?;
                        counter = counter + 1;
                    }
                }
                Err(e) => debug!("{}", e),
            }
        }
        Ok(counter)
    }

    fn remove_stash_path(&self, stash_dir_name: String) -> std::io::Result<()> {
        let mut stash_path = self.path.clone();
        stash_path.push(stash_dir_name);
        debug!("Removing path: {}", stash_path.display());
        fs::remove_dir_all(stash_path)?;
        Ok(())
    }

    pub fn stash(&self, files: &Vec<String>, keep: bool) -> std::io::Result<usize> {
        let now = Local::now().timestamp();

        let mut out_path = self.path.clone();
        out_path.push(now.to_string());
        fs::create_dir_all(out_path.clone())?;
        for file in files {
            let mut source_file_path = PathBuf::from(file);
            if !source_file_path.exists() {
                self.remove_stash_path(now.to_string())?;
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("{} doesn't exist", file),
                ));
            }
            if source_file_path.is_relative() {
                let mut current_dir = match env::current_dir() {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Error fetching env::current_dir: {}", e);
                        self.remove_stash_path(now.to_string())?;
                        return Err(e);
                    }
                };
                current_dir.push(source_file_path);
                source_file_path = current_dir;
            }
            let root_relative_path = match source_file_path.strip_prefix("/") {
                Ok(v) => v,
                Err(e) => {
                    error!("Should be unreachable error: {}", e);
                    return Err(Error::new(ErrorKind::InvalidInput, format!("{}", e)));
                }
            };
            let mut out_file_path = out_path.clone();
            out_file_path.push(root_relative_path);
            debug!("file: {}", source_file_path.display());
            debug!("out_file_path: {}", out_file_path.display());
            fs::create_dir_all(out_file_path.parent().expect("Should be unreachable error"))?;
            match fs::copy(file, out_file_path) {
                Ok(bytes) => debug!("File {} copied, total bytes: {}", file, bytes),
                Err(e) => {
                    error!("Error copying file: {}, message: {}", file, e);
                    self.remove_stash_path(now.to_string())?;
                    return Err(Error::new(ErrorKind::InvalidInput, format!("{}", e)));
                }
            }
        }
        if !keep {
            for in_file in files {
                fs::remove_file(in_file)?;
                debug!("File {} removed", in_file);
            }
        }

        Ok(files.len())
    }
}
