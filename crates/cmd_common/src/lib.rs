use clap::Parser;
use log::Level;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CommonArgs {
    #[arg(
        short,
        long,
        help = "Set the log level, defaults to debug when in debug mode, info otherwise."
    )]
    pub log_level: Option<Level>,
}

pub fn init_log(args: &CommonArgs) {
    let level = args.log_level.unwrap_or({
        if cfg!(debug_assertions) {
            Level::Debug
        } else {
            Level::Info
        }
    });

    simple_logger::init_with_level(level).unwrap();

    log::info!("using log level: {:?}", level);
}
