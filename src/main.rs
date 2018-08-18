extern crate clap_verbosity_flag;
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate log;
extern crate dirs;
extern crate failure;
//#[macro_use]
//extern crate failure_derive;
extern crate chrono;
extern crate mkdirp;
extern crate rusqlite;

mod db;

use clap_verbosity_flag::Verbosity;
use failure::Error;
use std::path::PathBuf;
use structopt::StructOpt;

//#[derive(Debug, Fail)]
//enum MyrelloError {
//    #[fail(display = "Failed to retrieve the home directory")]
//    HomeDirError,
//}

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(flatten)]
    verbose: Verbosity,
    #[structopt(short = "d", long = "db", parse(from_os_str))]
    dbfile: Option<PathBuf>,
    #[structopt(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, StructOpt)]
enum Cmd {
    /// Show all
    #[structopt(name = "show")]
    Show,
    /// Work on the task database
    #[structopt(name = "database")]
    Db(DbOpt),
    /// Work on tasks/todos
    #[structopt(name = "task")]
    Task(TaskOpt),
}

#[derive(Debug, StructOpt)]
struct DbOpt {
    #[structopt(subcommand)]
    cmd: DbCmd,
}

#[derive(Debug, StructOpt)]
struct TaskOpt {
    #[structopt(subcommand)]
    cmd: TaskCmd,
}

#[derive(Debug, StructOpt)]
enum DbCmd {
    /// Database initialization
    #[structopt(name = "init")]
    Init {
        /// Force the initialization. Current data found in the data file will be lost.
        #[structopt(short = "f", long = "force")]
        force: bool,
    },
}

#[derive(Debug, StructOpt)]
enum TaskCmd {
    /// Add a task initialization
    #[structopt(name = "add")]
    Add {
        #[structopt()]
        descr: Vec<String>,
    },
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    opt.verbose.setup_env_logger("myrello")?;
    trace!("myrello begin");
    let dbfile = match opt.dbfile {
        Some(x) => x,
        None => {
            let default_dir = dirs::data_dir().unwrap_or(PathBuf::from("./"));
            if !default_dir.exists() {
                trace!("creating not existing default directory {:?}", default_dir);
                mkdirp::mkdirp(&default_dir)?;
            }
            default_dir.join("myrello.db")
        }
    };
    trace!("Using {:?} as database", dbfile);
    match opt.cmd {
        Cmd::Db(dbcmd) => match dbcmd.cmd {
            DbCmd::Init { force } => {
                if force {
                    warn!(
                        "Forced database initialization! All previous data will be lost [{:?}]",
                        dbfile
                    );
                } else {
                    info!("database initialization [{:?}]", dbfile);
                }
                db::init(&dbfile)?;
            }
        },
        Cmd::Task(taskcmd) => match taskcmd.cmd {
            TaskCmd::Add { descr } => {
                let mut text = String::new();
                for x in descr {
                    text.push_str(&x);
                    text.push(' ');
                }
                info!("add a task with description {}", text);
                db::add_task(&dbfile, &text)?;
            }
        },
        Cmd::Show => {}
    };
    trace!("myrello end");
    Ok(())
}
