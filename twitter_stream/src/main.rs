use anyhow::{Context, Result};
use clap::Clap;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use twitter_stream::{
    create_rule, delete_rule, delete_rules, get_bearer_token, get_rules, stream_data, Opts, SubCmd,
};

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    let bearer_token = get_bearer_token(&opts);

    match opts.subcmd {
        None => {
            stream_data(&opts.file, opts.limit, &bearer_token).await?;
            println!("Done :)");
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
