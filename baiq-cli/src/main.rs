#![cfg_attr(feature="lint", feature(plugin))]
#![cfg_attr(feature="lint", plugin(clippy))]

extern crate baimax;
extern crate chrono;
#[macro_use]
extern crate clap;
extern crate penny;
extern crate serde;
extern crate serde_json;

use std::fs::File;
use std::io;
use std::io::Read;

use baimax::ast;
use baimax::ast::parse::Parsed;
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
        .arg(Arg::with_name("end-of-day")
                 .short("e")
                 .long("end-of-day")
                 .help("Sets the sender's end-of-day time (hh:mm)")
                 .takes_value(true))
        .arg(Arg::with_name("pretty")
                 .short("p")
                 .long("pretty")
                 .help("Return a pretty-printed representation")
                 .overrides_with_all(&["json", "json-pretty"]))
        .arg(Arg::with_name("json")
                 .short("j")
                 .long("json")
                 .help("Return a JSON representation")
                 .overrides_with_all(&["pretty", "json-pretty"]))
        .arg(Arg::with_name("json-pretty")
             .long("json-pretty")
             .help("Return a pretty-printed JSON representation")
             .overrides_with_all(&["pretty", "json"]))
        .arg(Arg::with_name("path")
                 .help("Sets the BAI file to search")
                 .requires_all(&["end-of-day"])
                 .required(true)
                 .index(1))
        .get_matches();

    let end_of_day = args.value_of("end-of-day")
        .expect("No end-of-day time specified.");
    let end_of_day = {
        let (hour, rest) = end_of_day.split_at(2);
        let (rest, minute) = rest.split_at(1);
        if rest != ":" {
            panic!(": expected in end-of-day time.")
        }
        let hour = hour.parse().expect("Invalid end-of-day hour.");
        let minute = minute.parse().expect("Invalid end-of-day minute.");
        chrono::NaiveTime::from_hms(hour, minute, 0)
    };

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
    let raw_records = baimax::parse::file(bytes.as_slice())
        .to_result()
        .expect("Syntax error");
    let file = {
            let mut parsed_records =
                raw_records
                    .iter()
                    .map(|r| ast::Record::parse(r).expect("Field syntax error"));
            ast::convert::convert(&mut parsed_records, &end_of_day)
        }
        .expect("Error converting record");

    match format {
        Format::Pretty => {
            println!("{}", file);
        }
        Format::Json => {
            serde_json::to_writer(io::stdout(), &file).unwrap();
            println!();
        }
        Format::JsonPretty => {
            serde_json::to_writer_pretty(io::stdout(), &file).unwrap();
            println!();
        }
    }
}
