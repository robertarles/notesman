extern crate clap;
use clap::{App, Arg};

pub fn arg_handler<'main>() -> clap::ArgMatches<'main> {
    return App::new("notesman")
        .version("0.6.0")
        .arg(
            Arg::with_name("todo_file")
                .help("The source todo file to be processed.")
                .required(true)
                .index(1),
        )
        .get_matches();
}

// add in formatting help
// - [x] --> done, will be archived
// - [ ] . --> work performed, will be journalled
// sections:
// ## active --> active TODO items
// ## backlog --> TODO items that are in the backlog
