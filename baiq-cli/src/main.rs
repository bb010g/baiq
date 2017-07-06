#![cfg_attr(feature="lint", feature(plugin))]
#![cfg_attr(feature="lint", plugin(clippy))]

extern crate baimax;
extern crate chrono;
#[macro_use]
extern crate clap;
extern crate nom;
extern crate penny;
extern crate serde;
extern crate serde_json;

use std::fs::File;
use std::io;
use std::io::{Read, Write};

use clap::{AppSettings, Arg};

enum Format {
    Pretty,
    Json,
    JsonPretty,
}

fn main() {
    let args = app_from_crate!()
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::ColoredHelp)
        .max_term_width(100)
        .arg(
            Arg::with_name("end-of-day")
                .short("e")
                .long("end-of-day")
                .help("Sets the sender's end-of-day time (hh:mm)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("pretty")
                .short("p")
                .long("pretty")
                .help("Return a pretty-printed representation")
                .overrides_with_all(&["json", "json-pretty"]),
        )
        .arg(
            Arg::with_name("json")
                .short("j")
                .long("json")
                .help("Return a JSON representation")
                .overrides_with_all(&["pretty", "json-pretty"]),
        )
        .arg(
            Arg::with_name("json-pretty")
                .long("json-pretty")
                .help("Return a pretty-printed JSON representation")
                .overrides_with_all(&["pretty", "json"]),
        )
        .arg(
            Arg::with_name("path")
                .help("Sets the BAI file to search")
                .requires_all(&["end-of-day"])
                .required(true)
                .index(1),
        )
        .get_matches();

    let path = args.value_of("path").expect("No path supplied");
    let file = File::open(path).expect("No file found at path");

    let format = if args.is_present("json") {
        Format::Json
    } else if args.is_present("json-pretty") {
        Format::JsonPretty
    } else {
        Format::Pretty
    };

    let bytes = file.bytes()
        .collect::<Result<Vec<_>, _>>()
        .expect("Byte reading error");

    let file = baimax::data::File::process(bytes.as_slice());
    let file = file.expect("Processing error");

    let mut stdout = io::stdout();
    let _ = match format {
        Format::Pretty => writeln!(stdout, "{}", file).or(Err(())),
        Format::Json => {
            serde_json::to_writer(&mut stdout, &file)
                .or(Err(()))
                .and_then(|()| writeln!(&mut stdout).or(Err(())))
        }
        Format::JsonPretty => {
            serde_json::to_writer_pretty(&mut stdout, &file)
                .or(Err(()))
                .and_then(|()| writeln!(&mut stdout).or(Err(())))
        }
    };
}
