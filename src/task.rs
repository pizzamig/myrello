use super::db;
use chrono::prelude::*;
use chrono::Duration;
use log::trace;
use prettytable::cell::Cell;
use prettytable::{cell, row, Attr, Table};
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

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

#[derive(Debug)]
pub enum TimeWindow {
    Today,
    Yesterday,
    Week,
    Month,
}

#[derive(Debug)]
pub struct TimeWindowParseError {
    pub text: String,
}

impl FromStr for TimeWindow {
    type Err = TimeWindowParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "today" => Ok(TimeWindow::Today),
            "yesterday" => Ok(TimeWindow::Yesterday),
            "week" => Ok(TimeWindow::Week),
            "month" => Ok(TimeWindow::Month),
            _ => Err(TimeWindowParseError {
                text: s.to_string(),
            }),
        }
    }
}

impl ToString for TimeWindowParseError {
    fn to_string(&self) -> String {
        format!("{} not a valid time window", self.text)
    }
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
#[cfg_attr(feature = "nightly", allow(clippy::ptr_arg))]
pub fn show(
    filename: &Path,
    tasks: &[Task],
    label: &[String],
    status: &str,
    reference: bool,
    storypoints: bool,
) {
    let mut stats = HashMap::new();
    let mut table = Table::new();
    let mut title = row![b => "Id", "Priority", "Status", "Labels", "Description"];
    if storypoints {
        title.add_cell(Cell::new("Story points").with_style(Attr::Bold));
    }
    if reference {
        title.add_cell(Cell::new("Reference").with_style(Attr::Bold));;
    }
    table.set_titles(title);
    for t in tasks {
        let task_labels: Vec<String> = db::get_labels(filename, t.id).unwrap_or_default();
        if check_label(label, &task_labels) && (status == "" || t.status == status) {
            let label_str = if task_labels.is_empty() {
                trace!("No labels for task {}", t.id);
                String::new()
            } else {
                let mut label_str = String::new();
                for l in task_labels {
                    label_str.push_str(&l);
                    label_str.push('\n');
                }
                label_str
            };
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
            let counter = stats.entry(t.status.as_str()).or_insert(0);
            *counter += 1;
        }
    }
    table.printstd();
    if status != "" {
        println!("tasks: {}", stats.get(status).unwrap_or(&0));
    } else if tasks.len() != 1 {
        let mut stattable = Table::new();
        for st in stats.keys() {
            let num = stats.get(st).unwrap_or(&0).to_string();
            let row = row![ b -> "status", &st, b -> "tasks", num];
            stattable.add_row(row);
        }
        stattable.printstd();
    }
}

#[cfg_attr(feature = "nightly", allow(clippy::ptr_arg))]
pub fn show_short(
    filename: &Path,
    tasks: &[Task],
    label: &[String],
    reference: bool,
    storypoints: bool,
) {
    let mut stats = HashMap::new();
    let mut table = Table::new();
    let mut title = row![b => "Id", "Priority", "Status", "Labels", "Description"];
    if storypoints {
        title.add_cell(Cell::new("Story points").with_style(Attr::Bold));
    }
    if reference {
        title.add_cell(Cell::new("Reference").with_style(Attr::Bold));;
    }
    table.set_titles(title);
    for t in tasks {
        let task_labels: Vec<String> = db::get_labels(filename, t.id).unwrap_or_default();
        if check_label(label, &task_labels)
            && (t.status == "in_progress" || t.priority == "high" || t.priority == "urgent")
        {
            let label_str = if task_labels.is_empty() {
                trace!("No labels for task {}", t.id);
                String::new()
            } else {
                let mut label_str = String::new();
                for l in task_labels {
                    label_str.push_str(&l);
                    label_str.push('\n');
                }
                label_str
            };
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
            let counter = stats.entry(t.status.as_str()).or_insert(0);
            *counter += 1;
        }
    }
    table.printstd();
    let mut stattable = Table::new();
    for st in stats.keys() {
        let num = stats.get(st).unwrap_or(&0).to_string();
        let row = row![ b -> "status", &st, b -> "tasks", num];
        //println!("status: {}\ttasks: {}", st, stats.get(st).unwrap_or(&0));
        stattable.add_row(row);
    }
    stattable.printstd();
}

#[cfg_attr(feature = "nightly", allow(clippy::ptr_arg))]
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
    let mut title = row![b => "Id", "Labels", "Description", "Completed"];
    if storypoints {
        title.add_cell(Cell::new("Story points").with_style(Attr::Bold));
    }
    if reference {
        title.add_cell(Cell::new("Reference").with_style(Attr::Bold));;
    }
    table.set_titles(title);
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
                let label_str = if task_labels.is_empty() {
                    trace!("No labels for task {}", t.id);
                    String::new()
                } else {
                    let mut label_str = String::new();
                    for l in task_labels {
                        label_str.push_str(&l);
                        label_str.push('\n');
                    }
                    label_str
                };
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
