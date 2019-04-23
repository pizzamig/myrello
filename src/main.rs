#![cfg_attr(feature = "nightly", feature(tool_lints))]
#[allow(unused_imports)]
mod db;
mod task;

use exitfailure::ExitFailure;
use failure::ResultExt;
use log::{debug, error, info, trace, warn};
use std::path::PathBuf;
use std::string::ToString;
use structopt::clap::Shell;
use structopt::StructOpt;
use structopt_flags::LogLevel;
use task::TimeWindow;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(flatten)]
    verbose: structopt_flags::Verbose,
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
struct ShowOpt {
    #[structopt(flatten)]
    show_opts: ShowCommonOpt,
    #[structopt(subcommand)]
    cmd: Option<ShowCmd>,
    /// The task id
    #[structopt(short = "t", long = "task")]
    task: Option<u32>,
}

#[derive(Debug, StructOpt)]
enum ShowCmd {
    /// Show tasks, except the completed ones
    #[structopt(name = "all")]
    All {
        #[structopt(flatten)]
        show_opts: ShowCommonOpt,
    },
    /// Show few tasks: high priority and/or in progress
    #[structopt(name = "short")]
    Short {
        #[structopt(flatten)]
        show_opts: ShowCommonOpt,
    },
    /// Show the backlog, all the tasks in todo
    #[structopt(name = "backlog")]
    Backlog {
        #[structopt(flatten)]
        show_opts: ShowCommonOpt,
    },
    /// Show the tasks that are "in_progress"
    #[structopt(name = "work")]
    Work {
        #[structopt(flatten)]
        show_opts: ShowCommonOpt,
    },
    /// Show the tasks that are "in_progress"
    #[structopt(name = "done")]
    Done {
        #[structopt(flatten)]
        show_opts: ShowCommonOpt,
        #[structopt(short = "T", long = "time")]
        time_window: Option<TimeWindow>,
    },
}

#[derive(Debug, StructOpt, Default)]
struct ShowCommonOpt {
    /// Show fields normally hidden, like story points
    #[structopt(short = "H", long = "hidden")]
    hidden: bool,
    /// Select one or more label as filter
    #[structopt(short = "l", long = "label", raw(number_of_values = "1"))]
    labels: Vec<String>,
    /// Show references as well
    #[structopt(short = "r", long = "reference")]
    reference: bool,
}

