extern crate clap;
extern crate env_logger;
extern crate log;

use env_logger::Builder;
use clap::{Arg, ArgMatches, App, SubCommand};
use log::{debug, LevelFilter};
use phototools::copier::Copier;
use std::io::Write;
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
                .takes_value(true))
            .arg(Arg::with_name("cp copy").short("c").long("cp-copy")
                .help("Uses 'cp' from the shell to copy files")
                .takes_value(false)))
        .get_matches();
   
    let verbosity = matches.occurrences_of("v");
    let mut log_builder = Builder::from_default_env();
    log_builder.format(|buf, record| {
        writeln!(buf,
            "[{}] - {}",
            record.level(),
            record.args()
        )
    });

    let _ = match verbosity {
        0 => log_builder.filter_level(LevelFilter::Info),
        1 => log_builder.filter_level(LevelFilter::Debug),
        _ => log_builder.filter_level(LevelFilter::Trace)
    };
    log_builder.init();
    
    if let Some(copy_matches) = matches.subcommand_matches("copy") {
        // Copy operation
        let cfg = CopyConfig::from(copy_matches).unwrap_or_else(|err| {
            println!("Problem initializing with arguments: {}", err);
            process::exit(1);
        });
        debug!("Using configuration {:?}", cfg);
        copy(cfg);
    }
}

#[derive(Debug)]
struct CopyConfig {
    from_dir: String,
    to_dir: String,
    min_size: u64,
    shell_cp: bool
}

impl CopyConfig {
    fn from(copy_matches: &ArgMatches) -> Result<CopyConfig, GenError> {
        let min_size_str = copy_matches.value_of("minimum size").unwrap_or("500");
        let src_dir = copy_matches.value_of("source directory").unwrap();
        let dst_dir = copy_matches.value_of("destination directory").unwrap();
        let shell_cp = copy_matches.occurrences_of("cp copy") > 0;

        let min_size = min_size_str.parse::<u64>()?;

        Ok(CopyConfig { 
            from_dir: String::from(src_dir), 
            to_dir: String::from(dst_dir),
            min_size,
            shell_cp
        })
    }
}

fn copy(config: CopyConfig) {
    debug!("Source dir: {}", config.from_dir);
    debug!("Target dir: {}", config.to_dir);

    Copier::new(config.min_size, config.shell_cp)
        .copy(&config.from_dir, &config.to_dir).unwrap();
}