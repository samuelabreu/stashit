#[macro_use]
extern crate clap;
use ansi_term::Colour::{Green, Red, Yellow};
use chrono::prelude::*;
use clap::{load_yaml, App, ArgMatches, Values};
use itertools::join;
use stashit::stash::StashIt;
use std::env;

extern crate pretty_env_logger;

fn main() {
    init_log();
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    let stash_it = StashIt::from_config();

    if matches.is_present("list") {
        list(&stash_it, matches);
        return;
    }

    if matches.occurrences_of("pop") > 0 {
        pop(&stash_it, matches);
        return;
    }

    if matches.occurrences_of("remove") > 0 {
        remove(&stash_it, matches);
        return;
    }

    stash(&stash_it, matches);
}

fn list(stash_it: &StashIt, matches: ArgMatches) {
    let indexes: Vec<&str> = matches
        .values_of("list")
        .unwrap_or(Values::default())
        .collect();

    let stashes = stash_it.list(indexes);
    for (pos, stash) in stashes.iter().enumerate() {
        let pos_str = Green.bold().paint(format!("[{}]", pos));
        let files_str = join(stash.files.iter(), ", ");
        println!(
            "{}: {} ({})",
            pos_str,
            Local.timestamp(stash.stash_dir_name, 0),
            Yellow.paint(files_str)
        );
    }
}

fn pop(stash_it: &StashIt, matches: ArgMatches) {
    let pop_number = value_t!(matches, "pop", i32).unwrap_or(-1);
    if pop_number < 0 {
        println!("{}", Red.paint("Wrong number for pop"));
    } else {
        match stash_it.pop(pop_number) {
            Ok(count) => println!(
                "{} file(s) restored",
                Green.bold().paint(format!("{}", count))
            ),
            Err(e) => println!("{}", Red.paint(e.to_string())),
        }
    }
}

fn remove(stash_it: &StashIt, matches: ArgMatches) {
    let remove_number = value_t!(matches, "remove", i32).unwrap_or(-1);
    if remove_number < 0 {
        println!("{}", Red.paint("Wrong number for remove"));
    } else {
        match stash_it.remove(remove_number) {
            Ok(()) => println!(
                "stash number {} removed!",
                Red.bold().paint(format!("{}", remove_number))
            ),
            Err(e) => println!("{}", Red.paint(e.to_string())),
        }
    }
}

fn stash(stash_it: &StashIt, matches: ArgMatches) {
    let keep = matches.is_present("keep");
    if let Some(in_v) = matches.values_of("input") {
        let files: Vec<String> = in_v.map(|v| String::from(v)).collect();
        match stash_it.stash(&files, keep) {
            Ok(count) => println!(
                "{} file(s) stashed!",
                Green.bold().paint(format!("{}", count))
            ),
            Err(e) => println!("{}", Red.paint(e.to_string())),
        }
    }
}

fn init_log() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
}
