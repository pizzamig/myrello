#![cfg_attr(feature = "nightly", feature(tool_lints))]
#[allow(unused_imports)]
use exitfailure::ExitFailure;
use failure::ResultExt;
use log::{debug, error, info, trace, warn};
use myrello::cli_opt::{Cmd, DbCmd, TaskCmd};
use myrello::cli_opt::{ShowCmd, ShowOpt};
use myrello::db;
use myrello::task;
use myrello::task::TimeWindow;
use std::path::PathBuf;
use structopt::clap::Shell;
use structopt::StructOpt;
use structopt_flags::LogLevel;

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

fn dbfile_default() -> Result<std::path::PathBuf, ExitFailure> {
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
    Ok(default_dir.join("myrello.db"))
}

fn cmd_dbinit(dbfile: std::path::PathBuf, force: bool) -> Result<(), ExitFailure> {
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
    Ok(())
}

fn cmd_show(mut showopt: ShowOpt, dbfile: &std::path::Path) -> Result<(), ExitFailure> {
    debug!("show: showopt => {:?}", showopt);
    if let Some(task_id) = showopt.task {
        task::show1task(dbfile, task_id);
    } else {
        let cmd = showopt.cmd.unwrap_or_else(|| ShowCmd::All {
            show_opts: Default::default(),
        });
        match cmd {
            ShowCmd::All { show_opts } => {
                let tasks = db::get_open_tasks(dbfile)?;
                showopt.show_opts.merge(&show_opts);
                task::show2(dbfile, &tasks, showopt.show_opts.as_show_params(""));
            }
            ShowCmd::Short { show_opts } => {
                let tasks = db::get_open_tasks(dbfile)?;
                showopt.show_opts.merge(&show_opts);
                task::show_short(
                    dbfile,
                    &tasks,
                    &showopt.show_opts.labels,
                    showopt.show_opts.reference,
                    showopt.show_opts.hidden,
                );
            }
            ShowCmd::Backlog { show_opts } => {
                let tasks = db::get_open_tasks(dbfile)?;
                showopt.show_opts.merge(&show_opts);
                task::show2(dbfile, &tasks, showopt.show_opts.as_show_params("todo"));
            }
            ShowCmd::Work { show_opts } => {
                let tasks = db::get_open_tasks(dbfile)?;
                showopt.show_opts.merge(&show_opts);
                task::show2(
                    dbfile,
                    &tasks,
                    showopt.show_opts.as_show_params("in_progress"),
                );
            }
            ShowCmd::Done {
                show_opts,
                time_window,
            } => {
                let tasks = db::get_done_tasks(dbfile)?;
                showopt.show_opts.merge(&show_opts);
                task::show_done(
                    dbfile,
                    &tasks,
                    &showopt.show_opts.labels,
                    &time_window.unwrap_or(TimeWindow::Today),
                    showopt.show_opts.reference,
                    showopt.show_opts.hidden,
                );
            }
        }
    }
    Ok(())
}

fn cmd_task_new(new_task: TaskCmd, dbfile: &std::path::Path) -> Result<(), ExitFailure> {
    if let TaskCmd::New {
        labels,
        priority,
        storypoint,
        reference,
        descr,
    } = new_task
    {
        let text = descr_to_string(&descr);
        info!("add a task with description {}", text);
        let new_id = db::add_task(dbfile, &text)
            .with_context(|_| format!("Failed to create the new task {}", text))?;
        if !labels.is_empty() {
            debug!("set labels");
            db::add_labels(dbfile, new_id, &labels).with_context(|_| {
                format!(
                    "Failed to create attach labels {:?} to new task {}",
                    &labels, new_id
                )
            })?;
        }
        if let Some(priority_str) = priority {
            debug!("set priority {}", priority_str);
            db::set_priority(dbfile, new_id, &priority_str).with_context(|_| {
                format!(
                    "Failed to set priority {} to the new task {}",
                    priority_str, new_id
                )
            })?;
        }
        if let Some(storypoint_u32) = storypoint {
            debug!("set storypoint {}", storypoint_u32);
            db::set_storypoint(dbfile, new_id, storypoint_u32).with_context(|_| {
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
    Ok(())
}

fn cmd_task_edit(edit_task: TaskCmd, dbfile: &std::path::Path) -> Result<(), ExitFailure> {
    if let TaskCmd::Edit {
        task,
        priority,
        reference,
        status,
        storypoint,
        descr,
    } = edit_task
    {
        if priority.is_none()
            && status.is_none()
            && descr.is_empty()
            && storypoint.is_none()
            && reference.is_none()
        {
            error!("You have to specify at least on attribute you want to edit");
        } else {
            if !descr.is_empty() {
                let text = descr_to_string(&descr);
                info!("edit the task with a new description {}", text);
                db::set_descr(&dbfile, task, &text).with_context(|_| {
                    format!("Failed to edit task {} with description {}", task, text)
                })?;
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

    Ok(())
}
fn descr_to_string(descr: &[String]) -> String {
    let mut rv = String::new();
    for x in descr {
        rv.push_str(&x);
        rv.push(' ');
    }
    rv
}

fn main() -> Result<(), ExitFailure> {
    let opt = Opt::from_args();
    opt.verbose.set_log_level();
    trace!("myrello begin");
    trace!("opt => {:?}", opt);
    let dbfile = match opt.dbfile {
        Some(x) => x,
        None => dbfile_default()?,
    };
    trace!("Using {:?} as database", dbfile);
    match opt.cmd {
        Cmd::Completion => {
            Opt::clap().gen_completions_to("myrello", Shell::Zsh, &mut std::io::stdout());
        }
        Cmd::Db(dbcmd) => match dbcmd.cmd {
            DbCmd::Init { force_flag } => cmd_dbinit(dbfile, force_flag.force)?,
        },
        Cmd::Task(taskcmd) => match taskcmd.cmd {
            TaskCmd::New {
                labels,
                priority,
                storypoint,
                reference,
                descr,
            } => {
                cmd_task_new(
                    TaskCmd::New {
                        labels,
                        priority,
                        storypoint,
                        reference,
                        descr,
                    },
                    &dbfile,
                )?;
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
                cmd_task_edit(
                    TaskCmd::Edit {
                        task,
                        priority,
                        reference,
                        status,
                        storypoint,
                        descr,
                    },
                    &dbfile,
                )?;
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
        Cmd::Show(showopt) => {
            cmd_show(showopt, &dbfile)?;
        }
    };
    trace!("myrello end");
    Ok(())
}
