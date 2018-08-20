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

pub fn show_tasks(filename: &Path, tasks: &[Task]) {
    let mut table = Table::new();
    table.set_titles(row![b => "Id", "Priority", "Status", "Labels", "Description"]);
    for t in tasks {
        let labels: Vec<String> = db::get_labels(filename, t.id).unwrap_or_default();
        if labels.is_empty() {
            trace!("No labels for task {}", t.id);
            table.add_row(row![
                b -> &t.id.to_string(),
                &t.priority,
                &t.status,
                "",
                &t.descr
            ]);
        } else {
            let mut label_str = String::new();
            for l in labels {
                label_str.push_str(&l);
                label_str.push('\n');
            }
            table.add_row(row![
                b -> &t.id.to_string(),
                &t.priority,
                &t.status,
                &label_str,
                &t.descr
            ]);
        }
    }
    table.printstd();
}

pub fn show_tasks_label(filename: &Path, tasks: &[Task], label: &String) {
    let mut table = Table::new();
    table.set_titles(row![b => "Id", "Priority", "Status", "Labels", "Description"]);
    for t in tasks {
        let labels: Vec<String> = db::get_labels(filename, t.id).unwrap_or_default();
        if labels.contains(label) {
            let mut label_str = String::new();
            for l in labels {
                label_str.push_str(&l);
                label_str.push('\n');
            }
            table.add_row(row![
                b -> &t.id.to_string(),
                &t.priority,
                &t.status,
                &label_str,
                &t.descr
            ]);
        }
    }
    table.printstd();
}
