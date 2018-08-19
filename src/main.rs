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
#[macro_use]
extern crate prettytable;
extern crate rusqlite;

mod db;
mod task;

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
    Show(ShowOpt),
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
struct ShowOpt {
    #[structopt(short = "a", long = "all")]
    all: bool,
    #[structopt(short = "l", long = "labels")]
    labels: bool,
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
        /// attach one or more label to the task
        #[structopt(short = "l", long = "label")]
        labels: Vec<String>,
        /// The task description
        #[structopt()]
        descr: Vec<String>,
    },
    /// Add a lable to an existing task
    #[structopt(name = "add-label")]
    AddLabel {
        /// attach one or more label to the task
        #[structopt(short = "l", long = "label")]
        labels: Vec<String>,
        /// The task description
        #[structopt(short = "t", long = "task")]
        task: u32,
    },
    /// Close a task
    #[structopt(name = "done")]
    Done {
        /// The task id
        #[structopt(short = "t", long = "task")]
        task: u32,
    },
    /// Delete a task
    #[structopt(name = "delete")]
    Delete {
        /// The task id
        #[structopt(short = "t", long = "task")]
        task: u32,
    },
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    opt.verbose.setup_env_logger("myrello")?;
    trace!("myrello begin");
    let dbfile = match opt.dbfile {
        Some(x) => x,
        None => {
            let default_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("./"));
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
            TaskCmd::Add { labels, descr } => {
                let mut text = String::new();
                for x in descr {
                    text.push_str(&x);
                    text.push(' ');
                }
                info!("add a task with description {}", text);
                let new_id = db::add_task(&dbfile, &text)?;
                if !labels.is_empty() {
                    db::add_labels(&dbfile, new_id, &labels)?;
                }
            }
            TaskCmd::AddLabel { labels, task } => {
                if labels.is_empty() {
                    error!("You have to specify at least one label");
                }
                db::add_labels(&dbfile, task, &labels)?;
            }
            TaskCmd::Done { task } => {
                info!("Completed task {}", task);
                db::complete_task(&dbfile, task)?;
            }
            TaskCmd::Delete { task } => {
                info!("Delete task {}", task);
                db::delete_task(&dbfile, task)?;
            }
        },
        Cmd::Show(showopt) => {
            let tasks = db::get_open_tasks(&dbfile)?;
            if showopt.all {
            } else if showopt.labels {
                task::show_tasks_labels(&dbfile, &tasks);
            } else {
                task::show_tasks(&tasks);
            }
        }
    };
    trace!("myrello end");
    Ok(())
}
