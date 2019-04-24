use super::db;
use chrono::prelude::*;
use chrono::Duration;
use prettytable::cell::Cell;
use prettytable::{cell, row, Attr, Table};
use std::collections::HashMap;
use std::path::Path;
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone)]
pub struct Task {
    pub id: u32,
    pub descr: String,
    pub priority: String,
    pub status: String,
    pub storypoints: u32,
}

#[derive(Debug)]
pub struct TaskDone {
    pub id: u32,
    pub descr: String,
    pub completion_date: String,
    pub storypoints: u32,
}

#[derive(Debug, EnumString, Display)]
pub enum TimeWindow {
    #[strum(serialize = "today")]
    Today,
    #[strum(serialize = "yesterday")]
    Yesterday,
    #[strum(serialize = "week")]
    Week,
    #[strum(serialize = "month")]
    Month,
}

fn check_label(labels: &[String], task_labels: &[String]) -> bool {
    if labels.is_empty() {
        return true;
    }
    for l in labels {
        if !task_labels.contains(l) {
            return false;
        }
    }
    true
}

fn label_to_str(labels: &[String]) -> String {
    let mut rv = String::new();
    if !labels.is_empty() {
        for l in labels {
            rv.push_str(l);
            rv.push('\n');
        }
    }
    rv
}

#[derive(Debug, Default)]
pub struct ShowParams<'a> {
    pub label: &'a [String],
    pub reference: bool,
    pub status: &'a str,
    pub storypoints: bool,
}

fn show_stats(stats: &HashMap<&str, u64>) {
    let mut stattable = Table::new();
    for st in stats.keys() {
        let num = stats.get(st).unwrap_or(&0).to_string();
        let row = row![ b -> "status", &st, b -> "tasks", num];
        stattable.add_row(row);
    }
    stattable.printstd();
}

fn set_title(table: &mut Table, param: &ShowParams) {
    let mut title = row![b => "Id", "Priority", "Status", "Labels", "Description"];
    if param.storypoints {
        title.add_cell(Cell::new("Story points").with_style(Attr::Bold));
    }
    if param.reference {
        title.add_cell(Cell::new("Reference").with_style(Attr::Bold));;
    }
    table.set_titles(title);
}

pub fn show2(filename: &Path, tasks: &[Task], param: ShowParams) {
    let mut stats = HashMap::new();
    let mut table = Table::new();
    set_title(&mut table, &param);
    for t in tasks {
        let task_labels: Vec<String> = db::get_labels(filename, t.id).unwrap_or_default();
        if check_label(param.label, &task_labels)
            && (param.status == "" || t.status == param.status)
        {
            let mut row = row![ b -> &t.id.to_string(), &t.priority, &t.status, &label_to_str(&task_labels), &t.descr];
            if param.storypoints {
                row.add_cell(Cell::new(&t.storypoints.to_string()));
            }
            if param.reference {
                let reference_str = db::get_refs(filename, t.id).unwrap_or_default();
                row.add_cell(Cell::new(&reference_str));
            }
            table.add_row(row);
            let counter = stats.entry(t.status.as_str()).or_insert(0u64);
            *counter += 1;
        }
    }
    table.printstd();
    if param.status != "" {
        println!("tasks: {}", stats.get(param.status).unwrap_or(&0));
    } else if tasks.len() != 1 {
        show_stats(&stats);
    }
}

pub fn show1task(filename: &Path, task_id: u32) {
    let tasks = db::get_open_tasks(&filename).unwrap_or_default();
    let task: Vec<_> = tasks.iter().filter(|x| x.id == task_id).cloned().collect();
    if !task.is_empty() {
        show2(
            &filename,
            &task,
            ShowParams {
                label: &[],
                status: "",
                reference: true,
                storypoints: true,
            },
        );
    }
}

pub fn show_short(
    filename: &Path,
    tasks: &[Task],
    label: &[String],
    reference: bool,
    storypoints: bool,
) {
    let mut stats = HashMap::new();
    let mut table = Table::new();
    set_title(
        &mut table,
        &ShowParams {
            label,
            status: "",
            reference,
            storypoints,
        },
    );
    for t in tasks {
        let task_labels: Vec<String> = db::get_labels(filename, t.id).unwrap_or_default();
        if check_label(label, &task_labels)
            && (t.status == "in_progress" || t.priority == "high" || t.priority == "urgent")
        {
            let label_str = label_to_str(&task_labels);
            let mut row =
                row![ b -> &t.id.to_string(), &t.priority, &t.status, &label_str, &t.descr];
            if storypoints {
                row.add_cell(Cell::new(&t.storypoints.to_string()));
            }
            if reference {
                let reference_str = db::get_refs(filename, t.id).unwrap_or_default();
                row.add_cell(Cell::new(&reference_str));
            }
            table.add_row(row);
            let counter = stats.entry(t.status.as_str()).or_insert(0u64);
            *counter += 1;
        }
    }
    table.printstd();
    if tasks.len() != 1 {
        show_stats(&stats);
    }
}

pub fn show_done(
    filename: &Path,
    tasks: &[TaskDone],
    label: &[String],
    timewindow: &TimeWindow,
    reference: bool,
    storypoints: bool,
) {
    let mut stats = HashMap::new();
    let mut table = Table::new();
    set_title(
        &mut table,
        &ShowParams {
            label,
            status: "",
            reference,
            storypoints,
        },
    );
    for t in tasks {
        let now: DateTime<Utc> = Utc::now();
        let completed: NaiveDateTime =
            NaiveDateTime::parse_from_str(t.completion_date.as_str(), "%Y-%m-%d %H:%M:%S").unwrap();
        let duration = now.naive_utc() - completed;
        let max_duration = match timewindow {
            TimeWindow::Today => Duration::days(1),
            TimeWindow::Yesterday => Duration::days(2),
            TimeWindow::Week => Duration::weeks(1),
            TimeWindow::Month => Duration::weeks(4),
        };
        if duration < max_duration {
            let task_labels: Vec<String> = db::get_labels(filename, t.id).unwrap_or_default();
            if check_label(label, &task_labels) {
                let label_str = label_to_str(&task_labels);
                let mut row =
                    row![ b -> &t.id.to_string(), &label_str, &t.descr, &t.completion_date];
                if storypoints {
                    row.add_cell(Cell::new(&t.storypoints.to_string()));
                }
                if reference {
                    let reference_str = db::get_refs(filename, t.id).unwrap_or_default();
                    row.add_cell(Cell::new(&reference_str));
                }
                table.add_row(row);
                let counter = stats.entry("done").or_insert(0);
                *counter += 1;
            }
        }
    }
    table.printstd();
    println!("tasks: {}", stats.get("done").unwrap_or(&0));
}
