extern crate seahorse;
extern crate squadcfg;

use seahorse::{App, Command, Context};
use std::env;
use std::fs;
use std::io::Read;

use squadcfg::admin;

fn main() {
    let args: Vec<String> = env::args().collect();
    let app = App::new()
        .name(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage("squadcfg [command]")
        .command(lint_command());

    app.run(args);
}

fn lint_command() -> Command {
    Command::new()
        .name("lint")
        .usage("squadcfg lint FILE")
        .action(lint_action)
}

fn lint_action(c: &Context) {
    let file_path = c.args.first().unwrap();
    let mut file = fs::File::open(file_path).unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    println!("linting: {}", file_path);

    match admin::parse_whitelist(&contents) {
        Ok(wl) => match lint_groups(&wl, file_path, &contents) {
            Err(err) => {
                println!("lint error: {}", err);
            }
            _ => {}
        },
        Err(err) => {
            println!("syntax error: {}", err);
        }
    }
}

use std::collections::HashMap;

fn lint_groups(
    whitelist: &admin::Whitelist,
    file_path: &str,
    contents: &str,
) -> Result<(), admin::Error> {
    let mut groups_index: HashMap<String, bool> = HashMap::new();

    for group in &whitelist.groups {
        groups_index.insert(group.name.to_lowercase(), true);
    }

    for player in &whitelist.players {
        if groups_index.get(&player.group.to_lowercase()).is_none() {
            return Err(admin::Error::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: format!(
                        "player `{}` has an invalid group name `{}`",
                        player.steam_id, player.group
                    ),
                },
                pest::Span::new(contents, player.span.start, player.span.end).unwrap(),
            )
            .with_path(file_path));
        }
    }

    Ok(())
}
