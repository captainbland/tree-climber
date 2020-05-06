#![feature(try_trait)]

extern crate regex;
extern crate unicode_segmentation;
extern crate vec_tree;
#[macro_use]
extern crate clap;

use core::fmt;
use regex::Regex;
use std::fmt::{Error, Formatter};
use std::{io, io::Read};
use unicode_segmentation::UnicodeSegmentation;
use vec_tree::*;
#[macro_use]
use clap::*;

mod climb;

use crate::climb::tree_tools::{
    get_ancestors_match_re, get_branch_match_re, get_descendants_match_re,
};
use climb::{print_tree, process_tree_cli};

fn main() {
    let ancestors_arg = "ancestors";
    let descendants_arg = "descendants";
    let branch_arg = "branch";
    let expression_arg = "expression";
    let verbose_arg = "verbose";

    let matches = App::new("Climb")
        .version("1.0")
        .about("Climbs trees.\n Example: climb -b 'test'")
        .arg(
            Arg::with_name(ancestors_arg)
                .short("a")
                .long("ancestors")
                .help("returns ancestors of the matching node")
                .takes_value(false),
        )
        .arg(
            Arg::with_name(descendants_arg)
                .short("d")
                .long("descendants")
                .help("returns descendants of the matching node"),
        )
        .arg(
            Arg::with_name(branch_arg)
                .short("b")
                .long("branch")
                .help("returns all ancestors and descendants of matching node"),
        )
        .arg(
            Arg::with_name(verbose_arg)
                .short("v")
                .long("verbose")
                .help("verbose output"),
        )
        .arg(
            Arg::with_name("expression")
                .short("e")
                .long("expression")
                .index(1),
        )
        .get_matches();

    let verbose_mode = matches.is_present(verbose_arg);
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    let tree = match process_tree_cli(buffer, verbose_mode) {
        Ok(s) => s.clone(),
        Err(s) => panic!("There was an error: {}", s),
    };

    let expression = matches
        .value_of(expression_arg)
        .expect("An expression is expected");
    let regex = Regex::new(expression).expect("Expression should be a valid regex");

    if matches.is_present(ancestors_arg) {
        print_tree(get_ancestors_match_re(&tree, regex).unwrap());
    } else if matches.is_present(descendants_arg) {
        print_tree(get_descendants_match_re(&tree, regex));
    } else if matches.is_present(branch_arg) {
        print_tree(get_branch_match_re(&tree, regex));
    } else {
        // default to branch behaviour
        print_tree(get_branch_match_re(&tree, regex));
    }
}
