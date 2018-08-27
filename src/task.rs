use super::db;
use prettytable::cell::Cell;
use prettytable::{Attr, Table};
use std::path::Path;

#[derive(Debug)]
pub struct Task {
    pub id: u32,
    pub descr: String,
    pub priority: String,
    pub status: String,
    pub storypoints: u32,
}

//#[derive(Debug)]
//pub struct TaskLabel {
//    pub id: u32,
//    pub descr: String,
//    pub labels: Vec<String>,
//}

pub fn show_tasks(filename: &Path, tasks: &[Task], reference: bool) {
    let mut table = Table::new();
    if reference {
        table.set_titles(
            row![b => "Id", "Priority", "Status", "Labels", "Description", "Reference"],
        );
    } else {
        table.set_titles(row![b => "Id", "Priority", "Status", "Labels", "Description"]);
    }
    for t in tasks {
        let labels: Vec<String> = db::get_labels(filename, t.id).unwrap_or_default();
        let label_str = if labels.is_empty() {
            trace!("No labels for task {}", t.id);
            String::new()
        } else {
            let mut label_str = String::new();
            for l in labels {
                label_str.push_str(&l);
                label_str.push('\n');
            }
            label_str
        };
        if reference {
            let reference_str = db::get_refs(filename, t.id).unwrap_or_default();
            table.add_row(
                row![ b -> &t.id.to_string(), &t.priority, &t.status, &label_str, &t.descr, &reference_str
        ],
            );
        } else {
            table.add_row(
                row![ b -> &t.id.to_string(), &t.priority, &t.status, &label_str, &t.descr ],
            );
        }
    }
    table.printstd();
}

#[allow(ptr_arg)]
pub fn show_tasks_label(filename: &Path, tasks: &[Task], label: &String, reference: bool) {
    let mut table = Table::new();
    if reference {
        table.set_titles(
            row![b => "Id", "Priority", "Status", "Labels", "Description", "Reference"],
        );
    } else {
        table.set_titles(row![b => "Id", "Priority", "Status", "Labels", "Description"]);
    }
    for t in tasks {
        let labels: Vec<String> = db::get_labels(filename, t.id).unwrap_or_default();
        if labels.contains(label) {
            let mut label_str = String::new();
            for l in labels {
                label_str.push_str(&l);
                label_str.push('\n');
            }
            if reference {
                let reference_str = db::get_refs(filename, t.id).unwrap_or_default();
                table.add_row(
                    row![ b -> &t.id.to_string(), &t.priority, &t.status, &label_str, &t.descr, &reference_str
        ],
                );
            } else {
                table.add_row(
                    row![ b -> &t.id.to_string(), &t.priority, &t.status, &label_str, &t.descr ],
                );
            }
        }
    }
    table.printstd();
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

#[allow(ptr_arg)]
pub fn show(filename: &Path, tasks: &[Task], label: &[String], reference: bool, storypoints: bool) {
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
                row![ b -> &t.id.to_string(), &t.priority, &t.status, &label_str, &t.descr];
            if storypoints {
                row.add_cell(Cell::new(&t.storypoints.to_string()));
            }
            if reference {
                let reference_str = db::get_refs(filename, t.id).unwrap_or_default();
                row.add_cell(Cell::new(&reference_str));
            }
            table.add_row(row);
        }
    }
    table.printstd();
}
