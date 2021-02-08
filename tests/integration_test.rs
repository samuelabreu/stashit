use std::io::prelude::*;
use std::{fs::File, path::PathBuf};

use chrono::Local;
use stashit::{StashConfig, StashIt};

#[test]
fn test_stash_config_default_path() {
    let stash_config = StashConfig::default();
    assert_eq!(stash_config.path, "~/.local/share/stashit/");
}

#[test]
fn test_list_empty_stash() {
    let mut stash_it = StashIt::default();
    stash_it.path = PathBuf::from("/tmp/integrationtest0/stashit/");
    let result = stash_it.list(vec![]);
    assert_eq!(result.len(), 0);
}

#[test]
fn test_list() -> std::io::Result<()> {
    let _ = std::fs::remove_dir_all("/tmp/integrationtest1/");
    let mut stash_it = StashIt::default();
    stash_it.path = PathBuf::from("/tmp/integrationtest1/stashit/");

    let now = Local::now().timestamp();

    let base_path = format!("/tmp/integrationtest1/stashit/{}/tmp/integrationtest/", now);
    let file_path = format!("{}/foo.txt", base_path);
    std::fs::create_dir_all(base_path)?;
    let mut file = File::create(file_path)?;
    file.write_all(b"integrationtest1!")?;

    let result = stash_it.list(vec![]);
    assert_eq!(result.len(), 1);
    let item = result.get(0).expect("Missing stash from integration_test");
    assert_eq!(item.stash_dir_name, now);
    assert_eq!(item.files.len(), 1);
    assert_eq!(item.files.get(0).expect("No file stashed"), "foo.txt");

    Ok(())
}

#[test]
fn test_stash_file_no_keep() -> std::io::Result<()> {
    let _ = std::fs::remove_dir_all("/tmp/integrationtest2/");
    let mut stash_it = StashIt::default();
    stash_it.path = PathBuf::from("/tmp/integrationtest2/stashit/");
    let now = Local::now().timestamp();

    let base_path = format!("/tmp/integrationtest2/stashit/{}/tmp/integrationtest/", now);
    let file_path = format!("{}/foo.txt", base_path);
    std::fs::create_dir_all(base_path)?;
    let mut file = File::create(file_path.clone())?;
    file.write_all(b"integrationtest1!")?;

    let files: Vec<String> = vec![file_path];
    let result = stash_it
        .stash(&files, false)
        .expect("could not stash files");
    assert_eq!(result, 1);

    let result = stash_it.list(vec![]);
    assert_eq!(result.len(), 1);
    let item = result.get(0).expect("Missing stash from integration_test");
    assert_eq!(item.stash_dir_name, now);
    assert_eq!(item.files.len(), 1);
    assert_eq!(item.files.get(0).expect("No file stashed"), "foo.txt");

    Ok(())
}

#[test]
fn test_remove() -> std::io::Result<()> {
    let _ = std::fs::remove_dir_all("/tmp/integrationtest3/");
    let mut stash_it = StashIt::default();
    stash_it.path = PathBuf::from("/tmp/integrationtest3/stashit/");

    let now = Local::now().timestamp();

    let base_path = format!("/tmp/integrationtest3/stashit/{}/tmp/integrationtest/", now);
    let file_path = format!("{}/foo.txt", base_path);
    std::fs::create_dir_all(base_path)?;
    let mut file = File::create(file_path)?;
    file.write_all(b"integrationtest1!")?;

    stash_it.remove(0)?;

    let result = stash_it.list(vec![]);
    assert_eq!(result.len(), 0);

    Ok(())
}

#[test]
fn test_pop() -> std::io::Result<()> {
    let _ = std::fs::remove_dir_all("/tmp/integrationtest4/");
    let _ = std::fs::remove_dir_all("/tmp/integrationtest/");
    let mut stash_it = StashIt::default();
    stash_it.path = PathBuf::from("/tmp/integrationtest4/stashit/");

    let now = Local::now().timestamp();

    let base_path = format!("/tmp/integrationtest4/stashit/{}/tmp/integrationtest/", now);
    let file_path = format!("{}/foo.txt", base_path);
    std::fs::create_dir_all(base_path)?;
    let mut file = File::create(file_path)?;
    file.write_all(b"integrationtest1!")?;

    let pop = stash_it.pop(0).expect("could not pop last stash");
    assert_eq!(pop, 1);

    let result = stash_it.list(vec![]);
    assert_eq!(result.len(), 0);

    assert!(std::fs::metadata("/tmp/integrationtest/foo.txt").is_ok());

    Ok(())
}
