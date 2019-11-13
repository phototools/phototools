// Do we need these?
// extern crate chrono;
// extern crate exif;
// extern crate regex;
use phototools::copier::Copier;

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    run(config);
}

struct Config {
    from_dir: String,
    to_dir: String
}

impl Config {
    fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() != 4 {
            return Err("Please provide 3 arguments: (copy|fix_exif_date) <source dir> <target dir>");
        }

        if args[1].ne("copy") {
            return Err("Currently only the 'copy' operation is supported.")
        }

        let from_dir = args[2].clone();
        let to_dir = args[3].clone();

        Ok(Config { from_dir, to_dir })
    }
}

fn run(config: Config) {
    println!("Source dir: {}", config.from_dir);
    println!("Target dir: {}", config.to_dir);

    match Copier::new().copy(&config.from_dir, &config.to_dir) {
        Ok(()) => return,
        Err(_) => return
    };
}