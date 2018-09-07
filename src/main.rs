#[macro_use(bson, doc)]
extern crate bson;

extern crate reqwest;
extern crate getopts;

mod crawler;
mod app;
mod persist;
mod logger;

use getopts::{Options, Matches};
use std::env;
use std::str::FromStr;
use app::{AppSettings, App};

fn usage(argv0: &str, opts: Options, wrapped_reason: Option<String>) {
    let brief = format!("Usage: {} [options]", argv0);
    print!("{}", opts.usage(&brief));

    if let Some(reason) = wrapped_reason {
        println!("\n{}", reason);
    }
}

enum Error {
    Usage(Option<String>),
}

type DirectemarResult<T> = Result<T, Error>;


use std::fmt::Display;
fn get_required_option<T>(matches: &Matches, option: &str) -> DirectemarResult<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    match matches.opt_str(option) {
        Some(opt_str) => {
            opt_str.parse().map_err(|e| {
                Error::Usage(Some(format!("Error parsing '{}': {}", option, e)))
            })
        }
        None => Err(Error::Usage(Some(format!("Missing option: {}", option)))),
    }
}

fn parse_appsettings(opts: &mut Options, args: &[String]) -> DirectemarResult<AppSettings> {
    opts.optopt("i", "poll-interval", "Poll interval", "INTERVAL");
    opts.optopt("t", "targets", "Comma separated target ids", "TARGETS");
    opts.optopt(
        "k",
        "connection-string",
        "MongoDB connection string",
        "CONNSTRING",
    );
    opts.optopt("d", "db", "MongoDB Database", "DB");
    opts.optopt("c", "collection", "MongoDB Collection", "COLLECTION");

    let matches = opts.parse(&args[1..]).map_err(
        |e| Error::Usage(Some(e.to_string())),
    )?;

    Ok(AppSettings {
        poll_interval: get_required_option(&matches, "poll-interval")?,
        targets: get_required_option::<String>(&matches, "targets")?
            .split(",")
            .map(|s| s.to_owned())
            .collect(),
        connection_string: get_required_option(&matches, "connection-string")?,
        database: get_required_option(&matches, "db")?,
        collection: get_required_option(&matches, "collection")?,
    })
}

fn main() {
    // Parse app settings
    let mut opts = Options::new();
    let args: Vec<String> = env::args().collect();
    let settings = match parse_appsettings(&mut opts, &args) {
        Ok(s) => s,
        Err(Error::Usage(wrapped_reason)) => {
            usage(&args[0], opts, wrapped_reason);
            return;
        }
    };

    let app = App::new(settings);
    app.start();
}