use exitfailure::ExitFailure;
use std::path::PathBuf;
use structopt::StructOpt;
use structopt_flags::{ForceFlag, LogLevel, Verbose};

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(flatten)]
    verbose: Verbose,
    #[structopt(flatten)]
    force_flag: ForceFlag,
    /// Specify the database file you want to use
    #[structopt(short = "d", long = "db", parse(from_os_str), raw(global = "true"))]
    dbfile: Option<PathBuf>,
}

struct Config {
    dbfile: PathBuf,
    force: bool,
}

impl From<Opt> for Config {
    fn from(opt: Opt) -> Self {
        Config {
            dbfile: opt.dbfile.unwrap_or_else(myrello::db::dbfile_default),
            force: opt.force_flag.force,
        }
    }
}

fn main() -> Result<(), ExitFailure> {
    let opt = Opt::from_args();
    opt.verbose.set_log_level();
    let config: Config = opt.into();
    if config.force {
        myrello::db::dbdir_create(&config.dbfile)?;
    }
    Ok(())
}
