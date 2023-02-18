extern crate clap;
extern crate env_logger;
extern crate log;

use env_logger::Builder;
use clap::{arg, value_parser, ArgMatches, Command};
use log::{debug, LevelFilter};
use phototools::copier::Copier;
use std::io::Write;
use std::path::PathBuf;
use std::process;

type GenError = Box<dyn std::error::Error>;

const DEFAULT_FILESIZE_MIN: &str = "500";

fn cli() -> Command {
    Command::new("Photo Tools")
        .bin_name("phototools")
        .about("Tool to organize photos and videos.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(arg!(--"verbose").short('v')
            .help("Set level of verbosity to verbose"))
        .arg(arg!(--"very-verbose").short('w')
            .help("Set level of verbosity to very verbose"))
        .subcommand(
            Command::new("copy")
                .about("Copies photos and videos from one directory to a target directory \
                    where all the items are organized in target folders based on date taken.")
                .arg(arg!(--"source-dir" <PATH>)
                    .short('s')
                    .required(true)
                    .help("The source directory tree")
                    .value_parser(value_parser!(PathBuf)))
                .arg(arg!(--"dest-dir" <PATH>)
                    .short('d')
                    .required(true)
                    .help("The destination directory root")
                    .value_parser(value_parser!(PathBuf)))
                .arg(arg!(--"min-size" <BYTES>)
                    .short('b')
                    .help("When copying only consider photos and videos of at least this size")
                    .value_parser(value_parser!(u16))
                    .default_value(DEFAULT_FILESIZE_MIN))
                .arg(arg!(--"cp-copy")
                    .short('c')
                    .help("Uses 'cp' from the shell to copy files"))
                )
}

fn main() {
    let matches = cli().get_matches();

    let mut verbosity = 0;
    if matches.get_flag("verbose") {
        verbosity = 1;
    }
    if matches.get_flag("very-verbose") {
        verbosity = 2;
    }

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
    println!("Logging level {}", verbosity);
    debug!("Logging level {:?}", verbosity);

    match matches.subcommand() {
        Some(("copy", sub_matches)) => {
            // Copy operation
            let cfg = CopyConfig::from(sub_matches).unwrap_or_else(|err| {
                println!("Problem initializing with arguments: {}", err);
                process::exit(1);
            });
            debug!("Using configuration {:?}", cfg);

            copy(cfg);
        }
        _ => unreachable!()
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
        let src_dir = copy_matches.get_one::<PathBuf>("source-dir").unwrap();
        let dst_dir = copy_matches.get_one::<PathBuf>("dest-dir").unwrap();        
        let min_size = copy_matches.get_one::<u16>("min-size").unwrap();
        let shell_cp = copy_matches.get_flag("cp-copy");

        Ok(CopyConfig {
            from_dir: src_dir.to_string_lossy().into(),
            to_dir: dst_dir.to_string_lossy().into(),
            min_size: *min_size as u64,
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