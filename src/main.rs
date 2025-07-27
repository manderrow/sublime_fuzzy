//! Very basic binary for using/testing `sublime_fuzzy` from the command line.
//!
//! Pass the query as first and target string as second parameters to `sfz`.
use sublime_fuzzy::{best_match, format_simple};

fn main() {
    let mut args = std::env::args();
    _ = args.next().unwrap();

    let q = args.next().expect("Missing query arg");
    let s = args.next().expect("Missing target arg");

    if let Some(m) = best_match(&q, &s) {
        println!("{}", format_simple(&m, &s, "<", ">"));
    } else {
        println!("No match");
    }
}
