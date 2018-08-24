use super::db;
use prettytable::Table;
use std::path::Path;

#[derive(Debug)]
pub struct Task {
    pub id: u32,
    pub descr: String,
    pub priority: String,
    pub status: String,
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
