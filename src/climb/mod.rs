use core::fmt;
use regex::Regex;
use std::fmt::{Error, Formatter};
use std::{io, io::Read};
use unicode_segmentation::UnicodeSegmentation;
use vec_tree::*;

pub mod processing_error;
pub mod tree_tools;

use tree_tools::get_ancestors_match_re;

macro_rules! wrap_str_err {
    ($st: expr) => {
        $st.map_err(|e| format!("{}", e))?
    };
}

macro_rules! v_println {
    ($verbose:expr, $($e:tt)*) => {
        if ($verbose) {
            println!($($e)*)
        }
    }
 }

struct Cursor {
    row: usize,
    col: usize,
}

impl Cursor {
    fn new(row: usize, col: usize) -> Cursor {
        Cursor { row, col }
    }
}

fn text_at_cursor(cursor: &Cursor, divided_text: &Vec<&str>) -> String {
    let row = cursor.row;
    let col = cursor.col;
    let graphemes = UnicodeSegmentation::graphemes(divided_text[row], true);
    //this bit profiles poorly. Fixme?
    let graphemes_vec = graphemes.collect::<Vec<&str>>();
    if divided_text.len() > row && graphemes_vec.len() > col {
        let selected_text = graphemes_vec[col..].join("");
        return selected_text;
    }
    return String::from("");
}

pub fn process_tree_cli(input: String, verbose: bool) -> Result<VecTree<String>, String> {
    let split: Vec<&str> = input.split("\n").collect();
    let leaf_re = wrap_str_err!(Regex::new(r"^├──\s(.*)"));
    let pipe_re = wrap_str_err!(Regex::new(r"^│"));
    let elbow_re = wrap_str_err!(Regex::new(r"^└──\s(.*)"));
    let mut stack: Vec<(Cursor, Index)> = Vec::new();
    let mut tree = VecTree::new();
    let root = tree.insert_root(".".to_string());
    let mut last_visisted_index = root;
    //println!("Root node: {:?}", parent);

    stack.push((Cursor::new(1, 0), root));
    let mut iter = 0;
    while !stack.is_empty() {
        v_println!(verbose, "\nIter: {}", iter);
        iter += 1;
        let (cursor, parent) = wrap_str_err!(stack.pop().ok_or("magic!"));

        v_println!(
            verbose,
            "Cursor at: {} {}; parent: {}",
            cursor.row,
            cursor.col,
            tree.get(parent).unwrap()
        );

        let text = text_at_cursor(&cursor, &split);
        // doing is_match and then captures_iter probably means doing more regexing than necessary
        if pipe_re.is_match(&text) {
            v_println!(verbose, "Pipe...");
            stack.push((Cursor::new(cursor.row + 1, cursor.col), parent));
        } else if let Some(leaf_data) = leaf_re.captures(&text) {
            let mut node_text: String = String::from("UNINITIALISED");
            v_println!(verbose, "Got leaf: {}", leaf_data[0].to_string());
            node_text = leaf_data[1].to_string();
            let me = tree.insert(node_text, parent);
            v_println!(
                verbose,
                "Adding {} as child to {}",
                tree.get(me).unwrap(),
                tree.get(parent).unwrap()
            );
            stack.push((Cursor::new(cursor.row + 1, cursor.col), parent));
            stack.push((Cursor::new(cursor.row + 1, cursor.col + 4), me));
        } else if let Some(elbow_data) = elbow_re.captures(&text) {
            v_println!(verbose, "Elbow...");
            let mut node_text = String::from("UNINITIALISED");
            v_println!(verbose, "Got elbow: {}", elbow_data[0].to_string());
            node_text = String::from(elbow_data[1].to_string());
            let me = tree.insert(node_text, parent);
            stack.push((Cursor::new(cursor.row + 1, cursor.col + 4), me));
            v_println!(
                verbose,
                "Adding {} as child to {}",
                tree.get(me).unwrap(),
                tree.get(parent).unwrap()
            );
        } else {
            v_println!(verbose, "No regexable text found here");
        }
    }
    return Ok(tree);
}

