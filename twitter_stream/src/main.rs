use anyhow::{Context, Result};
use clap::Clap;
use console::{Style, Term};
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use futures::StreamExt;
use std::{fs::OpenOptions, time::Instant};
use twitter_stream::{
    create_rule, delete_rule, delete_rules, get_bearer_token, get_rules, stream_data, Opts,
    StreamError, SubCmd,
};

pub async fn append2file() {}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    let bearer_token =
        get_bearer_token(opts.bearer_token.as_deref(), Some(opts.env_file.as_str()))?;

    match opts.subcmd {
        // Do the Streaming
        None => {
            let now = Instant::now();
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(&opts.file)?;

            let term = Term::stdout();
            let bold = Style::new().bold();
            println!("{}", bold.apply_to("Starting the stream..."));
            let mut connection_resets = 0;
            let mut processed = 0usize;
            let mut errors = 0usize;
            let mut finish = false;
            let green = Style::new().green();
            let red = Style::new().red();

            let (mut rate_limit, mut stream) = stream_data(&bearer_token).await?;
            if opts.verbose > 0 {
                println!("{:?}", rate_limit);
            }
            println!("\n");

            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(tweet_data) => {
                        jsonl::write(&mut file, &tweet_data)?;
                        processed += 1;

                        let mut progress = format!("{}", processed);
                        if let Some(limit) = opts.limit {
                            progress.push_str(&format!("/{}", limit));
                            if processed == limit {
                                finish = true;
                            }
                        }

                        term.clear_last_lines(2)?;
                        println!("{} {}", green.apply_to("Processed tweets  :"), progress);
                        println!("{} {}", red.apply_to("Errors encountered:"), errors);
                        if finish {
                            break;
                        }
                    }
                    Err(StreamError::SmallChunk) => {}
                    Err(StreamError::Parse(err)) => {
                        eprintln!(
                            "Couldn't parse tweet data:\n{}\n{:?}\n\n",
                            err.source, err.msg
                        );
                        errors += 1;
                    }
                    Err(StreamError::Reqwest(err)) => {
                        if opts.verbose > 0 {
                            eprintln!("Error reading chunk of data: {:#?}", err);
                        }
                        errors += 1;

                        if let Some(max_resets) = opts.max_resets {
                            if connection_resets >= max_resets {
                                println!(
                                    "Maximum number of connection resets ({}) reached...",
                                    max_resets
                                );
                                break;
                            }
                        }

                        if let Some(rest) = rate_limit.duration_until_reset() {
                            println!("Waiting for rate limit ({:?})...", rest);
                            tokio::time::sleep(rest).await;
                            println!("Resetting connection...\n\n");
                        }

                        let (rl, s) = stream_data(&bearer_token).await?;

                        connection_resets += 1;
                        rate_limit = rl;
                        stream = s;
                    }
                }
            }

            println!("Done :)\n{:?}", now.elapsed());
        }
        Some(SubCmd::ListRules) => {
            let rules = get_rules(&bearer_token).await?;
            println!("{}", rules);
        }
        Some(SubCmd::CreateRule(create_opts)) => {
            let rule_str =
                serde_json::to_string(&create_opts).context("Couldn't serialize rule")?;
            let rule = create_rule(rule_str, &bearer_token).await?;
            println!("{}", rule);
        }
        Some(SubCmd::DeleteRule(delete_opts)) => {
            // Delete all rules if --all
            if delete_opts.all {
                if delete_opts.force
                    || Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt("Do you want to delete all rules?")
                        .interact()
                        .unwrap()
                {
                    match get_rules(&bearer_token).await?.data {
                        Some(ids) => {
                            let ids = ids.into_iter().map(|o| o.id).collect::<Vec<_>>();
                            let n = delete_rules(ids, &bearer_token).await?;
                            println!("All rules deleted ({})", n);
                        }
                        None => {
                            println!("There are no rules in the stream")
                        }
                    }
                }
                return Ok(());
            }

            // Prompt select if no id was given
            let mut id = delete_opts.id;
            if id.is_none() {
                match get_rules(&bearer_token).await?.data {
                    Some(rules) => {
                        id = Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("Pick the rule to delete")
                            .default(0)
                            .items(&rules)
                            .interact_opt()?
                            .map(|i| rules[i].id.clone());
                    }
                    None => {
                        println!("There are no rules in the stream")
                    }
                }
            }

            // Delete 1 rule
            if let Some(id) = id {
                delete_rule(&id, &bearer_token).await?;
                println!("Rule {:?} deleted", id);
            }
        }
    }

    Ok(())
}
