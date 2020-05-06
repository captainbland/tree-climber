use crate::climb::print_tree;
use crate::climb::processing_error::ProcessingError;
use regex::Regex;
use std::collections::{BTreeSet, HashSet};
use std::{error, io};
use vec_tree::*;

macro_rules! wrap_str_err {
    ($st: expr) => {
        $st.map_err(|e| format!("{}", e))?
    };
}

/*
  Gets all ancestors for all nodes which happen to match the given regular expression
  Combine all of these ancestors into a single output tree with all duplications explicitly removed
*/
pub fn get_ancestors_match_re(
    tree: &VecTree<String>,
    regex: Regex,
) -> Result<VecTree<String>, ProcessingError> {
    let root = tree.get_root_index()?;
    let matching_descendants: Vec<Index> = tree
        .descendants(root)
        .filter(|i| tree.get(*i).map(|t| regex.is_match(t)).unwrap_or(false))
        .collect();

    let mut out_tree = VecTree::<String>::new();
    out_tree.insert_root(".".to_string());

    for matching_descendant in matching_descendants {
        accumulate_ancestors(tree, &mut out_tree, matching_descendant);
    }
    return Ok(out_tree);
}

fn accumulate_ancestors(
    tree: &VecTree<String>,
    out_tree: &mut VecTree<String>,
    matching_descendant: Index,
) -> Result<(), ProcessingError> {
    let mut this_list = Vec::<Index>::new();
    for descendant_ancestor in tree.ancestors(matching_descendant) {
        this_list.push(descendant_ancestor);
    }
    let mut reverse_list = Vec::<Index>::from(this_list);
    reverse_list.pop();
    reverse_list.reverse();
    let mut parent = out_tree.get_root_index()?;
    for descendant_ancestor in reverse_list {
        let descendent_ancestor_data = tree.get(descendant_ancestor)?.to_owned();

        //duplication check in output tree - if the node already exists, set the parent to be that child and skip adding the node in the current iteration
        let mut found_matching_child = false;
        for out_child in out_tree.children(parent) {
            if out_tree.get(out_child)?.clone() == descendent_ancestor_data {
                parent = out_child;
                found_matching_child = true;
            }
        }
        if !found_matching_child {
            parent = out_tree.insert(descendent_ancestor_data, parent);
        }
    }
    Ok(())
}

/*
  Gets all descendants for all nodes which match the regular expression
  Combines them all into a single vectree
*/
pub fn get_descendants_match_re(tree: &VecTree<String>, regex: Regex) -> VecTree<String> {
    let root = tree.get_root_index().unwrap();
    let matching_descendants: Vec<Index> = tree
        .descendants(root)
        .filter(|i| regex.is_match(tree.get(*i).unwrap()))
        .collect();

    let mut out_tree = VecTree::<String>::new();
    let root_node = out_tree.insert_root(".".to_string());

    for matching_descendant in matching_descendants {
        accumulate_descendants(tree, &mut out_tree, root_node, matching_descendant);
    }

    return out_tree;
}

fn accumulate_descendants(
    tree: &VecTree<String>,
    out_tree: &mut VecTree<String>,
    root_node: Index,
    matching_descendant: Index,
) -> Result<(), ProcessingError> {
    let mut stack = Vec::<(Index, Index)>::new();
    let in_parent = matching_descendant;
    let matching_descendant_str = tree.get(matching_descendant)?.clone();
    let out_parent = out_tree.insert(matching_descendant_str, root_node);
    stack.push((in_parent, out_parent));
    while !stack.is_empty() {
        let (in_parent, out_parent) = stack.pop()?;
        for child in tree.children(in_parent) {
            let child_data = tree.get(child)?.clone();
            let out_child = out_tree.insert(child_data, out_parent);
            stack.push((child, out_child));
        }
    }

    Ok(())
}

pub fn get_branch_match_re(tree: &VecTree<String>, regex: Regex) -> VecTree<String> {
    let root = tree.get_root_index().unwrap();
    let matching_descendants: Vec<Index> = tree
        .descendants(root)
        .filter(|i| regex.is_match(tree.get(*i).unwrap()))
        .collect();

    let mut out_tree = VecTree::<String>::new();
    let root_node = out_tree.insert_root(".".to_string());

    for matching_descendant in matching_descendants {
        let mut descendants_tree = VecTree::<String>::new();
        let mut ancestors_tree = VecTree::<String>::new();
        let descendants_root = descendants_tree.insert_root(".".to_string());
        ancestors_tree.insert_root(".".to_string());
        accumulate_descendants(
            tree,
            &mut descendants_tree,
            descendants_root,
            matching_descendant,
        );
        accumulate_ancestors(tree, &mut ancestors_tree, matching_descendant);
        let new_subtree = join_branch(&ancestors_tree, &descendants_tree);
        let out_root_idx = out_tree.get_root_index().unwrap();
        insert_tree(&mut out_tree, &new_subtree, out_root_idx);
    }
    return out_tree;
}

pub fn join_branch(
    ancestors_tree: &VecTree<String>,
    descendants_tree: &VecTree<String>,
) -> VecTree<String> {
    let mut branch_tree = ancestors_tree.clone();

    //find the last ancestor of the ancestors tree...
    let last_ancestor = branch_tree
        .descendants(ancestors_tree.get_root_index().unwrap())
        .last()
        .unwrap();

    //then insert the descendants tree as children at that point.
    let mut stack = Vec::<(Index, Index)>::new();
    stack.push((descendants_tree.get_root_index().unwrap(), last_ancestor));

    while !stack.is_empty() {
        let (descendants_parent, branch_parent) = stack.pop().unwrap();
        for descendant_child in descendants_tree.children(descendants_parent) {
            let branch_child = branch_tree.insert(
                descendants_tree.get(descendant_child).unwrap().clone(),
                branch_parent,
            );
            stack.push((descendant_child, branch_child));
        }
    }

    return branch_tree;
}

pub fn insert_tree(main_tree: &mut VecTree<String>, insert_tree: &VecTree<String>, from_main: Index) -> Result<(), ProcessingError> {
    let mut stack = Vec::<(Index, Index)>::new();
    stack.push((from_main, insert_tree.get_root_index()?));

    while !stack.is_empty() {
        let (main_idx, insert_idx) = stack.pop()?;
        for insert_child in insert_tree.children(insert_idx) {
            let mut next_main_child = main_idx;
            let mut found_matching_child = false;

            for main_child in main_tree.children(main_idx) {
                if insert_tree.get(insert_child)? == main_tree.get(main_child)? {
                    found_matching_child = true;
                    next_main_child = main_child;
                }
            }

            if !found_matching_child {
                next_main_child = main_tree.insert(insert_tree.get(insert_child)?.to_owned(), main_idx);
            }

            stack.push((main_child, insert_child));
        }
    }
    Ok(())

}