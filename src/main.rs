use anyhow::Context;
use clap::Parser;
use std::collections::HashSet;

use phylo::Result;
use phylo::bail;
use phylo::io::read_newick_from_file;
use phylo::tree::Tree;

mod args;
use crate::args::Args;

fn set_missing_tree_node_ids(tree: &Tree) -> Result<Tree> {
    let mut tree_with_all_ids = tree.clone();
    let mut seen_user_set_ids = HashSet::new();
    let mut count = 0;
    for node_idx in tree.postorder() {
        let id = tree.node_id(node_idx);
        if id.is_empty() {
            let mut new_id = format!("I{count}");
            while !seen_user_set_ids.insert(new_id.clone()) {
                count += 1;
                new_id = format!("I{count}");
            }
            tree_with_all_ids.node_mut(node_idx).id = new_id.clone();
        }
    }
    Ok(tree_with_all_ids)
}

fn avoid_zero_blen_after_root(tree: &Tree) -> Result<Tree> {
    let mut tree_with_non_zero_blen_after_root = tree.clone();
    let before_tree_length = tree.length;
    for root_child in tree.children(&tree.root) {
        if tree.node(root_child).blen == 0.0 {
            let sibling = tree.sibling(root_child);
            if sibling.is_none() {
                bail!(Tree, "Root child has no sibling");
            }
            let sibling = sibling.unwrap();
            let sibling_blen = tree.node(&sibling).blen;
            tree_with_non_zero_blen_after_root.set_blen(&sibling, sibling_blen / 2.0);
            tree_with_non_zero_blen_after_root.set_blen(root_child, sibling_blen / 2.0);
        }
    }
    if (tree_with_non_zero_blen_after_root.length - before_tree_length).abs() > 1e-6 {
        bail!(
            Tree,
            "Tree length changed after avoiding zero branch lengths after root"
        );
    }
    Ok(tree_with_non_zero_blen_after_root)
}

fn write_newick_with_and_without_brackets(tree: &Tree) -> Result<()> {
    let newick = tree.to_newick();
    // if there is a ) at the second last position, remove it and also the first char if its a (
    if newick.ends_with(");") {
        std::fs::write("output_with_brackets.newick", &newick)
            .context("Unable to write Newick tree to output file")?;
        let newick_wo = newick[1..newick.len() - 2].to_string() + ";";
        std::fs::write("output_without_brackets.newick", &newick_wo)
            .context("Unable to write Newick tree to output file")?;
    } else {
        std::fs::write("output_without_brackets.newick", &newick)
            .context("Unable to write Newick tree to output file")?;
        let newick_with_brackets = format!("({});", &newick[..newick.len() - 1]);
        std::fs::write("output_with_brackets.newick", &newick_with_brackets)
            .context("Unable to write Newick tree to output file")?;
    };
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    // If the tree is not rooted the parsing of the newick will root it
    let tree = read_newick_from_file(&args.i)
        .expect("Unable to parse Newick tree")
        .pop()
        .expect("Tree file was empty");
    // If there are missing node ids at the internal nodes, then they are set to I1, I2, ...
    let tree = set_missing_tree_node_ids(&tree)?;
    // If there are zero branch lengths after the root, they are set to half of the sibling branch length
    let tree = avoid_zero_blen_after_root(&tree)?;
    // The tree is written to two files, one with brackets around the whole tree and one without
    write_newick_with_and_without_brackets(&tree)?;
    Ok(())
}
