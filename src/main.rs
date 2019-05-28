extern crate clap;

mod license;

use clap::{App, Arg};
use license::Match;
use std::fs::File;
use std::io::prelude::*;


fn main() {
    let matches = App::new("copyup")
        .version("0.1")
        .author("Matt Carey")
        .about("Download a LICENSE file based on its SPDX identifier.")
        .arg(
            Arg::with_name("list")
                .short("l")
                .long("list")
                .help("Show a list of all available licenses.")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("stdout")
                .short("s")
                .long("stdout")
                .help("Output directly to STDOUT")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .help("Specify an output file [defaults to LICENSE].")
                .conflicts_with("stdout"),
        )
        .arg(
            Arg::with_name("spdx_identifier")
                .help("The SPDX identifier corresponding to the desired license.")
                .index(1)
                .required_unless("list"),
        )
        .get_matches();

    let list = license::fetch_list().expect("Unable to fetch license list.");

    if matches.is_present("list") {
        list.display_list();
        std::process::exit(0);
    }

    let spdx_id = matches.value_of("spdx_identifier").unwrap();

    match list.find_match(spdx_id) {
        Match::Exact(entry) => {
            let text = entry.fetch_text().expect("Unable to fetch license text.");
            if matches.is_present("stdout") {
                println!(
                    "{}",
                    text);
                std::process::exit(0);
            } else {
                let path = matches.value_of("output").unwrap_or("LICENSE");
                let mut file = match File::create(path) {
                    Err(e) => {
                        println!("Error creating file `{}': {}", path, e);
                        std::process::exit(1);
                    },
                    Ok(f) => f
                };

                if let Err(e) = file.write_all(text.as_bytes()) {
                        println!("Error writing to file `{}': {}", path, e);
                        std::process::exit(1);
                
                }
            }
        }
        Match::Closest(entry) => {
            println!("Did you mean `{}'?", &entry.license_id);
            std::process::exit(1);
        }
        Match::None => {
            println!("Unable to find a matching identifier or closest match.");
            std::process::exit(1);
        }
    }
}