impl ShowCommonOpt {
    fn merge(&mut self, to_merge: &mut ShowCommonOpt) {
        if to_merge.hidden {
            self.hidden = true;
        }
        if to_merge.reference {
            self.reference = true;
        }
        self.labels.append(&mut to_merge.labels);
    }
}
#[derive(Debug, StructOpt)]
struct TaskOpt {
    #[structopt(subcommand)]
    cmd: TaskCmd,
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
        /// the story points
        #[structopt(short = "S", long = "story-points")]
        storypoint: Option<u32>,
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
        /// The task id
        #[structopt(short = "t", long = "task")]
        task: u32,
    },
    /// Set a new description for the task
    #[structopt(name = "edit")]
    Edit {
        /// The task id
        #[structopt(short = "t", long = "task")]
        task: u32,
        /// the priority level
        #[structopt(short = "p", long = "priority")]
        priority: Option<String>,
        /// the task' status
        #[structopt(short = "s", long = "status")]
        status: Option<String>,
        /// the story points
        #[structopt(short = "S", long = "story-points")]
        storypoint: Option<u32>,
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

fn main() -> Result<(), ExitFailure> {
    let opt = Opt::from_args();
    env_logger::Builder::new()
        .filter(Some("myrello"), opt.verbose.get_level_filter())
        .filter(None, log::LevelFilter::Error)
        .try_init()
        .with_context(|_| {
            format!(
                "could net initialize the logger at level {}",
                opt.verbose.get_level_filter()
            )
        })?;
    trace!("opt => {:?}", opt);
    trace!("myrello begin");
    let dbfile = match opt.dbfile {
        Some(x) => x,
        None => {
            let default_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("./"));
            if !default_dir.exists() {
                trace!("creating not existing default directory {:?}", default_dir);
                mkdirp::mkdirp(&default_dir).with_context(|_| {
                    format!(
                        "Could not create the default directory {}",
                        default_dir
                            .to_str()
                            .unwrap_or("Directory name not printable")
                    )
                })?;
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
                db::init(&dbfile, force).with_context(|_| {
                    format!(
                        "Database initialization failed on file {}",
                        dbfile.to_str().unwrap_or("Filename not printable")
                    )
                })?;
            }
        },
        Cmd::Task(taskcmd) => match taskcmd.cmd {
            TaskCmd::New {
                labels,
                priority,
                storypoint,
                reference,
                descr,
            } => {
                let mut text = String::new();
                for x in descr {
                    text.push_str(&x);
                    text.push(' ');
                }
                info!("add a task with description {}", text);
                let new_id = db::add_task(&dbfile, &text)
                    .with_context(|_| format!("Failed to create the new task {}", text))?;
                if !labels.is_empty() {
                    debug!("set labels");
                    db::add_labels(&dbfile, new_id, &labels).with_context(|_| {
                        format!(
                            "Failed to create attach labels {:?} to new task {}",
                            &labels, new_id
                        )
                    })?;
                }
                if let Some(priority_str) = priority {
                    debug!("set priority {}", priority_str);
                    db::set_priority(&dbfile, new_id, &priority_str).with_context(|_| {
                        format!(
                            "Failed to set priority {} to the new task {}",
                            priority_str, new_id
                        )
                    })?;
                }
                if let Some(storypoint_u32) = storypoint {
                    debug!("set storypoint {}", storypoint_u32);
                    db::set_storypoint(&dbfile, new_id, storypoint_u32).with_context(|_| {
                        format!(
                            "Failed to set storypoints {} to the new task {}",
                            storypoint_u32, new_id
                        )
                    })?;
                }
                if let Some(ref_str) = reference {
                    debug!("set reference {}", ref_str);
                    db::set_reference(&dbfile, new_id, &ref_str).with_context(|_| {
                        format!(
                            "Failed to set reference {} to the new task {}",
                            ref_str, new_id
                        )
                    })?;
                }
                println!("Create a new task, with id {}", new_id);
            }
            TaskCmd::AddLabel { labels, task } => {
                if labels.is_empty() {
                    error!("You have to specify at least one label");
                } else {
                    db::add_labels(&dbfile, task, &labels).with_context(|_| {
                        format!("Failed to add labels {:?} to the task {}", labels, task)
                    })?;
                }
            }
            TaskCmd::Edit {
                task,
                priority,
                reference,
                status,
                storypoint,
                descr,
            } => {
                let mut text = String::new();
                if priority.is_none()
                    && status.is_none()
                    && descr.is_empty()
                    && storypoint.is_none()
                    && reference.is_none()
                {
                    error!("You have to specify at least on attribute you want to edit");
                } else {
                    if !descr.is_empty() {
                        for x in descr {
                            text.push_str(&x);
                            text.push(' ');
                            info!("edit the task with a new description {}", text);
                            db::set_descr(&dbfile, task, &text).with_context(|_| {
                                format!("Failed to edit task {} with description {}", task, text)
                            })?;
                        }
                    }
                    if let Some(priority) = priority {
                        db::set_priority(&dbfile, task, &priority).with_context(|_| {
                            format!("Failed to edit task {} with priority {}", task, priority)
                        })?;
                    }
                    if let Some(storypoint_u32) = storypoint {
                        debug!("set storypoint {}", storypoint_u32);
                        db::set_storypoint(&dbfile, task, storypoint_u32).with_context(|_| {
                            format!(
                                "Failed to edit task {} with storypoints {}",
                                task, storypoint_u32
                            )
                        })?;
                    }
                    if let Some(ref_str) = reference {
                        debug!("set reference {}", ref_str);
                        db::set_reference(&dbfile, task, &ref_str).with_context(|_| {
                            format!("Failed to edit task {} with reference {}", task, ref_str)
                        })?;
                    }
                    if let Some(status) = status {
                        if status == "done" {
                            warn!("To make a task as done, please use the command task-done");
                            db::complete_task(&dbfile, task).with_context(|_| {
                                format!("Failed to edit task {} with status done", task)
                            })?;
                        }
                        db::set_status(&dbfile, task, &status).with_context(|_| {
                            format!("Failed to edit task {} with status {}", task, status)
                        })?;
                    }
                }
            }
            TaskCmd::Start(task) => {
                info!("Start task {}", task.task_id);
                db::set_status(&dbfile, task.task_id, "in_progress")
                    .with_context(|_| format!("Failed to start task {}", task.task_id))?;
            }
            TaskCmd::Block(task) => {
                info!("Block task {}", task.task_id);
                db::set_status(&dbfile, task.task_id, "block")
                    .with_context(|_| format!("Failed to block task {}", task.task_id))?;
            }
            TaskCmd::Done(task) => {
                info!("Completed task {}", task.task_id);
                db::complete_task(&dbfile, task.task_id)?;
                db::set_status(&dbfile, task.task_id, "done")
                    .with_context(|_| format!("Failed to complete task {}", task.task_id))?;
            }
            TaskCmd::Delete(task) => {
                info!("Delete task {}", task.task_id);
                db::delete_task(&dbfile, task.task_id)
                    .with_context(|_| format!("Failed to delete task {}", task.task_id))?;
            }
            TaskCmd::Prio(task) => {
                info!("Increase priority of task {}", task.task_id);
                db::increase_priority(&dbfile, task.task_id).with_context(|_| {
                    format!("Faile to increase priority of task {}", task.task_id)
                })?;
            }
        },
        Cmd::Show(mut showopt) => {
            debug!("show: showopt => {:?}", showopt);
            if let Some(task_id) = showopt.task {
                // show only one task
                let tasks = db::get_open_tasks(&dbfile)?;
                let task: Vec<_> = tasks.iter().filter(|x| x.id == task_id).cloned().collect();
                if task.is_empty() {
                } else {
                    task::show(&dbfile, &task, &showopt.show_opts.labels, "", true, true);
                }
            } else {
                let cmd = match showopt.cmd {
                    Some(x) => x,
                    None => ShowCmd::All {
                        show_opts: Default::default(),
                    },
                };
                match cmd {
                    ShowCmd::All { mut show_opts } => {
                        let tasks = db::get_open_tasks(&dbfile)?;
                        showopt.show_opts.merge(&mut show_opts);
                        task::show(
                            &dbfile,
                            &tasks,
                            &showopt.show_opts.labels,
                            "",
                            showopt.show_opts.reference,
                            showopt.show_opts.hidden,
                        );
                    }
                    ShowCmd::Short { mut show_opts } => {
                        let tasks = db::get_open_tasks(&dbfile)?;
                        showopt.show_opts.merge(&mut show_opts);
                        task::show_short(
                            &dbfile,
                            &tasks,
                            &showopt.show_opts.labels,
                            showopt.show_opts.reference,
                            showopt.show_opts.hidden,
                        );
                    }
                    ShowCmd::Backlog { mut show_opts } => {
                        let tasks = db::get_open_tasks(&dbfile)?;
                        showopt.show_opts.merge(&mut show_opts);
                        task::show(
                            &dbfile,
                            &tasks,
                            &showopt.show_opts.labels,
                            "todo",
                            showopt.show_opts.reference,
                            showopt.show_opts.hidden,
                        );
                    }
                    ShowCmd::Work { mut show_opts } => {
                        let tasks = db::get_open_tasks(&dbfile)?;
                        showopt.show_opts.merge(&mut show_opts);
                        task::show(
                            &dbfile,
                            &tasks,
                            &showopt.show_opts.labels,
                            "in_progress",
                            showopt.show_opts.reference,
                            showopt.show_opts.hidden,
                        );
                    }
                    ShowCmd::Done {
                        mut show_opts,
                        time_window,
                    } => {
                        let tasks = db::get_done_tasks(&dbfile)?;
                        showopt.show_opts.merge(&mut show_opts);
                        task::show_done(
                            &dbfile,
                            &tasks,
                            &showopt.show_opts.labels,
                            &time_window.unwrap_or(TimeWindow::Today),
                            showopt.show_opts.reference,
                            showopt.show_opts.hidden,
                        );
                    }
                }
            }
        }
    };
    trace!("myrello end");
    Ok(())
}
