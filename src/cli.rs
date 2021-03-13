extern crate clap;
use clap::{Arg, App};

pub fn arg_handler <'main> () -> clap::ArgMatches<'main> {
    return App::new("notesman")
        .version("0.5.4")
        .arg(
            Arg::with_name("todo_file")
                .help("The source todo file to be processed.")
                .required(true)
                .index(1),
        )
        .get_matches();
}
