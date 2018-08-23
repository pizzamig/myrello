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
use structopt::clap::Shell;
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
    /// Specify the database file you want to use
    #[structopt(short = "d", long = "db", parse(from_os_str), raw(global = "true"))]
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
    /// Generate autocompletion for zsh
    #[structopt(name = "completion")]
    Completion,
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
    #[structopt(short = "l", long = "label")]
    label: Option<String>,
    #[structopt(short = "r", long = "reference")]
    reference: bool,
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
    /// Create a new task
    #[structopt(name = "new")]
    New {
        /// attach one or more label to the task
        #[structopt(short = "l", long = "label", raw(number_of_values = "1"))]
        labels: Vec<String>,
        /// set a reference to the task
        #[structopt(short = "r", long = "reference")]
        reference: Option<String>,
        /// set a priority
        #[structopt(short = "p", long = "priority")]
        priority: Option<String>,
        /// The task description
        #[structopt(raw(required = "true"))]
        descr: Vec<String>,
    },
    /// Add a label to an existing task
    #[structopt(name = "add-label")]
    AddLabel {
        /// attach one or more label to the task
        #[structopt(short = "l", long = "label", raw(number_of_values = "1"))]
        labels: Vec<String>,
        /// The task description
        #[structopt(short = "t", long = "task")]
        task: u32,
    },
    /// Set a new description for the task
    #[structopt(name = "edit")]
    Edit {
        /// The task description
        #[structopt(short = "t", long = "task")]
        task: u32,
        /// the priority level
        #[structopt(short = "p", long = "priority")]
        priority: Option<String>,
        /// the status level
        #[structopt(short = "s", long = "status")]
        status: Option<String>,
        /// set a reference to the task
        #[structopt(short = "r", long = "reference")]
        reference: Option<String>,
        /// The task description
        #[structopt()]
        descr: Vec<String>,
    },
    /// Close a task
    #[structopt(name = "done")]
    Done(OptTaskOnly),
    /// Start to work on a task
    #[structopt(name = "start")]
    Start(OptTaskOnly),
    /// Mark the task as blocked
    #[structopt(name = "block")]
    Block(OptTaskOnly),
    /// Delete a task
    #[structopt(name = "delete")]
    Delete(OptTaskOnly),
    /// Increase the priority of a task
    #[structopt(name = "prio")]
    Prio(OptTaskOnly),
}

#[derive(Debug, StructOpt)]
struct OptTaskOnly {
    /// The task id
    #[structopt(short = "t", long = "task")]
    task_id: u32,
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
        Cmd::Completion => {
            Opt::clap().gen_completions_to("myrello", Shell::Zsh, &mut std::io::stdout());
        }
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
            TaskCmd::New {
                labels,
                priority,
                reference,
                descr,
            } => {
                let mut text = String::new();
                for x in descr {
                    text.push_str(&x);
                    text.push(' ');
                }
                info!("add a task with description {}", text);
                let new_id = db::add_task(&dbfile, &text)?;
                if !labels.is_empty() {
                    debug!("set labels");
                    db::add_labels(&dbfile, new_id, &labels)?;
                }
                if let Some(priority_str) = priority {
                    debug!("set priority {}", priority_str);
                    db::set_priority(&dbfile, new_id, &priority_str)?;
                }
                if let Some(ref_str) = reference {
                    debug!("set reference {}", ref_str);
                    db::set_reference(&dbfile, new_id, &ref_str)?;
                }
                println!("Create a new task, with id {}", new_id);
            }
            TaskCmd::AddLabel { labels, task } => {
                if labels.is_empty() {
                    error!("You have to specify at least one label");
                } else {
                    db::add_labels(&dbfile, task, &labels)?;
                }
            }
            TaskCmd::Edit {
                task,
                priority,
                reference,
                status,
                descr,
            } => {
                let mut text = String::new();
                if priority.is_none() && status.is_none() && descr.is_empty() {
                    error!("You have to specify at least on attribute you want to edit");
                } else {
                    if !descr.is_empty() {
                        for x in descr {
                            text.push_str(&x);
                            text.push(' ');
                            info!("edit the task with a new description {}", text);
                            db::set_descr(&dbfile, task, &text)?;
                        }
                    }
                    if let Some(priority) = priority {
                        db::set_priority(&dbfile, task, &priority)?;
                    }
                    if let Some(ref_str) = reference {
                        debug!("set reference {}", ref_str);
                        db::set_reference(&dbfile, task, &ref_str)?;
                    }
                    if let Some(status) = status {
                        if status == "done" {
                            warn!("To make a task as done, please use the command task-done");
                            db::complete_task(&dbfile, task)?;
                        }
                        db::set_status(&dbfile, task, &status)?;
                    }
                }
            }
            TaskCmd::Start(task) => {
                info!("Start task {}", task.task_id);
                db::set_status(&dbfile, task.task_id, "in_progress")?;
            }
            TaskCmd::Block(task) => {
                info!("Block task {}", task.task_id);
                db::set_status(&dbfile, task.task_id, "block")?;
            }
            TaskCmd::Done(task) => {
                info!("Completed task {}", task.task_id);
                db::complete_task(&dbfile, task.task_id)?;
                db::set_status(&dbfile, task.task_id, "done")?;
            }
            TaskCmd::Delete(task) => {
                info!("Delete task {}", task.task_id);
                db::delete_task(&dbfile, task.task_id)?;
            }
            TaskCmd::Prio(task) => {
                info!("Increase priority of task {}", task.task_id);
                db::increase_priority(&dbfile, task.task_id)?;
            }
        },
        Cmd::Show(showopt) => {
            let tasks = db::get_open_tasks(&dbfile)?;
            if showopt.all {
            } else if let Some(label) = showopt.label {
                task::show_tasks_label(&dbfile, &tasks, &label, showopt.reference);
            } else {
                task::show_tasks(&dbfile, &tasks, showopt.reference);
            }
        }
    };
    trace!("myrello end");
    Ok(())
}
