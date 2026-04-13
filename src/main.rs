use anyhow::Context;
use clap::Parser;
use phylo::Result;
use phylo::io::read_newick_from_file;
use phylo::tree::Tree;
use std::collections::HashSet;

mod args;
use crate::args::Args;

pub(crate) fn set_missing_tree_node_ids(tree: &Tree) -> Result<Tree> {
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

fn main() -> Result<()> {
    let args = Args::parse();
    println!("args = {:?}", args);

    let tree = read_newick_from_file(&args.i)
        .expect("Unable to parse Newick tree")
        .pop()
        .expect("Tree file was empty");
    let tree = set_missing_tree_node_ids(&tree).unwrap();

    println!("root has n childer {}", tree.children(&tree.root).len());

    let newick = tree.to_newick();
    // if there is a ) at the second last position, remove it and also the first char if its a (
    if newick.ends_with(");") {
        std::fs::write(&args.ow, &newick).context("Unable to write Newick tree to output file")?;
        let newick_wo = newick[1..newick.len() - 2].to_string() + ";";
        std::fs::write(&args.owo, &newick_wo)
            .context("Unable to write Newick tree to output file")?;
    } else {
        std::fs::write(&args.owo, &newick).context("Unable to write Newick tree to output file")?;
        let newick_with_brackets = format!("({});",& newick[..newick.len() - 1]);
        std::fs::write(&args.ow, &newick_with_brackets)
            .context("Unable to write Newick tree to output file")?;
    };

    Ok(())
}