pub fn print_tree(tree: VecTree<String>) {
    let mut stack = Vec::<(Index, usize)>::new();
    let node = tree.get_root_index().expect("No root node in tree");
    stack.push((node, 0));

    while !stack.is_empty() {
        let (node, depth) = stack.pop().expect("pop after check fail");
        let val = tree.get(node).expect("plz");

        for _i in 0..depth {
            print!(" ");
        }

        print!("{}\n", val);

        let mut child_vec = Vec::<Index>::new();
        child_vec.extend(tree.children(node));

        for i in (0..child_vec.len()).rev() {
            stack.push((child_vec[i], depth + 2));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::climb::tree_tools::{
        get_ancestors_match_re, get_branch_match_re, get_descendants_match_re,
    };
    use crate::climb::{print_tree, process_tree_cli};
    use regex::Regex;
    use std::fs::File;
    use std::io::{BufReader, Read};

    #[test]
    fn test_process_cli_tree() {
        let test_data = ".
├── data
│   ├── comments.rs
│   ├── listing.rs
│   ├── mod.rs
│   ├── post.rs
│   ├── sub.rs
│   ├── thing.rs
│   └── user.rs
├── errors.rs
├── mod.rs
├── net
│   ├── auth.rs
│   └── mod.rs
└── tests.rs
           "
        .to_string();

        println!("Test data: {}", test_data);
        let result = process_tree_cli(test_data, true);
        print_tree(result.clone().unwrap());
        assert!(result.is_ok());
    }

    pub fn big_test_data() -> String {
        let file = File::open("dtree.txt").expect("Test file data");
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader
            .read_to_string(&mut contents)
            .expect("buf reader for test file data read");
        contents
    }

    pub fn test_data() -> String {
        let test_data = ".
├── data
│   ├── comments.rs
│   ├── listing.rs
│   ├── mod.rs
│   ├── mod.rs
│   ├── post.rs
│   ├── sub.rs
│   ├── thing.rs
│   └── user.rs
├── errors.rs
├── mod.rs
├── net
│   ├── auth.rs
│   └── mod.rs
└── tests.rs
           "
        .to_string();
        return test_data;
    }

    #[test]
    pub fn test_get_ancestors_leaf_cli_tree() {
        let result = process_tree_cli(test_data(), true);
        let tree = result.clone().unwrap();
        let my_regex = Regex::new(r"mod\.rs").unwrap();
        let ancestor_tree = get_ancestors_match_re(&tree, my_regex);
        println!("Parsed tree: ");
        print_tree(result.unwrap());
        println!("Ancestor tree: ");
        print_tree(ancestor_tree.unwrap());
    }

    #[test]
    pub fn test_get_ancestors_mid_cli_tree() {
        let result = process_tree_cli(test_data(), true);
        let tree = result.clone().unwrap();
        let my_regex = Regex::new(r"net").unwrap();
        let ancestor_tree = get_ancestors_match_re(&tree, my_regex);
        println!("Parsed tree: ");
        print_tree(result.unwrap());
        println!("Ancestor tree: ");
        print_tree(ancestor_tree.unwrap());
    }

    #[test]
    pub fn test_get_descendants_mid_cli() {
        let result = process_tree_cli(test_data(), true);
        let tree = result.clone().unwrap();
        let my_regex = Regex::new(r"net").unwrap();
        let ancestor_tree = get_descendants_match_re(&tree, my_regex);
        println!("Parsed tree: ");
        print_tree(result.unwrap());
        println!("Descendant tree: ");
        print_tree(ancestor_tree);
    }

    #[test]
    pub fn test_get_descendants_root_cli() {
        let result = process_tree_cli(test_data(), true);
        let tree = result.clone().unwrap();
        let my_regex = Regex::new(r"\.").unwrap();
        let ancestor_tree = get_descendants_match_re(&tree, my_regex);
        println!("Parsed tree: ");
        print_tree(result.unwrap());
        println!("Descendant tree: ");
        print_tree(ancestor_tree);
    }

    #[test]
    pub fn test_descendants_join_branch_cli() {
        let result = process_tree_cli(test_data(), true);
        let tree = result.clone().unwrap();
        let my_regex = Regex::new(r"mod\.rs").unwrap();
        let branch_tree = get_branch_match_re(&tree, my_regex);
        println!("Parsed tree: ");
        print_tree(result.unwrap());
        println!("Branch tree: ");
        print_tree(branch_tree);
    }
    //
    #[test]
    pub fn big_file_join_branch() {
        let result = process_tree_cli(big_test_data(), true);
        let tree = result.clone().unwrap();
        let my_regex = Regex::new(r"board-2.bin").unwrap();
        let branch_tree = get_branch_match_re(&tree, my_regex);
        println!("Parsed tree: ");
        print_tree(result.unwrap());
        println!("Branch tree: ");
        print_tree(branch_tree);
    }

}
