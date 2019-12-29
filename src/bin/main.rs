extern crate clap;

use clap::{Arg, ArgMatches, App, SubCommand};
use phototools::copier::Copier;

use std::process;

type GenError = Box<dyn std::error::Error>;

fn main() {
    let matches = App::new("Photo Tools")
        .version("0.1")
        .about("Tool to organize photos and videos.")
        .arg(Arg::with_name("v").short("v").multiple(true)
            .help("Sets level of verbosity"))
        .subcommand(SubCommand::with_name("copy")
            .about("Copies photos and videos from one directory to a target directory \
                where all the items are organized in target folders based on date taken.")
            .arg(Arg::with_name("minimum size").short("b").long("min-size").value_name("BYTES")
                .help("When copying only consider photos and videos of at least this size")
                .takes_value(true))
            .arg(Arg::with_name("source directory").short("s").long("source-dir").value_name("DIR")
                .required(true)
                .help("The source directory tree")
                .takes_value(true))
            .arg(Arg::with_name("destination directory").short("d").long("dest-dir").value_name("DIR")
                .required(true)
                .help("The destination directory root")
                .takes_value(true)))
        .get_matches();
   
    let verbosity = matches.occurrences_of("v");
    println!("Verbosity: {}", verbosity);

    if let Some(copy_matches) = matches.subcommand_matches("copy") {
        // Copy operation
        let cfg = CopyConfig::from(&matches, copy_matches).unwrap_or_else(|err| {
            println!("Problem initializing with arguments: {}", err);
            process::exit(1);
        });
        println!("Using configuration {:?}", cfg);
        copy(cfg);
    }
}

#[derive(Debug)]
struct CopyConfig {
    from_dir: String,
    to_dir: String,
    min_size: u32,
    verbosity: u8
}

impl CopyConfig {
    fn from(gen_matches: &ArgMatches, copy_matches: &ArgMatches) -> Result<CopyConfig, GenError> {
        let min_size_str = copy_matches.value_of("minimum size").unwrap_or("500");
        let src_dir = copy_matches.value_of("source directory").unwrap();
        let dst_dir = copy_matches.value_of("destination directory").unwrap();
        let min_size = min_size_str.parse::<u32>()?;

        Ok(CopyConfig { 
            from_dir: String::from(src_dir), 
            to_dir: String::from(dst_dir),
            min_size,
            verbosity: gen_matches.occurrences_of("v") as u8
        })
    }
}

fn copy(config: CopyConfig) {
    println!("Source dir: {}", config.from_dir);
    println!("Target dir: {}", config.to_dir);

    Copier::new(config.verbosity).copy(&config.from_dir, &config.to_dir, config.min_size).unwrap();
}