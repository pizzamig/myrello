use super::db;
use prettytable::cell::Cell;
use prettytable::{Attr, Table};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub struct Task {
    pub id: u32,
    pub descr: String,
    pub priority: String,
    pub status: String,
    pub storypoints: u32,
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
    } else {
        let mut stattable = Table::new();
        for st in stats.keys() {
            let num = stats.get(st).unwrap_or(&0).to_string();
            let mut row = row![ b -> "status", &st, b -> "tasks", num];
            //println!("status: {}\ttasks: {}", st, stats.get(st).unwrap_or(&0));
            stattable.add_row(row);
        }
        stattable.printstd();
    }
}
