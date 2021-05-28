use anyhow::Result;
use clap::{AppSettings, Clap};
use console::Style;
use csv::Writer;
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
use twitter_stream::StreamResponse;

/// Producs node and edges files with graph information from a JSON Lines file
/// with `StreamResponse` items
#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// JSON Lines file
    jsonl_file: String,
    /// Nodes output name
    #[clap(short, long)]
    nodes_file: Option<String>,
    /// Edges output name
    #[clap(short, long)]
    edges_file: Option<String>,
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

fn read_file<P: AsRef<Path>>(file: P, verbose: bool) -> Result<(Vec<StreamResponse>, usize)> {
    let txt = std::fs::read_to_string(file)?;
    let mut errors = 0;
    let items = txt
        .split('\n')
        .map(|line| serde_json::from_str::<StreamResponse>(line))
        .filter_map(|o| match o {
            Ok(res) => Some(res),
            Err(err) => {
                match err.classify() {
                    serde_json::error::Category::Eof => {}
                    _ => {
                        if verbose {
                            println!("{:?}", err);
                        }
                        errors += 1;
                    }
                }
                None
            }
        })
        .collect::<Vec<_>>();
    Ok((items, errors))
}

#[derive(Serialize)]
struct NodeRow<'a> {
    id: usize,
    label: &'a str,
    class: NodeClass,
    text: Option<&'a str>,
}

#[derive(Serialize)]
enum NodeClass {
    User,
    Tweet,
}

#[derive(Serialize)]
struct EdgeRow {
    source: usize,
    target: usize,
    #[serde(rename = "type")]
    typ: EdgeType,
    id: usize,
    class: EdgeClass,
}

#[allow(dead_code)]
#[derive(Serialize)]
enum EdgeType {
    Directed,
    Undirected,
}

#[derive(Serialize)]
enum EdgeClass {
    TweetOwner,
    ReferencedTweet,
    UserMention,
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    let bold = Style::new().bold();
    let green = Style::new().bold().green();
    let red = Style::new().bold().red();
    let verbose = opts.verbose > 0;

    // Get paths
    let source_file = Path::new(&opts.jsonl_file);
    let source_fn = source_file.file_stem().unwrap().to_str().unwrap();
    let path = source_file.parent().unwrap().to_path_buf();

    let nodes_fname = opts
        .nodes_file
        .unwrap_or(format!("{}_nodes.csv", source_fn));
    let mut nodes_path = path.clone();
    nodes_path.push(nodes_fname);

    let edges_fname = opts
        .edges_file
        .unwrap_or(format!("{}_edges.csv", source_fn));
    let mut edges_path = path;
    edges_path.push(edges_fname);

    println!("Obtaining graph from {}", bold.apply_to(&opts.jsonl_file));
    let (data, errors) = read_file(source_file, verbose)?;
    println!("Found {} items", green.apply_to(data.len()));
    if errors > 0 {
        println!("Found {} errors", red.apply_to(errors));
    }

    // Nodes
    let mut writer = Writer::from_path(&nodes_path)?;
    // 1. get user nodes
    let user_nodes = data
        .iter()
        .filter_map(|o| o.includes.users.get(0))
        .map(|o| &o.username)
        .collect::<HashSet<_>>()
        .into_iter()
        .enumerate()
        .map(|(i, label)| NodeRow {
            id: i,
            label,
            class: NodeClass::User,
            text: None,
        })
        .collect::<Vec<_>>();

    // 2. get tweet nodes
    let tweet_nodes = data
        .iter()
        .enumerate()
        .map(|(i, tweet)| NodeRow {
            id: i + user_nodes.len(),
            label: &tweet.data.id,
            class: NodeClass::Tweet,
            text: Some(&tweet.data.text),
        })
        .collect::<Vec<_>>();

    // 3. write to nodes file
    user_nodes.iter().chain(tweet_nodes.iter()).for_each(|row| {
        let res = writer.serialize(row);
        if res.is_err() && verbose {
            println!("{:?}", res);
        }
    });
    writer.flush()?;
    println!(
        "Nodes saved on: {}",
        bold.apply_to(nodes_path.to_str().unwrap())
    );

    // Edges
    let mut writer = Writer::from_path(&edges_path)?;
    let mut i = 0;

    // 1. Get nodes hashmap
    let nodes_map = user_nodes
        .iter()
        .chain(tweet_nodes.iter())
        .map(|row| (row.label, row.id))
        .collect::<HashMap<_, _>>();

    // 2. username -> tweet (tweet owner)
    data.iter()
        .filter_map(|tweet| {
            if let Some(user) = tweet.includes.users.get(0) {
                if let (Some(&source), Some(&target)) = (
                    nodes_map.get(user.username.as_str()),
                    nodes_map.get(tweet.data.id.as_str()),
                ) {
                    let edge = EdgeRow {
                        source,
                        target,
                        typ: EdgeType::Directed,
                        id: i,
                        class: EdgeClass::TweetOwner,
                    };
                    i += 1;
                    return Some(edge);
                }
            }
            None
        })
        .for_each(|row| {
            let res = writer.serialize(row);
            if res.is_err() && verbose {
                println!("{:?}", res);
            }
        });

    // 3. tweet -> refering tweets
    data.iter()
        .filter_map(|tweet| {
            if let (Some(references), Some(&source)) = (
                &tweet.data.referenced_tweets,
                nodes_map.get(tweet.data.id.as_str()),
            ) {
                let edge_info = references
                    .iter()
                    .filter_map(|reference| nodes_map.get(reference.id.as_str()))
                    .map(move |&target| (source, target));
                return Some(edge_info);
            }
            None
        })
        .flatten()
        .for_each(|(source, target)| {
            let row = EdgeRow {
                source,
                target,
                typ: EdgeType::Directed,
                id: i,
                class: EdgeClass::ReferencedTweet,
            };
            let res = writer.serialize(row);
            if res.is_err() && verbose {
                println!("{:?}", res);
            } else {
                i += 1;
            }
        });

    // 4. tweet -> user mentions
    data.iter()
        .filter_map(|tweet| {
            if let (Some(&source), Some(entities)) =
                (nodes_map.get(tweet.data.id.as_str()), &tweet.data.entities)
            {
                if let Some(mentions) = &entities.mentions {
                    let edge_info = mentions
                        .iter()
                        .filter_map(|mention| nodes_map.get(mention.username.as_str()))
                        .map(move |&target| (source, target));
                    return Some(edge_info);
                }
            }
            None
        })
        .flatten()
        .for_each(|(source, target)| {
            let row = EdgeRow {
                source,
                target,
                typ: EdgeType::Directed,
                id: i,
                class: EdgeClass::UserMention,
            };
            let res = writer.serialize(row);
            if res.is_err() && verbose {
                println!("{:?}", res);
            } else {
                i += 1;
            }
        });

    writer.flush()?;
    println!(
        "Edges saved on: {}",
        bold.apply_to(edges_path.to_str().unwrap())
    );

    Ok(())
}
