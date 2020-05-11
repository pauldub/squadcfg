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
        Ok(wl) => {
            println!("wl: {:?}", wl);
            println!("All is right!")
        }
        Err(err) => println!("lint error: {:?}", err),
    }
}
